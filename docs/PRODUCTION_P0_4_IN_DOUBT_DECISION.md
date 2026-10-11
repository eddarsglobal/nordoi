# NORDOI P0.4 decision record — fail-closed host delivery ledger

- Canonical identity: 16-byte delivery namespace plus 8-byte intent ID; fencing token is intentionally **not** part of the identity, so a new lease cannot replay the same delivery. The P0.3 signed request still binds the fence and payload.
- Conservative pre-execution WAL: record IN_DOUBT, `fsync`, only then call P0.3 Ed25519 durable admission and the target backend. Append a separate local acknowledgment record only when backend returns success. An error leaves IN_DOUBT forever until a separately designed reconciler is certified.
- Recovery: scan the complete fixed-record hash chain; torn and corrupt bytes abort before any new execution. Concurrent access requires exclusive create-new lock. No automatic lock breaking.
- Host policy: this is an opt-in production host facade. The kernel's direct public backend API still exists. No claim of global bypass elimination.
- External risk: the destination can have performed a side effect even if backend returns failure, loses connectivity, or crashes. This candidate prevents *automatic re-execution of the same namespace+intent ID by this facade*, but not all duplicates in the wider system.
- Unsolved: independent anti-rollback witness, identity issuance policy, destination idempotency/reconciliation, host sandbox, comprehensive adversarial review, and a mandatory runtime enforcement configuration.

P0.4 is validated only if Rustfmt, Clippy `-D warnings`, Cargo Check, tests on Ubuntu/macOS/Windows, P0.1/P0.2/P0.3 regressions and the root Release Gate pass. Until then this is a package candidate, not a certified implementation.

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
