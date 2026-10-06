# NORDOI `.noi` Pure Condition Specification — L0.9

Status: candidate.

Governance: this milestone is subordinate to `CONSTITUTION.md`, then to `laws/LAW_0001_NORDOI_MASTER_LAW.md`, the certified Testing & Release Law, and all certified semantic boundaries through V0.4. This document does not redefine those laws.

## Purpose

L0.9 introduces the first source-level boolean semantic values needed for future control flow while preserving the rule that semantic understanding must precede planning and execution.

Accepted result forms are deliberately small:

```noi
entry main returns true;
entry main returns false;
entry main returns 20 == 20;
entry main returns 20 != 22;
entry main returns 20 < 22;
entry main returns 20 <= 22;
entry main returns 22 > 20;
entry main returns 22 >= 22;
```

Empty bodies and the certified no-result `entry Name;` form remain accepted.

## Grammar

```text
PureCondition := BooleanLiteral | IntegerComparison
BooleanLiteral := "true" | "false"
IntegerComparison := CanonicalNonNegativeI64 Comparator CanonicalNonNegativeI64
Comparator := "==" | "!=" | "<" | "<=" | ">" | ">="
```

`==`, `!=`, `<=`, and `>=` are lexical pairs for this surface and must have no whitespace/comment between their two punctuation characters. Normal trivia around the comparator remains allowed.

## Explicit exclusions

L0.9 does not define:

- `if`, `else`, match, branching, loops or iteration;
- `&&`, `||`, `!` boolean logic;
- equality between boolean values;
- arithmetic inside comparison operands;
- named-binding references inside conditions;
- negative integers, floats, text or null conditions;
- condition planning;
- condition NAIR instructions;
- runtime execution;
- storage, effects, capabilities or host authority.

These omissions are intentional semantic boundaries, not missing implicit behavior.

## Canonical semantic form

L0.9 distinguishes semantic construction from truth value. `true` and `1 < 2` both evaluate to `BOOL(true)` but retain different canonical witnesses.

The witness domain is:

```text
NORDOI-L0.9-PURE-CONDITION\0
```

The witness commits to the certified C0.2 semantic unit, entry identity, resolved empty effect requirement set, exact boolean-vs-comparison form, comparison operands/operator when applicable, and evaluated boolean result. It excludes source IDs, paths, spans, comments, whitespace, ambient environment, runtime state and host authority.

## Purity and authority

Declaring an `effect` does not require or execute that effect. Every L0.9 condition has an empty required-effect set and grants no authority.

## Operational boundary

Successful L0.9 compilation reports:

```text
planning=UNDEFINED
nair=UNCHANGED
runtime=NOT_INVOKED
storage=NONE
authority=NONE
```

This is required by the existing constitutional separation between meaning, planning, IR and execution.

## Compatibility

Earlier certified boundaries remain frozen. In particular, L0.7 `expr` and L0.8/C0.9/C0.10/V0.4 binding commands must continue to reject the new condition syntax rather than silently widening their contracts.
