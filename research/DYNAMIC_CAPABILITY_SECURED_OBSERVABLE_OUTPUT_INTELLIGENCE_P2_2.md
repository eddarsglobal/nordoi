# P2.2 Dynamic Capability-Secured Observable Output Intelligence

## Decision

The safest next observable step is not a general string runtime. It is a scalar bridge from already-certified runtime computation to already-certified explicit console authority.

## Reuse instead of semantic duplication

P2.2 reuses V0.7 dynamic input computation. The contextual source token `emits` is lowered into the certified dynamic `returns` vertical for computation; P2.2 then governs only the observable boundary.

This keeps arithmetic/comparison semantics inside the existing NAIR/runtime proof surface and avoids a second evaluator.

## Authority ordering

The ConsoleWrite grant is checked before runtime execution. Although the reused V0.7 computation is pure and closed, denying early minimizes work and guarantees no P2.2 observable execution occurs without exact authority.

## Receipt composition

A dynamic observable receipt must commit both to what was computed and how that computation was certified. Therefore P2.2 includes the SHA-256 of the V0.7 canonical runtime receipt rather than only the rendered text.

## Deliberate limits

String interpolation is deferred. Adding it in P2.2 would mix three concerns at once: runtime values, text formatting semantics, and observable authority. P2.2 certifies the value-to-output bridge first.
