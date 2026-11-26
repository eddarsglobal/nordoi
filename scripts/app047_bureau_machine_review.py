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
PARENT="2ff01d93c456a58cd5b33ed3e40ad2b5ac4eecd3"
APP=ROOT/"examples/apps/app047_signed_admission_evidence"
GATE=ROOT/"governance/NORDOI_APP047_SIGNED_ADMISSION_EVIDENCE_V1.json"

def need(v,code):
    if not v: raise ValueError(code)

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--role",required=True,choices=guardian.ROLE_IDS)
    p.add_argument("--proof-dir",required=True,type=Path)
    a=p.parse_args()
    source=(APP/"ack_app047.py").read_text("utf-8")
    tests=(APP/"test_app047.py").read_text("utf-8")
    demo=(APP/"demo_app047.py").read_text("utf-8")
    wf=(ROOT/".github/workflows/nordoi_app047_signed_admission_evidence.yml").read_text("utf-8")
    g=json.loads(GATE.read_text("utf-8"))
    historical=guardian.audit(ROOT)
    current=research.audit(ROOT)
    need(historical["REVIEWS_SUBMITTED"]==0 and historical["BUREAU_CERTIFICATION"]=="PENDING"
         and historical["R6G"]=="LOCKED_BY_SEQUENCE"
         and current["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED","GUARDIAN_CONTRADICTION")
    need(g["parent_exact_sha"]==PARENT and g["total_tests_per_os"]==1190
         and g["new_tests_per_os"]==12 and g["prior_app_regressions_per_os"]==1178
         and g["total_expected_four_os_tests"]==4760 and g["demo_assertions_per_os"]==8
         and g["native_admission_gates_unmet"]==12 and g["production"]=="NOT_AUTHORIZED"
         and g["formal_bureau_certification"]=="NOT_CERTIFIED","GATE_CONTRADICTION")
    subprocess.run(["git","diff","--exit-code",PARENT,"--","src/","Cargo.toml","Cargo.lock",
        "RoadMap.md","Bureau.md","scripts/roadmap_guardian.py","scripts/app_research_guardian.py",
        "governance/NORDOI_BUREAU_PLAN_v1.json","examples/apps/app025_interrupted_dag_forensics/",
        "examples/apps/app014_portable_checkpoint/","examples/apps/app013_verified_handoff/","examples/apps/app026_verified_checkpoint_reconstruction/","examples/apps/app024_verified_concurrent_resume/","examples/apps/app027_verified_reconstructed_resume/","examples/apps/app028_local_exclusive_resume/","examples/apps/app029_reservation_forensics/","examples/apps/app030_verified_reservation_archive/","examples/apps/app031_pinned_evidence_transfer/","examples/apps/app020_signed_head_provenance/","examples/apps/app032_signed_evidence_transfer/","examples/apps/app033_local_nonce_consumption/","examples/apps/app034_reception_forensics/","examples/apps/app035_destination_bound_reception/","examples/apps/app036_signed_reception_ack/","examples/apps/app037_sender_ack_registry/","examples/apps/app038_signed_conflict_journal/","examples/apps/app039_conflict_journal_seal/","examples/apps/app040_journal_seal_continuity/","examples/apps/app041_exclusive_seal_successor/","examples/apps/app042_verified_seal_chain/","examples/apps/app043_portable_chain_archive/","examples/apps/app044_signed_chain_archive/","examples/apps/app045_archive_nonce_admission/","examples/apps/app046_portable_admission_evidence/"],
        cwd=ROOT,check=True)
    checks={
      "ATLAS": [(source,"prior.prior.audit(**args)"),(source,"LOCAL_ACCEPTANCE_NOT_MATCHED"),(source,"ACK_EVIDENCE_MISMATCH")],
      "FERRUM": [(source,"tempfile.TemporaryDirectory"),(tests,"test_11_readonly_restart"),(wf,"cargo build --offline --bin nordoi")],
      "BOUNDARY": [(source,"ACK_OUTPUT_INVALID"),(source,"RECIPIENT_KEY_MUST_DIFFER"),(tests,"test_06_identity_and_context_mismatch")],
      "CIPHER": [(source,"crypto.check_pubkey"),(source,"RECIPIENT_SIGNATURE_VERIFICATION_FAILED"),(tests,"test_04_wrong_namespace")],
      "LEDGER": [(source,"SOURCE_ACCEPTANCE_CHANGED"),(source,"ACK_EXPECTATION_MISMATCH"),(tests,"test_02_original_sources_relocated")],
      "CAPSTONE": [(source,'"reservation_release_authorized":False'),(source,'"execution_authorized":False'),(source,'"nonce_reset_authorized":False')],
      "ADVERSARY": [(tests,"test_03_sender_key_cannot_ack"),(tests,"test_07_tampered_ack_refused"),(tests,"test_09_corrupted_live_receipt_no_preparation")],
      "VERITY": [(wf,'test "$total" -eq 1190'),(wf,"macos-15-intel, macos-15, ubuntu-latest, windows-latest"),(demo,"assert len(claims)==8")],
      "GUARDIAN": [(source,'"production":"NOT_AUTHORIZED"'),(source,'"independent_historical_acceptance":"NOT_PROVEN"'),(source,'"live_nonce_state":"NOT_VERIFIED"')]
    }
    passed=[]
    for text,clause in checks[a.role]:
        need(clause in text,"CLAUSE_MISSING:"+clause)
        passed.append(clause)
    if a.role=="FERRUM":
        tree=ast.parse(source)
        calls=[n for n in ast.walk(tree) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute)]
        need(not any(n.func.attr in ("unlink","rmdir","rmtree","remove","rename","replace","execute","reserve","consume","Popen","accept","register","preserve","write_bytes","write_text","issue") for n in calls),"NO_LIVE_STATE_MUTATION")
        runs=[n for n in calls if n.func.attr=="run"]
        need(len(runs)==1 and isinstance(runs[0].args[0],ast.List) and any(isinstance(x,ast.Constant) and x.value=="verify" for x in runs[0].args[0].elts),"ONLY_SSHSIG_VERIFY_SUBPROCESS")
        passed.append("DETACHED_RECIPIENT_SIGNATURE_VERIFICATION")
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
    (a.proof_dir/("APP047_MACHINE_"+a.role+".json")).write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))

if __name__=="__main__":
    main()
