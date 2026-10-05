# NORDOI C0.6 — Pure Result → NAIR Representation Specification

Status: **CANDIDATE until Release Gate + exact-SHA CI + annotated tag**.

C0.6 is an additive compiler boundary above certified C0.5. It defines how the already-understood
pure optional integer result is represented using **existing NAIR 0.6 primitives**. It does not
change NAIR 0.6, execute the runtime, perform I/O, dispatch effects, inspect capabilities, or grant
host authority.

## 1. Input boundary

C0.6 accepts only a valid `PureResultExecutionPlan` produced by C0.5, or source that can be compiled
through the certified chain to such a plan.

The currently understood source forms remain:

```noi
entry main;
```

and:

```noi
entry main returns 42;
```

with the L0.6 integer restrictions unchanged.

## 2. Canonical operational representation

NAIR 0.6 already provides `Instruction::Const` and `Value::Int`. C0.6 MUST NOT introduce a new
return opcode or change the NAIR wire format.

A plan with no result lowers to:

```text
HALT
```

A plan with `result = INT(v)` lowers to exactly:

```text
CONST r0, INT(v)
HALT
```

where `r0` means `RegisterId(0)`.

The register choice is canonical for C0.6. No other register may be selected by this boundary.

## 3. Result binding

Canonical NAIR bytes describe the operational program, but do not themselves declare that a
particular register is the source-level program result. Therefore C0.6 publishes compiler metadata:

```text
result_register = NONE | r0
```

This binding is committed by the C0.6 witness. It is not serialized into NAIR 0.6 and is not host
metadata consumed by the runtime.

## 4. Zero authority and zero external effect

`Const` is pure register-local computation. It MUST NOT:

- create a domain or atom,
- mutate kernel state,
- persist data,
- perform I/O,
- schedule time work,
- render,
- create input bridges,
- emit or dispatch effects,
- request or grant a capability,
- obtain host authority.

A source `effect` declaration remains a semantic name declaration only; it does not become a C0.6
requirement or permission.

## 5. No-result zero-cost rule

If the C0.5 plan has no result, C0.6 emits no result register and no `Const`. The program remains the
single mandatory NAIR terminal `Halt`.

`result = INT(0)` is distinct from no result and MUST emit `Const r0, Int(0)`.

## 6. Validation

C0.6 fails closed unless the input plan has:

- zero semantic work items,
- zero required effects,
- no host authority.

The generated `NairProgram` MUST pass NAIR 0.6 validation and canonical serialization before the
artifact is published.

## 7. Canonical witness

`PureResultNairArtifact::canonical_c06_bytes()` uses domain:

```text
NORDOI-C0.6-PURE-RESULT-NAIR\0
```

and commits, in order:

1. the exact C0.5 canonical plan witness,
2. the optional canonical result-register binding,
3. the exact canonical NAIR 0.6 bytes.

It is a compiler witness, not a NAIR program or runtime receipt.

## 8. Compatibility law

C0.6 is additive. The following certified boundaries remain unchanged:

- C0.3 `nordoi plan`,
- C0.4 `nordoi lower`,
- V0.1 `nordoi run`,
- L0.6 `nordoi result`,
- C0.5 `nordoi result-plan`,
- NAIR 0.6 wire format and decoder,
- runtime and checkpoint formats,
- K1.18 semantic stability contracts.

In particular, `nordoi lower` and `nordoi run` continue to reject `entry main returns 42;`.

## 9. CLI inspection boundary

C0.6 adds:

```text
nordoi result-lower <path|->
```

This command compiles and prints the C0.6 artifact. It MUST NOT invoke the runtime.

## 10. Explicit non-goals

C0.6 does not define:

- runtime result extraction,
- process exit-code mapping,
- stdout/stderr output,
- host return ABIs,
- functions or calls,
- arithmetic expressions,
- mutable variables,
- effect execution,
- capabilities or authority,
- new NAIR instructions,
- a NAIR version bump.
