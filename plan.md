# StreamFund — Wave Program Plan

## What is StreamFund?

StreamFund is a Soroban smart contract on Stellar that enables continuous, second-by-second token streaming from funders to open-source maintainers. Built as part of the Drips Wave program, it extends the Drips Network's dependency-funding model to the Stellar ecosystem.

---

## Types of Work We'll Post

### Bug Fixes
- Arithmetic edge cases in `withdrawable_balance` at boundary timestamps
- TTL calculation drift under extreme stream durations
- Storage key collisions under concurrent stream creation
- Off-by-one errors in elapsed time computation

### New Features
- **Stream pause/resume** — allow senders to temporarily halt a stream without cancelling
- **Multi-recipient streams** — split a single escrow across multiple maintainers
- **Stream top-up** — allow senders to add more tokens to an active stream
- **Minimum stream duration guard** — reject streams shorter than a configurable threshold
- **Stream metadata** — attach an IPFS hash or description to a stream for UI display

### Testing
- Property-based fuzz tests for balance conservation invariants
- Testnet integration tests with real Stellar accounts
- Edge case tests: maximum `i128` amounts, zero-duration boundaries, same-block create+cancel
- Gas/instruction budget profiling per contract function

### Documentation
- Step-by-step contributor onboarding guide
- Contract ABI reference with examples for each function
- Event schema documentation for indexer developers
- Testnet walkthrough: deploy → create stream → withdraw → cancel

### Infrastructure
- GitHub Actions deployment pipeline to Stellar Testnet on merge to `main`
- WASM size regression check (fail CI if contract exceeds 64KB)
- Off-chain event indexer skeleton using Horizon event stream

---

## Sprint Structure

Each Wave sprint runs **2 weeks**. Issues are tagged by type and difficulty:

| Label | Meaning |
|---|---|
| `good-first-issue` | Self-contained, < 4 hours |
| `bug` | Confirmed defect with reproduction steps |
| `feature` | New contract or tooling capability |
| `docs` | Documentation or examples |
| `testing` | New test cases or test infrastructure |

Contributors pick up tagged issues, open a PR against `main`, and request review. Merged PRs are eligible for streaming rewards via the contract itself.

---

## Contribution Goals

- Keep the contract under 64KB WASM
- Maintain 100% test coverage on all contract entry points
- Every new function ships with unit tests and an updated `AUDIT_CHECKLIST.md` entry
- All PRs pass `cargo clippy -- -D warnings` before merge
