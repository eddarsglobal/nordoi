# NORDOI V0.6 — Core Computation & Functions Vertical Slice

Status: candidate specification. This document is subordinate to `CONSTITUTION.md`, `laws/LAW_0001_NORDOI_MASTER_LAW.md`, and every already-certified invariant through V0.5/N0.8.

## Purpose

V0.6 replaces a sequence of micro-milestones with one vertical production batch. It adds a closed, pure computation core from `.noi` source through semantic validation, canonical witness, proof-driven optimization, NAIR and the closed runtime.

## Source capability

V0.6 `core-run` accepts, after the existing optional module/type/effect prelude:

- immutable top-level `const` declarations;
- pure `fn name(param, ...) { ... }` declarations;
- immutable local `const` declarations inside function and conditional blocks;
- one `entry name returns expression;`;
- checked integer `+`, `-`, `*`, `/`;
- equality `==`, `!=` for values of the same kind;
- integer ordering `<`, `<=`, `>`, `>=`;
- boolean `&&`, `||`, `!`;
- pure function calls and nested calls;
- expression-form `if condition { ... } else { ... }`.

Integer literals remain canonical non-negative decimal i64 literals. Negative results may be produced by checked subtraction. Arithmetic overflow and division by zero fail closed before runtime publication.

## Function law

V0.6 functions are pure and inline-by-construction. They create no runtime call authority, stack frame, heap object, dynamic dispatch table or hidden effect. Direct or indirect recursion is rejected in V0.6. Maximums are bounded for functions, parameters, immutable bindings, expression nodes and call depth.

## Static-control law

All V0.6 programs are closed: there is no runtime input in this surface. Therefore all values and conditions are provable at compile time. Both `if` branches are semantically validated; only the proven result contributes to executable NAIR.

## Atomic optimization law

The complete semantic program is retained in the V0.6 witness. Operational NAIR is minimized independently. Every successfully closed V0.6 program lowers to exactly:

```text
CONST r0 <proved-result>
HALT
```

Therefore:

- runtime calls = 0;
- runtime branches = 0;
- function frames = 0;
- unused functions = 0 runtime instructions;
- unused immutable bindings = 0 runtime instructions;
- NAIR required minor = 0.6;
- persistent state/effects/authority = 0.

This deliberately does not add unused NAIR arithmetic/call opcodes. Dynamic data will justify such machinery only when a future certified capability makes it observably necessary.

## Canonicality

Comments, whitespace, source ID, source filename and function declaration order do not alter semantic identity. Function parameter order, function bodies, immutable binding definitions, entry name/expression and module/type/effect semantic identity do participate.

## Compatibility

All prior commands and certified surfaces remain available and unchanged. `--version` remains exactly:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

V0.6 introduces only the additive command:

```text
nordoi core-run <path|->
```
