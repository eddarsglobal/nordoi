# NAIR 0.6 — Native Effect Completion Semantics

NAIR 0.6 extends NAIR 0.5 with canonical native effect-completion declarations. It does not alter the
certified reaction, time, input, render, transaction or atom opcodes.

## Header

```text
magic: "NAIR"
major: 0
minor: 6
```

Supported historical minors remain 1 through 6. Canonical encoding always emits minor 6.

## New identity

```rust
CompletionSlot(u32)
```

Completion slots are single-assignment program-local identities.

## New instruction

```rust
Instruction::DefineEffectCompletion {
    dst: CompletionSlot,
    name: String,
    domain: DomainRef,
    projections: Vec<NairCompletionProjection>,
}
```

Binary opcode: `0x70`.

## Projection model

```rust
NairCompletionProjection {
    atom: AtomSlot,
    value: NairCompletionProjectionValue,
}
```

Values:

```text
0x00 OutcomeValue
0x01 Succeeded
0x02 IntentId
0x03 AttemptId
0x04 SourceSequence
```

## Authority exclusion

The canonical program contains no completion source ID and no delivery namespace. Host authority is
provided separately through `NairCompletionAuthority` at event-loop boot.

This preserves the constitutional rule that a program cannot create ambient external authority by
serializing it into itself.

## Runtime resolution

Native declarations are removed from the direct persistent-runtime bootstrap and resolved by
`AtomicEventLoop` into K1.12 completion projections. Programs using native completion declarations
therefore require explicit event-loop completion context.

## Version gating

A byte stream that contains opcode `0x70` while declaring minor 5 or lower is invalid. The decoder
does not infer a newer version from the opcode.

## Surface syntax

NAIR 0.6 does not define `.noi` source syntax. Frontends may target this representation later without
changing its certified semantics.
