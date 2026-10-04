# NORDOI K1.14 — Crash-Consistent Whole-Runtime Semantic Checkpoint & Recovery Core

K1.14 closes the crash-durability boundary left explicit by K1.12/K1.13. NORDOI can now persist
and recover the mutable semantic state of a booted `AtomicEventLoop` together with the governed
effect journal under one fencing epoch.

The K1.14 path is:

```text
same canonical NAIR 0.6 program
        ↓
normal boot + host authority
        ↓
AtomicEventLoop candidate cycle
        ↓
┌──────────────────────────────────────┐
│ K1.10 effect/audit checkpoint       │
│ K1.14 runtime semantic checkpoint   │
└──────────────────────────────────────┘
        ↓
atomic commit under one K1.8 fence
        ↓
only then publish candidate in memory
        ↓
crash / takeover
        ↓
boot same program
        ↓
validate program + audit ancestry + ID frontier
        ↓
restore exact semantic continuation
```

## What K1.14 persists

The runtime semantic checkpoint contains the mutable state needed for deterministic continuation:

- event-loop cycle and replay state;
- persistent-runtime tick and replay state;
- last accepted public input sequence;
- exact NAM atom values and versions;
- next transaction identity;
- render-node revisions;
- logical time;
- retained timers, deadlines, intervals and occurrence counts;
- next timer identity and configured fire budget;
- effect-completion source sequences;
- completed `EffectDeliveryKey` deduplication state;
- the current effect-intent allocation frontier;
- a cryptographic link to the K1.10 audit prefix;
- a domain-separated SHA-256 binding to the exact canonical NAIR program used to boot the runtime.

Static program structures and host authority are deliberately not serialized. Domains, dependency
structure, render bindings, native reaction declarations, completion projections and other static
program structures are rebuilt by booting the same canonical program. Host capabilities and source
authority are supplied again by the host, then the dynamic checkpoint is restored over that booted
shape.

## Atomic effect + runtime publication

`FencedRuntimeCheckpointStore` extends the certified fenced effect-journal store. Its combined commit
must atomically validate the active lease and replace both:

```text
EffectAuditCheckpoint (NDEFXA01)
RuntimeSemanticCheckpoint (NDRTSM01)
```

The durable cycle APIs evaluate a private candidate first. They publish the candidate to the live
runtime only after the host store reports a successful combined commit.

```text
candidate evaluation
        ↓
canonical effect checkpoint
canonical runtime checkpoint
        ↓
commit_effect_and_runtime_fenced(...)
        ↓
SUCCESS → publish candidate
FAILURE → publish nothing
```

A stale writer therefore cannot publish either half through the K1.14 protocol.

## Audit descendants after a runtime checkpoint

External delivery may continue after the last runtime checkpoint. Dispatch, retry, dead-letter and
in-doubt resolution can advance the K1.10 audit journal without changing NAM or logical runtime
state.

Recovery accepts such a newer effect checkpoint only when:

1. its audit history is at least as long as the prefix saved by K1.14;
2. the record at that saved prefix has exactly the saved audit root hash; and
3. its `next EffectIntentId` is exactly the frontier saved by K1.14.

This permits legitimate effect-only audit descendants while rejecting audit forks, rollback and a
later non-durable runtime cycle that allocated new effect identities.

## Recovery is exact continuation, not a semantic event

Recovery is bootstrap-only. A fresh runtime boots the same program, acquires an active fenced lease,
loads both checkpoint halves and validates them before publication.

K1.14 does **not** append a synthetic "recovery" event to replay identity. The restored event loop
continues from the saved replay state. Equal post-recovery causes therefore produce the same replay
identity as an uninterrupted execution from the same durable point.

## Fail-closed recovery

Recovery rejects, among other cases:

- a missing half of the runtime/effect bundle;
- an attempt to overwrite an existing runtime bundle before recovery establishes its lineage;
- stale fencing authority;
- a different canonical NAIR program;
- namespace mismatch;
- retry-policy mismatch;
- runtime checkpoint corruption;
- audit rollback or divergent audit ancestry;
- effect-intent frontier mismatch;
- atom/render shape mismatch;
- timer fire-budget mismatch;
- transaction/timer identity rollback behind boot state;
- atom-version or render-revision rollback behind boot state;
- recovery after the event loop has already started executing cycles.

## Canonical runtime format

K1.14 introduces the host-persistence format:

```text
NDRTSM01
version 1.0
```

The format is bounded, deterministic and protected by a SHA-256 digest with an explicit domain
separator. It is a corruption/integrity check relative to the stored bytes; K1.11 signed audit
anchors remain the separate cryptographic identity/attestation layer.

## API surface

Main K1.14 APIs:

```rust
AtomicEventLoop::semantic_checkpoint(...)
AtomicEventLoop::checkpoint_runtime_with_audited_journal(...)
AtomicEventLoop::recover_runtime_from_audited_journal(...)
AtomicEventLoop::cycle_to_with_runtime_checkpoint(...)
AtomicEventLoop::cycle_to_with_runtime_checkpoint_and_completions(...)
```

Host stores implement:

```rust
FencedRuntimeCheckpointStore
```

No filesystem, SQL database, cloud store or durability technology is built into the semantic core.
The host chooses that implementation and is responsible for honoring the atomic/durability contract
it advertises.

## Scope boundary

K1.14 certifies crash-consistent **semantic runtime state** when callers use the K1.14 durable APIs.
Legacy cycle APIs remain available and intentionally do not gain hidden persistence. Work performed
through a non-durable API after the latest K1.14 checkpoint may be rolled back to that last durable
point after a crash.

K1.14 does not persist arbitrary process memory, OS resources, host credentials or external-system
state, and it does not claim universal exactly-once delivery.

## NAIR status

K1.14 adds runtime durability, not new program semantics.

```text
NAIR = 0.6
```

There is no new opcode and `.noi` surface syntax remains unfrozen.

## Certification target

K1.14 adds **25 tests** to the 387 tests certified by K1.13, for an expected total of **412 tests**.
The package must pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

and the GitHub CI matrix on Linux, macOS and Windows before tag `k1.14` may be published.
