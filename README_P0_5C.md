# NORDOI Production P0.5C — Closed Build-Graph Boundary Candidate

P0.5C follows the certified P0.5B commit:

`24fb905b341d31296403f02040d44eeba13109ce`

The P0.5C read-only inventory reported three Cargo manifests, no direct raw consumer outside the P0.3 crate, and no certified closed deployment graph yet.

This candidate introduces one new boundary crate:

`production/native-effect-profile-p05c`

Its direct dependency set is intentionally exactly one package:

`nordoi-native-effect-p03`

Its public Rust surface intentionally re-exports only:

`nordoi_native_effect_p03::enforced::FencedProductionEffects`

The new gate verifies with Cargo metadata that the direct dependency graph stays exact and with negative compilation probes that a downstream consumer declaring only the P0.5C boundary cannot directly import:

- `nordoi_native_effect_p03`
- `nordoi_kernel`
- `nordoi_native_trust_p01`

This is a **closed build/import boundary for the selected profile**, not a claim that historical kernel APIs have been removed. It does not provide a concrete persistent fencing store, a real external destination backend, recipient-side exactly-once semantics, independent rollback anchoring, or production deployment authorization.

`Cargo.lock` is deliberately generated on the target machine using the repository's current exact dependency graph before the P0.5C gate is run. It must be committed with the candidate if validation succeeds.
