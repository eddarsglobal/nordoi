# NORDOI R0.8 — Three-Generation Independent Resource/Task Conformance (Candidate)

## Certified input, scope and authority

Base: **`r0.7` / `8452ce32a27ea93c4eb343dec7165ae4b6be704a`**; the R0.7 implementation and all historical certified files are immutable. R0.8 is an **additive-only, TEST-ONLY** experiment. It does **not** export a native scheduler, executable task semantics, kernel capability, compiler extension, or NAIR opcode.

This candidate creates a **separately specified bounded oracle** for a three-generation chain: root -> child -> grandchild. Each scope admits at most two tasks, two resources and one child, with no great-grandchild in this test configuration. Resource and task IDs are globally monotone within the model, grants are scope-exact, a rejected transition leaves the model unchanged, and closing a scope requires its children to close first.

## What changes vs. R0.7

- The three-generation independent oracle predicts receipts, exact `ModelError` classes, hierarchical `ScopeOutcome` aggregation and canonical per-scope reports without reading state/receipts from the R0.2 model.
- **26 Rust tests** map 1:1 to the R0.8 governance TSV; prior bounded two-scope witnesses are retained as regression tests.
- Fixed generated differential profile: **32 seeds x 160 attempts = 5,120 attempted comparisons**.
- Finite exact-word profile: **6^5 = 7,776 words** over one deliberately restricted six-action alphabet.
- **90 linear extensions** of three independent finish-before-join pairs, one task per scope generation; the earlier six-order two-task profile remains a regression.
- Checks cover non-ambient ancestor grants, cross-generation resource misuse, failure/cancellation propagation, leaf-first close, global event budget and atomic rejection.

## Installation (new files only)

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_8_HIERARCHICAL_CONFORMANCE_CANDIDATE.zip
rsync -av NORDOI_R0_8_HIERARCHICAL_CONFORMANCE_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Local validation (candidate, not certification)

```bash
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r08.rs
cargo fmt --all -- --check
cargo test --test resource_task_r08 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.7 -- .
git status --short
```

**Do not commit or tag** until the 26/26 Rust tests, Clippy, release gate and unchanged-history checks succeed. Then commit exactly these six new files. Certify the exact commit only after the five GitHub Actions jobs are green, then tag `r0.8`. The static validator by itself does **not** run Rust code, measure behavioral coverage, provide a security proof or certify CI.
