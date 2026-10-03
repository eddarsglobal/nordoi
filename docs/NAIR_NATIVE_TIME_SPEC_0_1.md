# NORDOI Native Time Binding Specification 0.1

K1.3 lifts K1.2 logical timers into canonical NAIR without introducing ambient
clock authority.

## Principle

```text
NAIR 0.4 timer declaration
        ↓
TimerSlot
        ↓ bootstrap resolution
TimerId in AtomicTimeCore
        ↓
explicit host logical-time advance
        ↓
canonical TimerFire sequence
```

The program owns semantic timer declarations. The event loop owns logical-time
progression.

## Timer slots

`TimerSlot` is a semantic ID, not a wall-clock handle and not a capability. Slots
are single-assignment and must be defined before cancellation.

## Deterministic runtime IDs

Native timers are allocated in validated program order. Host-created timers use the
same `AtomicTimeCore` identity allocator after native bootstrap. Equal programs
therefore produce equal native `TimerId` bindings.

## Cancellation

`CANCEL_TIMER` is a bootstrap declaration in K1.3. It can remove future timer work
before the first cycle. Cancellation does not erase the `TimerSlot → TimerId`
binding used for evidence and deterministic identity.

## Existing K1.2 laws remain authoritative

Native timers inherit:

- monotonic logical time
- canonical `(deadline, TimerId)` firing order
- lossless repeating deadlines
- bounded per-advance fire budget
- atomic failed advances
- atomic time+runtime event-loop publication

## Safe by omission

K1.3 does not add:

- wall-clock reads
- OS timers
- sleeping or blocking
- threads or async runtimes
- timer-triggered NAM actions
- dynamic timer creation from input handlers

Timer firings remain an explicit event-loop report. Future reaction/action semantics
may consume those firings only after their ownership, effect and transactional rules
are specified.
