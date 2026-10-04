# NORDOI L0.5 — Minimal Body Intelligence Record

## Decision

Introduce one contextual source form, `entry Name;`, plus an empty body, as the first body semantics.
Do not introduce general functions yet.

## Why an entry declaration first

NORDOI needs a first body construct before it can responsibly lower source toward execution. A full
function grammar would force decisions about parameters, return types, calls, blocks, local storage,
control flow, effect annotations and ABI all at once. Those decisions are not needed to prove the
compiler architecture.

A zero-work entrypoint is sufficient to prove:

- complete residual-body recognition;
- source-backed body diagnostics;
- HIR body representation;
- NSIR body representation;
- canonical identity above C0.2;
- purity by construction;
- compatibility with the semantic registry;
- a future seam for executable-plan and NAIR lowering.

## Contextual, not lexical, `entry`

`entry` remains an L0.1 `Identifier`. It is interpreted only as the first significant element of the
L0.4 residual body. This avoids expanding a global reserved-word set before the surface language is
mature.

## Why fail closed

L0.5 accepts only bodies it understands completely. Silently publishing a partly interpreted body
would create false semantic certainty and could later conflict with new syntax. Unsupported bodies
therefore fail at the L0.5 boundary while the older C0.2 boundary remains available for opaque
inspection.

## Why zero effects

L0.5 has no effect-operation syntax. Giving `entry` declared effects anyway would create a signature
feature before there is anything capable of using it. The entry therefore has an explicitly empty
resolved effect set. This also reinforces that declaration/resolution and authority are separate.

## Why wrap C0.2 instead of rewriting it

The C0.2 `NsirUnit` still reports its residual body as `UNLOWERED`. L0.5 introduces `NsirBodyUnit`
that wraps that certified semantic unit and proves that this particular residual body has been fully
understood by a later layer. This preserves old witnesses and APIs exactly instead of mutating their
meaning after certification.

## Canonical identity

L0.5 adds a new domain-separated witness rather than altering C0.1/L0.4/C0.2 witnesses. Formatting,
comments, source IDs and spans remain diagnostic metadata and do not affect identity.

## Alternatives rejected

### `fn main() {}`
Rejected for L0.5 because it prematurely commits NORDOI to function/block/call semantics.

### Treat any identifier as an implicit entry
Rejected because implicit semantics are hostile to auditability and canonical compilation.

### Infer entry from filename/module name
Rejected because filesystem metadata is not program semantics.

### Allow arbitrary source after `entry`
Rejected because that would publish partial body semantics.

### Attach every declared effect to the entry
Rejected because declaration is not use, requirement or authority.

## Security consequences

The accepted body has no operation that can touch host state. Unsupported syntax fails closed. Entry
names inherit the bounded ASCII executable identifier profile. No authority is serialized into the
body witness.

## Next question

The next compiler milestone should define a validated executable semantic plan for the already-known
`EMPTY` and pure `ENTRY` forms. Only after that should a narrowly scoped NSIR -> NAIR lowering be
considered.
