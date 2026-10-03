# NORDOI Native Input Binding Specification 0.1

K0.9 makes interaction policy canonical NAIR while preserving K0.8's authorized
adapter boundary.

## Principle

Programs describe **meaning**, not hardware acquisition.

```text
platform event
  → authorized adapter
  → Atomic Input Core
  → InputBatch
  → NAIR 0.3 bindings
  → NAM transaction
```

## Bridge ownership

Each `InputBridgeSlot` has one explicit NAM ownership domain. A binding may target
only an atom owned by that domain. Ownership is verified when the binding becomes
runtime-active.

## Deterministic selectors

A canonical selector consists of:

- optional `InputSource`
- optional `InputDeviceId`
- target filter (`ANY`, `GLOBAL`, or a NAIR `RenderNodeSlot`)
- one `InputSignal`

Wildcards are represented explicitly, never inferred from missing bytes or host
behavior.

## Atomic apply

`APPLY_INPUT` does not mutate atoms one event at a time. It plans matching values,
stages them in an `AtomicTransaction`, then performs one all-or-nothing commit.
This preserves K0.8's atomic input-to-state law inside canonical NAIR.

## Event history versus state projection

NAIR input bindings are a state projection mechanism. A key down followed by key
up remains two ordered events in `InputBatch`, while both may map to one final
boolean atom snapshot at `APPLY_INPUT`. Future event-handler/action semantics must
consume transition history explicitly rather than reconstructing it from state.

## Render targets

A selector that names a render node stores a `RenderNodeSlot`, not a runtime
`RenderNodeId`. At execution, that slot is resolved only through the exact render
binding map created by the current validated program.

## Zero-work rules

- no selector match → no transaction work
- final value identical to current atom → no downstream NAM work
- repeated bindings do not create an alternative dependency graph
- render invalidation still flows through NAM's single causal frontier
