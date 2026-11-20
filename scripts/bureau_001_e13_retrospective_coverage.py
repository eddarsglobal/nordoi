#!/usr/bin/env python3
"""NORDOI R6F/E13 — real Git tree retrospective coverage, without fake execution.

A frozen source/test-pair inventory is not a fresh test run, signed CI job,
external witness, or formal specialist review. All success paths exit 2 BLOCK.
Real source/manifest violations exit 3, without changing Rust or Git history.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e12_ai_only_native_evidence_intake as e12
import bureau_001_r1_audit as r1
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E13_RETRO_COVERAGE_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E13-RETROSPECTIVE-COVERAGE-V1"
CLAIM_SCHEMA="NORDOI-R6F-E13-UNTRUSTED-RETRO-RUN-CLAIMS-V1"
PARENT="4ce07de0ee0478ee05794303bf528d071ef07488"
RUST=re.compile(r"tests/g03_r4[a-x]_[a-z0-9_]+\.rs\Z")
PY_SOURCE=re.compile(r"research/g03_r(?:4[yz]|5[a-z]|6[a-e])_[a-z0-9_]+\.py\Z")
PY_TEST=re.compile(r"research/test_g03_r(?:4[yz]|5[a-z]|6[a-e])_[a-z0-9_]+\.py\Z")
class Blocked(ValueError):pass
def require(ok,why):
    if not ok:raise Blocked(why)

def deny(reason,**details):
    d={
        "result":"BLOCK","reason":reason,"human_testers":0,
        "founder_manual_test_duty":False,
        "independent_specialist_reviews":0,
        "independent_external_witness_receipts":0,
        "legacy_historical_execution_attestations_verified_by_e13":0,
        "native_admission_gates_unmet":12,
        "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
        "production":"NOT_AUTHORIZED"
    }
    d.update({k:v for k,v in details.items() if k not in d})
    return d

def audit(root,head):
    before=e12.audit(root,head)
    require(before["result"]=="BLOCK"
            and before["native_admission_gates_unmet"]==12,
            "E13_E12_C20_PARENT_NOT_DENIED")
    c=e3.strictly_load(r1.raw(root,CONTRACT))
    require(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
            and c["parent_e12_exact_head"]==PARENT and
            c["prior_e12"]=={
                "specialized_run_id":44080133471,
                "specialized_success":4,
                "rust_run_id":44080133579,
                "rust_success":5,
                "rust_mac_cancelled_original_job_id":134028240681,
                "rust_mac_successful_rerun_job_id":136187446167},
            "E13_PARENT_E12_C20_RUN_PROVENANCE_WRONG")
    require(c["pinned_baseline_source"]==r1.MANIFEST and
            c["handoff"]=="governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json" and
            c["expected_pinned_git_blobs"]==99 and
            c["expected_historical_rust_test_files"]==24 and
            c["expected_historical_python_source_test_pairs"]==33 and
            c["expected_historical_snapshots"]==4 and
            c["expected_historical_governance_basis_files"]==5 and
            tuple(c["expected_role_functions"])==tuple(e12.ROLES),
            "E13_HISTORIC_COVERAGE_COUNTS_NOT_PINNED")
    p=c["proof_classes"]
    require(p=={
        "source_blob_identity":"GIT_TREE_AND_WORKTREE_PIN_VERIFIED_NOT_RUNTIME_EXECUTION",
        "prior_owner_terminal_result":"OWNER_REPORTED_NOT_INDEPENDENTLY_REPLAYED_BY_E13",
        "prior_ci_run_status":"PRIOR_CI_OBSERVATION_NOT_FULL_HISTORICAL_TEST_EXECUTION_RECEIPT",
        "local_role_function":"MACHINE_FUNCTION_NOT_FORMAL_INDEPENDENT_REVIEW",
        "legacy_r4a_r6e_runtime_coverage":"NOT_PROVEN_BY_SOURCE_ONLY_INVENTORY"},
        "E13_FAKE_HISTORICAL_RUNTIME_OR_EXTERNAL_AUTHORITY")
    require(c["human_testers_allowed"] is False
            and c["founder_manual_test_duty"] is False
            and c["execution"]=="AI_AND_AUTOMATION_ONLY"
            and c["actual_external_witness_receipts"]==0
            and c["actual_independent_specialist_reviews"]==0
            and c["external_custodian"]=="NOT_ENROLLED"
            and c["owner_oob_root"]=="NOT_AUTHENTICATED"
            and c["global_monotonic_antirollback"]=="NOT_VERIFIED"
            and c["native_admission_gates_unmet"]==12
            and c["r6f"]=="IN_PROGRESS_UNCERTIFIED"
            and c["r6g"]=="LOCKED_BY_SEQUENCE"
            and c["production"]=="NOT_AUTHORIZED"
            and c["permission_to_merge"] is False
            and c["current_result"]=="IN_PROGRESS_UNQUALIFIED",
            "E13_ILLEGAL_CERTIFICATION_PROMOTION")
    r1.audit(root)
    h=e3.strictly_load(r1.raw(root,c["handoff"]))
    roles=h["review_functions"]
    require(len(roles)==9
        and tuple(x["role"] for x in roles)==tuple(e12.ROLES)
        and all(x["review_status"]=="NOT_SUBMITTED"
                and x["actual_execution_log"] is None
                and x["formal_verdict"] is None for x in roles)
        and h["next_milestone_state"]=="LOCKED_BY_SEQUENCE",
        "E13_BUREAU_FORMAL_REVIEW_EXECUTION_INVENTED")
    require(e3.check_head(root,head)==head,
            "E13_WRONG_EXACT_PR_HEAD")
    return deny("E13_RETRO_COVERAGE_AUDIT_SOURCE_ONLY",
                exact_source_head=head,source_pins_reported=99,
                formal_independent_roles_submitted=0)

def categories(pins):
    paths=set(pins)
    rust=sorted(p for p in paths if RUST.fullmatch(p))
    src=sorted(p for p in paths if PY_SOURCE.fullmatch(p))
    tests=sorted(p for p in paths if PY_TEST.fullmatch(p))
    snaps=sorted(set(r1.SNAPSHOTS)&paths)
    basis=sorted(set(r1.BUREAU_OLD)&paths)
    require(len(rust)==24 and len(src)==len(tests)==33
            and len(snaps)==4 and len(basis)==5
            and {p.replace("research/g03_","research/test_g03_",1)
                 for p in src}==set(tests),
            "E13_SOURCE_TEST_PAIRS_OR_RUST_SCOPE_INCOMPLETE")
    require(len(set(rust+src+tests+snaps+basis))==99
            and set(rust+src+tests+snaps+basis)==paths,
            "E13_MISSING_OR_DUPLICATED_LEGACY_SCOPE")
    return {"rust":rust,"python_source":src,"python_test":tests,
            "snapshots":snaps,"basis":basis}

def true_git_tree(root,head):
    require(e3.check_head(root,head)==head,
            "E13_WORKTREE_NOT_EXACT_PR_HEAD")
    p=subprocess.run(["git","ls-tree","-rz",head],
           cwd=root,capture_output=True,timeout=60)
    require(p.returncode==0,"E13_REAL_GIT_TREE_COMMAND_FAILED")
    entries={}
    for raw in p.stdout.split(b"\0"):
        if not raw:continue
        try:
            meta,path=raw.split(b"\t",1)
            mode,typ,sha=meta.decode("ascii").split(" ")
            rel=path.decode("utf-8")
        except (UnicodeError,ValueError) as exc:
            raise Blocked("E13_UNEXPECTED_REAL_GIT_TREE_FORMAT") from exc
        require(rel not in entries,"E13_DUPLICATE_GIT_TREE_ENTRY")
        entries[rel]=(mode,typ,sha)
    return entries

def verify_sources(root,head):
    audit(root,head)
    pins=r1.verify_manifest(root,r1.read_json(root,r1.MANIFEST))
    groups=categories(pins)
    tree=true_git_tree(root,head)
    for path,record in pins.items():
        require(path in tree,"E13_FROZEN_SOURCE_NOT_TRACKED:"+path)
        mode,kind,gitsha=tree[path]
        require(mode in ("100644","100755") and kind=="blob" and
                gitsha==record["git_blob_sha1"],
                "E13_GIT_TREE_BLOB_DIFFERS_FROM_FROZEN_MANIFEST:"+path)
    return groups,pins

def optimistic_claim(head,groups):
    return {
      "schema":CLAIM_SCHEMA,"project":"eddarsglobal/nordoi",
      "source_head":head,
      "asserted_rust_executed_pass":groups["rust"],
      "asserted_python_executed_pass":groups["python_test"],
      "self_declared_independent_bureau_reviews":9,
      "self_declared_external_custodian":True,
      "self_declared_twelve_native_gates_met":True
    }

def naive_accept_claim(claim):
    return (len(claim["asserted_rust_executed_pass"])==24
            and len(claim["asserted_python_executed_pass"])==33
            and claim["self_declared_independent_bureau_reviews"]==9
            and claim["self_declared_external_custodian"] is True
            and claim["self_declared_twelve_native_gates_met"] is True)

def strict_claim_intake(head,raw,groups):
    c=e3.strictly_load(raw)
    require(set(c)=={
        "schema","project","source_head",
        "asserted_rust_executed_pass","asserted_python_executed_pass",
        "self_declared_independent_bureau_reviews",
        "self_declared_external_custodian",
        "self_declared_twelve_native_gates_met"},
        "E13_UNTRUSTED_COVERAGE_CLAIM_UNKNOWN_FIELDS")
    require(c["schema"]==CLAIM_SCHEMA and
            c["project"]=="eddarsglobal/nordoi" and
            c["source_head"]==head,
            "E13_UNTRUSTED_COVERAGE_CLAIM_WRONG_SOURCE")
    require(type(c["asserted_rust_executed_pass"]) is list
        and type(c["asserted_python_executed_pass"]) is list
        and c["asserted_rust_executed_pass"]==groups["rust"]
        and c["asserted_python_executed_pass"]==groups["python_test"],
        "E13_INCOMPLETE_DUPLICATED_OR_INJECTED_LEGACY_FILES")
    require(type(c["self_declared_independent_bureau_reviews"]) is int
            and type(c["self_declared_external_custodian"]) is bool
            and type(c["self_declared_twelve_native_gates_met"]) is bool,
            "E13_INVALID_UNTRUSTED_AUTHORITY_FIELDS")
    return deny("E13_SOURCE_PINS_ARE_NOT_HISTORIC_RUNTIME_EXECUTION_RECEIPTS",
         exact_source_head=head,
         real_git_tree_and_worktree_source_blobs_verified=99,
         historical_rust_test_files_found=24,
         historical_python_test_pairs_found=33,
         untrusted_claimed_historical_test_file_executions=57,
         naive_coverage_laundering_would_promote=naive_accept_claim(c),
         actual_historical_runtime_evidence_replayed_by_e13=False,
         zero_independent_review_claims_verified=True,
         actual_external_trust_verified=False)

def exercise(root,head):
    groups,pins=verify_sources(root,head)
    claim=optimistic_claim(head,groups)
    require(naive_accept_claim(claim),"E13_UNTRUSTED_NAIVE_ATTACK_NOT_REPRODUCED")
    clean=strict_claim_intake(head,e12.e11.e10.canonical(claim),groups)
    require(clean["result"]=="BLOCK" and
            clean["naive_coverage_laundering_would_promote"] is True and
            clean["legacy_historical_execution_attestations_verified_by_e13"]==0,
            "E13_SOURCE_PASS_WRONGLY_CALLED_RUNTIME_PASS")
    fake=dict(claim)
    fake["asserted_rust_executed_pass"]=claim["asserted_rust_executed_pass"][:-1]
    refused=False
    try:strict_claim_intake(head,e12.e11.e10.canonical(fake),groups)
    except Blocked as e:
        refused="INCOMPLETE_DUPLICATED_OR_INJECTED" in str(e)
    require(refused,"E13_WITHHELD_RUST_COVERAGE_NOT_CAUGHT")
    return deny("E13_REAL_99_GIT_BLOBS_AND_57_TEST_FILES_SOURCE_ONLY",
         exact_source_head=head,
         actual_git_tree_blobs_compared_to_frozen_manifest=len(pins),
         actual_historical_rust_source_test_files=24,
         actual_historical_python_source_test_pairs=33,
         actual_r6b_r6e_snapshot_blobs=4,
         actual_historical_bureau_basis_blobs=5,
         forged_self_asserted_all_tests_pass=57,
         false_positive_naive_coverage_reproduced=True,
         strict_coverage_guard_rejected_execution_laundering=True,
         missing_rust_test_file_claim_rejected=True,
         actual_history_tests_reexecuted_by_this_program=0,
         no_claim_of_absence_of_prior_ci_tests=True)

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    actions=ap.add_mutually_exclusive_group(required=True)
    actions.add_argument("--audit",action="store_true")
    actions.add_argument("--exercise",action="store_true")
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--root",default=str(ROOT))
    a=ap.parse_args(argv)
    try:
        root=Path(a.root).resolve(strict=True)
        d=audit(root,a.expected_head) if a.audit else exercise(root,a.expected_head)
        code=2
    except (Blocked,e12.Blocked,e12.e11.Blocked,e12.e11.e10.Blocked,
            e3.Rejected,r1.Blocked,ValueError,KeyError,TypeError,OSError,
            subprocess.SubprocessError) as ex:
        d=deny("E13_RETRO_COVERAGE_EXPERIMENT_NOT_QUALIFIED",
               error_type=type(ex).__name__)
        code=3
    sys.stdout.buffer.write(e12.e11.e10.canonical(d))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":raise SystemExit(main())
