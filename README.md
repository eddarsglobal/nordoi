# NORDOI K0.4 — NAIR Core

K0.4 introduces the first executable version of **NAIR — NORDOI Atomic Intermediate Representation**.

NORDOI source syntax is deliberately not frozen yet. K0.4 establishes the machine-facing semantic bridge first.

```text
NORDOI / AI / Visual Frontend
             ↓
        Semantic Model
             ↓
          NAIR 0.1
             ↓
             NAM
```

## What K0.4 adds

- `NAIR` canonical binary magic and explicit format version `0.1`.
- Deterministic zero-dependency binary encoding and decoding.
- SSA-like single-assignment value registers.
- Domain, atom and transaction slots.
- Validate-before-execute semantics.
- NAIR execution directly on the existing NORDOI Atomic Machine kernel.
- Transaction commit/rollback preservation through NAIR.
- Rejection of use-before-definition, duplicate identities, active transactions at HALT, immediate self-dependencies and non-finite floats.
- Safe-by-omission privileged model: NAIR 0.1 cannot yet express network/filesystem/process/GPU/XR effects.

## NAIR 0.1 instructions

```text
CONST
CREATE_DOMAIN
CREATE_ATOM
CONNECT
BEGIN_TX
TX_SET
COMMIT
ROLLBACK
HALT
```

## Test gate

Every version remains subject to the NORDOI Testing & Release Law:

```bash
cargo check --all-targets
cargo test --all-targets
```

GitHub CI then repeats the tests on Linux, macOS and Windows.

## Specification

See `docs/NAIR_SPEC_0_1.md`.
