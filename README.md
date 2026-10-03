# NORDOI K1.5 — Native NAIR Reaction Semantics

K1.5 extends the certified K1.4 Atomic Reaction & Action Core by encoding those
reaction semantics natively in canonical **NAIR 0.5** and binding them to the
persistent `AtomicEventLoop`.

K1.5 does not invent a second reaction engine. NAIR declarations compile directly
into the already-certified K1.4 `AtomicReactionCore` model.

## Architecture

```text
                     canonical NAIR 0.5
                            │
             ┌──────────────┼───────────────┐
             ↓              ↓               ↓
        NAM/render/input  timers       DEFINE_REACTION
             │              │               │
             │         TimerSlot→TimerId     │
             │                              resolve
             │      Domain/Atom/Render bindings
             │              │               │
             └──────────────┴───────┬───────┘
                                    ↓
                           AtomicReactionCore
                                    │
                     ReactionSlot → ReactionId
                                    │
                     persistent AtomicEventLoop
                                    │
                  ┌─────────────────┴─────────────────┐
                  ↓                                   ↓
             Input causes                        TimerFire causes
                  │                                   │
                  └──────────── atomic cycle ─────────┘
```

## What K1.5 adds

- NAIR format **0.5**.
- `ReactionSlot(u32)` symbolic identities.
- canonical `DEFINE_REACTION` opcode `0x60`.
- native Input and Timer reaction triggers.
- native literal/Input/timer reaction-value projections.
- native `SET` and `EMIT_EFFECT` steps.
- canonical ordered effect declarations through `NairEffectSet`.
- `NairReactionAuthority`, supplied by the host and never serialized by code.
- event-loop bootstrap of reaction declarations exactly once.
- stable `ReactionSlot -> ReactionId` bindings.
- native reaction reports on every event-loop cycle.
- atomic rollback of logical time + runtime if reaction activation fails.
- deterministic phase order: Input bridges → Input reactions → Timer reactions.
- strict legacy-executor rejection through `ReactionContextRequired`.
- strict 0.5 opcode version gating while preserving 0.1–0.4 decoding.
- zero new external Rust dependencies.

## Security boundary: code declares, host authorizes

A NAIR program may declare that an action intends to request an effect. It may **not**
serialize a capability grant to itself.

```text
NAIR bytes
  ├─ declared effect: Network("api.example.test")
  └─ no authority grant

Host
  └─ NairReactionAuthority[ReactionSlot]
       └─ Capability::Network("api.example.test")
```

Without the exact externally supplied capability, privileged reaction bootstrap is
rejected by the existing K0.2 authority law.

This keeps least privilege structural rather than conventional.

## Native reaction declaration

K1.5 uses one complete declaration instruction instead of a mutable reaction builder:

```text
DEFINE_REACTION
  slot
  reaction name
  ownership domain
  trigger
  action name
  declared effects
  ordered steps
```

This prevents partially constructed reaction state from becoming a canonical runtime
object.

## Event-loop phase order

Within one cycle:

```text
1. canonicalize input
2. apply certified InputAtomBridge state projections
3. execute matching native Input reactions
4. execute canonical TimerFire reactions
5. flush NAM/render work
6. require quiescence
7. publish time + runtime + replay state together
```

Input and Timer reaction ordering inside their phase continues to use the K1.4 stable
`ReactionId` rules.

## Atomic failure example

A timer reaction projecting a logical deadline larger than `i64::MAX` cannot represent
that value as `Value::Int`. The candidate reaction therefore fails. K1.5 discards the
entire candidate cycle: logical time does not advance, NAM does not change, timer state
does not publish, and no effect intent escapes.

## External effects remain intentions

`EMIT_EFFECT` produces a validated K1.4 `EffectIntent`; it does not execute network,
filesystem, process, camera, microphone, location, GPU, XR or other platform APIs.

## Compatibility

The decoder accepts NAIR 0.1, 0.2, 0.3 and 0.4. Canonical re-encoding emits 0.5.
A `DEFINE_REACTION` bytecode under a declared version below 0.5 is rejected.

## Test corpus

K1.5 adds **16 native-reaction tests** to the **172 certified K1.4 tests**, for an
expected total of **188 tests**.

The K1.5 tests cover:

- NAIR 0.5 current version and 0.4 compatibility;
- strict reaction opcode version gating;
- single-assignment reaction slots;
- atom/timer reference validation;
- trigger/value-source compatibility;
- exact declared-effect validation;
- canonical binary round trips;
- legacy executor rejection before mutation;
- native Input execution;
- native timer occurrence/deadline projection;
- program-order reaction identities;
- denial of privileged effects without host authority;
- authorized effect intents without backend execution;
- Input-before-Timer cycle phase ordering;
- event-loop rollback when a reaction fails.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.5` can be tagged.

## Key specifications

- `docs/NAIR_SPEC_0_5.md`
- `docs/NAIR_NATIVE_REACTION_SPEC_0_1.md`
- `docs/REACTION_ACTION_CORE_SPEC_1_0.md`
- `docs/NAIR_SPEC_0_4.md`
- `docs/NAIR_NATIVE_TIME_SPEC_0_1.md`
- `docs/TIME_EVENT_LOOP_SPEC_1_2.md`
- `docs/PERSISTENT_RUNTIME_SPEC_1_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
