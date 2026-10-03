# NORDOI K1.9 — Deterministic Retry, Backoff & Dead-Letter Protocol

K1.9 extends the certified K1.8 fenced effect journal with bounded retry scheduling, deterministic
backoff, durable poison-effect quarantine and manual redrive.

The governed external-effect path is now:

```text
canonical cause
     ↓
NAIR 0.5 native reaction
     ↓
validated EffectIntent
     ↓
AtomicEffectOutbox
     ↓
K1.9 retry checkpoint
     ↓
K1.8 fenced ownership
     ↓
Governed retry dispatcher
     ↓
EffectDeliveryKey + EffectDeliveryFence
     ↓
host EffectBackend
     ↓
SUCCESS
  or RETRY_SCHEDULED
  or DEAD_LETTERED
```

## What K1.9 adds

- `EffectRetryTick(u64)` as explicit host scheduling input;
- `EffectRetryPolicy` with finite attempt budget;
- capped exponential backoff;
- deterministic jitter derived from stable delivery identity;
- retryable vs permanent `EffectBackendError` classification;
- durable per-intent `EffectRetryRecord` state;
- durable `DeadLetteredEffect` quarantine;
- `EffectRetryCheckpoint` canonical format `NDEFXR01`;
- K1.7/K1.8 checkpoint migration;
- retry-policy persistence and recovery mismatch rejection;
- manual dead-letter redrive preserving the original delivery key;
- explicit dead-letter discard;
- no head-of-line blocking by an older delayed effect;
- K1.8 fencing retained for every retry/redrive mutation;
- zero new Rust dependencies;
- NAIR remains 0.5.

## Three delivery states

```text
PENDING
    effect may run now

RETRY_SCHEDULED
    effect stays in the semantic outbox
    failed_attempts is durable
    next_eligible_tick is durable

DEAD_LETTERED
    effect is removed from active delivery
    full original request is durably quarantined
    stable delivery key is preserved
```

Dead-lettering happens when either:

- the backend returns `EffectBackendError::permanent(...)`; or
- a retryable failure reaches the configured `max_attempts`.

## Explicit retry clock

`EffectRetryTick` is intentionally separate from program `LogicalTime`.

NORDOI does not read a wall clock or sleep internally. The host supplies the current retry tick.
That tick may correspond to a durable scheduler, wall-time bucket, queue epoch or another monotonic
host domain.

Once an attempt outcome is durably published at tick `T`, a later governed attempt cannot publish
at a tick below `T`.

## Bounded deterministic backoff

For failure number `n >= 1`:

```text
base = min(max_backoff, initial_backoff * 2^(n-1))
jitter = stable_hash(delivery_key, n) mod (jitter_ticks + 1)
delay = min(max_backoff, base + jitter)
next_eligible = current_tick + delay
```

No ambient RNG is required. The same delivery key, retry policy and failure ordinal produce the
same scheduling decision after crash or writer takeover.

## Retry policy is durable

A K1.9 checkpoint records:

```text
max_attempts
initial_backoff_ticks
max_backoff_ticks
jitter_ticks
```

Recovery rejects a different configured policy. This prevents a restart from silently turning, for
example, a 3-attempt journal into a 20-attempt journal.

Certified K1.7/K1.8 `NDEFXJ01` checkpoints are accepted with:

```text
same outbox
same next intent ID
empty retry ledger
empty dead-letter ledger
no historical attempts invented
```

The first K1.9 commit then persists the configured policy.

## No poison-effect head-of-line lock

K1.8 dispatches the first pending intent. K1.9 refines that rule for delayed retries:

```text
ID 1 -> retry blocked until tick 100
ID 2 -> eligible now
```

At tick 20, K1.9 may dispatch ID 2 while ID 1 remains delayed. Among all currently eligible
intents, lower IDs still execute first.

## Dead-letter redrive

A quarantined effect can be explicitly redriven. K1.9 restores the exact original queued request,
including its original `EffectIntentId`.

Therefore:

```text
before dead-letter: key = namespace + intent 42
after redrive:      key = namespace + intent 42
```

This matters for destinations that deduplicate by idempotency key.

## Persistence-before-publication

Every retry/dead-letter state transition is candidate-first:

```text
assert current fence
      ↓
execute / classify
      ↓
mutate private candidate
      ↓
encode K1.9 checkpoint
      ↓
commit_fenced(...)
      ↓
publish local state
```

If persistence fails, local retry/dead-letter state is unchanged.

As before, if a remote call succeeds but durable acknowledgement fails, the same effect may be sent
again. The stable delivery key and current fence remain the cooperative boundary. K1.9 does not
claim universal exactly-once external effects.

## Backend compatibility

Existing K1.6-K1.8 code using:

```rust
EffectBackendError::new("temporary failure")
```

remains source-compatible and is classified as retryable.

Backends may now explicitly return:

```rust
EffectBackendError::permanent("invalid request")
```

for non-retryable failures.

## Replay law

Retry control is external delivery policy. Retry ticks, failure counters, policy, dead-letter
metadata and error strings do not alter the live deterministic event-loop replay key.

Recovered semantic pending outbox contents still affect recovery replay identity under the
certified K1.7 rule.

## NAIR law

**NAIR remains 0.5 in K1.9.**

Retry scheduling and dead-letter policy are runtime/host delivery concerns, not serialized program
authority.

## Tests

K1.9 adds **30 retry/dead-letter tests** to the **243 certified K1.8 tests**, for an expected suite
of **273 tests**.

The new corpus covers policy validation, durable retry scheduling, no-early execution, eligible
intent bypass, capped exponential backoff, deterministic jitter, permanent failure, attempt
exhaustion, dead-letter recovery, redrive identity, discard, legacy migration, checksum detection,
retry-tick monotonicity, writer takeover, stale-writer rejection, commit failure atomicity,
successful retry cleanup, failed acknowledgement persistence, capability/backend configuration
failures, replay independence, next-intent continuity and retry-policy drift rejection.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.9 is certified only after the local release gate and the cross-platform GitHub CI are fully
green, followed by publication of the `k1.9` tag.
