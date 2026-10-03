# NAIR 0.1 — NORDOI Atomic Intermediate Representation

NAIR 0.1 is the first canonical executable representation between future NORDOI frontends and the NORDOI Atomic Machine (NAM).

## Purpose

NAIR exists so that source syntax is not the machine contract.

```text
Human NORDOI source ─┐
AI/SI generation ────┼──> Semantic NORDOI ──> NAIR ──> NAM
Visual tooling ──────┘
```

## K0.4 invariants

1. **Canonical binary form** — equivalent NAIR instruction sequences encode deterministically.
2. **Versioned format** — every binary starts with `NAIR` and an explicit format version.
3. **Validate before execute** — malformed programs never reach NAM execution.
4. **Single-assignment registers** — NAIR 0.1 values use SSA-like register identity.
5. **Use-after-definition only** — a register, atom, domain, or transaction must exist before use.
6. **Transactions cannot leak across HALT** — every transaction must commit or roll back.
7. **No self-dependency** — immediate dependency cycles are rejected before NAM.
8. **Finite numeric canonicality** — NaN and infinities are rejected in NAIR 0.1 to avoid non-canonical numeric semantics.
9. **Safe by omission** — NAIR 0.1 has no filesystem, network, process, GPU, or XR operation. Privileged operations cannot be expressed until their effect/capability semantics are specified.
10. **No external runtime dependency** — K0.4 remains zero-dependency Rust.

## Instruction set 0.1

| Opcode | Instruction | Purpose |
| --- | --- | --- |
| `0x01` | `CONST` | Bind an immutable value register |
| `0x10` | `CREATE_DOMAIN` | Create an ownership domain |
| `0x11` | `CREATE_ATOM` | Create an atom owned by a domain |
| `0x12` | `CONNECT` | Add an atomic dependency edge |
| `0x20` | `BEGIN_TX` | Begin an atomic transaction |
| `0x21` | `TX_SET` | Stage an atom write |
| `0x22` | `COMMIT` | Atomically commit staged writes |
| `0x23` | `ROLLBACK` | Discard staged writes |
| `0xFF` | `HALT` | End the program |

NAIR 0.1 is intentionally small. New opcodes must justify their semantics, security model, deterministic encoding, validation rules, and runtime cost before admission.
