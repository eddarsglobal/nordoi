# NORDOI R0.11 — Production Boundary Readiness (candidate)

**Frozen certified baseline:** `r0.10` / `16de3cf71b4be39118eb2bb4b963c64ceeb5a65a`. **Status:** TEST-ONLY, NOT CERTIFIED. This package is research governance, not a production implementation or an authorization to change the K1.18 runtime, compiler, NAIR or host authority.

R0.11 introduces exactly **seven additive files** and no edits to the historical source, tests, manifest or release tooling. It defines **12 explicitly UNMET native readiness gates** and **20 research policy tests**. The 117 historic R0.3–R0.9 witness declarations and the 18 R0.10 tests remain frozen; passing them never implies native production readiness.

## Install

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_11_PRODUCTION_BOUNDARY_READINESS_CANDIDATE.zip
rsync -av NORDOI_R0_11_PRODUCTION_BOUNDARY_READINESS_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validate before committing

```bash
python3 scripts/validate_r11_readiness.py
python3 scripts/validate_r10_consolidation.py
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r11.rs
cargo fmt --all -- --check
cargo test --test resource_task_r11 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.10 -- .
git status --short
```

Expected, not pre-certified: `R0.11 static contract: PASS`, `R0.10 Git tree boundary: PASS`, 20 Rust tests PASS, `NORDOI release gate: PASS`. The seven new paths are the only expected untracked changes. `git diff` alone does not detect untracked files; the validator also scans them.

Do **not** commit/tag until all local checks pass. Commit, push, validate all **five** CI jobs on the precise commit SHA, then and only then create annotated tag `r0.11`. Even a green CI will never grant production authority: the twelve gates remain UNMET.
