# NORDOI R0.3 — Bounded Resource & Task State-Space Verification

**STATUS: CANDIDATE. Not a certified runtime, scheduler, formal proof, or universally complete model check.**

## Immutable baseline

- Source reference: `r0.2.1`, commit `d9b7153cac7d044cefdc57645758c331fd75546d`, CI `37971340027` (reported as successful by the maintainer).
- Source under analysis: `src/resource_task_r02.rs`, imported **only** inside `tests/resource_task_r03.rs`.
- No source, runtime, compiler, NAIR, kernel, authority, effect, capability, project manifest, public CLI, or CI modification is included.
- Historical `r0.2` and `r0.2.1` tags must remain unchanged.

## Why R0.3 exists

The 27 R0.2 cases exercise concrete lifecycles. R0.3 adds a deterministic breadth-first, finite exploration of a selected action alphabet. The explorer tries valid **and invalid** transitions. A rejected action must not change the complete model; an accepted action must add exactly one event and pass the frozen model's replay check. A successful closure report must reflect joined tasks, released resources, closed child scopes, sorted identities, and failure-over-cancellation precedence.

## Bounded exploration profiles

| Profile | Tasks | Resources | Children | Max action depth |
| --- | ---: | ---: | ---: | ---: |
| Task lifecycle | 1 | 0 | 0 | 6 |
| Resource & grant | 0 | 1 | 0 | 6 |
| Child-scope lifecycle | 0 | 0 | 1 | 4 |
| Cross-domain mixed lifecycle | 1 | 1 | 1 | 4 |

Shared limits: **8 accepted events** per state, **20,000 explored states maximum** per profile; exceeding the state cap is a **test failure**, not a silent truncated success. The explorer uses Rust ordered collections, breadth-first traversal, and complete debug snapshots (including the causal log) for deterministic visited-state identity. Every accepted edge is replay-checked.

Action alphabet: close, declare effect, host grant of `Read`, acquire, declare task, open child, task admission/start/cancellation/success/failure/acknowledgement/join, use, release, cross-scope release attempts, and grant revocation. Duplicate or phase-invalid operations are deliberately attempted. **This is exhaustive only within this selected alphabet, the generated reachable handles, and the explicit bounds.** No randomness, real threads, scheduling or external effects.

## Additional differential / adversarial cases

- Repeat identical exploration and assert equal summary counts.
- Compare reports for both two-task completion orders over five selected outcome pairs; sorted semantic identities and outcomes must agree.
- Reject a foreign-domain scope and foreign grant without state mutation; grant check is exercised after explicit effect declaration.
- Demonstrate atomic event-budget exhaustion.
- Assert that no `resource_task_r02` / `resource_task_r03` module is exported via `src/lib.rs`.

## What is not verified

Unbounded interleavings, complete transition-alphabet closure, arbitrary numbers of tasks/scopes/resources, liveness, fairness, deadlocks, OS threads, distributed authority, GPU/NPU backends, I/O safety, memory safety of real backends, crash recovery, temporal logic or production throughput. The explorer does not constitute a formal exhaustive proof. No semantic or delivery percentage is increased from this experiment without a separate governed decision.

## Future-Native Gate (6/6 decision basis)

- FNG1 CONSTITUTIONAL_DRIVER: R01-I01→I16 safety responsibilities guide the explored transitions.
- FNG2 NO_FEATURE_PARITY_ONLY: explores native resource/task invariants, not external concurrency API parity.
- FNG3 SIMPLICITY_GAIN: future source syntax can remain small while the implementation is audited beneath it; no new source syntax now.
- FNG4 SECURITY_OR_PROVABILITY_GAIN: rejection atomicity, deterministic replay, closure checks, and compact reproduction traces.
- FNG5 UNIVERSAL_ARCHITECTURE: pure model, no platform-specific scheduler assumption.
- FNG6 CERTIFIED_BOUNDARY_PRESERVATION: test-only additive source; CI must confirm no tracked certified file changed.

## Acceptance rules

1. Static R0.3 validator PASS with 9/9 unique obligations and witnesses.
2. Run R0.1, R0.2 and R0.2.1 validators (7/7 manifest expected).
3. Format **only** `tests/resource_task_r03.rs` with `rustfmt --edition 2021`, then `cargo fmt --all -- --check`; `cargo test --test resource_task_r03 -- --nocapture` succeeds; then full `./scripts/release_gate.sh` succeeds.
4. `git diff --exit-code r0.2.1 -- .` is silent before staging, and `git status --short` lists only the six added R0.3 files. No `cargo fmt` of preexisting certified files is needed.
5. Commit, push, five-job CI for the exact HEAD, then (only then) tag `r0.3`.

## Next design gate

After R0.3: inspect counterexample coverage and blind spots. Any opt-in integration of concurrency with compiler/NAIR/runtime requires a **new separate proposal and certification plan**. Do not conflate reference-model verification with production guarantees.
