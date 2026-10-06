# V0.4 Research Record — First Executed Pure Binding Program

## Constitutional question

How can NORDOI execute programs containing source-level names without making names a permanent runtime tax?

The C0.10 answer is compile-time erasure. V0.4 validates that this decision survives actual execution.

## Decision

Do not add a runtime binding abstraction.

V0.4 reuses the already-certified observed runtime path. It executes the exact C0.10 NAIR and independently reconstructs the expected transient register values from the C0.9 postfix binding plan and L0.8 canonical registry.

This is intentionally stronger than checking only the final result.

For:

```noi
const x = 20;
const y = 22;
entry main returns x + y;
```

V0.4 requires the complete transient evidence:

```text
r0=20
r1=22
r2=42
```

## Why no LOAD_BINDING opcode exists

A `LOAD_BINDING` instruction would encode a source abstraction into runtime machinery even though L0.8 bindings are immutable compile-time values. That would create unnecessary representation, lookup and possibly storage costs.

C0.10 instead emits the same `CONST` instruction as an equivalent literal. V0.4 confirms that this has no observable execution penalty.

## Replay versus receipt

Runtime replay identity describes the operational program plus canonical input. V0.4 receipt identity additionally commits to the C0.10 semantic witness.

Therefore these may intentionally differ:

```noi
const x = 42;
entry main returns x;
```

and

```noi
const x = 42;
entry main returns 42;
```

They may share runtime replay identity because they execute identical NAIR, while retaining distinct V0.4 receipts because source-level certified semantics are not identical.

This separation supports both Atomic Speed and Canonical Semantics.

## Security posture

V0.4 adds no host interaction. The validator fails closed on:

- forged result/intermediate registers;
- extra register observations;
- input activity;
- residual scheduled work;
- atoms/domains/transactions;
- render/input bridge state;
- effect or authority requirements;
- mismatch between C0.10 lowering and reconstructed binding erasure.

## Non-goals

V0.4 does not add mutable variables, assignment, runtime locals, functions, imports, calls, branching, I/O, effects, capability acquisition or new NAIR instructions.
