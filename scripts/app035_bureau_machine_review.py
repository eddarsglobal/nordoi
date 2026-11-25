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
PARENT="c85657b5cdf91d7d6d078ab40b2a31c3c4d2d95a"
APP=ROOT/"examples/apps/app035_destination_bound_reception"
GATE=ROOT/"governance/NORDOI_APP035_DESTINATION_BOUND_RECEPTION_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"bound_app035.py").read_text("utf-8")
    tests=(APP/"test_app035.py").read_text("utf-8")
    demo=(APP/"demo_app035.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app035_destination_bound_reception.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==1046
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==1034
         and g["total_expected_four_os_tests"]==4184 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/","examples/apps/app031_pinned_evidence_transfer/","examples/apps/app020_signed_head_provenance/","examples/apps/app032_signed_evidence_transfer/","examples/apps/app033_local_nonce_consumption/","examples/apps/app034_reception_forensics/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"signed.receive(packet"),(source,"DESTINATION_PARENT_CHANGED"),(source,"PUBLISHED_EVIDENCE_MISMATCH")],
      "FERRUM": [(source,"st.st_ino"),(tests,"test_09_actual_two_process_race"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"RECEIVER_REGISTRY_OVERLAP"),(source,"RECEIVER_DESTINATION_EXISTS"),(tests,"test_03_unexpected_destination_refused")],
      "CIPHER": [(source,"DESTINATION_IDENTITY_MISMATCH"),(source,"COMPLETION_RECEIPT_MISMATCH"),(tests,"test_11_invalid_signature_does_not_consume")],
      "LEDGER": [(source,"previous.consume(directory,expected)"),(source,'claimed/"COMPLETE.json"'),(tests,"test_10_failure_after_consumption_is_permanent")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,'"execution_authorized":False'),(source,'"nonce_reset_authorized":False')],
      "ADVERSARY": [(tests,"test_04_parent_substitution_refused"),(tests,"test_05_same_bytes_replacement_refused"),(tests,"test_08_identity_record_tampering_refused")],
      "VERITY": [(wf,'test "$total" -eq 1046'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"distributed_anti_replay":"NOT_PROVEN"'),(source,'"destination_signed_by_sender":False')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","Popen","run") for n in calls),"NO_RESET_OR_NATIVE_EXECUTION")
        audit=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=="audit")
        need(not any(isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr in ("consume","safe_write","mkdir","populate") for n in ast.walk(audit)),"AUDIT_NO_DIRECT_MUTATION")
        passed.append("DESTINATION_BINDING_NO_RESET_READONLY_AUDIT")
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
    (a.proof_dir/("APP035_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
