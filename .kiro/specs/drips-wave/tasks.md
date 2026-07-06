# Implementation Plan: Drips Wave — StreamFundContract

## Overview

Implement the `StreamFundContract` Soroban smart contract on Stellar end-to-end: core types, all four entry points (`create_stream`, `withdrawable_balance`, `withdraw`, `cancel_stream`), unit tests for every function and error branch, invariant verification tests, and a clean build.

Tasks are ordered so each builds on the previous. Types must exist before functions; functions before tests; all tests before the final build check.

## Tasks

- [ ] 1. Scaffold Soroban project and declare core types — confirm `Cargo.toml` has `soroban-sdk` with `contracttype` and `contracterror` features and `#![no_std]` at crate root; declare `StreamStatus` enum (`Active`, `Cancelled`) with `#[contracttype]`; declare `Stream` struct with fields `sender: Address`, `receiver: Address`, `token: Address`, `total_amount: i128`, `start_time: u64`, `stop_time: u64`, `withdrawn_amount: i128`, `status: StreamStatus`, `receiver_retained: i128` with `#[contracttype]`; declare `StorageKey` enum with variant `Stream(u64)` with `#[contracttype]`; declare `StreamError` enum with variants `StreamNotFound=1` through `Unauthorized=10` with `#[contracterror]` and `#[derive(Copy, Clone, Debug, Eq, PartialEq)]`; add `StreamFundContract` struct with `#[contract]` and empty `#[contractimpl]` stubs for all four entry points
  - [ ] 1.1 Confirm `Cargo.toml` dependencies and `#![no_std]` at crate root
  - [ ] 1.2 Declare `StreamStatus` enum with `#[contracttype]` and derives
  - [ ] 1.3 Declare `Stream` struct with all nine fields and `#[contracttype]`
  - [ ] 1.4 Declare `StorageKey` enum with `Stream(u64)` variant and `#[contracttype]`
  - [ ] 1.5 Declare `StreamError` enum with all ten variants and `#[contracterror]`
  - [ ] 1.6 Add `StreamFundContract` struct and `#[contractimpl]` block with empty stubs

- [ ] 2. Implement `create_stream` — call `sender.require_auth()` first; return `InvalidAmount` if `amount <= 0`; return `InvalidTimeRange` if `start_time >= stop_time`; return `StopTimeInPast` if `stop_time <= e.ledger().timestamp()`; return `SelfStream` if `sender == receiver`; return `DuplicateStreamId` if stream key already in storage; call `token::Client::transfer(sender → contract, amount)` to escrow; write `Stream` with `status: Active`, `withdrawn_amount: 0`, `receiver_retained: 0` to persistent storage; extend TTL to `(stop_time - current_time) + 31_536_000`; emit `stream_created` event
  - [ ] 2.1 Add `sender.require_auth()` as first statement
  - [ ] 2.2 Validate `amount > 0` → `InvalidAmount`
  - [ ] 2.3 Validate `start_time < stop_time` → `InvalidTimeRange`
  - [ ] 2.4 Validate `stop_time > current_time` → `StopTimeInPast`
  - [ ] 2.5 Validate `sender != receiver` → `SelfStream`
  - [ ] 2.6 Check storage for existing stream → `DuplicateStreamId`
  - [ ] 2.7 Call `token::Client::transfer` to escrow tokens into contract
  - [ ] 2.8 Write `Stream` record to persistent storage
  - [ ] 2.9 Extend storage TTL on the new key
  - [ ] 2.10 Emit `stream_created` event with all fields

- [ ] 3. Implement `withdrawable_balance` — load stream or return `StreamNotFound`; if `status == Cancelled` return `receiver_retained`; if `current_time <= start_time` return `0`; compute `elapsed = min(current_time, stop_time) - start_time`, `duration = stop_time - start_time`, `total_streamed = (total_amount * elapsed as i128) / duration as i128` using integer floor division; return `total_streamed - withdrawn_amount`; confirm no storage writes
  - [ ] 3.1 Load stream from storage; return `StreamNotFound` if absent
  - [ ] 3.2 Return `receiver_retained` immediately if `status == Cancelled`
  - [ ] 3.3 Return `0` if `current_time <= start_time`
  - [ ] 3.4 Compute `elapsed`, `duration`, and `total_streamed` with floor division
  - [ ] 3.5 Return `total_streamed - withdrawn_amount`
  - [ ] 3.6 Confirm function is read-only (no storage writes)

