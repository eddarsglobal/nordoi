# NORDOI Runtime Semantic Checkpoint & Recovery Specification 1.0

Status: K1.14 candidate specification.

## 1. Purpose

This specification defines crash-consistent persistence and recovery for the mutable semantic state
of `AtomicEventLoop`. It composes with the already-certified effect persistence, fencing, retry,
audit and completion layers rather than replacing them.

## 2. Design boundary

A runtime checkpoint is not a memory dump. Static program structure is reconstructed by booting the
same canonical NAIR program. Host authority is re-supplied by the host. Only mutable semantic state
needed to continue deterministic execution is serialized.

## 3. Required mutable state

The checkpoint SHALL bind:

- domain-separated canonical-program SHA-256;
- effect delivery namespace;
- K1.10 audit prefix count and root;
- next effect-intent identity;
- event-loop cycle and replay state;
- persistent-runtime tick and replay state;
- last accepted input sequence;
- next transaction identity;
- atom slot, resolved atom identity, exact value and version;
- render slot revision;
- logical time;
- next timer identity and configured fire budget;
- retained timers and occurrence counts;
- completion source sequence state;
- consumed effect-delivery keys.

## 4. Bundle publication

A conforming `FencedRuntimeCheckpointStore` SHALL atomically replace the K1.10 effect/audit
checkpoint and K1.14 runtime checkpoint under one active `EffectJournalLease`.

A successful store return is the persistence contract boundary. NORDOI cannot independently prove
filesystem flush, database replication or power-loss guarantees of arbitrary host implementations.

Before a first K1.14 publication, the runtime SHALL establish durable lineage. An empty store is a
valid new lineage. A partial legacy effect-only/runtime-only state SHALL fail closed rather than being
automatically migrated. If a complete runtime bundle already exists, a freshly booted runtime SHALL
recover it before replacement.

The live candidate SHALL NOT be published before the combined commit succeeds.

## 5. Effect-only progress after checkpoint

K1.10 effect delivery operations MAY advance the effect checkpoint after the runtime checkpoint.
This is valid only when the newer audit is a descendant of the prefix recorded by K1.14 and the
next effect-intent allocation identity remains equal to the runtime checkpoint frontier.

This distinction allows dispatch/retry/dead-letter progress without pretending that an uncheckpointed
runtime cycle is durable.

## 6. Recovery protocol

Recovery SHALL:

1. require event-loop cycle zero;
2. require an active fenced journal lease;
3. load both effect and runtime halves;
4. fail if exactly one half is missing;
5. decode and validate both canonical formats;
6. validate namespace and retry policy;
7. validate exact canonical program hash;
8. validate exact next-effect-intent frontier;
9. validate that current audit contains the recorded prefix root at the recorded height;
10. restore candidate runtime, time, completion and outbox state;
11. adopt the recovered retry/audit ledger;
12. publish the recovered candidate only after all checks succeed.

Recovery itself SHALL NOT alter replay identity.

## 7. Runtime shape validation

The booted atom slots and render slots SHALL exactly match the checkpoint. Resolved atom identities
SHALL match. Restored atom versions, render revisions, transaction identity and timer identity SHALL
not move behind the state produced by booting the same program.

The configured timer fire budget SHALL match exactly.

## 8. Canonical format

Magic:

```text
NDRTSM01
```

Version:

```text
major = 1
minor = 0
```

Integers are little-endian. Collections are serialized in canonical key order. Text is UTF-8 and
bounded. Non-finite floating-point values are rejected. The final 32 bytes are SHA-256 over an
explicit domain separator plus all preceding checkpoint bytes.

Current implementation limits:

- checkpoint: 256 MiB;
- atoms: 262,144;
- timers: 262,144;
- completion sources: 65,536;
- completed delivery keys: 262,144;
- one text value: 1 MiB.

## 9. Completion durability

K1.14 persists K1.12/K1.13 dynamic completion state: last accepted sequence per source and consumed
`EffectDeliveryKey`s. After recovery, a delivery key already transformed into a semantic completion
remains consumed.

Completion authority and projection policy are reconstructed from the current boot/host context and
are not serialized as privileges.

## 10. Non-claims

K1.14 does not claim:

- universal exactly-once external delivery;
- persistence of arbitrary process memory;
- persistence of host credentials or ambient OS resources;
- hidden durability for legacy non-K1.14 cycle APIs;
- any specific physical durability guarantee beyond the host store contract;
- a new NAIR program semantic.

NAIR therefore remains 0.6.
