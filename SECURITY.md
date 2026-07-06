# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Yes     |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report vulnerabilities by emailing the maintainers directly. Include:

1. A clear description of the vulnerability
2. Steps to reproduce or a proof-of-concept
3. The potential impact (funds at risk, denial of service, etc.)
4. Your suggested fix (optional)

You will receive acknowledgement within 48 hours. We aim to release a patch within 7 days of a confirmed critical issue.

## Known Security Properties

The following properties are explicitly guaranteed by the contract design:

| Property | Mechanism |
|---|---|
| No reentrancy | CEI pattern — storage written before all outbound transfers |
| No unauthorised withdrawal | `receiver.require_auth()` guards all `withdraw` calls |
| No unauthorised cancellation | `sender.require_auth()` guards all `cancel_stream` calls |
| No fund creation | Conservation invariant: `withdrawn + retained + refund == total_amount` |
| No integer overflow | All amounts use `i128`; `saturating_mul` used for intermediate products |
| No silent storage eviction | TTL extended to `stop_time + 1 year` on every write |

## Out of Scope

- Front-end or indexer components
- Third-party token contracts passed as the `token` parameter
- Stellar network-level issues

## Audit Status

This contract has **not yet been audited**. Do not use on mainnet with real funds until an independent audit has been completed.
