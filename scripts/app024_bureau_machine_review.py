#!/usr/bin/env python3
"""APP-024 nine ACTUAL source-checking machine functions, not Bureau signers.

Each role executes checks and writes SHA-pinned evidence. It does not pretend
to be a distinct AI agent or independent review/custody/certification.
"""
from __future__ import annotations
import argparse
import ast
import json
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from scripts import roadmap_guardian as guardian
from scripts import app_research_guardian as research

PARENT="45bac739f00deb710f8a6065d3ef616c810b8476"
APP="examples/apps/app024_verified_concurrent_resume/"
GATE="governance/NORDOI_APP024_VERIFIED_RESUME_RESEARCH_V1.json"
WF=".github/workflows/nordoi_app024_verified_concurrent_resume.yml"
ROLES=guardian.ROLE_IDS


def need(ok,reason):
    if not ok:raise ValueError(reason)


def clauses(path,*required):
    p=ROOT/path
    need(p.is_file() and not p.is_symlink(),"SOURCE_MISSING:"+path)
    data=p.read_text("utf-8")
    for part in required:
        need(part in data,"SOURCE_CONTRACT_MISSING:"+path+":"+part)
    return "SOURCE_CLAUSES_"+str(len(required))+"_PASS"


def frozen(*paths):
    r=subprocess.run(["git","diff","--exit-code",PARENT,"--",*paths],
                     cwd=ROOT,capture_output=True,timeout=30)
    need(r.returncode==0,"CERTIFIED_PARENT_SOURCE_CHANGED")
    return "C20_PARENT_IDENTICAL_PASS"


