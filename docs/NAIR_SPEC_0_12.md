# NAIR 0.12 — Bounded Runtime Call Extension

NAIR 0.12 is additive over NAIR 0.11.

## New instruction

```text
CALL_EVAL dst function_id args[] body
```

Canonical opcode: `0x0b`.

The instruction contains:

- destination SSA register;
- canonical `u32` function identity;
- bounded list of already-defined argument registers;
- bounded pure `CallExpr` body.

## CallExpr

V1.0 supports:

- literal `INT` / `BOOL` values;
- positional parameter references;
- checked integer addition;
- integer `== != < <= > >=`.

Bounds:

- maximum call arguments: 8;
- maximum call-expression depth: 32;
- maximum call-expression nodes: 256.

## Validation

Validation is fail-closed.

- Every argument register must exist before `CALL_EVAL`.
- Argument values must be scalar INT/BOOL at NAIR level.
- Parameter references must be in range.
- Checked integer overflow is rejected.
- Result register remains SSA single-assignment.
- `CALL_EVAL` requires format minor 12.

## Runtime observation

V1.0 adds an additive call-work observation:

```text
runtime_calls
call_body_instructions
max_call_depth
```

This observation does not modify the historical `NairExecutionReport` layout.

The V1.0 source layer certifies direct non-recursive calls only, so `max_call_depth <= 1`.

## Compatibility

Existing canonical encodings for NAIR 0.6 through 0.11 are unchanged. The base public tooling boundary continues to report NAIR 0.6.
