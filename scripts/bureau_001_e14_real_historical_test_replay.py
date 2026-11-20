#!/usr/bin/env python3
"""NORDOI R6F/E14 actual per-file historical replay; no native promotion.

Audit validates source/contract without executing test cases. Exercise runs
every pinned 24 Rust + 33 Python historical test file in a separate real OS
subprocess, with raw logs, checksums, execution counts and explicit skips.
Result success is always BLOCK exit 2; failed or incomplete evidence exits 3.
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

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e13_retrospective_coverage as e13
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E14_REAL_HISTORICAL_TEST_REPLAY.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E14-REAL-HISTORICAL-REPLAY-V1"
PARENT="4da31db4363931606530f130b4d5330510a88184"
EVIDENCE_SCHEMA="NORDOI-G03-R6F-E14-ACTUAL-HISTORICAL-TEST-REPLAY-EVIDENCE-V1"
RUST_SUMMARY=re.compile(rb"test result:\s*(ok|FAILED)\.\s*(\d+)\s+passed;\s*(\d+)\s+failed;\s*(\d+)\s+ignored",re.I)
PY_RAN=re.compile(rb"Ran\s+(\d+)\s+tests?\s+in\s+[\d.]+s")
PY_SKIP=re.compile(rb"OK\s*\(skipped=(\d+)\)",re.I)
class Blocked(ValueError):pass
def require(ok,reason):
    if not ok:raise Blocked(reason)
def digest(raw):
    return hashlib.sha256(raw).hexdigest()
def canonical(v):
    return (json.dumps(v,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode()
def denied(reason,**extra):
    d={"result":"BLOCK","reason":reason,"human_testers":0,
       "founder_manual_test_duty":False,"external_witness_receipts":0,
       "formal_independent_bureau_reviews":0,
       "native_admission_gates_unmet":12,
       "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
       "production":"NOT_AUTHORIZED"}
    d.update({k:v for k,v in extra.items() if k not in d})
    return d

def audit(root,head):
    old=e13.audit(root,head)
    c=e3.strictly_load(e13.r1.raw(root,CONTRACT))
    require(old["result"]=="BLOCK" and old["native_admission_gates_unmet"]==12
       and c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
       and c["parent_e13_exact_head"]==PARENT
       and c["prior_e13"]=={
           "specialized_run_id":44710468140,"specialized_success":4,
           "rust_run_id":44710468035,"rust_success":5,
           "e8_regression_run_id":44710468200,"e8_regression_success":4},
       "E14_PARENT_E13_EXACT_C20_INCORRECT")
    require(c["frozen_source_manifest"]==e13.r1.MANIFEST
       and c["expected_rust_file_targets"]==24
       and c["expected_python_file_targets"]==33
       and c["expected_total_targets"]==57
       and c["machine_replay_policy"]==
           "REAL_SUBPROCESS_PER_PINNED_FILE_NO_MOCKED_RESULT"
       and c["python_command_template"]==[
           "python","-B","-m","unittest","discover","-s",
           "research","-p","<exact pinned file basename>","-v"]
       and c["rust_command_template"]==[
           "cargo","test","--offline","--release","--test",
           "<exact pinned test stem>","--","--nocapture","--test-threads=1"],
       "E14_57_TARGET_REPLAY_COMMANDS_INCORRECT")
    require(c["platforms"]==[
        "macos-15-intel","macos-15","ubuntu-latest","windows-latest"]
       and c["skipped_cases_must_not_be_counted_as_passed"] is True
       and c["zero_test_target_is_failure"] is True
       and c["nonzero_return_code_is_failure"] is True
       and c["capture_only_real_process_output"] is True
       and c["allow_log_fabrication"] is False,
       "E14_REPLAY_CAPTURE_OR_PLATFORM_POLICY_INVALID")
    require(c["human_testers_allowed"] is False
       and c["founder_manual_test_duty"] is False
       and c["technical_execution"]=="AI_AND_AUTOMATION_ONLY"
       and c["independent_bureau_reviews_completed"]==0
       and c["external_custodian"]=="NOT_ENROLLED"
       and c["owner_oob_root"]=="NOT_AUTHENTICATED"
       and c["external_witness_receipts"]==0
       and c["independent_quorum"]=="NOT_PROVEN"
       and c["global_monotonic_antirollback"]=="NOT_VERIFIED"
       and c["native_admission_gates_unmet"]==12
       and c["r6f"]=="IN_PROGRESS_UNCERTIFIED"
       and c["r6g"]=="LOCKED_BY_SEQUENCE"
       and c["production"]=="NOT_AUTHORIZED"
       and c["merge_authorized"] is False
       and c["pre_ci_verdict"]=="IN_PROGRESS_UNQUALIFIED",
       "E14_FABRICATED_EXTERNAL_AUTHORITY_OR_HUMAN_TESTS")
    groups,pins=e13.verify_sources(root,head)
    require(len(groups["rust"])==24 and len(groups["python_test"])==33,
            "E14_FROZEN_TARGET_SET_NOT_COMPLETE")
    return groups,pins

def commands(groups):
    tasks=[]
    for path in groups["python_test"]:
        tasks.append((path,[sys.executable,"-B","-m","unittest",
              "discover","-s","research","-p",Path(path).name,"-v"]))
    for path in groups["rust"]:
        tasks.append((path,["cargo","test","--offline","--release",
              "--test",Path(path).stem,"--","--nocapture","--test-threads=1"]))
    require(len(tasks)==57 and len({p for p,_ in tasks})==57,
            "E14_NOT_57_DISTINCT_HISTORICAL_TEST_TARGETS")
    return tasks

def parse_count(path,raw,return_code):
    if path.startswith("research/"):
        ran=list(PY_RAN.finditer(raw))
        skip=list(PY_SKIP.finditer(raw))
        require(len(ran)==1,"E14_PYTHON_UNAMBIGUOUS_COUNT_ABSENT:"+path)
        total=int(ran[0][1])
        ignored=int(skip[-1][1]) if skip else 0
        require(0<=ignored<=total,"E14_PYTHON_SKIP_COUNT_INVALID:"+path)
        ok=(return_code==0 and bool(re.search(
             rb"(?m)^\s*OK(?:\s*\(skipped=\d+\))?\s*$",raw)))
        return {"ran":total,"passed":total-ignored if ok else 0,
                "skipped":ignored,"failed":0 if ok else total-ignored,
                "success":ok and total>0}
    matches=list(RUST_SUMMARY.finditer(raw))
    require(len(matches)==1,"E14_RUST_UNAMBIGUOUS_COUNT_ABSENT:"+path)
    match=matches[0]
    passed,failed,ignored=(int(match[i]) for i in (2,3,4))
    ok=(return_code==0 and match[1].lower()==b"ok" and failed==0)
    return {"ran":passed+failed+ignored,"passed":passed,
            "skipped":ignored,"failed":failed,
            "success":ok and passed+failed+ignored>0}

def run_target(root,path,cmd,out,pin):
    require(path.startswith(("tests/","research/")),
            "E14_UNAUTHORIZED_TEST_TARGET")
    start=time.monotonic()
    try:
        p=subprocess.run(cmd,cwd=root,stdout=subprocess.PIPE,
             stderr=subprocess.STDOUT,timeout=420,check=False)
        raw,code,problem=p.stdout,p.returncode,None
    except subprocess.TimeoutExpired as e:
        raw=(e.stdout or b"")+(e.stderr or b"")
        code,problem=124,"TIMEOUT"
    except OSError as e:
        raw=("OS_ERROR:"+type(e).__name__+"\n").encode()
        code,problem=127,"OS_ERROR"
    require(len(raw)<=12_000_000,"E14_OVERSIZED_REAL_TEST_OUTPUT:"+path)
    logname=path.replace("/","_").replace(".rs",".log").replace(".py",".log")
    require(logname.endswith(".log") and "/" not in logname,
            "E14_LOG_FILENAME_UNSAFE")
    (out/logname).write_bytes(raw)
    try: stats=parse_count(path,raw,code)
    except Blocked as e:
        stats={"ran":0,"passed":0,"skipped":0,
               "failed":0,"success":False}
        problem=problem or str(e)
    if path=="tests/g03_r4m_pinned_code_identity_lab.rs" and \
       sys.platform=="darwin" and stats["passed"]<6:
        stats["success"]=False
        problem="E14_NATIVE_MAC_R4M_SIX_SERIAL_CASES_NOT_PROVEN"
    return {
       "target_path":path,
       "kind":"rust" if path.startswith("tests/") else "python",
       "source_git_blob_sha1":pin["git_blob_sha1"],
       "real_command":cmd,"return_code":code,
       "elapsed_seconds":round(time.monotonic()-start,3),
       "ran_tests":stats["ran"],"passed_tests":stats["passed"],
       "skipped_tests":stats["skipped"],"failed_tests":stats["failed"],
       "success":stats["success"],"error":problem,
       "log_filename":logname,"raw_output_sha256":digest(raw),
       "log_sha256":digest((out/logname).read_bytes()),"raw_log_bytes":len(raw)
    }

def validate_manifest(out,head,targets=57):
    p=out/"MANIFEST.json"
    require(p.is_file() and not p.is_symlink(),"E14_NO_REAL_MANIFEST")
    m=e3.strictly_load(p.read_bytes())
    require(m["schema"]==EVIDENCE_SCHEMA and m["exact_source_head"]==head
       and m["expected_target_count"]==targets
       and m["human_testers"]==0
       and m["native_admission_gates_unmet"]==12
       and m["independent_bureau_reviews"]==0
       and m["r6g"]=="LOCKED_BY_SEQUENCE"
       and m["production"]=="NOT_AUTHORIZED",
       "E14_FALSE_MANIFEST_CUSTODY_OR_HEAD")
    records=m["records"]
    require(len(records)==targets
       and len({r["target_path"] for r in records})==targets
       and len({r["log_filename"] for r in records})==targets,
       "E14_INCOMPLETE_OR_DUPLICATE_TARGET_MANIFEST")
    require({x.name for x in out.iterdir()}==
       {"MANIFEST.json"}|{r["log_filename"] for r in records},
       "E14_MISSING_OR_INJECTED_LOG")
    for r in records:
        p=out/r["log_filename"]
        require(p.parent==out and not p.is_symlink() and p.is_file(),
                "E14_LOG_SYMLINK_OR_MISSING")
        raw=p.read_bytes()
        require(digest(raw)==r["log_sha256"]==r["raw_output_sha256"]
             and len(raw)==r["raw_log_bytes"],
             "E14_ACTUAL_RAW_LOG_DIGEST_CHANGED")
        require(r["passed_tests"]+r["skipped_tests"]
             +r["failed_tests"]==r["ran_tests"],
             "E14_SKIP_PASS_FAILURE_SUM_NOT_RECONCILED")
        require(r["success"] is True and r["return_code"]==0
             and r["ran_tests"]>0 and r["failed_tests"]==0,
             "E14_FAILED_OR_ZERO_EXECUTED_LEGACY_TARGET")
    return m

def exercise(root,head,dest):
    groups,pins=audit(root,head)
    out=Path(dest).absolute()
    require(not out.exists() and out.parent.is_dir()
            and not out.parent.is_symlink(),
            "E14_REAL_REPLAY_DESTINATION_MUST_BE_FRESH")
    out.mkdir(mode=0o700)
    entries=[run_target(root,p,cmd,out,pins[p])
             for p,cmd in commands(groups)]
    successful=sum(r["success"] for r in entries)
    m={"schema":EVIDENCE_SCHEMA,"exact_source_head":head,
       "host_os":platform.system(),"host_arch":platform.machine(),
       "python_version":platform.python_version(),
       "expected_target_count":57,"actual_target_records":len(entries),
       "passed_target_files":successful,"failed_or_empty_target_files":57-successful,
       "rust_target_files":sum(r["kind"]=="rust" for r in entries),
       "python_target_files":sum(r["kind"]=="python" for r in entries),
       "test_cases_executed":sum(r["passed_tests"]+r["failed_tests"] for r in entries),
       "test_cases_passed":sum(r["passed_tests"] for r in entries),
       "test_cases_skipped":sum(r["skipped_tests"] for r in entries),
       "real_subprocess_target_count":len(entries),
       "independent_bureau_reviews":0,"human_testers":0,
       "native_admission_gates_unmet":12,
       "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
       "production":"NOT_AUTHORIZED","records":entries}
    (out/"MANIFEST.json").write_bytes(canonical(m))
    require(successful==57,"E14_SOME_HISTORICAL_TARGETS_FAILED_REPLAY")
    validate_manifest(out,head)
    require(e13.e3.check_head(root,head)==head,
            "E14_SOURCE_HEAD_CHANGED_BY_EXPERIMENT")
    return denied("E14_REAL_57_HISTORICAL_FILE_TARGETS_REPLAYED_RESEARCH_ONLY",
         exact_source_head=head,actual_target_processes=57,
         real_rust_test_targets=24,real_python_test_targets=33,
         actual_test_cases_passed=m["test_cases_passed"],
         actual_test_cases_skipped=m["test_cases_skipped"],
         manifest_sha256=digest((out/"MANIFEST.json").read_bytes()),
         actual_independent_external_custody_proven=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    mode=p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--audit",action="store_true")
    mode.add_argument("--exercise",action="store_true")
    p.add_argument("--root",default=str(ROOT))
    p.add_argument("--expected-head",required=True)
    p.add_argument("--output-dir")
    a=p.parse_args(argv)
    try:
        root=Path(a.root).resolve(strict=True)
        if a.audit:
            groups,_=audit(root,a.expected_head)
            d=denied("E14_ONLY_SOURCE_AND_EXECUTION_CONTRACT_AUDITED",
                 exact_source_head=a.expected_head,
                 expected_rust_targets=len(groups["rust"]),
                 expected_python_targets=len(groups["python_test"]),
                 actual_tests_executed_by_audit=0)
        else:
            require(a.output_dir is not None,
                    "E14_REAL_REPLAY_REQUIRES_FRESH_OUTPUT_DIR")
            d=exercise(root,a.expected_head,a.output_dir)
        status=2
    except (Blocked,e13.Blocked,e13.e12.Blocked,
            e3.Rejected,e13.r1.Blocked,
            ValueError,KeyError,TypeError,OSError,
            subprocess.SubprocessError) as e:
        d=denied("E14_REAL_HISTORICAL_REPLAY_NOT_QUALIFIED",
            error_type=type(e).__name__,
            error_detail=str(e) if isinstance(e,Blocked) else None)
        status=3
    sys.stdout.buffer.write(canonical(d))
    sys.stdout.buffer.flush()
    return status

if __name__=="__main__":raise SystemExit(main())
