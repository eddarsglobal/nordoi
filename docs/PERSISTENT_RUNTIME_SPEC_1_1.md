# NORDOI Persistent Atomic Runtime 1.1

## Status

K1.1 candidate specification. Certification requires the repository release gate and
GitHub CI on Linux, macOS and Windows.

## Purpose

K1.0 proves one closed, isolated activation. K1.1 adds a persistent session model in
which bootstrap-created NAM atoms, ownership domains, render nodes and input bridges
survive across multiple deterministic input ticks.

## Execution model

```text
NAIR 0.3 program
      ↓ bootstrap once
NAM + Render + Input bindings
      ↓
PersistentAtomicRuntime
      ↓ tick(InputBatch)
canonical input validation
      ↓
input bridge applications
      ↓
atomic NAM transactions
      ↓
one deduplicated NAM frontier
      ↓
Atomic Render Core
      ↓
quiescent tick report
```

The source program is executed once at bootstrap. Persistent ticks do not recreate
program domains, atoms or render nodes. Instead, each `APPLY_INPUT` instruction in the
validated program defines a persistent input-application boundary that is replayed in
its original program order for every accepted tick.

## Persistent identities

The following runtime identities survive for the life of one persistent runtime:

- semantic `AtomSlot -> AtomId` bindings;
- ownership domains created during bootstrap;
- render nodes and their atom bindings;
- input bridges and selectors reconstructed from validated NAIR semantics.

A new persistent runtime boot creates a new isolated world. Identity is persistent
within one world, not global across worlds.

## Tick atomic publication

A tick is evaluated on private clones of NAM and the Atomic Render Core. Only a fully
successful, quiescent tick replaces the currently published persistent state.

Therefore a rejected tick:

- does not advance the tick counter;
- does not advance the session replay identity;
- does not update the last accepted input sequence;
- does not publish partial NAM/render mutation.

## Cross-tick input order

`InputBatch` already requires strictly increasing sequence numbers inside one batch.
K1.1 extends this law across ticks: the first sequence in a non-empty new tick must be
strictly greater than the last sequence accepted by the session.

Empty ticks are valid and do not change the stored last sequence.

## Rendering

After all persistent input-application boundaries have run, K1.1 performs at most one
NAM-to-render pump for that tick.

If no state value changes and no render node is pending, no frame is emitted.

This preserves `No Work Without Effect` across long-lived sessions.

## Replay identity

A persistent session replay key is derived incrementally from:

1. the canonical NAIR program bytes;
2. each accepted canonical input batch, in tick order and with tick boundaries
   preserved by length-delimited hashing.

The key is deterministic engineering metadata only. It is not cryptographic security
authority.

## Scope limits

K1.1 does not yet define:

- wall-clock time or timers;
- asynchronous I/O;
- networking;
- filesystem effects;
- threads/tasks as source semantics;
- hot code replacement;
- persistence to disk;
- the `.noi` source language frontend.

Those features must arrive through explicit future semantics and capability laws.
