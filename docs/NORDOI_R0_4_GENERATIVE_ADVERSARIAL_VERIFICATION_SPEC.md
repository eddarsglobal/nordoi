# NORDOI R0.4 — Deterministic Generative & Adversarial Resource/Task Verification

**STATUS: CANDIDATE / TEST-ONLY / NOT CERTIFIED.** No concurrency runtime, scheduler, language syntax, kernel semantics or production security claim.

## Immutable certified baseline

- R0.3 reference: tag `r0.3`, commit `cb9e54b16e7dfc91273976e689d18ae742ad6476`; CI `37974548950` (five jobs reportedly successful).
- Frozen model: `src/resource_task_r02.rs`, imported as a private module **only in `tests/resource_task_r04.rs`**.
- R0.2 integrity: existing `MANIFEST.sha256` remains owned by the certified r0.2.1 baseline; do not edit its seven entries.
- The candidate is **six NEW files**. No existing `src/`, `tests/`, governance, CI, `Cargo.toml`, `VERSION`, NAIR or CLI files are replaced.

## Motivation and hypotheses

R0.2 provided 27 concrete model scenarios. R0.3 traversed four finite state-space profiles with nine tests. R0.4 adds a different complement: adversarial **deterministic generated sequences** and **metamorphic examples**, while keeping replay and rejected-transition atomicity observable.

For each generated operation: rejected transitions MUST preserve the *whole* model and accepted-event count; accepted transitions MUST advance exactly one event and pass `replay_checked`; successful scope closure MUST agree with joined task reports, child outcomes, released resource handles and failure-before-cancellation precedence. The generator deliberately attempts duplicate, premature, unauthorized, external-I/O and foreign-domain actions.

## Explicit generation limits and replay

- Seeds: 16 fixed seeds (`0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987`), **not** host randomness.
- Generator: explicit SplitMix64-style fixed arithmetic in `Sequence::next`, no dependency or global state.
- Maximum **96 generated operations per seed**; maximum **48 accepted events per model**, 3 tasks/3 resources/2 child scopes at root.
- Action alphabet includes declare Read/Write, host grant, acquire/use/release, revoke, tasks and cancellation transitions, open/close child, foreign-domain and unsupported I/O attempts.
- On assertion failure, the Rust panic includes the generated sequence prefix (`trace`), seed is fixed in the test corpus, and full model snapshots are compared on same-seed reruns. Debug prints are **not canonical cross-toolchain bytes**.
- Reducer is delta-deletion on a **synthetic failure predicate**; it demonstrates reproducibility machinery but makes **no claim** to have found an actual R0.2 flaw, or global minimality.

This is not an exhaustive search or a proof of invariants for all input histories. Generator distribution is intentionally simple and biased, with dynamic handle pools; real scheduling/fairness/deadlock properties are outside the model.

## Twelve independently named Rust witnesses

The normative test-witness mapping lives in `governance/r04_adversarial_witnesses_v1.tsv`, one row per `#[test]`, with a required explicit limitation. Six categories are not conflated: bounded generated testing, metamorphic examples, synthetic shrinking, adversarial examples, and test-only/static boundary assertions.

## Future-Native Gate rationale

- **FNG1** constitutional driver: explicit ownership, grants, replay, bounded work, cancellation/failure visibility.
- **FNG2** no feature parity only: tests NORDOI native resource/task semantics, not external async API parity.
- **FNG3** simplicity gain: no new developer-visible keywords/API and zero production binary overhead.
- **FNG4** security/provability gain: negative transition atomicity and reproducible adversarial witnesses.
- **FNG5** universal architecture: pure host-neutral Rust reference without OS/backend assumptions.
- **FNG6** certified boundary preservation: no modifications to r0.3 tracked source; CI verifies unchanged semantic boundaries.

## Acceptance, refusal and certification policy

1. `python3 scripts/validate_r04_adversarial.py` -> PASS, 12/12 witnesses, exactly 16 seed values and declared limits.
2. `python3 scripts/validate_r02_manifest.py` -> 7/7; existing R0.1, R0.2, R0.3 validators -> PASS.
3. `rustfmt --edition 2021 tests/resource_task_r04.rs` **only**; `cargo fmt --all -- --check`; `cargo test --test resource_task_r04 -- --nocapture`; `./scripts/release_gate.sh` -> PASS. Fix issues only in added R0.4 files.
4. `git diff --exit-code r0.3 -- .` silent; `git status --short` exactly six candidate files. No `Cargo.lock`, `.git`, `target`, `__pycache__` committed.
5. After review: explicit 6-file `git add`, commit, push; ensure five CI jobs for exact HEAD green; only then annotate/push immutable tag `r0.4`.

**Do not** change the weighted 58.35% `PLANNING_NOT_CERTIFICATION` delivery score, global constitutional status, production kernel version, or claim formal correctness on the basis of this suite. Not a production engine; no new authority.