def run(role):
    old=guardian.audit(ROOT)
    track=research.audit(ROOT)
    need(old["REVIEWS_SUBMITTED"]==0 and
         old["BUREAU_CERTIFICATION"]=="PENDING" and
         old["R6G"]=="LOCKED_BY_SEQUENCE" and
         track["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED",
         "FORMAL_BUREAU_HISTORY_CHANGED")
    plan=json.loads((ROOT/GATE).read_text("utf-8"))
    need(plan["schema"]=="NORDOI_APP024_VERIFIED_PORTABLE_CONCURRENT_RESUME_V1"
         and plan["parent_exact_sha"]==PARENT
         and plan["new_tests_per_os"]==24
         and plan["prior_regressions_per_os"]==878
         and plan["total_tests_per_os"]==902
         and plan["max_native_in_flight"]==2
         and plan["formal_bureau_certification"]=="NOT_CERTIFIED"
         and plan["r6g"]=="LOCKED_BY_SEQUENCE"
         and plan["native_admission_gates_unmet"]==12
         and plan["production"]=="NOT_AUTHORIZED",
         "APP024_RESEARCH_SCOPE_INVALID")
    verdict=["LEGACY_BUREAU_GUARDIAN=PASS_WITH_CONTROLS",
             "APP024_C20_RESEARCH_CONTRACT=PASS"]
    if role=="ATLAS":
        verdict += [
          clauses(APP+"run_app024.py",
                  "portable.verify(checkpoint,expected_checkpoint)",
                  "portable.restored_root_matches(",
                  "concurrent.native_job","auditor.gate_true(",
                  "ThreadPoolExecutor(max_workers=limit"),
          frozen("src/","Cargo.toml","Cargo.lock",
                 "examples/apps/app011_verified_resume/",
                 "examples/apps/app014_portable_checkpoint/",
                 "examples/apps/app023_bounded_concurrent_dag/")
        ]
    elif role=="FERRUM":
        verdict += [
          clauses(APP+"run_app024.py",
                  "concurrent.native_job","concurrent.native_job,index,node",
                  "check_history(","ENGINE_PIN_MISMATCH",
                  "SOURCE_PIN_MISMATCH"),
          clauses(APP+"test_app024.py",
                  "ENGINE=legacy.ENGINE","test_01_real_parallel_native_checkpoint_resume"),
          frozen("src/","Cargo.toml","Cargo.lock")
        ]
    elif role=="BOUNDARY":
        verdict += [
          clauses(APP+"run_app024.py",
                  "RESTORED_ROOT_OVERLAPS_CHECKPOINT",
                  "RESTORED_ROOT_OVERLAPS_SOURCE",
                  "portable.verify(checkpoint,expected_checkpoint)",
                  "portable.restored_root_matches(",
                  "pinned.engine_sha256(engine)==engine_sha"),
          clauses(APP+"test_app024.py",
                  "test_13_wrong_external_pin_denied_before_effect",
                  "test_15_extra_unverified_job_blocks_resume",
                  "test_20_unrestored_directory_denied")
        ]
    elif role=="CIPHER":
        verdict += [
          clauses(APP+"run_app024.py",
                  "hashlib.sha256(raw).hexdigest()==manifest",
                  "pinned.source_tree_sha256(source)==source_sha",
                  "portable.verify(portable_dir,sha)"),
          clauses("RoadMap_APP024.md",
                  "aucune signature", "NOT_CERTIFIED","PRODUCTION=NOT_AUTHORIZED")
        ]
    elif role=="LEDGER":
        verdict += [
          clauses(APP+"run_app024.py",
                  "native_audit.verify_native_record(",
                  "portable.native_inventory(restored,events)",
                  "auditor.audit(restored,plan,joined,plan_sha)",
                  "json_line(new_events[k],newline)",
                  "HISTORICAL_NATIVE_BYTES_CHANGED"),
          clauses(APP+"test_app024.py",
                  "test_05_full_independent_app010_audit_passes",
                  "test_11_real_p27_files_independently_verified",
                  "test_12_historical_portable_prefix_untouched")
        ]
    elif role=="CAPSTONE":
        verdict += [
          clauses(APP+"run_app024.py",
                  "if not drain():","publication",
                  "CHECKPOINT_UNCERTAIN_JOB_REFUSED",
                  "effect_started","return 3"),
          clauses(APP+"test_app024.py",
                  "test_22_real_native_error_after_checkpoint_no_child_admitted",
                  "test_23_replaying_after_clean_success_is_denied")
        ]
    elif role=="ADVERSARY":
        src=(ROOT/APP/"test_app024.py").read_text("utf-8")
        names=[x.name for x in ast.walk(ast.parse(src))
               if isinstance(x,ast.FunctionDef) and x.name.startswith("test_")]
        need(len(names)==24 and len(set(names))==24,
             "TWENTYFOUR_REAL_NATIVE_TESTS_REQUIRED")
        verdict += ["APP024_24_UNITTEST_METHODS_IN_AST=PASS",
         clauses(APP+"test_app024.py",
                 "test_14_modified_restored_native_file_denied_before_effect",
                 "test_16_engine_pin_mismatch_denied",
                 "test_18_missing_portable_marker_denied",
                 "test_19_replaced_checkpoint_plan_denied",
                 "test_22_real_native_error_after_checkpoint_no_child_admitted")
        ]
    elif role=="VERITY":
        verdict += [
          clauses(WF,PARENT,
                  "macos-15-intel","macos-15","ubuntu-latest","windows-latest",
                  "cargo build --offline --bin nordoi",
                  "app024_verified_concurrent_resume|test_app024.py|24",
                  'test "$index" -eq 26','test "$total" -eq 902'),
          clauses(APP+"demo_app024.py",
                  "assert len(checks)==14",
                  "APP024_REAL_RUST_PORTABLE_CONCURRENT_RESUME=PASS")
        ]
    elif role=="GUARDIAN":
        for action in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            need(guardian.deny_transition(action)["TRANSITION"]=="BLOCK",
                 "FORBIDDEN_PROMOTION_ALLOWED")
        verdict += ["R6F_AND_R6G_TRANSITIONS_STILL_BLOCKED=PASS",
          frozen("RoadMap.md","Bureau.md",
                 "scripts/roadmap_guardian.py",
                 "scripts/app_research_guardian.py",
                 "governance/NORDOI_BUREAU_PLAN_v1.json"),
          clauses("RoadMap_APP024.md",
                  "APP024_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED",
                  "R6G=LOCKED_BY_SEQUENCE","MERGE=NONE")
        ]
    else:raise ValueError("ROLE_UNKNOWN")
    return verdict


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--role",choices=ROLES,required=True)
    p.add_argument("--proof-dir",type=Path,required=True)
    a=p.parse_args()
    sha=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,
           capture_output=True,check=True,text=True,timeout=30).stdout.strip()
    receipt={"schema":"NORDOI_APP024_MACHINE_BUREAU_REVIEW_V1",
      "role":a.role,"exact_head_sha":sha,"parent_exact_sha":PARENT,
      "actor":"ACTUAL_MACHINE_FUNCTION_NOT_INDEPENDENT_AI_AGENT",
      "formal_bureau_certification":"NOT_CERTIFIED",
      "historical_r6f":"IN_PROGRESS_UNCERTIFIED",
      "r6g":"LOCKED_BY_SEQUENCE",
      "native_gates_unmet":12,"production":"NOT_AUTHORIZED"}
    try:
        receipt["checks"]=run(a.role)
        receipt["status"]="PASS_WITH_CONTROLS_RESEARCH_ONLY"
        exit_code=0
    except (ValueError,guardian.Blocked,OSError,TypeError,KeyError,SyntaxError) as e:
        receipt["checks"]=[]
        receipt["status"]="BLOCK"
        receipt["error"]=str(e)
        exit_code=2
    a.proof_dir.mkdir(parents=True,exist_ok=True)
    (a.proof_dir/("APP024_BUREAU_MACHINE_"+a.role+".json")).write_text(
       json.dumps(receipt,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))
    return exit_code


if __name__=="__main__":
    raise SystemExit(main())
