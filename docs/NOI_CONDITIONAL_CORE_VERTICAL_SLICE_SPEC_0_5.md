# NORDOI V0.5 — Conditional Core Vertical Slice

**Status:** Candidate

## Governance

This batch is subordinate to `CONSTITUTION.md` as the supreme NORDOI authority, then to `laws/LAW_0001_NORDOI_MASTER_LAW.md`, and to every certified invariant through N0.8. It does not amend or duplicate those laws.

## Purpose

V0.5 deliberately replaces several micro-milestones with one certifiable vertical slice. It closes two source-to-runtime paths:

1. pure boolean/comparison execution;
2. pure compile-time `if/else` execution with immutable bindings.

## Direct pure condition execution

Accepted certified L0.9 forms such as:

```noi
entry main returns true;
entry main returns 20 <= 22;
```

are lowered from the certified C0.11 plan to canonical NAIR and executed through the existing closed observed runtime.

- boolean literals use existing `CONST BOOL` and remain NAIR 0.6;
- integer comparisons use the corresponding certified N0.8 comparison opcode and require NAIR 0.8;
- execution remains pure, authority-free and quiescent.

## Static conditional expression

V0.5 additionally accepts the additive experimental form:

```noi
const value = 20;
const limit = 22;

entry main returns if value < limit {
    value + 22
} else {
    0
};
```

Current V0.5 static-condition operands are canonical non-negative `i64` literals or immutable L0.8 binding names. Comparators are `==`, `!=`, `<`, `<=`, `>`, `>=`. Branches are L0.8 pure integer expressions: literals, binding references, `+`, and parenthesized grouping.

## Compile-time branch law

Both branches MUST be semantically validated before selection. A dead branch is not permitted to hide an invalid binding reference, overflow, malformed expression or unsupported construct.

After both branches validate, the pure condition is resolved at compile time. Only the selected branch is converted to the existing C0.9/C0.10 binding plan/lowering. The non-selected branch emits exactly zero NAIR instructions.

Therefore V0.5 defines:

```text
runtime branch count       = 0
dead branch instruction count = 0
```

This is an intentional application of Atomic Speed, No Work Without Effect and What You Do Not Use Must Cost Nothing.

## Runtime boundary

V0.5 introduces no new runtime state, atom, domain, transaction, render frame, input bridge, effect, capability or host authority. Existing closed runtime execution is reused unchanged.

## Canonical evidence

Domains:

```text
NORDOI-V0.5-CONDITION-NAIR\0
NORDOI-V0.5-CONDITION-EXECUTION-RECEIPT\0
NORDOI-V0.5-STATIC-IF-PLAN\0
NORDOI-V0.5-STATIC-IF-LOWERING\0
NORDOI-V0.5-STATIC-IF-EXECUTION-RECEIPT\0
```

Operationally identical selected branches MAY produce the same NAIR/replay key even when dead-branch semantics differ. The V0.5 plan/lowering/receipt MUST still preserve the complete validated source-level conditional identity.

## CLI

```text
nordoi condition-run <path|->
nordoi if-run <path|->
```

Historical CLI version text remains frozen.

## Explicit non-goals

V0.5 does not introduce dynamic runtime branching, loops, mutation, general functions, parameters, I/O, effects, authority, or a stable final surface syntax. Dynamic control flow requires a later governed NAIR/runtime design and MUST NOT be faked when compile-time elimination is possible.
