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
PARENT="8ff0654a157dd054b471e42e30f05a5062854bf7"
APP=ROOT/"examples/apps/app041_exclusive_seal_successor"
GATE=ROOT/"governance/NORDOI_APP041_EXCLUSIVE_SEAL_SUCCESSOR_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"successor_app041.py").read_text("utf-8")
    tests=(APP/"test_app041.py").read_text("utf-8")
    demo=(APP/"demo_app041.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app041_exclusive_seal_successor.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==1118
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==1106
         and g["total_expected_four_os_tests"]==4472 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/","examples/apps/app031_pinned_evidence_transfer/","examples/apps/app020_signed_head_provenance/","examples/apps/app032_signed_evidence_transfer/","examples/apps/app033_local_nonce_consumption/","examples/apps/app034_reception_forensics/","examples/apps/app035_destination_bound_reception/","examples/apps/app036_signed_reception_ack/","examples/apps/app037_sender_ack_registry/","examples/apps/app038_signed_conflict_journal/","examples/apps/app039_conflict_journal_seal/","examples/apps/app040_journal_seal_continuity/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"prior.transition(**args)"),(source,"CANDIDATE_TRANSITION_PIN_MISMATCH"),(source,"SUCCESSOR_PUBLICATION_NOT_VERIFIED")],
      "FERRUM": [(source,"directory.mkdir(mode=0o700)"),(tests,"test_04_two_real_competing_processes"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"SUCCESSOR_REGISTRY_OVERLAP"),(source,"SUCCESSOR_INCOMPLETE_OR_INVALID"),(tests,"test_07_incomplete_marker_permanent")],
      "CIPHER": [(source,"SUCCESSOR_RECORD_MISMATCH"),(source,"AUDIT_EXPECTED_SUCCESSOR_MISMATCH"),(tests,"test_09_tampered_transition_refused")],
      "LEDGER": [(source,"ALTERNATIVE_SUCCESSOR_BLOCKED"),(source,"DUPLICATE_SUCCESSOR_BLOCKED"),(tests,"test_03_distinct_valid_alternative_blocked")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,'"execution_authorized":False'),(source,'"nonce_reset_authorized":False')],
      "ADVERSARY": [(tests,"test_05_invalid_transition_pin_no_marker"),(tests,"test_08_injected_failure_permanent"),(tests,"test_11_overlap_and_invalid_current_journal")],
      "VERITY": [(wf,'test "$total" -eq 1118'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"distributed_fork_prevention":"NOT_PROVEN"'),(tests,"test_12_scope_and_authority")]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","consume","issue","Popen","run","accept","register","preserve") for n in calls),"NO_RESET_OR_NATIVE_EXECUTION")
        need(sum(n.func.attr=="mkdir" for n in calls)==1,"SOLE_EXCLUSIVE_MARKER")
        audit=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=="audit")
        need(not any(isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr in ("safe_write","mkdir") for n in ast.walk(audit)),"AUDIT_NO_DIRECT_MUTATION")
        passed.append("EXCLUSIVE_SUCCESSOR_READONLY_AUDIT")
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
    (a.proof_dir/("APP041_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
