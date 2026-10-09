# NORDOI R0.2.1 — Source Manifest Integrity Correction

**Status: CANDIDATE (NOT YET CERTIFIED).** The R0.2 Rust model passed CI on commit
`4a7fadfa04e2f7ba1172d09a7fd6395883877fd4`, tagged `r0.2`, with GitHub CI
`37968923100`. However, the package MANIFEST.sha256 was created before `cargo fmt`
and its two Rust file digests do not match the formatted source committed at r0.2.

This correction repairs **metadata only**. It adds an integrity gate to the existing
Ubuntu quality job. It neither revises model semantics nor retroactively edits the
published `r0.2` tag (which remains an immutable record of the known defect).

The ZIP is **strictly additive** and contains only:

- `scripts/validate_r02_manifest.py`
- `scripts/repair_r02_integrity.py`
- `research/R0_2_1_MANIFEST_INTEGRITY_DECISION.md`
- `README_R0_2_1.md`

The repair script runs **only** at the exact `r0.2` commit with clean tracked files,
verifies the tag, Rustfmt, committed Rust bytes, and all five unchanged artifact
hashes, and will refuse any unexpected mismatch. It then updates the two digests
in the already-tracked `MANIFEST.sha256` and surgically adds the validator to
`.github/workflows/ci.yml`. No manual hash copying is needed.

After copying the additive ZIP into the repo, run:

```bash
python3 scripts/repair_r02_integrity.py
python3 scripts/validate_r02_manifest.py
shasum -a 256 -c MANIFEST.sha256
python3 scripts/validate_r01_foundations.py
python3 scripts/validate_r02_reference.py
./scripts/release_gate.sh
git diff --check
git diff --name-only
git status --short
```

The ONLY tracked edits expected are `.github/workflows/ci.yml` and
`MANIFEST.sha256`. All other files in this package should appear untracked. In
particular, do not change `src/resource_task_r02.rs` or `tests/resource_task_r02.rs`.

Commit/CI/tag instructions follow only after local green checks. Name the next tag
`r0.2.1` rather than moving the existing `r0.2` tag.

**Scope of the guarantee:** SHA-256 content integrity of seven R0.2 artifacts at a
given revision. This does **not** imply package authenticity, cryptographic release
signature, production runtime safety or universal model correctness.
