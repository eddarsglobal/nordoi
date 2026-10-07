# NAIR 0.14 — Structured Call Control

## Status

Candidate additive NAIR minor for NORDOI V1.2.

## Purpose

NAIR 0.14 adds selective structured control inside bounded runtime call bodies without introducing jumps, labels, a general instruction pointer, or an unbounded VM stack.

## New CallExpr form

```text
IfElse {
    condition: CallExpr,
    then_expr: CallExpr,
    else_expr: CallExpr,
}
```

Canonical expression tag: `0x0b`.

## Validation

A valid structured call control expression must satisfy all existing call-expression limits plus:

1. `condition` has BOOL kind;
2. `then_expr` and `else_expr` have identical result kind;
3. expression node and nesting bounds are respected;
4. nested direct calls still satisfy the acyclic call-depth bound;
5. the enclosing `CALL_EVAL` retains explicit direct function identity and explicit argument registers.

## Runtime semantics

Execution is selective:

1. evaluate `condition`;
2. require BOOL;
3. increment runtime branch count exactly once;
4. evaluate only the selected arm;
5. do not evaluate the discarded arm.

The discarded arm performs zero runtime expression work and cannot produce selected-path calls or selected-path arithmetic failures.

## Compatibility

- NAIR 0.12 remains the minimum minor for simple `CALL_EVAL`.
- NAIR 0.13 remains the minimum minor for nested direct-call expressions.
- NAIR 0.14 is required only when a `CallExpr::IfElse` is present.
- NAIR 0.6 remains the base result for fully static erasure.

## Constitutional exclusions

NAIR 0.14 does not add:

- arbitrary jumps;
- labels;
- general CFG execution;
- recursion;
- indirect calls;
- function pointers;
- dynamic dispatch;
- an unbounded call stack;
- ambient authority.
