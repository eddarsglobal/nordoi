#!/usr/bin/env python3
"""R6F/E15: verify actual four-OS E14 artifact bundles and 228 raw logs.

CI must download the *named* artifacts from the exact frozen E14 GitHub run.
Checks do not confer independent GitHub/owner/external administrative custody.
Success exits 2 BLOCK; missing or falsified evidence exits 3.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e14_real_historical_test_replay as e14
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E15_FOUR_OS_RECEIPTS.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E15-FOUR-OS-RECEIPT-RECONCILIATION-V1"
RESULT_SCHEMA="NORDOI-G03-R6F-E15-FOUR-OS-ACTUAL-RECEIPT-VERIFY-V1"
PARENT="deb34302acdbe18563ab06eb10d852865e3b3c6a"
EXPECTED=[
 ("macos-15-intel",14618857785,"5eb067fc94695dc1a99747afedbcfb95ef6ac615e8de7d50d9198f6f14d606c4","Darwin","x86_64",1039,3),
 ("macos-15",14618821978,"b84511f0065f440182c4e1f46944e635588691761b29c78f1f86bbeab7fc39c7","Darwin","arm64",1039,3),
 ("ubuntu-latest",14617774638,"cb5ce82242c912edc037c3e9678af2f54a2fb4659fbb9bb59f51c44514211977","Linux","x86_64",881,19),
 ("windows-latest",14617984265,"6ee300dd620ff8e3fa7f8b005d12b378f0a46a5551c74e1c2f0539ccae53ddd2","Windows","AMD64",670,226)
]
class Blocked(ValueError):pass

def require(ok,reason):
    if not ok:raise Blocked(reason)

def deny(reason,**kwargs):
    d={"result":"BLOCK","reason":reason,"human_testers":0,
       "founder_manual_test_duty":False,
       "formal_independent_specialist_reviews":0,
       "external_witness_receipts":0,
       "native_admission_gates_unmet":12,
       "r6f":"IN_PROGRESS_UNCERTIFIED",
       "r6g":"LOCKED_BY_SEQUENCE","production":"NOT_AUTHORIZED"}
    d.update({k:v for k,v in kwargs.items() if k not in d})
    return d

def audit(root,head):
    groups,pins=e14.audit(root,head)
    c=e3.strictly_load(e14.e13.r1.raw(root,CONTRACT))
    require(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
      and c["parent_e14_exact_head"]==PARENT
      and c["prior_e14"]=={
          "specialized_run_id":44713831052,"specialized_success":4,
          "rust_run_id":44713831212,"rust_success":5,
          "e8_run_id":44713830975,"e8_success":4},
      "E15_GREEN_E14_PARENT_OR_C20_INVALID")
    expected_roles=[
       (name,aid,sha,system,arch,passed,skipped)
       for name,aid,sha,system,arch,passed,skipped in EXPECTED]
    observed_roles=[
       (x["id"],x["github_artifact_id"],x["sha256_manifest"],
        x["recorded_os"],x["recorded_arch"],
        x["expected_passed_cases"],x["expected_skipped_cases"])
       for x in c["artifacts"]]
    require(observed_roles==expected_roles and
       all(x["github_artifact_name"]==
          "NORDOI_E14_REAL_57_TEST_TARGETS_"+x["id"]
          for x in c["artifacts"]),
       "E15_UNPINNED_PRIOR_ARTIFACT_ID_HASH_OR_PLATFORMS")
    require(c["expected_platforms"]==4
      and c["expected_target_files_per_platform"]==57
      and c["expected_total_raw_logs"]==228
      and c["historical_rust_targets_per_platform"]==24
      and c["historical_python_targets_per_platform"]==33
      and c["actual_subprocess_pid_evidence_in_e14"]==
           "NOT_CAPTURED_PER_TARGET",
       "E15_228_LOG_COVERAGE_OR_PID_EVIDENCE_FABRICATED")
    require(c["negative_controls"]==[
      "SWAP_TWO_PLATFORM_BUNDLES","ALTER_ONE_LOG_BYTE",
      "REHASH_LOCAL_MANIFEST_WITHOUT_PINNED_REFERENCE",
      "REMOVE_ONE_LOG_FILE","INCORRECT_TEST_CASE_SKIP_CLASSIFICATION",
      "INJECT_UNPINNED_SOURCE_TARGET","ASSERT_NATIVE_GATES_MET"],
      "E15_ADVERSARIAL_TEST_COVERAGE_INCOMPLETE")
    require(c["human_testers_allowed"] is False
      and c["founder_manual_test_duty"] is False
      and c["technical_execution"]=="AI_AND_AUTOMATION_ONLY"
      and c["independent_specialist_reviews"]==0
      and c["independent_external_witness_receipts"]==0
      and c["external_custodian"]=="NOT_ENROLLED"
      and c["owner_oob_root"]=="NOT_AUTHENTICATED"
      and c["global_monotonic_antirollback"]=="NOT_VERIFIED"
      and c["native_admission_gates_unmet"]==12
      and c["r6f"]=="IN_PROGRESS_UNCERTIFIED"
      and c["r6g"]=="LOCKED_BY_SEQUENCE"
      and c["production"]=="NOT_AUTHORIZED"
      and c["permission_to_merge"] is False
      and c["pre_ci_verdict"]=="IN_PROGRESS_UNQUALIFIED",
      "E15_AI_ONLY_OR_NATIVE_GUARD_UNJUSTIFIED_PROMOTION")
    return c,groups,pins

def validate_platform(pdir,description,targets,pins):
    """Actual bytes from the preceding E14 run, not newly reexecuted E14 tests."""
    require(pdir.is_dir() and not pdir.is_symlink(),
            "E15_MISSING_REAL_DOWNLOADED_GITHUB_ARTIFACT:"+description["id"])
    raw=(pdir/"MANIFEST.json").read_bytes()
    require(e14.digest(raw)==description["sha256_manifest"],
            "E15_CHANGED_OR_CROSS_SWAPPED_PINNED_MANIFEST:"+description["id"])
    prior=e14.validate_manifest(pdir,PARENT,57)
    require(prior["host_os"]==description["recorded_os"]
       and prior["host_arch"]==description["recorded_arch"]
       and prior["passed_target_files"]==57
       and prior["failed_or_empty_target_files"]==0
       and prior["rust_target_files"]==24
       and prior["python_target_files"]==33
       and prior["actual_target_records"]==57
       and prior["real_subprocess_target_count"]==57
       and prior["test_cases_passed"]==description["expected_passed_cases"]
       and prior["test_cases_skipped"]==description["expected_skipped_cases"],
       "E15_OS_NATIVE_SKIP_OR_CASE_COUNT_NOT_GROUNDED:"+description["id"])
    bypath={row["target_path"]:row for row in prior["records"]}
    require(set(bypath)==set(targets),
            "E15_INJECTED_OR_MISSING_GIT_PINNED_HISTORICAL_TARGET")
    rust=0
    py=0
    for path,expected in targets.items():
        row=bypath[path]
        require(row["source_git_blob_sha1"]==pins[path]["git_blob_sha1"],
                "E15_SOURCE_BLOB_SUBSTITUTION:"+path)
        command=row["real_command"]
        if path.startswith("tests/"):
            rust+=1
            require(row["kind"]=="rust" and command==expected,
                    "E15_RUST_TEST_COMMAND_REBIND_FAILED:"+path)
        else:
            py+=1
            require(row["kind"]=="python"
              and type(command) is list and len(command)==len(expected)
              and command[1:]==expected[1:]
              and type(command[0]) is str
              and bool(re.search(r"(?i)(?:^|[/\\])python(?:3(?:\.\d+)?)?(?:\.exe)?$",command[0])),
              "E15_PYTHON_REAL_TEST_COMMAND_REBIND_FAILED:"+path)
        raw_log=(pdir/row["log_filename"]).read_bytes()
        counted=e14.parse_count(path,raw_log,row["return_code"])
        require(counted["success"] is True
          and counted["ran"]==row["ran_tests"]
          and counted["passed"]==row["passed_tests"]
          and counted["skipped"]==row["skipped_tests"]
          and counted["failed"]==row["failed_tests"],
          "E15_LOCAL_LOG_WAS_MISCOUNTED_OR_FORGED:"+path)
    require(rust==24 and py==33,"E15_MISSING_TARGET_KIND")
    return {
        "platform":description["id"],
        "host_os_recorded_by_E14":prior["host_os"],
        "host_arch_recorded_by_E14":prior["host_arch"],
        "artifact_id_claim":"GITHUB_RUN_ID_PINNED_IN_WORKFLOW_NOT_EXTERNAL_ATTESTATION",
        "actual_manifest_sha256":e14.digest(raw),
        "verified_log_files":len(prior["records"]),
        "rust_targets":rust,"python_targets":py,
        "cases_passed":prior["test_cases_passed"],
        "cases_skipped":prior["test_cases_skipped"],
        "formal_external_witness":False
    }

def reconcile(root,head,folder):
    c,groups,pins=audit(root,head)
    root_artifacts=Path(folder).resolve(strict=True)
    require(root_artifacts.is_dir() and not root_artifacts.is_symlink(),
            "E15_ARTIFACT_BUNDLE_ROOT_NOT_A_DIRECTORY")
    require({p.name for p in root_artifacts.iterdir()}==
            {x["id"] for x in c["artifacts"]},
            "E15_NOT_EXACTLY_FOUR_PREDECESSOR_ARTIFACTS")
    tasks=dict(e14.commands(groups))
    require(len(tasks)==57,"E15_EXPECTED_57_TASKS_NOT_PRESENT")
    reports=[]
    for description in c["artifacts"]:
        reports.append(validate_platform(root_artifacts/description["id"],
                     description,tasks,pins))
    require(len(reports)==4 and
       sum(x["verified_log_files"] for x in reports)==228,
       "E15_REAL_CROSS_OS_LOGS_NOT_COMPLETE")
    require(e3.check_head(root,head)==head,
            "E15_HEAD_MUTATED_AFTER_READ")
    document={
      "schema":RESULT_SCHEMA,"source_head_e15":head,
      "parent_head_e14":PARENT,
      "source_github_e14_workflow_run_id":44713831052,
      "real_prior_ci_artifacts_rechecked":4,
      "genuine_prior_raw_logs_verified":228,
      "historical_target_files_per_os":57,
      "platforms":reports,
      "formal_external_reviews":0,
      "external_witness_receipts":0,
      "native_admission_gates_unmet":12,
      "r6g":"LOCKED_BY_SEQUENCE",
      "production":"NOT_AUTHORIZED",
      "human_testers":0,
      "trust_boundary":"GITHUB_ARTIFACT_RECHECK_NOT_INDEPENDENT_EXTERNAL_CUSTODY"
    }
    return document

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    acts=ap.add_mutually_exclusive_group(required=True)
    acts.add_argument("--audit",action="store_true")
    acts.add_argument("--exercise",action="store_true")
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--root",default=str(ROOT))
    ap.add_argument("--artifact-dir")
    ap.add_argument("--result-path")
    args=ap.parse_args(argv)
    try:
        root=Path(args.root).resolve(strict=True)
        if args.audit:
            c,groups,pins=audit(root,args.expected_head)
            response=deny("E15_ONLY_SOURCE_AND_PINNED_FOUR_ARTIFACT_CONTRACT",
                source_head=args.expected_head,artifacts_expected=4,
                historical_target_paths=len(e14.commands(groups)),
                logs_not_downloaded_by_audit=228)
        else:
            require(args.artifact_dir and args.result_path,
                "E15_EXERCISE_REQUIRES_ACTUAL_DOWNLOADED_ARTIFACTS_AND_RESULT")
            output=Path(args.result_path).absolute()
            require(not output.exists() and output.parent.is_dir()
                    and not output.parent.is_symlink(),
                    "E15_RESULT_PATH_MUST_NOT_ALREADY_EXIST")
            document=reconcile(root,args.expected_head,args.artifact_dir)
            output.write_bytes(e14.canonical(document))
            response=deny("E15_FOUR_PINNED_E14_ARTIFACTS_AND_228_REAL_LOGS_RECHECKED",
                exact_source_head=args.expected_head,
                actual_artifacts_rechecked=4,
                actual_prior_historical_log_files_verified=228,
                total_cases_passed=sum(
                   x["cases_passed"] for x in document["platforms"]),
                total_cases_skipped=sum(
                   x["cases_skipped"] for x in document["platforms"]),
                generated_result_sha256=e14.digest(output.read_bytes()),
                no_new_historical_test_executions_claimed=True,
                external_custodian_certified=False)
        code=2
    except (Blocked,e14.Blocked,e14.e13.Blocked,
            e14.e13.e12.Blocked,e3.Rejected,
            KeyError,ValueError,TypeError,OSError,
            RuntimeError,subprocess.SubprocessError) as exc:
        response=deny("E15_CROSS_OS_EVIDENCE_NOT_QUALIFIED",
            error_type=type(exc).__name__,
            error_detail=str(exc) if isinstance(exc,Blocked) else None)
        code=3
    sys.stdout.buffer.write(e14.canonical(response))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":
    import subprocess
    raise SystemExit(main())
