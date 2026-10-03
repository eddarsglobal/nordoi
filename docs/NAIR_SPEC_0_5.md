# NAIR 0.5 — Native Reaction Semantics

Status: K1.5 candidate specification.

NAIR 0.5 extends NAIR 0.4 with canonical reaction declarations that map directly to
the certified K1.4 Atomic Reaction & Action Core. It does not redefine K1.4 reaction
meaning and does not grant privileged authority to serialized programs.

## Version

Canonical header:

```text
magic: NAIR
major: 0
minor: 5
```

The decoder continues to accept valid NAIR 0.1, 0.2, 0.3 and 0.4 binaries.
Canonical re-encoding emits 0.5. Reaction opcodes are invalid under a declared minor
version below 5.

## New semantic identity

`ReactionSlot(u32)` is a single-assignment symbolic identity in a NAIR program.
During governed event-loop bootstrap, reaction slots resolve in program-definition
order to monotonic K1.4 `ReactionId` values.

Runtime `ReactionId` handles are never serialized as substitutes for semantic slots.

## New opcode

```text
0x60 DEFINE_REACTION
```

`DEFINE_REACTION` contains one complete immutable reaction declaration:

```text
reaction slot
reaction name
domain reference
trigger
action name
canonical declared-effect set
ordered reaction steps
```

The complete declaration is encoded in one instruction so partially constructed
reaction builders cannot become observable canonical state.

## Trigger encoding

Native triggers are:

```text
INPUT(source?, device?, target, signal)
TIMER(timer_slot?, occurrence?)
```

Input target references reuse NAIR render-node slots. Timer references reuse NAIR
logical timer slots introduced by 0.4. Referenced slots must have been declared
before the reaction declaration.

## Reaction values

Native state-write projections are:

```text
LITERAL(value)
INPUT_VALUE
TIMER_OCCURRENCE
TIMER_DEADLINE_TICKS
```

The value source must be compatible with the trigger. Non-finite float literals are
not valid canonical reaction values.

## Reaction steps

Native steps are:

```text
SET(atom_slot, reaction_value)
EMIT_EFFECT(effect)
```

Every `SET` requires `StateWrite` in the declared-effect set. Every `EMIT_EFFECT`
requires the exact emitted effect in that set.

## Authority law

NAIR serializes declared effects, never granted capabilities.

A program cannot place a `Network`, `FileRead`, `FileWrite`, `Camera`, `Microphone`,
`Location`, `Gpu`, `Xr` or `Process` capability inside its own canonical bytes and
thereby authorize itself.

Privileged authority is supplied externally by the host through
`NairReactionAuthority`, keyed by semantic `ReactionSlot`. K1.4 exact-scope effect
checks remain authoritative during reaction bootstrap.

## Bootstrap semantics

`AtomicEventLoop` is the native K1.5 reaction execution surface:

1. validate the complete NAIR 0.5 program;
2. bootstrap native logical timers;
3. bootstrap persistent NAM/render/input state without time/reaction declarations;
4. resolve reaction slot references against the resulting semantic bindings;
5. register immutable reactions in `AtomicReactionCore`;
6. publish the event loop only if the complete bootstrap succeeds.

Legacy NAIR execution entry points reject `DEFINE_REACTION` with
`ReactionContextRequired` before NAM mutation becomes observable.

## Cycle ordering

Within one K1.5 event-loop cycle the deterministic causal phase order is:

```text
1. certified InputAtomBridge applications
2. native Input reactions in canonical event / ReactionId order
3. native Timer reactions in canonical timer-fire / ReactionId order
4. NAM/render propagation and quiescence check
5. atomic publication of time + runtime
```

This phase order is semantic and backend-independent.

## Atomic publication

Time and runtime remain private candidates for the duration of the cycle. A failure
in either input or timer reaction activation aborts the cycle. Logical time, timer
state, NAM state, render state, replay state and effect-intent reports remain at the
previous published boundary.

## Effect intents

`EMIT_EFFECT` maps to K1.4 `EffectIntent`. It does not call an operating system,
network stack, filesystem, process API or device backend. Execution of an external
effect remains outside the native reaction core.

## Replay

The event-loop replay domain is bumped for K1.5. Canonical NAIR 0.5 bytes include
native reaction declarations. Timer causes that can drive native reactions are also
included in the persistent candidate replay progression before cycle publication.
