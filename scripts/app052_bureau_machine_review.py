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
PARENT="4e01aa139bcfd0b5fff579de1ee8553e3e6720fa"
APP=ROOT/"examples/apps/app052_verified_exclusive_conflict_seal_chain"
GATE=ROOT/"governance/NORDOI_APP052_VERIFIED_EXCLUSIVE_CONFLICT_SEAL_CHAIN_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"chain_app052.py").read_text("utf-8")
    tests=(APP/"test_app052.py").read_text("utf-8")
    demo=(APP/"demo_app052.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app052_verified_exclusive_conflict_seal_chain.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==1250
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==1238
         and g["total_expected_four_os_tests"]==5000 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/","examples/apps/app031_pinned_evidence_transfer/","examples/apps/app020_signed_head_provenance/","examples/apps/app032_signed_evidence_transfer/","examples/apps/app033_local_nonce_consumption/","examples/apps/app034_reception_forensics/","examples/apps/app035_destination_bound_reception/","examples/apps/app036_signed_reception_ack/","examples/apps/app037_sender_ack_registry/","examples/apps/app038_signed_conflict_journal/","examples/apps/app039_conflict_journal_seal/","examples/apps/app040_journal_seal_continuity/","examples/apps/app041_exclusive_seal_successor/","examples/apps/app042_verified_seal_chain/","examples/apps/app043_portable_chain_archive/","examples/apps/app044_signed_chain_archive/","examples/apps/app045_archive_nonce_admission/","examples/apps/app046_portable_admission_evidence/","examples/apps/app047_signed_admission_evidence/","examples/apps/app048_sender_admission_ack_registry/","examples/apps/app049_admission_conflict_journal/","examples/apps/app050_conflict_journal_seal/","examples/apps/app051_exclusive_conflict_seal_successor/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"prior.stored(folder"),(source,"prior.prior.load_seal("),(tests,"test_01_real_two_step_chain")],
      "FERRUM": [(source,"snapshot(root/parent)==before"),(tests,"test_11_restart_readonly"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"MAX_STEPS=64"),(source,"prior.prior.prior.separate(root,live)"),(tests,"test_12_scope_and_overlap")],
      "CIPHER": [(source,"CHAIN_PLAN_PIN_MISMATCH"),(source,"CHAIN_DECISION_PIN_MISMATCH"),(tests,"test_08_external_decision_pin_rejected")],
      "LEDGER": [(source,"CHAIN_SEQUENCE_MISMATCH"),(source,"CHAIN_SEAL_CONTINUITY_MISMATCH"),(tests,"test_09_truncated_chain_not_expected_head")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,'"execution_authorized":False'),(source,'"nonce_reset_authorized":False')],
      "ADVERSARY": [(tests,"test_03_missing_decision"),(tests,"test_05_cycle_rejected"),(tests,"test_07_stored_member_tamper")],
      "VERITY": [(wf,'test "$total" -eq 1250'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"global_antirollback":"NOT_PROVEN"'),(source,'"historical_live_replay":"NOT_ATTESTED"')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","consume","Popen","run","accept","register","preserve","write_bytes","write_text","issue","mkdir","safe_write","admit") for n in calls),"NO_REGISTRY_MUTATION_OR_NATIVE_EXECUTION")
        need("prior.stored(folder" in source and "snapshot(root/parent)==before" in source,"READONLY_REPLAY_AND_RECHECK")
        passed.append("READONLY_CHAIN_REPLAY")
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
    (a.proof_dir/("APP052_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
