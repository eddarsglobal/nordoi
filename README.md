# NORDOI K1.1 — Persistent Atomic Runtime

K1.1 evolves the certified K1.0 closed runtime into NORDOI's first long-lived
application runtime without freezing the future `.noi` language surface.

## Architecture

```text
Authorized host input
        ↓
Canonical InputBatch
        ↓
PersistentAtomicRuntime
        ↓
persistent NAIR input bridges
        ↓
atomic NAM transactions
        ↓
deduplicated causal frontier
        ↓
Atomic Render Core
        ↓
quiescent tick
```

## What K1.1 adds

- `PersistentAtomicRuntime::boot()` — bootstrap one validated NAIR 0.3 world.
- `PersistentAtomicRuntime::tick()` — apply successive input batches without
  recreating atoms, domains, render nodes or input bridges.
- cross-tick monotonic input-sequence validation.
- atomic tick publication using private NAM/render candidates.
- persistent atom versions and values across ticks.
- zero-frame behavior when a tick has no visual effect.
- deterministic session replay identity over the ordered canonical tick trace.
- persistent runtime snapshots keyed by semantic `AtomSlot`.
- preservation of original `APPLY_INPUT` boundary order.
- zero new external Rust dependencies.

## K1.0 remains valid

`AtomicRuntime` and `run_closed()` remain available for isolated closed activations.
K1.1 adds persistence; it does not replace K1.0 semantics.

## Scope

K1.1 is still intentionally below the final language/frontend layer. It does not add
ambient timers, networking, filesystem access, threads, hot reload or source syntax.
Those capabilities require explicit future semantics and governed effects.

## Test corpus

K1.1 adds **14 persistent-runtime tests** to the 108 inherited K1.0 tests, for a total
of **122 tests**.

The new tests cover bootstrap quiescence, persistent atom identity, multi-tick state,
zero-work identical writes, unmatched input, cross-tick sequencing, failed-tick
isolation, deterministic session replay, empty ticks, persistent render targets,
repeated input boundaries and state-only persistent worlds.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.1` can be tagged.

## Key specifications

- `docs/PERSISTENT_RUNTIME_SPEC_1_1.md`
- `docs/ATOMIC_RUNTIME_SPEC_1_0.md`
- `docs/NAIR_SPEC_0_3.md`
- `docs/INPUT_CORE_SPEC_0_1.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
