# NORDOI R0.3 — Bounded State-Space Verification (candidate)

Baseline: **r0.2.1** / `d9b7153cac7d044cefdc57645758c331fd75546d`.

This ZIP is **additive only** and contains exactly six new files. No copies of G0.1/R0.1/R0.2/R0.2.1 source files. This avoids overwriting Rustfmt-certified files and changing the existing seven-entry `MANIFEST.sha256`.

## Install (from Downloads)

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_3_STATE_SPACE_CANDIDATE.zip
rsync -av NORDOI_R0_3_STATE_SPACE_CANDIDATE/ "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Validation

```bash
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r01_foundations.py
python3 scripts/validate_r02_reference.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r03.rs
cargo fmt --all -- --check
cargo test --test resource_task_r03 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.2.1 -- .
git status --short
```

**Do not tag or commit before all tests succeed.** The R0.3 Rust tests are not executed in the package-generation environment. Expected dedicated suite: **9 tests**; this is a test declaration, not a certified outcome.

The `rustfmt` command above formats **only the new R0.3 test file**. Do not reformat or replace any R0.2 certified source. No R0.3 file is added to the seven-entry R0.2.1 source manifest; Git commits and CI provide version tracking for the new tests.

The explorer is test-only: it does not run concurrent work, grant program authority, integrate into NAIR, or prove unbounded safety.
