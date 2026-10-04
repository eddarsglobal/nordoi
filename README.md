# NORDOI K1.16 — Governed Timer State Migration & Upgrade Continuity Core

K1.16 closes the timer boundary deliberately left fail-closed by K1.15. K1.15 can evolve a durable
program only when the source has zero pending timers. K1.16 adds a second, explicit timer-aware
upgrade path that can preserve native timer continuity without weakening the original K1.15 API.

The K1.16 path is:

```text
K1.15/K1.16-bound source runtime
        ↓
RuntimeUpgradePlan (atoms)
        +
RuntimeTimerUpgradePlan (native timers)
        ↓
RuntimeTimerAwareUpgradePlan (validated composite)
        ↓
exact RuntimeUpgradeAuthority grant
        ↓
private boot of target NAIR 0.6 program
        ↓
validate complete atom + timer dispositions
        ↓
deterministic atom migration
        ↓
deterministic timer carry/drop/default migration
        ↓
composite migration hash
        ↓
ProgramEpoch + SHA-256 upgrade lineage
        ↓
co-commit effect/audit + target runtime checkpoint
        ↓
ONLY THEN publish target runtime
```

## Legacy K1.15 upgrade remains fail-closed

This API keeps its certified K1.15 behavior:

```rust
AtomicEventLoop::upgrade_program_with_runtime_checkpoint(...)
```

If the source has pending timers, it still returns `PendingTimersUnsupported`.

Timer migration is opt-in through:

```rust
AtomicEventLoop::upgrade_program_with_runtime_checkpoint_and_timers(...)
```

No existing caller silently gains new timer semantics.

## Explicit timer dispositions

Every source native `TimerSlot` must be named exactly once as:

```text
Carry(source → target)
DropSource(source)
```

Every target native `TimerSlot` must be supplied exactly once either by a carry or by:

```text
KeepTargetDefault(target)
```

Missing, duplicate or unknown source/target timer slots fail closed.

## Semantic continuity vs runtime identity

A carried active timer preserves:

```text
next_deadline
interval
occurrences
```

but it adopts the `TimerId` already bound to the **target** `TimerSlot`.

This separation is critical:

```text
source semantic timer progress
        ≠
source runtime TimerId
```

The new target program owns its reaction topology. Carrying the old runtime ID would bind target
reactions to the wrong timer identity.

If the source timer slot is already canceled, carrying that slot cancels the target timer too. K1.16
does not resurrect target bootstrap work when source semantics say the timer no longer exists.

## Timer shape compatibility

K1.16 intentionally does not perform timer conversion:

```text
one-shot → one-shot                       allowed
repeating(5) → repeating(5)              allowed
one-shot → repeating                     rejected
repeating → one-shot                     rejected
repeating(5) → repeating(7)              rejected
```

A pending active source timer also cannot be carried into a target timer slot canceled by the target
bootstrap.

## Logical-time rules

A carried source timer must have a valid remaining deadline:

```text
source.next_deadline >= preserved LogicalTime
```

A `KeepTargetDefault` timer continues to obey K1.15:

```text
target default deadline >= preserved LogicalTime
```

But an explicit `Carry` may replace a target bootstrap deadline that has become old. The target
bootstrap state is private and overwritten before publication.

## Dynamic timer boundary

K1.16 migrates timers identified by native NAIR `TimerSlot` bindings. A **pending** timer allocated
through the runtime API without a native slot is rejected:

```text
DynamicSourceTimerUnsupported
```

No attempt is made to infer program meaning from a raw `TimerId`.

A dynamic timer that was allocated and later canceled does not block migration; its allocation
frontier still survives. K1.16 preserves:

```text
next TimerId = max(source frontier, target frontier)
```

so no old identity is silently reused.

## Canonical timer plan identity

K1.16 adds:

```rust
RuntimeTimerUpgradePlan
RuntimeTimerAwareUpgradePlan
TimerUpgradeRule
```

`RuntimeTimerAwareUpgradePlan::new(&atom_plan, &timer_plan)` verifies that both plans bind the same
source program, target program and source epoch, then freezes the composite migration hash used by
the runtime upgrade call.

The timer plan has canonical order-independent bytes and a domain-separated SHA-256 hash.

For a timer-aware upgrade, NORDOI derives:

```text
composite_plan_hash = SHA256(
    domain
    || atom_plan_hash
    || timer_plan_hash
)
```

The composite migration hash is used for:

```text
RuntimeUpgradeLineageRecord.plan_hash
upgrade lineage root
runtime replay transition
event-loop replay transition
```

The atom plan hash and timer plan hash remain separately visible in `RuntimeUpgradeReport`.

## Replay semantics

Different timer migration semantics are semantic differences and therefore must change replay
identity even when atom migration is identical.

K1.16 uses:

```text
NORDOI-ATOMIC-EVENT-LOOP-1.16
NORDOI-RUNTIME-PROGRAM-UPGRADE-1.1
```

The replay transition includes the composite migration identity. As required by C266, backend
receipts, effect audit roots, writer/fence metadata and the exact source-checkpoint digest remain
outside semantic replay meaning.

## Persistence-first publication

The target runtime and migrated time state remain private until the combined fenced persistence
boundary succeeds:

```text
private target program
      +
migrated atom state
      +
migrated timer state
      ↓
effect/audit checkpoint + runtime checkpoint
      ↓
one fenced host commit
      ↓
ONLY THEN live publication
```

A failed commit preserves source program hash, source epoch, source replay state and source timer
state exactly.

## Crash recovery

K1.16 does not add a new runtime-checkpoint field. The checkpoint remains:

```text
magic: NDRTSM01
format: 1.1
```

A timer-aware upgrade stores the composite migration hash in the already-certified latest-upgrade
`plan_hash` field. Therefore certified K1.15 checkpoints remain readable without a new format
revision.

After successful target publication, the normal K1.14/K1.15 recovery path restores the migrated timer
state, including the target timer ID, remaining deadline, interval and occurrence count.

## Main API additions

```rust
RuntimeTimerUpgradePlan
RuntimeTimerAwareUpgradePlan
TimerUpgradeRule
AtomicEventLoop::upgrade_program_with_runtime_checkpoint_and_timers(...)
```

`RuntimeUpgradeReport` now also exposes:

```text
timer_plan_hash
composite_plan_hash
migrated_timers
dropped_timers
defaulted_timers
```

The existing atom-only upgrade API remains available and keeps K1.15 semantics.

## Scope boundary

K1.16 does **not** claim:

- migration of pending dynamic/unbound timers;
- conversion between one-shot and repeating timers;
- repeating interval conversion;
- timer merge/split/fan-out;
- wall-clock translation;
- migration through arbitrary host callbacks;
- persistence of upgrade authority;
- new NAIR timer instructions;
- frozen `.noi` syntax.

## NAIR status

K1.16 governs runtime continuity for timer semantics already native in NAIR.

```text
NAIR = 0.6
```

No new opcode is introduced.

## Certification target

K1.16 adds **33 timer-upgrade tests** on top of the **444 tests certified by K1.15**, for an expected
total of **477 tests**.

The package must pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

and the GitHub CI matrix on Linux, macOS and Windows before tag `k1.16` may be published.

The Constitution now extends through **C284**.
