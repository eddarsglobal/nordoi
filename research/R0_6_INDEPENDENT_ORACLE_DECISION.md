# R0.6 architecture decision — independent conformance oracle

Status: **CANDIDATE**, not certified.
Certified reference baseline: `r0.5` at `6cebd4051304c81901669303265c0ad7456b7e12`.

## Accepted direction

Use an **independently implemented** reduced state machine as a differential
oracle for a declared finite profile of the R0.2 TEST-ONLY reference model.
The oracle predicts precise receipts and errors without consulting actual
model outcomes or cloning model internals. Stronger than a self-replay check
alone, but far narrower than full refinement or formal equivalence.

Reject any semantics drift, including error ordering, post-close operations,
rejected-event atomicity, and causal log exhaustion. Preserve the existing
release-gate, non-export and capability boundaries. Do **not** implement
scheduler/worker threads, async runtime, network effects or new NORDOI syntax.

## Unresolved / explicitly not proven

The independent oracle only models **one root scope, at most one task and one
resource, and Read effects**. The mutation witness is artificial; an expected
rejection is not a discovered vulnerability. No general concurrency, liveness,
unbounded state-space or production security conclusion follows. Cross-platform
success remains to be confirmed through CI, not this document.

The next step after CI certification, if successful, should be an explicit
refinement-gap inventory before anyone proposes exporting runtime concurrency.
