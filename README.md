# NORDOI K1.8 — Fenced Effect Journal Ownership & Concurrent Dispatch Protocol

K1.8 extends the certified K1.7 persistent effect journal with explicit multi-writer protection.
K1.7 can recover a durable outbox after crash; K1.8 defines which recovered writer is currently
authorized to mutate and dispatch that journal when multiple hosts/processes race for ownership.

The governed path is now:

```text
canonical cause
     ↓
NAIR 0.5 native reaction
     ↓
validated EffectIntent
     ↓
AtomicEffectOutbox
     ↓
canonical K1.7 checkpoint
     ↓
FencedEffectJournalStore
     ↓
EffectJournalLease(writer, fence)
     ↓
fenced checkpoint publication
     ↓
GovernedEffectDispatcher
     ↓
EffectDeliveryKey + EffectDeliveryFence
     ↓
host EffectBackend
     ↓
external system
```

## What K1.8 adds

- explicit `EffectJournalWriterId([u8; 16])` supplied by the host;
- monotonic `EffectDeliveryFence(u64)` writer epochs;
- `EffectJournalLease { namespace, writer, fence }`;
- `FencedEffectJournalStore` host boundary;
- `GovernedFencedEffectJournal`;
- stale-writer checks before candidate cycles and before external dispatch;
- atomic fenced checkpoint commit contract;
- takeover/recovery under a strictly newer fence;
- current fence propagated through `EffectDispatchRequest`;
- current fence exposed in `EffectDispatchReceipt`;
- stable K1.7 `EffectDeliveryKey` preserved across writer takeover;
- legacy K1.6/K1.7 effect backends remain source-compatible through the default context adapter;
- no clock/TTL requirement in the canonical core;
- zero new Rust dependencies;
- NAIR remains 0.5.

## Two identities, two jobs

K1.8 intentionally separates semantic request identity from writer authority:

```text
EffectDeliveryKey   = which semantic effect intent is this?
EffectDeliveryFence = which writer epoch is currently authorized?
```

A takeover therefore changes the fence but not the delivery key of an already-pending intent.

This distinction matters after a crash or failover. A cooperating destination can deduplicate
retries by stable key while rejecting stale writers by monotonically increasing fence.

## No ambient clock

K1.8 does not manufacture leases from wall time, synchronized clocks, machine IDs, process IDs,
or hidden randomness. The core accepts an explicit host-provided writer ID and a host-issued
monotonic fence.

A host may implement lease expiry or heartbeat policy externally, but canonical correctness does
not depend on time synchronization.

## Fenced store contract

For one `EffectDeliveryNamespace`, a conforming `FencedEffectJournalStore` promises:

```text
acquire(writer A) -> fence 1
acquire(writer B) -> fence 2

writer A + fence 1 => stale
writer B + fence 2 => current
```

Every successful acquisition must return a strictly newer non-zero fence. `assert_active`,
`load_fenced`, `commit_fenced`, and `release` must reject stale leases. `commit_fenced` must
validate the fence and replace the checkpoint as one atomic host-side operation.

NORDOI can certify this protocol surface and its local publication ordering. It cannot prove that
an arbitrary database, coordinator or custom host implementation honestly provides consensus,
linearizability or durable storage.

## Fenced cycle publication

`cycle_to_with_fenced_effect_journal(...)` performs:

```text
1. assert current lease
2. clone the event loop
3. evaluate complete candidate cycle
4. stage candidate effects
5. encode canonical outbox checkpoint
6. commit_fenced(current lease, checkpoint)
7. publish local candidate only after success
```

If the lease is stale or the fenced commit fails, the live cycle index, replay state, runtime,
time and outbox remain unchanged.

## Fenced dispatch

`dispatch_next_effect_with_fenced_journal(...)` performs a stale-writer check before calling the
backend. The backend receives:

```text
EffectDispatchRequest {
    queued,
    delivery_key: Some(stable semantic key),
    delivery_fence: Some(current writer fence),
}
```

After backend success, acknowledgement is still persisted on a private outbox candidate before
the live pending intent is removed.

## Important distributed-systems limit

There is an unavoidable boundary between a local journal and an arbitrary remote system. A writer
can be current during the pre-dispatch check and lose ownership immediately afterward while a
remote call is already in flight.

Therefore K1.8 does **not** claim universal exactly-once side effects or universal remote fencing.
A destination that needs those properties must cooperate by honoring the stable delivery key and/or
the supplied fence.

Legacy backends may ignore `delivery_fence`. They remain valid, but they do not become
fence-aware merely by running under K1.8.

## Replay law

Writer identity, active lease and fence are host ownership policy. They do not change deterministic
program meaning and therefore do not enter replay identity.

Recovered semantic outbox content and next semantic intent ID continue to affect replay exactly as
certified in K1.7.

## NAIR law

**NAIR remains 0.5 in K1.8.**

K1.8 adds no new program instruction. Writer IDs, leases, fencing epochs, persistence topology and
coordinator policy must not become serialized canonical program authority.

## Tests

K1.8 adds **19 fencing/concurrency tests** to the **224 certified K1.7 tests**, for an expected
suite of **243 tests**.

The new corpus covers:

- non-zero monotonically increasing fences;
- same-journal double-acquire rejection;
- takeover and stale-writer invalidation;
- stale checkpoint overwrite prevention;
- lease-required fail-closed behavior;
- candidate cycle publication under fencing;
- recovery by a new writer;
- stale cycle rejection before publication;
- fence propagation to external backends;
- stale dispatch rejection before backend work;
- stable delivery key across takeover;
- failed acknowledgement persistence and retry;
- release semantics;
- malformed zero/mismatched host leases;
- explicit host active-check failure;
- replay independence from fencing epoch.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.8 is certified only after the local gate and the cross-platform GitHub CI are fully green.
