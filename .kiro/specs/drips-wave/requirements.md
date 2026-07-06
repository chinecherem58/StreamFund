# Requirements Document

## Introduction

Drips Wave is a Soroban smart contract on the Stellar network that enables continuous, second-by-second token streaming from funders to open-source maintainers. It extends the Drips Network's core dependency-funding architecture to the Stellar ecosystem, providing financial predictability for public goods contributors without requiring manual transfers.

**Actors:**
- **Funder (Sender)** — an organization or individual who creates and funds token streams
- **Maintainer (Receiver)** — an open-source contributor who receives streamed tokens over time
- **Platform Operator** — deploys and maintains the contract

## Glossary

| Term | Definition |
|------|-----------|
| Stream | A time-bounded escrow that releases tokens linearly to a receiver |
| Funder / Sender | The address that creates and finances a stream |
| Maintainer / Receiver | The address that receives streamed tokens |
| `stream_id` | A caller-supplied `u64` that uniquely identifies a stream |
| Escrow | Tokens locked inside the contract for the duration of a stream |
| Withdrawable Balance | The portion of streamed tokens not yet claimed by the receiver |
| Dust | Sub-unit remainders from integer division retained in the contract |
| TTL | Time-To-Live — the number of ledger-seconds a storage entry is kept alive |
| SEP-41 | Stellar Ecosystem Proposal defining the standard fungible token interface on Soroban |
| CEI | Checks-Effects-Interactions — the security pattern requiring state writes before external calls |

## Requirements

### REQ-1: Stream Creation

**User Story:** As a funder, I want to create a token stream to a maintainer so that tokens flow to them continuously over a defined period without manual intervention.

#### Acceptance Criteria

1. WHEN a funder calls `create_stream` with a unique `stream_id`, valid `sender`, `receiver`, `token`, `amount`, `start_time`, and `stop_time`, THEN the system SHALL escrow `amount` tokens from `sender` into the contract and persist the stream record.

2. THE system SHALL require the `sender` address to authorize the call via `require_auth()` before any state is written or tokens are moved.

3. IF a stream with the provided `stream_id` already exists in storage, THEN the system SHALL reject the call with a `DuplicateStreamId` error and leave all state unchanged.

4. IF `start_time >= stop_time`, THEN the system SHALL reject the call with an `InvalidTimeRange` error.

5. IF `stop_time <= current_ledger_timestamp` at the time of creation, THEN the system SHALL reject the call with a `StopTimeInPast` error.

6. IF `amount <= 0`, THEN the system SHALL reject the call with an `InvalidAmount` error.

7. IF `sender == receiver`, THEN the system SHALL reject the call with a `SelfStream` error.

8. IF the token transfer from `sender` to the contract fails (e.g., insufficient balance), THEN the system SHALL abort and return a `TransferFailed` error, leaving storage unchanged.

9. WHEN a stream is successfully created, THE system SHALL emit an event with fields: `stream_id`, `sender`, `receiver`, `token`, `amount`, `start_time`, `stop_time`.

---

### REQ-2: Withdrawable Balance Query

**User Story:** As a maintainer, I want to query how much I can withdraw from a stream at any moment so I can plan when to collect my accrued funds.

#### Acceptance Criteria

1. WHEN `withdrawable_balance` is called with a valid `stream_id`, THE system SHALL return:
   ```
   floor((min(current_time, stop_time) - start_time) / (stop_time - start_time) * total_amount) - withdrawn_amount
   ```
   clamped to a minimum of 0.

2. IF `current_time < start_time`, THEN the system SHALL return `0`.

3. IF `current_time >= stop_time`, THEN the system SHALL return `total_amount - withdrawn_amount`.

4. IF `stream_id` does not exist in storage, THEN the system SHALL return a `StreamNotFound` error.

5. IF the stream status is `Cancelled`, THEN the system SHALL return `receiver_retained` (the accrued-but-unwithdrawn balance captured at cancellation time), NOT recalculate from current ledger time.

6. THE system SHALL use integer floor division for all calculations. Any remainder (dust) is retained in the contract until `stop_time` is reached.

---

### REQ-3: Token Withdrawal

**User Story:** As a maintainer, I want to withdraw my accrued stream balance at any time so I can access tokens I have earned without waiting for the stream to end.

#### Acceptance Criteria

1. WHEN `withdraw` is called with a valid `stream_id` and `amount` between `1` and `withdrawable_balance` (inclusive), THE system SHALL require the `receiver` address to authorize the call via `require_auth()`.

2. THE system SHALL update `withdrawn_amount` in storage BEFORE executing the outbound token transfer to prevent reentrancy exploits (CEI pattern).

3. WHEN the state update and token transfer both succeed, THE system SHALL emit an event with fields: `stream_id`, `receiver`, `amount`.

4. IF `amount <= 0` or `amount > withdrawable_balance`, THEN the system SHALL reject the call with an `InsufficientBalance` error without modifying any state.

