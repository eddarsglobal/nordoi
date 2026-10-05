# NORDOI NAIR 0.7 — Pure Integer Arithmetic Foundation

Status: **candidate until local Release Gate, exact-SHA GitHub CI, annotated tag and certification**.

## Purpose

NAIR 0.7 adds the smallest arithmetic primitive required to lower the certified L0.7/C0.7 pure-expression postfix semantics without hidden constant folding.

The only new instruction is:

```text
ADD_INT_CHECKED dst, lhs, rhs
```

Rust API:

```rust
Instruction::IntAddChecked { dst, lhs, rhs }
```

It reads two previously defined integer registers, computes checked signed-64-bit addition, and defines one fresh destination register.

## Wire format

Opcode `0x02` is introduced at minor `0.7`:

```text
0x02 | dst:u32le | lhs:u32le | rhs:u32le
```

The magic and major remain `NAIR` / `0`.

## Compatibility law

NAIR 0.7 uses **minimal-required-minor canonical encoding**:

- Programs that use only the certified NAIR 0.1–0.6 instruction set remain encoded with minor `0.6` and retain their exact bytes.
- Programs containing `ADD_INT_CHECKED` encode with minor `0.7`.
- The decoder accepts minors `0.1` through `0.7`.
- Opcode `0x02` under a header below `0.7` is invalid.

This protects existing C0.4/C0.6 witnesses and V0.1/V0.2 replay identities from retroactive byte changes.

## Validation before execution

A program containing `ADD_INT_CHECKED` is rejected before execution when:

- `lhs` or `rhs` is undefined,
- either operand is not `Value::Int`,
- `dst` is already defined,
- the signed `i64` addition overflows.

No wraparound is permitted.

## Effects and authority

`ADD_INT_CHECKED` is pure:

```text
input            = registers only
effect           = NONE
host authority   = NONE
I/O              = NONE
persistent state = NONE
runtime work     = one deterministic arithmetic instruction
```

It does not create atoms, domains, transactions, frames, bridges, effects, capabilities or host calls.

## Non-goals

NAIR 0.7 does not define subtraction, multiplication, division, comparisons, branching, variables, calls, source syntax or expression lowering. Those require separate milestones.
