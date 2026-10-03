# NORDOI K1.2 — Atomic Time & Event Loop

K1.2 extends the certified K1.1 Persistent Atomic Runtime with deterministic logical
time, bounded lossless timers and an atomic event-loop publication boundary.

It does **not** read the operating-system clock and does not freeze future `.noi`
syntax or native NAIR time instructions.

## Architecture

```text
Host / deterministic simulator
            ↓ explicit logical time
      AtomicTimeCore
            ↓ due TimerFire list
      AtomicEventLoop
            +
PersistentAtomicRuntime
            ↓
      atomic cycle publish
            ↓
NAM → Render → quiescence
```

## What K1.2 adds

- `LogicalTime` and `LogicalDuration` in abstract NORDOI logical ticks.
- `TimerId`, one-shot timers and repeating timers.
- deterministic timer ordering by `(deadline, TimerId)`.
- lossless repeating-timer catch-up across logical-time jumps.
- explicit bounded fire budget to prevent unbounded timer bursts.
- transactional logical-time advancement.
- `AtomicEventLoop` combining time and K1.1 persistent runtime.
- atomic cycle publication: failed runtime cycles cannot advance published time.
- `cycle_to()`, `cycle_by()` and `cycle_to_next_deadline()`.
- history-sensitive `EventLoopReplayKey`.
- zero new external Rust dependencies.

## Core rule

NORDOI K1.2 never asks "what time is it?" inside the semantic core.

Instead, a host provides an explicit logical target:

```text
current logical time = 40
host advances to     = 50
                         ↓
              deterministic timers
                         ↓
             persistent runtime tick
                         ↓
                 quiescent cycle
```

This makes tests, replay, simulations and future distributed coordination independent
from accidental wall-clock behavior.

## Timer semantics

Repeating timer deadlines are not silently coalesced:

```text
first = 5
period = 5
advance to 16

fires 5, 10, 15
next = 20
```

To avoid denial-of-service style timer explosions, each advance has a fire budget.
Exceeding the budget rejects the complete logical-time advance without partial state.

## Atomic event-loop cycle

`AtomicEventLoop::cycle_to()` evaluates both time and the persistent runtime on private
candidates. Only a complete successful cycle is published.

```text
published state
     ↓
private time candidate + private runtime candidate
     ↓
SUCCESS → publish both
ERROR   → publish neither
```

## K1.1 remains valid

`PersistentAtomicRuntime` remains available directly. K1.2 layers explicit logical
time around it; it does not replace its state, ownership, interaction, render or
replay semantics.

## Scope

Timer firings are reported to the host/event-loop layer in K1.2. They do not yet
mutate NAM automatically and are not yet NAIR instructions. Native time bindings
belong to a later version after this scheduler is certified.

K1.2 also does not add ambient OS clock access, threads, async/await, filesystem,
network or privileged device APIs.

## Test corpus

K1.2 adds **18 time/event-loop tests** to the 122 inherited K1.1 tests, for a total of
**140 tests**.

The new tests cover monotonic logical time, one-shot and repeating timers, canonical
same-deadline ordering, lossless catch-up, zero intervals, cancellation, past
deadlines, atomic fire-budget failure, zero-duration advance, event-loop bootstrap,
time+runtime cycles, timer-only cycles, failed-cycle isolation, replay identity and
wall-clock-free next-deadline driving.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.2` can be tagged.

## Key specifications

- `docs/TIME_EVENT_LOOP_SPEC_1_2.md`
- `docs/PERSISTENT_RUNTIME_SPEC_1_1.md`
- `docs/ATOMIC_RUNTIME_SPEC_1_0.md`
- `docs/NAIR_SPEC_0_3.md`
- `docs/INPUT_CORE_SPEC_0_1.md`
- `docs/RENDER_CORE_SPEC_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
