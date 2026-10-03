# NORDOI K1.3 — NAIR 0.4 Native Time Semantics

K1.3 extends the certified K1.2 Atomic Time & Event Loop by making logical timer
declarations part of canonical NAIR.

It does **not** introduce ambient wall-clock access and does not freeze future `.noi`
syntax.

## Architecture

```text
NAIR 0.4
  ├─ state / ownership
  ├─ render
  ├─ input
  └─ native timer declarations
              ↓
        AtomicEventLoop
              ↓
       AtomicTimeCore
              +
 PersistentAtomicRuntime
              ↓
        atomic cycles
```

## What K1.3 adds

- `TimerSlot(u32)` as a semantic NAIR identity.
- `SCHEDULE_TIMER_ONCE_AT` (`0x50`).
- `SCHEDULE_TIMER_REPEATING_AT` (`0x51`).
- `CANCEL_TIMER` (`0x52`).
- NAIR format minor 0.4 with backward decode support for 0.1–0.3.
- native timer bootstrap inside `AtomicEventLoop`.
- deterministic `TimerSlot → TimerId` bindings.
- native timer introspection through `native_timer_id()` and `timer_snapshot()`.
- explicit rejection of native time programs by execution paths without a time context.
- event-loop replay identity incorporating the canonical native-time program.
- zero new external Rust dependencies.

## Core law

A program may declare **when a logical timer is due**, but it may not ask the machine
for the current real-world time.

```text
program: timer at logical tick 100
                    ↓
          dormant schedule
                    ↓
host explicitly advances logical time
                    ↓
          deterministic fire
```

## Bootstrap-only scope

K1.3 native timer declarations execute once during `AtomicEventLoop::boot`.
Persistent NAM/render/input identities are then booted without re-running timer
declarations on every tick.

Timer-triggered actions and dynamic scheduling from event handlers are deliberately
out of scope until a future reaction/action layer has explicit transaction and effect
laws.

## Compatibility

- valid NAIR 0.1 binaries decode
- valid NAIR 0.2 binaries decode
- valid NAIR 0.3 binaries decode
- canonical encoding emits NAIR 0.4
- 0.4 timer opcodes cannot be smuggled under an older minor-version header

## Test corpus

K1.3 adds **18 native-time tests** to the 140 inherited K1.2 tests, for a total of
**158 tests**.

The new tests cover binary compatibility, version gating, slot single assignment,
zero intervals, cancellation order, canonical round-trip, context rejection,
native one-shot and repeating bootstrap, deterministic timer IDs, coexistence with
NAM state, next-deadline execution and replay identity.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.3` can be tagged.

## Key specifications

- `docs/NAIR_SPEC_0_4.md`
- `docs/NAIR_NATIVE_TIME_SPEC_0_1.md`
- `docs/TIME_EVENT_LOOP_SPEC_1_2.md`
- `docs/PERSISTENT_RUNTIME_SPEC_1_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
