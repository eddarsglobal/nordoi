#!/usr/bin/env python3
"""GOV-FIX1 APP research RoadMap extension; legacy 99 pinned blobs unchanged.

First invoke the exact historical RoadMapGuardian. This supplement validates
six exact Git C20 application research source identities and documentation,
without changing any source covered by BUREAU001-R1 frozen manifest.

No local register can turn CI observations into formal Bureau certification,
independent agents, independent external custody or native admission.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
from scripts import roadmap_guardian as legacy

PLAN = legacy.PLAN
HANDOFF = legacy.HANDOFF
SOURCE = legacy.SOURCE
ROLE_IDS = legacy.ROLE_IDS
Blocked = legacy.Blocked
require = legacy.require
read = legacy.read
read_json = legacy.read_json
EXTENSION = "governance/NORDOI_APP_RESEARCH_ROADMAP_V1.json"
ROADMAP_DOC = "RoadMap_APP_RESEARCH.md"
BUREAU_DOC = "Bureau_APP_RESEARCH.md"

FROZEN = (
    ("RoadMap.md", "7929805f3c99304be37ceacbe994631e93c90cae", "protected_roadmap_blob"),
    ("Bureau.md", "ffbb7b96bf978f481c4c54d42205fa262acc5e26", "protected_bureau_blob"),
    ("governance/NORDOI_BUREAU_PLAN_v1.json", "4a7a862084b59a87aa45474e921dbb7502ff86c1", "protected_plan_blob"),
    ("scripts/roadmap_guardian.py", "e7d6969b63c554fdf7ccd23c6b9fb6a9d1c9a715", "protected_guardian_blob"),
)

TRACK = "governance/NORDOI_APP_RESEARCH_TRACK_V1.json"
APP_CHAIN = (
    ("APP-016", "e988b214c8daa2b37c6abc36e351ce988bea7880", "f8455e602076ffabc49abf4f123cb8d1e70bc639", 168),
    ("APP-017", "c048f1e17590b8d863d38a067fe6ac00d78e61c2", "e988b214c8daa2b37c6abc36e351ce988bea7880", 169),
    ("APP-018", "0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad", "c048f1e17590b8d863d38a067fe6ac00d78e61c2", 170),
    ("APP-019", "fa89752c1a62f3577134017d6a90b62d46a25cf7", "0aa1fe01fe9391ed67a0fa20f8af7280af56a1ad", 171),
    ("APP-020", "6f0f5625e282b4cd5a9ad5d527087502f5842f71", "fa89752c1a62f3577134017d6a90b62d46a25cf7", 172),
    ("APP-021", "6e9c300922348d0ebab4318c7f9c2b4dd4cd76b4", "6f0f5625e282b4cd5a9ad5d527087502f5842f71", 173),
)

def audit_app_track(root, plan, roadmap, bureau):
    """Reject any unproved APP chain promotion or retroactive invented Bureau."""
    track = read_json(root, TRACK)
    p = read_json(root, EXTENSION)
    require(type(p) is dict and p.get("schema") ==
            "NORDOI_APP_RESEARCH_ROADMAP_EXTENSION_V1" and
            p.get("register") == TRACK,
            "APP_RESEARCH_OFFICIAL_PLAN_POINTER_MISSING")
    ids = [entry[0] for entry in APP_CHAIN]
    require(p.get("ids_in_strict_c20_order") == ids and
            p.get("reference_predecessor_head") == APP_CHAIN[0][2] and
            p.get("qualified_recent_head") == APP_CHAIN[-1][1] and
            p.get("observed_history") == "SIX_DRAFT_PRS_WITH_13_OF_13_EXACT_HEAD_CI" and
            p.get("app_track_technical_verdict") == "TECHNICAL_VALIDATED_RESEARCH_ONLY" and
            p.get("app_track_formal_bureau_certification") == "NOT_CERTIFIED" and
            p.get("historical_bureau_reviews") == "NOT_SUBMITTED" and
            p.get("app_next_candidate") == "APP-022" and
            p.get("app_next_status") == "LOCKED_BY_GOVERNANCE_REVIEW" and
            p.get("governance_correction") == "GOV-FIX1_IN_PROGRESS" and
            p.get("governance_correction_pr") == "DRAFT_PENDING" and
            p.get("native_admission") == "DENIED" and
            p.get("production") == "NOT_AUTHORIZED",
            "APP_RESEARCH_PLAN_PROMOTION_OR_REORDER")
    require(type(track) is dict and
            track.get("schema") == "NORDOI_APP_RESEARCH_GOVERNANCE_RECONCILIATION_V1" and
            track.get("project") == "eddarsglobal/nordoi" and
            track.get("source_of_truth") == EXTENSION and
            track.get("scope") == "NONPRIVILEGED_AGENT_RESEARCH_ONLY" and
            track.get("audit_date") == "2026-11-23" and
            track.get("chronology_rule") == "C20_EXACT_COMMIT_PARENT_AND_ALL_CHECKS_SUCCESS_AT_HEAD",
            "APP_RESEARCH_REGISTER_SCHEMA_INVALID")
    base = track.get("reference_baseline", {})
    require(base.get("milestone") == "CERT-015-R1" and
            base.get("head") == APP_CHAIN[0][2] and
            base.get("state") == "RESEARCH_ONLY",
            "APP_RESEARCH_BASELINE_INCORRECT")
    require(track.get("bureau_roles") == list(ROLE_IDS),
            "APP_RESEARCH_BUREAU_FUNCTIONS_REORDERED")
    semantics = track.get("bureau_semantics", {})
    require(semantics.get("role_report_requirement") ==
            "EXACT_SOURCE_PIN_EXECUTION_LOG_ADVERSARIAL_EVIDENCE_FORMAL_VERDICT" and
            semantics.get("machine_function_checks") ==
            "SEPARATE_DETERMINISTIC_AUTOMATED_CHECKS_NOT_INDEPENDENT_AI_AGENTS" and
            semantics.get("independent_agent_reports") == "NOT_SUBMITTED" and
            semantics.get("formal_certification") == "NOT_CERTIFIED" and
            semantics.get("previous_pr_comments") ==
            "SINGLE_COORDINATOR_TECHNICAL_SUMMARIES_NOT_FORMAL_BUREAU_SIGNOFF",
            "APP_RESEARCH_FAKE_BUREAU_CERTIFICATION")
    require(track.get("observed_checks_source") ==
            "GITHUB_CHECK_RUNS_EXACT_HEAD_VERIFIED_ON_2026_11_23_NOT_AN_INDEPENDENT_CUSTODIAN",
            "APP_RESEARCH_CI_ATTESTATION_OVERCLAIMED")
    records = track.get("milestones")
    require(type(records) is list and len(records) == len(APP_CHAIN),
            "APP_RESEARCH_MILESTONE_COUNT_INVALID")
    for i, ((id_, head, parent, pr), item) in enumerate(zip(APP_CHAIN, records)):
        require(type(item) is dict and
                (item.get("id"), item.get("head_exact_sha"),
                 item.get("parent_exact_sha"), item.get("pr_number")) ==
                (id_, head, parent, pr),
                "APP_RESEARCH_C20_EXACT_CHAIN_INVALID:" + id_)
        require(type(item.get("objective")) is str
                and len(item["objective"]) > 15,
                "APP_RESEARCH_OBJECTIVE_UNDEFINED:" + id_)
        require(item.get("current_technical_status") ==
                "TECHNICAL_VALIDATED_RESEARCH_ONLY" and
                item.get("pr_state") == "DRAFT_OPEN_UNMERGED" and
                type(item.get("exact_head_github_success")) is int and
                item.get("exact_head_github_success") == 13 and
                type(item.get("exact_head_github_total")) is int and
                item.get("exact_head_github_total") == 13 and
                item.get("independent_bureau_review") == "NOT_SUBMITTED" and
                item.get("formal_bureau_certification") == "NOT_CERTIFIED" and
                item.get("external_security_certification") == "NOT_PROVEN" and
                item.get("production") == "NOT_AUTHORIZED",
                "APP_RESEARCH_UNSUPPORTED_PROMOTION:" + id_)
        require(id_ in roadmap and ("#"+str(pr)) in roadmap and
                head in roadmap, "APP_RESEARCH_ROADMAP_NOT_MATCHED:" + id_)
        if i > 0:
            require(parent == APP_CHAIN[i-1][1],
                    "APP_RESEARCH_PARENT_LINK_BROKEN:" + id_)
    nxt = track.get("next_candidate", {})
    require(nxt.get("id") == "APP-022" and
            nxt.get("state") == "LOCKED_BY_GOVERNANCE_REVIEW" and
            nxt.get("no_external_custody_claim") is True,
            "APP022_UNLOCKED_WITHOUT_GOVERNANCE")
    protected = track.get("protected_baseline", {})
    require(protected.get("r6f") == "IN_PROGRESS_UNCERTIFIED" and
            protected.get("r6g") == "LOCKED_BY_SEQUENCE" and
            protected.get("native_gates_remaining") == 12 and
            protected.get("global_monotonic_antirollback") == "NOT_VERIFIED" and
            protected.get("release") == "NOT_AUTHORIZED" and
            protected.get("merge") == "NONE",
            "APP_RESEARCH_PROTECTED_GATES_CHANGED")
    require("## 7. Branche applicative IA" in roadmap and
            "FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED" in bureau and
            "APP-022" in roadmap and "LOCKED_BY_GOVERNANCE_REVIEW" in roadmap,
            "APP_RESEARCH_GOVERNANCE_DOCS_MISSING")
    return {"APP_TRACK": "TECHNICAL_VALIDATED_RESEARCH_ONLY",
            "APP_MILESTONES": len(records),
            "APP_C20_PARENT_CHAIN": "EXACT_PINNED_METADATA",
            "APP_EXACT_HEAD_CI_OBSERVATIONS": 13*len(records),
            "APP_BUREAU_FORMAL_CERTIFICATION": "NOT_CERTIFIED",
            "APP_NEXT": "LOCKED_BY_GOVERNANCE_REVIEW",
            "APP_MACHINE_REVIEW_IS_INDEPENDENT_AGENT": False}



def frozen_blobs(root):
    ext = read_json(root, EXTENSION)
    for path, expected, field in FROZEN:
        raw = (Path(root)/path).read_bytes()
        if os.name == "nt":
            raw = raw.replace(b"\r\n", b"\n")
        digest = hashlib.sha1(b"blob "+str(len(raw)).encode("ascii")
                              +b"\x00"+raw).hexdigest()
        require(digest == expected and ext.get(field) == expected,
                "FROZEN_G03_SOURCE_MUST_STAY_BYTE_IDENTICAL:"+path)
    require(ext.get("protected_historical_roadmap") == "RoadMap.md"
            and ext.get("protected_historical_bureau") == "Bureau.md"
            and ext.get("protected_historical_plan") == PLAN
            and ext.get("historical_immutability") ==
            "REQUIRED_BY_BUREAU001_R1_FROZEN_GIT_BLOB_MANIFEST"
            and ext.get("extension_qualifies_as") ==
            "SEPARATE_VERSIONED_RESEARCH_ROADMAP_NOT_RETROACTIVE_EDIT"
            and ext.get("project") == "eddarsglobal/nordoi",
            "APP_EXTENSION_FROZEN_BASIS_INVALID")


def audit(root):
    root = Path(root).resolve()
    legacy_verdict = legacy.audit(root)
    require(legacy_verdict["BUREAU_GUARDIAN"] == "PASS_WITH_CONTROLS"
            and legacy_verdict["REVIEWS_SUBMITTED"] == 0
            and legacy_verdict["BUREAU_CERTIFICATION"] == "PENDING"
            and legacy_verdict["R6G"] == "LOCKED_BY_SEQUENCE",
            "LEGACY_BUREAU_NO_LONGER_FAIL_CLOSED")
    frozen_blobs(root)
    track = audit_app_track(
        root, read_json(root, PLAN),
        read(root, ROADMAP_DOC), read(root, BUREAU_DOC))
    return dict(legacy_verdict, **track,
                APP_ROADMAP_EXTENSION="FROZEN_HISTORICAL_BLOBS_PRESERVED",
                INDEPENDENT_REVIEW_REPORTS="NOT_SUBMITTED")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path,
                        default=Path(__file__).resolve().parents[1])
    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("--audit",action="store_true")
    actions.add_argument("--attempt-promote",
                         choices=("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"))
    args = parser.parse_args(argv)
    try:
        verdict = audit(args.root)
    except (Blocked,OSError,TypeError,ValueError,KeyError,AttributeError) as e:
        print(json.dumps({"APP_RESEARCH_GUARDIAN":"BLOCK",
                          "reason":str(e) if isinstance(e,Blocked)
                          else type(e).__name__},sort_keys=True))
        return 2
    if args.attempt_promote:
        print(json.dumps(legacy.deny_transition(args.attempt_promote),
                         sort_keys=True))
        return 2
    print(json.dumps(verdict,sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
