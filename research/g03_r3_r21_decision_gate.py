#!/usr/bin/env python3
"""NORDOI G0.3 R3-R21 — a finite, pinned engineering decision gate.

R3-R21 is a documented recommendation, NEVER merge authorization.
No benchmarks, source edits, subprocesses, network calls or production actions.
CI checks manifest policy and negative test cases; the exact owner Mac R20
report can be verified only if its real bytes are supplied locally.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

SCHEMA = "NORDOI-G03-R3-R21-ENGINEERING-DECISION-v1"
R20_SCHEMA = "NORDOI-G03-R3-R20-INTEGRATION-READINESS-v1"
BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CAND_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
BASE_KERNEL = "dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2"
CAND_KERNEL = "35194549eee9d6798de2c9afb091a668c2d491a40c548693ee6e87a3aa29aba0"
OWNER_R20_SHA256 = "b2aa841c9109cd23be5967fa534426cdf05541b9665c91240db752bbb17057d5"
DECISION = "ACCEPT_WITH_CONTROLS_FOR_OWNER_MERGE_DECISION"
R20_DECISION = "READY_FOR_GOVERNED_MERGE_DECISION_WITH_PERFORMANCE_EXCEPTION"
ADDED = {
  ".github/workflows/g03_r3_r13_paired.yml",
  "research/G03_R3_R13_ENGINEERING.md",
  "research/g03_r3_r13_kernel_probe.rs",
  "research/g03_r3_r13_paired_mac.py",
  "tests/g03_r3_r13_redundant_lookup.rs",
}

class Blocked(Exception):
    pass

def need(predicate, reason):
    if not predicate:
        raise Blocked(reason)

def read_json(path, max_size):
    p=Path(path)
    need(p.is_file() and not p.is_symlink(),"missing or linked evidence file")
    stat=p.stat()
    need(0 < stat.st_size <= max_size and stat.st_nlink==1,
        "oversized, empty or hard-linked evidence")
    raw=p.read_bytes()
    try:
        data=json.loads(raw)
    except (ValueError,UnicodeError) as exc:
        raise Blocked("invalid JSON evidence") from exc
    need(isinstance(data,dict),"JSON root not object")
    return data,raw

def validate_manifest(m):
    need(m.get("schema")==SCHEMA,"decision schema mismatch")
    need(m.get("scope")=="ENGINEERING_RECOMMENDATION_ONLY",
         "scope is not engineering-only")
    need(m.get("decision")==DECISION,"unexpected engineering recommendation")
    need(m["baseline"]["sha"]==BASE_SHA and
         m["baseline"]["kernel_sha256"]==BASE_KERNEL,
         "base source digest drift")
    need(m["candidate"]["sha"]==CAND_SHA and
         m["candidate"]["kernel_sha256"]==CAND_KERNEL and
         m["candidate"]["pr"]==40 and
         m["candidate"]["pr_state_at_review"]=="DRAFT_OPEN_UNMERGED",
         "candidate or PR source status drift")
    mac=m["r20_owner_mac"]
    need(mac["report_sha256"]==OWNER_R20_SHA256 and
         mac["schema"]==R20_SCHEMA and mac["decision"]==R20_DECISION,
         "R20 Mac evidence reference drift")
    need(mac["original_tests_passed"]==2496 and
         mac["candidate_tests_passed"]==2499 and
         mac["security_contract"]=="PASS_BOTH",
         "R20 test results unsupported")
    need(mac["tracked_common_files_identical"]==659 and
         mac["candidate_added_paths"]==5 and
         mac["candidate_removed_paths"]==0 and
         mac["single_source_delta"]=="EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY",
         "R20 exact source delta not preserved")
    ci=m["r20_github_ci"]
    need(ci["governed_readiness_run"]==42475969873 and
         ci["general_gate_run"]==42475970206 and
         ci["conclusion_at_review"]=="SUCCESS" and
         set(ci["governed_matrix"])==
         {"ubuntu-latest","macos-latest","windows-latest"},
         "R20 cross-OS CI provenance mismatch")
    perf=m["historical_mac_intel_performance"]
    need(perf["origin"]=="R3-R16 FIX1, not rerun in R3-R20/R3-R21" and
         perf["noop_set_gain_percent"]==23.1812 and
         perf["changed_set_flush_gain_percent"]==3.9061 and
         perf["denied_capability_gain_percent"]==-1.3758 and
         perf["denied_capability_delta_ns_per_op"]==-0.17 and
         perf["denied_capability_candidate_faster_blocks"]=="2/12" and
         perf["denied_capability_exception"]=="OPEN_EXPLICIT" and
         perf["latency_root_cause"]=="NOT_ESTABLISHED",
         "historical performance exception missing or falsely resolved")
    code=m["codegen_review"]
    need(all(code.get(k) is True for k in (
         "require_textually_equivalent_after_local_branch_rebase",
         "witness_textually_equivalent_after_local_branch_rebase",
         "contains_textually_equivalent_after_memcmp_stub_relocation")) and
         code["machine_byte_identity"]=="NOT_ESTABLISHED" and
         code["layout_causes_slowdown"]=="NOT_PROVEN",
         "assembly outcome exaggerated or changed")
    gov=m["governance"]
    need(gov["public_language_changes"]=="NONE" and
         gov["permission_policy_changes"]=="NONE" and
         gov["kernel_changes_by_r21"]=="NONE" and
         gov["merge_authorization"]=="NONE" and
         gov["owner_merge_approval"]=="NOT_GIVEN" and
         gov["production"]=="NOT_AUTHORIZED" and
         gov["external_human_evaluator_or_signature"]=="NOT_REQUIRED" and
         gov["owner_action"]=="EXPLICIT_SEPARATE_AUTHORIZATION_REQUIRED_FOR_ANY_MERGE" and
         gov["pr40_allowed_only_if_heads_still_pinned"] is True,
         "governance authorization guard failed")
    risk=m["residual_risk"]
    need(risk["classification"]==
         "MEASURED_MINOR_DENIED_CAPABILITY_PERFORMANCE_REGRESSION" and
         risk["assessment"]==
         "ACCEPTABLE_FOR_GOVERNED_REVIEW_NOT_AUTOMATIC_MERGE" and
         risk["deferred_owner_tradeoff_approval"] is True and
         "revalidate" in risk["mitigation"] and "revert" in risk["mitigation"],
         "residual performance risk or mitigation suppressed")
    return True

def validate_r20(d,m):
    need(d["schema"]==R20_SCHEMA and d["decision"]==R20_DECISION,
         "R20 evidence schema/decision mismatch")
    need(d["frozen_base_sha"]==BASE_SHA and
         d["frozen_candidate_sha"]==CAND_SHA and
         d["r13_candidate_pr"]==40,
         "R20 source identities not pinned")
    guard=d["source_guard"]
    need(guard["source_delta"]=="EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY" and
         guard["baseline_sha256"]==BASE_KERNEL and
         guard["candidate_sha256"]==CAND_KERNEL,
         "R20 kernel patch evidence failed")
    inventory=d["tracked_inventory"]
    need(inventory["all_other_common_files_byte_identical"] is True and
         inventory["common_tracked_files"]==659 and
         inventory["candidate_added_files"]==5 and
         set(inventory["only_candidate_added_files"])==ADDED and
         inventory["removed_files"]==[],
         "R20 unapproved extra/deleted tracked paths")
    for role,count in (("baseline",2496),("candidate",2499)):
        v=d["verification"][role]
        need(v["cargo_fmt"]=="PASS" and
             v["cargo_test_release_all_targets"]=="PASS" and
             v["cargo_clippy_all_targets_deny_warnings"]=="PASS" and
             v["external_security_contract"]=="PASS" and
             v["cargo_test_result"]["failed"]==0 and
             v["cargo_test_result"]["passed"]==count and
             v["real_speed_measurements"]=="NONE",
             "R20 real Rust/Clippy/permission gate incomplete: "+role)
    perf=d["historical_performance_evidence"]
    source=m["historical_mac_intel_performance"]
    need(perf["noop_set_ab_percent"]==source["noop_set_gain_percent"] and
         perf["changed_set_flush_ab_percent"]==source["changed_set_flush_gain_percent"] and
         perf["denied_capability_ab_percent"]==source["denied_capability_gain_percent"] and
         perf["denied_capability_ab_delta_ns_per_op"]==source["denied_capability_delta_ns_per_op"] and
         perf["denied_capability_root_cause"]=="NOT_ESTABLISHED" and
         perf["disposition"]=="EXPLICIT_PERFORMANCE_EXCEPTION_REQUIRES_ENGINEERING_DECISION",
         "R20 historical slowdown wrongly removed")
    need(d["no_new_performance_measurements"] is True and
         d["human_evaluation_gate"]=="CLOSED_NOT_REQUIRED" and
         d["merge_authorization"]=="NONE" and
         d["production"]=="NOT_AUTHORIZED" and
         d["original_repo_modified"] is False,
         "R20 governance/provenance changed")
    return True

def private_receipt(path,value):
    p=Path(path)
    need(not p.exists() and not p.is_symlink() and p.parent.is_dir(),
         "private output destination unsafe")
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL
    if hasattr(os,"O_NOFOLLOW"):flags|=os.O_NOFOLLOW
    fd=os.open(p,flags,0o600)
    with os.fdopen(fd,"w",encoding="utf-8",newline="\n") as f:
        json.dump(value,f,sort_keys=True,indent=2)
        f.write("\n")
    if os.name=="posix":
        need((p.stat().st_mode & 0o077)==0,"receipt permissions not private")

def selftest(manifest):
    assert validate_manifest(manifest)
    import copy
    examples=[
        ("denied_capability_exception","CLOSED"),
        ("latency_root_cause","PROVED_NOT_SLOW"),
        ("denied_capability_gain_percent",0.0)
    ]
    for field,value in examples:
        m=copy.deepcopy(manifest)
        m["historical_mac_intel_performance"][field]=value
        try:validate_manifest(m)
        except Blocked:pass
        else:raise AssertionError("altered performance exception accepted: "+field)
    for field,value in (("merge_authorization","ALLOW"),("production","AUTHORIZED"),
                        ("owner_merge_approval","GIVEN")):
        m=copy.deepcopy(manifest)
        m["governance"][field]=value
        try:validate_manifest(m)
        except Blocked:pass
        else:raise AssertionError("unauthorized governance mutation accepted")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R321_NEGATIVE_") as root:
        p=Path(root)/"report.json"
        private_receipt(p,{"synthetic":True})
        try:private_receipt(p,{})
        except Blocked:pass
        else:raise AssertionError("duplicate evidence overwrite was allowed")
    print("G03_R3_R21_GOVERNANCE_NEGATIVE_SELFTEST=PASS")
    print("OWNER_R20_PRIVATE_REPORT=NOT_REPLAYED_IN_SELFTEST")
    print("MERGE_AUTHORIZATION=NONE")
    print("PRODUCTION=NOT_AUTHORIZED")

def verify_owner(manifest,report_file,out):
    report,raw=read_json(report_file,500000)
    need(hashlib.sha256(raw).hexdigest()==OWNER_R20_SHA256,
         "actual owner Mac R20 report SHA256 differs")
    validate_r20(report,manifest)
    receipt={
      "schema":"NORDOI-G03-R3-R21-OWNER-DECISION-CUSTODY-v1",
      "r20_owner_report_sha256":OWNER_R20_SHA256,
      "source_pin_status":"PASS",
      "test_custody_status":"PASS_REAL_R20_MAC_EVIDENCE",
      "historical_performance_exception":"OPEN_EXPLICIT",
      "engineering_recommendation":DECISION,
      "merge_authorization":"NONE",
      "owner_merge_approval":"NOT_GIVEN",
      "production":"NOT_AUTHORIZED",
      "original_repo_modified":False,
    }
    if out:private_receipt(out,receipt)
    print("G03_R3_R21_OWNER_MAC_R20_EVIDENCE=PASS")
    print("SOURCE_AND_SECURITY_EVIDENCE=PASS")
    print("DENIED_CAPABILITY_PERFORMANCE_EXCEPTION=OPEN")
    print("ENGINEERING_DECISION="+DECISION)
    print("MERGE_AUTHORIZATION=NONE")
    print("OWNER_MERGE_APPROVAL=NOT_GIVEN")
    print("PRODUCTION=NOT_AUTHORIZED")
    print("PRIVATE_RECEIPT="+str(out) if out else "PRIVATE_RECEIPT=NONE")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    s=p.add_subparsers(dest="action",required=True)
    a=s.add_parser("selftest")
    a.add_argument("--manifest",required=True)
    b=s.add_parser("verify-owner")
    b.add_argument("--manifest",required=True)
    b.add_argument("--r20-report",required=True)
    b.add_argument("--out")
    args=p.parse_args()
    try:
        manifest,_=read_json(args.manifest,24000)
        validate_manifest(manifest)
        if args.action=="selftest":selftest(manifest)
        else:verify_owner(manifest,args.r20_report,args.out)
        return 0
    except (Blocked,OSError,ValueError,KeyError,TypeError,AttributeError,
            AssertionError,UnicodeError) as exc:
        print("G03_R3_R21_GATE=BLOCKED",file=sys.stderr)
        print("REASON="+str(exc)[:230],file=sys.stderr)
        print("MERGE_AUTHORIZATION=NONE",file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
