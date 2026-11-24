#!/usr/bin/env python3
"""APP-025: nine executed Bureau MACHINE role checks; NOT independent agents.

Each role checks actual source and frozen governance and issues a SHA-pinned
machine receipt. Source inspections do not establish formal Bureau reviews,
external trust custody, secure antirollback, or native production admission.
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

PARENT="0cd342d32c4dac9ac99ace569244df0c682a9af6"
APP="examples/apps/app025_interrupted_dag_forensics/"
GATE="governance/NORDOI_APP025_INTERRUPTED_DAG_FORENSICS_V1.json"
WF=".github/workflows/nordoi_app025_interrupted_forensics.yml"
ROLES=guardian.ROLE_IDS


def need(v,reason):
    if not v:raise ValueError(reason)


def clauses(file,*required):
    path=ROOT/file
    need(path.is_file() and not path.is_symlink(),"SOURCE_NOT_REGULAR:"+file)
    raw=path.read_text("utf-8")
    for word in required:
        need(word in raw,"REQUIRED_CLAUSE_MISSING:"+file+":"+word)
    return "SOURCE_CLAUSES_"+str(len(required))+"_PASS"


def frozen(*paths):
    r=subprocess.run(["git","diff","--exit-code",PARENT,"--",*paths],
                     cwd=ROOT,capture_output=True,timeout=30)
    need(r.returncode==0,"PROTECTED_PARENT_SOURCES_MODIFIED")
    return "FROZEN_PARENT_UNCHANGED_PASS"


def role_checks(role):
    historic=guardian.audit(ROOT)
    app=research.audit(ROOT)
    need(historic["REVIEWS_SUBMITTED"]==0 and
         historic["BUREAU_CERTIFICATION"]=="PENDING" and
         historic["R6G"]=="LOCKED_BY_SEQUENCE" and
         app["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED",
         "FORMAL_BUREAU_HISTORY_MUTATED")
    gate=json.loads((ROOT/GATE).read_text("utf-8"))
    need(gate["schema"]=="NORDOI_APP025_INTERRUPTED_DAG_FORENSICS_RESEARCH_V1"
         and gate["parent_exact_sha"]==PARENT
         and gate["new_tests_per_os"]==24
         and gate["prior_app_regressions_per_os"]==902
         and gate["total_tests_per_os"]==926
         and gate["total_expected_four_os_tests"]==3704
         and gate["formal_bureau_certification"]=="NOT_CERTIFIED"
         and gate["r6g"]=="LOCKED_BY_SEQUENCE"
         and gate["native_admission_gates_unmet"]==12
         and gate["production"]=="NOT_AUTHORIZED",
         "APP025_GOVERNANCE_GATE_INVALID")
    checks=["HISTORICAL_ROADMAP_GUARDIAN=PASS_WITH_CONTROLS",
            "APP025_EXACT_C20_PARENT_AND_RESEARCH_GATE=PASS"]
    if role=="ATLAS":
        checks += [
          clauses(APP+"audit_app025.py",
              "portable.verify(checkpoint,checkpoint_pin)",
              "audit.audit(root,plan,joined,plan_sha)",
              "ORIGINAL_CHECKPOINT_REUSABLE",
              "CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT",
              "COMPLETE_VERIFIED","UNCERTAIN_BLOCKED"),
          frozen("src/","Cargo.toml","Cargo.lock",
                 "examples/apps/app024_verified_concurrent_resume/",
                 "examples/apps/app014_portable_checkpoint/")
        ]
    elif role=="FERRUM":
        source=(ROOT/APP/"audit_app025.py").read_text("utf-8")
        tree=ast.parse(source)
        imports=[n.module for n in ast.walk(tree)
                 if isinstance(n,ast.ImportFrom)]
        names=[alias.name for n in ast.walk(tree) if isinstance(n,ast.Import)
               for alias in n.names]
        need("subprocess" not in imports and "subprocess" not in names,
             "FORENSIC_RUNTIME_MUST_NOT_INVOKE_RUST_PROCESS")
        checks += ["NO_SUBPROCESS_IMPORT_IN_RUNTIME_AST=PASS",
                   clauses(APP+"test_app025.py",
                           "integration.RealPortableConcurrentResume",
                           "test_02_real_complete_concurrent_dag_is_verified"),
                   frozen("src/","Cargo.toml","Cargo.lock")
        ]
    elif role=="BOUNDARY":
        checks += [
          clauses(APP+"audit_app025.py",
              "native.real_directory(root",
              "native.bounded_regular_file(suffix",
              "SUFFIX_EXTERNAL_PIN_MISMATCH",
              "ROOT","STOP_NO_AUTOMATIC_NATIVE_REPLAY"),
          clauses(APP+"test_app025.py",
              "test_08_concurrent_orphan_without_suffix_is_blocked",
              "test_10_truncated_suffix_blocks_even_with_corresponding_real_files",
              "test_15_additional_unlogged_native_job_blocks")
        ]
    elif role=="CIPHER":
        checks += [
          clauses(APP+"audit_app025.py",
              "portable.verify(checkpoint,checkpoint_pin)",
              "must(sha256(raw)==suffix_pin",
              "combined_events_sha256",
              "global_antirollback",
              "external_pin_authentication"),
          clauses("RoadMap_APP025.md",
              "SHA-256", "NOT_CERTIFIED","PRODUCTION=NOT_AUTHORIZED")
        ]
    elif role=="LEDGER":
        checks += [
          clauses(APP+"audit_app025.py",
              "audit.event_records(joined)",
              "audit.audit(root,plan,joined,plan_sha)",
              "verified_native_jobs",
              "original_events",
              "CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT"),
          clauses(APP+"test_app025.py",
              "test_06_completed_history_preserves_real_native_file_bytes",
              "test_07_clean_partial_audit_writes_no_user_data",
              "test_14_corrupted_new_native_bytes_block")
        ]
    elif role=="CAPSTONE":
        checks += [
          clauses(APP+"audit_app025.py",
              "CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT",
              "SEAL_NEW_CHECKPOINT_BEFORE_ANY_FURTHER_RESUME",
              "UNCERTAIN_BLOCKED",
              "STOP_NO_AUTOMATIC_NATIVE_REPLAY"),
          clauses(APP+"test_app025.py",
              "test_17_native_error_cannot_be_promoted_to_checkpoint",
              "test_24_external_hash_does_not_claim_antirollback_or_signature")
        ]
    elif role=="ADVERSARY":
        ast_tree=ast.parse((ROOT/APP/"test_app025.py").read_text("utf-8"))
        tests=[n.name for n in ast.walk(ast_tree)
               if isinstance(n,ast.FunctionDef) and n.name.startswith("test_")]
        need(len(tests)==24 and len(set(tests))==24,
             "APP025_TWENTYFOUR_REAL_NATIVE_TESTS_REQUIRED")
        checks += ["APP025_AST_TWENTYFOUR_TESTS=PASS",
                   clauses(APP+"test_app025.py",
                     "test_18_duplicate_native_event_blocks_even_with_new_pin",
                     "test_19_wrong_event_step_blocks_even_with_rehashed_suffix",
                     "test_20_wrong_real_native_decision_blocks",
                     "test_22_injected_json_duplicate_key_cannot_pass_even_with_pin")
        ]
    elif role=="VERITY":
        checks += [
          clauses(WF,PARENT,
             "macos-15-intel","macos-15",
             "ubuntu-latest","windows-latest",
             "cargo build --offline --bin nordoi",
             "app025_interrupted_dag_forensics|test_app025.py|24",
             'test "$index" -eq 27',
             'test "$total" -eq 926'),
          clauses(APP+"demo_app025.py",
             "assert len(claims)==14",
             "APP025_REAL_RUST_INTERRUPTED_DAG_FORENSICS=PASS")
        ]
    elif role=="GUARDIAN":
        for action in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            need(guardian.deny_transition(action)["TRANSITION"]=="BLOCK",
                 "PRIVILEGED_PROMOTION_NOT_BLOCKED")
        checks += ["THREE_PRIVILEGED_TRANSITIONS_BLOCKED=PASS",
            frozen("RoadMap.md","Bureau.md",
                   "scripts/roadmap_guardian.py",
                   "scripts/app_research_guardian.py",
                   "governance/NORDOI_BUREAU_PLAN_v1.json"),
            clauses("RoadMap_APP025.md",
                   "APP025_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED",
                   "R6G=LOCKED_BY_SEQUENCE","MERGE=NONE")
        ]
    else:raise ValueError("UNRECOGNIZED_BUREAU_ROLE")
    return checks


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--role",choices=ROLES,required=True)
    p.add_argument("--proof-dir",type=Path,required=True)
    a=p.parse_args()
    head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,
             check=True,capture_output=True,text=True,timeout=30).stdout.strip()
    receipt={
      "schema":"NORDOI_APP025_MACHINE_BUREAU_REVIEW_V1",
      "role":a.role,"source_sha":head,"parent_exact_sha":PARENT,
      "review_mode":"ACTUAL_DETERMINISTIC_MACHINE_ROLE_NOT_INDEPENDENT_AI_AGENT",
      "formal_bureau_certification":"NOT_CERTIFIED",
      "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
      "native_gates_unmet":12,
      "production":"NOT_AUTHORIZED"
    }
    try:
        receipt["checks"]=role_checks(a.role)
        receipt["status"]="PASS_WITH_CONTROLS_RESEARCH_ONLY"
        rc=0
    except (ValueError,guardian.Blocked,OSError,KeyError,TypeError,SyntaxError) as e:
        receipt["checks"]=[]
        receipt["status"]="BLOCK"
        receipt["error"]=str(e)
        rc=2
    a.proof_dir.mkdir(parents=True,exist_ok=True)
    (a.proof_dir/("APP025_MACHINE_"+a.role+".json")).write_text(
       json.dumps(receipt,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))
    return rc

if __name__=="__main__":
    raise SystemExit(main())