- [ ] 4. Implement `withdraw` — load stream or return `StreamNotFound`; call `receiver.require_auth()`; return `InvalidAmount` if `amount <= 0`; get `available` from `withdrawable_balance`; return `InsufficientBalance` if `amount > available` or if cancelled stream's retained balance is exhausted; increment `withdrawn_amount` and decrement `receiver_retained` (if cancelled) before any transfer (CEI); write updated stream and extend TTL; call `token::Client::transfer(contract → receiver, amount)`; emit `stream_withdrawn` event
  - [ ] 4.1 Load stream; return `StreamNotFound` if absent
  - [ ] 4.2 Call `stream.receiver.require_auth()`
  - [ ] 4.3 Validate `amount > 0` → `InvalidAmount`
  - [ ] 4.4 Compute available balance and check `amount <= available` → `InsufficientBalance`
  - [ ] 4.5 Check exhausted retained balance on cancelled streams → `InsufficientBalance`
  - [ ] 4.6 CEI: update `withdrawn_amount` (and `receiver_retained`) in storage before transfer
  - [ ] 4.7 Extend TTL on updated stream key
  - [ ] 4.8 Call `token::Client::transfer` to release tokens to receiver
  - [ ] 4.9 Emit `stream_withdrawn` event

- [ ] 5. Implement `cancel_stream` — load stream or return `StreamNotFound`; assert `status == Active` or return `StreamNotActive`; call `sender.require_auth()`; compute `elapsed`, `duration`, `streamed`, `receiver_payout = streamed - withdrawn_amount`, `sender_refund = total_amount - streamed`; CEI: set `status = Cancelled`, set `receiver_retained = receiver_payout`, write stream and extend TTL; if `sender_refund > 0` call `token::Client::transfer(contract → sender, sender_refund)`; emit `stream_cancelled` event
  - [ ] 5.1 Load stream; return `StreamNotFound` if absent
  - [ ] 5.2 Assert `status == Active` → `StreamNotActive` otherwise
  - [ ] 5.3 Call `stream.sender.require_auth()`
  - [ ] 5.4 Compute `elapsed`, `duration`, `streamed`, `receiver_payout`, `sender_refund`
  - [ ] 5.5 CEI: set `status = Cancelled`, set `receiver_retained`, write stream to storage
  - [ ] 5.6 Extend TTL on updated stream key
  - [ ] 5.7 Transfer `sender_refund` to sender if > 0
  - [ ] 5.8 Emit `stream_cancelled` event

- [ ] 6. Write unit tests for `create_stream` — happy path (stream stored, tokens escrowed, event emitted), `DuplicateStreamId` on duplicate ID, `InvalidTimeRange` on bad time range, `StopTimeInPast` on past stop time, `InvalidAmount` on zero/negative amount, `SelfStream` on sender==receiver, auth failure without sender auth
  - [ ] 6.1 Happy path test
  - [ ] 6.2 `DuplicateStreamId` test
  - [ ] 6.3 `InvalidTimeRange` test
  - [ ] 6.4 `StopTimeInPast` test
  - [ ] 6.5 `InvalidAmount` test
  - [ ] 6.6 `SelfStream` test
  - [ ] 6.7 Authorization failure test

- [ ] 7. Write unit tests for `withdrawable_balance` — `t < start_time` returns 0, `t == start_time` returns 0, `t` midway returns floor-divided amount, `t == stop_time` returns remainder, `t > stop_time` returns full remainder, after partial withdrawal subtracts correctly, cancelled stream returns `receiver_retained`, missing stream returns `StreamNotFound`
  - [ ] 7.1 `t < start_time` returns 0
  - [ ] 7.2 `t == start_time` returns 0
  - [ ] 7.3 Midpoint returns correct floor amount
  - [ ] 7.4 `t == stop_time` returns full remainder
  - [ ] 7.5 `t > stop_time` returns full remainder
  - [ ] 7.6 After partial withdrawal subtracts correctly
  - [ ] 7.7 Cancelled stream returns `receiver_retained`
  - [ ] 7.8 `StreamNotFound` on missing stream

