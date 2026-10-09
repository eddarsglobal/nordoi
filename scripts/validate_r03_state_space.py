#!/usr/bin/env python3
"""R0.3 candidate static contract validation (not a Rust test runner)."""
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/NORDOI_R0_3_BOUNDED_STATE_SPACE_VERIFICATION_SPEC.md"
RESEARCH = ROOT / "research/R0_3_STATE_SPACE_DECISION.md"
README = ROOT / "README_R0_3.md"
MATRIX = ROOT / "governance/r03_state_space_obligations_v1.tsv"
TEST = ROOT / "tests/resource_task_r03.rs"
BASELINE = "d9b7153cac7d044cefdc57645758c331fd75546d"


def verify() -> None:
    docs = DOC.read_text(encoding="utf-8")
    research = RESEARCH.read_text(encoding="utf-8")
    readme = README.read_text(encoding="utf-8")
    source = TEST.read_text(encoding="utf-8")
    with MATRIX.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if len(rows) != 9:
        raise ValueError(f"Expected 9 governance witnesses, found {len(rows)}")
    expected_ids = [f"R03-V{i:02}" for i in range(1, 10)]
    actual_ids = [row["id"] for row in rows]
    if actual_ids != expected_ids:
        raise ValueError(f"Witness IDs not canonical: {actual_ids}")
    declared = re.findall(r"(?m)^fn ([a-z0-9_]+)\(\)\s*\{", source)
    test_functions = re.findall(r"(?m)#\[test\]\s*\nfn ([a-z0-9_]+)\(\)", source)
    if len(test_functions) != 9 or len(set(test_functions)) != 9:
        raise ValueError(f"Expected exactly 9 distinct Rust tests, got {test_functions}")
    if set(row["witness"] for row in rows) != set(test_functions):
        raise ValueError("R0.3 matrix witnesses must map bijectively to Rust test names")
    if not set(test_functions).issubset(set(declared)):
        raise ValueError("Witness functions are not declared")
    allowed = {"BOUNDED_EXPLORED", "TESTED_EXAMPLE_ONLY", "STATIC_CHECK_ONLY"}
    if any(row["status"] not in allowed or not row["limitation"].strip() for row in rows):
        raise ValueError("Every witness needs an allowed status and a stated limitation")
    if sum(row["status"] == "BOUNDED_EXPLORED" for row in rows) != 4:
        raise ValueError("Exactly four bounded exploration profiles are required")
    for fng in range(1, 7):
        if f"FNG{fng}" not in docs:
            raise ValueError(f"Future-Native rule FNG{fng} not documented")
    for term in [
        "MAX_VISITED: usize = 20_000", "MAX_EVENTS: usize = 8",
        "VecDeque", "BTreeSet", "replay_checked()",
        "validate_closed_report", "all_two_task_completion_and_join_orders_keep_canonical_report",
        '#[path = "../src/resource_task_r02.rs"]',
    ]:
        if term not in source:
            raise ValueError(f"Required bounded verification mechanism not found: {term}")
    # Rustfmt may wrap a long assert_eq! call across multiple lines.
    # Check the assertion's operands, not its incidental whitespace.
    if not re.search(r"assert_eq!\s*\(\s*next\.model\s*,\s*before\s*,", source):
        raise ValueError("Required rejected-transition atomicity assertion not found")
    if not (BASELINE in docs and BASELINE in readme):
        raise ValueError("Explicit r0.2.1 certified baseline not recorded")
    if not all("no" in content.lower() or "not" in content.lower() for content in [docs, research, readme]):
        raise ValueError("Missing limitations/disclaimers")
    for forbidden in ["std::thread::spawn", "tokio::spawn", "async_std::task::spawn"]:
        if forbidden in source:
            raise ValueError(f"Production-like task execution prohibited: {forbidden}")
    lib = ROOT / "src/lib.rs"
    if lib.exists() and ("resource_task_r02" in lib.read_text() or "resource_task_r03" in lib.read_text()):
        raise ValueError("Reference model must not become part of public library")
    print("R0.3 static contract: PASS")
    print("Certified input baseline: r0.2.1 / " + BASELINE)
    print("Witnesses: 9/9; bounded exploration profiles: 4")
    print("Source integration: TEST-ONLY (no runtime/NAIR authority)")
    print("Rust compilation, actual state counts and full release gate: NOT RUN by this script")


if __name__ == "__main__":
    verify()