5. IF the token transfer from the contract to `receiver` fails, THEN the system SHALL revert the `withdrawn_amount` increment and return a `TransferFailed` error.

6. IF `stream_id` does not exist in storage, THEN the system SHALL return a `StreamNotFound` error.

7. Partial withdrawals SHALL be permitted; a receiver may call `withdraw` multiple times across the stream's lifetime.

---

### REQ-4: Stream Cancellation

**User Story:** As a funder, I want to cancel an active stream so I can recover unstreamed tokens if I no longer wish to fund a receiver.

#### Acceptance Criteria

1. WHEN `cancel_stream` is called, THE system SHALL require the `sender` address to authorize the call via `require_auth()`.

2. WHEN cancellation is authorized, THE system SHALL:
   - Calculate `streamed_amount = floor((min(current_time, stop_time) - start_time) / duration * total_amount)`
   - Calculate `receiver_payout = streamed_amount - withdrawn_amount` (retained for receiver to withdraw later)
   - Calculate `sender_refund = total_amount - streamed_amount`
   - Mark the stream as `Cancelled` in storage and record `receiver_retained = receiver_payout`
   - Transfer `sender_refund` back to `sender` (if > 0)
   - Retain `receiver_payout` in the contract for later withdrawal by `receiver`

3. AFTER cancellation, THE system SHALL permit the `receiver` to call `withdraw` to claim their retained `receiver_payout`.

4. AFTER cancellation, THE system SHALL reject further `cancel_stream` calls on the same `stream_id` with a `StreamNotActive` error.

5. IF `stream_id` does not exist in storage, THEN the system SHALL return a `StreamNotFound` error.

6. IF the sender refund transfer fails, THEN the system SHALL revert all state changes and return a `TransferFailed` error.

7. WHEN cancellation succeeds, THE system SHALL emit an event with fields: `stream_id`, `sender`, `sender_refund`, `receiver_payout`.

---

### REQ-5: Stream Lifecycle and State

**User Story:** As a platform operator, I want streams to follow well-defined states so the contract behaves predictably and is auditable.

#### Acceptance Criteria

1. THE `Stream` struct SHALL include a `status` field supporting values: `Active`, `Cancelled`, `Completed`.

2. A stream SHALL transition from `Active` to `Cancelled` only via `cancel_stream` by the `sender`.

3. A stream is considered `Completed` when `current_time >= stop_time`; no explicit transition call is required.

4. The system SHALL reject `cancel_stream` calls on streams in `Cancelled` or `Completed` state.

5. The system SHALL reject `withdraw` calls on `Cancelled` streams when `receiver_retained == 0`.

6. THE system SHALL extend the storage TTL of a stream to at least `stop_time + 31_536_000` ledger-seconds on creation and on each write interaction, preventing silent ledger eviction.

---

### REQ-6: Security and Authorization

**User Story:** As a protocol user, I want all sensitive operations to require correct authorization so that no third party can steal, redirect, or cancel streams they do not own.

#### Acceptance Criteria

1. `create_stream` and `cancel_stream` SHALL require `sender` authorization via `require_auth()`.

2. `withdraw` SHALL require `receiver` authorization via `require_auth()`.

3. No address other than `sender` or `receiver` SHALL be able to cancel or withdraw from a stream.

4. Authorization checks SHALL be the first operation in each mutating function, before any state reads or writes.

5. All state mutations SHALL be committed before outbound token transfers (CEI pattern).

6. THE system SHALL NOT expose any admin or superuser function that can unilaterally move user funds.

---

### REQ-7: Event Emission

**User Story:** As a frontend developer or indexer, I want the contract to emit structured events so I can build real-time UIs and off-chain indices without polling storage.

#### Acceptance Criteria

1. `create_stream` SHALL emit event topic `["stream_created", stream_id]` with data `{sender, receiver, token, amount, start_time, stop_time}`.

2. `withdraw` SHALL emit event topic `["stream_withdrawn", stream_id]` with data `{receiver, amount}`.

3. `cancel_stream` SHALL emit event topic `["stream_cancelled", stream_id]` with data `{sender, sender_refund, receiver_payout}`.

4. All events SHALL be published via `e.events().publish(topics, data)` following Soroban event conventions.

---

### REQ-8: Non-Functional Requirements

#### Performance
- `withdrawable_balance` SHALL execute as a read-only call (no ledger writes) completing within a single ledger close (~5 seconds on Stellar mainnet).
- No contract function SHALL perform unbounded iteration over all streams.

#### Scalability
- The contract SHALL support up to `u64::MAX` concurrent stream IDs with no degradation in per-stream correctness.
- Each stream SHALL occupy isolated persistent storage under `StorageKey::Stream(u64)`.

#### Token Compatibility
- The contract SHALL only interact with token contracts implementing the Soroban token interface (`token::Client` / SEP-41).

#### Precision
- Stream amounts SHALL use `i128` arithmetic throughout to prevent overflow on large values.
- Integer floor division is authoritative; no floating-point arithmetic is permitted.
