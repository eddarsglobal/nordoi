#!/usr/bin/env python3
"""R0.13 TEST-ONLY: auditable static boundary, not a crypto signer or native permission gate."""
from __future__ import annotations
import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE_SHA = "851d978c36f4c12fc16c115e89a7774e33ce361b"
PREVIOUS_SHA = "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
NEW = {
    "README_R0_13.md", "docs/NORDOI_R0_13_EVIDENCE_INTEGRITY_CHAIN_SPEC.md",
    "governance/r13_integrity_chain_v1.tsv", "governance/r13_integrity_witnesses_v1.tsv",
    "research/R0_13_INTEGRITY_TRUST_DECISION.md", "scripts/validate_r13_integrity.py",
    "tests/resource_task_r13.rs",
}
AREAS = (
    "LIFECYCLE_SEMANTICS", "EXECUTOR_SCHEDULING", "MEMORY_OWNERSHIP",
    "CAPABILITY_AUTHORITY", "CANCELLATION_CLEANUP", "EFFECT_IO_BOUNDARY",
    "CRASH_RECOVERY", "ADVERSARIAL_TESTING", "CROSS_PLATFORM",
    "COMPILER_NAIR_BINDING", "RESOURCE_LIMITS", "INDEPENDENT_REVIEW",
)


def read_tsv(rel: str, fields: tuple[str, ...]) -> list[dict[str, str]]:
    with (ROOT / rel).open(encoding="utf-8", newline="") as f:
        rd = csv.DictReader(f, delimiter="\t")
        if tuple(rd.fieldnames or ()) != fields:
            raise RuntimeError(f"R0.13 invalid schema: {rel}")
        result = list(rd)
    if any(set(row) != set(fields) or any(v is None or not v.strip() or "\r" in v for v in row.values()) for row in result):
        raise RuntimeError(f"R0.13 malformed row: {rel}")
    return result


def run_git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)


