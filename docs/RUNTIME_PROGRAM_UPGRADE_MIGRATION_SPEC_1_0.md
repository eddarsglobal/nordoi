# NORDOI Runtime Program Upgrade & Semantic Migration Specification 1.0

Status: K1.15 candidate.

## 1. Purpose

K1.14 binds a durable semantic checkpoint to the exact canonical NAIR program that produced it.
That rule remains intact. K1.15 does not weaken `ProgramMismatch`; instead it adds a distinct,
explicit transition protocol from one canonical program identity to another.

## 2. Preconditions

An upgrade requires:

1. a source `AtomicEventLoop` with an established K1.14 durable lineage;
2. an active fenced audited effect journal;
3. an explicit target NAIR program;
4. target reaction/completion authority supplied by the host;
5. an explicit `RuntimeUpgradeAuthority` grant for the exact source→target program hashes;
6. a `RuntimeUpgradePlan` whose source epoch equals the live source epoch;
7. zero pending source timers.

No precondition is inferred from filenames, build versions or host process identity.

## 3. Atom migration

Each source atom slot SHALL appear exactly once as either:

- `Copy { source, target }`; or
- `DropSource { source }`.

Each target atom slot SHALL appear exactly once as either the target of a `Copy` or as:

- `KeepTargetDefault { target }`.

Copy is one-to-one. K1.15 does not merge, fan out, split or transform values. The target program
retains its own atom identity and ownership; only the semantic `Value` is copied. The migrated target
atom version becomes strictly greater than both its target bootstrap version and copied source
version.

## 4. Static structure

The target program is booted normally. Its domains, ownership, dependency graph, render graph,
input bridges, reactions and completion routes are the target static structure. K1.15 does not copy
those structures from the source runtime.

## 5. Mutable state continuity

K1.15 carries forward:

- cycle/tick;
- logical time;
- last input sequence;
- transaction identity frontier;
- completion source sequence/delivery-dedup state;
- effect outbox and effect identity frontier.

Target render revisions start from the target bootstrap graph. Target native timers also originate
from target bootstrap, but their deadlines must not be before the preserved logical time.

## 6. Timer rule

Pending source timers make the upgrade fail. K1.15 refuses to guess whether a source timer should be
renamed, dropped, merged or reinterpreted by the target program. A later milestone may introduce a
separate stable timer-migration protocol.

## 7. Program epoch

Fresh boot starts at `ProgramEpoch(0)`. Every successful upgrade increments the epoch by exactly one.
Epoch exhaustion fails closed.

Runtime-local identities that are reconstructed by target bootstrap are interpreted inside the new
program epoch. Externally durable `EffectIntentId` / `EffectDeliveryKey` identity does not reset.

## 8. Lineage

Each successful transition derives:

```text
SHA256(
  "NORDOI-RUNTIME-UPGRADE-LINEAGE-1.0" ||
  previous_lineage_root ||
  target_epoch ||
  source_program_hash ||
  target_program_hash ||
  migration_plan_hash ||
  source_runtime_checkpoint_hash
)
```

The target checkpoint retains that root and the latest transition record.

## 9. Replay

Upgrade is semantic work. It changes replay identity. Recovery is not an upgrade and still does not
add a synthetic replay cause.

## 10. Durable publication

The target runtime remains private until `commit_effect_and_runtime_fenced(...)` succeeds under the
active lease. The effect checkpoint used in the combined commit is the existing current audited
journal state with the preserved source outbox.

If commit fails, the source runtime remains unchanged.

## 11. Runtime checkpoint format

`NDRTSM01` advances from format 1.0 to 1.1. The 1.1 decoder accepts 1.0 and verifies legacy bytes
with the legacy K1.14 digest domain. A legacy 1.0 checkpoint maps to epoch 0 / zero lineage / no last
upgrade record.

## 12. Authority

`RuntimeUpgradeAuthority` is host-injected. It is not serialized in NAIR or runtime state and has no
default grants. The program cannot authorize its own replacement.

## 13. Non-goals

K1.15 does not provide arbitrary migration code, timer migration, host resource migration, hidden
schema inference or universal external exactly-once behavior.
