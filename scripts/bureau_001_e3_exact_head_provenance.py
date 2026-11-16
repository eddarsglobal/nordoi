#!/usr/bin/env python3
"""BUREAU-001 E3: bind actual no-human CI evidence to exact GitHub PR head.

Read-only verification of E2 real nine-machine-suite evidence. Any JSON and
log contents are SAME-ADMIN machine self-reports, NOT independent custody.

The GitHub job MUST first checkout github.event.pull_request.head.sha and
pass that value as --expected-head. This code checks actual Git HEAD, not
a branch name, github.sha (which can be a synthetic PR merge), or an
unverified name printed in a test log.

--audit checks actual repository source, PR event pin and E2 historical
provenance contradiction. --verify additionally verifies actual nine logs
and original machine record under a regular-file/no-symlink boundary.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

sys.path.insert(0,str(Path(__file__).resolve().parent))
import bureau_001_e2_machine_only as e2

SCHEMA="NORDOI-G03-R6F-BUREAU001-E3-EXACT-HEAD-EVIDENCE-BOUNDARY-V1"
POLICY="governance/NORDOI_R6F_BUREAU001_E3_EXACT_HEAD_PROVENANCE.json"
BASE_E2="33a677224feddc580b9507d6c3fa826cabe27004"
OBSERVED_E2_MERGE="4736162aee852e7b9bbe43cfe186121c632ec4bb"
ROLES=e2.EXPECTED_ROLES
HEX40=re.compile(r"^[0-9a-f]{40}$")
HEX64=re.compile(r"^[0-9a-f]{64}$")

class Rejected(ValueError):
    pass

def need(ok,why):
    if not ok:raise Rejected(why)

def strictly_load(data):
    def unique(pairs):
        ret={}
        for k,v in pairs:
            if k in ret:raise Rejected("DUPLICATE_EVIDENCE_KEY:"+k)
            ret[k]=v
        return ret
    def invalid(v):
        raise Rejected("NONFINITE_EVIDENCE_VALUE")
    need(type(data) is bytes and 0<len(data)<=512*1024,
         "MISSING_OR_OVERSIZE_JSON_EVIDENCE")
    try:
        v=json.loads(data.decode("utf-8"),
                     object_pairs_hook=unique,parse_constant=invalid)
    except (UnicodeError,json.JSONDecodeError) as ex:
        raise Rejected("INVALID_EVIDENCE_JSON") from ex
    need(type(v) is dict,"EVIDENCE_NOT_JSON_OBJECT")
    return v

def real_file_bytes(path,limit=8*1024*1024):
    path=Path(path)
    # Open the actual inode; not a substituted pathname followed by a
    # later open. This prevents simple symlink substitution, not same-admin
    # coherent forgery of both log AND manifest before verification.
    st=os.lstat(path)
    need(stat.S_ISREG(st.st_mode) and not stat.S_ISLNK(st.st_mode),
         "LOG_OR_MANIFEST_NOT_REGULAR_FILE")
    fd=os.open(path,os.O_RDONLY|getattr(os,"O_NOFOLLOW",0))
    try:
        now=os.fstat(fd)
        need(stat.S_ISREG(now.st_mode) and
             now.st_dev==st.st_dev and now.st_ino==st.st_ino and
             now.st_size<=limit,
             "LOG_INODE_CHANGED_OR_OVERSIZE")
        with os.fdopen(fd,"rb",closefd=False) as f:
            contents=f.read(limit+1)
        need(len(contents)<=limit,"EVIDENCE_FILE_OVERSIZED")
        return contents
    finally:
        os.close(fd)

def check_head(root,expected_head):
    need(type(expected_head) is str and HEX40.fullmatch(expected_head),
         "EXPECTED_PR_HEAD_NOT_EXACT_SHA40")
    root=Path(root).resolve(strict=True)
    check=subprocess.run(["git","rev-parse","--verify","HEAD^{commit}"],
          cwd=root,check=True,capture_output=True,text=True,timeout=20)
    real=check.stdout.strip()
    need(real==expected_head,"CHECKOUT_HEAD_DIFFERS_FROM_PR_EVENT_HEAD")
    status=subprocess.run(["git","status","--porcelain"],cwd=root,
          check=True,capture_output=True,text=True,timeout=20).stdout
    need(not status,"SOURCE_WORKTREE_CHANGED_DURING_EVIDENCE_REVIEW")
    return real

def audit(root,expected_head):
    root=Path(root).resolve(strict=True)
    actual=check_head(root,expected_head)
    previous=e2.audit(root)
    p=strictly_load(real_file_bytes(root/POLICY))
    obs=p["observed_e2_run"]
    need(p["schema"]==SCHEMA and p["project"]=="eddarsglobal/nordoi"
         and p["parent_e2_exact_head"]==BASE_E2,
         "E3_POLICY_SOURCE_NOT_EXACT")
    need(obs["github_pr_candidate_head"]==BASE_E2 and
         obs["actual_checkout_head_from_e2_json"]==OBSERVED_E2_MERGE and
         obs["commit_matches_pr_head"] is False and
         obs["workflow_run"]==43932867642 and
         obs["job_id"]==133530096908 and
         obs["artifact_id"]==14258877979 and
         obs["actual_machine_suites_passed"]==9 and
         obs["result_scope"]==
          "GITHUB_SYNTHETIC_PR_MERGE_CHECKOUT_PASS_NO_EXACT_PR_HEAD_ASSERTION",
         "E3_HISTORICAL_SYNTHETIC_MERGE_FACT_LOST")
    hashes=obs["per_role_log_digests_reported_in_e2_job_log"]
    need(tuple(hashes)==ROLES and all(HEX64.fullmatch(x) for x in hashes.values())
         and obs["independent_specialists_authenticated"]==0
         and obs["human_testers_used"]==0
         and obs["external_custodian_proven"] is False,
         "E3_OLD_E2_ROLE_HASHES_OR_AUTHORITY_FALSIFIED")
    need(p["human_testers_allowed"] is False and
         p["independent_specialist_reviews"]==0 and
         p["real_external_custodian"]=="NOT_ENROLLED" and
         p["owner_oob_root"]=="NOT_AUTHENTICATED" and
         p["global_monotonic_antirollback"]=="NOT_VERIFIED" and
         p["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
         p["r6g"]=="LOCKED_BY_SEQUENCE" and
         p["native_admission_gates_unmet"]==12 and
         p["production"]=="NOT_AUTHORIZED" and
         p["no_privileged_transition"] is True,
         "E3_FALSE_EXTERNAL_AUTHORITY_OR_PRIVILEGED_PROMOTION")
    return dict(result="E3_EXACT_HEAD_SOURCE_AUDIT_PASS",
                checked_out_head=actual,
                prior_e2_machine_checkout=OBSERVED_E2_MERGE,
                prior_e2_pr_head=BASE_E2,
                prior_e2_was_synthetic_merge=True,
                nine_historical_real_machine_suites="9/9_PASS_ON_MERGE_REF",
                human_testers=0,independent_specialist_reviews=0,
                native_admission_gates_unmet=12,
                r6g="LOCKED_BY_SEQUENCE",production="NOT_AUTHORIZED",
                e2_source_preflight=previous["frozen_r1_git_blobs"])

def validate_manifest(m,expected_head,logs):
    need(type(m) is dict and
         m.get("schema")==e2.SCHEMA and
         m.get("source_head_before_after")==expected_head and
         m.get("source_worktree_clean") is True and
         m.get("runner_os")=="Darwin" and
         m.get("runner_architecture")=="x86_64" and
         m.get("role_suite_count_expected")==9 and
         m.get("role_suite_count_executed")==9 and
         m.get("all_nine_machine_suites_passed") is True and
         m.get("real_human_testers")==0 and
         m.get("independent_specialist_reviews_received")==0 and
         m.get("outside_administrative_authority_authenticated") is False and
         m.get("native_admission_gates_unmet")==12 and
         m.get("r6g")=="LOCKED_BY_SEQUENCE" and
         m.get("production")=="NOT_AUTHORIZED",
         "E3_UNVERIFIED_HEAD_OR_UNTRUE_EXECUTION_CLAIM")
    rs=m.get("machine_results")
    need(type(rs) is list and len(rs)==9 and
         tuple(x.get("role") for x in rs)==ROLES and
         type(logs) is dict and tuple(logs)==ROLES,
         "E3_MISSING_DUPLICATED_OR_FABRICATED_ROLE")
    for record,(role,command,control) in zip(rs,e2.specs()):
        data=logs[role]
        need(record.get("execution_kind")=="ACTUAL_DETERMINISTIC_MACHINE_TEST"
          and record.get("command")==command
          and record.get("expected_control")==control
          and record.get("exit_code")==0
          and record.get("status")=="PASS"
          and record.get("claimed_independent_specialist") is False
          and record.get("human_tester") is False
          and type(record.get("elapsed_monotonic_seconds")) in (int,float)
          and record["elapsed_monotonic_seconds"]>=0,
          "E3_ROLE_COMMAND_NOT_ACTUAL_OR_FAKE_PASS:"+role)
        sha=record.get("log_sha256")
        need(type(sha) is str and HEX64.fullmatch(sha) and
             sha==hashlib.sha256(data).hexdigest(),
             "E3_LOG_BYTES_DIFFER_FROM_ROLE_SHA256:"+role)
        try:e2.verify_result(data,control,record["exit_code"])
        except e2.EvidenceError as ex:
            raise Rejected("E3_REAL_ROLE_COUNTS_NOT_SUPPORTED:"+role) from ex
    return dict(
       result="E3_NINE_ROLE_ACTUAL_EVIDENCE_BOUND_TO_EXACT_CHECKED_OUT_PR_HEAD",
       exact_source_head=expected_head,role_logs_sha256_verified=9,
       machine_role_suites=9,human_testers=0,
       independent_specialist_reviews=0,
       external_custodian_authenticated=False,
       r6g="LOCKED_BY_SEQUENCE",
       native_admission_gates_unmet=12,production="NOT_AUTHORIZED",
       evidence_class="SELF_CONSISTENT_CI_MACHINE_PROOF_NOT_INDEPENDENT_AUTHORITY"
    )

def verify(root,expected_head,evidence_dir):
    root=Path(root).resolve(strict=True)
    pre=audit(root,expected_head)
    p=Path(evidence_dir).resolve(strict=True)
    need(p!=root and root not in p.parents,
         "E3_EVIDENCE_DIRECTORY_INSIDE_SOURCE")
    need(p.is_dir(),"E3_EVIDENCE_NOT_DIRECTORY")
    manifest=strictly_load(real_file_bytes(p/"E2_MACHINE_EVIDENCE.json"))
    logs={role:real_file_bytes(p/(role+".log"))
          for role in ROLES}
    result=validate_manifest(manifest,expected_head,logs)
    need(check_head(root,expected_head)==pre["checked_out_head"],
         "E3_SOURCE_CHANGED_DURING_VERIFICATION")
    return result

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    a=parser.add_mutually_exclusive_group(required=True)
    a.add_argument("--audit",action="store_true")
    a.add_argument("--verify",action="store_true")
    parser.add_argument("--root",type=Path,
                        default=Path(__file__).resolve().parents[1])
    parser.add_argument("--expected-head",required=True)
    parser.add_argument("--evidence-dir",type=Path)
    x=parser.parse_args(argv)
    try:
        if x.audit:result=audit(x.root,x.expected_head)
        else:
            need(x.evidence_dir is not None,"E3_EVIDENCE_DIR_REQUIRED")
            result=verify(x.root,x.expected_head,x.evidence_dir)
        print(json.dumps(result,sort_keys=True))
        return 0
    except (Rejected,e2.EvidenceError,OSError,KeyError,ValueError,
            TypeError,subprocess.SubprocessError) as err:
        print(json.dumps({
          "result":"E3_REMEDIATE_PROVENANCE_NOT_VERIFIED",
          "reason":str(err) if isinstance(err,(Rejected,e2.EvidenceError))
                   else type(err).__name__,
          "independent_specialist_reviews":0,
          "r6g":"LOCKED_BY_SEQUENCE",
          "native_admission_gates_unmet":12,
          "production":"NOT_AUTHORIZED"
        },sort_keys=True))
        return 3

if __name__=="__main__":
    raise SystemExit(main())
