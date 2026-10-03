# NORDOI K0.6 — NAIR → Atomic Render Bridge 0.1

K0.6 connects **NAIR runtime atoms directly to the Atomic Render Core** without
introducing a second dependency engine.

```text
NORDOI / AI / Visual Frontend
             ↓
          NAIR 0.1
             ↓
             NAM
             ↓
  deduplicated AtomId frontier
             ↓
   NAIR → Render Bridge 0.1
             ↓
    Atomic Render Core 0.1
             ↓
      minimal RenderBatch
             ↓
 Web / Native / GPU / XR backends
```

## What K0.6 adds

- `NairRenderBridge` for canonical `AtomSlot -> AtomId -> RenderNodeId` causality.
- Binding of semantic NAIR atom slots to render nodes through `NairExecutionReport`.
- Direct consumption of NAM's already-deduplicated scheduler frontier.
- Zero render work for identical NAIR writes.
- Zero render work for rolled-back NAIR transactions.
- Dependency-driven render invalidation without recomputing dependencies.
- Dirty-reason merging across the bridge.
- Shared Screen/World bridge semantics.
- `NairRenderFrame` reports scheduled atoms and the resulting render batch.
- Zero external Rust dependencies remain.

## Architectural rule

```text
render_frontier
    =
RenderBindings(NAM.atomic_work_frontier)
```

The bridge must not invent a parallel dependency graph.

## Tests

K0.6 adds **8 NAIR-render bridge tests** on top of the 39 inherited tests, for a
total of **47 tests**.

The release gate remains mandatory:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI then repeats testing on Linux, macOS and Windows.

## Specifications

- `docs/NAIR_SPEC_0_1.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/NAIR_RENDER_BRIDGE_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
