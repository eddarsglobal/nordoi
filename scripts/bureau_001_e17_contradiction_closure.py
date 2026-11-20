#!/usr/bin/env python3
"""R6F/E17: source-bound four-OS E16 report reconciliation and closure gates.

Real GitHub E16 artifacts required, pinned manifest SHA256 for each OS.
This validates archived machine statements, NOT nine external signers,
independent witnesses, a new execution of E14, or R6F certification.
Successful research audit returns BLOCK/exit 2. Failure returns exit 3.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e16_nine_machine_review_dossier as e16
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E17_CLOSURE_GATES.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E17-CONTRADICTION-CLOSURE-GATES-V1"
RESULT_SCHEMA="NORDOI-G03-R6F-E17-ACTUAL-FOUR-OS-CLOSURE-REVIEW-V1"
PARENT="dc6f9d54872a1aad899800280bb74e64eb3fe9fd"
ARTS=(
("macos-15-intel",14621492812,"a9b563c9eb525d8b3732340e59ea93b60e4e9a6ae794997b92ec9bed99535bad"),
("macos-15",14621781982,"0b676441d00d06bf7b3ab72c774a610d724a76e33970ff6bb648d4d053ebefdb"),
("ubuntu-latest",14622180298,"9211139b25e01365ac4ba0b721aa7945e1221cfdca75442264a22a7228bd6277"),
("windows-latest",14621413674,"ee8e88093ae3775ad8b496050bc50420202d73c4f3faf83e01aafeb413ce793f"),
)
OPEN={
"CROSS_OS_SKIPS_NOT_NATIVE_EQUIVALENCE":"OPEN_PORTABILITY_BOUNDARY",
"E14_PER_TARGET_PIDS_NOT_CAPTURED":"OPEN_PROVENANCE_LIMIT",
"LOCAL_SIGNATURE_NOT_INDEPENDENT_CUSTODY":"OPEN_EXTERNAL_DEPENDENCY",
"FOUR_GITHUB_ARTIFACTS_SAME_ADMIN_DOMAIN":"OPEN_EXTERNAL_DEPENDENCY",
"NINE_MACHINE_FUNCTIONS_NOT_FORMAL_REVIEWS":"OPEN_FORMAL_REVIEW",
"TWELVE_EXTERNAL_NATIVE_ADMISSIONS_UNMET":"OPEN_EXTERNAL_DEPENDENCY",
}
TECH={"HISTORIC_GIT_PINS_NOT_RUNTIME_PROOF":"RESOLVED_TECHNICAL_NOT_EXTERNAL"}
class Blocked(ValueError):pass
def require(ok,reason):
    if not ok:raise Blocked(reason)
def canonical(v):return e16.canonical(v)
def sha(raw):return e16.sha(raw)
def deny(reason,**kwargs):
    data={"result":"BLOCK","reason":reason,"formal_independent_reviews":0,
          "external_witness_receipts":0,"human_testers":0,
          "founder_manual_test_duty":False,"native_admission_gates_unmet":12,
          "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
          "production":"NOT_AUTHORIZED"}
    data.update({k:v for k,v in kwargs.items() if k not in data})
    return data

def audit(root,head):
    e16c,handoff,groups,pins=e16.audit(root,head)
    c=e3.strictly_load((root/CONTRACT).read_bytes())
    require(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
      and c["parent_e16_exact_head"]==PARENT
      and c["prior_e16"]=={
          "specialized_run_id":44720657493,"specialized_success":4,
          "rust_run_id":44720657509,"rust_success":5,
          "e8_run_id":44720657394,"e8_success":4},
      "E17_GREEN_E16_SOURCE_C20_NOT_PINNED")
    require(c["source_handoff"]==e16c["source_handoff"]
      and c["source_e16_contract"]==e16.CONTRACT
      and c["expected_archive_count"]==4
      and c["expected_machine_role_reports"]==36
      and c["expected_contradiction_records"]==28
      and c["machine_report_assurance"]==
         "FOUR_ARCHIVED_GITHUB_MACHINE_REVIEW_SETS_NOT_INDEPENDENT_AUTHORITIES",
      "E17_FOUR_OS_MACHINE_SCOPE_INVALID")
    rows=c["e16_artifacts"]
    require(len(rows)==4 and tuple(
        (r["id"],r["github_artifact_id"],r["manifest_sha256"]) for r in rows
        )==ARTS and all(r["github_artifact_name"]==
        "NORDOI_E16_NINE_REAL_MACHINE_REPORTS_"+r["id"] for r in rows),
        "E17_GITHUB_E16_MANIFEST_OR_ARCHIVE_PINS_CHANGED")
    require(set(c["expected_open_issues"])==set(OPEN)
      and len(c["expected_open_issues"])==6
      and c["expected_technical_resolved"]==
          ["HISTORIC_GIT_PINS_NOT_RUNTIME_PROOF"],
      "E17_UNSUPPORTED_CLOSURE_OF_CONTRADICTIONS")
    policy=c["closure_policy"]
    require(policy=={
       "technical_source_evidence":"SATISFIED_FOR_LIMITED_HISTORICAL_PROVENANCE",
       "formal_nine_bureau_reports":"NOT_SUBMITTED",
       "independent_owner_root":"NOT_AUTHENTICATED",
       "external_custodian":"NOT_ENROLLED",
       "global_monotonic_antirollback":"NOT_VERIFIED",
       "independent_quorum":"NOT_PROVEN",
       "outside_signed_receipts":0,
       "native_conditions_met":0,
       "native_conditions_unmet":12,
       "r6f":"IN_PROGRESS_UNCERTIFIED",
       "r6g":"LOCKED_BY_SEQUENCE",
       "production":"NOT_AUTHORIZED"},
       "E17_AUTHORITY_OR_NATIVE_CLOSURE_FABRICATED")
    require(c["technical_execution"]=="AI_AND_AUTOMATION_ONLY"
      and c["human_testers_allowed"] is False
      and c["founder_manual_test_duty"] is False
      and c["allow_self_certifying_from_nine_machine_reports"] is False
      and c["allow_same_admin_signature_as_independence"] is False
      and c["allow_merge"] is False
      and c["status_before_exact_ci"]=="IN_PROGRESS_UNQUALIFIED"
      and len(handoff["review_functions"])==9
      and all(x["review_status"]=="NOT_SUBMITTED"
              and x["formal_verdict"] is None for x in handoff["review_functions"]),
      "E17_FORMAL_REVIEW_OR_HUMAN_TESTS_MISREPRESENTED")
    return c,e16c,pins

def unsafe_naive_certification(machine_reports,green_jobs):
    """Negative control only: this invalid rule would certify machine tests."""
    return machine_reports==36 and green_jobs==9

def verify_bundle(root,head,basedir):
    c,e16c,pins=audit(root,head)
    base=Path(basedir).resolve(strict=True)
    require(base.is_dir() and
            {x.name for x in base.iterdir()}==
               {x["id"] for x in c["e16_artifacts"]},
            "E17_MISSING_EXTRA_OR_SWAPPED_PLATFORM_BUNDLE")
    verified=0;total=0;aggregates=[]
    # E16 SHA256 belongs to the runner checkout. Windows can convert text
    # blob LF to worktree CRLF while Git blob identity remains unchanged.
    # Compare only the two deterministic byte representations to the
    # original E16 Git blob, never to arbitrary reporter-supplied hashes.
    original_blobs={}
    def source_digests(path,platform):
        if path not in original_blobs:
            proc=subprocess.run(["git","show",PARENT+":"+path],cwd=root,
                capture_output=True,check=False,timeout=30)
            require(proc.returncode==0,
                    "E17_ORIGINAL_E16_GIT_BLOB_CANNOT_BE_READ:"+path)
            original_blobs[path]=proc.stdout
        blob=original_blobs[path]
        values={sha(blob)}
        if platform=="windows-latest":
            crlf=blob.replace(b"\r\n",b"\n").replace(b"\n",b"\r\n")
            values.add(sha(crlf))
        return values
    for a in c["e16_artifacts"]:
        folder=base/a["id"]
        require(folder.is_dir() and not folder.is_symlink(),
                "E17_PLATFORM_FOLDER_MISSING_OR_SYMLINK")
        raw=(folder/"MANIFEST.json").read_bytes()
        require(sha(raw)==a["manifest_sha256"],
                "E17_PRIOR_E16_MANIFEST_SHA_WRONG:"+a["id"])
        m=e3.strictly_load(raw)
        require(raw==canonical(m)
          and m["schema"]==e16.MANIFEST_SCHEMA
          and m["source_head_e16"]==PARENT
          and m["parent_e15_head"]==e16.PARENT
          and m["e15_anchor_sha256"]==e16c["e15_anchor"]["sha256"]
          and m["real_machine_processes"]==9
          and m["actual_e14_raw_logs_verified"]==228
          and m["formal_independent_specialist_reviews"]==0
          and m["external_witness_receipts"]==0
          and m["native_admission_gates_unmet"]==12
          and m["human_testers"]==0
          and m["r6g"]=="LOCKED_BY_SEQUENCE"
          and m["production"]=="NOT_AUTHORIZED",
          "E17_ARCHIVED_E16_AUTHORITY_BOUNDARY_FALSE")
        rows=m["reports"]
        require(len(rows)==9 and tuple(r["role"] for r in rows)==e16.ROLES
          and len({r["pid"] for r in rows})==9
          and {x.name for x in folder.iterdir()}==
               {"MANIFEST.json"} | {r["role"]+".json" for r in rows},
          "E17_NINE_REAL_MACHINE_REPORTS_MISSING_OR_INJECTED")
        for row in rows:
            require(row["filename"]==row["role"]+".json"
                    and type(row["pid"]) is int and row["pid"]>0,
                    "E17_BAD_ROLE_OR_PID")
            fp=folder/row["filename"]
            require(fp.is_file() and not fp.is_symlink(),
                    "E17_MISSING_ROLE_OR_SYMLINK")
            rr=fp.read_bytes()
            require(sha(rr)==row["sha256"],"E17_CHANGED_ROLE_SHA")
            role=e3.strictly_load(rr)
            require(rr==canonical(role)
              and role["schema"]==e16.REPORT_SCHEMA
              and role["role"]==row["role"]
              and role["source_head_e16"]==PARENT
              and role["parent_e15_head"]==e16.PARENT
              and role["process_pid_observed_by_launcher"]==row["pid"]
              and role["formal_review_status"]=="NOT_SUBMITTED"
              and role["formal_independent_specialist_verdict"] is None
              and role["external_administrative_independence_proven"] is False
              and role["actual_e14_artifacts_verified"]==4
              and role["actual_e14_raw_logs_verified"]==228
              and role["native_admission_gates_unmet"]==12
              and role["human_testers"]==0
              and role["r6g"]=="LOCKED_BY_SEQUENCE"
              and role["production"]=="NOT_AUTHORIZED",
              "E17_FAKE_FORMAL_REVIEW_OR_MACHINE_ROLE_CLAIM")
            files=role["source_evidence"]
            require([x["path"] for x in files]==
                e16c["role_source_targets"][row["role"]],
                "E17_ROLE_REBOUND_TO_WRONG_SOURCE")
            for f in files:
                p=f["path"]
                require(p in pins and f["source_git_blob_sha1"]==
                  pins[p]["git_blob_sha1"] and
                  f["source_sha256"] in source_digests(p,a["id"]),
                  "E17_REAL_GIT_SOURCE_NOT_MATCHED_TO_ROLE")
                checks=f["true_E14_test_records_by_platform"]
                require([v["platform"] for v in checks]==
                   [x["id"] for x in c["e16_artifacts"]]
                   and all(type(v["passed"]) is int and v["passed"]>=0
                     and type(v["skipped"]) is int and v["skipped"]>=0
                     and type(v["ran"]) is int and v["ran"]>0
                     and v["ran"]==v["passed"]+v["skipped"]
                     and bool(re.fullmatch(r"[0-9a-f]{64}",v["raw_log_sha256"]))
                     for v in checks),
                  "E17_FALSE_CASE_COUNTS_OR_LOG_REFERENCES")
            verified+=1
        matrix=m["contradictions"]
        require(len(matrix)==7 and
          tuple(x["id"] for x in matrix)==e16.ISSUES and
          all(x["status"]==(OPEN|TECH)[x["id"]]
              and type(x["observation"]) is str and x["observation"]
              and type(x["reference"]) is str and x["reference"]
              for x in matrix),
          "E17_COHERENT_SELF_REHASH_FALSE_CLOSURE_REJECTED")
        total+=len(matrix)
        aggregates.append({"os":a["id"],"archived_e16_manifest_sha256":sha(raw),
             "machine_reports_revalidated":9,
             "contradictions_unresolved":6,
             "technical_provenance_contradictions_resolved":1,
             "formal_independent_reviews":0})
    require(verified==36 and total==28,
            "E17_NOT_FOUR_SETS_NINE_REPORTS_AND_SEVEN_ISSUES")
    require(e3.check_head(root,head)==head,
            "E17_EXACT_SOURCE_HEAD_CHANGED")
    return {
        "schema":RESULT_SCHEMA,"source_head_e17":head,
        "parent_e16_head":PARENT,
        "real_archived_e16_machine_reports_revalidated":verified,
        "real_archived_e16_contradictions_revalidated":total,
        "four_ci_artifact_platforms":aggregates,
        "distinct_open_contradictions":list(OPEN),
        "distinct_technical_provenance_closures":list(TECH),
        "open_blockers":6,"technical_provenance_limit_resolved":1,
        "formal_independent_reviews":0,"outside_signed_receipts":0,
        "owner_oob_root":"NOT_AUTHENTICATED",
        "external_custodian":"NOT_ENROLLED",
        "monotonic_antirollback":"NOT_VERIFIED",
        "native_admission_gates_unmet":12,"human_testers":0,
        "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
        "production":"NOT_AUTHORIZED",
        "e17_does_not_reexecute_e14_or_e16":True,
        "e17_does_not_certify_machine_roles_as_independent":True
    }

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    act=parser.add_mutually_exclusive_group(required=True)
    act.add_argument("--audit",action="store_true")
    act.add_argument("--exercise",action="store_true")
    parser.add_argument("--expected-head",required=True)
    parser.add_argument("--root",default=str(ROOT))
    parser.add_argument("--artifact-dir")
    parser.add_argument("--result-path")
    a=parser.parse_args(argv)
    try:
        root=Path(a.root).resolve(strict=True)
        if a.audit:
            audit(root,a.expected_head)
            result=deny("E17_SOURCE_ONLY_CONTRACT_AUDIT",
                exact_head=a.expected_head,
                real_archived_machine_reports_rechecked=0)
        else:
            require(a.artifact_dir and a.result_path,
                    "E17_ACTUAL_GITHUB_E16_ARCHIVES_REQUIRED")
            output=Path(a.result_path).absolute()
            require(output.parent.is_dir() and not output.exists(),
                    "E17_NEED_FRESH_RESULT_FILE")
            doc=verify_bundle(root,a.expected_head,a.artifact_dir)
            output.write_bytes(canonical(doc))
            result=deny("E17_SIX_OPEN_BLOCKERS_NOT_CLOSED",
                exact_head=a.expected_head,
                e16_machine_reports_verified=36,
                e16_contradiction_records_verified=28,
                open_closure_conditions=6,
                technical_provenance_reconciled=1,
                output_sha256=sha(output.read_bytes()),
                naive_nine_job_claim_would_falsely_certify=
                    unsafe_naive_certification(36,9))
        code=2
    except (Blocked,e16.Blocked,e16.e15.Blocked,e16.e15.e14.Blocked,
            e16.e15.e14.e13.Blocked,e16.e15.e14.e13.e12.Blocked,
            e3.Rejected,KeyError,ValueError,TypeError,OSError) as err:
        result=deny("E17_REAL_CONTRADICTION_GATES_NOT_QUALIFIED",
           error_type=type(err).__name__,
           diagnostic=str(err)[:240] if isinstance(err,Blocked) else None)
        code=3
    sys.stdout.buffer.write(canonical(result))
    return code

if __name__=="__main__":raise SystemExit(main())
