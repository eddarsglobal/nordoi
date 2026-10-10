# NORDOI R0.9 — Sibling-Branch Resource/Task Conformance Specification

**Parent certification:** `r0.8` / `35fc2487ce180ff1fdc1a0f2fa8df3213127865d`. **Status:** candidate, TEST-ONLY. All historical certified files and the R0.2 reference implementation stay unchanged. One new standalone integration test (`tests/resource_task_r09.rs`) implements its own bounded oracle and imports the reference model as a private module only.

## B1. Fork topology and scope accounting

Root scope (0) may open **exactly two** sibling child scopes (1, 2); each child may open **one** leaf child (3, 4). No fifth generation or third root child is allowed. The maximum of five scopes is fixed by the test fixture, **not** a certified NORDOI language/runtime limit. Each scope has two cumulative task and resource slots. Closing a child does not recycle the parent's child slots.

## B2. Independent prediction

`Oracle::decide` maintains its own separate `OScope`, `OGrant`, `OResource`, and `OTask` vectors and counters. It predicts exact `ModelError` precedence, receipts, state updates, canonical close reports and hierarchy aggregation *without* inspecting the SUT model, SUT receipts or query helpers. Only after prediction does `Trial::step` invoke R0.2. Each rejected attempt must leave both cloned candidate states unchanged. `replay_checked()` is supplemental self-replay evidence, not the independent oracle.

## B3. Branch and cousin authority boundaries

Read/Write declarations and grants are scope-exact. A grant from sibling A cannot authorize sibling B, a parent cannot authorize its grandchild implicitly, and a resource in leaf A cannot be used or released by leaf B. The same denial applies when the attacker supplies a valid grant owned by the *wrong* scope. Revocation prevents subsequent use but cannot prevent release and closure in the matching scope.

## B4. Non-recyclable quotas and ordered closure

Local task/resource creation quotas are cumulative, not renewed after release or completion. Parent closure fails with `OutstandingChildren` while either sibling is open. Within a branch, leaf closure precedes child closure. When children close in reverse order, reports must still be sorted by canonical scope identity. Children are not silently closed by parent close.

## B5. Failure and cancellation precedence

A failed sibling task dominates a cancelled task in another branch, yielding `ScopeOutcome::Failed` for the root. Cancellation in both leaves propagates as cancelled through both ancestors and to the root. Termination, join and close are distinct model events, not real parallel work or asynchronous preemption.

## B6. Bounded generated corpus

Exactly **40 deterministic seeds × 160 generated attempts = 6,400 comparisons** under a five-scope bootstrap script. Rejection atomicity and accepted-event logging are checked for every operation. A fixed xorshift generator has no OS randomness; neither the sample nor its count is a statistical safety estimate.

## B7. Bounded exhaustive word corpus

Exactly **7^5 = 16,807 words** of length five over a seven-action alphabet (selected spawn/close actions). The test enumerates every one of these words and compares all transition results. This is not exhaustive over the full command alphabet, arbitrary orderings or larger scope trees.

## B8. Four-task valid interleavings

Four tasks are distributed over root, the two siblings and one grandchild. All **2,520 valid linear extensions** of four finish-before-join constraints are checked (8! / 2^4), with canonical final leaf/child/root reports. The result is strictly a sequential enumeration of orders, not OS-level concurrent scheduling.

## B9. Witness traceability and synthetic mutation

23 Rust `#[test]` functions match 23 governance TSV rows 1:1, each with its type and explicit limitation. The synthetic mutant changes a *test-expected receipt* intentionally to demonstrate mismatch detection; it is not a discovered defect in R0.2. No runtime/NAIR/kernel API is exported.

## B10. Certification gate and non-claims

Local success requires 23/23 tests, strict Rustfmt/Clippy, all-target cargo checks/tests, all frozen validators, silent `git diff --exit-code r0.8 -- .` and exactly six untracked candidate files. The actual certified release requires **five GitHub Actions successes on the exact commit SHA** and a later annotated tag. No actual hardware/OS concurrency, thread interleaving, distributed cancellation, scheduler fairness, FFI safety, memory safety proof, arbitrary-depth model checking, or absolute security theorem follows from these tests. Any future native-resource/task integration needs a separately governed API/authority decision.
