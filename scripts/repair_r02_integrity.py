#!/usr/bin/env python3
"""One-time R0.2.1 metadata repair: rehash only the two rustfmt-changed files.

Preconditions are deliberately strict. Never modifies Rust, runtime, or test code.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from validate_r02_manifest import EXPECTED, entries, mismatches, sha256

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "4a7fadfa04e2f7ba1172d09a7fd6395883877fd4"
RUST_PATHS = {"src/resource_task_r02.rs", "tests/resource_task_r02.rs"}
WORKFLOW = ROOT / ".github/workflows/ci.yml"
NEEDLE = "      - name: Rustfmt\n        run: cargo fmt --all -- --check\n"
STEP = "      - name: R0.2 source manifest integrity\n        run: python3 scripts/validate_r02_manifest.py\n"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def main() -> int:
    try:
        if git("rev-parse", "HEAD") != BASELINE:
            raise ValueError("HEAD is not certified r0.2; refuse auto-repair")
        if git("rev-parse", "r0.2^{commit}") != BASELINE:
            raise ValueError("r0.2 tag doesn't match expected baseline")
        if git("status", "--porcelain", "--untracked-files=no"):
            raise ValueError("tracked files are dirty; first inspect/restore tracked changes")
        # Rust files must be exactly those tracked at the certified R0.2 commit.
        for name in RUST_PATHS:
            source_in_commit = subprocess.check_output(
                ["git", "show", f"r0.2:{name}"], cwd=ROOT
            )
            if (ROOT / name).read_bytes() != source_in_commit:
                raise ValueError(f"current Rust file differs from r0.2: {name}")
        subprocess.check_call(["cargo", "fmt", "--all", "--", "--check"], cwd=ROOT)
        current = mismatches(ROOT)
        if set(current) != RUST_PATHS:
            raise ValueError(f"expected exactly two known rustfmt mismatches, got {current}")
        records = entries(ROOT)
        if tuple(name for name, _ in records) != EXPECTED:
            raise ValueError("manifest contents unexpected")
        for name, digest in records:
            if name not in RUST_PATHS and sha256(ROOT / name) != digest:
                raise ValueError(f"non-Rust artifact altered: {name}")

        workflow_before = WORKFLOW.read_text(encoding="utf-8")
        if STEP in workflow_before or workflow_before.count(NEEDLE) != 1:
            raise ValueError("CI workflow unexpected or already patched; inspect manually")
        # Guard against a different remote workflow being overwritten.
        certified_workflow = subprocess.check_output(
            ["git", "show", "r0.2:.github/workflows/ci.yml"], cwd=ROOT
        ).decode("utf-8")
        if workflow_before != certified_workflow:
            raise ValueError("CI workflow differs from r0.2; inspect manually")

        patched_manifest = "".join(
            f"{sha256(ROOT / name) if name in RUST_PATHS else digest}  {name}\n"
            for name, digest in records
        )
        patched_workflow = workflow_before.replace(NEEDLE, NEEDLE + STEP, 1)
        with (ROOT / "MANIFEST.sha256").open("w", encoding="ascii", newline="\n") as out:
            out.write(patched_manifest)
        with WORKFLOW.open("w", encoding="utf-8", newline="\n") as out:
            out.write(patched_workflow)

        if mismatches(ROOT):
            raise ValueError("manifest still inconsistent after patch")
        if WORKFLOW.read_text(encoding="utf-8") != patched_workflow:
            raise ValueError("CI patch write verification failed")
        print("R0.2.1 manifest repair: PASS")
        print("Updated: MANIFEST.sha256 (two Rust SHA-256 entries only)")
        print("Updated: .github/workflows/ci.yml (new integrity gate only)")
        print("Preserved: src/, tests/, Cargo.toml, VERSION, semantics, certified r0.2 tag")
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError, UnicodeError) as exc:
        print(f"R0.2.1 manifest repair: FAIL ({exc})", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
