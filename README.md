# NORDOI K1.7 — Persistent Effect Journal & Recovery Protocol

K1.7 extends the certified K1.6 governed effect outbox with a crash-recovery protocol
that can be backed by an explicit host persistence implementation. It preserves the
core constitutional boundary: NORDOI does not acquire ambient filesystem authority and
does not pretend that a successful external side effect can always be rolled back.

The delivery path is now:

```text
canonical cause
     ↓
NAIR 0.5 native reaction
     ↓
validated EffectIntent
     ↓
AtomicEffectOutbox candidate
     ↓
canonical EffectOutboxCheckpoint
     ↓
host EffectJournalStore.commit(...)
     ↓
local event-loop publication
     ↓
GovernedEffectDispatcher
     ↓
EffectDispatchRequest + stable EffectDeliveryKey
     ↓
host EffectBackend
     ↓
external system
```

## What K1.7 adds

- canonical versioned `EffectOutboxCheckpoint` encoding;
- explicit `EffectDeliveryNamespace([u8; 16])` supplied by the host;
- stable `EffectDeliveryKey { namespace, intent }` for retry-aware backends;
- corruption detection with a deterministic checkpoint checksum;
- bounded checkpoint, pending-count and string decoding limits;
- `EffectJournalStore` as a host-injected atomic persistence boundary;
- `GovernedEffectJournal` for checkpoint, recovery and persisted acknowledgement;
- event-loop recovery before the first cycle only;
- recovered outbox state participates explicitly in replay identity;
- delivery namespace and backend receipt metadata remain outside deterministic program meaning;
- `cycle_to_with_effect_journal(...)` evaluates on a private clone and publishes locally only
  after the journal accepts the candidate checkpoint;
- `dispatch_next_effect_with_journal(...)` executes against a private outbox candidate and
  persists the acknowledgement before removing the live pending intent;
- backend success followed by journal failure leaves the live intent pending for retry;
- the retry receives the same `EffectDeliveryKey`;
- existing K1.6 `EffectBackend` implementations remain source-compatible through the default
  `execute_with_context(...)` adapter;
- no built-in filesystem/database backend and zero new Rust dependencies.

## Durability boundary

K1.7 certifies the **protocol**, canonical checkpoint format, recovery validation and
publication ordering. Physical durability depends on the host implementation of
`EffectJournalStore`.

The store contract is strict: once `commit(bytes)` returns success, that implementation
is promising that the new checkpoint atomically replaced the previous checkpoint and
is recoverable according to the durability guarantees advertised by that host.

The NORDOI core can validate the checkpoint bytes and enforce protocol ordering. It
cannot prove that an arbitrary host backend actually called `fsync`, used a correct
filesystem, survived power loss, or honestly implemented its contract. A malicious or
broken host remains a trust boundary.

K1.7 therefore distinguishes:

```text
certified NORDOI journal protocol
            ≠
universal physical-media durability
```

A host may implement the store with SQLite/WAL, an append-only log, a database
transaction or another mechanism, but that backend is not part of K1.7 core.

## Candidate publication law

`cycle_to_with_effect_journal(...)` performs:

```text
1. clone current event-loop state
2. execute the complete candidate cycle on the clone
3. stage new effect envelopes in the candidate outbox
4. encode the complete candidate outbox checkpoint
5. ask the host journal store to commit it
6. only after successful journal commit, publish the candidate event-loop state
```

If steps 2–5 fail, the live event loop is unchanged.

K1.7 does not yet claim durable recovery of NAM atoms, logical time, render state or
other runtime state. It certifies durable effect-journal recovery when the host store
honors its contract. Broader whole-runtime persistence remains a separate future law.

## Retry and idempotency law

K1.6 already refused to claim universal exactly-once delivery. K1.7 makes retries safer
without weakening that rule.

A host provides one stable `EffectDeliveryNamespace` for a journal lifetime. NORDOI
combines it with the deterministic `EffectIntentId`:

```text
EffectDeliveryKey = journal namespace + effect intent id
```

The core never generates the namespace from ambient randomness. The host is responsible
for provisioning a namespace appropriate for its deployment and reusing the same
namespace when recovering the same journal.

A retry-aware backend receives:

```text
EffectDispatchRequest {
    queued,
    delivery_key: Some(...)
}
```

If the destination supports idempotency/client-request tokens, the backend can forward
this key. If the destination does not support deduplication, duplicate external effects
remain possible across the classic failure window where the destination completes but
the local acknowledgement cannot be persisted.

Therefore K1.7 provides **stable retry identity**, not a universal exactly-once claim.

## Recovery law

A checkpoint contains:

- format magic and version;
- effect delivery namespace;
- next monotonic `EffectIntentId`;
- every pending `QueuedEffectIntent` in canonical identity order;
- cycle and ordinal metadata;
- reaction identity;
- action name;
- exact effect scope;
- deterministic checksum.

Recovery validates the entire checkpoint before installing it. Corrupt, truncated,
oversized, version-incompatible or namespace-mismatched checkpoints fail closed.

Recovery into `AtomicEventLoop` is accepted only before the first cycle. This avoids
silently replacing a live delivery history after runtime execution has started.

The recovered outbox content and next intent identity participate in replay identity.
The host-only delivery namespace does not: two environments may use different delivery
namespaces without changing the deterministic program meaning of the pending intents.

## Checkpoint resource limits

K1.7 rejects hostile or accidental oversized persisted input before unbounded allocation:

```text
maximum checkpoint bytes : 64 MiB
maximum pending intents   : 65,536
maximum encoded string    : 1 MiB
```

These are format-decoder safety limits, not application quotas. Future versions may
make deployment budgets more granular while preserving fail-closed decoding.

## NAIR version

**NAIR remains 0.5 in K1.7.**

K1.7 adds host persistence and delivery protocol semantics. It does not add a new
serialized NORDOI program instruction. Journal stores, namespaces, durability policy,
idempotency routing and backend receipts are environmental runtime policy and SHALL NOT
be smuggled into canonical NAIR authority.

## External result boundary

K1.7 still does not allow an effect backend to mutate NAM directly. A successful or
failed environmental result that needs to influence the program must later return
through a separately governed semantic completion/input mechanism.

That future re-entry mechanism is intentionally not invented inside the persistence
layer.

## Test corpus

K1.7 adds **19 persistence/recovery tests** to the **205 certified K1.6 tests**, for an
expected total of **224 tests**.

The new tests cover canonical checkpoint round-trip, byte stability, corruption and
truncation rejection, store failures, empty recovery, namespace mismatch, persisted
candidate publication, failed-persistence rollback, pre-cycle recovery, explicit replay
recovery identity, late-recovery rejection, monotonic IDs after recovery, stable delivery
keys, retry after acknowledgement-persistence failure, persisted acknowledgement,
backend failure preservation and namespace-separated delivery identity.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.7` can be tagged.

## Key specifications

- `docs/EFFECT_JOURNAL_RECOVERY_SPEC_1_0.md`
- `docs/EFFECT_OUTBOX_DISPATCH_SPEC_1_0.md`
- `docs/NAIR_NATIVE_REACTION_SPEC_0_1.md`
- `docs/NAIR_SPEC_0_5.md`
- `research/EFFECT_PERSISTENCE_INTELLIGENCE_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
