# C0.7 Pure Expression Execution Plan — Architecture Record

## Why this milestone exists

L0.7 proved that NORDOI can understand and evaluate a small pure expression language. The next boundary must preserve that calculation as compiler semantics before any operational lowering is chosen.

C0.7 therefore answers one question only:

> What exact pure calculation has been validated and selected for execution?

It does not answer how that calculation is encoded as NAIR.

## Preserve structure, not only the answer

Constant-folding `20 + 22` immediately to `42` would erase information that L0.7 deliberately made semantic: evaluation structure.

Keeping postfix operations in the plan has several advantages:

- the future lowering can be audited against the exact source semantics,
- parenthesized structure remains observable in compiler identity,
- optimization can become an explicit later phase rather than an accidental semantic rewrite,
- the plan remains compact and iterative.

The postfix form also avoids introducing a large recursive AST into the compiler plan.

## Why work remains zero

`work_item_count` describes runtime work units, not semantic-expression nodes. C0.7 has not yet defined runtime instructions for the expression. Marking each postfix node as runtime work would claim an execution model before that model exists.

C0.7 therefore carries:

```text
postfix semantics != runtime work
work = 0
```

The later lowering milestone must make any operational cost explicit.

## Why there is no NAIR change

NAIR already contains arithmetic-related primitives in the broader runtime architecture, but C0.7 intentionally does not select a lowering strategy yet.

Possible later strategies include:

- faithful instruction-per-operation lowering,
- validated constant folding,
- a hybrid optimization plan.

Choosing among them belongs to a separate compiler milestone with its own tests and witness.

## Security and authority

A pure expression requires no effect and no authority. C0.7 validates that invariant again at its publication boundary, including an adversarial test where an otherwise-valid L0.7 unit is manually given an effect requirement.

The plan cannot grant authority because it contains no authority object and `requires_host_authority()` is permanently false.

## Compatibility

C0.7 is additive.

Frozen:

- L0.7 pure-expression witness,
- C0.5 pure-result plan,
- C0.6 pure-result NAIR representation,
- V0.2 pure-result execution,
- C0.3/C0.4/V0.1,
- NAIR 0.6 wire format,
- runtime/checkpoint semantics,
- K1.18 kernel stability surfaces.

The new CLI surface is `expr-plan`; no earlier command is repurposed.
