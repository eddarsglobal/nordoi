# NORDOI Effect Outbox & Dispatch Specification 1.0

**Introduced:** K1.6  
**NAIR format:** unchanged at 0.5  
**Status:** candidate until the K1.6 release gate and cross-platform CI are certified

## 1. Purpose

K1.5 produces validated `EffectIntent` objects but intentionally performs no external
I/O. K1.6 defines the boundary by which a committed intent can later be handed to a
host backend without weakening deterministic runtime semantics or least privilege.

## 2. Semantic planes

K1.6 separates execution into two planes.

### Deterministic intent plane

The event loop owns:

- reaction execution;
- effect-intent production;
- deterministic intent identity;
- deterministic outbox ordering;
- atomic publication of the outbox with the event-loop cycle;
- replay identity for newly committed effect envelopes.

### Environmental delivery plane

The host owns:

- current dispatch authority;
- backend selection;
- operating-system / network / device interaction;
- backend success or failure;
- opaque external receipt metadata.

The environmental delivery plane is not part of deterministic NAM replay.

## 3. Effect envelope

Every published external intent receives:

```text
QueuedEffectIntent {
  id: EffectIntentId,
  cycle: u64,
  ordinal: u64,
  intent: EffectIntent
}
```

`EffectIntentId` is assigned monotonically by the event-loop outbox in canonical intent
order. `ordinal` is zero-based within one published cycle.

Input-reaction intents precede Timer-reaction intents because K1.5 already certifies
that causal phase order.

## 4. Candidate publication

`AtomicEventLoop::cycle_to` evaluates time, runtime, reactions and outbox staging on
private candidate state.

Only after all candidate work succeeds are the following published together:

- logical time;
- timer state;
- NAM/runtime state;
- render state;
- event-loop replay state;
- effect outbox state.

An effect staging failure therefore aborts the same candidate publication boundary.

## 5. Dispatch authority

A queued intent does not authorize itself.

Immediately before backend invocation, `GovernedEffectDispatcher` derives the exact
required capability from the effect and checks it against the current
`EffectDispatchAuthority`.

Examples:

```text
Network("api.example.test") -> Capability::Network("api.example.test")
FileRead("/data/x")          -> Capability::FileRead("/data/x")
Camera                       -> Capability::Camera
```

Exact scope remains authoritative. A capability for one network scope does not satisfy
another scope.

Dispatch authority is mutable by the host so revocation can take effect after an intent
was queued but before it is executed.

## 6. Internal effects

`Pure`, `StateRead` and `StateWrite` are NAM/internal semantic effects and have no
external capability. They SHALL NOT be handed to an external effect backend.

If such an intent reaches the outbox, external dispatch fails closed with
`InternalEffectNotDispatchable` and the intent remains pending.

## 7. Backend contract

A host backend implements:

```text
EffectBackend::supports(effect)
EffectBackend::execute(queued_intent)
```

The core performs no direct platform I/O. K1.6 ships no privileged backend.

Backend support declaration prevents accidental routing to an incompatible backend;
it is not a substitute for dispatch authority.

## 8. Acknowledgement law

The outbox removes an intent only after backend `execute` returns success.

These failures preserve the pending intent:

- missing capability;
- revoked capability;
- unsupported backend;
- backend-reported failure.

A backend error does not mutate deterministic replay identity.

## 9. Replay law

K1.6 advances the event-loop replay domain to `NORDOI-ATOMIC-EVENT-LOOP-1.6` so a
K1.6 execution identity cannot be confused with a K1.5 identity that had no semantic
outbox.

Newly committed effect envelopes participate in the event-loop replay key through:

- intent id;
- cycle;
- ordinal;
- reaction id;
- action name;
- exact effect variant and scope.

Backend receipts and backend success/failure do not participate in deterministic replay.

## 10. Exactly-once limitation

K1.6 does not claim universal exactly-once external side-effect semantics.

External systems can observe an operation before a local process knows whether final
acknowledgement was durably recorded. Future backends may use `EffectIntentId` as an
idempotency/deduplication key, but exactly-once behavior requires an appropriate
protocol or cooperating destination.

## 11. Durability limitation

`AtomicEffectOutbox` is an in-memory semantic outbox in K1.6. It is atomically published
with the live event-loop object, but crash-durable storage is outside K1.6.

A future persistence layer must certify how pending effect envelopes survive process,
machine or distributed failures before NORDOI may claim durable delivery.

## 12. Backend-result re-entry

A backend SHALL NOT mutate NAM directly as a completion mechanism.

Future external results must re-enter through a governed semantic completion/input
cause so that state changes remain ordered, validated and replay-accounted.
