# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.3

V1.2 — Nested Structured Runtime Control in Function Bodies is the certified baseline.

Estimated Production Profile 1 completion entering V1.3 = **86%**, remaining = **14%**.

## V1.3 candidate closure target

V1.3 shifts the profile from runtime layering toward practical language usability by introducing real `.noi` modules and deterministic static imports.

Candidate proof targets:

- canonical dotted module identities map to deterministic source-root paths;
- every file declares the exact module identity being loaded;
- reachable imports are resolved before lowering;
- transitive imports work;
- missing modules fail closed;
- duplicate canonical module identities fail closed;
- ambiguous short aliases fail closed;
- recursive/cyclic import graphs fail closed;
- imported modules are library-only in the first V1.3 slice;
- qualified imported pure function calls compose with V1.0/V1.1/V1.2 semantics;
- module order does not change the canonical witness;
- import mechanics are erased before NAIR;
- no new NAIR minor is introduced;
- runtime filesystem authority remains NONE;
- public certified version boundary remains unchanged.

If and only if V1.3 passes the complete local Release Gate, 20/20 focused tests, positive/negative multi-file smoke proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **89% complete / 11% remaining**.

The remaining Production Profile 1 work should then concentrate on practical type composition across modules, capability-based I/O integration, diagnostics/source spans across module graphs, package/distribution metadata, hardening/fuzzing, and a reference application.
