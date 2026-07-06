# Design Document

## Overview

Drips Wave is a single Soroban smart contract (`StreamFundContract`) deployed on Stellar. It holds token escrow and manages the full lifecycle of continuous payment streams. There is no off-chain component required for core stream operations — all accounting is on-chain and deterministic based on ledger timestamps.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Stellar Network                          │
│                                                                 │
│   Funder Wallet          StreamFundContract        Maintainer   │
│   ───────────           ──────────────────         ─────────── │
│   create_stream ──────► escrow tokens                           │
│   cancel_stream ──────► refund unstreamed ─────────────────►   │
│                         retain accrued                          │
│                                          withdraw ◄─────────    │
│                         release accrued ───────────────────►    │
│                                                                 │
│   ┌──────────────────────────────────────────────┐             │
│   │  Persistent Storage (per stream_id)          │             │
│   │  Stream { sender, receiver, token,           │             │
│   │           total_amount, start_time,          │             │
│   │           stop_time, withdrawn_amount,       │             │
│   │           status, receiver_retained }        │             │
│   └──────────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

---

## Components and Interfaces

### StreamFundContract

The single deployable contract unit. Exposes four public entry points:

| Function | Caller | Mutates State | Emits Event |
|---|---|---|---|
| `create_stream` | Funder | Yes | `stream_created` |
| `withdrawable_balance` | Anyone | No | None |
| `withdraw` | Maintainer | Yes | `stream_withdrawn` |
| `cancel_stream` | Funder | Yes | `stream_cancelled` |

### Token Interface

The contract interacts with any SEP-41 compliant token contract via `soroban_sdk::token::Client`. It calls only `transfer` — it does not rely on `approve`/`transfer_from` patterns.

### Storage Interface

Each stream is stored independently in `e.storage().persistent()` under `StorageKey::Stream(u64)`. No shared/global state exists between streams.

---

## Data Models

### `Stream` Struct

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub sender: Address,           // Funder who created the stream
    pub receiver: Address,         // Maintainer receiving the stream
    pub token: Address,            // SEP-41 token contract address
    pub total_amount: i128,        // Total tokens escrowed at creation
    pub start_time: u64,           // Ledger timestamp when streaming begins
    pub stop_time: u64,            // Ledger timestamp when streaming ends
    pub withdrawn_amount: i128,    // Cumulative tokens already withdrawn
    pub status: StreamStatus,      // Active | Cancelled | Completed
    pub receiver_retained: i128,   // Claimable balance captured at cancellation
}
```

### `StreamStatus` Enum

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Active,
    Cancelled,
    // Completed is inferred (no storage write): current_time >= stop_time
}
```

### `StorageKey` Enum

```rust
#[contracttype]
pub enum StorageKey {
    Stream(u64),
}
```

### `StreamError` Enum

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    StreamNotFound      = 1,
    DuplicateStreamId   = 2,
    InvalidTimeRange    = 3,
    StopTimeInPast      = 4,
    InvalidAmount       = 5,
    SelfStream          = 6,
    InsufficientBalance = 7,
    StreamNotActive     = 8,
    TransferFailed      = 9,
    Unauthorized        = 10,
}
```

---

## Function Flows

### `create_stream`

```
sender.require_auth()
→ validate: amount > 0, start_time < stop_time, stop_time > now, sender != receiver
→ assert stream_id not in storage  (→ DuplicateStreamId)
→ token::Client::transfer(sender → contract, amount)
→ write Stream { status: Active, withdrawn_amount: 0, receiver_retained: 0 } + extend TTL
→ emit stream_created event
```

### `withdrawable_balance`

```
load stream  (→ StreamNotFound if absent)
if status == Cancelled → return receiver_retained
if now <= start_time   → return 0
elapsed  = min(now, stop_time) - start_time
duration = stop_time - start_time
streamed = (total_amount * elapsed as i128) / duration as i128   ← floor
return streamed - withdrawn_amount
```

### `withdraw`

```
load stream  (→ StreamNotFound if absent)
receiver.require_auth()
validate amount > 0  (→ InvalidAmount)
available = withdrawable_balance(stream_id)
assert amount <= available  (→ InsufficientBalance)
[CEI] stream.withdrawn_amount += amount  (if Cancelled: stream.receiver_retained -= amount)
write stream + extend TTL
token::Client::transfer(contract → receiver, amount)
emit stream_withdrawn event
```

### `cancel_stream`

```
load stream  (→ StreamNotFound if absent)
assert status == Active  (→ StreamNotActive)
sender.require_auth()
elapsed       = max(0, min(now, stop_time) - start_time)
duration      = stop_time - start_time
streamed      = (total_amount * elapsed as i128) / duration as i128
receiver_payout = streamed - withdrawn_amount
sender_refund   = total_amount - streamed
[CEI] stream.status = Cancelled
      stream.receiver_retained = receiver_payout
      write stream + extend TTL
