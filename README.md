# NORDOI K1.13 — Native NAIR Effect Completion Semantics

K1.13 lifts the certified K1.12 governed effect-completion re-entry core into canonical NAIR without
freezing `.noi` surface syntax and without serializing host authority.

The native path is:

```text
NAIR 0.6 program
    ↓
DEFINE_EFFECT_COMPLETION (0x70)
    ↓
CompletionSlot
    ↓
host-supplied NairCompletionAuthority
    ↓
(source, delivery namespace)
    ↓
K1.12 AtomicEffectCompletionCore
    ↓
audited delivered attempt
    ↓
normal NAM transaction projection
    ↓
render flush / deterministic replay
```

## What K1.13 adds

- NAIR format **0.6**;
- `CompletionSlot(u32)`;
- native `Instruction::DefineEffectCompletion`;
- opcode `0x70`;
- `NairCompletionProjection`;
- `NairCompletionProjectionValue`;
- `NairCompletionBinding`;
- host-supplied `NairCompletionAuthority`;
- native completion bootstrap into the certified K1.12 completion core;
- exact slot → `(EffectCompletionSourceId, EffectDeliveryNamespace)` binding;
- direct-executor rejection when native completion context is unavailable;
- explicit missing-binding failure at event-loop boot;
- native completion binding/count introspection;
- backward decode support for NAIR 0.1 through 0.5;
- event-loop replay domain `NORDOI-ATOMIC-EVENT-LOOP-1.13`;
- no new Rust dependency.

## Why authority is not in NAIR

A program may declare what a completion means, but it may not mint its own authority.

Canonical NAIR serializes:

```text
CompletionSlot
name
domain
projection atom
projection value selector
```

It does **not** serialize:

```text
EffectCompletionSourceId
EffectDeliveryNamespace
journal writer identity
fence
signing key
trust epoch
backend receipt
```

Those remain host-governed resources. The host explicitly binds a declared `CompletionSlot` to an
exact source and delivery namespace:

```rust
completion_authority.bind(
    CompletionSlot(0),
    EffectCompletionSourceId(7),
    namespace,
);
```

A missing binding fails closed during event-loop boot.

## Native declaration

The canonical instruction is conceptually:

```rust
Instruction::DefineEffectCompletion {
    dst: CompletionSlot(0),
    name: "remote-result".into(),
    domain: DomainRef::Root,
    projections: vec![
        NairCompletionProjection::new(
            AtomSlot(0),
            NairCompletionProjectionValue::OutcomeValue,
        ),
    ],
}
```

Supported projection values remain exactly those certified by K1.12:

```text
OutcomeValue
Succeeded
IntentId
AttemptId
SourceSequence
```

K1.13 does not introduce a second completion engine. Native declarations resolve directly into
`AtomicEffectCompletionCore` projections and exact source authority at boot.

## NAIR 0.6 canonical encoding

Header:

```text
magic: "NAIR"
major: 0
minor: 6
```

Native completion opcode:

```text
0x70 DEFINE_EFFECT_COMPLETION

u32  completion_slot
str  name
domain_ref
u32  projection_count
repeat projection_count:
    u32 atom_slot
    u8  projection_value
```

Projection tags:

```text
0x00 OutcomeValue
0x01 Succeeded
0x02 IntentId
0x03 AttemptId
0x04 SourceSequence
```

An opcode `0x70` declared under NAIR 0.5 or earlier is invalid. Older valid programs remain
readable, and canonical re-encoding emits the current 0.6 format.

## Validation laws

A native completion declaration must satisfy all of the following before boot:

- its `CompletionSlot` is single-assignment;
- its name is non-empty;
- its domain already exists;
- it contains at least one projection;
- every projected `AtomSlot` already exists;
- the same atom is not projected twice inside one declaration.

At bootstrap, K1.13 additionally requires:

- an explicit host binding for every native completion slot;
- a non-zero completion source;
- ownership compatibility between the declared domain and each resolved atom;
- no duplicate runtime projection for the same resolved source/namespace/atom route.

## Execution boundary

Legacy/direct NAIR execution surfaces do not have K1.12 completion state, audited delivery history or
host route authority. Therefore they reject native completion declarations with
`CompletionContextRequired` rather than silently ignoring them.

`AtomicEventLoop` is the native orchestration surface. It strips time/reaction/completion
declarations from the direct persistent-runtime bootstrap, resolves those declarations in their
specialized certified cores, and then publishes one integrated event loop.

## Boot APIs

Existing boot APIs remain source-compatible for programs without native completion declarations.
K1.13 adds:

```rust
AtomicEventLoop::boot_with_authorities(
    program,
    reaction_authority,
    completion_authority,
)
```

and the fire-budget variant:

```rust
AtomicEventLoop::boot_with_fire_budget_and_authorities(...)
```

Existing `boot`, `boot_with_fire_budget`, and reaction-authority-only APIs use an empty completion
authority. Consequently, a program containing native completion declarations must use an explicit
completion authority or boot fails closed.

## Replay

The native declaration itself is canonical program meaning, so it is already committed through the
NAIR 0.6 program bytes hashed at event-loop boot.

The host binding is authority, not program meaning. Merely changing a source/namespace binding does
not rewrite canonical NAIR bytes. When an actual completion is accepted, however, its source,
sequence, delivery identity, attempt, outcome and projected writes remain deterministic semantic
cause exactly as certified in K1.12.

K1.13 therefore bumps the event-loop replay domain to:

```text
NORDOI-ATOMIC-EVENT-LOOP-1.13
```

## Security boundary

K1.13 does not allow a NAIR program to:

- invent a source identity;
- invent a delivery namespace;
- bypass K1.10 audit correlation;
- complete an undelivered attempt;
- bypass K1.12 delivery-key deduplication;
- choose an arbitrary runtime atom from a callback payload;
- bypass atom ownership;
- gain direct NAM mutation authority.

The new layer changes declaration location, not the K1.12 trust model.

## Durability boundary

K1.13 inherits K1.12's explicit durability limit. Native declaration does not make completion
consumption, source sequence or completion-induced NAM state crash-persistent. Whole-runtime
persistence remains a separate future milestone.

## Surface language

K1.13 changes the canonical intermediate representation only. It does **not** freeze `.noi` syntax.
Future language syntax may compile to the 0.6 declaration once the surface-language design is ready.

## Tests

K1.13 adds **25 native-completion tests** on top of the **362 certified K1.12 tests**, for an expected corpus of **387 tests**. Coverage includes:

- current format and backward decoding;
- opcode version gating;
- canonical round-trip stability;
- all five projection tags;
- slot single-assignment;
- name/projection/atom validation;
- direct-executor fail-closed behavior;
- persistent-runtime rejection outside event-loop bootstrap;
- missing and zero-source host bindings;
- exact event-loop native binding;
- runtime authority installation;
- runtime projection installation;
- host binding exclusion from canonical program bytes;
- semantic declaration differences changing canonical bytes;
- multiple native routes.

## Certification

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

K1.13 is certified only after the local release gate and cross-platform GitHub CI are fully green,
followed by publication of the `k1.13` tag.
