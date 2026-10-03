# NAIR 0.2 — Native Atomic Render Semantics

NAIR 0.2 extends the canonical NORDOI Atomic Intermediate Representation with
backend-independent render semantics. It does not introduce DOM, Metal, Vulkan,
WebGPU, DirectX or any other platform API into the machine contract.

## Compatibility

- Canonical encoding produced by K0.7 is version `0.2`.
- Valid state-only `0.1` binaries remain readable.
- A binary declaring `0.1` may contain only the 0.1 instruction set.
- Render opcodes require format `0.2` or later.

## Render identities

`RenderNodeSlot(u32)` is a semantic NAIR identity. Like registers, atom slots and
domain slots, a render node slot is single assignment and must be defined before
use.

## Instruction set additions

| Opcode | Instruction | Operands |
| --- | --- | --- |
| `0x30` | `CREATE_RENDER_NODE` | slot, primitive, space |
| `0x31` | `CREATE_RENDER_CHILD` | slot, parent-slot, primitive, space |
| `0x32` | `BIND_RENDER_ATOM` | atom-slot, node-slot, dirty-mask |
| `0x33` | `SET_RENDER_VISIBLE` | node-slot, bool |
| `0x34` | `SET_RENDER_OPACITY` | node-slot, finite f32 in `0..=1` |
| `0x35` | `SET_RENDER_POSITION` | node-slot, finite f32 x/y/z |
| `0x36` | `RENDER_FLUSH` | none |

Render primitives in 0.2 are `Group`, `Quad`, `Text`, and `Mesh`. Render spaces
are `Screen` and `World`.

## Validation laws

1. Render node slots are single assignment.
2. Parent slots must exist before a child is declared.
3. Atom and node slots in a render binding must already exist.
4. Empty dirty masks are invalid.
5. Opacity must be finite and within `0..=1`.
6. Position components must be finite.
7. Render instructions require an explicit render execution context.
8. Validation happens before any NAM or render mutation.

## Frame semantics

`RENDER_FLUSH` consumes NAM's deduplicated pending atom frontier, translates it
through render bindings, and emits the minimum correct `RenderBatch`.

At `HALT`, a final implicit flush occurs only when NAM or the Atomic Render Core
still has pending work. This guarantees both:

- no valid work is silently dropped;
- no empty frame is manufactured merely because execution halted.

## No parallel dependency engine

NAIR 0.2 does not introduce a separate render dependency graph. Atom invalidation
continues to flow through NAM's canonical dependency scheduler and only then into
render bindings.
