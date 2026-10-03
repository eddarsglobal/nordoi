# NAIR Native Render Runtime 0.1

This specification defines how NAIR 0.2 render instructions execute against the
Atomic Render Core.

## Execution surfaces

State-only programs use:

```text
execute_nair(kernel, program)
```

Programs containing native render instructions use:

```text
execute_nair_with_render(kernel, render, program)
```

Calling the state-only entry point with a render program returns
`RenderContextRequired` before the program mutates runtime state.

## Slot binding

During one execution:

```text
RenderNodeSlot -> RenderNodeId
AtomSlot       -> AtomId
```

Bindings are local to the exact validated execution report. Semantic slots never
become backend handles.

## Explicit frame boundary

`RENDER_FLUSH` performs:

```text
kernel.flush()
    ↓
deduplicated AtomId frontier
    ↓
AtomicRenderCore.invalidate_atoms(...)
    ↓
AtomicRenderCore.flush()
    ↓
NairRenderFrame
```

Repeated invalidation of the same render node remains collapsed by the Render
Core dirty-mask semantics.

## HALT

HALT checks for residual NAM work or pending render nodes. If either exists, one
final frame is emitted. Otherwise HALT performs zero render work.

## Atomic no-work preservation

An identical transactional atom write after a clean frame must produce:

```text
0 scheduled atoms
0 render updates
```

A rolled-back transaction must satisfy the same rule.
