# NORDOI C0.11 — Pure Condition Execution Plan

Status: Candidate
Track: Compiler / semantic planning
Depends on: certified L0.9 Pure Boolean & Comparison Foundation

## Governance

This milestone is subordinate to `CONSTITUTION.md`, then to
`laws/LAW_0001_NORDOI_MASTER_LAW.md`, then to already-certified semantic contracts.
It does not restate or weaken those laws.

## Purpose

C0.11 introduces the first execution-plan representation for a pure boolean condition.
It turns validated L0.9 condition semantics into a canonical compiler plan while deliberately
leaving control flow, NAIR lowering and runtime execution undefined.

The milestone exists so future control-flow work can consume a stable boolean-plan contract
instead of coupling source syntax directly to branches or runtime behavior.

## Accepted source surface

Exactly the L0.9 surface remains accepted:

```noi
entry main returns true;
entry main returns false;
entry main returns 20 <= 22;
```

C0.11 introduces no new source syntax.

## Plan forms

A `PureConditionExecutionPlan` has one of two forms:

- `Empty`
- `Entry(PureConditionEntryPlan)`

An entry plan contains:

- the canonical semantic entry name;
- the resolved effect set;
- optionally one `PlannedPureCondition`.

A planned condition retains the exact L0.9 semantic condition:

- `Bool(value)`; or
- `IntCompare { lhs, comparator, rhs }`.

It also records the deterministic boolean result.

## Required invariants

C0.11 MUST preserve the exact L0.9 witness carried by the plan.

C0.11 MUST NOT collapse semantically distinct conditions merely because they have the same truth value.
For example `true` and `1 < 2` both evaluate to `true`, but retain distinct C0.11 identity.

C0.11 MUST report:

```text
work = 0
runtime storage = 0
host authority = false
```

A declared but unused effect MUST NOT become an execution requirement.

## Canonical witness

The C0.11 witness domain is:

```text
NORDOI-C0.11-PURE-CONDITION-PLAN\0
```

The witness commits to:

- exact L0.9 canonical bytes;
- plan form;
- entry identity;
- exact condition form, operands and comparator;
- boolean result;
- zero-work count;
- zero-runtime-storage count;
- resolved effect requirements;
- absence of host authority.

Source IDs, source spans, comments, whitespace, filesystem paths, host authority and runtime state
are excluded from semantic identity unless already committed by a lower certified semantic layer.

## Explicit non-goals

C0.11 defines none of the following:

- `if` / `else` syntax;
- branching;
- loops;
- short-circuit logic;
- boolean operators;
- condition bindings;
- NAIR comparison opcodes;
- NAIR branch opcodes;
- runtime execution;
- runtime storage;
- I/O;
- effects or capabilities;
- authority.

## CLI witness

The additive inspection command is:

```text
nordoi condition-plan <path|->
```

It MUST state that branching/lowering are undefined and that NAIR/runtime remain untouched.

## Release requirement

C0.11 is not certified until all of the following are true:

1. `cargo fmt --all -- --check` passes;
2. `cargo clippy --all-targets -- -D warnings` passes;
3. `cargo check --all-targets` passes;
4. `cargo test --all-targets` passes;
5. GitHub CI is green for the exact commit SHA on all required jobs;
6. only after that proof, annotated tag `c0.11` is pushed.
