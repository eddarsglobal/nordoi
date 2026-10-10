# NORDOI R0.14 — Evidence Authentication & Revocation Governance (CANDIDATE)

**TEST-ONLY** · **NOT CERTIFIED** · frozen certified base `r0.13` `35e048dd83f2c072a6c454494ac099906156dc38`.

Seven additive files only, no kernel/NAIR/compiler/manifest/CI modifications. **44 direct Rust tests plus 2 imported unchanged SHA-256 FIPS tests = 46 expected tests**. Rust and Release Gate are **NOT RUN** by packaging; user Mac must certify.

Research-only: 12/12 native requirements **UNMET**; 12/12 historical evidence **ABSENT**; 12/12 integrity rows **ABSENT**; 12/12 R0.14 authentication rows **ABSENT**. Synthetic digests, producer/reviewer names, root, and revocations are PUBLIC deterministic test fixtures, **NOT** real authentication, cryptographic signatures, or independent attestation. **RESEARCH_REVIEW_ONLY** is the maximum mock decision, never native authorization.

## Install on Mac
```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_14_AUTHENTICATION_REVOCATION_CANDIDATE.zip
rsync -av NORDOI_R0_14_AUTHENTICATION_REVOCATION_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validate (no commit/tag yet)
```bash
python3 scripts/validate_r14_authentication.py
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r14.rs
cargo fmt --all -- --check
cargo test --test resource_task_r14 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.13 -- .
git status --short
```

Expect **46 Rust tests**, full Release Gate PASS, and exactly seven new untracked R0.14 files. Historical validators R0.10–R0.13 should be run only in isolated detached worktrees at their certified tags when re-audited: they intentionally reject files of later versions. DO NOT edit previous validators.

Do not commit or tag `r0.14` until macOS gate PASS and GitHub Actions 5/5 on the exact commit SHA.
