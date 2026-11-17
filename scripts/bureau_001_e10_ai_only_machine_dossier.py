#!/usr/bin/env python3
"""R6F/E10: nine real subprocess machine-function reports, not independent audits.

CLI --audit/--exercise success returns exit 2 BLOCK, source/evidence errors 3.
Output files are only created in a fresh requested disposable directory.
No external testimony, no human tests, no production or native capabilities.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e9_event_ledger_binding as e9
import bureau_001_e3_exact_head_provenance as e3
import roadmap_guardian as guardian

CONTRACT="governance/NORDOI_R6F_BUREAU001_E10_AI_ONLY_MACHINE_DOSSIER.json"
PARENT="c789f02c637d9d8215772a67ca2e85cef35e64af"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E10-NINE-AUTOMATED-FUNCTIONAL-EVIDENCE-V1"
ROLES=tuple(guardian.ROLE_IDS)
SHA40=re.compile(r"^[a-f0-9]{40}$")
class Blocked(ValueError):pass

def need(ok,why):
    if not ok:raise Blocked(why)

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode()

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def source(root,rel):
    path=Path(root)/rel
    need(not path.is_symlink() and path.is_file(),
         "E10_REQUIRED_SOURCE_INVALID:"+rel)
    raw=path.read_bytes()
    need(0<len(raw)<=500000,"E10_SOURCE_SIZE_INVALID:"+rel)
    return raw

def head_at(root):
    p=subprocess.run(["git","rev-parse","HEAD"],cwd=root,
                     capture_output=True,text=True,timeout=30)
    need(p.returncode==0 and SHA40.fullmatch(p.stdout.strip()),
         "E10_GIT_HEAD_UNAVAILABLE")
    return p.stdout.strip()

def deny(reason,**info):
    d=dict(result="BLOCK",reason=reason,
      independent_reviewers=0,independent_bureau_agents=0,
      authenticated_external_witness_receipts=0,
      owner_oob_root="NOT_AUTHENTICATED",
      global_monotonic_antirollback="NOT_VERIFIED",
      human_testers=0,founder_manual_test_duty=False,
      formal_bureau_reviews="NOT_SUBMITTED",
      native_admission_gates_unmet=12,
      r6f="IN_PROGRESS_UNCERTIFIED",
      r6g="LOCKED_BY_SEQUENCE",production="NOT_AUTHORIZED")
    d.update({k:v for k,v in info.items() if k not in d})
    return d

def contract(root,head):
    prev=e9.audit(root,head)
    c=e3.strictly_load(source(root,CONTRACT))
    need(prev["result"]=="BLOCK" and prev["native_admission_gates_unmet"]==12,
         "E10_PREDECESSOR_NOT_DENIED")
    need(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
         and c["parent_e9_exact_head"]==PARENT and
         c["prior_e9"]=={
           "specialized_run_id":44056262617,"specialized_success":4,
           "rust_run_id":44056262413,"rust_success":5},
         "E10_PREDECESSOR_C20_PIN_INVALID")
    need(tuple(c["roles"])==ROLES and
         c["role_machine_verdict"]=="MACHINE_CHECK_PASS_ONLY" and
         c["real_independent_ai_agents_launched"]==0 and
         c["independent_specialist_reviews"]==0 and
         c["external_legal_or_human_testers"]==0 and
         c["human_testers_allowed"] is False and
         c["owner_manual_test_duty"] is False and
         c["founder_role"]==
           "FOUNDER_DIRECTION_AND_ASSISTANCE_NOT_MANUAL_TESTER" and
         c["execution_policy"]==
           "AI_AND_AUTOMATION_ALL_TECHNICAL_TESTS_DOCUMENTATION_FORMALITIES",
         "E10_AI_ONLY_POLICY_OR_NINE_ROLE_FABRICATION")
    need(c["actual_external_witness_receipts"]==0 and
         c["actual_external_guardian"]=="NOT_ENROLLED" and
         c["owner_oob_root"]=="NOT_AUTHENTICATED" and
         c["global_monotonic_antirollback"]=="NOT_VERIFIED" and
         c["external_quorum"]=="NOT_PROVEN" and
         c["formal_bureau_reviews"]=="NOT_SUBMITTED" and
         c["immutable_controls"]=={
           "native_admission_gates_unmet":12,
           "r6f":"IN_PROGRESS_UNCERTIFIED",
           "r6g":"LOCKED_BY_SEQUENCE",
           "production":"NOT_AUTHORIZED",
           "permission_to_merge":False},
         "E10_UNAUTHORIZED_EXTERNAL_AUTHORITY")
    cs=c["contradictions"]
    need([x["id"] for x in cs]==[
           "C01","C02","C03","C04","C05"] and
         len({x["source"] for x in cs})==5,
         "E10_CONTRADICTION_MATRIX_INVALID")
    for row in cs:
        source(root,row["source"])
        need(row["status"]!="RESOLVED_GLOBALLY",
             "E10_UNPROVEN_GLOBAL_REMEDIATION")
    intake=c["external_evidence_intake"]
    need(all(intake[x] is True for x in (
         "must_have_real_distinct_administrative_control",
         "must_have_owner_oob_authenticated_root",
         "must_have_protected_monotonic_history",
         "must_have_real_external_witness_receipts",
         "must_have_true_independent_quorum",
         "self_certified_or_same_admin_artifacts_must_be_rejected",
         "can_be_prepared_and_checked_by_AI")) and
         intake["fabrication_allowed"] is False and
         intake["current_acceptance"]=="NONE",
         "E10_FALSE_EXTERNAL_EVIDENCE_INTAKE")
    need(head_at(root)==head and e3.check_head(root,head)==head,
         "E10_SOURCE_HEAD_MISMATCH")
    return c

TARGETS={
    "ATLAS":("Cargo.toml","src/resource_task_r02.rs"),
    "FERRUM":("tests/g03_r4m_pinned_code_identity_lab.rs",),
    "BOUNDARY":("scripts/bureau_001_e9_event_ledger_binding.py",),
    "CIPHER":("scripts/bureau_001_e6_signed_receipt_rebinding.py",),
    "LEDGER":("scripts/bureau_001_e7_atomic_local_nonce_race.py",
              "scripts/bureau_001_e8_local_ledger_split_brain.py",
              "scripts/bureau_001_e9_event_ledger_binding.py"),
    "CAPSTONE":("scripts/roadmap_guardian.py",),
    "ADVERSARY":("scripts/bureau_001_e8_local_ledger_split_brain.py",
                 "scripts/bureau_001_e9_event_ledger_binding.py"),
    "VERITY":("RoadMap.md","Bureau.md"),
    "GUARDIAN":("governance/NORDOI_BUREAU_R6F_HANDOFF_v1.json",
                 "scripts/roadmap_guardian.py"),
}

def role_check(root,head,role):
    need(role in ROLES,"E10_INVALID_BUREAU_FUNCTION")
    c=contract(root,head)
    hashes={p:digest(source(root,p)) for p in TARGETS[role]}
    if role=="ATLAS":
        need(b"[package]" in source(root,"Cargo.toml") or
             b"[workspace]" in source(root,"Cargo.toml"),
             "E10_RUST_MANIFEST_MISSING")
    elif role=="FERRUM":
        need(b"CDHash" in source(root,TARGETS[role][0]),
             "E10_NATIVE_CODE_IDENTITY_TEST_MISSING")
    elif role=="BOUNDARY":
        need(b"E9_EVENT_PERSISTED_ROW_MISMATCH" in
             source(root,TARGETS[role][0]),
             "E10_EVENT_ROW_BOUNDARY_CHECK_MISSING")
    elif role=="CIPHER":
        signed=e9.e8.e7.e6.make(head,"a"*64,"b"*64,"c"*64,"d"*64)
        parsed=e9.e8.e7.e6.parse(signed,head)
        need(parsed["source_head"]==head and
             parsed["challenge_nonce_sha256"]=="c"*64,
             "E10_E6_CANONICAL_PACKET_REJECTED")
    elif role=="LEDGER":
        need([x["id"] for x in c["contradictions"]]==[
           "C01","C02","C03","C04","C05"],
             "E10_LEDGER_CONTRADICTION_IDS_WRONG")
    elif role=="CAPSTONE":
        d=e9.outcome("TEST",r6g="UNLOCK",
                     production="AUTHORIZED",native_admission_gates_unmet=0)
        need(d["result"]=="BLOCK" and
             d["production"]=="NOT_AUTHORIZED" and
             d["native_admission_gates_unmet"]==12,
             "E10_CAPABILITY_DENY_BYPASSED")
    elif role=="ADVERSARY":
        a=e9.e8.signed_event("A","a"*64,"b"*64,head)
        b=e9.e8.signed_event("B","c"*64,"b"*64,head)
        d=e9.e8.reconcile([a,b])
        need(d["result"]=="BLOCK" and d["split_brain_detected"] is False,
             "E10_NEGATIVE_INDICATOR_NOT_REPRODUCED")
    elif role=="VERITY":
        p=subprocess.run(["git","status","--porcelain"],
                         cwd=root,capture_output=True,text=True,timeout=30)
        need(p.returncode==0 and not p.stdout.strip(),
             "E10_DIRTY_SOURCE_WORKTREE")
    elif role=="GUARDIAN":
        d=guardian.audit(root)
        need(d["BUREAU_GUARDIAN"]=="PASS_WITH_CONTROLS" and
             d["REVIEWS_SUBMITTED"]==0 and
             d["NATIVE_ADMISSION_GATES"]=="12_UNMET" and
             d["R6G"]=="LOCKED_BY_SEQUENCE",
             "E10_GUARDIAN_FALSE_PROMOTION")
    return {
      "schema":"NORDOI-R6F-E10-MACHINE-CHECK-V1",
      "role":role,"source_head":head,
      "source_sha256":hashes,
      "pid":os.getpid(),
      "verdict":"MACHINE_CHECK_PASS",
      "scope":"REAL_LOCAL_MACHINE_FUNCTION_NOT_INDEPENDENT_SPECIALIST_REVIEW",
      "formal_independent_review":"NOT_SUBMITTED",
      "independent_witness":False,
      "native_admission":False,
    }

def audit(root,head):
    c=contract(root,head)
    need(len(ROLES)==9 and len(TARGETS)==9,
         "E10_NINE_MACHINE_FUNCTIONS_NOT_PINNED")
    return deny("E10_NINE_MACHINE_FUNCTIONS_CONTRACT_ONLY",
        source_head=head,automated_roles_configured=len(ROLES),
        actual_machine_checks_executed=0,
        formal_independent_reviews=0,
        contradictions_recorded=len(c["contradictions"]))

def sealed_report(raw):
    obj=json.loads(raw)
    need(type(obj) is dict and set(obj)=={
        "schema","role","source_head","source_sha256","pid",
        "verdict","scope","formal_independent_review",
        "independent_witness","native_admission"},
        "E10_ROLE_REPORT_SCHEMA_MISMATCH")
    need(obj["role"] in ROLES and obj["verdict"]=="MACHINE_CHECK_PASS"
         and obj["formal_independent_review"]=="NOT_SUBMITTED"
         and obj["independent_witness"] is False
         and obj["native_admission"] is False,
         "E10_ROLE_REPORT_PROMOTION_OR_FABRICATION")
    need(canonical(obj)==raw,"E10_NONCANONICAL_ROLE_REPORT")
    return obj

def exercise(root,head,folder):
    c=contract(root,head)
    output=Path(folder).absolute()
    need(not output.exists() and output.parent.is_dir()
         and not output.parent.is_symlink(),
         "E10_OUTPUT_DIRECTORY_MUST_BE_FRESH")
    output.mkdir(mode=0o700)
    checks=[]
    children=[]
    try:
        # Launch all nine actual role processes before collecting any output.
        # Their overlapping lifetimes make OS PID reuse impossible.
        for role in ROLES:
            p=subprocess.Popen([sys.executable,"-B",
                str(Path(__file__).resolve()),"--role",role,
                "--expected-head",head,"--root",str(root)],
                cwd=root,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            children.append((role,p))
        for role,p in children:
            raw,err=p.communicate(timeout=120)
            need(p.returncode==2,
                 "E10_ACTUAL_ROLE_PROCESS_FAILED:"+role+":"+err[:160].decode(errors="replace"))
            d=sealed_report(raw)
            need(d["role"]==role and d["source_head"]==head
                 and d["pid"]==p.pid,"E10_REPORT_SOURCE_ROLE_PID_MISMATCH")
            need(d["source_sha256"]==
                 {path:digest(source(root,path)) for path in TARGETS[role]},
                 "E10_SOURCE_SHA_NOT_ACTUALLY_CHECKED")
            fn=role+".json"
            (output/fn).write_bytes(raw)
            checks.append({"role":role,"path":fn,
                           "report_sha256":digest(raw),"pid":d["pid"]})
    finally:
        for _,p in children:
            if p.poll() is None:
                p.kill()
                p.communicate()
    need(len({x["pid"] for x in checks})==9,
         "E10_NINE_DISTINCT_OS_PROCESSES_NOT_EXECUTED")
    manifest={
      "schema":"NORDOI-R6F-E10-NINE-REAL-MACHINE-REPORTS-V1",
      "exact_source_head":head,
      "machine_function_count":9,
      "actual_distinct_child_processes":9,
      "reports":checks,
      "contradiction_matrix":c["contradictions"],
      "external_evidence_intake":c["external_evidence_intake"],
      "formal_independent_reviews":0,
      "external_witness_receipts":0,
      "native_admission_gates_unmet":12,
      "r6f":"IN_PROGRESS_UNCERTIFIED",
      "r6g":"LOCKED_BY_SEQUENCE",
      "production":"NOT_AUTHORIZED",
      "human_testers":0,
    }
    (output/"MANIFEST.json").write_bytes(canonical(manifest))
    validate_dossier(output,head)
    return deny("E10_REAL_NINE_MACHINE_CHECKS_WITH_TRACEABLE_REPORTS_NO_CERTIFICATION",
        exact_source_head=head,machine_function_count=9,
        actual_distinct_child_processes=9,
        generated_report_files=9,
        contradiction_matrix_rows=len(c["contradictions"]),
        output_manifest_sha256=digest(canonical(manifest)),
        actual_external_authority_proven=False)

def validate_dossier(folder,head):
    output=Path(folder)
    need(output.is_dir() and not output.is_symlink(),
         "E10_OUTPUT_NOT_REGULAR_DIRECTORY")
    m=e3.strictly_load(source(output,"MANIFEST.json"))
    need(m["schema"]=="NORDOI-R6F-E10-NINE-REAL-MACHINE-REPORTS-V1"
         and m["exact_source_head"]==head
         and m["machine_function_count"]==9
         and m["actual_distinct_child_processes"]==9
         and m["formal_independent_reviews"]==0
         and m["external_witness_receipts"]==0
         and m["native_admission_gates_unmet"]==12
         and m["r6g"]=="LOCKED_BY_SEQUENCE"
         and m["production"]=="NOT_AUTHORIZED"
         and m["human_testers"]==0,
         "E10_FALSE_MANIFEST_AUTHORITY")
    need([x["role"] for x in m["reports"]]==list(ROLES)
         and {x.name for x in output.iterdir()}==
          {"MANIFEST.json"}|{r+".json" for r in ROLES},
         "E10_INCOMPLETE_NINE_FUNCTION_DOSSIER")
    need(len({x["pid"] for x in m["reports"]})==9,
         "E10_FAKE_DISTINCT_PROCESSES")
    for entry in m["reports"]:
        need(entry["path"]==entry["role"]+".json",
             "E10_ROLE_PATH_MISMATCH")
        raw=source(output,entry["path"])
        d=sealed_report(raw)
        need(d["role"]==entry["role"] and
             d["source_head"]==head and
             d["pid"]==entry["pid"] and
             entry["report_sha256"]==digest(raw),
             "E10_CHANGED_OR_MISBOUND_ROLE_EVIDENCE")
    return m

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    modes=ap.add_mutually_exclusive_group(required=True)
    modes.add_argument("--audit",action="store_true")
    modes.add_argument("--exercise",action="store_true")
    modes.add_argument("--role",choices=ROLES)
    ap.add_argument("--root",default=str(ROOT))
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--output-dir")
    args=ap.parse_args(argv)
    try:
        root=Path(args.root).resolve(strict=True)
        if args.audit: d=audit(root,args.expected_head)
        elif args.role:
            d=role_check(root,args.expected_head,args.role)
        else:
            need(args.output_dir is not None,
                 "E10_EXERCISE_NEEDS_FRESH_OUTPUT_DIR")
            d=exercise(root,args.expected_head,args.output_dir)
        code=2
    except (Blocked,e9.Blocked,e9.e8.Blocked,e9.e8.e7.Blocked,
            e9.e8.e7.e6.Blocked,e3.Rejected,guardian.Blocked,
            ValueError,KeyError,TypeError,OSError,subprocess.SubprocessError) as ex:
        d=deny("E10_SOURCE_OR_NINE_PROCESS_EXPERIMENT_NOT_QUALIFIED",
               error_type=type(ex).__name__)
        code=3
    # Byte-exact canonical JSON: avoid Windows text stdout CRLF conversion.
    sys.stdout.buffer.write(canonical(d))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":raise SystemExit(main())
