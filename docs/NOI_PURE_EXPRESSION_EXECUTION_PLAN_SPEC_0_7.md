# NORDOI C0.7 — Pure Expression Execution Plan Specification

Status: **candidate** until the local Release Gate, exact-SHA GitHub CI, annotated tag `c0.7`, and release proof all pass.

## 1. Purpose

C0.7 turns the already-validated L0.7 pure-expression semantics into a compiler-owned execution plan without lowering to NAIR or invoking the runtime.

The input boundary is:

```text
NsirPureExpressionUnit (L0.7)
```

The output boundary is:

```text
PureExpressionExecutionPlan (C0.7)
```

C0.7 introduces no new source syntax and does not modify any earlier canonical witness.

## 2. Accepted semantics

C0.7 accepts exactly the semantics already published by L0.7:

- empty body,
- `entry Name;`,
- `entry Name returns <L0.7 pure expression>;`.

The L0.7 expression subset remains:

- canonical non-negative `i64` literals,
- binary `+`,
- parentheses,
- checked arithmetic,
- bounded postfix semantic representation.

No variable, call, branch, function, subtraction, multiplication, division, I/O, effect operation, capability or host authority is added.

## 3. Planned representation

A planned expression contains only:

```text
ops:  canonical L0.7 postfix operations
value: validated i64 result
```

Example:

```noi
entry main returns 20 + 22;
```

is planned as:

```text
[INT(20), INT(22), ADD] => INT(42)
```

The plan preserves evaluation structure. It does not collapse the expression to the value alone.

Therefore these two sources remain distinct in C0.7 identity even though both evaluate to `6`:

```noi
entry main returns (1 + 2) + 3;
entry main returns 1 + (2 + 3);
```

## 4. Work, effects and authority

C0.7 defines no runtime work items.

```text
work_item_count = 0
required_effects = []
host_authority = NONE
```

The postfix operations are compiler semantic structure, not runtime work items.

If an L0.7 entry is manually constructed with any required effect, C0.7 fails closed before publishing a plan.

Effect resolution is not authority.

## 5. Canonical witness

C0.7 adds:

```text
canonical_c07_bytes()
```

with domain:

```text
NORDOI-C0.7-PURE-EXPRESSION-PLAN\0
```

The witness commits to:

- exact L0.7 witness bytes,
- plan form,
- entry name,
- optional canonical postfix operation sequence,
- optional validated result,
- zero runtime work count,
- resolved effect IDs,
- no-host-authority flag.

It excludes:

- source IDs,
- file names and paths,
- source spans,
- comments and whitespace,
- CLI state,
- runtime state,
- host authority,
- ambient environment.

`canonical_l07_bytes()` is not changed.

## 6. No lowering and no runtime

C0.7 does **not** define expression-to-NAIR lowering.

It does not change:

- NAIR 0.6,
- C0.6 pure-result lowering,
- V0.2 pure-result execution,
- C0.4 lowering,
- V0.1 execution.

The certified `result`, `result-plan`, `result-lower` and `result-run` commands continue to reject L0.7 addition syntax.

C0.7 adds only:

```text
nordoi expr-plan <path|->
```

## 7. Validation-before-publication

The public transition is:

```text
compile_pure_expression_execution_plan_boundary(source)
  -> compile_pure_expression_boundary(source)
  -> validate_pure_expression_execution_plan(...)
  -> PureExpressionExecutionPlan
```

No plan is published before L0.7 validation and C0.7 purity validation have succeeded.

## 8. Release law

Certification requires, in order:

1. `cargo fmt --all`
2. `./scripts/release_gate.sh`
3. `NORDOI release gate: PASS`
4. commit and push
5. GitHub Actions queried by the exact commit SHA
6. five green jobs
7. annotated tag `c0.7`
8. pushed tag proof

Until then C0.7 remains a candidate.
