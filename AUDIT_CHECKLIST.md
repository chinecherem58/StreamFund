# Security Audit Checklist

This checklist is for use by internal reviewers and external auditors before mainnet deployment.

## Authorization

- [ ] `create_stream` — `sender.require_auth()` is the first call, before any reads or writes
- [ ] `withdraw` — `receiver.require_auth()` is called before any balance check or state mutation
- [ ] `cancel_stream` — `sender.require_auth()` is called after status check but before any state mutation
- [ ] No function allows a third-party address to move funds
- [ ] No admin or superuser key exists in the contract

## Checks-Effects-Interactions (CEI)

- [ ] `withdraw` — `withdrawn_amount` incremented in storage before `token::Client::transfer`
- [ ] `cancel_stream` — `status` and `receiver_retained` written to storage before `token::Client::transfer`
- [ ] `create_stream` — token transfer happens before stream record is written (safe: no outbound transfer, only inbound)
- [ ] No function reads stale state after an outbound transfer

## Arithmetic and Overflow

- [ ] All amounts use `i128` — no `u128` or `u64` for token values
- [ ] `saturating_mul` used for `total_amount * elapsed` intermediate product
- [ ] `saturating_sub` used for timestamp subtraction to prevent underflow
- [ ] Integer floor division is intentional and documented; dust is bounded by `duration`
- [ ] `withdrawn_amount` can never exceed `total_amount` (verified by invariant tests)
- [ ] `receiver_retained` can never be negative (set once at cancellation, decremented by withdraw only within available balance)

## Input Validation

- [ ] `amount <= 0` rejected before any state change
- [ ] `start_time >= stop_time` rejected before any state change
- [ ] `stop_time <= current_time` rejected before any state change
- [ ] `sender == receiver` rejected before any state change
- [ ] Duplicate `stream_id` rejected before token transfer

## Storage and TTL

- [ ] All streams use `e.storage().persistent()` — not `temporary()` or `instance()`
- [ ] TTL extended on every write (create, withdraw, cancel)
- [ ] TTL formula: `(stop_time - current_time) + 31_536_000` seconds, converted to ledgers at 5s/ledger
- [ ] `saturating_sub` and `saturating_add` used in TTL calculation to prevent underflow/overflow
- [ ] No stream record is explicitly deleted (they expire via TTL after the retention period)

## Token Interface

- [ ] Only `token::Client::transfer` is called — not `approve`, `transfer_from`, or raw XDR
- [ ] Token address is stored in the `Stream` struct and used consistently on all transfers
- [ ] Contract never constructs or assumes a token address — always uses the one provided at creation

## Event Emission

- [ ] `stream_created` emitted after successful escrow and storage write
- [ ] `stream_withdrawn` emitted after successful state update and token transfer
- [ ] `stream_cancelled` emitted after successful state update and refund transfer
- [ ] All event topics and data match the schema in `design.md`
- [ ] Event correctness verified by `test_events.rs`

## Stream Lifecycle

- [ ] `Cancelled` streams cannot be cancelled again (`StreamNotActive` returned)
- [ ] `Completed` streams (past `stop_time`) cannot be cancelled (`StreamNotActive` returned)
- [ ] Receiver can withdraw retained balance from a `Cancelled` stream
- [ ] Receiver cannot withdraw more than `receiver_retained` from a `Cancelled` stream
- [ ] `withdrawable_balance` on a `Cancelled` stream returns `receiver_retained`, not a time-based recalculation

## Conservation Invariant

- [ ] At all times: `withdrawn_amount + receiver_retained + sender_refund == total_amount`
  - Verified by `invariant_partial_withdraw_then_cancel`
  - Verified by `invariant_cancel_then_full_withdraw_empties_contract`
  - Verified by `invariant_withdraw_full_at_stop_time`
  - Verified by `invariant_no_tokens_created_or_destroyed`
- [ ] `withdrawable_balance` never returns a negative value (verified by `invariant_balance_never_negative`)
- [ ] `withdrawn_amount` is monotonically non-decreasing (verified by `invariant_withdrawn_amount_monotonic`)

## Test Coverage

- [ ] All 4 contract functions have happy-path tests
- [ ] All `StreamError` variants are exercised by at least one test
- [ ] All time-boundary cases tested: `t < start`, `t == start`, `t midway`, `t == stop`, `t > stop`
- [ ] All 3 events verified by `test_events.rs`
- [ ] All 6 conservation invariants tested
- [ ] Authorization failure tested for all 3 mutating functions

## Pre-Mainnet Gates

- [ ] `cargo test` — all tests pass with zero failures
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo build --target wasm32-unknown-unknown --release` — clean build
- [ ] Compiled WASM ≤ 64KB (Soroban contract size limit)
- [ ] Deployed and smoke-tested on Stellar Testnet
- [ ] External security audit completed by a qualified firm
- [ ] Audit report reviewed and all critical/high findings resolved
- [ ] Contract ID and WASM hash recorded for mainnet deployment
