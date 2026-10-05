# NORDOI C0.9 — Pure Binding Execution Plan Specification

Status: candidate until certified by the NORDOI release process.

## 1. Governance

This milestone is governed by `CONSTITUTION.md`, then by
`laws/LAW_0001_NORDOI_MASTER_LAW.md`, then by already certified semantic contracts.

C0.9 must preserve, in particular:

- Atomic Speed;
- Zero Legacy Debt;
- Impossible States First;
- No Work Without Effect;
- What You Do Not Use Must Cost Nothing;
- Canonical Semantics;
- Validate Before Execute;
- authority != intent;
- safe by omission.

C0.9 does not amend or duplicate those laws.

## 2. Purpose

C0.9 converts the fully resolved L0.8 immutable pure-binding semantics into a canonical
execution plan.

It answers:

> What pure computation is intended, which canonical binding identities participate, and
> what result is implied?

It deliberately does **not** answer:

> How are bindings represented in NAIR or runtime memory?

That remains undefined after C0.9.

## 3. Source boundary

C0.9 consumes the certified L0.8 boundary.

Example:

```noi
const x = 20;
const y = 22;

entry main returns x + y;
```

L0.8 resolves canonical binding IDs:

```text
#1 x = INT(20)
#2 y = INT(22)
```

and the expression:

```text
[BINDING(1), BINDING(2), ADD]
```

C0.9 preserves that exact identity and postfix order in the plan.

## 4. Plan model

C0.9 defines:

```text
PlannedPureBindingExpression
PureBindingEntryPlan
PureBindingPlanForm
PureBindingExecutionPlan
```

The plan contains the complete L0.8 semantic unit and an explicit plan form.

For an expression it preserves:

- canonical binding references;
- literal nodes;
- addition nodes;
- exact postfix structure;
- computed checked-i64 result.

It must not silently rewrite a binding reference into an integer literal.

## 5. Zero-storage law

A L0.8 `const` declaration is an immutable semantic binding, not a runtime storage request.

Therefore C0.9 defines:

```text
work_item_count()            = 0
runtime_storage_item_count() = 0
required_effects()           = []
requires_host_authority()    = false
```

This is a plan invariant.

C0.9 does not allocate:

- atoms;
- globals;
- heap slots;
- stack slots;
- registers;
- domains;
- transactions;
- capability slots;
- runtime objects.

A future lowering milestone may choose a representation only if it remains conformant with the
Constitution and the certified C0.9 semantics.

## 6. Binding identity preservation

Two bindings with the same value are still distinct semantic identities.

Example:

```noi
const x = 42;
const y = 42;
entry main returns x;
```

is distinct from:

```noi
const x = 42;
const y = 42;
entry main returns y;
```

even though both evaluate to `42`.

Likewise a binding reference and an equal literal remain structurally distinct:

```noi
const x = 42;
entry main returns x;
```

is distinct from:

```noi
const x = 42;
entry main returns 42;
```

C0.9 must preserve those distinctions.

## 7. Canonical witness

The C0.9 witness domain is:

```text
NORDOI-C0.9-PURE-BINDING-PLAN\0
```

`canonical_c09_bytes()` commits to:

- the complete L0.8 canonical witness;
- plan form;
- entry name if present;
- optional postfix expression;
- exact semantic op sequence;
- computed result;
- binding count;
- zero work count;
- zero runtime-storage count;
- resolved effect IDs;
- no-authority flag.

It excludes:

- source IDs;
- source paths;
- source spans;
- comments;
- whitespace;
- CLI output;
- ambient environment;
- runtime observations.

## 8. Validation before publication

`compile_pure_binding_execution_plan_boundary()` must:

1. compile and validate L0.8;
2. reject any invalid L0.8 source;
3. require an entry plan to be pure;
4. construct the C0.9 plan;
5. publish only the validated plan.

Unknown bindings, duplicate bindings and checked-add overflow fail before plan publication.

## 9. Compatibility

C0.9 is additive.

Frozen boundaries remain frozen:

- L0.8 `bindings` remains semantic-only;
- C0.7 `expr-plan` does not accept L0.8 `const` source;
- C0.8 `expr-lower` does not accept L0.8 `const` source;
- V0.3 `expr-run` does not accept L0.8 `const` source;
- NAIR remains unchanged;
- runtime remains unchanged;
- kernel remains unchanged;
- the certified `--version` string remains unchanged.

## 10. CLI

C0.9 adds:

```text
nordoi bindings-plan <path|->
```

It reports the canonical binding registry, exact postfix plan, result, zero work,
zero storage, zero effects, no authority, L0.8 witness and C0.9 witness.

It must report:

```text
lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED
```

## 11. Explicit non-goals

C0.9 does not define:

- binding-to-register lowering;
- constant folding policy;
- runtime storage;
- mutable variables;
- assignment;
- general expressions;
- functions;
- calls;
- branches;
- loops;
- I/O;
- effects;
- capabilities;
- host authority;
- NAIR changes;
- runtime execution.

## 12. Certification requirements

Certification requires:

1. local `cargo fmt --all`;
2. local `./scripts/release_gate.sh` PASS;
3. exact-SHA GitHub `NORDOI Test Gate`;
4. five required jobs green:
   - Check
   - Format & Clippy
   - Tests (ubuntu-latest)
   - Tests (macos-latest)
   - Tests (windows-latest)
5. annotated tag `c0.9`.

Until all five conditions are proven, C0.9 remains a candidate.
