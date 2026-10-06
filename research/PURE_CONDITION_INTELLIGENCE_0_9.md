# Pure Condition Intelligence — L0.9

## Decision

The first control-flow prerequisite should be boolean meaning, not branching machinery. NORDOI therefore introduces a small pure-condition semantic boundary before any branch opcode, execution plan or runtime change.

## Why comparisons before `if`

A branch without a separately certified condition semantics would combine parsing, typing, comparison behavior, control flow, IR design and runtime execution in one change. That would make invalid states easier to create and harder to audit. L0.9 isolates truth production first.

## Why no logical combinators yet

`&&`, `||` and negation raise evaluation-order and future short-circuit questions. L0.9 avoids freezing those semantics prematurely. Six integer comparators and boolean literals are enough to establish the bool result type required by later control-flow work.

## Why integer literals only

Named bindings and arithmetic already have separate certified semantic histories. Folding them immediately into L0.9 would make the first boolean boundary depend on several widening rules at once. A later milestone can compose the independently certified layers.

## Canonical identity

Truth value alone is not sufficient identity. `true`, `1 < 2`, and `1 != 2` may all evaluate to true but are distinct semantic constructions and therefore have distinct L0.9 witnesses.

## Performance law

L0.9 performs compile-time semantic evaluation only. It adds no runtime object, no allocation, no storage, no lookup and no instruction. This keeps unused runtime cost at zero while establishing a future condition contract.

## Security

Malformed operators, non-canonical integers, chained comparisons, unsupported identifiers and ambiguous forms fail closed. Two-character comparators cannot contain hidden trivia. No declared effect becomes authority.

## Next possible frontier

After L0.9 certification, C0.11 can define a pure-condition execution plan while still avoiding branch semantics. Only after a stable plan exists should NAIR comparison representation be considered.
