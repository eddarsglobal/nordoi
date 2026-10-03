# NORDOI NAIR → Atomic Render Bridge 0.1

## Status

K0.6 bootstrap specification.

## Purpose

The bridge converts the canonical atomic work frontier produced by NAM into the
minimum correct render frontier. It does not duplicate dependency analysis.

```text
NAIR AtomSlot
    ↓ execution binding
Runtime AtomId
    ↓ NAM scheduler / dependency closure
Deduplicated AtomId frontier
    ↓ NairRenderBridge
Atomic Render Core bindings
    ↓
Minimal RenderBatch
```

## Laws

1. `AtomSlot` is resolved through the exact `NairExecutionReport` that created it.
2. Unknown slots are rejected before a render binding can be installed.
3. The bridge consumes NAM's scheduler using `AtomicKernel::flush()`.
4. If NAM has zero scheduled work, the bridge must create zero new render work.
5. Rollback must never produce render work.
6. Identical state writes must never produce render work.
7. Dependency closure is computed by NAM once; the bridge must not recompute it.
8. Multiple dirty reasons for the same node collapse into one render update.
9. One atom may drive multiple screen/world nodes without backend coupling.

## Non-goals of 0.1

K0.6 does not yet add render opcodes to NAIR. It establishes the verified runtime
bridge first. Render creation and property mutation opcodes are reserved for the
next NAIR format increment after this causal bridge is certified.
