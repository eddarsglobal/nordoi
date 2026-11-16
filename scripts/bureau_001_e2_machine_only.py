#!/usr/bin/env python3
"""BUREAU-001 E2 — real machine-only role suite execution, zero human testers.

This launcher is a deterministic executor of existing Rust/Python tests
assigned to nine Bureau *functional specialties*. It does NOT invoke nine
independent AI agents, sign as outside reviewers, enroll an external custodian
or imply independent administrative authority. Human testing is prohibited by
owner policy; explicit owner governance authorization is a separate matter.

--audit: source-read-only, no test execution or evidence writes.
--run: on actual macOS, run nine concrete machine suites and store evidence
       and digests OUTSIDE the source tree; never invent test success.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

sys.path.insert(0,str(Path(__file__).resolve().parent))
import roadmap_guardian as guardian
import bureau_001_r1_audit as r1

PROJECT="eddarsglobal/nordoi"
PARENT="90f2ea2d7fde2116aaabb3afb3def35650afa3d5"
POLICY="governance/NORDOI_R6F_BUREAU001_E2_NO_HUMAN_TESTING_POLICY.json"
E1="governance/NORDOI_R6F_BUREAU001_E1_INDEPENDENT_REVIEW_REQUESTS.json"
R2="governance/NORDOI_R6F_BUREAU001_R2_REVIEW_REGISTER.json"
HANDOFF="governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json"
EXPECTED_MANIFEST_SHA="NORDOI-G03-R6F-BUREAU001-E2-ZERO-HUMAN-TESTING-POLICY-V1"
EXPECTED_ROLES=(
    "ATLAS","FERRUM","BOUNDARY","CIPHER","LEDGER",
    "CAPSTONE","ADVERSARY","VERITY","GUARDIAN")
SCHEMA="NORDOI-G03-R6F-BUREAU001-E2-ACTUAL-MACHINE-ROLE-TESTS-V1"

class EvidenceError(Exception):
    pass

def require(test,reason):
    if not test:raise EvidenceError(reason)

def load(root,p):
    f=(Path(root)/p)
    require(f.is_file() and not f.is_symlink(),"SOURCE_MISSING_OR_SYMLINK:"+p)
    return json.loads(f.read_text(encoding="utf-8"))

def source_hash(root,rel):
    f=(Path(root)/rel)
    require(f.is_file() and not f.is_symlink(),"SOURCE_MISSING:"+rel)
    p=subprocess.run(["git","hash-object","--",rel],cwd=root,
                     capture_output=True,text=True,check=True,timeout=20)
    return p.stdout.strip()

def audit(root):
    root=Path(root).resolve(strict=True)
    policy=load(root,POLICY); e1=load(root,E1)
    r2=load(root,R2); handoff=load(root,HANDOFF)
    require(policy["schema"]==EXPECTED_MANIFEST_SHA
        and policy["project"]==PROJECT
        and policy["source_e1_exact_head"]==PARENT
        and policy["scope"]=="ALL_FUTURE_NORDOI_TESTING_AND_REVIEW_EXECUTION"
        and policy["human_testers_allowed"] is False
        and policy["human_reviewer_outreach_allowed"] is False
        and policy["human_review_packet_distribution_allowed"] is False
        and policy["human_test_execution_count"]==0,
        "E2_OWNER_ZERO_HUMAN_POLICY_NOT_ENFORCED")
    require(policy["frozen_source_policy"].startswith("Do not rewrite")
        and policy["manual_owner_privileged_authorization"]==
           "STILL_REQUIRED_BY_FROZEN_GOVERNANCE_AND_IS_NOT_A_TEST",
        "E2_FROZEN_GOVERNANCE_AND_OWNER_APPROVAL_DISTINCTION_LOST")
    require(tuple(x["role"] for x in policy["nine_bureau_machine_test_functions"])==EXPECTED_ROLES
        and tuple(x["role"] for x in e1["independent_review_requests"])==EXPECTED_ROLES
        and tuple(x["role"] for x in r2["roles"])==EXPECTED_ROLES
        and tuple(x["role"] for x in handoff["review_functions"])==EXPECTED_ROLES
        and tuple(guardian.ROLE_IDS)==EXPECTED_ROLES,"E2_ROLE_SET_CHANGED")
    require(policy["status"]["requests_sent"]==0 if "status" in policy
            else policy["historic_review_status"]=="0_INDEPENDENT_SPECIALIST_SUBMISSIONS",
            "E2_FAKE_INDEPENDENT_EVIDENCE")
    require(e1["status"]["requests_sent"]==0 and
            e1["status"]["reviews_received"]==0 and
            e1["status"]["operator_contacted"] is False and
            e1["status"]["operator_enrolled"] is False,
            "E1_INVITATION_OR_EXTERNAL_REVIEW_ALREADY_CLAIMED")
    require(r2["submissions"]==[] and
            r2["independent_specialist_agents_executed"]==0,
            "E2_NINE_SOURCE_ROLES_MISLABELED_AS_REAL_REVIEWS")
    require(len(policy["superseded_e1_actions"])==3 and
            all("DO_NOT_SEND" in item or "NOT_COMPATIBLE" in item
                for item in policy["superseded_e1_actions"]),
            "E2_E1_HUMAN_TEST_OUTREACH_NOT_CANCELLED")
    require(policy["real_external_custodian"]=="NOT_ENROLLED"
        and policy["owner_oob_root"]=="NOT_AUTHENTICATED"
        and policy["global_monotonic_antirollback"]=="NOT_VERIFIED"
        and policy["formal_r6f_closure"]=="HOLD_MISSING_INDEPENDENT_EVIDENCE"
        and policy["r6f"]=="IN_PROGRESS_UNCERTIFIED"
        and policy["r6g"]=="LOCKED_BY_SEQUENCE"
        and policy["native_admission_gates_unmet"]==12
        and policy["production"]=="NOT_AUTHORIZED",
        "E2_FALSE_ELEVATION_OR_EXTERNAL_AUTHORITY")
    for p,h in zip(e1["independent_review_requests"],r2["roles"]):
        source=p["baseline_research_anchor"]
        require(source["path"]==h["source_path"]
            and source["git_blob_sha1"]==h["git_blob_sha1"]
            and source_hash(root,source["path"])==h["git_blob_sha1"]
            and p["work_order_status"]=="PREPARED_NOT_DISPATCHED"
            and p["independent_execution"]=="NOT_STARTED"
            and p["reviewer_identity"] is None
            and p["evidence_submitted"] is False,
            "E2_E1_SOURCE_BLOB_OR_ACTUAL_INDEPENDENT_EXECUTION_MISMATCH:"+p["role"])
    prior=r1.audit(root)
    require(prior["FROZEN_SOURCE_BLOBS"]=="99/99_PASS",
            "E2_ORIGINAL_99_FROZEN_GIT_BLOBS_CHANGED")
    return {
        "source_policy":"ZERO_HUMAN_TESTERS_ENFORCED",
        "role_source_pins":"9/9_PASS",
        "frozen_r1_git_blobs":"99/99_PASS",
        "historical_human_review_requests":"CANCELLED_UNSENT",
        "real_human_testers":0,"actual_independent_specialist_reviews":0,
        "real_external_custodian":"NOT_ENROLLED",
        "r6g":"LOCKED_BY_SEQUENCE","native_admission_gates_unmet":12,
        "production":"NOT_AUTHORIZED"
    }

def specs():
    py=[sys.executable,"-B","-m","unittest","discover"]
    return [
        ("ATLAS",["cargo","test","--offline","--release","--test",
                   "g03_r4a_structured_native_lab","--","--test-threads=1"],
         "RUST_PASS_10"),
        ("FERRUM",["cargo","test","--offline","--release","--test",
                   "g03_r4m_pinned_code_identity_lab","--","--nocapture",
                   "--test-threads=1"],"RUST_PASS_6"),
        ("BOUNDARY",py+["-s","research","-p",
                        "test_g03_r6f_r4_fd_race.py","-v"],"PYTHON_PASS_24"),
        ("CIPHER",py+["-s","research","-p",
                      "test_g03_r6f_r6_external_custody_feasibility.py","-v"],
         "PYTHON_PASS_24"),
        ("LEDGER",py+["-s","research","-p",
                      "test_g03_r6f_r5_inplace_aba.py","-v"],"PYTHON_PASS_24"),
        ("CAPSTONE",py+["-s","scripts","-p",
                        "test_bureau_001_r8_closure_gate.py","-v"],"PYTHON_PASS_24"),
        ("ADVERSARY",py+["-s","scripts","-p",
                         "test_bureau_001_r9_final_evidence_decision.py","-v"],
         "PYTHON_PASS_24"),
        ("VERITY",["cargo","fmt","--all","--","--check"],"EXIT_ZERO"),
        ("GUARDIAN",py+["-s","tests","-p",
                        "test_roadmap_guardian.py","-v"],"PYTHON_PASS_24"),
    ]

def verify_result(log,expected,code):
    require(code==0,"E2_MACHINE_COMMAND_NONZERO")
    if expected.startswith("PYTHON_PASS_"):
        n=int(expected.rsplit("_",1)[1])
        require(re.search(rb"Ran "+str(n).encode()+rb" tests? in ",log)
                and b"\nOK" in log and not re.search(rb"\nFAILED",log),
                "E2_WRONG_PYTHON_CASE_COUNT_OR_FAILURE")
    elif expected.startswith("RUST_PASS_"):
        n=int(expected.rsplit("_",1)[1])
        require(re.search(rb"test result: ok\.\s*"+str(n).encode()+
                 rb" passed;\s*0 failed;\s*0 ignored",log),
                "E2_WRONG_RUST_NATIVE_PASS_COUNT_OR_FAILURE_OR_IGNORED")
    return True

def run(root,output):
    root=Path(root).resolve(strict=True)
    baseline=audit(root)
    require(sys.platform=="darwin",
            "E2_FULL_NINE_MACHINE_ROLE_RUN_REQUIRES_ACTUAL_MACOS_NATIVE_XPC")
    out=Path(output).expanduser()
    require(not out.exists(),"E2_EVIDENCE_OUTPUT_MUST_BE_NEW_DIRECTORY")
    parent=out.parent.resolve(strict=True)
    require(parent!=root and root not in parent.parents,
            "E2_EVIDENCE_MUST_BE_WRITTEN_OUTSIDE_SOURCE_TREE")
    out.mkdir(mode=0o700)
    chmod=os.stat(out).st_mode & 0o777
    require(chmod==0o700,"E2_EVIDENCE_OUTPUT_PERMISSIONS_NOT_0700")
    before=subprocess.check_output(["git","rev-parse","HEAD"],cwd=root,
                                   text=True).strip()
    require(re.fullmatch("[0-9a-f]{40}",before)!=None,
            "E2_SOURCE_HEAD_MISSING")
    results=[]
    try:
        for role,command,expected in specs():
            begin=time.monotonic()
            p=subprocess.run(command,cwd=root,capture_output=True,timeout=1200)
            log=p.stdout+b"\n--- STDOUT/STDERR SEPARATOR ---\n"+p.stderr
            digest=hashlib.sha256(log).hexdigest()
            (out/(role+".log")).write_bytes(log)
            record={
                "role":role,"execution_kind":"ACTUAL_DETERMINISTIC_MACHINE_TEST",
                "claimed_independent_specialist":False,"human_tester":False,
                "command":command,"exit_code":p.returncode,
                "log_sha256":digest,"expected_control":expected,
                "elapsed_monotonic_seconds":round(time.monotonic()-begin,3)}
            try:verify_result(log,expected,p.returncode)
            except EvidenceError as e:
                record["status"]="FAIL";record["failure_reason"]=str(e)
                results.append(record)
                break
            record["status"]="PASS";results.append(record)
        after=subprocess.check_output(["git","rev-parse","HEAD"],cwd=root,
                                      text=True).strip()
        clean=subprocess.run(["git","status","--porcelain"],
                             cwd=root,capture_output=True,text=True,check=True).stdout
        require(before==after and not clean,"E2_SOURCE_HEAD_CHANGED_OR_DIRTY")
        summary={
            "schema":SCHEMA,"source_head_before_after":before,
            "source_worktree_clean":True,"runner_os":platform.system(),
            "runner_architecture":platform.machine(),
            "role_suite_count_expected":9,"role_suite_count_executed":len(results),
            "real_human_testers":0,"machine_results":results,
            "all_nine_machine_suites_passed":len(results)==9
               and all(x["status"]=="PASS" for x in results),
            "independent_specialist_reviews_received":0,
            "outside_administrative_authority_authenticated":False,
            "native_admission_gates_unmet":12,"r6g":"LOCKED_BY_SEQUENCE",
            "production":"NOT_AUTHORIZED", "preflight":baseline}
        (out/"E2_MACHINE_EVIDENCE.json").write_text(
            json.dumps(summary,indent=2,sort_keys=True)+"\n",encoding="utf-8")
        require(summary["all_nine_machine_suites_passed"],
                "E2_ONE_OR_MORE_ACTUAL_MACHINE_SUITES_FAILED")
        return summary
    finally:
        # Never call external network services or modify the source checkout.
        pass

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1])
    group=p.add_mutually_exclusive_group(required=True)
    group.add_argument("--audit",action="store_true")
    group.add_argument("--run",action="store_true")
    p.add_argument("--output",type=Path)
    arg=p.parse_args(argv)
    try:
        if arg.audit:
            result=audit(arg.root)
        else:
            require(arg.output is not None,"E2_OUTPUT_DIRECTORY_REQUIRED")
            result=run(arg.root,arg.output)
        print(json.dumps(result,sort_keys=True))
        return 0
    except (EvidenceError,subprocess.CalledProcessError,subprocess.TimeoutExpired,
            OSError,ValueError,KeyError) as e:
        print(json.dumps({"E2":"REMEDIATE_MACHINE_TEST_NOT_CERTIFIED",
                          "reason":str(e),"human_testers":0,
                          "r6g":"LOCKED_BY_SEQUENCE",
                          "native_admission_gates_unmet":12,
                          "production":"NOT_AUTHORIZED"},sort_keys=True))
        return 3

if __name__=="__main__":
    sys.exit(main())
