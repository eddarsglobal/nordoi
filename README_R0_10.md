# NORDOI R0.10 — Verification Consolidation (candidate)

**Frozen certified baseline:** `r0.9` / `860d09774c881cf23f2a1233877109ed49c21a01`. **Status:** TEST-ONLY. This is **not** a runtime/concurrency/security certification or an authority to alter K1.18, NAIR or the compiler.

Only six new files (docs, governance, research, script, test, README). No modifications of the frozen source, old tests, Cargo dependencies, kernel, compiler, runtime or NAIR.

R0.10 does not inflate cross-scope behavior counts: it audits **117 historical witnesses** from R0.3 through R0.9, and adds **18 R0.10 tests**. It checks historical matrix-to-test traceability, limitations, finite coverage arithmetic, independent-oracle isolation indicators, deterministic material fingerprinting and three narrow R0.2 smoke regressions.

## Install

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_10_VERIFICATION_CONSOLIDATION_CANDIDATE.zip
rsync -av NORDOI_R0_10_VERIFICATION_CONSOLIDATION_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validate (before commit)

```bash
python3 scripts/validate_r10_consolidation.py
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r10.rs
cargo fmt --all -- --check
cargo test --test resource_task_r10 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.9 -- .
git status --short
```

**Expected (not performed in the candidate workspace):** R0.10 static contract PASS; local Git boundary PASS on a full repository; 18 tests passed; `NORDOI release gate: PASS`. The six paths must be the only changes. The Git diff check shown before the commit is empty because the candidate files remain untracked.

Do **not** commit or tag until local checks pass. After commit, check exact commit SHA and five CI jobs, then and only then annotate and push `r0.10`. Finite witnesses **do not** constitute formal verification or proof of production security.
