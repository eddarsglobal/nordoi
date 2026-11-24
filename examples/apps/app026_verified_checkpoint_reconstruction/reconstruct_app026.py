#!/usr/bin/env python3
"""Reconstruct a portable checkpoint from a verified proper extension only.
No native task execution, no automatic replay, no replacement of old evidence.
Requires a quiescent, trusted host and externally obtained input pins.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "app025_interrupted_dag_forensics"))
import audit_app025 as forensic

portable = forensic.portable
native = forensic.native
SCHEMA = "NORDOI_APP026_VERIFIED_CHECKPOINT_RECONSTRUCTION_V1"


def reconstruct(checkpoint, checkpoint_pin, root, suffix, suffix_pin, target):
    # Caller must hold the host quiescent; no hostile-host race guarantee.
    target = target.absolute()
    native.real_directory(target.parent, "DESTINATION_PARENT_NOT_REAL")
    parent = target.parent.resolve(strict=True)
    for source in (checkpoint, root):
        actual = source.resolve(strict=True)
        forensic.must(parent != actual and actual not in parent.parents,
                      "DESTINATION_OVERLAPS_ORIGINAL_EVIDENCE")
    forensic.must(not target.exists() and not target.is_symlink(),
                  "DESTINATION_ALREADY_EXISTS")
    report = forensic.reconcile(checkpoint, checkpoint_pin, root, suffix, suffix_pin)
    forensic.must(report["status"] == "CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT",
                  "NOT_VERIFIED_PROPER_EXTENSION")
    original, _, old_events = portable.verify(checkpoint, checkpoint_pin)
    prefix = native.bounded_regular_file(checkpoint/"handoff"/"native-prefix.jsonl",
        forensic.audit.MAX_EVENTS_BYTES, "ORIGINAL_PREFIX_INVALID")
    raw = native.bounded_regular_file(suffix, forensic.MAX_SUFFIX_BYTES, "SUFFIX_INVALID")
    forensic.must(forensic.sha256(prefix) == original["prefix_sha256"]
                  and forensic.sha256(raw) == suffix_pin
                  and forensic.sha256(prefix+raw) == report["combined_events_sha256"],
                  "INPUT_CHANGED_AFTER_AUDIT")
    plan = checkpoint/"handoff"/"plan.json"
    with tempfile.TemporaryDirectory(prefix="nordoi_app026_stage_") as tmp:
        stage = Path(tmp)
        joined = stage/"joined.jsonl"
        joined.write_bytes(prefix+raw)
        handoff = stage/"handoff"
        inner = portable.inner.seal(handoff, root, plan, joined, report["plan_sha256"])
        staged = stage/"portable"
        packed = portable.pack(staged, handoff, root, inner["manifest_sha256"])
        pin = packed["portable_sha256"]
        manifest, _, events = portable.verify(staged, pin)
        forensic.must(manifest["prefix_sha256"] == report["combined_events_sha256"]
                      and manifest["plan_sha256"] == original["plan_sha256"]
                      and len(events) == report["verified_events_total"]
                      and len(old_events) < len(events) < report["total_dag_nodes"]
                      and events[:len(old_events)] == old_events
                      and manifest["committed"] == report["verified_native_jobs"]
                      and manifest["skipped"] == report["skipped_nodes"],
                      "STAGED_CHECKPOINT_CONTRADICTION")
        inventory = {x["path"]: x for x in manifest["native_files"]}
        forensic.must(all(inventory.get(x["path"]) == x for x in original["native_files"]),
                      "ORIGINAL_NATIVE_BYTES_CHANGED")
        forensic.must(forensic.reconcile(checkpoint, checkpoint_pin, root, suffix, suffix_pin) == report,
                      "SOURCE_CHANGED_BEFORE_PUBLICATION")
        # Publish exclusively from private verified stage, not live user files.
        # APP-014 verifies the full copy BEFORE its final marker; incomplete
        # publication is unknown, not a successful rollback.
        try:
            published = portable.pack(target, staged/"handoff", staged/"native",
                                      manifest["inner_manifest_sha256"])
            forensic.must(published["portable_sha256"] == pin, "PUBLICATION_PIN_CHANGED")
            final, _, final_events = portable.verify(target, pin)
            forensic.must(final == manifest and final_events == events,
                          "PUBLICATION_VERIFICATION_FAILED")
        except Exception as exc:
            raise forensic.Blocked("PUBLICATION_OUTCOME_UNKNOWN_NO_REPLAY") from exc
    return {"schema": SCHEMA, "status": "CHECKPOINT_RECONSTRUCTED",
            "previous_checkpoint_sha256": checkpoint_pin, "suffix_sha256": suffix_pin,
            "new_checkpoint_sha256": pin, "combined_events_sha256": manifest["prefix_sha256"],
            "verified_events": len(events), "remaining_nodes": report["total_dag_nodes"]-len(events),
            "native_jobs_verified": manifest["committed"], "native_processes_started": 0,
            "next_action": "INDEPENDENTLY_PIN_AND_VERIFY_NEW_CHECKPOINT_BEFORE_RESUME",
            "external_pin_authentication": "NOT_PROVEN", "global_antirollback": "NOT_PROVEN",
            "hostile_host_concurrency": "NOT_COVERED", "crash_durability": "UNCLAIMED",
            "formal_bureau_certification": "NOT_CERTIFIED", "production": "NOT_AUTHORIZED"}


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    for name in ("portable-dir", "restore-root", "suffix-events", "destination"):
        p.add_argument("--"+name, type=Path, required=True)
    p.add_argument("--expected-portable-sha256", required=True)
    p.add_argument("--expected-suffix-sha256", required=True)
    a = p.parse_args(argv)
    try:
        result = reconstruct(a.portable_dir, a.expected_portable_sha256, a.restore_root,
                             a.suffix_events, a.expected_suffix_sha256, a.destination)
        rc = 0
    except Exception as exc:
        result = {"schema": SCHEMA, "status": "BLOCK",
                  "reason": getattr(exc, "code", "EVIDENCE_OR_PUBLICATION_NOT_VERIFIED"),
                  "next_action": "STOP_NO_AUTOMATIC_NATIVE_REPLAY",
                  "production": "NOT_AUTHORIZED", "formal_bureau_certification": "NOT_CERTIFIED"}
        rc = 3
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
