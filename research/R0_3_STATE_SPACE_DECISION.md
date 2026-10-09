# R0.3 Architectural Decision — Finite State-Space Verification

Decision: add a bounded test-only explorer to the R0.2 reference semantics, rather than implement a scheduler or change `.noi` grammar.

## Alternative assessment

1. Immediately introduce threads / async executors: rejected; unproved semantics and unwanted authority migration.
2. Introduce a new NAIR operation: deferred; would affect certified boundaries before the state model is more thoroughly challenged.
3. Introduce a test-only, breadth-first model explorer: selected; deterministic and opt-in with explicit coverage limits.

## Safety design

Every rejected operation is compared against an exact pre-operation clone; every accepted operation must increment its causal log by one and replay exactly; successful closure is checked against independently tracked resource liveness, task identities/outcomes, and child closure. Finite budgets protect CI and make reproduction bounded. On failure the test prints an operation trace, not an unqualified security claim.

## Open questions

- Generate a richer legal transition alphabet with transparent, reviewable coverage metrics.
- Add stronger schedule equivalence and partial-order reduction without concealing paths.
- Consider independent executable specifications and property-based mutation testing.
- Separate safety invariants (finite traces) from liveness properties (infinite traces).
- Establish explicit interfaces for future authority-preserving opt-in execution without altering frozen baselines.

No change to the 58.35% **PLANNING_NOT_CERTIFICATION** delivery estimate or to 322-principle evidence claims is justified solely by these new tests.
