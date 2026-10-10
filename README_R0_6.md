# NORDOI R0.6 — Independent Resource/Task Model Conformance (CANDIDATE)

**Frozen certified input:** `r0.5` at `6cebd4051304c81901669303265c0ad7456b7e12`.

## Purpose

A separate finite-state **specification oracle** predicts acceptance, typed receipts,
error codes, event-budget accounting, and closed-scope outcomes for a selected
R0.2 resource/task command slice. The system under test is the frozen,
**TEST-ONLY** `src/resource_task_r02.rs` module. The oracle is not derived by
cloning the model or reading its state. A second run with the same script
is deterministic; mutations in a synthetic expected record are detected.

## Reproducible finite evidence

- 15 dedicated Rust tests with 1:1 governance witness mappings.
- 32 fixed xorshift64 seeds × 96 attempted steps = 3,072 oracle comparisons.
- 5-symbol alphabet, 5 events per word = 5^5 = 3,125 exhaustively enumerated **finite words**.
- Single root scope, no children, at most **one resource and one task**;
  multiple grants are allowed, and failed commands do not consume event budget.
- Effects: modeled `Read` plus deliberately rejected `ExternalIo`; no effects occur.
- Deterministic rejection atomicity checks (including exact model snapshots)
  and accepted-event replay checks are extra consistency witnesses.

**Critical distinction:** expected errors and *synthetic oracle mutation* are not
vulnerabilities discovered in the reference model. Bounded differential testing
is not formal verification, exhaustive global-state proof, scheduler validation,
cryptographic capability enforcement, or proof of production security.

No source, runtime, compiler, NAIR, Cargo config, CI config, or old manifest
file is modified. This package contains **six new files only**. Do not rename,
recreate, move, or retag any certified version.

## Mac release gate

```sh
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r06.rs
cargo fmt --all -- --check
cargo test --test resource_task_r06 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.5 -- .
git status --short
```

The dedicated Rust tests, Rust compiler, Clippy, full release gate and GitHub CI
have **NOT** been run by the package author in this environment. The candidate
must pass the checks on the user's Rust toolchain before any commit/tag.
