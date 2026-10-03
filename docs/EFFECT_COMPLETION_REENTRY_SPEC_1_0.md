# NORDOI Effect Completion Re-entry Specification 1.0

Status: K1.12 candidate specification.

## 1. Scope

This specification defines how results originating outside NORDOI may re-enter deterministic program
semantics. It does not define transport protocols, callback servers, provider SDKs or native NAIR
syntax.

## 2. Identities

A completion contains:

- `EffectCompletionSourceId`: explicit non-zero host source identity;
- `EffectCompletionSequence`: explicit non-zero sequence monotonic per source;
- `EffectDeliveryKey`: stable K1.7 semantic delivery identity;
- `EffectAttemptId`: K1.10 physical attempt identity;
- `EffectCompletionOutcome`: success/failure plus bounded NORDOI `Value` payload.

These identities are not interchangeable.

## 3. Authority

Authority is deny-by-default and exact over `(EffectCompletionSourceId,
EffectDeliveryNamespace)`. No source gains ambient authority from process identity, network origin,
thread identity or callback registration alone.

## 4. Audit correlation

The completion core MUST locate a matching K1.10 `AttemptPrepared` record and MUST verify that its
`EffectDeliveryKey` exactly equals the completion key. The same attempt MUST have a terminal audit
record of `AttemptDelivered` or `InDoubtAssumedDelivered`.

Other terminal states do not authorize semantic re-entry.

## 5. Ordering

A batch is canonicalized by `(source, sequence, delivery key, attempt)`. Duplicate source/sequence
pairs are invalid. Each accepted source sequence MUST be greater than that source's previously
accepted sequence.

## 6. Deduplication

One stable `EffectDeliveryKey` may be accepted as a semantic completion at most once in one live
completion-core state. Physical retries do not create a second semantic entitlement.

## 7. Projection

A projection is pre-registered for an exact `(source, namespace)` route and contains:

- expected ownership domain;
- target atom;
- projection selector.

Selectors are `OutcomeValue`, `Succeeded`, `IntentId`, `AttemptId`, and `SourceSequence`.
Registration and application both validate ownership.

## 8. Atomicity

The entire batch is evaluated on cloned completion-core and NAM candidates. Every projection uses a
normal `AtomicTransaction`. Failure at any point discards all candidate writes and completion-state
advancement.

## 9. Runtime phase

Completion application occurs after input/timer reactions and before NAM/render flush. Completion
writes therefore participate in the same invalidation/render frontier as other state writes.

## 10. Replay

Accepted completion application reports are canonically encoded into the persistent runtime replay
state. This includes source, sequence, delivery key, attempt, outcome and exact projected writes.

## 11. Fencing

The high-level event-loop completion APIs require an active governed audited-effect journal. A stale
K1.8 lease fails before semantic completion publication.

## 12. Bounds

- batch maximum: 4096 completions;
- text payload maximum: 1 MiB;
- float payloads must be finite;
- `u64` identity projections must fit in `Value::Int` (`i64`).

## 13. Non-goals

K1.12 does not provide whole-runtime persistence, destination exactly-once semantics, transport
identity authentication, callback networking, native NAIR completion triggers or `.noi` syntax.
