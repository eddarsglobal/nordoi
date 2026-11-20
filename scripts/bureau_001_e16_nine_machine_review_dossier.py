#!/usr/bin/env python3
"""NORDOI R6F/BUREAU-001 E16: real nine-process, source-bound machine reviews.

Four original E14 artifacts and one real, pinned E15 reconciliation archive
are mandatory inputs. Each Bureau function executes in an actual OS subprocess
and checks true Git-pinned historical log bytes. These are machine reviews,
NOT nine independent AI agents or nine independently authenticated signers.
All verified work remains BLOCK/exit 2. Error is exit 3.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e15_cross_os_receipt_reconciliation as e15
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E16_NINE_MACHINE_REVIEW_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E16-NINE-MACHINE-REVIEW-CONSOLIDATION-V1"
REPORT_SCHEMA="NORDOI-G03-R6F-BUREAU001-E16-ACTUAL-MACHINE-FUNCTION-REPORT-V1"
MANIFEST_SCHEMA="NORDOI-G03-R6F-BUREAU001-E16-NINE-MACHINE-REPORTS-V1"
PARENT="8af265158bfbeb9c1406d2c310d505281dc98180"
ROLES=("ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
       "CAPSTONE","ADVERSARY","VERITY","GUARDIAN")
ISSUES=("CROSS_OS_SKIPS_NOT_NATIVE_EQUIVALENCE",
        "E14_PER_TARGET_PIDS_NOT_CAPTURED",
        "LOCAL_SIGNATURE_NOT_INDEPENDENT_CUSTODY",
        "FOUR_GITHUB_ARTIFACTS_SAME_ADMIN_DOMAIN",
        "NINE_MACHINE_FUNCTIONS_NOT_FORMAL_REVIEWS",
        "HISTORIC_GIT_PINS_NOT_RUNTIME_PROOF",
        "TWELVE_EXTERNAL_NATIVE_ADMISSIONS_UNMET")
class Blocked(ValueError): pass

def require(condition,reason):
    if not condition:raise Blocked(reason)
def canonical(value):return e15.e14.canonical(value)
def sha(value):return hashlib.sha256(value).hexdigest()
def read(root,path):return (root/path).read_bytes()
def deny(reason,**kwargs):
    d={"result":"BLOCK","reason":reason,"human_testers":0,
       "founder_manual_test_duty":False,"formal_independent_reviews":0,
       "external_witness_receipts":0,"native_admission_gates_unmet":12,
       "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
       "production":"NOT_AUTHORIZED"}
    d.update({k:v for k,v in kwargs.items() if k not in d})
    return d

def audit(root,head):
    e15c,groups,pins=e15.audit(root,head)
    c=e3.strictly_load(read(root,CONTRACT))
    require(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
        and c["parent_e15_exact_head"]==PARENT
        and c["prior_e15"]=={
           "specialized_run_id":44716501372,"specialized_success":4,
           "rust_run_id":44716501419,"rust_success":5,
           "e8_run_id":44716501426,"e8_success":4},
        "E16_E15_EXACT_GREEN_PARENT_NOT_PINNED")
    require(c["source_handoff"]==
              "governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json"
        and c["source_e15_contract"]==e15.CONTRACT
        and c["e15_anchor"]=={
            "github_run_id":44716501372,
            "artifact_id":14618579942,
            "artifact_name":"NORDOI_E15_ACTUAL_RECONCILIATION_macos-15-intel",
            "sha256":"fc457cce0ad9004d66467f85be97376549c653f464cfa03427457646c97ca576"},
        "E16_ACTUAL_E15_ARCHIVE_UNPINNED")
    require(c["prior_e14_run_id"]==44713831052
        and c["expected_actual_prior_e14_artifacts"]==4
        and c["expected_actual_prior_e14_raw_logs"]==228
        and c["expected_role_machine_processes"]==9
        and c["expected_source_anchored_machine_reports"]==9
        and tuple(c["role_source_targets"])==ROLES
        and tuple(c["contradiction_ids"])==ISSUES
        and c["claim_policy"]==
            "REAL_MACHINE_EVIDENCE_CHECK_NOT_EXTERNAL_CERTIFIED_REVIEW",
        "E16_ROLE_OR_CONTRADICTION_COVERAGE_INVALID")
    handoff=e3.strictly_load(read(root,c["source_handoff"]))
    require(tuple(r["role"] for r in handoff["review_functions"])==ROLES
        and all(r["review_status"]=="NOT_SUBMITTED" and
                r["formal_verdict"] is None for r in handoff["review_functions"])
        and handoff["mission_state"]=="ASSIGNED_REVIEWS_NOT_SUBMITTED",
        "E16_INVENTED_FORMAL_BUREAU_REVIEWS")
    testpaths=set(groups["rust"]+groups["python_test"])
    for role,paths in c["role_source_targets"].items():
        require(type(paths) is list and 2<=len(paths)<=4
            and len(paths)==len(set(paths)) and set(paths)<=testpaths,
            "E16_ROLE_HAS_UNPINNED_OR_DUPLICATED_SOURCE:"+role)
    require(c["formal_independent_reviews_submitted"]==0
        and c["external_authentic_witnesses"]==0
        and c["out_of_band_owner_root"]=="NOT_AUTHENTICATED"
        and c["independent_custodian"]=="NOT_ENROLLED"
        and c["global_monotonic_antirollback"]=="NOT_VERIFIED"
        and c["technical_execution"]=="AI_AND_AUTOMATION_ONLY"
        and c["founder_manual_test_duty"] is False
        and c["human_testers_allowed"] is False
        and c["native_admission_gates_unmet"]==12
        and c["r6f"]=="IN_PROGRESS_UNCERTIFIED"
        and c["r6g"]=="LOCKED_BY_SEQUENCE"
        and c["production"]=="NOT_AUTHORIZED"
        and c["merge_authorized"] is False
        and c["pre_ci_verdict"]=="IN_PROGRESS_UNQUALIFIED",
        "E16_AUTHORITY_HUMAN_OR_NATIVE_PROMOTION")
    return c,handoff,groups,pins

def actual_anchor(root,head,anchor_file,artifact_dir):
    c,hand,groups,pins=audit(root,head)
    ap=Path(anchor_file).resolve(strict=True)
    require(ap.is_file() and not ap.is_symlink() and
        sha(ap.read_bytes())==c["e15_anchor"]["sha256"],
        "E16_E15_ACTUAL_ARCHIVE_HASH_NOT_PINNED")
    anchor=e3.strictly_load(ap.read_bytes())
    require(anchor["schema"]==e15.RESULT_SCHEMA
        and anchor["source_head_e15"]==PARENT
        and anchor["parent_head_e14"]==e15.PARENT
        and anchor["source_github_e14_workflow_run_id"]==44713831052
        and anchor["real_prior_ci_artifacts_rechecked"]==4
        and anchor["genuine_prior_raw_logs_verified"]==228
        and anchor["historical_target_files_per_os"]==57
        and anchor["formal_external_reviews"]==0
        and anchor["external_witness_receipts"]==0
        and anchor["native_admission_gates_unmet"]==12
        and anchor["r6g"]=="LOCKED_BY_SEQUENCE"
        and anchor["human_testers"]==0
        and anchor["trust_boundary"]==
           "GITHUB_ARTIFACT_RECHECK_NOT_INDEPENDENT_EXTERNAL_CUSTODY",
        "E16_E15_ARCHIVED_RECONCILIATION_CLAIM_INCORRECT")
    current=e15.reconcile(root,head,artifact_dir)
    require(current["parent_head_e14"]==anchor["parent_head_e14"]
        and current["source_github_e14_workflow_run_id"]==
            anchor["source_github_e14_workflow_run_id"]
        and current["platforms"]==anchor["platforms"]
        and current["genuine_prior_raw_logs_verified"]==228,
        "E16_E15_ANCHOR_DISAGREES_WITH_LIVE_RECHECK_OF_228_LOGS")
    return c,hand,groups,pins,current

def role_report(root,head,artifact_dir,anchor_file,role,pid):
    require(role in ROLES,"E16_UNKNOWN_BUREAU_ROLE")
    c,hand,groups,pins,evidence=actual_anchor(
        root,head,anchor_file,artifact_dir)
    task=next(x for x in hand["review_functions"] if x["role"]==role)
    proof=[]
    for path in c["role_source_targets"][role]:
        pin=pins[path]["git_blob_sha1"]
        fp=root/path
        require(fp.is_file() and
            pin==e15.e14.e13.true_git_tree(root,head)[path][2],
            "E16_ROLE_SOURCE_NOT_REAL_GIT_PINNED:"+path)
        source_sha=sha(fp.read_bytes())
        per_os=[]
        for platform in evidence["platforms"]:
            name=platform["platform"]
            m=e3.strictly_load(read(Path(artifact_dir)/name,"MANIFEST.json"))
            record=next((x for x in m["records"]
                         if x["target_path"]==path),None)
            require(record is not None and record["success"] is True
                and record["source_git_blob_sha1"]==pin
                and record["ran_tests"]>0
                and record["return_code"]==0
                and record["passed_tests"]+record["skipped_tests"]==
                    record["ran_tests"],
                "E16_ROLE_TARGET_HAS_NO_TRUE_RUN_RECEIPT:"+role+":"+name)
            per_os.append({
                "platform":name,"passed":record["passed_tests"],
                "skipped":record["skipped_tests"],
                "ran":record["ran_tests"],
                "raw_log_sha256":record["raw_output_sha256"]})
        proof.append({"path":path,"source_git_blob_sha1":pin,
                      "source_sha256":source_sha,
                      "true_E14_test_records_by_platform":per_os})
    if role=="FERRUM":
        native=next(x for x in proof if "r4m_pinned_code_identity" in x["path"])
        p={v["platform"]:v for v in native["true_E14_test_records_by_platform"]}
        require(p["macos-15-intel"]["passed"]>=6
            and p["macos-15"]["passed"]>=6,
            "E16_FERRUM_SIX_REAL_MAC_NATIVE_ASSERTIONS_MISSING")
    require(isinstance(pid,int) and pid>0,"E16_INVALID_PROCESS_PID")
    return {
        "schema":REPORT_SCHEMA,"role":role,"process_pid_observed_by_launcher":pid,
        "source_head_e16":head,"parent_e15_head":PARENT,
        "source_handoff":c["source_handoff"],
        "assigned_task_id":task["task_id"],
        "assigned_source_loci_unverified_labels":task["source_loci"],
        "machine_check_status":"SOURCE_AND_ACTUAL_E14_LOGS_VERIFIED",
        "formal_review_status":"NOT_SUBMITTED",
        "formal_independent_specialist_verdict":None,
        "external_administrative_independence_proven":False,
        "actual_e14_artifacts_verified":4,
        "actual_e14_raw_logs_verified":228,
        "e15_anchor_sha256":c["e15_anchor"]["sha256"],
        "source_evidence":proof,
        "native_admission_gates_unmet":12,
        "r6g":"LOCKED_BY_SEQUENCE",
        "production":"NOT_AUTHORIZED",
        "human_testers":0
    }

def matrix(c,role_reports,evidence):
    require(tuple(r["role"] for r in role_reports)==ROLES,
        "E16_NOT_NINE_ORDERED_ROLES")
    totals={p["platform"]:(p["cases_passed"],p["cases_skipped"])
            for p in evidence["platforms"]}
    require(totals=={
         "macos-15-intel":(1039,3),"macos-15":(1039,3),
         "ubuntu-latest":(881,19),"windows-latest":(670,226)},
         "E16_FALSE_POSITIVE_PLATFORM_TOTALS")
    factual=[
       ("CROSS_OS_SKIPS_NOT_NATIVE_EQUIVALENCE",
        "Mac: 3 SKIP each; Linux: 19; Windows: 226. Skips are not passes.",
        "E14_4_PLATFORM_MANIFESTS","OPEN_PORTABILITY_BOUNDARY"),
       ("E14_PER_TARGET_PIDS_NOT_CAPTURED",
        "E14 logs record 57 commands per OS but not a historical PID per target.",
        "E15_CONTRACT","OPEN_PROVENANCE_LIMIT"),
       ("LOCAL_SIGNATURE_NOT_INDEPENDENT_CUSTODY",
        "Actual local SSHSIG coverage does not authenticate a separate administrator.",
        "R6E_AND_E12_NEGATIVE_TESTS","OPEN_EXTERNAL_DEPENDENCY"),
       ("FOUR_GITHUB_ARTIFACTS_SAME_ADMIN_DOMAIN",
        "Four successful GitHub runners are not four autonomous custodians.",
        "E15_ACTUAL_RECONCILIATION","OPEN_EXTERNAL_DEPENDENCY"),
       ("NINE_MACHINE_FUNCTIONS_NOT_FORMAL_REVIEWS",
        "Nine actual processes do not constitute nine independently signed specialist reports.",
        "BUREAU_R6F_HANDOFF","OPEN_FORMAL_REVIEW"),
       ("HISTORIC_GIT_PINS_NOT_RUNTIME_PROOF",
        "E13 99 source pins alone were insufficient; E14 actual logs are separate.",
        "E13_SOURCE_AND_E14_EXECUTION","RESOLVED_TECHNICAL_NOT_EXTERNAL"),
       ("TWELVE_EXTERNAL_NATIVE_ADMISSIONS_UNMET",
        "No actual external custodian, OOB owner root, global antirollback or quorum.",
        "E12_12_GATE_NEGATIVE_CONTRACT","OPEN_EXTERNAL_DEPENDENCY"),
    ]
    require(tuple(x[0] for x in factual)==ISSUES,
        "E16_CONTRADICTION_IDS_NOT_EXACT")
    return [{"id":id,"observation":msg,"reference":ref,"status":state}
            for id,msg,ref,state in factual]

def verify_dossier(root,head,artifacts,anchor,folder):
    c,hand,groups,pins,evidence=actual_anchor(root,head,anchor,artifacts)
    out=Path(folder).absolute()
    raw=read(out,"MANIFEST.json")
    m=e3.strictly_load(raw)
    require(raw==canonical(m) and
        m["schema"]==MANIFEST_SCHEMA and m["source_head_e16"]==head
        and m["parent_e15_head"]==PARENT
        and m["e15_anchor_sha256"]==c["e15_anchor"]["sha256"]
        and m["real_machine_processes"]==9
        and m["actual_e14_raw_logs_verified"]==228
        and m["formal_independent_specialist_reviews"]==0
        and m["external_witness_receipts"]==0
        and m["native_admission_gates_unmet"]==12
        and m["human_testers"]==0 and m["r6g"]=="LOCKED_BY_SEQUENCE"
        and m["production"]=="NOT_AUTHORIZED",
        "E16_DOSSIER_MANIFEST_NOT_CANONICAL_OR_UNAUTHORIZED")
    require(len(m["reports"])==9
        and tuple(x["role"] for x in m["reports"])==ROLES
        and len({x["pid"] for x in m["reports"]})==9
        and len({x["filename"] for x in m["reports"]})==9,
        "E16_NINE_REAL_MACHINE_REPORTS_MISSING_DUPLICATED")
    require({x.name for x in out.iterdir()}==
        {"MANIFEST.json"}|{r["filename"] for r in m["reports"]},
        "E16_MISSING_EXTRA_OR_INJECTED_MACHINE_REPORT")
    reports=[]
    for entry in m["reports"]:
        path=out/entry["filename"]
        require(path.parent==out and path.is_file() and
            not path.is_symlink(),"E16_MISSING_OR_SYMLINK_MACHINE_REPORT")
        raw_report=path.read_bytes()
        require(sha(raw_report)==entry["sha256"],
            "E16_ROLE_REPORT_HASH_TAMPERED:"+entry["role"])
        report=e3.strictly_load(raw_report)
        require(raw_report==canonical(report)
            and report["process_pid_observed_by_launcher"]==entry["pid"],
            "E16_ROLE_NONCANONICAL_OR_PID_REPLACED")
        expected=role_report(root,head,artifacts,anchor,
                             entry["role"],entry["pid"])
        require(report==expected,
            "E16_COHERENT_SAME_ADMIN_ROLE_REWRITE_REJECTED:"+entry["role"])
        reports.append(report)
    require(m["contradictions"]==matrix(c,reports,evidence),
        "E16_CONTRADICTION_MATRIX_REWRITTEN")
    return m

def exercise(root,head,artifacts,anchor,folder):
    audit(root,head)
    actual_anchor(root,head,anchor,artifacts)
    out=Path(folder).absolute()
    require(not out.exists() and out.parent.is_dir()
         and not out.parent.is_symlink(),"E16_OUTPUT_NEEDS_FRESH_FOLDER")
    out.mkdir(mode=0o700)
    script=str(Path(__file__).resolve())
    processes=[]
    try:
        for role in ROLES:
            p=subprocess.Popen([
                sys.executable,"-B",script,"--role",role,
                "--root",str(root),"--expected-head",head,
                "--artifact-dir",str(artifacts),"--e15-anchor",str(anchor)],
                cwd=root,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            processes.append((role,p))
        entries=[]
        role_reports=[]
        for role,p in processes:
            raw,err=p.communicate(timeout=360)
            require(p.returncode==2,
                "E16_REAL_ROLE_CHILD_FAILED:"+role+
                ":"+err[:180].decode(errors="replace"))
            report=e3.strictly_load(raw)
            require(raw==canonical(report)
                and report["role"]==role
                and report["process_pid_observed_by_launcher"]==p.pid,
                "E16_CHILD_ACTUAL_ROLE_PID_OR_BYTES_MISMATCH")
            fn=role+".json"
            (out/fn).write_bytes(raw)
            entries.append({"role":role,"filename":fn,
                            "sha256":sha(raw),"pid":p.pid})
            role_reports.append(report)
        require(len({r["pid"] for r in entries})==9,
            "E16_NOT_NINE_DISTINCT_REAL_PROCESS_IDS")
        c,hand,groups,pins,evidence=actual_anchor(
            root,head,anchor,artifacts)
        m={
          "schema":MANIFEST_SCHEMA,
          "source_head_e16":head,"parent_e15_head":PARENT,
          "e15_anchor_sha256":c["e15_anchor"]["sha256"],
          "real_machine_processes":9,
          "actual_e14_raw_logs_verified":228,
          "formal_independent_specialist_reviews":0,
          "external_witness_receipts":0,
          "native_admission_gates_unmet":12,
          "human_testers":0,
          "r6f":"IN_PROGRESS_UNCERTIFIED",
          "r6g":"LOCKED_BY_SEQUENCE",
          "production":"NOT_AUTHORIZED",
          "contradictions":matrix(c,role_reports,evidence),
          "reports":entries}
        (out/"MANIFEST.json").write_bytes(canonical(m))
        verify_dossier(root,head,artifacts,anchor,out)
        return deny("E16_NINE_REAL_SOURCE_ANCHORED_MACHINE_FUNCTION_CHECKS",
            source_head=head,
            real_bureau_machine_child_processes=9,
            source_anchored_machine_reports=9,
            contradictions_recorded=7,
            actual_prior_e14_log_checks=228,
            actual_formal_independent_bureau_reports=0,
            manifest_sha256=sha(read(out,"MANIFEST.json")))
    finally:
        for role,p in processes:
            if p.poll() is None:
                p.kill()
                p.communicate()

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    modes=ap.add_mutually_exclusive_group(required=True)
    modes.add_argument("--audit",action="store_true")
    modes.add_argument("--role",choices=ROLES)
    modes.add_argument("--exercise",action="store_true")
    modes.add_argument("--verify",action="store_true")
    ap.add_argument("--root",default=str(ROOT))
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--artifact-dir")
    ap.add_argument("--e15-anchor")
    ap.add_argument("--output-dir")
    args=ap.parse_args(argv)
    try:
        root=Path(args.root).resolve(strict=True)
        if args.audit:
            audit(root,args.expected_head)
            d=deny("E16_ONLY_SOURCE_CONTRACT_VERIFIED",
                source_head=args.expected_head,
                source_anchored_machine_reports_executed=0)
        else:
            require(args.artifact_dir and args.e15_anchor,
                "E16_REQUIRES_REAL_E14_AND_E15_ARTIFACTS")
            if args.role:
                d=role_report(root,args.expected_head,
                     args.artifact_dir,args.e15_anchor,args.role,os.getpid())
            elif args.exercise:
                require(args.output_dir,"E16_OUTPUT_DIR_REQUIRED")
                d=exercise(root,args.expected_head,args.artifact_dir,
                    args.e15_anchor,args.output_dir)
            else:
                require(args.output_dir,"E16_VERIFY_NEEDS_OUTPUT_DIR")
                m=verify_dossier(root,args.expected_head,args.artifact_dir,
                     args.e15_anchor,args.output_dir)
                d=deny("E16_COMPLETE_NINE_MACHINE_REPORTS_REVERIFIED",
                    exact_source_head=args.expected_head,
                    source_anchored_machine_reports=len(m["reports"]),
                    contradictions_recorded=len(m["contradictions"]))
        code=2
    except (Blocked,e15.Blocked,e15.e14.Blocked,
            e15.e14.e13.Blocked,e15.e14.e13.e12.Blocked,
            e3.Rejected,KeyError,ValueError,TypeError,OSError,
            subprocess.SubprocessError) as exc:
        d=deny("E16_MACHINE_DOSSIER_NOT_QUALIFIED",
           error_type=type(exc).__name__,
           diagnostic=str(exc)[:240] if isinstance(exc,Blocked) else None)
        code=3
    sys.stdout.buffer.write(canonical(d))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":raise SystemExit(main())
