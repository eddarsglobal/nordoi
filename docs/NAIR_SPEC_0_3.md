# NAIR 0.3 — Native Interaction Semantics

Status: K0.9 candidate specification.

NAIR 0.3 extends NAIR 0.2 with backend-independent interaction bindings. It does
not encode raw operating-system or hardware events into executable programs. A
host adapter supplies an already-normalized `InputBatch`; NAIR declares how that
batch may project into NAM state.

## Version

Canonical header:

```text
magic: NAIR
major: 0
minor: 3
```

The runtime continues to decode valid NAIR 0.1 and 0.2 binaries. Canonical
re-encoding emits 0.3.

## New semantic identities

`InputBridgeSlot(u32)` is a single-assignment NAIR identity. It resolves at runtime
to one `InputAtomBridge` owned by one NAM domain.

## Input target references

```text
ANY
GLOBAL
RENDER_NODE(RenderNodeSlot)
```

A render-node target is resolved through the render bindings created by the same
validated execution. Foreign runtime render IDs are never serialized into NAIR.

## New opcodes

```text
0x40 CREATE_INPUT_BRIDGE
0x41 BIND_INPUT_ATOM
0x42 APPLY_INPUT
```

### CREATE_INPUT_BRIDGE

Creates one bridge bound to an explicit `DomainRef`.

```text
CREATE_INPUT_BRIDGE dst, domain
```

### BIND_INPUT_ATOM

Declares one semantic input selector and the atom it may update.

```text
BIND_INPUT_ATOM bridge, atom,
                source?, device?, target, signal
```

Selectors support wildcard source/device/target filters and the K0.8 canonical
signals: key state, pointer X/Y, pointer button, generic button, axis and XR pose
X/Y/Z.

The atom must already exist, the bridge must already exist, and a referenced
render target must already exist.

### APPLY_INPUT

```text
APPLY_INPUT bridge
```

`APPLY_INPUT` is an explicit atomic interaction boundary. The supplied immutable
`InputBatch` is matched against the bridge bindings. All resulting writes are
staged and committed through one `AtomicTransaction` for that bridge domain.

No matching binding means zero NAM work. Multiple transitions mapping to one atom
collapse to the deterministic final state snapshot inside the transaction; the
ordered transition history remains in the `InputBatch`.

## Context law

A program containing native input instructions requires an explicit input batch
context before execution. A program containing render instructions requires an
explicit render context. Programs requiring both must use the combined execution
path.

The four K0.9 execution surfaces are:

```text
execute_nair
execute_nair_with_render
execute_nair_with_input
execute_nair_with_render_and_input
```

Missing context is rejected before runtime execution begins.

## Canonical full interaction loop

```text
InputBatch
   ↓
NAIR selector/binding semantics
   ↓
APPLY_INPUT
   ↓
AtomicTransaction
   ↓
NAM changed-state frontier
   ↓
Render binding
   ↓
RENDER_FLUSH / HALT final flush
   ↓
RenderBatch
```

## Security

NAIR 0.3 still cannot poll raw hardware, install global hooks, request OS device
permissions, generate haptics, or access privileged input APIs. Those remain host
adapter/effect-capability concerns and are safe by omission.
