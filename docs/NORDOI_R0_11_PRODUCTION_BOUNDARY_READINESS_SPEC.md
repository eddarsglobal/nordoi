# NORDOI R0.11 — Production Boundary Readiness Contract

**Frozen baseline:** `r0.10` / `16de3cf71b4be39118eb2bb4b963c64ceeb5a65a`. **Candidate scope:** TEST-ONLY, NO runtime authority, NO native scheduler, NO compiler/NAIR change.

## Separation of evidence and implementation

R0.2–R0.9 comprise a bounded deterministic task/resource reference and experimental verification suites. R0.10 audits **117** historical witness declarations and adds **18** consolidation tests. None of those facts demonstrates real OS-thread scheduling, asynchronous cancellation, external capability enforcement or native memory safety. No real native integration is delivered by R0.11.

## Native readiness register

The machine-readable `governance/r11_native_readiness_gates_v1.tsv` defines exactly **12** individually testable decision areas. They deliberately start at `status=UNMET`, `evidence_scope=RESEARCH_ONLY` with explicit acceptance criteria and limitations. Presence of criteria is not satisfaction of criteria. To consider the native project independently for technical review, each requirement would need independently obtained native implementation evidence, review of adverse test findings, and explicit accountable governance approval. This candidate contains none of those items.

## Bounded advisory semantics

A Rust integration *test-only* helper reads the registry, checks its structure and calculates a three-valued synthetic **advisory**: `Invalid`, `Blocked`, or `ReviewEligible`. Missing, duplicate or ill-formed gate entries fail closed. A mock CI PASS or mock approval cannot override a single `UNMET` gate. Synthetic completion of all gates without approval still blocks review. A synthetic fully verified set with mock CI PASS and mock approval yields only `ReviewEligible` — NEVER `ProductionApproved`, never an executable or a native capability.

Inputs to that helper are ordinary, forgeable test data. Its `ReviewEligible` case is not actual security certification; no authenticated approval mechanism or real verification evidence exists. No code in this candidate can authorize a production deployment.

## Six distinctions that must remain explicit

1. `cargo test` passing is not a proof of real native concurrency.
2. Static source scans are not memory-safety certification or a comprehensive export audit.
3. A modeled `HostPermit` is not nonforgeable native authority against untrusted Rust.
4. Finite state exploration is not mathematical verification of all schedules.
5. Git tags/CI provide repository and pipeline traceability, not an independent penetration test.
6. The advisory review state is not a production-release state.

## Release conditions for research R0.11 only

- Python validator checks 12/12 research-only UNMET gates, 20/20 test-to-matrix bijection and strict `r0.10` Git boundary including untracked paths.
- R0.2–R0.10 historical validators PASS without edits; `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo check --all-targets`, `cargo test --all-targets`, release gate PASS.
- Exactly seven new files, `src/`, `Cargo.toml`, `Cargo.lock`, `tests/resource_task_r02.rs` through `r10.rs` and all other certified paths unchanged.
- Five GitHub Actions jobs PASS on the exact commit SHA, then annotated `r0.11` research tag may be published. Production integration remains BLOCKED.
