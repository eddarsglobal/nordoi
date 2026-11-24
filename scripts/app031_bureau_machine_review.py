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
PARENT="19504a25c614ce063904b17adad6ef1642aa601e"
APP=ROOT/"examples/apps/app031_pinned_evidence_transfer"
GATE=ROOT/"governance/NORDOI_APP031_PINNED_EVIDENCE_TRANSFER_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"transfer_app031.py").read_text("utf-8")
    tests=(APP/"test_app031.py").read_text("utf-8")
    demo=(APP/"demo_app031.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app031_pinned_evidence_transfer.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==998
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==986
         and g["total_expected_four_os_tests"]==3992 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"prior.verify(staged,archive_pin)"),(source,"NONCANONICAL_TRANSFER"),(source,"TRANSFER_ARCHIVE_PIN_MISMATCH")],
      "FERRUM": [(source,"portable.safe_write"),(tests,"test_01_real_receiver_separate_process"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"TRANSFER_PATH_OR_DUPLICATE"),(source,"RECEIVER_DESTINATION_EXISTS"),(tests,"test_05_path_traversal_even_rehashed")],
      "CIPHER": [(source,"TRANSFER_PIN_MISMATCH"),(source,"TRANSFER_ENTRY_DIGEST"),(tests,"test_04_wrong_archive_pin")],
      "LEDGER": [(source,"SOURCE_CHANGED_DURING_TRANSFER"),(source,"RECEIVER_VERIFICATION_CHANGED"),(tests,"test_12_original_preserved_no_transport_claim")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,"RECEIVER_PUBLICATION_UNKNOWN_NO_REPLAY"),(source,'"execution_authorized":False')],
      "ADVERSARY": [(tests,"test_06_duplicate_entry_even_rehashed"),(tests,"test_07_forged_native_bytes_even_rehashed"),(tests,"test_11_noncanonical_and_truncated")],
      "VERITY": [(wf,'test "$total" -eq 998'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"distributed_anti_replay":"NOT_PROVEN"'),(source,'"sender_identity_authenticated":False')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","Popen","run") for n in calls),"EVIDENCE_TRANSFER_ONLY")
        passed.append("NO_DIRECT_DELETE_MOVE_OR_NATIVE_EXECUTION")
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
    (a.proof_dir/("APP031_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
