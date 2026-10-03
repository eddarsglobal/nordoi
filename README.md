# NORDOI K1.6 — Governed Effect Outbox & Dispatch Core

K1.6 closes the boundary between the certified K1.5 native reaction semantics and
future real platform I/O without allowing external side effects to contaminate the
deterministic NAM/NAIR execution core.

K1.5 can validate and emit `EffectIntent` values. K1.6 gives those intentions a
governed lifecycle:

```text
canonical cause
     │
     ↓
NAIR 0.5 native reaction
     │
     ↓
validated EffectIntent
     │
     ├── candidate cycle still private
     │
     ↓
AtomicEffectOutbox
     │
     ├── deterministic EffectIntentId
     ├── cycle + ordinal
     └── committed with the successful event-loop cycle
                 │
                 └──── deterministic boundary ends here
                              │
                              ↓
                    GovernedEffectDispatcher
                              │
                 exact current capability check
                              │
                              ↓
                       host EffectBackend
                              │
                    external / environmental I/O
```

## What K1.6 adds

- `AtomicEffectOutbox` integrated into `AtomicEventLoop` publication;
- deterministic `EffectIntentId(u64)` identities;
- stable Input-before-Timer effect ordering inherited from K1.5;
- `QueuedEffectIntent { id, cycle, ordinal, intent }` envelopes;
- effect envelopes included in event-loop replay progression;
- `EffectDispatchAuthority` with exact capability scope and runtime revocation;
- `GovernedEffectDispatcher` as a separate post-commit dispatch boundary;
- host-injected `EffectBackend` trait;
- explicit backend support checks;
- backend receipts that are excluded from deterministic replay identity;
- failed, denied or unsupported dispatch leaves the intent pending;
- successful backend completion is required before the intent is acknowledged;
- internal effects (`Pure`, `StateRead`, `StateWrite`) cannot cross the external
  dispatch boundary;
- no built-in network/filesystem/process/device backend;
- zero new external Rust dependencies.

## NAIR version

**NAIR remains 0.5 in K1.6.**

K1.6 does not add a new serialized opcode and does not let a program serialize a
backend, runtime permission grant or delivery receipt. The native reaction declaration
from K1.5 remains the canonical source of effect intent.

This is deliberate: host execution authority is environmental policy, not program
bytecode.

## Two authority boundaries

K1.6 intentionally validates authority twice at different semantic boundaries.

### 1. Intent authority

During native reaction bootstrap, K1.5 requires the exact declared effect and the
exact capability needed to produce that effect intent.

### 2. Dispatch authority

Immediately before external dispatch, K1.6 checks the capability again against the
current `EffectDispatchAuthority`.

Therefore a capability that was valid when the program booted can be revoked before
the queued operation is handed to a backend.

```text
program declaration
      +
reaction authority
      ↓
validated intent
      +
current dispatch authority
      ↓
host backend execution
```

A queued intent is never itself a capability.

## Atomicity law

External I/O is not rollbackable in the same sense as NAM state. K1.6 therefore does
not execute a backend while a NAM/time/render candidate is still private.

The order is:

```text
1. canonicalize causes
2. execute input bridges / reactions / timers on candidate state
3. validate complete cycle
4. stage effect envelopes in candidate outbox
5. publish NAM + render + time + replay + outbox together
6. only later may the host dispatch pending effects
```

If steps 1–4 fail, no new effect envelope is published.

Once a host backend has performed an external action, NORDOI does **not** claim that
it can universally undo that action. This is why dispatch is a separate delivery
plane.

## Delivery semantics

K1.6 guarantees deterministic intent identity and ordering inside one event-loop
history. It does **not** claim universal exactly-once external delivery.

A backend can fail after receiving an intent, and a process can theoretically fail at
an arbitrary host boundary. `EffectIntentId` is therefore designed to be usable by
future idempotent/deduplicating backends, but destination-level exactly-once semantics
require cooperation from that destination or a stronger certified protocol.

K1.6's in-memory outbox is a semantic outbox. Crash-durable persistence is not claimed
until a future storage/persistence law certifies it.

## Replay boundary

The deterministic event-loop replay key incorporates newly committed effect envelopes,
including their deterministic identity, reaction identity, action name and exact effect
scope.

Backend execution results and backend receipt references are deliberately excluded.
Environmental success/failure must not rewrite what the deterministic program meant to
request.

If an external result later needs to affect NORDOI state, it must return through a
future governed semantic input/completion cause rather than mutating NAM directly from
the backend.

## Host backend contract

K1.6 provides only the interface:

```text
supports(effect) -> bool
execute(QueuedEffectIntent) -> EffectBackendReceipt | EffectBackendError
```

The core contains no HTTP client, filesystem implementation, process launcher, camera,
microphone, location, GPU or XR backend.

A backend is host code and therefore an explicit trust boundary. NORDOI governs what
it chooses to dispatch; it cannot prevent arbitrary unrelated behavior inside a
malicious host process.

## Test corpus

K1.6 adds **17 effect-dispatch tests** to the **188 certified K1.5 tests**, for an
expected total of **205 tests**.

The K1.6 tests cover:

- deterministic effect-intent identity;
- monotonic identities across cycles;
- Input-before-Timer outbox ordering;
- zero outbox work for unmatched cycles;
- failed-cycle effect rollback;
- dispatch deny-by-default;
- exact capability scope;
- successful acknowledge-after-execute behavior;
- unsupported backend preservation;
- backend failure preservation/retry;
- runtime capability revocation;
- explicit re-grant;
- internal-effect dispatch rejection;
- dispatch receipts excluded from replay identity;
- equal traces producing equal effect envelopes;
- pending intent survival across later cycles;
- deterministic pending dispatch order.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.6` can be tagged.

## Key specifications

- `docs/EFFECT_OUTBOX_DISPATCH_SPEC_1_0.md`
- `docs/NAIR_NATIVE_REACTION_SPEC_0_1.md`
- `docs/NAIR_SPEC_0_5.md`
- `docs/REACTION_ACTION_CORE_SPEC_1_0.md`
- `research/EFFECT_EXECUTION_INTELLIGENCE_0_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
