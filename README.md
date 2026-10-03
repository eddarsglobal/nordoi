# NORDOI K1.4 — Atomic Reaction & Action Core

K1.4 extends the certified K1.3 NAIR 0.4 Native Time foundation with a deterministic,
transactional reaction layer connecting semantic causes to NAM actions.

K1.4 deliberately remains **below NAIR**. It certifies reaction/action semantics
before a future NAIR version encodes them as canonical instructions.

## Architecture

```text
Canonical InputBatch              AtomicTimeCore
        │                              │
        │ InputEvent                   │ TimerFire
        └──────────────┬───────────────┘
                       ↓
              AtomicReactionCore
                       │
          deterministic ReactionId order
                       │
             ┌─────────┴─────────┐
             ↓                   ↓
     NAM AtomicTransaction   EffectIntent
             │                   │
             ↓                   └─ validated only;
      candidate kernel              no OS/API call
             │
             ↓
      atomic batch publish
```

## What K1.4 adds

- `AtomicReactionCore`.
- monotonic `ReactionId` identities.
- `ReactionTrigger::Input` using the existing canonical `InputSelector` model.
- `ReactionTrigger::Timer` using deterministic logical `TimerFire` causes.
- `TimerSelector` for any timer, one timer or one exact occurrence.
- `ReactionValue` projections from input values and logical timer data.
- `ReactionStep::Set` for NAM writes through ownership-aware atomic transactions.
- `ReactionStep::EmitEffect` for validated **effect intents**, never direct platform calls.
- reuse of K0.2 `ActionSpec`, `Effect`, `CapabilitySet` and exact authority checks.
- whole-batch candidate-state publication: a late failure publishes none of the earlier
  candidate reaction mutations.
- canonical timer ordering by `(deadline, TimerId, occurrence)`.
- zero new external Rust dependencies.

## Core law

A semantic cause may request an action, but a reaction may publish only state it owns
and may externalize only an effect it both declared and was explicitly authorized to
request.

```text
cause
  ↓
match trigger
  ↓
validate ownership + declared effects + authority
  ↓
private candidate transactions
  ↓
all reactions succeed?
  ├─ no  → discard candidate
  └─ yes → publish NAM + return validated effect intents
```

## External effects remain outside the core

K1.4 can produce a validated `EffectIntent`, for example a scoped network or file
request. It does not execute that request. Network stacks, filesystems, processes,
devices and other privileged backends remain outside the canonical Reaction Core.

## NAIR scope

NAIR remains 0.4 in K1.4. No reaction opcode is introduced yet. This is intentional:
K1.5 can integrate the now-certified reaction semantics into canonical NAIR without
inventing action opcodes before their transaction and effect laws are proven.

## Test corpus

K1.4 adds **14 dedicated reaction/action tests** to the **158 inherited K1.3 tests**,
for an expected total of **172 tests** once the full Release Gate runs.

The new tests cover reaction ordering, zero-work matching, input projection,
identical-state suppression, late-failure atomicity, effect declaration, capability
authority, intent-only external effects, timer ordering/projection, incompatible value
sources, public input canonicalization, exact routing and identity non-reuse.

## Mandatory release gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```

GitHub CI must repeat the gate on Linux, macOS and Windows before `k1.4` can be tagged.

## Key specifications

- `docs/REACTION_ACTION_CORE_SPEC_1_0.md`
- `docs/NAIR_SPEC_0_4.md`
- `docs/NAIR_NATIVE_TIME_SPEC_0_1.md`
- `docs/TIME_EVENT_LOOP_SPEC_1_2.md`
- `docs/PERSISTENT_RUNTIME_SPEC_1_1.md`
- `docs/TESTING_AND_RELEASE_LAW.md`
