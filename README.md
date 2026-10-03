# NORDOI K0.7 — NAIR 0.2 Native Render Semantics

K0.7 moves rendering from an external runtime bridge into the canonical NAIR
instruction set while preserving the single NAM causal frontier introduced in K0.6.

```text
NORDOI / AI / Visual Frontend
             ↓
          NAIR 0.2
   state + native render ops
             ↓
             NAM
             ↓
 canonical atomic work frontier
             ↓
    Atomic Render Core 0.1
             ↓
 deterministic RenderFrame(s)
             ↓
 Web / Native / GPU / XR backends
```

## What K0.7 adds

- `RenderNodeSlot` as a canonical single-assignment NAIR identity.
- Native NAIR creation of Screen and World render nodes.
- Native parent/child render hierarchy creation.
- Native atom-to-render binding with explicit `DirtyMask` semantics.
- Native visibility, opacity and 3D-position updates.
- Explicit `RENDER_FLUSH` frame boundaries.
- A final implicit HALT flush only when real state/render work remains.
- `execute_nair_with_render()` for programs that contain render instructions.
- `execute_nair()` remains compatible with state-only programs and rejects render
  programs before mutation when no render context was supplied.
- NAIR binary format 0.2 with deterministic render opcode encoding.
- Backward reading of valid NAIR 0.1 state programs.
- 0.2 render opcodes cannot be smuggled inside a binary declaring format 0.1.
- Zero external Rust dependencies remain.

## NAIR 0.2 render opcodes

| Opcode | Instruction | Purpose |
| --- | --- | --- |
| `0x30` | `CREATE_RENDER_NODE` | Create a root Screen/World node |
| `0x31` | `CREATE_RENDER_CHILD` | Create a child under an existing node |
| `0x32` | `BIND_RENDER_ATOM` | Bind NAM atom invalidation to a node |
| `0x33` | `SET_RENDER_VISIBLE` | Change subtree visibility |
| `0x34` | `SET_RENDER_OPACITY` | Change normalized opacity |
| `0x35` | `SET_RENDER_POSITION` | Change finite `[x,y,z]` position |
| `0x36` | `RENDER_FLUSH` | Emit one deterministic render frame |

The state/transaction opcodes from NAIR 0.1 remain unchanged.

## Frame law

```text
explicit RENDER_FLUSH
        = one frame boundary

HALT
        = final implicit flush only if pending work exists
```

Therefore an idle halt produces no artificial empty frame, while unflushed valid
work can never be silently lost.

## Tests

K0.7 adds **15 native NAIR-render tests** on top of the 47 inherited tests, for a
total of **62 tests**.

The release gate remains mandatory:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI then repeats testing on Linux, macOS and Windows.

## Specifications

- `docs/NAIR_SPEC_0_1.md` — retained historical contract
- `docs/NAIR_SPEC_0_2.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/NAIR_RENDER_BRIDGE_SPEC_0_1.md`
- `docs/NAIR_NATIVE_RENDER_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
