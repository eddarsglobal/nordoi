# NORDOI Effect Retry, Backoff & Dead-Letter Protocol 1.0

Status: K1.9 candidate specification.

## 1. Purpose

K1.6 created governed external effect dispatch. K1.7 made the outbox durable. K1.8 added
multi-writer fencing. K1.9 defines what happens when a valid external effect cannot be delivered
successfully after one or more backend executions.

The protocol must prevent:

- infinite immediate retry loops;
- unbounded attempt storms;
- one poisoned intent permanently blocking later eligible intents;
- silent loss after repeated failures;
- retry-policy drift after recovery;
- stale-writer retry execution;
- hidden dependence on wall clocks or ambient randomness.

## 2. Non-goals

K1.9 does not promise universal exactly-once external effects. It does not add distributed clocks,
network consensus, OS timers, random number generators, or NAIR program instructions. It does not
replace backend-specific rate limiting or circuit breakers.

## 3. Retry time domain

`EffectRetryTick(u64)` is an explicit host-supplied monotonic scheduling coordinate. It is not
NORDOI logical program time.

A host may map retry ticks to:

- durable scheduler epochs;
- wall-clock-derived monotonic slots;
- queue scheduler turns;
- another explicit monotonic domain.

The core never reads a wall clock. Once an attempt outcome is durably published at tick `T`, later
attempt publication may not use a tick lower than `T`.

## 4. Retry policy

`EffectRetryPolicy` contains:

- `max_attempts` — total backend executions allowed, including the first attempt;
- `initial_backoff_ticks` — delay after the first retryable failure;
- `max_backoff_ticks` — hard cap for exponential delay;
- `jitter_ticks` — maximum deterministic jitter added before the cap.

Invalid policies are rejected before use:

- `max_attempts == 0`;
- `initial_backoff_ticks == 0`;
- `max_backoff_ticks < initial_backoff_ticks`;
- `jitter_ticks > max_backoff_ticks`.

## 5. Deterministic backoff

After retryable failure number `n >= 1`:

```text
base = min(max_backoff, initial_backoff * 2^(n-1))
jitter = deterministic_hash(delivery_key, n) mod (jitter_ticks + 1)
delay = min(max_backoff, base + jitter)
next_eligible = current_retry_tick + delay
```

The jitter source is stable semantic delivery identity, not ambient randomness. Equal delivery
identity, policy and failure ordinal produce equal scheduling decisions across crash recovery and
writer takeover.

## 6. Backend failure classification

`EffectBackendError::new(...)` remains backward-compatible and means retryable failure.

`EffectBackendError::permanent(...)` means retrying the same request is not expected to succeed
without semantic/operator change.

Only actual backend execution failures consume retry budget. Capability denial, unsupported
backend configuration and other pre-dispatch policy failures do not increment attempt counts.

## 7. Durable delivery states

A semantic effect may be in one of three K1.9 delivery states:

```text
PENDING
  no retry delay currently blocks execution

RETRY_SCHEDULED
  still in semantic outbox
  failed_attempts > 0
  next_eligible_tick recorded durably

DEAD_LETTERED
  removed from active outbox
  complete original queued request retained durably
  stable delivery key retained
  failed_attempts + terminal reason + last error retained
```

Dead-letter reasons in K1.9 are:

- `PermanentBackendFailure`;
- `AttemptsExhausted`.

## 8. Eligibility and ordering

Among intents eligible at the supplied retry tick, dispatch order remains ascending
`EffectIntentId`.

A delayed lower-ID intent does not block a higher-ID intent that is already eligible. This avoids a
single poison intent creating permanent head-of-line blocking for unrelated later work.

If all pending intents are delayed, dispatch produces a `Deferred { next_eligible_tick }` outcome
and creates zero backend work.

## 9. Publication ordering

For retryable backend failure:

```text
1. assert current fence
2. execute backend
3. compute candidate retry state
4. encode K1.9 checkpoint
5. commit_fenced(candidate checkpoint)
6. publish local retry state
```

For permanent/exhausted failure:

```text
1. assert current fence
2. execute backend
3. move intent on private candidate from outbox to dead-letter ledger
4. commit_fenced(candidate checkpoint)
5. publish local outbox + dead-letter state
```

For backend success:

```text
1. assert current fence
2. execute backend
3. acknowledge on private outbox candidate
4. clear retry record
5. commit_fenced(candidate checkpoint)
6. publish local acknowledgement
```

If checkpoint persistence fails, local state is not published. As in K1.7/K1.8, a successful
remote call followed by failed acknowledgement persistence may therefore be retried. Stable
`EffectDeliveryKey` and current fence remain the cooperative protection boundary.

## 10. Dead-letter redrive

Manual redrive moves the exact original `QueuedEffectIntent` back into the active outbox, clears
its prior retry record, and preserves its original `EffectIntentId`.

Therefore the stable `EffectDeliveryKey(namespace, intent_id)` is unchanged by dead-letter and
redrive.

Dead letters may also be explicitly discarded. Both redrive and discard require current fenced
journal ownership and successful durable commit before local publication.

## 11. Checkpoint format

K1.9 introduces canonical retry checkpoint magic:

```text
NDEFXR01
```

Format 1.0 contains:

- delivery namespace;
- persisted retry policy (or absent marker for migrated legacy checkpoints);
- embedded certified K1.7/K1.8 outbox checkpoint bytes;
- last published retry tick;
- ordered retry records;
- ordered dead-letter records;
- trailing checksum.

Decoder limits are checked before allocation:

- retry checkpoint: 128 MiB maximum;
- embedded outbox checkpoint: existing 64 MiB maximum;
- retry records: 65,536 maximum;
- dead letters: 65,536 maximum;
- strings: existing 1 MiB maximum.

## 12. Legacy migration

A certified `NDEFXJ01` K1.7/K1.8 checkpoint is accepted by the K1.9 decoder. It becomes:

```text
same namespace
same pending outbox
same next intent ID
empty retry ledger
empty dead-letter ledger
no persisted retry policy yet
```

The first K1.9 checkpoint commit persists the configured policy and full K1.9 state.

## 13. Retry-policy recovery law

Once a K1.9 checkpoint contains a policy, recovery through `GovernedRetryEffectJournal` requires an
exact policy match. Silent policy drift is rejected. Policy migration must therefore be an explicit
future operation rather than an accidental consequence of process restart.

## 14. Replay boundary

Retry ticks, attempt counts, retry policy, dead-letter metadata, backend error strings and writer
fences are delivery/runtime policy, not canonical program meaning. Retry/dead-letter transitions do
not mutate the live event-loop replay key.

The semantic pending outbox restored at startup continues to affect recovery replay identity under
the K1.7 law.

## 15. NAIR boundary

K1.9 adds no new program instruction. NAIR remains 0.5.

Retry policy, retry ticks, attempt state and dead-letter control must not become serialized program
authority.
