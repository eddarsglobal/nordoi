# NORDOI L0.7 — Pure Expression Intelligence Record

## Decision

Introduce the first computed source result as an additive semantic boundary rather than extending
L0.6 in place. The initial language is canonical non-negative `i64` literals, binary `+`, and
parenthesis grouping.

## Why postfix

A compact postfix semantic representation is preferable to freezing a large public AST:

- evaluation is iterative and naturally stack-based;
- source trivia and parenthesis tokens disappear while evaluation structure remains explicit;
- the representation is close to future IR lowering without being NAIR itself;
- long flat addition chains do not require recursive semantic evaluation;
- a strict node budget is easy to enforce;
- witness encoding is compact and deterministic.

The surface remains free to evolve because the stable contract is semantic operations, not private
Rust layout.

## Why preserve grouping

Although addition over the current non-negative checked domain produces the same mathematical sum
under reassociation when no overflow occurs, canonicalizing or sorting expressions now would create
a dangerous future law. When more operators, costs, values, or effects exist, evaluation structure
may matter. L0.7 therefore records exact postfix evaluation order and does not claim associativity or
commutativity as canonical language semantics.

## Overflow policy

Checked `i64` arithmetic is part of the milestone. Silent host overflow, debug/release divergence,
wraparound, and ambient numeric policy are prohibited. `i64::MAX + 1` is rejected before NSIR
publication.

## Compatibility strategy

L0.7 does not change L0.6, C0.5, C0.6, or V0.2. The new `expr` command is a separate inspection
boundary. This makes the next bridge explicit: a future compiler milestone may map certified postfix
operations to an execution plan/NAIR sequence, but that decision is not smuggled into L0.7.

## Security and resource bounds

The structural parser already caps group nesting. L0.7 additionally caps semantic expression nodes
at 1024. Evaluation is iterative and allocates only bounded vectors/stacks proportional to that
limit. No I/O, host authority, capability lookup, runtime call, dynamic plugin, network operation, or
filesystem inference is introduced.
