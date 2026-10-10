#!/usr/bin/env python3
"""Static R0.4 contract validation; NOT a Rust test runner or formal proof."""
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "cb9e54b16e7dfc91273976e689d18ae742ad6476"
TEST = ROOT / "tests/resource_task_r04.rs"
DOC = ROOT / "docs/NORDOI_R0_4_GENERATIVE_ADVERSARIAL_VERIFICATION_SPEC.md"
RESEARCH = ROOT / "research/R0_4_ADVERSARIAL_VERIFICATION_DECISION.md"
README = ROOT / "README_R0_4.md"
MATRIX = ROOT / "governance/r04_adversarial_witnesses_v1.tsv"


def validate() -> None:
    source = TEST.read_text(encoding="utf-8")
    docs = DOC.read_text(encoding="utf-8")
    readme = README.read_text(encoding="utf-8")
    research = RESEARCH.read_text(encoding="utf-8")
    with MATRIX.open(encoding="utf-8", newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    if len(rows) != 12 or [r["id"] for r in rows] != [f"R04-A{i:02}" for i in range(1, 13)]:
        raise ValueError("R0.4 requires 12 unique sequential governance obligations")
    tests = re.findall(r"#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(\s*\)", source)
    if len(tests) != 12 or len(set(tests)) != 12:
        raise ValueError(f"R0.4 needs 12 unique Rust test declarations, found {tests}")
    if {r["witness"] for r in rows} != set(tests):
        raise ValueError("R0.4 matrix must map bijectively to declared Rust tests")
    valid = {"GENERATED_BOUNDED", "METAMORPHIC_EXAMPLE", "SYNTHETIC_SHRINK", "ADVERSARIAL_EXAMPLE", "STATIC_CHECK_ONLY"}
    for row in rows:
        if row["classification"] not in valid or len(row["limitation"].strip()) < 20:
            raise ValueError(f"Incomplete status/limitation: {row['id']}")
    seed_match = re.search(r"const SEEDS\s*:\s*\[u64;\s*16\]\s*=\s*\[([^]]+)\]", source)
    if not seed_match:
        raise ValueError("Versioned 16-seed corpus not declared")
    seeds = [int(x.strip()) for x in seed_match.group(1).split(",") if x.strip()]
    if seeds != [0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987]:
        raise ValueError("Deterministic R0.4 seed corpus drift")
    required = [
        "MAX_GENERATED_STEPS: usize = 96", "MAX_ACCEPTED_EVENTS: usize = 48",
        "wrapping_add", "wrapping_mul", "replay_checked()", "fn reduce<",
        '#[path = "../src/resource_task_r02.rs"]', "include_str!(\"../src/lib.rs\")",
    ]
    for token in required:
        if token not in source:
            raise ValueError(f"Missing R0.4 witness mechanism: {token}")
    # Rustfmt may wrap long assert_eq! invocations across lines.
    if not re.search(r"assert_eq!\s*\(\s*self\.model\s*,\s*before\b", source):
        raise ValueError("Rejected-transition atomicity check missing")
    if not re.search(r"assert_eq!\s*\(\s*self\.model\.event_count\(\)\s*,\s*events\s*\+\s*1", source):
        raise ValueError("Accepted-event increment check missing")
    for token in ["FNG1", "FNG2", "FNG3", "FNG4", "FNG5", "FNG6", "not an exhaustive", "synthetic"]:
        if token.lower() not in docs.lower():
            raise ValueError(f"Scope disclaimer/gate absent: {token}")
    if BASELINE not in docs or BASELINE not in readme:
        raise ValueError("Frozen r0.3 baseline identity missing")
    if "TEST-ONLY" not in docs or "TEST-ONLY" not in readme:
        raise ValueError("Isolation boundary missing")
    if "not" not in research.lower():
        raise ValueError("Missing analysis of limitations")
    for forbidden in ["std::thread::spawn", "tokio::spawn", "unsafe {", "async_std::task::spawn"]:
        if forbidden in source:
            raise ValueError(f"Out-of-scope runtime behavior: {forbidden}")
    lib = ROOT / "src/lib.rs"
    if lib.exists() and "resource_task_r04" in lib.read_text(encoding="utf-8"):
        raise ValueError("Test-only model was exported to production library")
    print("R0.4 static contract: PASS")
    print("Certified input baseline: r0.3 / " + BASELINE)
    print("Governance witnesses: 12/12; fixed corpus: 16 seeds x 96 steps; max accepted events: 48")
    print("Isolation: TEST-ONLY; no runtime/NAIR/compiler/authority extension")
    print("Rust compilation, behavioral results, security proof and CI: NOT RUN by this validator")


if __name__ == "__main__":
    validate()
