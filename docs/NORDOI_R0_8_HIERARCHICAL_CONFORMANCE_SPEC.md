# NORDOI R0.8 — Hierarchical Resource/Task Reference Conformance Specification

**Input certification:** `r0.7` / `8452ce32a27ea93c4eb343dec7165ae4b6be704a`. **Status:** Candidate, TEST-ONLY. **Interface:** existing R0.2 resource/task model, imported exclusively by a Rust integration test. **No** kernel, compiler, runtime scheduler, NAIR or host authority export.

## Finite, independently predicted model

The oracle in `tests/resource_task_r08.rs` implements its own `Oracle::decide()` and state vectors: it does not derive predictions from the system under test (SUT), its internal state, receipts, `phase()`, `outcome()`, or `report()`. Each operation is independently predicted and compared against the frozen R0.2 transition model, including exact denial precedence and results. On every rejection, both model and oracle state must remain unchanged; only accepted transitions count toward the global event quota. `replay_checked()` additionally checks the accepted R0.2 causal log; that replay is not used as an independent oracle.

### H1 — Hierarchy admissibility
One root (index 0), at most one child (index 1), at most one grandchild (index 2). Root and child each have child budget 1; grandchild child budget 0. Total at most three scopes. Parent scopes cannot close with open children.

### H2 — Local budgets and identity
Each scope has task and resource budgets 2 each, counted cumulatively, not reusable after completion or release. Scope/task/resource/grant identity is allocated from a monotone per-domain sequence, not transferred by inheritance.

### H3 — Scope-exact authority
Effects must be declared in the same scope; parent/child/grandchild grants are not ambient across generations. A foreign-domain handle or permit never confers same-domain authority; ExternalIo stays unsupported.

### H4 — Resource lifecycle
A resource cannot be used or released by another scope, including its parent. Revoking a grant prevents further use but does not prevent explicit release and close.

### H5 — Task lifecycle
Declared -> admitted -> running -> terminal -> joined (admitted may terminate directly). Invalid transitions are atomic rejections; cancellation request and subsequent acknowledgement are distinct. A requested cancellation may still yield failure.

### H6 — Outcome precedence
Closure sorts task and child reports by identity. Failed task/child takes priority over cancellation; cancellation takes priority over success. The hierarchy propagates child outcome to parent and grandparent only at explicit closure.

### H7 — Global budget and replay
The accepted-event budget is global to all generations. Rejections spend no accepted events. Exactly replayable accepted traces and stable semantic reports are checked against the R0.2 model within this finite fixture; this is not durable/signed audit evidence.

### H8 — Generated differential corpus
**32 deterministic seeds × 160 attempted commands = 5,120 comparisons**, including invalid indices, unsupported effects, denial precedences, grants, tasks, resource operations and scope closures. Pseudorandom inputs are deterministic and fixed, not exhaustive or statistically sampled.

### H9 — Finite-word profile
**6^5 = 7,776 five-operation words** over a restricted alphabet (open root child, open grandchild, create leaf task, close leaf, close child, close root). Exhaustive over these words only, not the full command alphabet or unbounded sequences.

### H10 — Cross-generation interleavings
Exactly **90 legal linear extensions** of three independent finish-before-join constraints (6!/2^3), one task in each generation. Each legal order must yield identical canonical leaf, child and root closure reports; the six two-task orders from R0.7 remain regressions. This is sequential schedule enumeration, not real concurrency.

### H11 — Negative witnesses
Targeted comparisons exercise nested capacity exhaustion, denied grandchild use of ancestor resources, denied use of ancestor grants, outstanding child/resource/task precedence, failure propagation and cancellation propagation. Every unexpected difference must fail a test.

### H12 — Governance / Future-Native gate
Twenty-six named tests map 1:1 to the TSV. No tag or promotion to runtime until Rust compilation, formatting, Clippy, all tests, clean Git diff, and five exact-SHA CI jobs succeed. Future integration requires an explicit governance decision, API isolation review, capability/sandbox model and architecture-specific evidence. A passing fixture does **not** prove global scheduler safety or prevent all exploits.

## Explicit non-claims

This package is not an implementation of parallel scheduling, async runtime, native OS resources, memory-safe task ownership across FFI, cryptographic attestation, browser policy, formal exhaustive verification, or a universally secure language. It is restricted to one three-generation chain and bounded scenarios. Even all 90 valid orderings do not simulate real threads or races.
