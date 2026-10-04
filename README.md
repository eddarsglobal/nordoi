# NORDOI K1.17 — Governed Dynamic Timer Identity Migration & Upgrade Continuity Core

K1.17 extends the certified K1.16 timer-migration protocol to the last timer-state class that K1.16
intentionally left fail-closed: **active dynamic timers allocated through the runtime API without a
native NAIR `TimerSlot`**.

K1.16 remains unchanged and still rejects active dynamic timers on its timer-aware upgrade API.
K1.17 adds a separate, explicit full-timer upgrade path.

```text
K1.16-bound durable source runtime
        ↓
RuntimeUpgradePlan                  (atoms)
        +
RuntimeTimerUpgradePlan             (native TimerSlot timers)
        +
RuntimeDynamicTimerUpgradePlan      (active dynamic TimerId timers)
        ↓
RuntimeAllTimerUpgradePlan
        ↓
exact RuntimeUpgradeAuthority grant
        ↓
private target boot
        ↓
validate native + dynamic timer identity rules
        ↓
deterministic atom + timer migration
        ↓
full composite migration hash
        ↓
ProgramEpoch + durable upgrade lineage
        ↓
one fenced effect/runtime bundle commit
        ↓
ONLY THEN target publication
```

## K1.16 remains fail-closed

The certified K1.16 API keeps its original behavior:

```rust
AtomicEventLoop::upgrade_program_with_runtime_checkpoint_and_timers(...)
```

If an active source timer is not bound to a native `TimerSlot`, it still returns:

```text
DynamicSourceTimerUnsupported
```

Dynamic timer migration is opt-in through the new K1.17 API:

```rust
AtomicEventLoop::upgrade_program_with_runtime_checkpoint_and_dynamic_timers(...)
```

No existing caller silently gains raw-`TimerId` migration semantics.

## Dynamic timer migration rules

K1.17 adds:

```rust
DynamicTimerUpgradeRule::Carry {
    source: TimerId,
    target: TimerId,
}

DynamicTimerUpgradeRule::DropSource {
    source: TimerId,
}
```

Every **active dynamic source timer** must have exactly one disposition.

A dynamic timer that was already canceled has no remaining timer state and therefore requires no
individual disposition. Its allocation history is still protected by the preserved `next TimerId`
frontier.

## Why raw TimerId is allowed only here

K1.16 correctly states that a raw `TimerId` is not sufficient **program-level** identity. K1.17 does
not weaken that law by allowing arbitrary inference.

A raw dynamic timer identity has migration meaning only when it appears inside a canonical
`RuntimeDynamicTimerUpgradePlan` bound to:

```text
exact source program hash
exact target program hash
exact source ProgramEpoch
exact durable source runtime state
explicit source→target upgrade authority
```

There is still no ambient rule such as “matching numbers imply the same timer.”

## Preserve or remap identity

A carried dynamic timer may preserve its exact runtime identity:

```text
Carry(TimerId(7) → TimerId(7))
```

or it may explicitly move to a fresh identity:

```text
Carry(TimerId(7) → TimerId(42))
```

The carried timer preserves:

```text
next_deadline
interval
occurrences
```

Only its runtime identity changes when the plan explicitly remaps it.

`RuntimeAllTimerUpgradeReport.dynamic_timer_mappings` exposes the exact source→target mapping used
by the published upgrade. The historical `RuntimeUpgradeReport` remains unchanged and is available
as `RuntimeAllTimerUpgradeReport.upgrade`.

## Freshness law for remapped identities

If `source != target`, the target identity must be fresh relative to both source and target
allocation frontiers:

```text
target >= max(source.next_timer_id, target.next_timer_id)
```

This prevents a remap from silently reusing an old/canceled timer identity.

Exact preservation is the only exception:

```text
source == target
```

because preserving an already-live source identity is continuation, not identity reuse.

The target identity `0` is invalid and `u64::MAX` is rejected because it cannot advance the
allocation frontier safely.

## Native target identities remain reserved

A dynamic target `TimerId` may not collide with any identity bound to a target native `TimerSlot`.
This remains true even when the target native timer is canceled at bootstrap.

```text
target TimerSlot(5) → TimerId(1), then canceled

Dynamic Carry(... → TimerId(1))
        ↓
REJECT
```

The native slot still owns that identity in the target reaction topology.

