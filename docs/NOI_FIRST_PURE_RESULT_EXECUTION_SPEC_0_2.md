# NORDOI V0.2 — First Executed Pure Result Specification

Status: **candidate** until the Testing & Release Law is satisfied.

## 1. Scope

V0.2 is the first NORDOI milestone that executes a source-level pure result end-to-end:

```text
.noi source
  -> L0.6 pure-result semantics
  -> C0.5 pure-result execution plan
  -> C0.6 NAIR 0.6 representation
  -> closed AtomicRuntime
  -> transient result-register observation
  -> validated source-level result
```

For:

```noi
entry main returns 42;
```

C0.6 already lowers to the existing NAIR 0.6 program:

```text
CONST r0, INT(42)
HALT
```

V0.2 executes that exact program and publishes `INT(42)` only after validating that the
runtime's final transient register snapshot contains `r0 = INT(42)`.

## 2. Non-goals

V0.2 does **not** define:

- program stdout/stderr or source-level I/O,
- effects or effect handlers,
- host authority or capabilities,
- variables, expressions, operators, functions, calls or mutable source state,
- durable register state,
- register checkpointing,
- a new NAIR opcode,
- a new NAIR wire-format version,
- a new runtime checkpoint format.

CLI text printed by `nordoi result-run` is tooling inspection output, not program I/O.

## 3. Transient register observation

NAIR registers are already part of NAIR 0.6 execution. V0.2 adds an **observation API** that
retains the final register map after execution for validation layers.

This map:

- is not serialized into NAIR,
- is not persisted into runtime checkpoints,
- is not an atom or ownership domain,
- is not host authority,
- is not part of source semantic identity,
- must not create scheduled work.

Existing NAIR execution APIs and their report shapes remain available unchanged. The new
observation API is additive.

## 4. Closed runtime observation

`run_closed_observed(program, input)` uses the same closed-runtime execution engine and
replay-key derivation as `run_closed(...)`, while additionally retaining transient final
register values.

For equal program bytes and equal canonical input, the legacy runtime report and replay key
must remain equal between observed and non-observed execution.

## 5. V0.2 validation

For a result-bearing C0.6 artifact, V0.2 requires all of the following:

1. semantic work item count is zero;
2. required effects are empty;
3. host authority is absent;
4. the program is exactly `CONST result-register, INT(value); HALT`;
5. final register count is exactly one;
6. the certified result register exists after execution;
7. its value equals the source-level pure result exactly;
8. canonical input is empty;
9. zero domains, atoms, transactions, frames and input bridges are created;
10. scheduled work is zero and runtime is quiescent.

For a result-free C0.6 artifact, the program must be exactly `HALT` and the final register map
must be empty.

Any mismatch fails closed.

## 6. V0.2 receipt

The deterministic receipt domain is:

```text
NORDOI-V0.2-PURE-RESULT-EXECUTION-RECEIPT\0
```

The receipt commits to:

- exact C0.6 compiler witness,
- canonical empty input,
- runtime replay key,
- bounded runtime execution counts,
- final register count,
- optional certified result-register binding and observed integer value,
- quiescence.

The receipt is execution evidence only. It is not source semantics, NAIR, a checkpoint, or an
authority token.

## 7. CLI

V0.2 adds:

```text
nordoi result-run <path|->
```

The certified V0.1 `nordoi run` command remains unchanged and continues to reject L0.6
`returns` syntax.

## 8. Compatibility

V0.2 preserves:

- NAIR 0.6 canonical bytes and opcodes,
- C0.1/L0.4/C0.2/L0.5/C0.3/C0.4/L0.6/C0.5/C0.6 witnesses,
- V0.1 source execution semantics,
- runtime checkpoint format and semantics,
- Kernel K1.18 semantics,
- authority/effect/capability laws,
- CLI `--version` output.
