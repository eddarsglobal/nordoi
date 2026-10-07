# NORDOI V0.8 — Dynamic Control Flow Vertical Slice

Status: candidate specification for V0.8 certification.

## Purpose

V0.8 introduces the first NORDOI control-flow decision that is intentionally unknown at compile time and therefore must be resolved during runtime execution.

The milestone is additive over certified V0.7. Existing V0.7 dynamic input/arithmetic/comparison behavior remains unchanged.

## Surface

V0.8 adds the `branch-run` vertical slice. It accepts the V0.7 body surface plus a structured expression:

```noi
if <bool-expression> {
    <expression>
} else {
    <expression>
}
```

Example:

```noi
input key_code;

entry main returns if key_code > 40 {
    100
} else {
    200
};
```

With canonical keyboard key-code `41`, the result is `INT(100)`. With key-code `39`, the result is `INT(200)`.

## Static-erasure law

If the condition is compile-time known, no runtime branch may be emitted. Only the selected branch is lowered.

```noi
entry main returns if 2 > 1 { 42 } else { 7 };
```

MUST lower to:

```text
CONST r0 INT(42)
HALT
```

and remain NAIR `0.6`.

A static condition may select a dynamic expression. In that case the selected dynamic expression survives, but runtime branch cost remains zero.

## Dynamic-branch law

When the condition cannot be known until runtime, V0.8 emits exactly the structured branch primitive needed for the decision.

The V0.8 vertical slice requires both dynamic branch arms to be statically reducible and to produce the same value kind (`INT` or `BOOL`). The unselected value is not published as a register result.

A canonical example lowers to:

```text
READ_INPUT_KEY_CODE r0 event=0
CONST r1 INT(40)
INT_GT r2 r0 r1
BRANCH_VALUE r3 cond=r2 then=INT(100) else=INT(200)
HALT
```

The runtime report records `runtime-branches=1`.

## Why structured branch instead of general jumps

V0.8 does not add arbitrary jump targets, mutable instruction pointers, a general control-flow graph executor, runtime function frames, recursion, or a VM stack.

`BRANCH_VALUE` is a bounded structured decision. It is sufficient to prove dynamic path selection while preserving the Atomic Machine principle that runtime machinery exists only where dynamic semantics require it.

## NAIR 0.10

V0.8 adds:

```text
BRANCH_VALUE dst,condition,then_value,else_value
```

Canonical opcode: `0x09`.

Rules:

- `condition` MUST reference an already-defined `BOOL` register;
- `then_value` and `else_value` MUST have the same kind;
- V0.8 permits branch values only of kind `INT` or `BOOL`;
- `dst` is a new SSA register;
- executing the instruction increments the runtime branch count by exactly one;
- programs containing `BRANCH_VALUE` require NAIR minor `0.10`.

Earlier minors remain frozen: base `0.6`, integer arithmetic `0.7`, integer comparison `0.8`, input register `0.9`.

## Determinism and authority

The condition may depend only on explicit semantic values, including the canonical input boundary introduced in V0.7. `BRANCH_VALUE` grants no authority and cannot read host state.

Same source + same canonical input MUST produce the same NAIR bytes, replay key, selected result, branch count, witness, and receipt.

## Fail-closed rules

V0.8 rejects:

- non-boolean `if` conditions;
- branch result-kind mismatch;
- dynamic branch arms that are not statically reducible in the V0.8 slice;
- unsupported branch-value kinds;
- unknown names;
- duplicate dynamic-scope names;
- constants depending on runtime input;
- missing/non-keyboard canonical input when required;
- runtime integer overflow in retained dynamic arithmetic.

## Compatibility

The public certified version string remains intentionally frozen:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

The `branch-run` report exposes the actual required artifact minor as `nair-minor=0.10`.

## Certification gate

V0.8 may be tagged only after:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo check --all-targets`
4. `cargo test --all-targets`
5. V0.8 focused tests
6. true-path smoke proof (`41 -> INT(100)`)
7. false-path smoke proof (`39 -> INT(200)`)
8. exact-SHA multi-platform GitHub CI
9. annotated `v0.8` tag
