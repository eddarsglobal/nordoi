# P0.6B Decision — Per-Dispatch Anchor Barrier

## Decision

Integrate the already certified P0.6A rollback-witness contract inside `ProductionEffectBackend`, while preserving the closed `FencedProductionEffects` field/method surface.

`GuardedEffectBackend` is split internally into durable admission and admitted execution. Its public `EffectBackend` behavior remains verify-then-execute for direct P0.3 use. P0.4 can therefore reserve first, perform durable P0.2 admission, cross the anchor barrier, and only then invoke the real destination.

`FencedProductionEffects::new` now requires an injected `MonotonicAnchor` plus explicit `AnchorOpenMode::{Initialize, Recover}`. Recovery never initializes a missing anchor. First boot rejects an already initialized anchor.

## Required execution order

`P0.4 reserve/fsync -> P0.2 admission/fsync -> anchor advance/read-back -> external effect -> P0.4 ACK/fsync -> anchor advance/read-back`

If the first anchor barrier fails, the external effect is not called. If the second barrier fails after the effect returned success, the call returns an error but the delivery remains locally acknowledged and duplicate execution is denied.

## Scope

P0.6B certifies the placement and fail-closed use of the anchor contract in the selected fenced production path. It does not certify the physical independence, durability, monotonicity, or adversarial resistance of a concrete anchor backend.

`INDEPENDENT ANCHOR BACKEND QUALITY: NOT CERTIFIED`

`KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED`

`PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED`
