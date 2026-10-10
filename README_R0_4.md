# NORDOI R0.4 — Generative & Adversarial Verification (candidate)

Baseline is the **certified r0.3** tag at `cb9e54b16e7dfc91273976e689d18ae742ad6476`. This package has **six additive files only**. It is a **TEST-ONLY** extension and grants no authority to NORDOI programs. **Not certified** until release gate and GitHub CI complete.

Install on Mac after downloading ZIP:

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_4_ADVERSARIAL_CANDIDATE.zip
rsync -av NORDOI_R0_4_ADVERSARIAL_CANDIDATE/ "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

Validate without touching existing Rust sources:

```bash
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r04.rs
cargo fmt --all -- --check
cargo test --test resource_task_r04 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.3 -- .
git status --short
```

Expected only if actual toolchain passes: static PASS, **12/12 Rust test witnesses**, existing **7/7 SHA-256**, `NORDOI release gate: PASS`, and only six new files. Rustfmt operates **only on the new R0.4 test file**, so the historic manifest stays valid. All tests are bounded; synthetic counterexample shrinking does not demonstrate a real bug.

**Do not commit/tag** before the complete release gate, base-diff and GitHub CI checks. Historical tags are immutable; no reweighting of project delivery percentages.
