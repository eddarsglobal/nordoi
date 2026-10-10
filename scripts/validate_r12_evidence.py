#!/usr/bin/env python3
"""R0.12 TEST-ONLY evidence intake metadata audit; never promotes native readiness."""
from __future__ import annotations

import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE_SHA = "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
NEW = {
    "README_R0_12.md",
    "docs/NORDOI_R0_12_EVIDENCE_ADMISSION_SPEC.md",
    "governance/r12_evidence_ledger_v1.tsv",
    "governance/r12_evidence_witnesses_v1.tsv",
    "research/R0_12_EVIDENCE_BOUNDARY_DECISION.md",
    "scripts/validate_r12_evidence.py",
    "tests/resource_task_r12.rs",
}
AREAS = (
    "LIFECYCLE_SEMANTICS", "EXECUTOR_SCHEDULING", "MEMORY_OWNERSHIP",
    "CAPABILITY_AUTHORITY", "CANCELLATION_CLEANUP", "EFFECT_IO_BOUNDARY",
    "CRASH_RECOVERY", "ADVERSARIAL_TESTING", "CROSS_PLATFORM",
    "COMPILER_NAIR_BINDING", "RESOURCE_LIMITS", "INDEPENDENT_REVIEW",
)


def rows(rel: str, fields: tuple[str, ...]) -> list[dict[str, str]]:
    path = ROOT / rel
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or []) != fields:
            raise RuntimeError(f"R0.12 invalid TSV schema: {rel}")
        result = list(reader)
    if any(set(row) != set(fields) or any(value is None or not value.strip() for value in row.values()) for row in result):
        raise RuntimeError(f"R0.12 malformed TSV row: {rel}")
    return result


def run_git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)


