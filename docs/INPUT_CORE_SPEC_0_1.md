# NORDOI Atomic Input & Interaction Core 0.1

Status: K0.8 bootstrap specification.

## Purpose

The Atomic Input & Interaction Core defines one backend-independent semantic event
model for keyboard, mouse, touch, pen, gamepad and XR interaction. Platform APIs
translate device-specific input into this model; platform APIs do not define NORDOI
interaction semantics.

```text
platform adapter
      ↓
validated normalized event
      ↓
Atomic Input Core
      ↓
deterministic InputBatch
      ↓
InputAtomBridge
      ↓
one NAM transaction
      ↓
canonical atomic work frontier
```

## Sources

`InputSource` currently defines:

- `Keyboard`
- `Mouse`
- `Touch`
- `Pen`
- `Gamepad`
- `XrController`
- `XrHand`

This is a semantic source class, not an operating-system device API.

## Event payloads

Core 0.1 defines:

- key transitions with stable numeric key codes and repeat metadata;
- pointer movement with pointer identity, position and delta;
- pointer button transitions;
- generic controller buttons;
- scroll deltas;
- normalized controller axes in `[-1, 1]`;
- XR pose position plus unit quaternion orientation.

Non-finite numeric input is rejected before entering the queue. Negative floating
zero is canonicalized to positive zero. XR orientation is normalized and a zero
quaternion is rejected.

Text composition / IME, haptics, raw hardware polling and platform permission
acquisition are intentionally not part of Core 0.1. They remain safe by omission
until their contracts are defined.

## Deterministic order

Each accepted event receives a monotonically increasing `InputSequence`. Sequence
space never wraps: exhaustion is an error instead of silently changing order.

The sequence is assigned at acceptance time. Coalescing may remove an older
replaceable event, but the surviving event retains the newest sequence, preserving
the meaning "latest accepted state".

## Routing

Every accepted event receives an immutable `InputTarget` snapshot:

- `Global`, or
- `RenderNode(RenderNodeId)`.

Keyboard and gamepad input without an explicit target use the current focus target,
falling back to `Global`. Pointer/touch/pen/XR input without an explicit target is
`Global`; a hit-test or host adapter may provide an explicit render target.

Changing focus never retargets an event already queued.

## Safe coalescing

Only replaceable state samples may coalesce, and only when consecutive events share
the same source, device and target:

- pointer movement for the same pointer (latest position plus accumulated delta);
- axis state for the same axis (latest value);
- XR pose state.

Transitions and additive events are barriers and are never discarded:

- key down/up;
- pointer button down/up;
- generic controller button down/up;
- scroll.

This prevents an optimization from erasing observable interaction history.

## NAM bridge

`InputAtomBridge` binds `InputSelector` + `InputSignal` to an atom owned by one
explicit `DomainId`.

A drained `InputBatch` is converted into staged atom writes and committed through
one `AtomicTransaction`:

- all matching writes are validated before commit;
- repeated state writes to the same atom collapse to the final staged value;
- this is a state projection only; complete transition history remains in `InputBatch`;
- ownership rules remain enforced;
- a batch with no matching binding creates zero NAM work;
- an input value identical to current state creates zero downstream work through
  the existing NAM No Work Without Effect law.

Core 0.1 atom-state signals include key state, pointer position, pointer button state,
generic button state, gamepad/controller axes, and XR pose position. Scroll remains an
additive event in `InputBatch` and is intentionally not projected into a scalar state
binding in Core 0.1.

## Security boundary

Core 0.1 does not poll hardware. It accepts already-delivered semantic events from a
host adapter. Direct raw-device access, OS permission prompts, global key capture,
camera-derived gestures and similar privileged acquisition are outside this core and
must receive explicit effect/capability contracts before becoming canonical NORDOI
operations.

## Required invariants

1. Invalid numeric input enters neither queue nor NAM.
2. Accepted order is deterministic and never silently wraps.
3. Focus is snapshotted at event acceptance.
4. Transition events are never coalesced away.
5. Replaceable state samples may coalesce only on an identical route/control.
6. Input-to-NAM application is one atomic transaction per batch.
7. Ownership remains authoritative across input bindings.
8. No matching input means zero kernel work.
9. Identical resulting state means zero new downstream work.
