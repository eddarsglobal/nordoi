# NORDOI R0.12 — Evidence Admission Boundary (candidate)

**Frozen certified baseline:** `r0.11` / `d61487bd2392890b3aaaa6ef33fce4371cfb6850`. **Status:** TEST-ONLY, NOT CERTIFIED. The new ledger contains 12 `ABSENT` proposals bound to the twelve `UNMET` native readiness gates from R0.11. It does not contain verified artifacts or grant host/runtime authority.

Only **seven additive files** (including 29 test-only Rust tests); no edit of the kernel, compiler, NAIR, CI, Cargo manifest or frozen prior tags. Even a green CI will NOT authorize production.

## Install on Mac
```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_12_EVIDENCE_ADMISSION_CANDIDATE.zip
rsync -av NORDOI_R0_12_EVIDENCE_ADMISSION_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validation (must be run before commit)
```bash
python3 scripts/validate_r12_evidence.py
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r12.rs
cargo fmt --all -- --check
cargo test --test resource_task_r12 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.11 -- .
git status --short
```

Historical `validate_r10_consolidation.py` and `validate_r11_readiness.py` were designed to accept only the files of *their own* release and will reject future untracked R0.12 files. Check them at their original tags via a temporary detached Git worktree rather than modifying any certified validator. The R0.12 validator verifies compatibility with the frozen R0.11 ledger and Git base.

**Expected, not pre-certified:** `R0.12 static contract: PASS`, 29/29 tests PASS, `NORDOI release gate: PASS`. Confirm seven new paths only. Commit and GitHub `5/5` on exact SHA before tagging `r0.12`.
