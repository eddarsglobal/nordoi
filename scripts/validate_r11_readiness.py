#!/usr/bin/env python3
"""R0.11 TEST-ONLY readiness audit; never issues production authorization."""
from __future__ import annotations

import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE_SHA = "16de3cf71b4be39118eb2bb4b963c64ceeb5a65a"
NEW = {
    "README_R0_11.md",
    "docs/NORDOI_R0_11_PRODUCTION_BOUNDARY_READINESS_SPEC.md",
    "governance/r11_readiness_witnesses_v1.tsv",
    "governance/r11_native_readiness_gates_v1.tsv",
    "research/R0_11_PRODUCTION_BOUNDARY_DECISION.md",
    "scripts/validate_r11_readiness.py",
    "tests/resource_task_r11.rs",
}
AREAS = (
    "LIFECYCLE_SEMANTICS", "EXECUTOR_SCHEDULING", "MEMORY_OWNERSHIP",
    "CAPABILITY_AUTHORITY", "CANCELLATION_CLEANUP", "EFFECT_IO_BOUNDARY",
    "CRASH_RECOVERY", "ADVERSARIAL_TESTING", "CROSS_PLATFORM",
    "COMPILER_NAIR_BINDING", "RESOURCE_LIMITS", "INDEPENDENT_REVIEW",
)


def read_rows(path: Path, fields: tuple[str, ...]) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or []) != fields:
            raise RuntimeError(f"R0.11 invalid TSV header: {path.name}")
        rows = list(reader)
    if any(set(row) != set(fields) or any(value is None for value in row.values()) for row in rows):
        raise RuntimeError(f"R0.11 malformed TSV fields: {path.name}")
    return rows


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, text=True, capture_output=True, check=False)


def static_check() -> None:
    for rel in NEW:
        f = ROOT / rel
        if not f.is_file() or f.is_symlink():
            raise RuntimeError(f"R0.11 required additive file missing or symlinked: {rel}")

    tests = (ROOT / "tests/resource_task_r11.rs").read_text(encoding="utf-8")
    names = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", tests)
    witnesses = read_rows(
        ROOT / "governance/r11_readiness_witnesses_v1.tsv",
        ("id", "witness", "classification", "limitation"),
    )
    if len(names) != 20 or len(set(names)) != 20 or len(witnesses) != 20:
        raise RuntimeError("R0.11 requires exactly twenty distinct tests and witnesses")
    if names != [entry["witness"] for entry in witnesses]:
        raise RuntimeError("R0.11 test order is not mapped bijectively to governance witnesses")
    allowed_classifications = {
        "STATIC_BOUNDARY", "GATE_REGISTRY", "BLOCKED_DEFAULT", "EVIDENCE_SEPARATION",
        "NEGATIVE_POLICY", "SYNTHETIC_POLICY", "DETERMINISM_EXAMPLE",
        "DIRECT_REFERENCE_SMOKE", "ISOLATION_INDICATOR",
    }
    for i, row in enumerate(witnesses, 1):
        if row["id"] != f"R11-P{i:02}" or row["classification"] not in allowed_classifications:
            raise RuntimeError(f"R0.11 witness identity/classification invalid: {i}")
        if len(row["limitation"].strip()) < 65:
            raise RuntimeError(f"R0.11 witness limitation too short: {i}")

    gates = read_rows(
        ROOT / "governance/r11_native_readiness_gates_v1.tsv",
        ("id", "area", "status", "evidence_scope", "acceptance_criterion", "limitation"),
    )
    if len(gates) != 12:
        raise RuntimeError("R0.11 requires exactly twelve native readiness gates")
    for i, row in enumerate(gates, 1):
        if row["id"] != f"R11-G{i:02}" or row["area"] != AREAS[i - 1]:
            raise RuntimeError(f"R0.11 gate identity/order mismatch: {i}")
        if row["status"] != "UNMET" or row["evidence_scope"] != "RESEARCH_ONLY":
            raise RuntimeError(f"R0.11 gate {i} cannot claim native verification")
        if len(row["acceptance_criterion"].strip()) < 30 or len(row["limitation"].strip()) < 30:
            raise RuntimeError(f"R0.11 gate {i} is underspecified")

    historical = ROOT / "governance/r10_consolidation_witnesses_v1.tsv"
    historical_rust = ROOT / "tests/resource_task_r10.rs"
    if historical.is_file() and historical_rust.is_file():
        previous = read_rows(historical, ("id", "witness", "classification", "limitation"))
        prev_names = re.findall(
            r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)",
            historical_rust.read_text(encoding="utf-8"),
        )
        if len(previous) != 18 or prev_names != [row["witness"] for row in previous]:
            raise RuntimeError("R0.10 certified witness declarations are inconsistent")
    else:
        raise RuntimeError("R0.11 requires the full certified R0.10 project baseline")

    docs = "\n".join((ROOT / f).read_text(encoding="utf-8") for f in NEW if f.endswith(".md"))
    for required in ("r0.10", BASE_SHA, "TEST-ONLY", "117", "18", "12", "20", "UNMET", "CI", "not"):
        if required not in docs:
            raise RuntimeError(f"R0.11 documentation omits required context: {required}")
    for required in (
        '#[path = "../src/resource_task_r02.rs"]',
        'include_str!("../governance/r11_native_readiness_gates_v1.tsv")',
        "enum Advisory", "ReviewEligible", "fn assess(", "fn parse_registry(",
        "R11_WITNESS version=1", "replay_checked()",
    ):
        if required not in tests:
            raise RuntimeError(f"R0.11 test-only evidence missing: {required}")
    for forbidden in (
        "std::thread::spawn(", "tokio::spawn(", "std::process::Command::new(",
        "unsafe {", "#[ignore]", "#[should_panic]", "ProductionAuthorized",
    ):
        if forbidden in tests:
            raise RuntimeError(f"R0.11 test source contains forbidden runtime/authorization pattern: {forbidden}")
    for prod in ("src/lib.rs", "Cargo.toml"):
        path = ROOT / prod
        if not path.is_file() or "resource_task_r11" in path.read_text(encoding="utf-8"):
            raise RuntimeError(f"R0.11 absent or leaked into production: {prod}")
    print("R0.11 static contract: PASS")
    print(f"Certified input baseline: r0.10 / {BASE_SHA}")
    print("Readiness gates: 12/12 UNMET; evidence scope: RESEARCH_ONLY")
    print("R0.11 governance witnesses: 20/20; R0.10 consolidation: 18/18 preserved")
    print("Research status: TEST-ONLY; a passing audit is NOT a native deployment authorization")