## Native and dynamic timers migrate together

K1.17 does not replace K1.16 native timer migration. The full plan combines all three semantic
components:

```rust
RuntimeAllTimerUpgradePlan::new(
    &atom_plan,
    &native_timer_plan,
    &dynamic_timer_plan,
)
```

The atom plan and native timer plan keep all K1.15/K1.16 laws. The dynamic plan adds only the
explicit active-unbound timer dispositions.

## Canonical identity

`RuntimeDynamicTimerUpgradePlan` has deterministic canonical bytes ordered by `TimerId`, independent
of host iteration or rule insertion order.

K1.17 derives a full composite hash:

```text
full_plan_hash = SHA256(
    domain
    || atom_plan_hash
    || native_timer_plan_hash
    || dynamic_timer_plan_hash
)
```

That full hash becomes the semantic `plan_hash` committed by:

```text
RuntimeUpgradeLineageRecord
upgrade lineage root
runtime upgrade replay
atomic event-loop upgrade replay
```

The atom/native/composite hashes remain inspectable in the unchanged historical
`RuntimeUpgradeReport` nested as `RuntimeAllTimerUpgradeReport.upgrade`; the dynamic plan hash is
reported by the K1.17 wrapper.

## Replay semantics

Dynamic timer migration is semantic work. These two upgrades are different traces:

```text
TimerId(7) → TimerId(7)
TimerId(7) → TimerId(42)
```

They therefore produce different replay identity.

K1.17 uses:

```text
NORDOI-ATOMIC-EVENT-LOOP-1.17
NORDOI-RUNTIME-PROGRAM-UPGRADE-DYNAMIC-TIMER-1.0
```

As required by C266, external delivery/audit metadata still does not become semantic replay meaning.

## Persistence-first publication

Dynamic timer migration does not weaken K1.14/K1.15 durability:

```text
private target runtime
      +
atom migration
      +
native timer migration
      +
dynamic timer migration
      ↓
effect/audit checkpoint + runtime checkpoint
      ↓
one fenced atomic host commit
      ↓
ONLY THEN live target publication
```

If validation or persistence fails, the source program, dynamic timers, replay state, program epoch
and durable lineage remain unchanged.

## Crash recovery

K1.17 requires no new checkpoint field. The runtime checkpoint remains:

```text
magic  : NDRTSM01
format : 1.1
```

The existing time checkpoint already stores timer snapshots by `TimerId`. Therefore a successfully
migrated dynamic timer is recovered with its target identity, deadline, interval and occurrence
count.

The timer-allocation frontier is restored as well, so neither dropped nor remapped identities are
silently reallocated.

## Main API additions

```rust
DynamicTimerUpgradeRule
RuntimeDynamicTimerUpgradePlan
RuntimeAllTimerUpgradePlan
RuntimeAllTimerUpgradeReport
AtomicEventLoop::upgrade_program_with_runtime_checkpoint_and_dynamic_timers(...)
```

The new API returns `RuntimeAllTimerUpgradeReport`. Its `upgrade` field is the exact unchanged
K1.16 `RuntimeUpgradeReport`; K1.17 adds only these wrapper fields:

```text
dynamic_timer_plan_hash
migrated_dynamic_timers
dropped_dynamic_timers
dynamic_timer_mappings
```

## Compatibility guarantees

K1.17 intentionally keeps these certified formats unchanged:

```text
NAIR       = 0.6
NDRTSM01   = 1.1
```

No new NAIR opcode is introduced.

K1.15 atom-only and K1.16 native-timer-aware upgrade APIs remain available with their certified
fail-closed boundaries.

## Scope boundary

K1.17 does **not** claim:

- arbitrary timer merge/split/fan-out;
- wall-clock translation;
- automatic inference of dynamic timer meaning;
- host callback execution as migration logic;
- migration of arbitrary process resources;
- persistence of upgrade authority;
- new NAIR timer opcodes;
- frozen `.noi` syntax.

Dynamic timer migration is an explicit runtime-resource migration protocol, not a general resource
migration engine.

## Certification target

K1.17 adds **36 dynamic-timer upgrade tests** on top of the **477 tests certified by K1.16**, for an
expected total of **513 tests**.

The package must pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

and the GitHub CI matrix on Linux, macOS and Windows before tag `k1.17` may be published.

The Constitution now extends through **C304**.
