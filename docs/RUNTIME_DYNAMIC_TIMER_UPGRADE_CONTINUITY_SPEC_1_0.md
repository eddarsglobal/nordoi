# NORDOI Runtime Dynamic Timer Upgrade Continuity Specification 1.0

Status: K1.17 candidate

## 1. Purpose

K1.17 governs active timers that were allocated through the runtime API and therefore do not possess
a native NAIR `TimerSlot` identity.

K1.16 deliberately rejects those timers. K1.17 adds a separate protocol rather than weakening the
certified K1.16 path.

## 2. Preconditions

A K1.17 dynamic-timer-aware upgrade requires all K1.15/K1.16 preconditions:

- established durable runtime lineage;
- active fenced writer lease;
- exact source and target canonical program hashes;
- exact source `ProgramEpoch`;
- explicit host `RuntimeUpgradeAuthority` for the source→target pair;
- complete atom migration plan;
- complete native timer migration plan;
- canonical dynamic timer migration plan.

## 3. Dynamic source set

For the durable source checkpoint:

```text
active_dynamic_timers =
    active timer snapshots
    MINUS all TimerId values bound to source native TimerSlot declarations
```

Every member of that set MUST have one explicit dynamic disposition.

Canceled dynamic timers have no active snapshot and therefore require no rule. Their allocation
history remains protected by the source `next_timer_id` frontier.

## 4. Rules

The canonical dynamic plan supports:

```text
Carry(source TimerId → target TimerId)
DropSource(source TimerId)
```

Duplicate source dispositions and duplicate carried target identities fail closed.

A rule that references a source native timer identity fails closed. A rule that references an
unknown/inactive dynamic source timer also fails closed.

## 5. Target identity rules

Target `TimerId(0)` is invalid.

`TimerId(u64::MAX)` cannot be used as a carried target because a safe successor allocation frontier
cannot be represented.

A carried dynamic target identity MUST NOT equal any target native `TimerSlot` binding, even when the
native timer was canceled by target bootstrap.

If the dynamic timer is remapped (`source != target`), target identity MUST satisfy:

```text
target >= max(source.next_timer_id, target.bootstrap.next_timer_id)
```

This makes the remapped identity fresh.

If `source == target`, exact identity preservation is permitted even though that identity lies below
the allocation frontier.

## 6. State continuity

A carried dynamic timer preserves exactly:

```text
next_deadline
interval
occurrences
```

The target timer snapshot uses the target identity declared by the plan.

K1.17 does not convert timer cadence or synthesize elapsed firings during upgrade.

## 7. Allocation frontier

After migration:

```text
target.next_timer_id = max(
    source.next_timer_id,
    target.bootstrap.next_timer_id,
    every carried dynamic target id + 1
)
```

Therefore old/canceled source identities and freshly remapped target identities are never
silently reused.

## 8. Composite plan identity

The dynamic timer plan has a domain-separated SHA-256 hash over:

```text
source program hash
target program hash
source ProgramEpoch
ordered dynamic dispositions
```

The full K1.17 migration identity is:

```text
SHA256(
    K1.17 full-composite domain
    || atom plan hash
    || native timer plan hash
    || dynamic timer plan hash
)
```

The full composite identity is committed into the existing durable upgrade-lineage `plan_hash`.

## 9. Replay

K1.17 dynamic timer upgrades use the replay domain:

```text
NORDOI-RUNTIME-PROGRAM-UPGRADE-DYNAMIC-TIMER-1.0
```

Different dynamic timer mappings MUST produce different semantic replay identity.

Audit roots, delivery receipts, effect fences, writer identities and the source checkpoint digest
remain excluded from semantic replay meaning.

## 10. Durability

The target remains private until the existing K1.14 fenced combined effect/runtime bundle commit
succeeds.

Failure before that boundary publishes no program change and no dynamic timer migration.

## 11. Recovery

No checkpoint schema extension is needed. `NDRTSM01` format 1.1 already stores all active timer
snapshots and the allocation frontier.

After successful K1.17 publication, ordinary exact-program recovery restores the migrated dynamic
timers under their target identities.

## 12. Compatibility

K1.17 preserves:

```text
NAIR 0.6
NDRTSM01 1.1
K1.15 atom-only upgrade API
K1.16 native-timer-aware upgrade API
```

The K1.16 API continues to reject active dynamic timers.

The K1.17 API returns `RuntimeAllTimerUpgradeReport`. The already-certified `RuntimeUpgradeReport`
shape is not extended or changed; it is nested unchanged in the wrapper as `upgrade`, while the
wrapper reports the dynamic plan hash, dynamic migration counts and exact source→target identity
mapping.

## 13. Non-goals

K1.17 does not certify:

- automatic semantic inference from raw timer numbers;
- arbitrary timer transformation code;
- timer merge/split/fan-out;
- wall-clock migration;
- general process-resource migration;
- persisted upgrade authority;
- `.noi` surface syntax.
