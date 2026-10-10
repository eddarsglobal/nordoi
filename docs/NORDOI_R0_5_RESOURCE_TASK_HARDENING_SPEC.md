# NORDOI R0.5 — Resource & Task Verification Hardening

**Status:** candidate; never claim CERTIFIED until the local Rust release gate and exact-commit GitHub CI both pass.
**Certified input baseline:** `r0.4` / `3ce87f1943c8c97ee304cebad9eff236ae646fa6`.
**Integration:** **TEST-ONLY**. Frozen `src/resource_task_r02.rs`; no `src/lib.rs` export, no Cargo feature, no NAIR/runtime/compiler modifications. No OS threads, external effects, network grants or real parallel work are introduced.

## Rationale

R0.1 defined 16 resource/task obligations (several remain unimplemented). R0.2 established an isolated deterministic transition model. R0.3 explored bounded state graphs and R0.4 used 16 deterministic generative seeds (16 x 96 steps) and a synthetic reducer. R0.5 adds a **small finite-profile exhaustive ordering witness**, rather than claiming that more random seeds constitute complete verification.

## Finite schedule domain

A root scope declares and admits exactly three tasks with semantic outcomes:

- Task 0: `Succeeded`;
- Task 1: `Failed(23)`;
- Task 2: cancellation requested then `Cancelled`.

After the initial declarations/admissions/cancel request, six model events remain. For each task, the terminal event must precede its corresponding `Join`. There are **90** linear extensions (`6! / 2^3 = 90`). The checker visits all 90 paths, asserts that each accepted transition increments the event log by exactly one, runs `replay_checked()` after each transition and at the final closure, and compares the sorted `ScopeReport` across every valid ordering. This is exhaustive **within that fixed profile** only. Other task counts, outcomes, arbitrary interleavings, hosts and real concurrency are not exhausted.

## Adversarial boundaries

The other 13 independent witnesses check two nested-scope close permutations; per-scope rather than ambient host grants; revoked grant cleanup; no post-release reuse; cumulative task and resource quotas (as defined in the R0.2 model); event caps 0, 1, 2; failure precedence over cancellation; illegal lifecycle transitions; foreign domain identities/host permits; synthetic mutation sensitivity; minimization of an *expected* invalid `ExternalIo` declaration; repeated in-process transcripts; and the test-only integration boundary.

**Atomic rejection condition:** on each negative witness, the entire `Model` value, accepted log length and `replay_checked()` identity must remain unchanged. Invalid operations must not silently grant authority.

**Test limitation:** debug strings and in-process comparisons are diagnostic witnesses only, not durable canonical binary serialization, cryptographic attestation, provenance guarantees, or an empirical production performance measurement. A minimized *expected rejection* is not an implementation vulnerability. No R0.5 formal mathematical proof is claimed.

## Constitutional future-native gates

| Gate | R0.5 requirement | Scope |
|---|---|---|
| FNG1 | Explicit scope/host authority; cross-scope and foreign-domain identities never confer grants | finite model tests |
| FNG2 | No use-after-release or resurrection; revocation cannot block cleanup | finite model tests |
| FNG3 | Child scopes and joins must complete before parents close | two close orders and 90 task orders |
| FNG4 | Cancellation is not silent success; failure dominates cancellation in reports | selected failure/cancel examples |
| FNG5 | Bounded work: 90 finite linearizations, local event budgets, no external scheduler | finite test harness |
| FNG6 | Deterministic replay, atomic rejections, explicit negative and synthetic witness provenance | in-memory model tests |

## Acceptance protocol

1. `python3 scripts/validate_r05_hardening.py` — static structure and 14-to-14 governance traceability; **not a Rust execution**.
2. `python3 scripts/validate_r02_manifest.py` — historic seven source hashes remain valid.
3. `rustfmt --edition 2021 tests/resource_task_r05.rs`, `cargo fmt --all -- --check`, `cargo test --test resource_task_r05 -- --nocapture`, `./scripts/release_gate.sh` — actual local Rust gate.
4. `git diff --exit-code r0.4 -- .` and `git status --short` — additivity, no tracked modifications.
5. Stage six files only; GitHub CI must pass five jobs for the **exact commit SHA**; only then publish the annotated `r0.5` tag.

**Non-claims:** production concurrency, scheduler fairness, real-time deadlines, full state-space coverage, liveness for unbounded systems, formal verification, security impossible to breach, a new source-language feature, or an upward recalculation of certification percentage.
