#!/usr/bin/env python3
"""R0.15 static research-only policy audit; not a native authorization."""
from __future__ import annotations

import csv
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "86fc90a79f5d0cef83c730bcd7c02175307fffd3"
NEW = {
    "README_R0_15.md",
    "docs/NORDOI_R0_15_INDEPENDENT_VERIFICATION_SPEC.md",
    "governance/r15_trust_policy_registry_v1.tsv",
    "governance/r15_verification_witnesses_v1.tsv",
    "research/R0_15_VERIFICATION_POLICY_DECISION.md",
    "scripts/validate_r15_verification.py",
    "tests/resource_task_r15.rs",
}
AREAS = (
    "LIFECYCLE_SEMANTICS", "EXECUTOR_SCHEDULING", "MEMORY_OWNERSHIP",
    "CAPABILITY_AUTHORITY", "CANCELLATION_CLEANUP", "EFFECT_IO_BOUNDARY",
    "CRASH_RECOVERY", "ADVERSARIAL_TESTING", "CROSS_PLATFORM",
    "COMPILER_NAIR_BINDING", "RESOURCE_LIMITS", "INDEPENDENT_REVIEW",
)


def tsv(rel: str, header: tuple[str, ...]) -> list[dict[str, str]]:
    with (ROOT / rel).open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f, delimiter="\t")
        if tuple(reader.fieldnames or ()) != header:
            raise RuntimeError(f"R0.15 unexpected schema: {rel}")
        rows = list(reader)
    if any(set(r) != set(header) or any(v is None or not v.strip() or "\r" in v for v in r.values()) for r in rows):
        raise RuntimeError(f"R0.15 malformed row: {rel}")
    return rows


