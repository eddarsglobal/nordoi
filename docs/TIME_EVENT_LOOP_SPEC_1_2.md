# NORDOI Atomic Time & Event Loop Specification 1.2

Status: K1.2 candidate specification.

## 1. Purpose

K1.2 introduces deterministic logical time and timer scheduling without granting
ambient access to an operating-system clock. It also introduces an `AtomicEventLoop`
that publishes logical-time advancement and one persistent-runtime tick as a single
accepted cycle.

This layer is intentionally below native NAIR time semantics. Timer firing is
reported by the event loop but is not yet a NAIR instruction or an implicit state
mutation.

## 2. Logical time

`LogicalTime(u64)` is an abstract monotonic instant measured in NORDOI logical ticks.
`LogicalDuration(u64)` is a difference in the same abstract unit.

The core does not define a relationship between one logical tick and wall-clock
seconds, milliseconds, CPU cycles, display frames or operating-system timer units.
A host adapter may choose such a mapping later, but the mapping is outside canonical
K1.2 semantics.

Rules:

- initial logical time is zero;
- accepted time may remain equal or increase;
- logical time never decreases;
- arithmetic overflow is an error;
- reading an OS wall/monotonic clock is not part of `AtomicTimeCore`.

## 3. Timer identity and scheduling

Every accepted timer receives a monotonic `TimerId`.

K1.2 supports:

- one-shot timer at an absolute logical deadline;
- one-shot timer after a logical duration;
- repeating timer at an absolute first deadline plus interval;
- repeating timer after an initial delay plus interval;
- cancellation.

A repeating interval must be greater than zero. Scheduling a deadline before the
current logical time is rejected without mutation.

## 4. Deterministic firing order

Due timers are processed in ascending canonical order:

```text
(deadline, TimerId)
```

Therefore two independent implementations with the same timer state and same target
logical time observe the same firing order.

Each `TimerFire` contains:

- timer identity;
- exact scheduled logical deadline;
- one-based occurrence number.

## 5. Repeating timers are lossless

When logical time jumps across multiple repeating deadlines, every elapsed deadline
is emitted in order. K1.2 does not silently collapse repeating timer occurrences.

Example:

```text
first deadline = 5
interval       = 5
advance to     = 16

fires: 5, 10, 15
next deadline: 20
```

## 6. Bounded timer bursts

Lossless repeating semantics can otherwise produce unbounded work after a very large
time jump. `AtomicTimeCore` therefore has an explicit maximum fire budget per
advance. The default is `4096` firings.

If an advance would exceed the configured budget:

- the advance fails;
- logical time is unchanged;
- timer occurrence counts are unchanged;
- deadlines are unchanged;
- no partial firing report is published.

The budget is a runtime safety bound, not timer coalescing.

## 7. Atomic time advance

`advance_to()` evaluates on a private candidate scheduler and publishes the candidate
only after the complete advance succeeds. Overflow, budget failure or invalid time
movement cannot leave partially advanced timer state.

## 8. Atomic event-loop cycle

`AtomicEventLoop` contains:

```text
PersistentAtomicRuntime
AtomicTimeCore
cycle index
replay identity
```

`cycle_to(target, input)` performs:

1. clone current logical-time state;
2. clone current persistent runtime;
3. advance candidate logical time to `target`;
4. run one candidate persistent-runtime input tick;
5. require the runtime tick to succeed and finish quiescent;
6. publish both candidates together;
7. advance event-loop cycle identity.

If either time advancement or runtime execution fails, neither candidate is
published.

## 9. Future timers are dormant state

A scheduled timer whose deadline lies in the future is not unfinished NAM or render
work. Event-loop runtime quiescence therefore means NAM/render quiescence; it does not
require an empty timer scheduler.

## 10. Driving without wall clock

`cycle_to_next_deadline()` advances directly to the next logical timer deadline. It
allows deterministic simulations/tests and future virtual-time execution without any
wall-clock read.

A production host may later map platform time to explicit logical-time advancement,
but that adapter will require its own effect/capability contract.

## 11. Replay identity

`EventLoopReplayKey` is history-sensitive. It includes successful timer schedule and
cancel operations plus successful cycle boundaries, logical times, runtime replay
identity and emitted timer occurrences.

Equal accepted event-loop traces produce equal replay keys.

As with K1.0/K1.1 replay keys, this identity is deterministic but not cryptographic and
must not be used as authentication or authorization.

## 12. Deliberate exclusions

K1.2 does not yet define:

- NAIR timer opcodes;
- timer-to-atom bindings;
- async/await;
- threads;
- wall-clock time zones or calendars;
- OS timer APIs;
- sleep semantics;
- timer callbacks that execute arbitrary code;
- network/file/device effects.

Those features require later semantics and security review.
