# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.2

V1.1 — Acyclic Bounded Runtime Call Graphs is the certified baseline.

Estimated Production Profile 1 completion entering V1.2 = **82%**, remaining = **18%**.

## V1.2 candidate closure target

V1.2 closes the next runtime-language composition gap by allowing selective structured `if/else` control inside bounded pure runtime function bodies.

Candidate proof targets:

- structured function-body conditions may depend on runtime parameters;
- only the selected arm executes;
- discarded-arm expression work is zero;
- selected arms may invoke statically known acyclic pure callees;
- direct and indirect recursion remain rejected before runtime;
- maximum call depth remains statically certified and bounded to 8;
- structured function control requires NAIR 0.14;
- V1.0 simple calls remain NAIR 0.12;
- V1.1 acyclic nested calls remain NAIR 0.13;
- fully static function control still erases to NAIR 0.6 `CONST + HALT`;
- runtime call and branch work remains explicitly observed;
- authority remains NONE;
- canonical input boundary remains preserved;
- public certified version boundary remains unchanged.

If and only if V1.2 passes the complete local Release Gate, 20/20 focused tests, positive/negative runtime smoke proofs, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **86% complete / 14% remaining**.

The remaining Production Profile 1 work should then concentrate primarily on usable modules/imports, practical type composition, capability-based I/O integration, developer tooling/diagnostics, packaging/distribution, hardening, and a reference application.
