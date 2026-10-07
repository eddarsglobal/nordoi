# NAIR 0.11 — Selective Branch Expression Evaluation

Status: V0.9 candidate specification.

## Purpose

NAIR 0.11 extends the structured dynamic branch boundary without introducing arbitrary control-flow jumps. It adds one instruction:

```text
BRANCH_EVAL dst condition then_expr else_expr
```

The condition is an already-defined `BOOL` register. `then_expr` and `else_expr` are bounded pure expression trees. Both trees are validated before execution, but only the selected tree is evaluated at runtime.

## Branch expression forms

A branch expression may contain:

- canonical `INT` or `BOOL` literal values;
- references to already-defined `INT` or `BOOL` registers;
- checked integer addition;
- integer comparisons `== != < <= > >=`.

The maximum expression depth is 32 and maximum expression node count per arm is 256. Unsupported value kinds fail closed.

Nested dynamic branch expressions are not part of 0.11. Source-level statically decidable nested branches may be folded before lowering.

## Selective execution law

Execution MUST:

1. read the boolean condition;
2. select exactly one branch expression;
3. evaluate only the selected expression;
4. publish exactly one result to `dst`;
5. increment `runtime_branches` by one;
6. increment `selected_branch_instructions` by the number of evaluated branch-expression nodes;
7. leave `discarded_branch_instructions` at zero.

The unselected expression must perform no runtime arithmetic, comparison, register read, host call, effect, allocation, or authority acquisition.

## Compatibility

- `BRANCH_VALUE` remains NAIR 0.10 and byte-compatible.
- `READ_INPUT_KEY_CODE` remains NAIR 0.9.
- integer comparison remains 0.8.
- checked integer addition remains 0.7.
- fully static `CONST + HALT` remains 0.6.

Programs containing `BRANCH_EVAL` require minor version 11. Existing programs keep their minimum required minor version.