- [ ] 8. Write unit tests for `withdraw` — happy path (state updated, tokens transferred, event emitted), multiple partial withdrawals accumulate correctly, post-cancel withdrawal claims retained balance, `InsufficientBalance` on over-amount, `InvalidAmount` on zero/negative, `StreamNotFound` on missing stream, auth failure without receiver auth, second withdraw after retained balance exhausted returns `InsufficientBalance`
  - [ ] 8.1 Happy path test
  - [ ] 8.2 Multiple partial withdrawals test
  - [ ] 8.3 Post-cancel withdrawal test
  - [ ] 8.4 `InsufficientBalance` test
  - [ ] 8.5 `InvalidAmount` test
  - [ ] 8.6 `StreamNotFound` test
  - [ ] 8.7 Authorization failure test
  - [ ] 8.8 Exhausted retained balance test

- [ ] 9. Write unit tests for `cancel_stream` — cancel before stream starts (full refund, zero receiver payout), cancel midway (correct split), cancel after stop (zero sender refund), `stream_cancelled` event fields correct, `StreamNotActive` on already-cancelled stream, `StreamNotFound` on missing stream, auth failure without sender auth, receiver can withdraw retained balance after cancellation
  - [ ] 9.1 Cancel before start: full refund to sender
  - [ ] 9.2 Cancel midway: correct accounting split
  - [ ] 9.3 Cancel after stop: zero sender refund
  - [ ] 9.4 Event fields correct
  - [ ] 9.5 `StreamNotActive` on cancelled stream
  - [ ] 9.6 `StreamNotFound` on missing stream
  - [ ] 9.7 Authorization failure test
  - [ ] 9.8 Receiver withdraws retained balance after cancel

- [ ] 10. Verify balance conservation invariants — test `create → partial withdraw → cancel` sums to `total_amount`; test `create → cancel → full withdraw` empties contract balance for that stream; test `create → withdraw at stop_time` equals `total_amount`; test `withdrawable_balance` never negative across all time points; test `withdrawn_amount` never decreases across sequential withdrawals
  - [ ] 10.1 `create → partial withdraw → cancel` conservation check
  - [ ] 10.2 `create → cancel → full withdraw` empties stream balance
  - [ ] 10.3 `create → withdraw at stop_time` equals `total_amount`
  - [ ] 10.4 `withdrawable_balance` non-negative across all time points
  - [ ] 10.5 `withdrawn_amount` monotonically non-decreasing

- [ ] 11. Build and validate clean compilation — run `cargo build --target wasm32-unknown-unknown --release` with no errors; run `cargo test` with all tests passing; run `cargo clippy -- -D warnings` with no warnings; confirm compiled `.wasm` is within Soroban contract size limits
  - [ ] 11.1 `cargo build --target wasm32-unknown-unknown --release` succeeds
  - [ ] 11.2 `cargo test` all tests pass
  - [ ] 11.3 `cargo clippy -- -D warnings` clean
  - [ ] 11.4 `.wasm` artifact within Soroban size limits

## Task Dependency Graph

```json
{
  "waves": [
    { "wave": 1, "tasks": [1] },
    { "wave": 2, "tasks": [2] },
    { "wave": 3, "tasks": [3] },
    { "wave": 4, "tasks": [4] },
    { "wave": 5, "tasks": [5] },
    { "wave": 6, "tasks": [6, 7, 8, 9] },
    { "wave": 7, "tasks": [10] },
    { "wave": 8, "tasks": [11] }
  ]
}
```

## Notes

- All four contract functions must follow the CEI (Checks-Effects-Interactions) pattern: authorization and input validation first, storage writes second, token transfers last.
- `StreamStatus::Completed` is a derived state (inferred when `current_time >= stop_time`) — it does not require a storage write or explicit enum variant.
- The `withdrawable_balance` function is read-only and must not write to storage; it is safe to call internally from `withdraw` and `cancel_stream`.
- Use `e.ledger().timestamp()` (not wall-clock time) as the authoritative time source throughout.
- Soroban's integer types: `u64` for timestamps, `i128` for token amounts. Never mix without explicit casting.
