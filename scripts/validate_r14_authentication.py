#!/usr/bin/env python3
"""R0.14 static and git boundary. Does not verify signatures or authorize production."""
from __future__ import annotations
import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "35e048dd83f2c072a6c454494ac099906156dc38"
NEW = {
    "README_R0_14.md", "docs/NORDOI_R0_14_AUTHENTICATION_REVOCATION_SPEC.md",
    "governance/r14_authentication_registry_v1.tsv", "governance/r14_authentication_witnesses_v1.tsv",
    "research/R0_14_AUTHENTICATION_REVOCATION_DECISION.md", "scripts/validate_r14_authentication.py",
    "tests/resource_task_r14.rs",
}
AREAS = ('LIFECYCLE_SEMANTICS', 'EXECUTOR_SCHEDULING', 'MEMORY_OWNERSHIP', 'CAPABILITY_AUTHORITY', 'CANCELLATION_CLEANUP', 'EFFECT_IO_BOUNDARY', 'CRASH_RECOVERY', 'ADVERSARIAL_TESTING', 'CROSS_PLATFORM', 'COMPILER_NAIR_BINDING', 'RESOURCE_LIMITS', 'INDEPENDENT_REVIEW')

def tsv(rel: str, header: tuple[str, ...]) -> list[dict[str,str]]:
    with (ROOT / rel).open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f, delimiter="\t")
        if tuple(reader.fieldnames or ()) != header:
            raise RuntimeError(f"R0.14 bad schema: {rel}")
        rows = list(reader)
    if any(set(row) != set(header) or any(v is None or not v.strip() or "\r" in v for v in row.values()) for row in rows):
        raise RuntimeError(f"R0.14 malformed rows: {rel}")
    return rows

def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)

