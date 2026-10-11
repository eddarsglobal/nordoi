# Production P0.1 — native trust decision

Baseline: certified research tag `r0.23`, commit `0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5`. The branch `production/p0.1` is exploratory until each hard gate is satisfied.

## Actual P0.1 scope

1. Add isolated, authentic strict Ed25519 signature verification with canonical domain-separated messages.
2. Bind signer identity, epoch, exact monotonic in-memory sequence, intent, logical capability/resource, and payload digest.
3. Require signatures from the *current* trusted key for rotation to next epoch and revocation.
4. Keep root lineages independent and fail-closed.
5. Add dedicated CI on macOS, Windows, and Linux, plus Rustfmt/Clippy and check.

## Non-claims and exit plan

The new crate is **NOT** integrated into `src/effect_dispatch`, `src/effect_audit`, or attestation backends. It is not a production authorization system. Any trust root material supplied from an untrusted source defeats its purpose; the bootstrap must come from a protected host-controlled trust store. In-memory counters and revocations are reset on process restart. The next milestone needs crash-consistent durable state stored/updated before external dispatch, tied to the existing fenced transaction/journal semantics; authorization and effect mutation must be failure-atomic. An independent verification implementation and key lifecycle/operational policy must then be tested. Integrate with existing effect boundary using an explicit opt-in profile, leaving the legacy behavior unchanged by default. Test backend failure/retry, replay after restart, stale writer, duplicate delivery, misbinding, compromised key, and signer/key swap.

**Never change** the `r0.23` tag or claim the 12 research native readiness gates `MET` or 12 independent external verifiers `PRESENT` solely because this crate passes. A production deployment gate requires a separate documented security review, provenance and signed reproducible artifacts.
