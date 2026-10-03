# NORDOI Effect Fencing Protocol 1.0

Status: K1.8 certified specification.

## 1. Purpose

K1.7 makes the governed effect outbox durable and recoverable, but a durable journal may be
opened by more than one host process after crash, failover, partition, operator error, or an
orchestrator race. K1.8 adds a host-governed fencing protocol so stale writers can be rejected
without introducing ambient clocks or a mandatory distributed-system dependency into the core.

## 2. Identities

K1.8 adds:

- `EffectJournalWriterId([u8; 16])`: explicit host-provided writer identity;
- `EffectDeliveryFence(u64)`: monotonic writer epoch;
- `EffectJournalLease { namespace, writer, fence }`: proof of current journal ownership.

Fence zero is invalid for an acquired lease.

## 3. Store boundary

`FencedEffectJournalStore` is host injected. A conforming implementation must guarantee, per
`EffectDeliveryNamespace`:

1. every successful `acquire` returns a strictly greater, non-zero fence;
2. a newer acquisition makes older leases stale;
3. `assert_active`, `load_fenced`, `commit_fenced`, and `release` reject stale leases;
4. `commit_fenced` atomically validates the supplied fence and replaces the checkpoint;
5. a stale writer can never overwrite state committed under a newer fence.

The NORDOI core cannot prove that an arbitrary host implementation truly provides linearizable
compare-and-swap, consensus, durable media, or partition safety. Those remain host trust
boundaries.

## 4. No ambient lease clock

K1.8 deliberately defines no implicit TTL, wall-clock expiry, process heartbeat, or background
renewal inside the core. A host may layer such policies above the protocol, but the canonical
core consumes only an explicit lease/fence result.

This avoids making correctness depend on clock synchronization.

## 5. Candidate-cycle publication

`cycle_to_with_fenced_effect_journal(...)` performs:

1. verify that the current lease is active;
2. evaluate the full event-loop cycle on a private clone;
3. stage effect intents in the candidate outbox;
4. capture the canonical K1.7 checkpoint;
5. call `commit_fenced(lease, checkpoint)`;
6. publish the local candidate only after successful fenced commit.

Any stale-fence or persistence failure leaves the live event loop unchanged.

## 6. Dispatch

`dispatch_next_effect_with_fenced_journal(...)` performs:

1. verify the active lease before backend execution;
2. derive the stable K1.7 `EffectDeliveryKey(namespace, intent)`;
3. pass the current `EffectDeliveryFence` to the backend context;
4. execute the governed backend;
5. acknowledge on a private outbox candidate;
6. persist that acknowledgement with `commit_fenced`;
7. only then remove the live pending intent.

A stale writer detected before step 4 cannot call the backend through this API.

## 7. Race after pre-dispatch validation

No local API can universally make a remote side effect and a local fencing-store transition one
atomic operation. Ownership may change after the pre-dispatch check but before/during an external
call. Therefore K1.8 exposes the fence to the backend request context.

A cooperating destination can combine:

- `EffectDeliveryKey` for semantic retry deduplication; and
- `EffectDeliveryFence` for stale-writer rejection.

Legacy backends remain compatible and may ignore the fence. In that case NORDOI does not claim
universal duplicate suppression or exactly-once execution.

## 8. Recovery

A new writer may acquire a higher fence and recover the same K1.7 checkpoint before the first
cycle. The fence itself does not enter replay identity because it is host ownership policy, not
program meaning. Recovered semantic pending state continues to participate in replay exactly as
certified in K1.7.

## 9. NAIR

K1.8 adds no serialized program instruction. NAIR remains 0.5. Writer identities, leases and
fences must not be serialized as canonical program authority.
