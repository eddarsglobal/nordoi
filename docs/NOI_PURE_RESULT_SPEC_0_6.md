# NORDOI L0.6 — Pure Result Foundation

Status: **candidate** until the Testing & Release Law is satisfied.

## 1. Scope

L0.6 adds the first source-level pure result without changing the certified L0.5 boundary.

The only new body spelling is:

```noi
entry Name returns DecimalInteger;
```

`entry` and `returns` are contextual identifiers. The lexer does not gain keywords.

L0.6 does **not** define expressions, operators, variables, calls, parameters, function return types,
control flow, I/O, effects, host authority, NAIR lowering, or runtime execution.

## 2. Compatibility law

The certified L0.5 boundary remains unchanged:

```noi
entry main;
```

is still understood by L0.5, while:

```noi
entry main returns 42;
```

must still fail at the L0.5 `body` boundary. L0.6 is exposed separately through
`analyze_pure_result_unit`, `compile_pure_result_boundary`, and `nordoi result`.

No earlier canonical witness is redefined.

## 3. Accepted body forms

At the L0.6 boundary, after the optional module declaration and L0.4 type/effect prelude, the body is
exactly one of:

1. empty/trivia-only,
2. `entry Name;`,
3. `entry Name returns DecimalInteger;`.

Any additional significant token fails closed.

Comments and ASCII whitespace may appear between significant elements.

## 4. Integer literal profile

The L0.6 result literal is deliberately narrower than a future numeric language.

Accepted spelling:

```text
0
[1-9][0-9]*
```

The value must fit in the non-negative range `0..=i64::MAX`.

Rejected at L0.6:

- negative signs,
- explicit `+`,
- leading zeroes other than the literal `0`,
- `_` separators,
- suffixes,
- hexadecimal/octal/binary prefixes,
- decimals/floats,
- arithmetic expressions.

This avoids freezing general operator, sign, numeric-base, suffix, coercion, overflow, or constant-folding
semantics prematurely.

## 5. Semantic model

L0.6 publishes a compiler-owned pure-result unit:

```text
NsirPureResultUnit
  semantic: certified C0.2 registry semantics
  form:
    EMPTY
    ENTRY(name, required_effects = {}, result = NONE | INT(i64))
```

A result entry requires zero semantic effects. Declared effects in the module do not become entry
requirements and do not grant authority.

`returns 42` is semantic data only. It performs no work.

## 6. Canonical identity

L0.6 adds:

```text
canonical_l06_bytes()
```

with domain:

```text
NORDOI-L0.6-PURE-RESULT\0
```

The witness commits to:

- certified C0.2 semantic registry identity,
- body form,
- entry name when present,
- resolved required-effect IDs (currently empty),
- optional integer result value.

It excludes:

- source ID,
- file/path name,
- source spans,
- whitespace/comments,
- host authority,
- runtime state,
- CLI state.

## 7. Execution boundary

L0.6 does not alter C0.3, C0.4, or V0.1.

Therefore:

```noi
entry main returns 42;
```

must not silently enter `nordoi plan`, `nordoi lower`, or `nordoi run`.

The next compiler milestone may explicitly lift the L0.6 result into a semantic execution plan. That
transition requires its own law, tests, Release Gate, exact-SHA CI, and certification.

## 8. Security properties

L0.6 is fail-closed and bounded by the existing source/token/name/declaration limits.

The new result form:

- creates no capability,
- grants no authority,
- performs no host access,
- invokes no runtime,
- invokes no plugin/provider/model,
- performs no I/O,
- allocates no runtime domain/atom/transaction,
- serializes no authority into source or semantic identity.

## 9. Tooling

Additive inspection command:

```text
nordoi result <path|->
```

The command prints the validated L0.6 result semantics and witness, then explicitly reports:

```text
execution=NOT_PLANNED nair=UNCHANGED runtime=NOT_INVOKED authority=NONE
```

The certified `nordoi --version` output remains unchanged.
