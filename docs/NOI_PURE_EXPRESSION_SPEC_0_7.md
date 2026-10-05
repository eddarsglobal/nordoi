# NORDOI `.noi` Pure Expression Specification 0.7

Status: **L0.7 candidate**. This specification is additive. It does not modify the certified L0.6
pure-result boundary, C0.5 result plan, C0.6 result-to-NAIR lowering, V0.2 execution boundary,
NAIR 0.6, runtime authority, checkpoint formats, or kernel semantics.

## 1. Purpose

L0.7 introduces the first source-level value that is **computed from a pure expression** rather than
copied from a single literal. The supported subset is deliberately small enough to have complete,
fail-closed semantics:

```noi
entry main returns 20 + 22;
```

The expression is evaluated by the compiler semantic boundary. L0.7 does not create an execution
plan, lower the expression to NAIR, or invoke the runtime.

## 2. Contextual surface

`entry` and `returns` remain ordinary lexical identifiers outside the exact contextual positions
already established by earlier layers. L0.7 does not add lexer keywords.

The L0.7 body forms are:

```text
<empty body>
entry Name;
entry Name returns PureExpression;
```

The expression grammar is:

```text
PureExpression := Operand ("+" Operand)*
Operand        := CanonicalNonNegativeI64 | "(" PureExpression ")"
```

No other operator has L0.7 semantics.

## 3. Integer literals

L0.7 inherits the L0.6 integer profile exactly:

- `0` is valid;
- otherwise the first digit must be `1` through `9`;
- all following characters must be ASCII digits;
- the parsed value must be in `0..=i64::MAX`.

Examples rejected by L0.7 include `-1`, `+1`, `01`, `1_000`, `0x2a`, and `42i64`.

## 4. Operators and grouping

Only binary `+` is defined. Parentheses may group subexpressions. Bracket and brace groups are
undefined and rejected.

Without parentheses, `+` is left-associative:

```text
1 + 2 + 3  =>  ((1 + 2) + 3)
```

L0.7 deliberately preserves evaluation structure. `(1 + 2) + 3` and `1 + (2 + 3)` calculate the
same current value but have distinct L0.7 semantic witnesses. This milestone does not establish
reassociation or commutativity as a language law.

## 5. Canonical semantic representation

The frontend converts each valid expression to postfix operations:

```text
20 + 22       => [INT(20), INT(22), ADD]
1 + (2 + 3)   => [INT(1), INT(2), INT(3), ADD, ADD]
(1 + 2) + 3   => [INT(1), INT(2), ADD, INT(3), ADD]
```

The semantic evaluator is iterative. `MAX_PURE_EXPRESSION_NODES = 1024` limits the postfix node
count. Structural group nesting remains independently bounded by the certified L0.2 parser.

## 6. Arithmetic law

`ADD` uses checked `i64` addition. Overflow is a semantic error and fails closed before publication
of L0.7 NSIR.

There is no wraparound, saturation, bigint promotion, float conversion, host-dependent numeric
behavior, or undefined arithmetic.

## 7. Purity and authority

An L0.7 expression:

- requires zero semantic effects;
- performs zero I/O;
- receives zero capabilities;
- receives zero host authority;
- creates no atoms, domains, transactions, frames, timers, or bridges;
- invokes no runtime.

A source declaration such as `effect Network;` remains only a declaration. It does not become a
requirement or grant authority merely because an expression exists in the same module.

## 8. Witness

`NsirPureExpressionUnit::canonical_l07_bytes()` uses the domain:

```text
NORDOI-L0.7-PURE-EXPRESSION\0
```

It commits to:

- the existing C0.2 semantic module/registry witness;
- body form;
- entry name;
- resolved required-effect set (currently empty);
- presence/absence of an expression;
- exact canonical postfix operation sequence;
- the checked evaluated value.

It excludes source IDs, file names, paths, comments, whitespace, spans, CLI state, runtime state,
capabilities, and host authority.

## 9. Compatibility

L0.7 is exposed separately through:

```text
nordoi expr <path|->
```

Certified older boundaries remain frozen. In particular, the source:

```noi
entry main returns 20 + 22;
```

must continue to be rejected by `nordoi result`, `result-plan`, `result-lower`, `result-run`,
`body`, `plan`, `lower`, and `run` until a later milestone explicitly bridges L0.7 semantics into
those layers.

The certified `nordoi --version` text remains unchanged.

## 10. Explicitly undefined in L0.7

L0.7 does **not** define subtraction, multiplication, division, modulo, unary operators, negative
literals, booleans, strings, floats, variables, bindings, calls, functions, conditionals,
comparisons, casts, user-defined operators, operator overloading, general precedence, effects,
capabilities, authority, expression-to-plan lowering, expression-to-NAIR lowering, or expression
runtime execution.
