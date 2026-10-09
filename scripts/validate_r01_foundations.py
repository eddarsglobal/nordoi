#!/usr/bin/env python3
"""Static integrity validator for R0.1 SPECIFICATION ONLY (no Rust certification)."""
import csv
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXPECTED_COMMIT = "3d1a648bbe7e4e1d7a89affa649277b734660009"

def rows(path):
    with (ROOT / path).open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))

def validate():
    matrix = rows("governance/constitutional_conformance_v1.tsv")
    assert [r["id"] for r in matrix] == [f"C{i}" for i in range(1, 323)]
    assert sum(r["evidence_status"] == "CERTIFIED" for r in matrix) == 310
    assert sum(r["evidence_status"] == "PARTIAL" for r in matrix) == 12
    delivery = rows("governance/nordoi_v1_delivery_model_v1.tsv")
    assert sum(int(r["weight_percent"]) for r in delivery) == 100
    assert sum(int(r["weight_percent"]) * int(r["completion_percent"]) for r in delivery) == 5835
    gate = rows("governance/future_native_gate_v1.tsv")
    decisions = rows("governance/r01_future_native_decision_v1.tsv")
    assert [r["id"] for r in gate] == [f"FNG{i}" for i in range(1, 7)]
    assert [r["rule"] for r in decisions] == [f"FNG{i}" for i in range(1, 7)]
    assert all(r["decision"].startswith("PASS_") and r["evidence"] and r["limitation"] for r in decisions)
    invariants = rows("governance/r01_resource_task_invariants_v1.tsv")
    assert [r["id"] for r in invariants] == [f"R01-I{i:02d}" for i in range(1, 17)]
    assert all(r["proof_obligation"] and r["status"] != "CERTIFIED" for r in invariants)
    doc = (ROOT / "docs/NORDOI_R0_1_NATIVE_RESOURCES_STRUCTURED_CONCURRENCY_SPEC.md").read_text()
    assert EXPECTED_COMMIT in doc and "NOT CERTIFIED" in doc
    for name in ("src/conformance_g01.rs", "src/lib.rs", "src/bin/nordoi.rs"):
        assert (ROOT / name).is_file(), name
    print("R0.1 static integrity: PASS")
    print("G0.1 baseline: 322 principles / 310 certified current-scope / 12 partial")
    print("Delivery: 58.35% PLANNING_NOT_CERTIFICATION (unchanged)")
    print("Future-Native decisions: 6/6 documented (design only)")
    print("Resource/task proof obligations: 16/16 specified (NOT implemented)")
    print("Rust release gate: NOT RUN by this validator")

if __name__ == "__main__":
    validate()
