# NORDOI NAIR 0.8 — Pure Boolean & Integer Comparison Foundation

Status: **candidate until local Release Gate, exact-SHA GitHub CI, annotated tag and certification**.

## Governance

This milestone is subordinate to `CONSTITUTION.md` and `laws/LAW_0001_NORDOI_MASTER_LAW.md`. It does not restate or replace those laws.

## Purpose

NAIR 0.8 adds the smallest pure comparison layer required by the certified L0.9/C0.11 condition semantics. Boolean constants already exist as `Value::Bool`; N0.8 therefore adds only the integer comparison operations that NAIR cannot currently express.

## Instructions

```text
INT_EQ dst, lhs, rhs   opcode 0x03
INT_NE dst, lhs, rhs   opcode 0x04
INT_LT dst, lhs, rhs   opcode 0x05
INT_LE dst, lhs, rhs   opcode 0x06
INT_GT dst, lhs, rhs   opcode 0x07
INT_GE dst, lhs, rhs   opcode 0x08
```

Rust API names are `IntEq`, `IntNe`, `IntLt`, `IntLe`, `IntGt`, and `IntGe`. Each reads two previously defined `Value::Int` registers and defines one fresh destination register containing `Value::Bool`.

Each encoding is exactly:

```text
opcode:u8 | dst:u32le | lhs:u32le | rhs:u32le
```

No per-instruction comparison tag is added. This keeps every comparison one byte smaller than a generic compare opcode plus operator tag.

## Format compatibility

- The certified NAIR base remains `0.6`.
- Checked integer addition remains `0.7`.
- Programs containing an integer comparison require `0.8`.
- `NAIR_LATEST_FORMAT_MINOR` becomes `8`.
- The decoder accepts certified minors `0.1` through `0.8`.
- Existing 0.6 and 0.7 programs retain their minimal required minor and canonical bytes.
- Comparison opcodes under a header below 0.8 are invalid.

## Validate before execute

A comparison fails before execution if:

- `lhs` or `rhs` is undefined,
- either operand is not `Value::Int`,
- `dst` is already defined.

No coercion is defined. Signed `i64` ordering is exact. Comparisons cannot overflow.

## Effects, work and authority

The six operations are pure register computations. They create no atoms, domains, transactions, frames, bridges, timers, effects, capabilities, host calls or persistent state. They grant no authority. Runtime work is exactly one deterministic comparison instruction.

## Non-goals

N0.8 does **not** define source syntax, compiler lowering, boolean logic, branching, jumps, `if`, loops, variables, calls, effects or authority. Compiler plan → NAIR lowering remains a later milestone.
