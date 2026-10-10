# NORDOI R0.5 — Resource & Task Verification Hardening (candidate)

**Frozen input:** certified `r0.4` at commit `3ce87f1943c8c97ee304cebad9eff236ae646fa6`.
**Scope:** six **additive-only** files. Reference model `src/resource_task_r02.rs` and the entire certified kernel/compiler/NAIR remain untouched. R0.5 is **TEST-ONLY**, not a scheduler, runtime, thread engine, formal verification, universal security proof, or source-language feature.

## New evidence

- **14 Rust test witnesses** and one bijective TSV traceability matrix.
- **90 admissible event interleavings**, computed by enumerating terminal/join actions for 3 tasks subject only to each task's terminal-before-join constraint: `6! / 2^3 = 90`. Each path checks the final sorted report and the model's accepted-log replay; these 90 are exhaustive **only inside this narrow finite profile**, not across NORDOI concurrency.
- Two three-level/sibling scope close orders with explicit failure-over-cancellation precedence.
- Atomic rejections of cross-scope grants, foreign-domain handles, revoked grants, released resources, exhausted budgets and illegal lifecycle edges.
- A synthetic report mutation and a shrinker of a **known expected rejection** for testing the checking mechanism. Neither is evidence of a real defect.
- In-process deterministic trace comparison; no stable cross-compiler binary report or cryptographic attestations claimed.

## Install (macOS)

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_5_HARDENING_CANDIDATE.zip
rsync -av NORDOI_R0_5_HARDENING_CANDIDATE/ "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Release gate (run locally before any commit)

```bash
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r05.rs
cargo fmt --all -- --check
cargo test --test resource_task_r05 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.4 -- .
git status --short
```

**Expected only after actual local execution:** R0.5 static `PASS`, matrix `14/14`, Rust tests `14 passed`, R0.2 manifest `7/7`, `NORDOI release gate: PASS`, and precisely six untracked R0.5 files, with no tracked files changed. Note that the static validator **does not execute Rust** and **does not count states**. `rustfmt` runs only on the new file, so it does not invalidate the historic R0.2 manifest.

## Governance / release

Do **not** commit, push, or tag until all gates pass. Then stage only the six listed files; obtain GitHub CI's 5/5 success **for the exact candidate commit** before tagging `r0.5`. Existing tags are immutable and the G0.1 322-principle baseline / 58.35% planning figure remain unchanged. No claim of actual production-task execution.
