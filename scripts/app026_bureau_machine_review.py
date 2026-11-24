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
PARENT="f56ca58bcd164075ec4a088696b5a278862289f9"
APP=ROOT/"examples/apps/app026_verified_checkpoint_reconstruction"
GATE=ROOT/"governance/NORDOI_APP026_CHECKPOINT_RECONSTRUCTION_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"reconstruct_app026.py").read_text("utf-8")
    tests=(APP/"test_app026.py").read_text("utf-8")
    demo=(APP/"demo_app026.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app026_checkpoint_reconstruction.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==938
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==926
         and g["total_expected_four_os_tests"]==3752 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT"),(source,"portable.inner.seal"),(source,"portable.pack")],
      "FERRUM": [(source,"native_processes_started"),(tests,"NativeInterruptedForensicTests"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"DESTINATION_OVERLAPS_ORIGINAL_EVIDENCE"),(source,"DESTINATION_ALREADY_EXISTS"),(tests,"test_10_no_output_inside_evidence")],
      "CIPHER": [(source,"forensic.sha256(raw) == suffix_pin"),(source,"portable.verify(target, pin)"),(source,"external_pin_authentication")],
      "LEDGER": [(source,"events[:len(old_events)] == old_events"),(source,"ORIGINAL_NATIVE_BYTES_CHANGED"),(tests,"test_11_original_bytes_preserved")],
      "CAPSTONE": [(source,"PUBLICATION_OUTCOME_UNKNOWN_NO_REPLAY"),(source,"STOP_NO_AUTOMATIC_NATIVE_REPLAY"),(source,"INDEPENDENTLY_PIN_AND_VERIFY_NEW_CHECKPOINT_BEFORE_RESUME")],
      "ADVERSARY": [(tests,"test_05_orphan_effect_rejected"),(tests,"test_08_truncated_suffix_rejected"),(tests,"test_09_existing_destination_not_overwritten")],
      "VERITY": [(wf,'test "$total" -eq 938'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production": "NOT_AUTHORIZED"'),(source,'"formal_bureau_certification": "NOT_CERTIFIED"'),(source,'"global_antirollback": "NOT_PROVEN"')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        imports=[n.module for n in ast.walk(tree) if isinstance(n,ast.ImportFrom)]
        names=[x.name for n in ast.walk(tree) if isinstance(n,ast.Import) for x in n.names]
        need("subprocess" not in imports+names,"UNEXPECTED_PROCESS_IMPORT")
        passed.append("NO_SUBPROCESS_IMPORT")
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
    (a.proof_dir/("APP026_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
