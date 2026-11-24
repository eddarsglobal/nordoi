#!/usr/bin/env python3
"""Deterministic role-specific checks, not independent Bureau certification."""
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
PARENT="e61ae006d6bab8ae3e139c8992d12e12e7cad96e"
APP=ROOT/"examples/apps/app027_verified_reconstructed_resume"
GATE=ROOT/"governance/NORDOI_APP027_RECONSTRUCTED_RESUME_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"resume_app027.py").read_text("utf-8")
    tests=(APP/"test_app027.py").read_text("utf-8")
    demo=(APP/"demo_app027.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app027_reconstructed_resume.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==950
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==938
         and g["total_expected_four_os_tests"]==3800 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT"),(source,"RECONSTRUCTED_CHAIN_MISMATCH"),(source,"run_app024.py")],
      "FERRUM": [(source,"resume.pinned.engine_sha256"),(tests,"test_01_three_missing_native_jobs"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"ATTEMPT_OVERLAPS_PROTECTED_TREE"),(source,"ATTEMPT_ALREADY_EXISTS"),(tests,"test_10_existing_attempt_no_overwrite")],
      "CIPHER": [(source,"EXTERNAL_PIN_REQUIRED"),(source,"portable.verify(new,new_pin)"),(source,"external_pin_authentication")],
      "LEDGER": [(source,"events[:len(old_events)]==old_events"),(source,"HISTORICAL_BYTES_CHANGED"),(tests,"test_03_preserve_history_and_original_packages")],
      "CAPSTONE": [(source,"ATTEMPT_OUTCOME_UNKNOWN_STOP_NO_REPLAY"),(source,"STOP_NO_AUTOMATIC_NATIVE_REPLAY"),(source,"EXPLICIT_RESEARCH_EXECUTION_REQUIRED")],
      "ADVERSARY": [(tests,"test_08_original_checkpoint_not_reconstruction"),(tests,"test_09_engine_or_source_wrong"),(tests,"test_11_completed_workspace_cannot_replay")],
      "VERITY": [(wf,'test "$total" -eq 950'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"formal_bureau_certification":"NOT_CERTIFIED"'),(source,'"global_antirollback":"NOT_PROVEN"')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and isinstance(n.func.value,ast.Name) and n.func.value.id=="subprocess"]
        need(len(calls)==1 and calls[0].func.attr=="run","UNEXPECTED_PROCESS_ENTRYPOINT")
        need(any(k.arg=="timeout" for k in calls[0].keywords),"TIMEOUT_REQUIRED")
        passed.append("ONE_BOUNDED_SUBPROCESS_ENTRYPOINT")
    if a.role=="ADVERSARY":
        need(sum(isinstance(n,ast.FunctionDef) and n.name.startswith("test_") for n in ast.walk(ast.parse(tests)))==12,"TEST_COUNT")
        passed.append("TWELVE_TESTS")
    if a.role=="GUARDIAN":
        for action in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            need(guardian.deny_transition(action)["TRANSITION"]=="BLOCK","PRIVILEGED_TRANSITION")
        passed.append("THREE_PRIVILEGED_TRANSITIONS_BLOCKED")
    head=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()
    receipt={"role":a.role,"source_sha":head,"parent_exact_sha":PARENT,
        "review_mode":"DETERMINISTIC_MACHINE_NOT_INDEPENDENT_AGENT",
        "checks":passed,"status":"PASS_WITH_CONTROLS_RESEARCH_ONLY",
        "formal_bureau_certification":"NOT_CERTIFIED","production":"NOT_AUTHORIZED"}
    a.proof_dir.mkdir(parents=True,exist_ok=True)
    (a.proof_dir/("APP027_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
