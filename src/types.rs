use soroban_sdk::{contracttype, Address};

/// Current lifecycle state of a stream.
///
/// `Completed` is a derived state — it requires no storage write.
/// A stream is considered completed when `current_time >= stop_time`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Active,
    Cancelled,
}

/// Storage key namespace for persistent stream records.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Stream(u64),
}

/// A time-bounded linear token stream held in escrow by the contract.
///
/// Tokens are released linearly from `start_time` to `stop_time` based
/// on elapsed ledger time. The receiver may withdraw their accrued balance
/// at any point. The sender may cancel early, recovering unstreamed tokens
/// while leaving the receiver's accrued balance claimable.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    /// Address that created and funded the stream.
    pub sender: Address,
    /// Address that receives streamed tokens.
    pub receiver: Address,
    /// SEP-41 token contract address.
    pub token: Address,
    /// Total tokens escrowed at creation.
    pub total_amount: i128,
    /// Ledger timestamp at which streaming begins.
    pub start_time: u64,
    /// Ledger timestamp at which streaming ends.
    pub stop_time: u64,
    /// Cumulative tokens already withdrawn by the receiver.
    pub withdrawn_amount: i128,
    /// Current lifecycle status of the stream.
    pub status: StreamStatus,
    /// Receiver's claimable balance captured at cancellation time.
    /// Zero while the stream is Active.
    pub receiver_retained: i128,
}
