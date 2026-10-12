# P0.5C decision — Closed build graph, not global kernel closure

## Certified input

P0.5B is certified on `production/p0.1` at commit:

`24fb905b341d31296403f02040d44eeba13109ce`

P0.5B established a source-level regression lock around the P0.5A fenced host facade. It intentionally left historical kernel dispatch surfaces present.

## P0.5C problem

The next question is narrower and mechanically testable: can a designated consumer be forced, by its direct Cargo dependency graph, to see only the selected fenced facade rather than the raw production adapter, kernel dispatch trait, or trust primitives?

## Decision

Add a dedicated boundary crate `nordoi-production-profile-p05c` that has exactly one normal direct dependency: `nordoi-native-effect-p03`.

The boundary crate publicly re-exports only `FencedProductionEffects` and a profile identifier. It does not publicly re-export `GuardedEffectBackend`, `ProductionEffectBackend`, `EffectBackend`, `DurableTrustGate`, or direct kernel/trust APIs.

A downstream Cargo package that declares only this boundary crate as a direct dependency is compilation-tested. Imports of the transitive P0.3, kernel and trust crates must fail. Cargo metadata is also checked so a future direct dependency addition cannot silently widen the profile graph.

## What this proves if the gate passes

It proves a selected Cargo import boundary exists whose direct dependency graph and public source surface are closed to the currently inventoried raw production types.

It also gives CI a fail-closed regression check if that boundary widens.

## What this does not prove

It does not remove historical public APIs from `nordoi_kernel` or `nordoi-native-effect-p03`.

It does not prove that every future application will choose this boundary crate.

It does not create a concrete production fencing store, destination backend, or signing-key service.

It does not solve whole-ledger rollback, recipient-side exactly-once delivery, supply-chain attestation, process sandboxing, or operational key protection.

Therefore:

- kernel-wide mandatory enforcement: **not certified**;
- production deployment authorization: **not granted**.
