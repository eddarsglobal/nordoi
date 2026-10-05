# C0.9 Pure Binding Execution Plan — Intelligence Record

## Decision

After L0.8, NORDOI needs a compiler planning boundary before any binding-aware NAIR lowering.

The key architectural question is whether a named binding should immediately become runtime
storage. The constitutional answer is **no**.

L0.8 bindings are immutable semantic names. C0.9 therefore plans their use while keeping runtime
storage at zero.

## Why not lower directly?

Direct L0.8 -> NAIR lowering would mix three separate decisions:

1. source/name semantics;
2. execution planning;
3. operational representation.

That would weaken `IR Before Surface Lock-In`, make representation choices harder to change, and
risk turning source names into accidental runtime memory obligations.

C0.9 creates a stable semantic seam.

## Why preserve `BINDING(id)` rather than inline the value?

Inlining every binding value at planning time would be computationally valid for the current tiny
language, but semantically lossy.

For:

```noi
const x = 42;
const y = 42;
```

`x` and `y` are distinct canonical identities.

Preserving `BINDING(id)` allows future tooling, diagnostics, optimization proofs and lowering
strategies to distinguish semantic identity from numerical coincidence.

A future optimizer may prove that a binding can disappear operationally. That optimization should
be a later explicit theorem, not an implicit loss of identity in C0.9.

## Zero runtime storage

C0.9 introduces the explicit invariant:

```text
runtime_storage_item_count = 0
```

This is deliberately stronger than merely "runtime not invoked".

It records that the semantic existence of a binding does not itself request runtime state.

This directly serves:

- Atomic Speed;
- No Work Without Effect;
- What You Do Not Use Must Cost Nothing.

## Canonicality

Declaration order remains non-semantic because L0.8 already assigns IDs canonically by name.

C0.9 preserves:

- L0.8 witness;
- canonical binding IDs;
- exact postfix structure;
- result value.

Whitespace, comments, source IDs and spans remain outside canonical meaning.

## Future boundary

C0.9 intentionally leaves the next question open:

> How should a pure binding reference lower operationally?

A future compiler/NAIR milestone may choose, under constitutional review, among strategies such as:

- direct literal materialization;
- shared SSA value materialization;
- compile-time elimination;
- another representation justified by measured cost.

C0.9 does not prejudge that choice.

## Security

C0.9 introduces no new authority, I/O, host interaction, mutation or runtime persistence.

All invalid binding references and arithmetic overflow are rejected before plan publication.

The attack surface should therefore remain materially limited to compiler planning code and CLI
presentation, with existing frontend bounds inherited from L0.8.
