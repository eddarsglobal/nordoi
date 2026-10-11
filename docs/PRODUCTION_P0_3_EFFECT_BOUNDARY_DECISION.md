# P0.3 — Native kernel effect backend adapter, research/production-track candidate

Certified baseline for this package (not a global production release):
- Research tag `r0.23`, commit `0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5`.
- Production P0.1+P0.2 FIXED1, commit `e0068bc58442d99a7fed7f43cb00549e29530acf`.
- CI P0.1 5/5 and P0.2 5/5 succeeded on the exact production baseline.

## What the code actually does

The new standalone `production/native-effect-p03` Rust crate **depends on the actual `nordoi_kernel` root crate** and implements `nordoi_kernel::EffectBackend`, whose `execute_with_context` method is used by the existing kernel dispatcher and audit journal. The wrapper owns its backend and its P0.2 DurableTrustGate. The legacy `execute` method always denies. Each context-rich request is canonically bound to effect kind and scope, action, reaction ID, cycle, ordinal, intent ID, persistent delivery namespace, and fencing token, then checked against a real P0.1 Ed25519 signed envelope. Valid signed admission is written and synchronized to the P0.2 journal **before** forwarding the effect to the wrapped backend. A missing, malformed, unsigned, replayed or mismatched request denies before downstream execution.

## Critical limitations (fail closed, do NOT claim production authorization)

1. **Opt-in adapter, not mandatory kernel enforcement.** Existing unwrapped backends are still accessible by existing kernel callers. This patch does not change `src/` or the legacy kernel interface. A later stage must require the wrapper for *all* production host-effect paths and verify there is no bypass.
2. **No atomic commit with an external destination.** If backend execution fails, the Ed25519 sequence has already been committed. Retrying with the same envelope must fail, and retrying with a new signed sequence may perform a duplicate operation. Existing kernel audit/retry/fencing must be reconciled with the trust journal, with explicit `IN_DOUBT` semantics and destination-level idempotency.
3. **Rollback of an entire valid P0.2 journal** is not detected without a separately trusted monotonic anchor. Crash-after-fsync means the authorization may be consumed without any external effect.
4. The `SignedEnvelopeSource` must be a trusted host implementation supplying **pre-authorized signatures**. Do not expose root signing keys to untrusted NOI programs, LLMs, plugins or host-controlled payloads. Signing arbitrary requests on demand defeats the policy.
5. Versioned fixed canonical request bindings are separate from actual platform path/URL normalization policy; P0.3 authorizes exact byte scopes only. Invalid or empty context denies; it does not provide filesystem isolation or network sandboxing.
6. This candidate is not an independent external audit; it does not close the outstanding historical native gates.

## Test/CI acceptance

The standalone crate has its own `Cargo.lock` generated on developer Mac and committed for `--locked` CI. Run `scripts/production_p03_gate.sh`, followed by `production_p02_gate.sh`, `production_p01_gate.sh` and `release_gate.sh`. CI P0.3 has quality, check, Ubuntu, macOS and Windows jobs. P0.1 and P0.2 CI continue independently. No production authorization is granted by these green checks.

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
