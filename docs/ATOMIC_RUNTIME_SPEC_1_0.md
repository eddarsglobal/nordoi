# NORDOI Atomic Runtime Specification 1.0

## Status

This document defines the first closed NORDOI runtime capsule delivered by K1.0.
It does not freeze the future surface language and it does not define a persistent
application event loop. It defines one complete, isolated and reproducible activation.

## Runtime boundary

A closed activation is defined by exactly two semantic inputs:

1. one validated canonical `NairProgram`;
2. one canonicalized `InputBatch` supplied by an authorized host adapter.

The runtime creates fresh NAM and Atomic Render Core instances for the activation.
No previous activation state is implicitly imported.

## Canonical input boundary

`InputBatch::canonicalized()` performs the same numeric normalization as
`AtomicInputCore::submit()` and requires strictly increasing `InputSequence` values.
This closes the gap created by the public `InputBatch` structure: manually assembled
batches cannot bypass finite-value, axis-range or XR-orientation normalization.

`InputBatch::canonical_bytes()` defines the deterministic semantic identity used by
the K1.0 replay mechanism. It is not a platform packet format.

## Closed execution

`AtomicRuntime::execute(program, input)` performs the following sequence:

```text
NAIR canonical validation
        ↓
InputBatch canonicalization
        ↓
Replay-key derivation
        ↓
fresh NAM + fresh Atomic Render Core
        ↓
execute_nair_with_render_and_input
        ↓
HALT / final causal flush
        ↓
quiescence check
        ↓
final semantic atom snapshot
        ↓
RuntimeReport
```

`run_closed(program, input)` is the convenience entry point.

## Error isolation

The runtime owns all mutable activation state locally. If the activation fails,
that local state is dropped and no `RuntimeReport` is returned. K1.0 therefore does
not expose a partially successful closed activation as a valid result.

This does not mean every NAIR instruction is globally transactional. It means the
closed runtime capsule does not publish its internal partially-mutated state after
an error.

## Quiescent success

A successful closed activation MUST end with:

- zero pending NAM work;
- zero pending render nodes.

Residual work is a runtime error. This makes successful K1.0 reports complete
activation boundaries rather than arbitrary mid-flight snapshots.

## Final atom snapshot

Every atom created through a semantic `AtomSlot` is captured as:

- runtime `AtomId`;
- final atom version;
- final `Value`.

The snapshot is keyed by `AtomSlot` in deterministic `BTreeMap` order.

## Replay key

`RuntimeReplayKey` is derived from:

- a domain separator identifying NORDOI Atomic Runtime 1.0;
- canonical NAIR bytes;
- canonical input bytes.

The current implementation uses a stable FNV-1a-based 64-bit fingerprint.
It exists for deterministic replay identity and regression diagnostics only.
It is **not** a cryptographic hash, signature, authenticity proof or security token.

Two executions with equal canonical program and canonical input MUST produce the
same replay key. A replay key collision remains theoretically possible and MUST NOT
be used as a security decision.

## Determinism scope

K1.0 requires deterministic behavior for the bootstrap kernel semantics currently
represented by NAM, NAIR, Atomic Input Core and Atomic Render Core. Platform backends,
external effects, clocks, randomness, networking, storage and future parallel/GPU
execution require their own explicit deterministic or effect-governed contracts.

## Non-goals

K1.0 intentionally does not yet define:

- persistent multi-activation application sessions;
- source-language syntax;
- a final compiler frontend;
- OS event polling;
- network/filesystem/process effects;
- a cryptographic replay format;
- persistent save-state serialization.

Those layers must build on the certified K1.0 causal model rather than bypass it.
