# NORDOI Production Profile 1 — Progress Ledger

This is an engineering progress ledger, not constitutional law. `CONSTITUTION.md` remains the supreme authority.

## Certified baseline entering V1.0

V0.9 is certified at commit `4eb391b35531b970876ac592f69c1989c700fdee`, tag `v0.9`, with local Release Gate PASS, 20/20 dedicated V0.9 tests PASS, selective runtime proofs, and exact-SHA five-job CI PASS (run `37595803690`).

Estimated Production Profile 1 completion entering V1.0 = **70%**, remaining = **30%**.

## V1.0 candidate closure targets

V1.0 closes the first bounded runtime-call gap:

- source-level direct pure function declarations;
- explicit integer parameters;
- dynamic direct calls from the entry expression;
- NAIR `0.12 CALL_EVAL`;
- canonical function identity;
- bounded argument vectors;
- bounded pure call-expression bodies;
- runtime call count observation;
- call-body work observation;
- certified maximum runtime call depth of one;
- compile-time erasure of fully static calls back to NAIR `0.6`;
- deterministic witness, replay, receipt, result, and call-work observations;
- no recursion;
- no indirect calls;
- no function values or closures;
- no arbitrary jump/return address mechanism;
- no general runtime call stack;
- no host callback;
- no hidden authority.

If and only if V1.0 passes the complete local Release Gate, 20/20 focused tests, runtime-call smoke proof, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **77% complete / 23% remaining**.

The percentage is a readiness estimate, not a claim that source lines or milestone count correspond linearly to production readiness.
