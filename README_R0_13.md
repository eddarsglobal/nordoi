# NORDOI R0.13 — Evidence Integrity & Trust Chain (CANDIDATE)

**Certified frozen baseline:** `r0.12` / `851d978c36f4c12fc16c115e89a7774e33ce361b`. **TEST-ONLY; NOT CERTIFIED.**

This is an additive research-only integrity gate. There are **12/12 R0.11 native gates `UNMET`, 12/12 R0.12 evidence rows `ABSENT`, and 12/12 R0.13 integrity records `ABSENT`**. Nothing in this package is an authenticated producer identity, signed attestation, completed independent review, scientific proof, or native deployment authorization.

**Seven additive files only**, including **34 top-level Rust tests** plus **two inherited FIPS SHA-256 vector tests** from the unchanged frozen `src/effect_audit/hash.rs` (therefore `cargo test --test resource_task_r13` should report **36 tests**). An immutable frozen R0.2 model smoke test is retained. No changes to the kernel, NAIR, compiler, Cargo.toml, CI, or prior tags.

## Installation on Mac
```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_13_EVIDENCE_INTEGRITY_CHAIN_CANDIDATE.zip
rsync -av NORDOI_R0_13_EVIDENCE_INTEGRITY_CHAIN_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validation (do not commit/tag yet)
```bash
python3 scripts/validate_r13_integrity.py
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r13.rs
cargo fmt --all -- --check
cargo test --test resource_task_r13 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.12 -- .
git status --short
```

The frozen **R0.10, R0.11, and R0.12 validators** can refuse future untracked versions by design; check each in a detached temporary git worktree at its own certified tag if historical re-validation is desired. Never edit historical validators.

**Expected but NOT confirmed here:** `R0.13 static contract: PASS`, `36 passed; 0 failed` (34 R0.13 and 2 imported certified SHA-256 vector tests), and `NORDOI release gate: PASS` on the Mac. Require CI 5/5 on exact commit SHA before publishing annotated `r0.13` tag. **SHA-256 consistency is not a digital signature or source authentication.**
