# NORDOI Atomic Render Core 0.1

K0.5 introduces the first backend-independent render core for NORDOI.

## Purpose

The render core does not rasterize pixels itself. It establishes the semantic contract that future WebGPU, Metal, Vulkan, DirectX, software, terminal and XR backends must consume.

Its job is to answer one question with the smallest possible amount of work:

> What is the minimum render frontier that actually changed?

## Unified space model

The same render graph currently supports two spaces:

- `Screen`
- `World`

This is deliberate. 2D UI and 3D world content are not separate rendering languages. They are nodes interpreted in different spaces by a shared core.

Future versions may add view/head/hand/room spaces without creating a separate XR rendering model.

## Render primitives

Render Core 0.1 defines semantic primitives only:

- `Group`
- `Quad`
- `Text`
- `Mesh`

Backends decide how those primitives are implemented.

## Dirty reasons

A node may be invalidated for one or more explicit reasons:

- `TRANSFORM`
- `APPEARANCE`
- `CONTENT`
- `VISIBILITY`
- `STRUCTURE`

Dirty masks merge before flush. Repeating the same invalidation creates no additional render job.

## Atomic laws

### No Work Without Effect
Writing an identical render property produces zero pending render work.

### Minimum Render Frontier
A local property change dirties only the affected node. A hierarchical transform/visibility change dirties only the affected subtree.

### Duplicate Work Collapse
Multiple invalidations of one node before flush collapse into one update containing the union of dirty reasons.

### Deterministic Flush
Pending render nodes are flushed in stable `RenderNodeId` order.

### Invalid State Rejection
Opacity outside `0..=1` and non-finite transform values are rejected before they can become render state.

## NAM bridge

Atoms may be bound to render nodes with an explicit dirty reason. When NAM reports changed atoms, the render core converts those atom IDs into the minimal deduplicated render frontier.

This is the beginning of the NORDOI state-to-render path:

```text
NAM state change
      ↓
changed AtomId set
      ↓
render bindings
      ↓
deduplicated dirty nodes
      ↓
RenderBatch
      ↓
backend
```

## Backend contract

A platform renderer implements the `RenderBackend` trait and receives a `RenderBatch`.

The Atomic Render Core remains independent of Web, DOM, WebGPU, Metal, Vulkan, DirectX or any specific platform API.