def static() -> None:
    for rel in NEW:
        p = ROOT / rel
        if not p.is_file() or p.is_symlink():
            raise RuntimeError(f"R0.14 missing or symlinked path: {rel}")
    r11 = tsv("governance/r11_native_readiness_gates_v1.tsv", ("id", "area", "status", "evidence_scope", "acceptance_criterion", "limitation"))
    r12 = tsv("governance/r12_evidence_ledger_v1.tsv", ("id", "gate_id", "area", "baseline_sha", "status", "evidence_scope", "artifact_ref", "reviewer_ref", "limitation"))
    r13 = tsv("governance/r13_integrity_chain_v1.tsv", ("id", "gate_id", "evidence_id", "area", "baseline_sha", "status", "scope", "artifact_sha256", "previous_chain_sha256", "chain_sha256", "producer_ref", "reviewer_ref", "signature_ref", "trust_root", "revocation", "limitation"))
    r14 = tsv("governance/r14_authentication_registry_v1.tsv", ("id", "gate_id", "evidence_id", "integrity_id", "area", "baseline_sha", "status", "scope", "producer_key", "reviewer_key", "trust_root", "authentication", "revocation", "limitation"))
    if any(len(r) != 12 for r in (r11,r12,r13,r14)):
        raise RuntimeError("R0.14 needs 12 frozen rows at every level")
    for i,(a,b,c,d) in enumerate(zip(r11,r12,r13,r14),1):
        if not (a["id"] == b["gate_id"] == c["gate_id"] == d["gate_id"] == f"R11-G{i:02}"
            and b["id"] == c["evidence_id"] == d["evidence_id"] == f"R12-E{i:02}"
            and c["id"] == d["integrity_id"] == f"R13-L{i:02}"
            and d["id"] == f"R14-A{i:02}"
            and a["area"] == b["area"] == c["area"] == d["area"] == AREAS[i-1]
            and a["status"] == "UNMET" and a["evidence_scope"] == "RESEARCH_ONLY"
            and b["status"] == "ABSENT" and b["evidence_scope"] == "RESEARCH_ONLY"
            and b["baseline_sha"] == "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
            and b["artifact_ref"] == b["reviewer_ref"] == "NONE"
            and c["status"] == "ABSENT" and c["scope"] == "RESEARCH_ONLY"
            and c["baseline_sha"] == "851d978c36f4c12fc16c115e89a7774e33ce361b"
            and all(c[k] == "NONE" for k in ("artifact_sha256","previous_chain_sha256","chain_sha256","producer_ref","reviewer_ref","signature_ref","trust_root","revocation"))
            and d["baseline_sha"] == BASE and d["status"] == "ABSENT" and d["scope"] == "RESEARCH_ONLY"
            and all(d[k] == "NONE" for k in ("producer_key","reviewer_key","trust_root","authentication","revocation"))
            and len(d["limitation"]) >= 65):
            raise RuntimeError(f"R0.14 forged canonical row {i}")
    source = (ROOT / "tests/resource_task_r14.rs").read_text(encoding="utf-8")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(", source)
    witnesses = tsv("governance/r14_authentication_witnesses_v1.tsv", ("id", "witness", "classification", "limitation"))
    if len(tests) != 44 or len(set(tests)) != 44 or len(witnesses) != 44 or [w["witness"] for w in witnesses] != tests:
        raise RuntimeError("R0.14 exact test-to-witness bijection failure")
    classes = {"RESEARCH_AUTHENTICATION", "REVOCATION_GOVERNANCE", "FROZEN_BASELINE", "FAIL_CLOSED"}
    if any(w["id"] != f"R14-P{i:02}" or w["classification"] not in classes or len(w["limitation"]) < 65 for i,w in enumerate(witnesses,1)):
        raise RuntimeError("R0.14 malformed witness")
    for needle in ('#[path = "../src/resource_task_r02.rs"]', '#[path = "../src/effect_audit/hash.rs"]',
                   'include_str!("../governance/r11_native_readiness_gates_v1.tsv")',
                   'include_str!("../governance/r12_evidence_ledger_v1.tsv")',
                   'include_str!("../governance/r13_integrity_chain_v1.tsv")',
                   'R14_WITNESS version=1', 'ResearchReviewOnly', 'fn assess(', 'fn revoke(', 'fn fixture_commit(', 'fn historical_boundary('):
        if needle not in source:
            raise RuntimeError(f"R0.14 mandatory Rust research boundary missing: {needle}")
    for forbidden in ('unsafe {', 'std::thread::spawn(', 'tokio::spawn(', '#[ignore]', '#[should_panic]', 'ProductionAuthorized', 'NativeAuthorized'):
        if forbidden in source: raise RuntimeError(f"R0.14 prohibited capability: {forbidden}")
    for rel in ("src/lib.rs", "Cargo.toml"):
        contents = (ROOT/rel).read_text(encoding="utf-8")
        if 'resource_task_r14' in contents or 'r14_authentication_registry' in contents:
            raise RuntimeError(f"R0.14 native or manifest export: {rel}")
    docs = "\n".join((ROOT/rel).read_text(encoding="utf-8") for rel in NEW if rel.endswith(".md"))
    for word in ("r0.13", BASE, "TEST-ONLY", "RESEARCH_ONLY", "UNMET", "ABSENT", "SHA-256", "NOT", str(44)):
        if word not in docs: raise RuntimeError(f"R0.14 missing documentary limitation: {word}")
    print("R0.14 static contract: PASS")
    print(f"Certified baseline: r0.13 / {BASE}")
    print("Native readiness: 12/12 UNMET; R0.12 evidence and R0.13 integrity: 12/12 ABSENT")
    print("R0.14 real authentication and trust roots: 12/12 ABSENT; governance witnesses: 44/44")
    print("Synthetic PUBLIC fixtures only: NO real digital signature verification and NO native authority")

def git_boundary() -> None:
    p = git("rev-parse", "--show-toplevel")
    if p.returncode or Path(p.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.13 Git boundary: NOT CHECKED (incomplete checkout)")
        return
    tag = git("rev-parse", "r0.13^{commit}")
    if tag.returncode or tag.stdout.strip() != BASE:
        raise RuntimeError("R0.13 tag SHA mismatch")
    tree = git("ls-tree", "-r", "--name-only", "r0.13")
    if tree.returncode: raise RuntimeError("Cannot read frozen R0.13 tree")
    frozen = set(tree.stdout.splitlines())
    if frozen & NEW: raise RuntimeError("R0.14 overlaps certified file")
    changes = git("diff", "--name-only", "r0.13", "--", ".")
    if changes.returncode or set(changes.stdout.splitlines()) - NEW:
        raise RuntimeError("Certified R0.13 tracked paths changed")
    new = git("ls-files", "--others", "--exclude-standard")
    if new.returncode or set(new.stdout.splitlines()) - NEW:
        raise RuntimeError("Unexpected untracked paths")
    if any(not (ROOT/rel).is_file() for rel in frozen):
        raise RuntimeError("Missing certified R0.13 file")
    print("R0.13 Git boundary: PASS (seven additive files only)")

if __name__ == "__main__":
    static()
    git_boundary()
    print("Rust execution, Release Gate, real cryptographic authentication and CI: NOT RUN by static validator")
