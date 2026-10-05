# L0.6 Pure Result Intelligence Record

## Decision

The first observable source value is introduced as a literal semantic result, not as general expression
syntax and not as runtime output.

Chosen spelling:

```noi
entry main returns 42;
```

This keeps `returns` contextual and leaves the lexer keyword-free.

## Why not `return 42;` inside a block?

A block would force statement sequencing, scope, function bodies, termination rules, unreachable code,
control flow, and eventually local bindings. L0.6 does not need any of those to establish a pure result.

## Why not `entry main = 42;`?

Freezing `=` now would conflate declaration/assignment/equality/result binding before those concepts have
separate semantics.

## Why only canonical non-negative i64 decimal?

Negative values require a decision about whether `-` is part of the literal or a unary operator.
Separators, suffixes and alternate bases require additional lexical and type rules. L0.6 therefore takes
the smallest canonical integer surface that has a unique value mapping.

## Compatibility

L0.5 stays frozen. A result-bearing entry is intentionally rejected by L0.5/C0.3/C0.4/V0.1 until a
future compiler milestone explicitly adopts L0.6 result semantics.

## Future path

A safe sequence is:

```text
L0.6 pure result semantics
  ↓ future compiler result plan
  ↓ future explicit NAIR result representation
  ↓ future closed-runtime result execution
```

If NAIR needs an explicit result/return instruction, that should be introduced as a separately versioned
NAIR change rather than smuggling result semantics through atoms, rendering, effects, or host I/O.
