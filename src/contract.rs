use soroban_sdk::{contract, contractimpl, token, Address, Env};

use crate::errors::StreamError;
use crate::events;
use crate::storage::{load_stream, save_stream, stream_exists};
use crate::types::{Stream, StreamStatus};

#[contract]
pub struct StreamFundContract;

#[contractimpl]
impl StreamFundContract {
    /// Create a continuous token stream from `sender` to `receiver`.
    ///
    /// Escrows `amount` tokens into the contract for the duration of the stream.
    /// Tokens are released linearly between `start_time` and `stop_time` based
    /// on elapsed ledger time.
    ///
    /// # Authorization
    /// Requires `sender` authorization.
    ///
    /// # Errors
    /// - `InvalidAmount`     — `amount <= 0`
    /// - `InvalidTimeRange`  — `start_time >= stop_time`
    /// - `StopTimeInPast`    — `stop_time <= current ledger timestamp`
    /// - `SelfStream`        — `sender == receiver`
    /// - `DuplicateStreamId` — a stream with `stream_id` already exists
    /// - `TransferFailed`    — token transfer from sender to contract failed
    pub fn create_stream(
        e: Env,
        stream_id: u64,
        sender: Address,
        receiver: Address,
        token: Address,
        amount: i128,
        start_time: u64,
        stop_time: u64,
    ) -> Result<(), StreamError> {
        // ── 1. Authorization ────────────────────────────────────────────────
        sender.require_auth();

        // ── 2. Input validation ─────────────────────────────────────────────
        if amount <= 0 {
            return Err(StreamError::InvalidAmount);
        }
        if start_time >= stop_time {
            return Err(StreamError::InvalidTimeRange);
        }
        let current_time = e.ledger().timestamp();
        if stop_time <= current_time {
            return Err(StreamError::StopTimeInPast);
        }
        if sender == receiver {
            return Err(StreamError::SelfStream);
        }
        if stream_exists(&e, stream_id) {
            return Err(StreamError::DuplicateStreamId);
        }

        // ── 3. Escrow tokens ────────────────────────────────────────────────
        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&sender, &e.current_contract_address(), &amount);

        // ── 4. Persist stream record ─────────────────────────────────────────
        // Clone `token` before moving it into the struct so we can reference
        // it in the event emission below.
        let token_for_event = token.clone();
        let stream = Stream {
            sender: sender.clone(),
            receiver: receiver.clone(),
            token,
            total_amount: amount,
            start_time,
            stop_time,
            withdrawn_amount: 0,
            status: StreamStatus::Active,
            receiver_retained: 0,
        };
        save_stream(&e, stream_id, &stream);

        // ── 5. Emit event ────────────────────────────────────────────────────
        events::stream_created(
            &e,
            stream_id,
            &sender,
            &receiver,
            &token_for_event,
            amount,
            start_time,
            stop_time,
        );

