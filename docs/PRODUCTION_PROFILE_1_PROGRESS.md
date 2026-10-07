# NORDOI Production Profile 1 — Progress Ledger

This is an engineering progress ledger, not constitutional law. `CONSTITUTION.md` remains the supreme authority.

## Certified baseline entering V0.9

V0.8 is certified at commit `96817a06c2479f06a9dd50d2b2568afb15e1eacf`, tag `v0.8`, with local Release Gate PASS, 20/20 dedicated V0.8 tests PASS, dynamic true/false runtime proofs, and exact-SHA five-job CI PASS (run `37592074408`).

Estimated Production Profile 1 completion entering V0.9 = **64%**, remaining = **36%**.

## V0.9 candidate closure targets

V0.9 closes the first selective dynamic branch-body gap:

- source-level dynamic `if/else` whose selected arm may perform checked pure runtime computation;
- lazy evaluation of exactly one selected branch body;
- zero observable evaluation work from the unselected branch body;
- NAIR `0.11` structured `BRANCH_EVAL` instruction;
- bounded pure branch-expression trees over canonical values and already-defined registers;
- checked integer addition and integer comparison inside a selected branch body;
- preservation of NAIR `0.10 BRANCH_VALUE` when both dynamic-branch arms are compile-time values;
- preservation of NAIR `0.9` when a static condition selects a dynamic straight-line expression;
- preservation of base NAIR `0.6` for fully static programs;
- deterministic witness, replay, receipt, result, and branch-work observations;
- no arbitrary jumps or instruction-pointer API;
- no runtime function call stack;
- no recursion;
- no speculative evaluation of both branch bodies;
- no hidden authority.

If and only if V0.9 passes the complete local Release Gate, 20/20 focused tests, true/false selective runtime proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **70% complete / 30% remaining**.

The percentage is a readiness estimate, not a claim that source lines or milestone count correspond linearly to production readiness.
