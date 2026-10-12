# Production P0.6A Decision — Rollback Witness + Monotonic Anchor Contract

## Decision

Create an explicit, separately trusted monotonic-anchor contract without pretending that a second local file is independent.

P0.2 and P0.4 expose bounded witness values containing the current byte position and chain digest. During initialization/replay they build an in-memory index of every verified prefix. After each successful fsync-backed append, the new prefix witness is added. The index is bounded indirectly by the existing 16 MiB journal/ledger limits.

The P0.6A crate binds both chains into one atomically compare-and-advanced `ProductionAnchorRecord` with a monotonic generation. It supplies only the generic contract and recovery algorithm; it does not supply a production anchor backend.

To preserve P0.5C, `nordoi-rollback-anchor-p06` has **no normal Cargo dependency** on the raw P0.1/P0.3 crates or the kernel. The library consumes only the narrow `ProductionWitnessSource` abstraction. Concrete P0.2/P0.4 adapters are dev-only test code in P0.6A; a deployable adapter must be integrated behind the selected protected profile in P0.6B.

## Recovery rules

Given anchored witness A and current local witness L for each chain:

1. `L.position < A.position` => rollback, deny.
2. Equal positions with different digests => fork, deny.
3. `L.position > A.position` => require A to be a verified prefix of L; otherwise deny.
4. Missing anchor during recovery => deny.
5. If either chain is legitimately ahead, compare-and-advance the combined record and read it back.
6. A failed/conflicting/corrupt/unavailable anchor => deny.

This handles the crash window where a local fsync succeeded but the external anchor had not yet advanced: on recovery, the local chain may catch the anchor up only by proving ancestry.

## What P0.6A proves

- P0.2 can expose and verify historical chain witnesses from its already authenticated replay.
- P0.4 can expose and verify historical delivery-ledger witnesses from its already authenticated replay.
- The generic P0.6A contract can consume those witnesses without acquiring a normal raw production dependency.
- A preserved independent anchor can detect restoration of an older valid local state.
- Missing, ahead, forked, conflicting, or uncommitted anchors fail closed under the P0.6A contract.
- The P0.5C closed build-graph rule remains intact.

## What P0.6A does not prove

- No real independent monotonic storage backend is selected or certified.
- No TPM, Secure Enclave, cloud KMS, remote quorum, append-only transparency service, or other external anchor is implied.
- P0.6A does not yet advance the anchor between local durable admission and every external effect.
- P0.6A does not yet provide the deployable protected-profile adapter; that is P0.6B work.
- Historical kernel bypass APIs still exist outside the selected production profile.
- Recipient-side exactly-once delivery is not claimed.

P0.6B must integrate a genuinely independent anchor implementation and the protected witness adapter into the dispatch ordering, then prove `local durable commit -> anchor durable advance -> external effect` for the selected production profile.

**INDEPENDENT MONOTONIC ANTI-ROLLBACK: NOT CERTIFIED**

**KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED**

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
