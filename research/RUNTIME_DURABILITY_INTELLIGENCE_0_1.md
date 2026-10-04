# NORDOI Runtime Durability Intelligence 0.1

K1.14 treats runtime durability as a semantic publication problem, not as a generic serialization
feature.

The key distinction is between three kinds of state:

1. **static program state** — rebuilt from canonical NAIR;
2. **host authority/policy** — supplied again by the environment and never minted by persisted data;
3. **mutable semantic state** — checkpointed because losing it would change deterministic
   continuation.

The effect journal cannot simply be embedded inside a runtime snapshot because external dispatch may
legitimately advance after the last runtime cycle. Conversely, the runtime checkpoint cannot accept
an arbitrary newer effect journal because a non-durable later cycle may have allocated new effect
identities.

K1.14 therefore binds the runtime snapshot to two effect-journal facts:

```text
audit prefix root + height
next EffectIntentId frontier
```

The audit prefix proves ancestry. The identity frontier distinguishes effect-only delivery progress
from later semantic cycles that allocated new effects.

This produces the desired recovery relation:

```text
newer audit descendant + same intent frontier
    → compatible with saved runtime state

shorter/forked audit
    → reject

same/descendant audit + different intent frontier
    → reject
```

The protocol also preserves the architectural rule that recovery is not new program meaning. A
synthetic "recovered" replay event would make a crash change deterministic identity. Instead K1.14
restores the exact replay accumulator from the durable semantic point and continues from there.

Finally, the combined store interface deliberately remains host-injected. NORDOI defines the atomic
contract but does not smuggle filesystem/database/network authority into its kernel. That keeps the
runtime portable across embedded storage, local databases, transactional cloud stores, replicated
systems or future verified persistence engines.
