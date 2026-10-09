# NORDOI R0.2.1 — Integrity Remediation Decision

## Incident

In the final preparation of R0.2, `cargo fmt --all` reformatted the new Rust model
and test files. The SHA-256 manifest, produced **before formatting**, remained
unchanged. The user's `shasum -a 256 -c MANIFEST.sha256` output showed five
passing files and two failing entries (`src/resource_task_r02.rs` and
`tests/resource_task_r02.rs`). The exact R0.2 commit nevertheless passed all
five CI jobs because the CI workflow had no manifest integrity check.

## Decision

- Preserve tag `r0.2` and commit `4a7fadfa04e2f7ba1172d09a7fd6395883877fd4`.
- Publish a separate **r0.2.1** integrity-only follow-up after certification.
- Recompute only the two incorrect SHA-256 entries from byte-exact `r0.2` Rust
  files; fail closed if anything else differs.
- Make the check an automatic quality-gate step in GitHub CI.
- Keep `src/`, `tests/`, kernel, NAIR, runtime, effects, capabilities, public
  version and model semantics unchanged.
- Defer any R0.3 model exploration until the integrity-fix release is green.

## Evidence requirement

`python3 scripts/validate_r02_manifest.py` and
`shasum -a 256 -c MANIFEST.sha256` must pass locally **and** CI must pass for the
exact new commit. No claim of certified concurrency semantics follows from these
checks.
