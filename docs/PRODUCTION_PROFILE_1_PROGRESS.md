# NORDOI Production Profile 1 Progress

## Certified baseline entering V1.1

V1.0 — Bounded Runtime Function Calls is the certified baseline.

Estimated Production Profile 1 completion entering V1.1 = **77%**, remaining = **23%**.

## V1.1 candidate closure target

V1.1 closes the next language/runtime composition gap by adding statically acyclic, bounded pure runtime call graphs.

Candidate proof targets:

- pure function bodies may directly call other pure functions;
- complete call graph resolved before lowering;
- direct and indirect recursion rejected before runtime;
- maximum call depth statically certified and bounded to 8;
- nested dynamic calls execute through NAIR 0.13;
- simple V1.0 calls remain NAIR 0.12;
- fully static call graphs still erase to NAIR 0.6 CONST + HALT;
- runtime reports exact call count and maximum active depth;
- runtime branches remain zero for this slice;
- authority remains NONE;
- canonical input boundary remains preserved;
- public certified version boundary remains unchanged.

If and only if V1.1 passes the complete local Release Gate, 20/20 focused tests, runtime smoke proof, exact-SHA multi-platform CI, and annotated certification tag, the planned readiness estimate becomes **82% complete / 18% remaining**.

The remaining Production Profile 1 work should then concentrate primarily on usable modules/imports, practical type composition, capability-based I/O integration, developer tooling/diagnostics, packaging/distribution, hardening, and a reference application.
