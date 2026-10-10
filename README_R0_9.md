# NORDOI R0.9 — Sibling-Branch Boundary Conformance (Candidate)

**Frozen certified baseline:** `r0.8` / `35fc2487ce180ff1fdc1a0f2fa8df3213127865d`.

This is an **additive-only, TEST-ONLY** experiment using the frozen R0.2 reference implementation as a system under test. It introduces **no** scheduler, runtime execution, kernel export, NAIR opcode, compiler change, new dependency or host authority. The independent test oracle is adapted from the R0.8 oracle to a finite branching fixture, not imported from the SUT's state or receipts.

## New boundary tested

```
root (scope 0; child budget 2)
├─ sibling A (scope 1; child budget 1)
│  └─ leaf A (scope 3; child budget 0)
└─ sibling B (scope 2; child budget 1)
   └─ leaf B (scope 4; child budget 0)
```

Each scope is limited to **two cumulative tasks and two cumulative resources**. Rejected operations must preserve both independent oracle and SUT state; accepted events count against one global budget. The oracle predicts exact receipts, error classes, scoped grant/resource behavior, task outcomes, and canonical reports, before applying a command to the SUT. All tests remain sequential.

- **23 named Rust tests ↔ 23 governance witnesses**, including separate negative boundary cases.
- **40 fixed seeds × 160 attempted operations = 6,400 attempted comparisons**, after establishing a five-scope fixture.
- **7^5 = 16,807 five-operation words**, exhaustive only over the chosen seven-action alphabet.
- **2,520 admissible finish/join orders of four tasks** (8! / 2^4), all expected to yield the same canonical closure reports.
- Negative cases cover siblings/cousins, non-ambient grants, cross-scope handle misuse, revoked grants, quota exhaustion, closure blockers, failure and cancellation precedence.

## Install (six new files, without modifying certified history)

```bash
cd "$HOME/Downloads"
unzip -o NORDOI_R0_9_BRANCH_BOUNDARY_CANDIDATE.zip
rsync -av NORDOI_R0_9_BRANCH_BOUNDARY_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"
```

## Run local checks before any commit

```bash
python3 scripts/validate_r09_branches.py
python3 scripts/validate_r08_hierarchy.py
python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r09.rs
cargo fmt --all -- --check
cargo test --test resource_task_r09 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.8 -- .
git status --short
```

**Do not commit or tag `r0.9` until all Rust tests and local release gates PASS.** Commit exactly the six new files, then verify all five CI jobs have conclusion `success` on the *exact* commit SHA before creating an annotated `r0.9` tag.

The static Python validator **does not** compile or run Rust; finite differential tests are **not** a formal proof of runtime security. No production promotion is authorized by this candidate.