        Ok(())
    }

    /// Return the amount the receiver can withdraw from `stream_id` right now.
    ///
    /// This is a read-only view — it writes nothing to storage.
    ///
    /// For a cancelled stream the value is the retained balance captured at
    /// cancellation time, not a time-based recalculation.
    ///
    /// # Errors
    /// - `StreamNotFound` — no stream with `stream_id` exists
    pub fn withdrawable_balance(e: Env, stream_id: u64) -> Result<i128, StreamError> {
        let stream = load_stream(&e, stream_id)?;

        // Cancelled streams: return the pre-computed retained balance.
        if stream.status == StreamStatus::Cancelled {
            return Ok(stream.receiver_retained);
        }

        let current_time = e.ledger().timestamp();

        // Stream has not started yet.
        if current_time <= stream.start_time {
            return Ok(0);
        }

        let elapsed = current_time
            .min(stream.stop_time)
            .saturating_sub(stream.start_time) as i128;
        let duration = (stream.stop_time - stream.start_time) as i128;

        // Integer floor division — dust is retained in escrow until stop_time.
        let total_streamed = stream.total_amount.saturating_mul(elapsed) / duration;

        Ok(total_streamed - stream.withdrawn_amount)
    }

    /// Withdraw `amount` tokens from `stream_id` to the receiver.
    ///
    /// Follows the Checks-Effects-Interactions pattern: storage is updated
    /// before the outbound token transfer.
    ///
    /// # Authorization
    /// Requires `receiver` authorization.
    ///
    /// # Errors
    /// - `StreamNotFound`       — no stream with `stream_id` exists
    /// - `InvalidAmount`        — `amount <= 0`
    /// - `InsufficientBalance`  — `amount > withdrawable_balance`
    /// - `TransferFailed`       — token transfer to receiver failed
    pub fn withdraw(e: Env, stream_id: u64, amount: i128) -> Result<(), StreamError> {
        // ── 1. Load stream ───────────────────────────────────────────────────
        let mut stream = load_stream(&e, stream_id)?;

        // ── 2. Authorization ─────────────────────────────────────────────────
        stream.receiver.require_auth();

        // ── 3. Checks ────────────────────────────────────────────────────────
        if amount <= 0 {
            return Err(StreamError::InvalidAmount);
        }

        let available = Self::withdrawable_balance(e.clone(), stream_id)?;

        if amount > available {
            return Err(StreamError::InsufficientBalance);
        }

        // ── 4. Effects (state update before transfer — CEI) ──────────────────
        stream.withdrawn_amount += amount;
        if stream.status == StreamStatus::Cancelled {
            stream.receiver_retained -= amount;
        }
        let receiver = stream.receiver.clone();
        let token_addr = stream.token.clone();
        save_stream(&e, stream_id, &stream);

        // ── 5. Interaction (outbound transfer) ───────────────────────────────
        let token_client = token::Client::new(&e, &token_addr);
        token_client.transfer(&e.current_contract_address(), &receiver, &amount);

        // ── 6. Emit event ────────────────────────────────────────────────────
        events::stream_withdrawn(&e, stream_id, &receiver, amount);

        Ok(())
    }

    /// Cancel an active stream.
    ///
    /// Returns unstreamed tokens to `sender` immediately. The receiver's
    /// accrued-but-unwithdrawn balance is retained in escrow and remains
    /// claimable via `withdraw`.
    ///
    /// # Authorization
    /// Requires `sender` authorization.
    ///
    /// # Errors
    /// - `StreamNotFound`   — no stream with `stream_id` exists
    /// - `StreamNotActive`  — stream is already Cancelled or Completed
    /// - `TransferFailed`   — token refund to sender failed
    pub fn cancel_stream(e: Env, stream_id: u64) -> Result<(), StreamError> {
        // ── 1. Load stream ───────────────────────────────────────────────────
        let mut stream = load_stream(&e, stream_id)?;

        // ── 2. Status check ──────────────────────────────────────────────────
        // Treat streams past their stop_time as Completed — not cancellable.
        let current_time = e.ledger().timestamp();
        if stream.status == StreamStatus::Cancelled || current_time >= stream.stop_time {
            return Err(StreamError::StreamNotActive);
        }

        // ── 3. Authorization ─────────────────────────────────────────────────
        stream.sender.require_auth();

        // ── 4. Accounting ────────────────────────────────────────────────────
        let elapsed = current_time
            .saturating_sub(stream.start_time)
            .min(stream.stop_time - stream.start_time) as i128;
        let duration = (stream.stop_time - stream.start_time) as i128;

        let streamed = if current_time <= stream.start_time {
            0_i128
        } else {
            stream.total_amount.saturating_mul(elapsed) / duration
        };

        let receiver_payout = streamed - stream.withdrawn_amount;
        let sender_refund = stream.total_amount - streamed;

        // ── 5. Effects (state update before transfer — CEI) ──────────────────
        stream.status = StreamStatus::Cancelled;
        stream.receiver_retained = receiver_payout;
        let sender = stream.sender.clone();
        let token_addr = stream.token.clone();
        save_stream(&e, stream_id, &stream);

        // ── 6. Interaction: refund unstreamed tokens to sender ────────────────
        if sender_refund > 0 {
            let token_client = token::Client::new(&e, &token_addr);
            token_client.transfer(&e.current_contract_address(), &sender, &sender_refund);
        }

        // ── 7. Emit event ─────────────────────────────────────────────────────
        events::stream_cancelled(&e, stream_id, &sender, sender_refund, receiver_payout);

        Ok(())
    }
}