def static() -> None:
    for rel in NEW:
        path = ROOT / rel
        if not path.is_file() or path.is_symlink():
            raise RuntimeError(f"Missing / linked R0.15 path: {rel}")
    g = tsv("governance/r11_native_readiness_gates_v1.tsv", (
        "id", "area", "status", "evidence_scope", "acceptance_criterion", "limitation"))
    e = tsv("governance/r12_evidence_ledger_v1.tsv", (
        "id", "gate_id", "area", "baseline_sha", "status", "evidence_scope", "artifact_ref", "reviewer_ref", "limitation"))
    i = tsv("governance/r13_integrity_chain_v1.tsv", (
        "id", "gate_id", "evidence_id", "area", "baseline_sha", "status", "scope", "artifact_sha256",
        "previous_chain_sha256", "chain_sha256", "producer_ref", "reviewer_ref", "signature_ref", "trust_root", "revocation", "limitation"))
    a = tsv("governance/r14_authentication_registry_v1.tsv", (
        "id", "gate_id", "evidence_id", "integrity_id", "area", "baseline_sha", "status", "scope", "producer_key",
        "reviewer_key", "trust_root", "authentication", "revocation", "limitation"))
    p = tsv("governance/r15_trust_policy_registry_v1.tsv", (
        "id", "gate_id", "evidence_id", "integrity_id", "authentication_id", "area", "baseline_sha", "status", "scope",
        "policy", "verifier_ref", "trust_root", "limitation"))
    if any(len(rows) != 12 for rows in (g, e, i, a, p)):
        raise RuntimeError("R0.15 requires exactly twelve frozen and policy entries per version")
    for idx, (rg, re_, ri, ra, rp) in enumerate(zip(g, e, i, a, p), 1):
        n = f"{idx:02}"
        if not (
            rg["id"] == re_["gate_id"] == ri["gate_id"] == ra["gate_id"] == rp["gate_id"] == f"R11-G{n}"
            and re_["id"] == ri["evidence_id"] == ra["evidence_id"] == rp["evidence_id"] == f"R12-E{n}"
            and ri["id"] == ra["integrity_id"] == rp["integrity_id"] == f"R13-L{n}"
            and ra["id"] == rp["authentication_id"] == f"R14-A{n}"
            and rp["id"] == f"R15-V{n}"
            and rg["area"] == re_["area"] == ri["area"] == ra["area"] == rp["area"] == AREAS[idx - 1]
            and rg["status"] == "UNMET" and rg["evidence_scope"] == "RESEARCH_ONLY"
            and re_["status"] == "ABSENT" and re_["evidence_scope"] == "RESEARCH_ONLY"
            and re_["baseline_sha"] == "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
            and re_["artifact_ref"] == re_["reviewer_ref"] == "NONE"
            and ri["status"] == "ABSENT" and ri["scope"] == "RESEARCH_ONLY"
            and ri["baseline_sha"] == "851d978c36f4c12fc16c115e89a7774e33ce361b"
            and all(ri[x] == "NONE" for x in (
                "artifact_sha256", "previous_chain_sha256", "chain_sha256", "producer_ref", "reviewer_ref",
                "signature_ref", "trust_root", "revocation"))
            and ra["status"] == "ABSENT" and ra["scope"] == "RESEARCH_ONLY"
            and ra["baseline_sha"] == "35e048dd83f2c072a6c454494ac099906156dc38"
            and all(ra[x] == "NONE" for x in (
                "producer_key", "reviewer_key", "trust_root", "authentication", "revocation"))
            and rp["status"] == "ABSENT" and rp["scope"] == "RESEARCH_ONLY" and rp["baseline_sha"] == BASE
            and rp["policy"] == "DUAL_SYNTHETIC_REVIEW"
            and rp["verifier_ref"] == rp["trust_root"] == "NONE"
            and len(rp["limitation"]) >= 65
        ):
            raise RuntimeError(f"R0.15 canonical historical/policy lineage mismatch at gate {idx}")
    source = (ROOT / "tests/resource_task_r15.rs").read_text(encoding="utf-8")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(", source)
    witnesses = tsv("governance/r15_verification_witnesses_v1.tsv", (
        "id", "witness", "classification", "limitation"))
    if len(tests) != 39 or len(set(tests)) != 39 or len(witnesses) != 39 or [w["witness"] for w in witnesses] != tests:
        raise RuntimeError("R0.15 test/witness bijection mismatch")
    if any(w["id"] != f"R15-P{idx:02}" or w["classification"] not in {"FROZEN_BASELINE", "TRUST_POLICY", "FAIL_CLOSED"}
           or len(w["limitation"]) < 65 for idx, w in enumerate(witnesses, 1)):
        raise RuntimeError("R0.15 witness governance schema mismatch")
    for required in (
        '#[path = "../src/effect_audit/hash.rs"]',
        'include_str!("../governance/r11_native_readiness_gates_v1.tsv")',
        'include_str!("../governance/r12_evidence_ledger_v1.tsv")',
        'include_str!("../governance/r13_integrity_chain_v1.tsv")',
        'include_str!("../governance/r14_authentication_registry_v1.tsv")',
        'R15_WITNESS version=1', 'ResearchReviewOnly', 'fn canonical_policy(', 'fn assess(',
    ):
        if required not in source:
            raise RuntimeError(f"R0.15 mandatory guard missing: {required}")
    for forbidden in ('unsafe {', 'std::thread::spawn(', 'tokio::spawn(', '#[ignore]', '#[should_panic]',
                      'ProductionAuthorized', 'NativeAuthorized'):
        if forbidden in source:
            raise RuntimeError(f"R0.15 prohibited construct: {forbidden}")
    for rel in ("src/lib.rs", "Cargo.toml"):
        file = ROOT / rel
        if file.exists() and ("resource_task_r15" in file.read_text(encoding="utf-8") or
                              "r15_trust_policy_registry" in file.read_text(encoding="utf-8")):
            raise RuntimeError(f"R0.15 is exported into a frozen production interface: {rel}")
    docs = "\n".join((ROOT / rel).read_text(encoding="utf-8") for rel in NEW if rel.endswith(".md"))
    for required in ("r0.14", BASE, "TEST-ONLY", "RESEARCH_ONLY", "UNMET", "ABSENT", "SHA-256", "NOT", "39"):
        if required not in docs:
            raise RuntimeError(f"R0.15 missing documentary research constraint: {required}")
    print("R0.15 static contract: PASS")
    print(f"Certified baseline: r0.14 / {BASE}")
    print("Native readiness: 12/12 UNMET; historical evidence, integrity and authentication: 12/12 ABSENT")
    print("Independent external verification: 12/12 ABSENT; governance witnesses: 39/39")
    print("Research-only public review fixtures; NO real independent verification and NO native authority")


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)


def git_boundary() -> None:
    current = git("rev-parse", "--show-toplevel")
    if current.returncode != 0 or Path(current.stdout.strip()).resolve() != ROOT.resolve():
        print("R0.14 Git boundary: NOT CHECKED (reconstructed partial checkout)")
        return
    tag = git("rev-parse", "r0.14^{commit}")
    if tag.returncode or tag.stdout.strip() != BASE:
        raise RuntimeError("Certified tag r0.14 SHA does not match")
    tree = git("ls-tree", "-r", "--name-only", "r0.14")
    if tree.returncode:
        raise RuntimeError("Cannot inspect frozen r0.14 tree")
    frozen = set(tree.stdout.splitlines())
    if frozen & NEW:
        raise RuntimeError("R0.15 overlaps existing certified files")
    tracked = git("diff", "--name-only", "r0.14", "--", ".")
    if tracked.returncode or set(tracked.stdout.splitlines()) - NEW:
        raise RuntimeError("An r0.14 frozen tracked path was changed")
    untracked = git("ls-files", "--others", "--exclude-standard")
    if untracked.returncode or set(untracked.stdout.splitlines()) - NEW:
        raise RuntimeError("Unexpected untracked paths outside R0.15")
    if any(not (ROOT / item).is_file() for item in frozen):
        raise RuntimeError("Missing r0.14 frozen file")
    print("R0.14 Git boundary: PASS (seven additive files only)")


if __name__ == "__main__":
    static()
    git_boundary()
    print("Rust execution, signature authentication, production review, release gate and CI: NOT RUN by static validator")