def static_check() -> None:
    for rel in NEW:
        path = ROOT / rel
        if not path.is_file() or path.is_symlink():
            raise RuntimeError(f"R0.12 required additive file missing/symlinked: {rel}")

    native = rows(
        "governance/r11_native_readiness_gates_v1.tsv",
        ("id", "area", "status", "evidence_scope", "acceptance_criterion", "limitation"),
    )
    ledger = rows(
        "governance/r12_evidence_ledger_v1.tsv",
        ("id", "gate_id", "area", "baseline_sha", "status", "evidence_scope", "artifact_ref", "reviewer_ref", "limitation"),
    )
    if len(native) != 12 or len(ledger) != 12:
        raise RuntimeError("R0.12 requires twelve certified research gates and twelve evidence ledger rows")
    for i, (gate, entry) in enumerate(zip(native, ledger), 1):
        if (
            gate["id"] != f"R11-G{i:02}"
            or gate["area"] != AREAS[i - 1]
            or gate["status"] != "UNMET"
            or gate["evidence_scope"] != "RESEARCH_ONLY"
            or entry["id"] != f"R12-E{i:02}"
            or entry["gate_id"] != gate["id"]
            or entry["area"] != gate["area"]
            or entry["baseline_sha"] != BASE_SHA
            or entry["status"] != "ABSENT"
            or entry["evidence_scope"] != "RESEARCH_ONLY"
            or entry["artifact_ref"] != "NONE"
            or entry["reviewer_ref"] != "NONE"
            or len(entry["limitation"].strip()) < 65
        ):
            raise RuntimeError(f"R0.12 admission ledger has noncanonical or forged row {i}")

    source = (ROOT / "tests/resource_task_r12.rs").read_text(encoding="utf-8")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", source)
    witnesses = rows(
        "governance/r12_evidence_witnesses_v1.tsv",
        ("id", "witness", "classification", "limitation"),
    )
    if len(tests) != 29 or len(set(tests)) != 29 or len(witnesses) != 29:
        raise RuntimeError("R0.12 must have exactly 29 distinct tests and witnesses")
    if tests != [row["witness"] for row in witnesses]:
        raise RuntimeError("R0.12 witnesses do not map one-to-one to Rust test order")
    allowed = {"ISOLATION_SMOKE", "LEDGER_BASELINE", "SYNTHETIC_ADVISORY", "SCHEMA_NEGATIVE", "HISTORIC_BOUNDARY", "FAIL_CLOSED"}
    for i, w in enumerate(witnesses, 1):
        if w["id"] != f"R12-P{i:02}" or w["classification"] not in allowed or len(w["limitation"]) < 65:
            raise RuntimeError(f"R0.12 malformed test witness {i}")

    for required in (
        '#[path = "../src/resource_task_r02.rs"]',
        'include_str!("../governance/r11_native_readiness_gates_v1.tsv")',
        'include_str!("../governance/r12_evidence_ledger_v1.tsv")',
        "enum Advisory", "ResearchReviewOnly", "fn assess(",
        "R12_WITNESS version=1", "replay_checked()",
    ):
        if required not in source:
            raise RuntimeError(f"R0.12 source missing expected research-only isolation: {required}")
    for forbidden in (
        "std::thread::spawn(", "tokio::spawn(", "unsafe {", "#[ignore]",
        "#[should_panic]", "ProductionAuthorized", "NativeAuthorized",
    ):
        if forbidden in source:
            raise RuntimeError(f"R0.12 forbidden native/testing pattern: {forbidden}")
    for rel in ("src/lib.rs", "Cargo.toml"):
        content = (ROOT / rel).read_text(encoding="utf-8")
        if "resource_task_r12" in content or "r12_evidence_ledger" in content:
            raise RuntimeError(f"R0.12 must not enter runtime/compiler/NAIR: {rel}")
    docs = "\n".join((ROOT / p).read_text(encoding="utf-8") for p in NEW if p.endswith(".md"))
    for required in ("r0.11", BASE_SHA, "TEST-ONLY", "12", "29", "UNMET", "RESEARCH_ONLY", "NOT", "CI"):
        if required not in docs:
            raise RuntimeError(f"R0.12 documents omit a mandatory constraint: {required}")
    print("R0.12 static contract: PASS")
    print(f"Certified input baseline: r0.11 / {BASE_SHA}")
    print("Evidence admission ledger: 12/12 ABSENT; native gates: 12/12 UNMET")
    print("Governance witnesses: 29/29; research-only advisory cannot grant runtime authority")


def git_boundary() -> None:
    root = run_git("rev-parse", "--show-toplevel")
    if root.returncode or Path(root.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.11 Git tree boundary: NOT CHECKED (no complete Git root)")
        return
    target = run_git("rev-parse", "r0.11^{commit}")
    if target.returncode or target.stdout.strip() != BASE_SHA:
        raise RuntimeError("Certified r0.11 tag absent or incorrectly targeted")
    tree = run_git("ls-tree", "-r", "--name-only", "r0.11")
    if tree.returncode:
        raise RuntimeError("Cannot enumerate frozen r0.11 files")
    frozen = set(tree.stdout.splitlines())
    if frozen & NEW:
        raise RuntimeError("New R0.12 paths overwrite the frozen baseline")
    differences = run_git("diff", "--name-only", "r0.11", "--", ".")
    if differences.returncode or set(differences.stdout.splitlines()) - NEW:
        raise RuntimeError("Frozen historical files modified since r0.11")
    untracked = run_git("ls-files", "--others", "--exclude-standard")
    if untracked.returncode or set(untracked.stdout.splitlines()) - NEW:
        raise RuntimeError("Unexpected untracked files in R0.12 candidate")
    if any(not (ROOT / path).is_file() for path in frozen):
        raise RuntimeError("Certified historic file missing from the worktree")
    print("R0.11 Git tree boundary: PASS (seven additive files, frozen sources untouched)")


if __name__ == "__main__":
    static_check()
    git_boundary()
    print("Rust execution, release gate, cryptographic evidence verification and CI: NOT RUN by this validator")
