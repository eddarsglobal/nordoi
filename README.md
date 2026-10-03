# NORDOI K0.5 — Atomic Render Core 0.1

K0.5 introduces the first **backend-independent Atomic Render Core**.

NORDOI now has a direct architectural path from atomic state to a minimal render frontier:

```text
NORDOI / AI / Visual Frontend
             ↓
          NAIR 0.1
             ↓
             NAM
             ↓
      changed AtomId set
             ↓
    Atomic Render Core 0.1
             ↓
     minimal RenderBatch
             ↓
 Web / Native / GPU / XR backends
```

## What K0.5 adds

- One render graph for both `Screen` and `World` space.
- Semantic primitives: `Group`, `Quad`, `Text`, `Mesh`.
- Explicit dirty reasons: transform, appearance, content, visibility and structure.
- Minimal subtree invalidation for hierarchical transform/visibility changes.
- Atom-to-render bindings for the first NAM → render bridge.
- Duplicate render invalidation collapse.
- Deterministic render flush order.
- Backend-independent `RenderBackend` contract.
- Rejection of non-finite transforms and invalid normalized opacity.
- Zero external Rust dependencies remain.

## Atomic rendering rule

```text
render_work(change)
    =
minimum_correct_render_frontier(change)
```

Unrelated render nodes must remain untouched.

## Tests

K0.5 adds 11 render-core tests on top of the 28 inherited tests, for a total of **39 tests**.

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
- `docs/TESTING_AND_RELEASE_LAW.md`
