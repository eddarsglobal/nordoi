# NORDOI G0.3 R4W FIX2 — Missing GitHub Actions job forensic reconciliation

**Purpose:** Re-execute the two *absent* GitHub Actions jobs without touching any owner-validated R4W research source, rewriting the historic CI results or falsely claiming independent administration.

## Frozen accepted research and incident evidence

- Owner Mac Intel 14.5, Rust 1.98.1: **R4W FIX1 = 238 regular PASS, 0 fail, 3 intentionally ignored, 23/23 source Git blob pins, Rustfmt/Clippy strict PASS, SCRIPT_EXIT=0**, exact code commit **`1637e220553843e1eb6435c869e7fe2418ee6196`**. Original Mac owner worktree untouched and PR #87 DRAFT_UNMERGED.
- [Dedicated R4W run 42672817980](https://github.com/eddarsglobal/nordoi/actions/runs/42672817980) at the same code commit: GitHub Actions reports **`conclusion=failure`**, yet its full REST API `/jobs?per_page=100&filter=all` result has **`total_count=3`** and ALL three are SUCCESS: native macOS 15 Apple Silicon R4W **14/14 PASS** and previous R4V **13/13 PASS**, Windows portable and Ubuntu portable. The fourth expected **`native-macos-custodian-enrollment-denial (macos-15-intel)` job is absent from the GitHub job list entirely**.
- [General Test Gate run 42672817934](https://github.com/eddarsglobal/nordoi/actions/runs/42672817934): GitHub Actions reports **`conclusion=failure`**, yet its full `/jobs?per_page=100&filter=all` result has **`total_count=4`** and ALL four are SUCCESS: Format & Clippy, Ubuntu, macOS and Windows. The fifth expected **`Check` job** is absent entirely.
- REST `/commits/1637.../check-runs?per_page=100` reports **seven check runs, all `success`**; the GitHub Actions check suites for these two workflows nevertheless report **`failure`**. Neither the job-list pagination nor an actually failing known job explains the discrepancy.
- `.github/workflows/ci.yml` is **byte-for-byte identical** to the parent R4V commit where all five jobs passed. The dedicated workflow is also structurally equivalent to the previous successful 2x2 matrix R4V experiment. **Workflow source regression is not established.**
- GitHub denied the previous failed-workflow retry with **`HTTP 403: This workflow run cannot be retried`**. No failed job ID exists to rerun individually.
- **Root cause unknown.** The available evidence does not distinguish GitHub orchestration, entitlement/billing, transient runner provisioning, capacity or other platform events. Do not fabricate a definite diagnosis.

## FIX2 implementation — minimal additive, not a rewriting trick

A new **independent draft stacked PR** is branched from the exact owner-approved R4W commit. It adds this report and one diagnostic workflow, leaving the R4W PR #87 and validated commit untouched.

The new workflow `.github/workflows/g03_r4w_fix2_missing_ci_replay.yml` defines EXACTLY the missing two controls as distinct non-matrix jobs:

1. **FIX2 dedicated Intel source-pinned R4W replay** on `macos-15-intel`: test the original R4W and R4V Git blob identities; execute real SSHSIG R4W **14 regular** tests; prior R4V **13 regular** tests; strict Rustfmt; strict `cargo clippy --offline --all-targets -- -D warnings`.
2. **FIX2 general Linux cargo Check source-pinned replay** on `ubuntu-latest`: confirm original `ci.yml` and original R4W Git blobs; execute **exact `cargo check --all-targets`**, using the same toolchain setup as the existing Test Gate.

The file SHA-1 Git blob identity pins **must equal original R4W**:
- R4W test: `e885d0768f87f12ec4604e062eccb8abd3168053`
- R4V test: `46b2934d795eb4cc56a6cb883c2ec2da14e22706`
- existing general CI file `.github/workflows/ci.yml`: `1eae37c4860bf47f16b7faa6b5616d33b3c455d2`

No change to Rust research files, public `.noi`, compiler/kernel/runtime, original workflows, `main`, original Mac owner worktree, or PR #87. No merge. The diagnostic workflow is only an **additional CI witness for the same content**, NOT a replacement for the historical workflow conclusions.

## Strict closure acceptance

- New FIX2 PR must stay DRAFT_UNMERGED with exactly the two new nonproduction documentation/CI files.
- Dedicated FIX2 workflow **2/2 SUCCESS**, including a *real* Intel GitHub runner and real Linux cargo Check, with original source pins PASS.
- New commit's **unchanged** NORDOI general Test Gate must be **5/5 SUCCESS** to establish full workflow confidence at the FIX2 GitHub commit; if a job again fails to start, record it as unresolved.
- Verify REST `jobs.total_count`, check-runs, workflow run `conclusion`, head commit, and source pins; do not certify solely from a subset of successful jobs.
- **Historical R4W workflows remain FAILURE and are not rewritten or counted as 4/4/5/5.**
- There is no need to repeat the owner Mac 238 tests for an additive research workflow-only change, but no claim that a new owner-run was performed.

The R4W science remains **PASS_WITH_CONTROLS_RESEARCH_ONLY** on the owner's Mac. No true unrelated independent guardian, out-of-band approved signing root or externally monotonic log has been enrolled. All **12 native admission gates remain UNMET**; **PRODUCTION NOT_AUTHORIZED**.

```ini
G03_R4W_OWNER_MAC_INTEL=238_PASS_0_FAIL_3_IGNORED
R4W_HISTORIC_DEDICATED=FAILURE_MISSING_INTEL_JOB
R4W_HISTORIC_GENERAL=FAILURE_MISSING_CHECK_JOB
R4W_FIX2_MISSION=TARGETED_REPLAY_ONLY
FIX2_SOURCE_CHANGES=NONE
FIX2_HISTORIC_STATUS_REWRITE=FORBIDDEN
REAL_INDEPENDENT_ADMIN=NOT_ESTABLISHED
GLOBAL_ANTIROLLBACK=NOT_IMPLEMENTED
PR87=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```