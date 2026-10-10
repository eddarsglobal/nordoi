# R0.5 Architectural Decision — Verification hardening without production integration

**Baseline:** `r0.4` (`3ce87f1943c8c97ee304cebad9eff236ae646fa6`).

## Decision

Add only `tests/resource_task_r05.rs` as executable evidence, plus a static validator, governance matrix, spec, research record and README. Treat `src/resource_task_r02.rs` as frozen reference and leave `src/lib.rs`, `Cargo.toml`, the K1.18 kernel, NAIR 0.6, runtime/capability handling and every previously certified file unchanged.

## Why

Prior R0.4 generative coverage is bounded and seed-specific. R0.5 explores 90 valid orderings for one fixed 3-task partial order and audits failure precedence, replay and atomic rejections. Its smaller exhaustive-within-profile claim is more precise than overstating a larger pseudorandom corpus. We additionally test synthetically altered reports to demonstrate that the test oracle detects known changes, and shrink a deliberate error trace for diagnostics.

## Rejected alternatives

- Modifying the reference transition engine to facilitate testing: would break additive-only historic boundaries.
- Claiming production scheduling/fairness: no real task is executed and no backend is integrated.
- Reporting a minimized negative test as a discovered bug: it is an **expected rejection**, not an implementation defect.
- Writing an unstable debug snapshot to a persistent, falsely canonical proof artifact: cross-toolchain byte stability is not established.
- Changing original R0.2 SHA-256 manifest for new R0.5 tests: unrelated to its seven frozen artifacts.

## Limitations and later obligations

The 90 schedules cover only terminal/join orders with outcomes fixed at success/failure/cancel; state explosion beyond this small profile is not addressed. Tests on trusted in-memory Rust cannot prove protection against hostile native code. Future versions should consider independent specification oracles, property-based shrinking for genuinely discovered failures, differential replay encodings, mutation testing under CI and stronger formal model-checking where feasible. All such steps need their own governance/release gates.