if sender_refund > 0 → token::Client::transfer(contract → sender, sender_refund)
emit stream_cancelled event
```

---

## Storage Design

| Aspect | Decision |
|--------|---------|
| Layer | `e.storage().persistent()` — survives ledger archival |
| Key | `StorageKey::Stream(stream_id: u64)` per stream |
| TTL extension | Every write calls `extend_ttl(key, min_ttl, max_ttl)` where `max_ttl = (stop_time − current_time) + 31_536_000` |
| Isolation | No shared/global state; streams cannot affect each other |
| Cleanup | Streams are never explicitly deleted; they expire naturally after TTL |

---

## Event Schema

### `stream_created`
- **Topics:** `(Symbol::new("stream_created"), stream_id: u64)`
- **Data:** `(sender: Address, receiver: Address, token: Address, amount: i128, start_time: u64, stop_time: u64)`

### `stream_withdrawn`
- **Topics:** `(Symbol::new("stream_withdrawn"), stream_id: u64)`
- **Data:** `(receiver: Address, amount: i128)`

### `stream_cancelled`
- **Topics:** `(Symbol::new("stream_cancelled"), stream_id: u64)`
- **Data:** `(sender: Address, sender_refund: i128, receiver_payout: i128)`

---

## Correctness Properties

The following invariants must hold across all valid operation sequences:

### Property 1: No Over-Withdrawal
`withdrawn_amount ≤ total_amount` at all times, for every stream in any state.

**Validates: Requirements REQ-3.2, REQ-3.4**

### Property 2: Conservation at Cancellation
At the moment `cancel_stream` executes: `streamed_amount + sender_refund == total_amount`.

**Validates: Requirements REQ-4.2**

### Property 3: Full Token Conservation
After cancellation: `withdrawn_amount + receiver_retained + sender_refund == total_amount`. No tokens are created or destroyed.

**Validates: Requirements REQ-4.2, REQ-4.3**

### Property 4: Non-Negative Withdrawable Balance
`withdrawable_balance(stream_id) ≥ 0` for any valid stream at any ledger timestamp.

**Validates: Requirements REQ-2.1, REQ-2.2**

### Property 5: Monotonic Withdrawn Amount
`withdrawn_amount` never decreases over the lifetime of a stream.

**Validates: Requirements REQ-3.2, REQ-3.7**

### Property 6: Dust Bound
The integer remainder from floor division is always `< duration`, meaning a receiver can claim all tokens by `stop_time`.

**Validates: Requirements REQ-2.6**

---

## Error Handling

All error conditions return typed `StreamError` variants, enabling callers to pattern-match precisely without parsing strings:

| Error | Trigger |
|-------|---------|
| `StreamNotFound` | `stream_id` not in storage |
| `DuplicateStreamId` | `stream_id` already exists |
| `InvalidTimeRange` | `start_time >= stop_time` |
| `StopTimeInPast` | `stop_time <= current_ledger_timestamp` |
| `InvalidAmount` | `amount <= 0` |
| `SelfStream` | `sender == receiver` |
| `InsufficientBalance` | `amount > withdrawable_balance` |
| `StreamNotActive` | `cancel_stream` on non-Active stream |
| `TransferFailed` | Token client call fails |
| `Unauthorized` | `require_auth` fails |

No panics or `expect`/`unwrap` calls appear in production paths; all fallible operations return `Result<_, StreamError>`.

---

## Security Considerations

- **CEI pattern:** All storage writes occur before any token transfer to prevent reentrancy.
- **Authorization boundary:** `require_auth` is the first statement in each mutating function.
- **No admin keys:** The contract holds no privileged authority over user funds beyond the stream rules.
- **Integer safety:** All arithmetic uses `i128`. Token supply on Stellar is bounded at `i128::MAX`.
- **Token interface enforcement:** Only addresses implementing `token::Client` (SEP-41) are accepted; calling a non-token address would panic the token client, aborting the transaction.

---

## Testing Strategy

### Unit Tests (in `#[cfg(test)]` module)

- Each function: happy path, all error branches, authorization failure
- Time boundary cases: `t < start_time`, `t == start_time`, `t between`, `t == stop_time`, `t > stop_time`
- Cancellation accounting: `receiver_payout + sender_refund == total_amount` verified numerically
- Post-cancel withdraw: receiver can still withdraw retained balance

### Property / Fuzz Tests

- **Invariant:** `withdrawn_amount ≤ total_amount` survives arbitrary sequences of `withdraw` and `cancel_stream`
- **Invariant:** `withdrawable_balance` is always `≥ 0`
- **Invariant:** Sum of all transfers out never exceeds `total_amount`

### Integration Tests (Soroban Testnet)

- Deploy contract, mint mock token, execute full stream lifecycle end-to-end
- Verify event emission matches schema
- Verify TTL extension on each write

---

## Out of Scope (v1)

- Stream-to-stream chaining
- Multi-token streams
- Upgradeable contract proxy
- On-chain stream enumeration (deferred to off-chain indexer consuming events)
- Fee collection by platform operator
