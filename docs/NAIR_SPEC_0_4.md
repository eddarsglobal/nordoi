# NAIR 0.4 — Native Logical-Time Semantics

Status: K1.3 candidate specification.

NAIR 0.4 extends NAIR 0.3 with deterministic logical-timer declarations. It does
not read wall-clock time, sleep threads, install operating-system timers or advance
logical time implicitly. `AtomicEventLoop` remains the governed authority that
advances K1.2 logical time.

## Version

Canonical header:

```text
magic: NAIR
major: 0
minor: 4
```

The decoder continues to accept valid NAIR 0.1, 0.2 and 0.3 binaries. Canonical
re-encoding emits 0.4. Time opcodes are rejected when a binary declares an older
minor version.

## New semantic identity

`TimerSlot(u32)` is a single-assignment semantic timer identity. At K1.3 event-loop
bootstrap it resolves deterministically to one runtime `TimerId` allocated by the
same `AtomicTimeCore` that services host-created timers.

## New opcodes

```text
0x50 SCHEDULE_TIMER_ONCE_AT
0x51 SCHEDULE_TIMER_REPEATING_AT
0x52 CANCEL_TIMER
```

### SCHEDULE_TIMER_ONCE_AT

```text
SCHEDULE_TIMER_ONCE_AT dst, logical_deadline
```

Declares one one-shot timer at an absolute NORDOI `LogicalTime` deadline.

### SCHEDULE_TIMER_REPEATING_AT

```text
SCHEDULE_TIMER_REPEATING_AT dst, first_deadline, interval
```

Declares one repeating timer. `interval` must be greater than zero. K1.2 lossless
catch-up, deterministic `(deadline, TimerId)` ordering and bounded fire-budget
semantics remain authoritative.

### CANCEL_TIMER

```text
CANCEL_TIMER timer
```

Cancels a timer slot that has already been declared earlier in the same validated
program. The semantic slot remains bound to its runtime timer identity even when
the timer is no longer active.

## Bootstrap semantics

K1.3 native time instructions are bootstrap-time declarations. `AtomicEventLoop`
validates the complete NAIR program, applies native timer declarations to a fresh
`AtomicTimeCore`, removes time declarations from the persistent NAM/render/input
bootstrap program, then boots `PersistentAtomicRuntime` exactly once.

This separation preserves persistent runtime identity while ensuring there is only
one published logical-time scheduler.

## Context law

All legacy NAIR execution entry points reject a program containing native time
instructions with `TimeContextRequired` before NAM or render mutation occurs.

Native time programs must be booted through `AtomicEventLoop` in K1.3.

## Explicit-time law

A native timer declaration does not authorize time observation or progression.
Only an explicit host/event-loop call such as `cycle_to`, `cycle_by`, or
`cycle_to_next_deadline` may advance logical time.

## Replay

The K1.3 event-loop replay domain includes canonical NAIR 0.4 program bytes.
Changing a native timer declaration therefore changes the initial event-loop replay
identity even before the first logical-time cycle.
