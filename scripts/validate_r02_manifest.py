#!/usr/bin/env python3
"""Fail-closed R0.2 tracked-source integrity gate; Python standard library only.

This gate checks exact SHA-256 digests for seven R0.2 artifacts. It does not
certify concurrency or runtime semantics, and never rewrites any files.
"""
from __future__ import annotations

import hashlib
import hmac
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "MANIFEST.sha256"
EXPECTED = (
    "README_R0_2.md",
    "docs/NORDOI_R0_2_DETERMINISTIC_RESOURCE_TASK_REFERENCE_SPEC.md",
    "governance/r02_reference_obligations_v1.tsv",
    "research/R0_2_REFERENCE_MODEL_DECISION.md",
    "scripts/validate_r02_reference.py",
    "src/resource_task_r02.rs",
    "tests/resource_task_r02.rs",
)
LINE = re.compile(r"([0-9a-f]{64})  ([a-zA-Z0-9_./-]+)")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def entries(root: Path) -> list[tuple[str, str]]:
    data = (root / "MANIFEST.sha256").read_bytes()
    if not data.endswith(b"\n") or b"\r" in data:
        raise ValueError("manifest must have LF line endings and a trailing newline")
    lines = data.decode("ascii").splitlines()
    if len(lines) != len(EXPECTED):
        raise ValueError(f"expected {len(EXPECTED)} manifest lines, got {len(lines)}")
    parsed: list[tuple[str, str]] = []
    for i, line in enumerate(lines, 1):
        match = LINE.fullmatch(line)
        if match is None:
            raise ValueError(f"invalid manifest syntax on line {i}")
        digest, name = match.groups()
        if name != EXPECTED[i - 1]:
            raise ValueError(f"unexpected/duplicate/out-of-order filename on line {i}: {name}")
        parsed.append((name, digest))
    return parsed


def mismatches(root: Path) -> list[str]:
    bad: list[str] = []
    for name, digest in entries(root):
        target = root / name
        if target.is_symlink() or not target.is_file():
            bad.append(name)
        elif not hmac.compare_digest(sha256(target), digest):
            bad.append(name)
    return bad


def main() -> int:
    try:
        bad = mismatches(ROOT)
    except (ValueError, OSError, UnicodeError) as exc:
        print(f"R0.2 manifest integrity: FAIL ({exc})", file=sys.stderr)
        return 1
    if bad:
        print("R0.2 manifest integrity: FAIL", file=sys.stderr)
        for name in bad:
            print(f"  MISMATCH: {name}", file=sys.stderr)
        return 1
    print(f"R0.2 manifest integrity: PASS ({len(EXPECTED)}/{len(EXPECTED)} artifacts)")
    print("scope: source/package integrity only; no production concurrency claim")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
