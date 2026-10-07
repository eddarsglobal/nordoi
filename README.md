# NORDOI V1.2 — Nested Structured Runtime Control in Function Bodies

V1.2 is additive over certified V1.1. It allows a bounded pure runtime function body to contain structured `if/else` control while preserving NORDOI's acyclic call-graph law, selective evaluation, deterministic witnesses, zero hidden authority, and no general VM stack.

Minimal V1.2 example:

```noi
fn bias(x) returns if x > 40 {
    x + 100
} else {
    x + 200
};

input key_code;
entry main returns bias(key_code);
```

Run it with:

```bash
cargo run --quiet --bin nordoi -- function-control-run program.noi 41
```

Expected TRUE-path proof: `INT(141)`, `runtime-calls=1`, `runtime-branches=1`, `max-call-depth=1`, `nair-minor=0.14`, authority `NONE`, input boundary `CANONICAL`.

With key-code `39`, only the else arm executes and the result is `INT(239)`.

## NAIR 0.14

NAIR 0.14 extends `CallExpr` with one structured primitive:

```text
IF(condition, then_expr, else_expr)
```

This expression is valid only inside bounded `CALL_EVAL` bodies. The condition must evaluate to BOOL, both arms must have the same value kind, and exactly one arm is evaluated at runtime.

Compatibility remains additive:

- simple direct runtime call -> NAIR 0.12;
- acyclic nested direct calls -> NAIR 0.13;
- structured runtime control inside a call body -> NAIR 0.14;
- fully static programs -> base NAIR 0.6 `CONST + HALT`.

## Selective execution law

For a dynamic function-body `if`:

- the condition executes once;
- `runtime-branches` increments once;
- exactly one arm executes;
- the discarded arm performs zero runtime expression work;
- calls in an unselected arm do not execute;
- overflow in an unselected arm is not observed;
- selected-arm failures still fail closed.

## Call-graph law remains frozen

V1.2 does not introduce recursion, indirect dispatch, function pointers, arbitrary jumps, or an unbounded call stack. The complete direct call graph remains statically known, acyclic, and bounded to a maximum call depth of 8.

The public certified tool boundary intentionally remains:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```
