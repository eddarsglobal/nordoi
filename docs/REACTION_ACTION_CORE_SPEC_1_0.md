# NORDOI Atomic Reaction & Action Core Specification 1.0

Status: K1.4 certified

## 1. Purpose

K1.4 introduces the first backend-independent causal reaction layer between accepted
semantic causes and NAM state mutation.

The layer is deliberately below NAIR. K1.4 certifies reaction meaning first; a later
milestone may encode these semantics in canonical NAIR only after the reaction laws
are stable.

## 2. Semantic boundary

The Reaction Core accepts only already-semantic causes:

- canonical `InputEvent` values contained in an `InputBatch`;
- logical `TimerFire` values produced by the Atomic Time Core.

It does not poll devices, read wall clocks, invoke network APIs, access files or call
platform services.

## 3. Reaction identity and ordering

Each registered reaction receives a monotonically increasing `ReactionId`.

For input activation, canonical input sequence order is preserved. For timer
activation, fires are canonicalized by `(deadline, TimerId, occurrence)`. For one
cause, matching reactions execute in ascending `ReactionId` order.

Therefore a reaction trace has one deterministic ordering independent of hash-map
iteration, backend callback order or host thread scheduling.

## 4. Trigger model

`ReactionTrigger` currently supports:

- `Input(InputSelector)`
- `Timer(TimerSelector)`

Input selectors reuse NORDOI's certified source/device/target/signal filtering.
Timer selectors may match any timer, one timer, or one exact timer occurrence.

## 5. Action model

A `ReactionSpec` binds:

- a semantic name;
- one ownership `DomainId`;
- one trigger;
- one existing `ActionSpec` effect declaration;
- one explicit `CapabilitySet` authority set;
- one or more ordered `ReactionStep` values.

K1.4 steps are:

- `Set { atom, value }`
- `EmitEffect { effect }`

A state write requires the action to declare `Effect::StateWrite`. An effect intent
requires the exact effect declaration and, when required by K0.2 authority law, the
matching scoped capability.

## 6. Reaction values

A state write may use:

- a literal NORDOI `Value`;
- the value extracted by a matching input selector;
- a timer occurrence number;
- a timer deadline in logical ticks.

Trigger-incompatible value sources are rejected when the reaction is registered.
Timer integer projection is checked before conversion into `Value::Int`.

## 7. Atomic publication

Reaction activation uses candidate-state publication:

1. canonicalize/normalize the incoming cause batch;
2. clone the current `AtomicKernel` into private candidate state;
3. evaluate matching reactions deterministically;
4. perform every reaction's NAM writes through an `AtomicTransaction` in its declared
   ownership domain;
5. collect validated external effect intents without executing them;
6. publish the candidate kernel only after the complete activation succeeds.

If any later reaction fails, earlier candidate transactions are discarded with the
candidate kernel. The previously published kernel remains unchanged.

This gives K1.4 a stronger batch boundary than merely committing each reaction in
place.

## 8. Effect intents are not effects

`EmitEffect` produces an `EffectIntent` in the successful reaction report. It does
not perform the external effect.

The intent contains:

- the originating `ReactionId`;
- the declared action name;
- the validated `Effect`.

A future governed executor may consume such intents only under its own certified
publication, retry and failure laws.

## 9. Zero-work behavior

- no matching reaction => no NAM transaction;
- identical state projection => transaction commit reports zero changed atoms and
  schedules no new NAM work;
- effect-only reaction => no NAM transaction;
- failed activation => no candidate state or intent report is published.

## 10. Explicit non-goals

K1.4 does not add:

- NAIR reaction opcodes;
- `.noi` surface syntax;
- dynamic timer scheduling from reactions;
- arbitrary user functions or closures inside canonical semantics;
- direct OS/network/file/device execution;
- threads, async scheduling or hidden ambient state.

## 11. Certification obligations

K1.4 must preserve the complete inherited test corpus and add dedicated evidence for:

- deterministic ReactionId ordering;
- exact input filtering;
- timer canonicalization;
- input/timer value projection;
- ownership enforcement;
- batch rollback on late failure;
- declared-effect enforcement;
- scoped capability enforcement;
- intent-only external effects;
- zero work for unmatched/identical state;
- public input canonicalization rejection;
- non-reused reaction identity after unregister.
