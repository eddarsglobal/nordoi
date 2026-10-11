# NORDOI Production P0.4 — Durable in-doubt delivery identity (CANDIDATE)

Baseline: `production/p0.1` commit `35122a1d15c5db66e6603246cb7b43b59ebe29b0` (P0.1/P0.2/P0.3 CI 15/15). Research tag `r0.23` unchanged.

This additive host-side `ProductionEffectBackend` wraps the actual P0.3 `GuardedEffectBackend`, implements the real `nordoi_kernel::EffectBackend` trait, and writes a bounded, hash-chained delivery-identity reservation **before** signed admission and downstream external work. An uncertain outcome remains `InDoubt` across reboots; even a new signed sequence or new fencing token for the SAME namespace + intent ID cannot automatically reissue the operation. A locally successful backend returns `LocallyAcknowledged` only after another fsynced record. Contextless calls deny.

This is intentionally **fail-closed at the cost of availability**: invalid/missing signed envelopes may reserve a delivery ID before P0.3 refuses it. An external operator must reconcile such IN_DOUBT identities; do not automatically clear or retry them. Concurrent writers are denied with an exclusive lock marker. A crash may leave the lock marker behind: safely determine that the previous owner is dead *before* supervised cleanup. There is no automated stale-lock override.

## What is NOT solved

1. This wrapper is **opt-in**. Existing unwrapped kernel `EffectBackend` implementations remain available, so kernel-wide mandatory policy is NOT demonstrated.
2. This ledger and P0.2 signed trust journal are separate files. The ordering is conservative (reservation before trust admission), but they are not an atomic cross-file transaction. Some denials may remain falsely IN_DOUBT.
3. Neither local receipt nor journal state proves destination-side exactly-once execution or destination idempotency. An operational reconciliation protocol is needed.
4. Replacing the entire ledger with an older valid copy is not detected without an independent monotonic anchor. Ledger paths and signing keys must remain host-protected.
5. The exclusive lock uses `create_new` and a lock-file marker; crash-stale markers require trusted operator intervention. It is not OS process-liveness attestation.
6. The current test candidate is NOT audited as production-safe. It should never enable live external effects by default.

Patch: 6 new files and a one-file additive change (`pub mod production;`). No `Cargo.toml` or `Cargo.lock` change. Before any commit: run `bash scripts/production_p04_gate.sh`, `bash scripts/production_p03_gate.sh`, `bash scripts/production_p02_gate.sh`, `bash scripts/production_p01_gate.sh`, `bash scripts/release_gate.sh`. New CI covers 5 jobs (quality, check, Ubuntu, macOS, Windows).

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
