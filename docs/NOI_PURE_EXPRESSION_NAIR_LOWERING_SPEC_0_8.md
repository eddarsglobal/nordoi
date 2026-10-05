# NORDOI C0.8 — Pure Expression → NAIR Lowering Foundation

Status: **candidate** until the local Release Gate, exact-SHA GitHub CI, annotated tag `c0.8`, and release proof all pass.

## 1. Purpose

C0.8 lowers the certified C0.7 pure-expression execution plan to canonical NAIR while preserving the exact L0.7 postfix calculation order.

The input boundary is:

```text
PureExpressionExecutionPlan (C0.7)
```

The output boundary is:

```text
PureExpressionNairArtifact (C0.8)
```

C0.8 introduces no new source syntax, no runtime execution, no I/O, no effect dispatch, and no authority.

## 2. Faithful postfix lowering

C0.8 performs no hidden constant folding.

Each postfix node lowers one-for-one:

```text
INT(v) -> CONST fresh_register, INT(v)
ADD    -> ADD_INT_CHECKED fresh_register, lhs, rhs
```

The program always ends in:

```text
HALT
```

Example:

```noi
entry main returns 20 + 22;
```

L0.7/C0.7 postfix:

```text
[INT(20), INT(22), ADD]
```

C0.8 NAIR:

```text
CONST r0, INT(20)
CONST r1, INT(22)
ADD_INT_CHECKED r2, r0, r1
HALT
```

The final result register is `r2`.

## 3. Register law

Register allocation is deterministic SSA order.

- `r0` is the first lowered postfix node,
- every later postfix node receives the next fresh register,
- registers are never overwritten,
- `ADD` consumes the two top postfix stack registers in `lhs`, `rhs` order,
- the single final stack register is the source-level result register.

For a valid expression with `N` postfix nodes, C0.8 emits exactly `N + 1` instructions including `HALT`.

## 4. Grouping is preserved

C0.8 preserves the C0.7 plan structure rather than only the final value.

Therefore:

```noi
entry main returns (1 + 2) + 3;
```

and:

```noi
entry main returns 1 + (2 + 3);
```

both evaluate to `6`, but produce distinct NAIR instruction streams and distinct C0.8 witnesses.

## 5. NAIR version selection

C0.8 relies on certified N0.7 minimal-required-minor encoding.

- no expression / `entry Name;` -> `HALT` -> NAIR 0.6,
- literal-only expression -> `CONST; HALT` -> NAIR 0.6,
- any expression using `ADD_INT_CHECKED` -> NAIR 0.7.

C0.8 does not rewrite old C0.4/C0.6 NAIR bytes.

## 6. Pure zero-authority boundary

C0.8 accepts only plans satisfying:

```text
work_item_count = 0
required_effects = []
host_authority = NONE
```

Effect declaration is not effect requirement. Effect resolution is not authority.

## 7. Canonical witness

C0.8 adds:

```text
canonical_c08_bytes()
```

with domain:

```text
NORDOI-C0.8-PURE-EXPRESSION-NAIR\0
```

The witness commits to:

- exact C0.7 witness bytes,
- optional final result-register binding,
- exact canonical NAIR bytes.

It excludes source IDs, file paths, spans, comments, whitespace, runtime state, ambient environment, capabilities, and host authority.

Earlier witnesses are unchanged.

## 8. Validation before publication

The public transition is:

```text
compile_pure_expression_nair_boundary(source)
  -> compile_pure_expression_execution_plan_boundary(source)
  -> lower_pure_expression_plan_to_nair(plan)
  -> NairProgram::validate()
  -> NairProgram::canonical_bytes()
  -> PureExpressionNairArtifact
```

No artifact is published before all validation succeeds.

## 9. CLI

C0.8 adds only:

```text
nordoi expr-lower <path|->
```

The certified `result-lower` and `result-run` boundaries remain frozen and continue to reject L0.7 addition syntax.

## 10. Release law

Certification requires, in order:

1. `cargo fmt --all`
2. `./scripts/release_gate.sh`
3. `NORDOI release gate: PASS`
4. commit and push
5. GitHub Actions queried by exact commit SHA
6. five green jobs
7. annotated tag `c0.8`
8. pushed tag proof

Until then C0.8 remains a candidate.