def check_static() -> None:
    for rel in NEW:
        p = ROOT / rel
        if not p.is_file() or p.is_symlink():
            raise RuntimeError(f"R0.13 missing or symlinked additive path: {rel}")
    fields = ("id", "gate_id", "evidence_id", "area", "baseline_sha", "status", "scope",
              "artifact_sha256", "previous_chain_sha256", "chain_sha256", "producer_ref", "reviewer_ref",
              "signature_ref", "trust_root", "revocation", "limitation")
    chain = read_tsv("governance/r13_integrity_chain_v1.tsv", fields)
    r12 = read_tsv("governance/r12_evidence_ledger_v1.tsv", ("id", "gate_id", "area", "baseline_sha", "status", "evidence_scope", "artifact_ref", "reviewer_ref", "limitation"))
    r11 = read_tsv("governance/r11_native_readiness_gates_v1.tsv", ("id", "area", "status", "evidence_scope", "acceptance_criterion", "limitation"))
    if len(chain) != 12 or len(r11) != 12 or len(r12) != 12:
        raise RuntimeError("R0.13 requires exact 12/12/12 historical gates, evidence, and integrity rows")
    for i, (a, b, c) in enumerate(zip(r11, r12, chain), 1):
        if not (a["id"] == b["gate_id"] == c["gate_id"] == f"R11-G{i:02}"
                and b["id"] == c["evidence_id"] == f"R12-E{i:02}"
                and c["id"] == f"R13-L{i:02}"
                and a["area"] == b["area"] == c["area"] == AREAS[i-1]
                and a["status"] == "UNMET" and a["evidence_scope"] == "RESEARCH_ONLY"
                and b["status"] == "ABSENT" and b["evidence_scope"] == "RESEARCH_ONLY"
                and b["baseline_sha"] == PREVIOUS_SHA and b["artifact_ref"] == b["reviewer_ref"] == "NONE"
                and c["baseline_sha"] == BASE_SHA and c["status"] == "ABSENT" and c["scope"] == "RESEARCH_ONLY"
                and all(c[f] == "NONE" for f in fields[7:15])
                and len(c["limitation"]) >= 65):
            raise RuntimeError(f"R0.13 forged or unverified integrity row {i}")
    source = (ROOT / "tests/resource_task_r13.rs").read_text(encoding="utf-8")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(", source)
    witness = read_tsv("governance/r13_integrity_witnesses_v1.tsv", ("id", "witness", "classification", "limitation"))
    if len(tests) != 34 or len(set(tests)) != 34 or len(witness) != 34 or [x["witness"] for x in witness] != tests:
        raise RuntimeError("R0.13 governance witness/test bijection must be exactly 34")
    allowed = {"FROZEN_BASELINE", "HASH_INTEGRITY", "RESEARCH_GOVERNANCE", "FAIL_CLOSED"}
    for i, item in enumerate(witness, 1):
        if item["id"] != f"R13-P{i:02}" or item["classification"] not in allowed or len(item["limitation"]) < 65:
            raise RuntimeError(f"R0.13 invalid governance witness {i}")
    for needle in ('#[path = "../src/effect_audit/hash.rs"]', '#[path = "../src/resource_task_r02.rs"]',
                   'include_str!("../governance/r12_evidence_ledger_v1.tsv")',
                   'include_str!("../governance/r11_native_readiness_gates_v1.tsv")',
                   'R13_WITNESS version=1', 'ResearchReviewOnly', 'fn assess(', 'fn chain_commit(',
                   'fn synthetic_chain(', 'fn frozen_input_boundary('):
        if needle not in source:
            raise RuntimeError(f"R0.13 missing mandatory test-only control: {needle}")
    for forbidden in ('unsafe {', 'std::thread::spawn(', 'tokio::spawn(', '#[ignore]',
                      'ProductionAuthorized', 'NativeAuthorized', '#[should_panic]'):
        if forbidden in source:
            raise RuntimeError(f"R0.13 forbidden native pattern: {forbidden}")
    for rel in ("src/lib.rs", "Cargo.toml"):
        s=(ROOT/rel).read_text(encoding="utf-8")
        if "resource_task_r13" in s or "r13_integrity_chain" in s:
            raise RuntimeError(f"R0.13 exported new native authority via {rel}")
    docs = "\n".join((ROOT / rel).read_text(encoding="utf-8") for rel in NEW if rel.endswith(".md"))
    for keyword in ("r0.12", BASE_SHA, "TEST-ONLY", "UNMET", "ABSENT", "RESEARCH_ONLY", "34", "SHA-256", "NOT"):
        if keyword not in docs:
            raise RuntimeError(f"R0.13 missing mandatory research limitation: {keyword}")
    print("R0.13 static contract: PASS")
    print(f"Certified baseline: r0.12 / {BASE_SHA}")
    print("Historical readiness: 12/12 UNMET; historical evidence: 12/12 ABSENT")
    print("New synthetic integrity registry: 12/12 ABSENT; R0.13 governance witnesses: 34/34")
    print("SHA-256 test-only implementation reused from frozen kernel; no signature verification or native authority")


def check_git() -> None:
    p=run_git("rev-parse", "--show-toplevel")
    if p.returncode or Path(p.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.12 Git boundary: NOT CHECKED (incomplete checkout)")
        return
    tag=run_git("rev-parse", "r0.12^{commit}")
    if tag.returncode or tag.stdout.strip() != BASE_SHA:
        raise RuntimeError("R0.12 certified commit mismatch")
    tree=run_git("ls-tree", "-r", "--name-only", "r0.12")
    if tree.returncode: raise RuntimeError("Cannot enumerate r0.12 frozen paths")
    frozen=set(tree.stdout.splitlines())
    if frozen & NEW: raise RuntimeError("R0.13 overwrites certified path")
    changed=run_git("diff", "--name-only", "r0.12", "--", ".")
    if changed.returncode or set(changed.stdout.splitlines())-NEW:
        raise RuntimeError("R0.13 has modified certified paths")
    unknown=run_git("ls-files", "--others", "--exclude-standard")
    if unknown.returncode or set(unknown.stdout.splitlines())-NEW:
        raise RuntimeError("R0.13 unexpected untracked paths")
    if any(not (ROOT / rel).is_file() for rel in frozen):
        raise RuntimeError("Frozen r0.12 file missing from worktree")
    print("R0.12 Git boundary: PASS (seven additive files only)")


if __name__ == "__main__":
    check_static()
    check_git()
    print("Rust execution, signature authentication, production proof, release gate and CI: NOT RUN by this validator")
