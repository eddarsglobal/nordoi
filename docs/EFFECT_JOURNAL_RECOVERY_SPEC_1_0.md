# NORDOI Effect Journal & Recovery Specification 1.0

**Release:** K1.7  
**Status:** Candidate until the K1.7 release gate and cross-platform CI pass.

## 1. Scope

This specification defines the persistence protocol for the K1.6 semantic effect
outbox. It covers canonical checkpoint encoding, host store contracts, recovery,
resource limits, retry identity and persisted acknowledgement. It does not define a
filesystem/database implementation and does not certify whole-runtime persistence.

## 2. Checkpoint identity

A checkpoint is the complete recoverable state of one effect journal:

```text
EffectOutboxCheckpoint {
    delivery namespace,
    next EffectIntentId,
    ordered pending QueuedEffectIntent[]
}
```

The checkpoint is a full replacement snapshot rather than an append API. A host store
may internally use WAL, append-only pages, transactions or another implementation, but
that representation is outside canonical NORDOI semantics.

## 3. Canonical binary format

All integers use little-endian encoding.

```text
8 bytes   magic = "NDEFXJ01"
u16       format major = 1
u16       format minor = 0
16 bytes  EffectDeliveryNamespace
u64       next_intent_id
u64       pending_count
repeated pending_count times:
  u64     intent_id
  u64     cycle
  u64     ordinal
  u64     reaction_id
  string  action_name
  effect  exact Effect value/scope
u64       FNV-1a checksum of all preceding checkpoint bytes
```

A string is encoded as `u64 byte_length` followed by exact UTF-8 bytes.

The checksum detects accidental corruption and malformed transfer. It is not a
cryptographic signature and does not authenticate a hostile store.

## 4. Decoder safety budgets

Before allocation, recovery enforces:

- checkpoint <= 64 MiB;
- pending intent count <= 65,536;
- each encoded string <= 1 MiB;
- `EffectIntentId != 0`;
- strictly increasing persisted intent IDs;
- `ReactionId != 0`;
- `next_intent_id > max(pending intent id)` when pending intents exist.

Any violation fails closed.

## 5. Host store contract

```text
trait EffectJournalStore {
    load() -> Option<bytes>
    commit(bytes) -> commit receipt
}
```

A successful `commit` is a host assertion that the supplied checkpoint atomically
replaced the previous checkpoint and is recoverable according to that implementation's
durability policy.

The NORDOI core deliberately does not contain a default filesystem or database backend.

## 6. Journal-aware cycle publication

`cycle_to_with_effect_journal` uses a private clone:

```text
live event loop
      ↓ clone
candidate cycle executes
      ↓
candidate outbox checkpoint
      ↓
store.commit(checkpoint)
      ├─ failure -> discard candidate, live state unchanged
      └─ success -> candidate becomes live state
```

This certifies effect-journal publication ordering. It does not make all event-loop
state crash-durable because NAM/time/render persistence is outside K1.7.

## 7. Recovery

Recovery is allowed only before event-loop cycle 1.

The host constructs `GovernedEffectJournal(namespace, store)`, calls `recover`, and the
checkpoint namespace must exactly equal the configured journal namespace. The event
loop then installs pending intents and `next_intent_id`.

Recovered semantic outbox state participates in replay identity. The delivery namespace
does not participate in deterministic program meaning.

## 8. Stable delivery key

For persistent dispatch:

```text
EffectDeliveryKey {
    namespace: EffectDeliveryNamespace,
    intent: EffectIntentId
}
```

The key is exposed in `EffectDispatchRequest`. Backends that support idempotency/client
request tokens SHOULD map this stable key to the destination protocol.

Legacy K1.6 backends remain valid because `execute_with_context` defaults to calling the
existing `execute(QueuedEffectIntent)` method.

## 9. Persisted acknowledgement

Persistent dispatch proceeds on a private outbox clone:

```text
peek pending intent
revalidate exact capability
execute backend with stable delivery key
remove intent from candidate outbox
commit candidate checkpoint
  failure -> live outbox still pending
  success -> publish acknowledged candidate
```

If external execution succeeds but checkpoint commit fails, a retry is possible and
uses the same key. Duplicate prevention still requires a cooperating destination.

## 10. Non-goals

K1.7 does not claim:

- universal exactly-once external side effects;
- cryptographic authenticity of checkpoints;
- a built-in disk/database implementation;
- durable NAM/time/render recovery;
- distributed consensus or multi-machine journal replication;
- semantic re-entry of external completion results.

Those require separate laws and certification.
