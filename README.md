# NORDOI K1.10 — Durable Effect Attempt Audit & In-Doubt Recovery Protocol

K1.10 extends the certified K1.9 retry/dead-letter protocol with a durable, hash-chained record of
external delivery attempts and an explicit recovery state for the crash window between remote I/O
and durable outcome publication.

The governed path becomes:

```text
canonical cause
     ↓
NAIR 0.5 native reaction
     ↓
validated EffectIntent
     ↓
AtomicEffectOutbox
     ↓
K1.9 retry/dead-letter state
     ↓
K1.10 AttemptPrepared durable commit
     ↓
external EffectBackend
     ↓
terminal audit + delivery-state candidate
     ↓
fenced durable commit
     ↓
local publication
```

## What K1.10 adds

- `EffectAttemptId` monotonic attempt identity;
- `EffectAuditSequence` monotonic audit ordering;
- `EffectAuditHash` SHA-256 chain identity;
- `EffectAuditEvent` / `EffectAuditRecord` append-only audit model;
- `EffectInDoubtAttempt` as an explicit unresolved crash-window state;
- `EffectAuditLedger` with deterministic hash chaining;
- `EffectAuditCheckpoint` canonical format `NDEFXA01`;
- migration from K1.9 `NDEFXR01` and K1.7/K1.8 `NDEFXJ01` checkpoints;
- `GovernedAuditedEffectJournal` integrating K1.8 fencing and K1.9 retry policy;
- prepare-before-I/O persistence;
- explicit `assume delivered` resolution after external reconciliation;
- explicit retry authorization for in-doubt delivery, preserving the original delivery key;
- audited dead-letter redrive/discard;
- no automatic redispatch while an attempt is in-doubt;
- zero new Rust dependencies;
- NAIR remains 0.5.

## Why K1.10 exists

K1.9 correctly persists retry/dead-letter state after a backend outcome, but no generic software
stack can atomically commit one transaction across NORDOI's local journal and an arbitrary remote
HTTP service, database, file system or process.

The critical failure window is:

```text
remote side effect succeeds
        ↓
process crashes / durable commit fails
        ↓
local journal still sees the intent as pending
```

Blindly retrying that intent can duplicate the external effect. Pretending the operation completed
can lose it. K1.10 therefore refuses to guess.

## Prepare before external I/O

Before calling an external backend, K1.10 commits an `AttemptPrepared` audit record containing:

```text
attempt ID
full queued intent
stable EffectDeliveryKey
current EffectDeliveryFence
explicit EffectRetryTick
previous failed-attempt count
```

Only after that commit succeeds may the backend run.

If prepare persistence fails, backend execution is zero.

## Terminal outcomes

A successfully persisted attempt ends with exactly one terminal audit event:

```text
AttemptDelivered
AttemptRetryScheduled
AttemptDeadLettered
InDoubtAssumedDelivered
InDoubtRetryAuthorized
```

If the backend runs but the terminal commit fails, `AttemptPrepared` remains the last durable event.
That is an explicit in-doubt attempt.

## In-doubt recovery

Recovery reconstructs the audit chain. If the final durable event is an unresolved
`AttemptPrepared`, K1.10 exposes `EffectInDoubtAttempt` and blocks automatic dispatch.

The host must choose one of two explicit operations after reconciliation:

```text
assume delivered
    remove the pending intent without another backend call

retry authorized
    keep the same pending intent and allow a later backend call
    with the same EffectDeliveryKey
```

Retry authorization is intentionally explicit because it may create a duplicate at a destination
that does not honor idempotency.

## Hash-chained audit

Every record hashes:

```text
domain separator
sequence
previous record hash
canonical event bytes
```

using SHA-256. The checkpoint itself is also SHA-256 protected.

This makes accidental or unauthorized mutation detectable when the attacker cannot also replace the
trusted checkpoint/root reference. It is not a digital signature and does not by itself prove who
wrote a record. Signed attestations remain a separate future layer.

## Ownership transfer

`EffectDeliveryKey` and `EffectDeliveryFence` retain distinct meanings:

```text
EffectDeliveryKey   = stable semantic request identity
EffectDeliveryFence = current writer epoch
EffectAttemptId     = one durable delivery attempt
```

After takeover, an in-doubt intent can be explicitly authorized for retry. Its delivery key remains
stable while the newly prepared attempt carries the newer fence.

## Replay law

Audit sequences, attempt IDs, receipt references, in-doubt state and audit hashes are delivery
metadata. They do not change deterministic program replay identity.

Recovered semantic pending outbox state continues to affect recovery replay identity under the
certified K1.7 rule.

## NAIR law

**NAIR remains 0.5 in K1.10.**

Attempt audit, receipt metadata and in-doubt resolution are runtime/host delivery protocol, not new
program instructions or authority.

## Tests

K1.10 adds **22 audit/checkpoint/crash-window tests** on top of the **273 certified K1.9 tests**, for an expected suite of **295 tests**. The new
corpus covers SHA-256 vectors, legacy checkpoint migration, canonical byte stability, tamper
detection, prepare-before-backend ordering, delivered/retry/dead-letter audit sequences, deferred
retry behavior, preflight failures, prepare-commit failure, terminal-commit ambiguity, automatic
redispatch blocking, recovery of in-doubt state, explicit retry authorization, assumed-delivered
resolution, takeover fence changes with stable delivery keys, audited redrive, audit preservation
across later cycles and replay independence.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.10 is certified only after the local release gate and cross-platform GitHub CI are fully green,
followed by publication of the `k1.10` tag.
