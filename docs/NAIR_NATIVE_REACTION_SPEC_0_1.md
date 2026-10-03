# NORDOI NAIR Native Reaction Bridge Specification 0.1

Status: Certified by NORDOI K1.5; inherited unchanged by K1.6.

## Purpose

This bridge binds canonical NAIR 0.5 reaction declarations to the certified K1.4
Atomic Reaction & Action Core without creating a second reaction semantics.

## Binding model

A `ReactionSlot` is resolved only during `AtomicEventLoop` bootstrap. The bridge uses
already-created bindings for:

- `DomainSlot -> DomainId`
- `AtomSlot -> AtomId`
- `RenderNodeSlot -> RenderNodeId`
- `TimerSlot -> TimerId`

It then constructs one K1.4 `ReactionSpec` and registers it in program-definition
order. The resulting `ReactionSlot -> ReactionId` binding remains stable for the
lifetime of the event loop.

## Authority boundary

The canonical program provides only its `ActionSpec` declarations. Capability grants
come from an external `NairReactionAuthority` table. An absent entry means an empty
`CapabilitySet`.

This makes serialized code incapable of self-authorizing a privileged effect.

## Persistent activation

Native reactions are not recreated on every runtime tick. They are bootstrapped once
and reused by the persistent event loop.

Input activation consumes the canonical input batch supplied to the cycle. Timer
activation consumes only `TimerFire` values emitted by the certified Atomic Time
Core for that same candidate cycle.

## Phase law

Input bridges precede native Input reactions. Native Input reactions precede native
Timer reactions. Within each reaction family the K1.4 ordering laws remain unchanged.

## Publication law

Reaction application occurs against the private runtime candidate owned by the
candidate event-loop cycle. If a later reaction fails, the candidate runtime and the
candidate logical-time advance are both discarded.

No effect intent is returned from a failed cycle.

## Compatibility

NAIR 0.1–0.4 programs contain no native reaction declaration and preserve their
previous semantics. Direct/legacy execution surfaces reject 0.5 reaction declarations
rather than silently ignoring them.
