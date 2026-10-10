# R0.7 Research Decision — Expand Independent Oracle to One Child Scope

Certified predecessor: `r0.6` / `9c12bee933c13eff313aa11e49c02d56d86e91f3`.
This R0.7 artifact is **TEST-ONLY**, not a runtime promotion.

**Decision:** provide an independently authored, deterministic two-scope
transition oracle that extends the narrower R0.6 single-scope oracle. Compare
exact typed result signatures and `ModelError` variants, normalize handles by
model-domain-safe ordinals, and compare sorted per-scope task/child outcomes.

**Why:** internal replay checks alone cannot expose a shared conceptual bug
between accepted model events and their receipts. The new oracle has its own
transition implementation, budgets, counters, task phases and reports.
SUT state is used **only** to check actual output, rejection atomicity and
replay consistency, never as the expected specification.

**Constraints:** one root and one child only; two tasks and resources per scope;
no grandchildren, external host effects or real thread execution; bounded
finite-word and seeded sequences; no claims of universal safety or completeness.
The six admissible finish/join orderings demonstrate model order invariance in
this particular profile. Synthetic mutation tests do **not** establish a real
NORDOI security defect.

**Governance:** 18 tests / 18 classifications / 18 explicitly limited witness
rows. Existing `r0.2` manifest and CI remain unchanged. If the Mac compiler,
Clippy or exact GitHub SHA gate fail, do not publish a tag. The prior tagged
baseline remains authoritative.
