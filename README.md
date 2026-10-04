# NORDOI K1.15 — Governed Program Upgrade & Deterministic Semantic State Migration Core

K1.15 closes the program-evolution boundary made explicit by K1.14. K1.14 correctly refuses to
recover a durable runtime under different canonical NAIR bytes. K1.15 adds a separate, explicit and
persistence-first protocol for changing the program while preserving only the semantic state that a
host-authorized migration plan names.

The K1.15 path is:

```text
K1.14-bound source runtime
        ↓
explicit RuntimeUpgradePlan
        ↓
exact RuntimeUpgradeAuthority grant
        ↓
private boot of target NAIR 0.6 program
        ↓
validate complete source/target atom dispositions
        ↓
deterministic atom copy/drop/default migration
        ↓
new ProgramEpoch + SHA-256 lineage root
        ↓
co-commit effect/audit + target runtime checkpoint
        ↓
ONLY THEN publish target runtime
```

## No implicit schema guessing

Every source atom must have exactly one disposition:

```text
Copy(source → target)
DropSource(source)
```

Every target atom must be supplied exactly once by either a copy or:

```text
KeepTargetDefault(target)
```

K1.15 intentionally does not execute host closures, scripts, arbitrary code or implicit converters
inside the migration core. It does not merge or split atoms. A migration that cannot be expressed by
the certified deterministic rules must wait for a later explicitly governed migration layer.

## Explicit upgrade authority

`RuntimeUpgradeAuthority` is host-supplied and has no ambient grants. Authority is exact to:

```text
(source canonical program hash, target canonical program hash)
```

The authority is not serialized into NAIR or runtime checkpoint bytes and may be revoked by the host.
The program therefore cannot authorize its own replacement.

## Program epochs and lineage

K1.15 introduces:

```text
ProgramEpoch
RuntimeUpgradeHash
RuntimeUpgradeLineageRecord
```

Fresh boot begins at epoch 0. Every successful program upgrade increments the epoch by exactly one.
The runtime checkpoint format is revised from 1.0 to 1.1 while keeping the same `NDRTSM01` magic.
K1.15 can decode certified K1.14 1.0 checkpoints and represents them as epoch 0 with an empty upgrade
lineage.

Every successful upgrade extends a domain-separated SHA-256 lineage root over:

- the previous lineage root;
- the new program epoch;
- source canonical program hash;
- target canonical program hash;
- migration-plan hash;
- exact source semantic-checkpoint hash.

The latest transition record is retained in the runtime checkpoint for inspection and recovery.

## Replay semantics

A program upgrade is a semantic event, unlike ordinary crash recovery. K1.15 therefore changes both
runtime and event-loop replay identity using the source replay state, target boot state, migration
plan, source checkpoint and new epoch.

Equal source state + equal target program + equal migration plan produce equal upgraded replay
identity. Different migration plans produce different replay identity.

The event-loop replay domain is:

```text
NORDOI-ATOMIC-EVENT-LOOP-1.15
```

## Preserved state

K1.15 preserves across an accepted upgrade:

- logical time;
- event-loop cycle / persistent-runtime tick;
- last public input sequence;
- transaction identity frontier;
- effect outbox and `EffectIntentId` frontier;
- effect audit/retry/dead-letter state through the existing journal;
- completion source sequence frontier;
- consumed `EffectDeliveryKey` deduplication state.

The target program defines the new static ownership, dependency, render, reaction and completion
structure. Target render revisions begin from target bootstrap state.

## Timer boundary

K1.15 deliberately requires **zero pending source timers** at the upgrade boundary. Silent timer
reinterpretation would be unsafe because timer identities and timer-slot topology may change with the
program.

Target native timers are permitted only when their declared deadlines are not before the preserved
logical time. Timer migration/translation is intentionally deferred to a separately certifiable
future protocol.

## Persistence-first publication

`AtomicEventLoop::upgrade_program_with_runtime_checkpoint(...)` constructs the entire target runtime
privately. It then co-commits the existing effect/audit checkpoint and new target runtime checkpoint
under the active K1.8 fence.

```text
commit success → publish target runtime
commit failure → source runtime remains live and unchanged
```

No target program state becomes visible before durable commit.

## Runtime checkpoint format 1.1

K1.15 keeps:

```text
NDRTSM01
```

and advances the runtime-checkpoint format to `1.1`. Added fields are:

```text
ProgramEpoch
upgrade lineage root
optional latest RuntimeUpgradeLineageRecord
```

The decoder remains backward-compatible with certified K1.14 format 1.0, including the K1.14
SHA-256 domain separator. Re-encoding uses canonical format 1.1.

## API surface

Main K1.15 APIs:

```rust
RuntimeUpgradePlan
AtomUpgradeRule
RuntimeUpgradeAuthority
ProgramEpoch
RuntimeUpgradeHash
RuntimeUpgradeReport
AtomicEventLoop::upgrade_program_with_runtime_checkpoint(...)
```

Introspection additions:

```rust
AtomicEventLoop::program_hash()
AtomicEventLoop::program_epoch()
AtomicEventLoop::upgrade_chain_root()
AtomicEventLoop::last_upgrade()
RuntimeSemanticCheckpoint::program_epoch()
RuntimeSemanticCheckpoint::upgrade_chain_root()
RuntimeSemanticCheckpoint::last_upgrade()
```

## Scope boundary

K1.15 does **not** claim:

- arbitrary user-code migration inside the kernel;
- automatic field/schema inference;
- merge/split transforms;
- migration of pending source timers;
- migration of arbitrary host memory/resources;
- authority persistence;
- universal exactly-once external behavior.

It certifies an explicit deterministic atom-state migration and crash-consistent target publication
under the existing fenced runtime/effect persistence boundary.

## NAIR status

K1.15 changes runtime evolution protocol, not canonical program semantics.

```text
NAIR = 0.6
```

No new opcode is introduced and `.noi` surface syntax remains unfrozen.

## Certification target

K1.15 adds **32 tests** (30 program-upgrade integration tests + 2 checkpoint-format compatibility tests) on top of the 412
tests certified by K1.14, for an expected total of **444 tests**. The package must pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

and the GitHub CI matrix on Linux, macOS and Windows before tag `k1.15` may be published.

The Constitution now extends through **C266**.
