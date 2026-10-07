# NAIR 0.10 — Structured Dynamic Branch Value

Status: candidate extension for NORDOI V0.8 certification.

NAIR 0.10 is additive over 0.9 and introduces one structured runtime control-flow primitive.

## Instruction

```text
BRANCH_VALUE dst, condition, then_value, else_value
```

Opcode: `0x09`.

Canonical encoding after the opcode:

```text
u32 dst
u32 condition
Value then_value
Value else_value
```

`condition` must be an already-defined boolean SSA register. `dst` must be fresh. The two values must have the same supported kind. V0.8 supports `INT` and `BOOL` branch values.

Execution reads the boolean condition and publishes exactly one selected value to `dst`. The execution report increments `runtime_branches` by one for each executed `BRANCH_VALUE`.

The instruction grants no authority, performs no I/O, schedules no kernel work, creates no frame, and mutates no existing register.

## Compatibility

Programs without `BRANCH_VALUE` preserve their prior required minor and canonical bytes. In particular:

- base constants remain 0.6;
- checked integer addition remains 0.7;
- integer comparisons remain 0.8;
- dynamic input register reads remain 0.9.
