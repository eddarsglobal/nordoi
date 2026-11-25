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
PARENT="2210c7029ccc5c8d23a4e8161c78434b15ee94ae"
APP=ROOT/"examples/apps/app033_local_nonce_consumption"
GATE=ROOT/"governance/NORDOI_APP033_LOCAL_NONCE_CONSUMPTION_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"nonce_app033.py").read_text("utf-8")
    tests=(APP/"test_app033.py").read_text("utf-8")
    demo=(APP/"demo_app033.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app033_local_nonce_consumption.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==1022
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==1010
         and g["total_expected_four_os_tests"]==4088 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/","examples/apps/app031_pinned_evidence_transfer/","examples/apps/app020_signed_head_provenance/","examples/apps/app032_signed_evidence_transfer/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"signed.receive(packet"),(source,"CHALLENGE_CONTEXT_MISMATCH"),(source,"transfer.prior.verify(target,archive_pin)")],
      "FERRUM": [(source,"claimed.mkdir(mode=0o700)"),(tests,"test_04_actual_two_process_race"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"RECEIVER_REGISTRY_OVERLAP"),(source,"CHALLENGE_NOT_ISSUED"),(tests,"test_10_registry_overlap_refused")],
      "CIPHER": [(source,"secrets.token_hex(32)"),(source,"signed.statement"),(tests,"test_05_invalid_signature_does_not_consume")],
      "LEDGER": [(source,"NONCE_ALREADY_CONSUMED_OR_UNCERTAIN"),(source,'claimed/"COMPLETE.json"'),(tests,"test_03_distinct_receiver_process_restart")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,'"execution_authorized":False'),(source,'"automatic_retry":False')],
      "ADVERSARY": [(tests,"test_08_empty_consumption_marker_blocks"),(tests,"test_09_publication_failure_retains_marker"),(tests,"test_02_replay_new_destination")],
      "VERITY": [(wf,'test "$total" -eq 1022'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"distributed_anti_replay":"NOT_PROVEN"'),(source,'"global_antirollback":"NOT_PROVEN"')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","Popen","run") for n in calls),"NO_REPLAY_RESET_OR_NATIVE_EXECUTION")
        passed.append("EXCLUSIVE_MARKER_NO_RESET_ENTRYPOINT")
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
    (a.proof_dir/("APP033_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
