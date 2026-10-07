# NORDOI Production Profile 1 — Progress Ledger

This is an engineering progress ledger, not constitutional law. `CONSTITUTION.md` remains the supreme authority.

## Certified baseline entering V0.8

V0.7 is certified at commit `23f8ed60ce062d1d624842d8dc6c49c5ad02c523`, tag `v0.7`, with local Release Gate PASS, 20/20 dedicated V0.7 tests PASS, runtime proof `INT(42)`, and exact-SHA five-job CI PASS (run `37573867582`).

Estimated Production Profile 1 completion entering V0.8 = **58%**, remaining = **42%**.

## V0.8 candidate closure targets

V0.8 closes the first runtime control-flow gap:

- source-level structured `if/else` whose condition may depend on canonical runtime input;
- compile-time erasure of statically decidable branches;
- NAIR `0.10` structured `BRANCH_VALUE` instruction;
- typed same-kind `INT`/`BOOL` branch values;
- explicit runtime branch-count observation;
- deterministic replay/witness/receipt binding of the selected result;
- no arbitrary jumps or instruction-pointer API;
- no runtime function call stack;
- no recursion;
- no hidden authority.

If and only if V0.8 passes the complete local Release Gate, focused true/false branch proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **64% complete / 36% remaining**.

The percentage is a readiness estimate, not a claim that source lines or milestone count correspond linearly to production readiness.
