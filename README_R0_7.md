# NORDOI R0.7 — Two-Scope Independent Conformance Expansion (CANDIDATE)

**Frozen certified baseline:** `r0.6` / `9c12bee933c13eff313aa11e49c02d56d86e91f3`.

R0.7 expands the **TEST-ONLY** independent specification oracle introduced in R0.6.
The new oracle is separately implemented in `tests/resource_task_r07.rs` and compares
its own predicted typed receipts/errors and normalized closure reports to the
frozen `src/resource_task_r02.rs` pure reference model. Oracle expectations do
**not** come from reading the model's internal state or using its results as predictions.

## Finite validation profile

- 18 dedicated Rust witnesses, individually mapped to 18 governance rows.
- 32 deterministic seeds × 128 attempted operations = **4,096 generated comparisons**;
  each generated trial starts with one child-creation event.
- Five-symbol alphabet, five events per word = **5^5 = 3,125 finite words**.
- All **6 admissible orders** of two task-completion/join pairs across root and child.
- Up to **two scopes**, one root and its one direct child; no grandchildren;
  **at most two tasks and two resources per scope** (four of each total).
- Read/Write declarations, scope-specific host grants, revocation, releases,
  cancellation, failures, joins, ordered closure reports and explicit error codes.
- A global **64-accepted-event** bound per generated trial; rejected operations
  leave both model and oracle unchanged and consume zero accepted events.
- Exact state snapshot equality on rejection and accepted causal replay checks.

## Non-claims / certified boundaries

**TEST-ONLY**, not a production runtime, scheduler, concurrency implementation,
formal proof, cryptographic capability implementation, host-I/O rollback system,
or full language/NAIR/IR conformance. Resource/task limits refer to this finite
profile only. The injected oracle mutation is **synthetic**, not a real defect.

No earlier tracked file must be changed. The candidate contains **six new files only**.
The source file `src/resource_task_r02.rs` is included by path in the dedicated
integration test but is not exported from the certified library. Historic tags
must not be changed. The previous `r0.2` SHA-256 manifest remains untouched.

## Mac install and release gate

```sh
cd "$HOME/Downloads"
unzip -o NORDOI_R0_7_MULTISCOPE_CONFORMANCE_CANDIDATE.zip
rsync -av NORDOI_R0_7_MULTISCOPE_CONFORMANCE_CANDIDATE/ \
  "/Users/noury/Documents/App_py/NORDOI/Github/nordoi/"
cd "/Users/noury/Documents/App_py/NORDOI/Github/nordoi"

python3 scripts/validate_r07_multiscope.py
python3 scripts/validate_r06_conformance.py
python3 scripts/validate_r05_hardening.py
python3 scripts/validate_r04_adversarial.py
python3 scripts/validate_r03_state_space.py
python3 scripts/validate_r02_manifest.py
rustfmt --edition 2021 tests/resource_task_r07.rs
cargo fmt --all -- --check
cargo test --test resource_task_r07 -- --nocapture
./scripts/release_gate.sh

git diff --exit-code r0.6 -- .
git status --short
```

**Do not commit or tag yet.** The authoring container has no Rust compiler,
`rustfmt` or Clippy. This candidate's Rust tests and complete Release Gate
**have not run here**. If any Mac test fails, correct only new R0.7 files;
re-run the complete local release gate, then request a clean exact-SHA CI run
before publishing any immutable `r0.7` tag.
