# NORDOI R0.7 — Independent Two-Scope Conformance Specification

Frozen certification input: `r0.6` / `9c12bee933c13eff313aa11e49c02d56d86e91f3`.
R0.7 is a **TEST-ONLY** experiment, **not** a runtime semantics promotion.

## Normative finite profile

- **MS1 / Identity:** one root, optionally one child, no deeper nesting.
  Child identity and global resource/task/grant ordinals are normalized only
  after operations succeed; foreign-domain handles stay invalid.
- **MS2 / Independent transition oracle:** an independently written `Oracle`
  predicts exact error variants, typed receipt kind/ordinal and event budget.
  It never derives predictions from `Model::phase`, `Model::outcome`,
  `Model::report` or the actual SUT receipt.
- **MS3 / Hierarchy:** the parent cannot close before its child; child closure
  contributes its outcome to the parent report. No grandchild capacity exists.
- **MS4 / Capabilities:** declaration and grant are distinct. Grants and
  resources are scope and effect exact; revocation blocks use without blocking
  cleanup, and unsupported ExternalIo stays denied.
- **MS5 / Bounded tasks:** at most two tasks per scope. Declared -> admitted ->
  running/terminal -> joined transitions, cancellation and failure precedence
  and sorted closure reports are checked against oracle output.
- **MS6 / Bounded resources:** at most two resources per scope. Release does
  not reclaim cumulative admission capacity and cannot resurrect a resource.
- **MS7 / Atomic rejection:** any rejected operation preserves exact SUT
  snapshot, exact oracle state and accepted event count.
- **MS8 / Replay consistency:** every accepted causal event is checked with
  `Model::replay_checked`; this is an internal consistency check, not an
  independent oracle for the expected receipt.
- **MS9 / Order invariance:** all six valid interleavings of two independent
  complete/join pairs produce the same canonical, sorted parent report.
- **MS10 / Boundary preservation:** source and runtime are untouched; no
  new library exports or authority from a TEST-ONLY integration test.

## Measured profiles (run by Rust tests, not by the Python validator)

1. Generated profile: 32 seeds × 128 attempted operations = 4,096 attempts.
   The fixed first child-creation event per seed is separately verified.
2. Enumerated profile: 5^5 = 3,125 words of length 5 over **only** the
   `Child(root)`, `Task(root)`, `Task(child)`, `Close(root)`, `Close(child)` symbols.
3. Interleaving profile: all 6 valid linear extensions of two ordered pairs
   (`finish(root)` before `join(root)`, `finish(child)` before `join(child)`).
4. Model limits: one root, at most one child, up to two tasks and resources
   per scope; global event budget 64 per normal trial, smaller for a budget test.
   Other grant counts are bounded by the accepted-event budget, not by a
   claim that two grants suffice.

## Scope boundaries and evidence taxonomy

The finite word test is exhaustive **only over its chosen alphabet and depth**.
The generated profile explores a deterministic sample, not the state space.
Completion/join permutations are simulated model operations, not actual
parallel execution or proof of absence of races. This verification is not a
formal universal proof, stress test of an OS scheduler, real capability
security boundary, signed replay format, cryptographic audit, or production
concurrency certification. Synthetic oracle mutations are deliberately
introduced to test detection and are **not** model vulnerabilities.

Evidence classes: `GENERATED_DIFFERENTIAL`, `FINITE_EXHAUSTIVE`,
`BOUNDED_DIFFERENTIAL`, `FINITE_INTERLEAVINGS`, `DETERMINISM_EXAMPLE`,
`SYNTHETIC_MUTATION`, `STATIC_BOUNDARY`.

**Frozen:** `r0.6` code, `r0.2` model, K1.18 kernel and NAIR 0.6. No existing
files are replaced, patched, or reformatted by the ZIP package itself.
