# Effect Fencing Intelligence 0.1

K1.8 research question: how can NORDOI prevent stale recovered writers from corrupting or
concurrently dispatching a durable effect journal without making clocks, a particular database,
or a distributed coordinator part of canonical language semantics?

## Selected principle

Use monotonic fencing epochs rather than trusting a local lock or time-based lease alone.

A writer receives an explicit host-issued epoch. Every protected journal mutation must carry that
epoch. The host persistence authority rejects an epoch older than the currently active one.

This has three useful properties for NORDOI:

1. no clock synchronization is required by the core;
2. stale process memory cannot regain authority merely because it still holds an old object;
3. the persistence/backend implementation stays replaceable.

## Important limit

Fencing the journal cannot by itself make an already-started remote network operation disappear.
The external destination must cooperate if stale-writer rejection is required at that boundary.
K1.8 therefore forwards the current fence in `EffectDispatchRequest` while retaining K1.7's stable
semantic delivery key.

The pair is intentional:

```text
stable key  -> which semantic intent is this?
fence       -> which writer epoch is allowed to attempt it now?
```

## Non-goals

- no universal exactly-once claim;
- no mandatory Raft/Paxos/database/Redis/etcd dependency;
- no hidden wall clock or automatic TTL;
- no host-generated authority serialized into NAIR;
- no claim that an arbitrary `FencedEffectJournalStore` honestly implements linearizability.
