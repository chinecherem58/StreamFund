# StreamFund — Drips Wave

A Soroban smart contract on Stellar that enables continuous, second-by-second token streaming from funders to open-source maintainers.

## Overview

StreamFund extends the [Drips Network](https://drips.network) dependency-funding model to the Stellar ecosystem. A funder escrows tokens into the contract which releases them linearly to a receiver over a defined time window. The receiver can withdraw their accrued balance at any point; the funder can cancel early and recover unstreamed tokens.

## Contract Functions

| Function | Caller | Description |
|---|---|---|
| `create_stream` | Sender | Escrow tokens and create a linear stream |
| `withdrawable_balance` | Anyone | Query claimable balance (read-only) |
| `withdraw` | Receiver | Pull accrued tokens to receiver wallet |
| `cancel_stream` | Sender | Cancel stream; refund unstreamed tokens |

## Architecture

```
Funder ──create_stream──► [StreamFundContract escrow] ──withdraw──► Maintainer
       ◄──cancel_stream─── refund unstreamed             retained ──► Maintainer
```

All accounting is on-chain and deterministic based on `e.ledger().timestamp()`. No oracle or off-chain component is required for core operations.

## Security Model

- **CEI pattern** — all storage writes happen before any outbound token transfer
- **Typed authorization** — `require_auth()` is the first call in every mutating function
- **No admin keys** — the contract holds no privileged authority over user funds
- **Typed errors** — every failure returns a stable `StreamError` discriminant
- **Integer arithmetic** — all amounts use `i128`; no floating-point

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- A C linker (MSVC `link.exe`, `gcc`, or `clang`)

```sh
rustup target add wasm32-unknown-unknown
```

## Build

```sh
# Run all tests
cargo test

# Build optimised WASM artifact
cargo build --target wasm32-unknown-unknown --release
```

The compiled artifact will be at:
```
target/wasm32-unknown-unknown/release/stream_fund.wasm
```

## Deploy to Testnet

```sh
export STELLAR_SECRET_KEY=S...your...secret...key
bash scripts/deploy_testnet.sh
```

The script will:
1. Build the optimised WASM
2. Check it fits within the 64KB Soroban size limit
3. Deploy to Stellar Testnet
4. Run a smoke test against the live contract
5. Write the contract ID to `.env.testnet`

## Pre-Mainnet Checklist

Before deploying to mainnet, work through every item in [`AUDIT_CHECKLIST.md`](AUDIT_CHECKLIST.md). Key gates:

- `cargo test` passes with zero failures
- `cargo clippy -- -D warnings` is clean
- WASM artifact ≤ 64KB
- Testnet deployment smoke-tested
- External security audit completed

## Stream Lifecycle

```
          create_stream
               │
               ▼
           [Active] ──── cancel_stream ──► [Cancelled]
               │                               │
         stop_time reached                withdraw (retained)
               │                               │
           [Completed]                     [Exhausted]
               │
         withdraw (full)
```

States `Cancelled` and `Completed` both permit the receiver to claim their accrued balance. `Completed` is a derived state — no explicit transition call is needed.

## Token Conservation Invariant

At any point in a stream's lifecycle:

```
withdrawn_amount + receiver_retained + sender_refund == total_amount
```

This invariant is verified by the property tests in `src/tests/test_invariants.rs`.

## Event Schema

| Event | Topics | Data |
|---|---|---|
| `stream_created` | `("created", stream_id)` | `(sender, receiver, token, amount, start_time, stop_time)` |
| `stream_withdrawn` | `("withdrawn", stream_id)` | `(receiver, amount)` |
| `stream_cancelled` | `("cancelled", stream_id)` | `(sender, sender_refund, receiver_payout)` |

## License

MIT
