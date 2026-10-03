# NORDOI K1.12 — Governed Effect Completion Re-entry Core

K1.12 closes the semantic loop deliberately left open since K1.6: an external effect result may
influence NORDOI state only by re-entering as an explicit governed cause. A host callback never
receives a privileged direct-NAM mutation path.

The governed re-entry path is:

```text
K1.10/K1.11 governed effect delivery
        ↓
external system produces a result
        ↓
EffectCompletionBatch
        ↓
exact source + namespace authority
        ↓
K1.10 audit correlation
        ↓
canonical ordering + deduplication
        ↓
pre-registered completion projections
        ↓
normal NAM transactions
        ↓
render flush / quiescence
        ↓
atomic event-loop publication
```

## What K1.12 adds

- `EffectCompletionSourceId`;
- `EffectCompletionSequence`;
- `EffectCompletionOutcome::{Success, Failure}`;
- `EffectCompletion` and `EffectCompletionBatch`;
- exact `EffectCompletionAuthority` grants over `(source, namespace)`;
- `EffectCompletionProjection` and projection-value selectors;
- `AtomicEffectCompletionCore`;
- per-source monotonic sequence tracking;
- once-per-`EffectDeliveryKey` semantic deduplication;
- K1.10 audit correlation by `EffectAttemptId` + `EffectDeliveryKey`;
- only `AttemptDelivered` or `InDoubtAssumedDelivered` may re-enter semantics;
- canonical completion-batch ordering;
- bounded text/batch input and finite-float validation;
- ordinary NAM transaction projection with ownership enforcement;
- whole-batch candidate rollback;
- completion processing before render flush;
- completion causes in deterministic replay identity;
- stale fenced journal ownership rejected before re-entry;
- combined audited-journal + completion cycle surface;
- no new Rust dependency;
- NAIR remains 0.5.

## Completion is a cause, not a callback mutation

The host may submit a completion object, but the host does not receive the kernel or an unrestricted
atom setter. The core validates authority, delivery history and projection policy, then applies the
result through normal NAM transactions.

```text
host result
   ↓
validate
   ↓
plan projection
   ↓
AtomicTransaction
   ↓
commit candidate
```

If validation or any transaction fails, the published event loop remains unchanged.

## Exact source authority

Completion sources have no authority by default. The host must explicitly grant:

```text
EffectCompletionSourceId + EffectDeliveryNamespace
```

A grant for one delivery namespace does not authorize the same source for another namespace.
Revocation is immediate for future completion cycles.

## Audit correlation

Every completion binds:

```text
EffectDeliveryKey
EffectAttemptId
EffectCompletionSourceId
EffectCompletionSequence
EffectCompletionOutcome
```

The completion core scans the certified K1.10 audit history and requires the exact attempt to have
been prepared with the same delivery key. The attempt must then have terminated as either:

```text
AttemptDelivered
or
InDoubtAssumedDelivered
```

Retry-scheduled, dead-lettered, unknown and unresolved in-doubt attempts cannot manufacture a
semantic result.

## Semantic deduplication

`EffectCompletionSequence` is monotonic per source. In addition, one `EffectDeliveryKey` can produce
at most one accepted semantic completion in a live event-loop state.

These are distinct rules:

```text
source sequence  → protects source stream ordering/replay
stable delivery key → protects semantic effect-result uniqueness
```

A later completion for the same delivery key is rejected even if it has a newer source sequence or
a different physical attempt.

## Canonical ordering

Completion batches are canonicalized by source, sequence, delivery key and attempt identity before
application. Duplicate `(source, sequence)` entries fail the entire batch.

Equal completion causes under equal authority/projection configuration therefore produce equal
application ordering and equal replay identity.

## Projection model

K1.12 intentionally does not add native NAIR completion triggers yet. Host policy pre-registers
projections from one exact `(source, namespace)` route to owned NAM atoms.

Supported values are:

```text
OutcomeValue
Succeeded
IntentId
AttemptId
SourceSequence
```

Projection registration validates current atom ownership. Application validates ownership again,
so an ownership transfer cannot silently preserve obsolete completion write authority.

## Atomic runtime phase

The persistent runtime phase is now conceptually:

```text
1. canonical input
2. input bridge
3. native Input reactions
4. native Timer reactions
5. governed Effect completions
6. NAM/render flush
7. quiescence
8. publication
```

The completion core and kernel are evaluated on candidates. A failure in any completion in the batch
publishes none of the batch, no time progress, no runtime progress and no completion sequence/dedup
state.

## Replay semantics

Unlike K1.8–K1.11 delivery/trust metadata, an accepted completion is program-visible semantic input.
Its canonical cause and projected writes therefore participate in persistent-runtime replay identity,
and consequently in the event-loop replay key.

K1.12 bumps the event-loop replay domain to:

```text
NORDOI-ATOMIC-EVENT-LOOP-1.12
```

## Fencing

The high-level event-loop completion surface requires an active K1.10 audited effect journal lease.
A stale writer that lost the K1.8 fence cannot continue accepting external completions into its local
semantic runtime.

## Durability boundary

K1.12 does **not** claim whole-runtime crash durability. Completion sequence/dedup state and the NAM
changes produced by a completion are live event-loop state. K1.10/K1.11 continue to durably protect
effect delivery/audit/trust state, but a future whole-runtime persistence milestone is required to
atomically recover completion-consumption state together with NAM after process loss.

This limitation is explicit rather than hidden behind an exactly-once claim.

## NAIR

K1.12 establishes completion re-entry laws below NAIR first.

**NAIR remains 0.5.**

A later milestone may encode native completion triggers/projections only after this core is certified.

## Tests

K1.12 adds **39 completion/re-entry tests** on top of the **323 certified K1.11 tests**, for an expected corpus of **362 tests**. Coverage includes:

- exact source authority and revocation;
- zero/duplicate/non-monotonic source sequences;
- stable-delivery-key deduplication;
- unknown/mismatched/not-delivered attempts;
- assumed-delivered reconciliation;
- projection existence and duplicate projection rejection;
- ownership enforcement;
- success/failure and identity projections;
- multi-domain transactions;
- full-batch rollback;
- bounded/non-finite input rejection;
- canonical replay bytes;
- event-loop replay inclusion;
- no cycle publication on rejected completion;
- stale-fence rejection;
- persistence-first audited cycle publication.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.12 is certified only after the local release gate and cross-platform GitHub CI are fully green,
followed by publication of the `k1.12` tag.
