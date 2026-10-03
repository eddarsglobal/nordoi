# NAIR Native Effect Completion Specification 0.1

**Milestone:** K1.13  
**Canonical NAIR:** 0.6

## Purpose

This specification binds NAIR 0.6 native completion declarations to the governed effect completion
re-entry core certified in K1.12.

The bridge deliberately separates **program meaning** from **runtime authority**:

- NAIR declares the completion slot, semantic projection domain and target atom projections;
- the host binds that slot to an exact completion source and effect-delivery namespace;
- K1.12 validates the actual completion cause against K1.10 audit history before state mutation.

## Instruction

```text
0x70 DEFINE_EFFECT_COMPLETION
```

Canonical payload:

```text
u32  completion_slot
str  name
domain_ref
u32  projection_count
repeat projection_count:
    u32 atom_slot
    u8  projection_value
```

Projection values:

```text
0x00 OutcomeValue
0x01 Succeeded
0x02 IntentId
0x03 AttemptId
0x04 SourceSequence
```

## Validation

A declaration is invalid when:

- the slot was already defined;
- the name is empty after trimming;
- its domain reference is not yet defined;
- it contains no projections;
- a projected atom is not yet defined;
- one declaration projects the same atom more than once.

## Host binding

A valid declaration does not itself carry authority. Before event-loop boot, the host supplies:

```text
CompletionSlot → EffectCompletionSourceId + EffectDeliveryNamespace
```

The source must be non-zero. Missing binding fails boot.

## Bootstrap

For each native declaration, event-loop bootstrap:

1. resolves the declaration domain to a `DomainId`;
2. resolves every `AtomSlot` to its runtime `AtomId`;
3. grants the exact host-bound `(source, namespace)` route in `AtomicEffectCompletionCore`;
4. registers each projection through the normal K1.12 projection API;
5. therefore revalidates atom ownership;
6. stores the slot-to-binding mapping for introspection.

No alternate completion mutation path exists.

## Direct execution

The direct NAIR executors and `PersistentAtomicRuntime::boot` do not possess the complete governed
completion context. They reject programs containing `DEFINE_EFFECT_COMPLETION` with
`CompletionContextRequired`.

`AtomicEventLoop` is the native orchestration surface.

## Replay

The serialized declaration is canonical program meaning and is committed by the NAIR bytes in the
initial event-loop replay state. Host bindings are not serialized. Actual accepted completion causes
remain replay meaning through the K1.12 canonical completion report.

## Compatibility

NAIR 0.6 continues decoding supported NAIR 0.1–0.5 programs. Opcode `0x70` is rejected under a
declared minor lower than 6.
