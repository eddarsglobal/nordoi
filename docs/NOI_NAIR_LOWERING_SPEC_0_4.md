# NORDOI C0.4 — Semantic Plan → NAIR Lowering Foundation

Status: **candidate** until local Release Gate, exact-commit CI, and annotated tag certification.

## 1. Purpose

C0.4 is the first explicit compiler boundary that lowers certified source semantics into existing
NAIR. It is deliberately narrow. It accepts only the C0.3 zero-work semantic plan and emits one
valid NAIR 0.6 program:

```text
C0.3 SemanticExecutionPlan
  ├─ semantic work items = 0
  ├─ required effects = 0
  └─ host authority = NONE
          ↓ validate
C0.4 lowering
          ↓
NAIR 0.6 [Halt]
```

C0.4 does not execute that program.

## 2. Supported semantic subset

Only plans already validated by C0.3 are eligible. Today these originate from the complete L0.5
body forms:

```noi
// empty/trivia-only body
```

or

```noi
entry main;
```

Both are zero-work programs. The entry name is semantic provenance, not runtime work.

## 3. Exact NAIR lowering

NAIR 0.6 requires a terminal `Instruction::Halt` for a valid program. Therefore both supported
C0.3 forms lower to exactly:

```rust
NairProgram::from_instructions(vec![Instruction::Halt])
```

No other NAIR instruction may be emitted by C0.4.

This means distinct zero-work source plans may have identical raw NAIR bytes. That is intentional:
NAIR contains operational meaning, not unnecessary source provenance.

## 4. Compiler provenance witness

C0.4 introduces:

```text
canonical_c04_bytes()
```

with domain separator:

```text
NORDOI-C0.4-NAIR-LOWERING\0
```

The witness commits to:

1. the exact C0.3 semantic-plan witness; and
2. the exact canonical NAIR bytes emitted from that plan.

Therefore `entry main;` and `entry other;` may lower to identical `[Halt]` NAIR bytes while still
having distinct C0.4 lowering witnesses.

`canonical_c04_bytes()` is a compiler witness. It is **not** the NAIR wire format.

## 5. Validation law

Before publication, C0.4 requires:

- semantic work item count = 0;
- required semantic effect count = 0;
- no host authority requirement;
- generated `NairProgram::validate()` succeeds;
- generated `NairProgram::canonical_bytes()` succeeds.

Any violation fails closed.

## 6. Authority separation

C0.4 does not query, infer, serialize, grant, borrow, or consume host authority.

```text
declared effect ≠ authority
resolved effect ≠ authority
semantic plan ≠ authority
NAIR bytes ≠ authority grant
```

The existing constitutional rule that host authority remains outside program bytes is unchanged.

## 7. Runtime boundary

C0.4 does not call `execute_nair`, `run_closed`, the atomic runtime, the event loop, effect dispatch,
or any backend. Compilation and execution remain separate operations.

The CLI command:

```text
nordoi lower <path|->
```

prints the lowering artifact and canonical NAIR bytes for inspection only.

## 8. Compatibility

C0.4 preserves unchanged:

- C0.1 `canonical_identity_bytes()`;
- L0.4 `canonical_semantic_bytes()`;
- C0.2 `canonical_c02_bytes()`;
- L0.5 `canonical_l05_bytes()`;
- C0.3 `canonical_c03_bytes()`;
- NAIR format version 0.6;
- K1.18 kernel semantic stability floor;
- certified `nordoi --version` output.

C0.4 does not modify `src/nair`, runtime, runtime checkpoint, kernel, CI, or Release Gate.

## 9. Non-goals

C0.4 does not define:

- general statements or expressions;
- function calls or returns;
- state allocation or mutation;
- effect execution;
- capability acquisition;
- host I/O;
- entry-name serialization into NAIR;
- runtime invocation;
- a new NAIR opcode or format revision.

Those require later explicit language and compiler milestones.

## 10. Security properties

C0.4 inherits the frontend bounds and fail-closed parsing laws, validates semantic purity before
lowering, validates emitted NAIR before publication, serializes no host authority, introduces no
network/filesystem/shell behavior, and performs no runtime execution.