def git_boundary() -> None:
    root = git("rev-parse", "--show-toplevel")
    if root.returncode != 0 or Path(root.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.10 Git tree boundary: NOT CHECKED (outside a complete Git repository)")
        return
    baseline = git("rev-parse", "r0.10^{commit}")
    if baseline.returncode != 0 or baseline.stdout.strip() != BASE_SHA:
        raise RuntimeError("R0.10 certified tag missing or targets unexpected SHA")
    diff = git("diff", "--name-only", "r0.10", "--", ".")
    if diff.returncode != 0:
        raise RuntimeError(f"Cannot inspect historical Git diff: {diff.stderr}")
    if set(diff.stdout.splitlines()) - NEW:
        raise RuntimeError("Frozen source modifications detected since certified r0.10")
    tree = git("ls-tree", "-r", "--name-only", "r0.10")
    if tree.returncode != 0:
        raise RuntimeError("Unable to inspect certified Git tree")
    frozen_paths = set(tree.stdout.splitlines())
    if NEW & frozen_paths:
        raise RuntimeError("R0.11 new path collides with a certified historical file")
    for rel in frozen_paths:
        if not (ROOT / rel).is_file():
            raise RuntimeError(f"Certified file missing from working tree: {rel}")
    other = git("ls-files", "--others", "--exclude-standard")
    if other.returncode != 0:
        raise RuntimeError("Unable to enumerate untracked files")
    if set(other.stdout.splitlines()) - NEW:
        raise RuntimeError("Unexpected untracked files block the R0.11 release boundary")
    print("R0.10 Git tree boundary: PASS (frozen paths unchanged, exactly seven additive paths)")


if __name__ == "__main__":
    static_check()
    git_boundary()
    print("Rust tests, Clippy, release gate and CI: NOT RUN by this validator")
