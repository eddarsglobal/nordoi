# NORDOI Runtime Timer Upgrade Continuity Specification 1.0

Status: K1.16 candidate

## 1. Purpose

K1.15 certifies deterministic atom migration between durable programs but deliberately rejects every
upgrade with pending source timers. K1.16 adds an explicit timer-state migration protocol without
weakening the K1.15 fail-closed path.

The legacy API remains unchanged:

```text
upgrade_program_with_runtime_checkpoint(...)
```

and continues to reject pending source timers.

Timer-aware upgrades use:

```text
upgrade_program_with_runtime_checkpoint_and_timers(...)
```

with a validated `RuntimeTimerAwareUpgradePlan` built from the atom `RuntimeUpgradePlan` and a
separate `RuntimeTimerUpgradePlan`.

## 2. Timer migration plan

Every source native `TimerSlot` has exactly one explicit disposition:

```text
Carry(source TimerSlot -> target TimerSlot)
DropSource(source TimerSlot)
```

Every target native `TimerSlot` is supplied exactly once by one `Carry` or by:

```text
KeepTargetDefault(target TimerSlot)
```

Duplicate, missing or unknown slot dispositions fail closed.

The timer plan binds exact source/target canonical program hashes and the exact source `ProgramEpoch`.
`RuntimeTimerAwareUpgradePlan::new(...)` requires these identities to match the atom plan before the
combined upgrade can be submitted. The timer plan has deterministic canonical bytes and a domain-separated SHA-256 plan hash independent of rule
input order.

## 3. Native timer scope

K1.16 migrates native timers that are represented by NAIR `TimerSlot` declarations. A pending runtime
timer that is not present in the source event loop's native timer-binding map is rejected as a
`DynamicSourceTimerUnsupported` boundary.

A dynamically allocated timer that was already canceled may leave only its identity frontier. The
frontier is preserved through `next_timer_id`, so its identity is never silently reused after the
program upgrade.

## 4. Carry semantics

A carried active timer preserves exactly:

```text
next_deadline
interval
occurrences
```

but adopts the target runtime's `TimerId` associated with the target `TimerSlot`.

This is essential because native target reactions are bootstrapped against target timer identities.
Preserving the source `TimerId` would make the target program's timer/reaction topology inconsistent.

A source timer that is no longer pending is treated as canceled. Carrying that source slot removes
the target timer from the target time state, preserving cancellation instead of resurrecting work.

## 5. Shape compatibility

K1.16 does not reinterpret timer kind or cadence.

```text
one-shot -> one-shot                     allowed
repeating(interval X) -> repeating(X)    allowed
one-shot -> repeating                    rejected
repeating -> one-shot                    rejected
repeating(X) -> repeating(Y), X != Y     rejected
```

A pending active source timer cannot be carried into a target timer slot that the target bootstrap
itself canceled, because there is no active target timer shape to receive the carried state.

## 6. Logical-time boundary

A source pending timer must have `next_deadline >= preserved LogicalTime`.

For `KeepTargetDefault`, the K1.15 rule remains: the target default timer may not begin in the
preserved past.

For `Carry`, the source timer state replaces the target bootstrap state before the target time core is
restored. Therefore a target bootstrap deadline that is now in the past is harmless when an explicit
`Carry` overwrites it with a valid source deadline.

## 7. Identity frontiers

K1.16 preserves:

```text
LogicalTime
next TimerId frontier
ProgramEpoch progression
event-loop cycle/tick
input frontier
completion deduplication
effect outbox / EffectIntentId frontier
```

The target timer uses its target-native `TimerId`; future timer allocation starts at the maximum of
the source and target `next_timer_id` frontiers.

## 8. Replay and lineage

The K1.15 atom plan hash remains independently inspectable. K1.16 additionally computes a timer-plan
hash and then a domain-separated composite migration hash:

```text
composite = SHA256(domain || atom_plan_hash || timer_plan_hash)
```

For a timer-aware upgrade, the composite hash is the migration identity committed into:

```text
RuntimeUpgradeLineageRecord.plan_hash
upgrade lineage root
runtime replay transition
event-loop replay transition
```

This means different timer migration semantics cannot accidentally share replay identity even when
atom migration is identical.

The exact source checkpoint hash remains durable-lineage evidence only and does not become replay
meaning, preserving C266.

## 9. Persistence-first publication

Timer-aware upgrade follows the same K1.15 publication law:

```text
source live runtime
      ↓
private target boot
      ↓
atom migration
      ↓
timer migration
      ↓
new epoch + lineage + replay
      ↓
combined fenced effect/runtime commit
      ↓
ONLY THEN publish target runtime
```

Any validation or persistence failure preserves the live source program and source timer state.

## 10. Checkpoint compatibility

K1.16 does not change runtime checkpoint structure.

```text
magic: NDRTSM01
format: 1.1
```

K1.15 checkpoints remain readable. For timer-aware K1.16 upgrades, the already-existing
`RuntimeUpgradeLineageRecord.plan_hash` stores the composite migration hash rather than requiring a
new checkpoint field.

## 11. Non-goals

K1.16 does not certify:

- migration of pending dynamic/unbound timers;
- interval conversion;
- one-shot/repeating conversion;
- arbitrary timer merge/split/fan-out;
- wall-clock translation;
- host callback execution during migration;
- new NAIR opcodes;
- frozen `.noi` syntax.

## 12. NAIR status

NAIR remains format 0.6. Timer migration is a governed runtime-upgrade protocol over already-certified
native timer semantics, not a new canonical program instruction.
