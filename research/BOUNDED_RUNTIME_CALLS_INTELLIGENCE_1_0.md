# Dynamic Runtime Call Intelligence 1.0

V1.0 adds runtime calls as a structured semantic primitive rather than importing a general VM execution model.

The design intentionally separates **call semantics** from **arbitrary control transfer**. `CALL_EVAL` cannot jump to an instruction address. It receives bounded arguments and a bounded pure expression body encoded canonically in NAIR. This makes the call observable and real at runtime while preserving deterministic validation, SSA result assignment, and zero hidden authority.

The crucial restraint is depth. Function bodies cannot call functions in V1.0. Consequently the runtime does not need a dynamically growing stack, return-address machinery, indirect dispatch, or recursion guards. The certified maximum call depth is one.

This is a deliberate staging point. Future milestones may widen call composition only if they preserve explicit cost bounds, deterministic identity, memory safety, capability isolation, and proof-friendly validation.
