#!/usr/bin/env python3
"""R6F/E11 actual coherent rewrite of an E10 machine dossier and local rebind.

No human tests, no network, no production, no externally independent authority.
A valid local SHA manifest alone is NOT a witness. CLI audit/exercise success
always returns exit 2 BLOCK; errors return 3.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e10_ai_only_machine_dossier as e10
import bureau_001_e3_exact_head_provenance as e3

CONTRACT="governance/NORDOI_R6F_BUREAU001_E11_REBOUND_MACHINE_EVIDENCE.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E11-REBOUND-MACHINE-REPORTS-V1"
PARENT="ae84f34795c680a6929453f0d4723bec47189a4e"
ATTACK_STEPS=(
    "create_genuine_e10_nine_process_report_bundle",
    "verify_untampered_e10_manifest",
    "forge_one_local_role_source_sha256",
    "recalculate_local_report_digest_in_manifest",
    "reproduce_old_local_validator_false_acceptance",
    "new_guard_compares_every_role_source_sha_to_actual_repo_bytes",
    "restore_original_bytes_and_revalidate",
)
class Blocked(ValueError):pass

def require(condition,why):
    if not condition:raise Blocked(why)

def deny(reason,**details):
    d=e10.deny(reason)
    d.update({k:v for k,v in details.items() if k not in d})
    return d

def audit(root,head):
    before=e10.audit(root,head)
    c=e3.strictly_load(e10.source(root,CONTRACT))
    require(before["result"]=="BLOCK" and
            before["native_admission_gates_unmet"]==12 and
            c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi" and
            c["parent_e10_exact_head"]==PARENT and
            c["prior_e10"]=={
                "specialized_run_id":44067965884,"specialized_success":4,
                "rust_run_id":44067966070,"rust_success":5},
            "E11_E10_PARENT_OR_C20_PIN_INVALID")
    require(tuple(c["roles"])==e10.ROLES and
            tuple(c["attack_steps"])==ATTACK_STEPS and
            c["execution"]=="AI_AND_AUTOMATED_MACHINES_ONLY" and
            c["human_testers_allowed"] is False and
            c["founder_manual_test_duty"] is False and
            c["independent_bureau_agents_launched"]==0 and
            c["formal_independent_specialist_reviews"]==0 and
            c["same_admin_report_signature_is_independent"] is False and
            c["external_witness_receipts"]==0 and
            c["external_custodian"]=="NOT_ENROLLED" and
            c["owner_oob_root"]=="NOT_AUTHENTICATED" and
            c["global_monotonic_antirollback"]=="NOT_VERIFIED" and
            c["unknown_external_branches_proven_complete"] is False and
            c["native_admission_gates_unmet"]==12 and
            c["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
            c["r6g"]=="LOCKED_BY_SEQUENCE" and
            c["production"]=="NOT_AUTHORIZED" and
            c["permission_to_merge"] is False and
            c["pre_ci_verdict"]=="IN_PROGRESS_UNQUALIFIED" and
            c["evidence_boundary"]==
            "RECOMPUTING_LOCAL_SOURCE_HASHES_AGAINST_EXACT_GIT_HEAD_DOES_NOT_PROVE_INDEPENDENT_EXTERNAL_CUSTODY",
            "E11_AI_ONLY_OR_EXTERNAL_AUTHORITY_CONTRACT_INVALID")
    require(e10.head_at(root)==head,
            "E11_EXPECTED_GIT_HEAD_INCORRECT")
    return deny("E11_REAL_COHERENT_DOSSIER_TAMPER_CONTRACT_ONLY",
                source_head=head,e11_lab_executed=False)

def rebind_actual_repo_sources(root,folder,head):
    """Narrow LOCAL correction: compare report hashes against actual Git source."""
    audit(root,head)
    manifest=e10.validate_dossier(folder,head)
    require(e10.canonical(manifest)==e10.source(folder,"MANIFEST.json"),
            "E11_MANIFEST_NOT_CANONICAL")
    contract=e10.contract(root,head)
    require(manifest["contradiction_matrix"]==contract["contradictions"] and
            manifest["external_evidence_intake"]==
            contract["external_evidence_intake"],
            "E11_COHERENTLY_REWRITTEN_CONTRADICTION_OR_INTAKE")
    for entry in manifest["reports"]:
        role=entry["role"]
        report=e10.sealed_report(e10.source(folder,entry["path"]))
        require(report["source_head"]==head and
                report["role"]==role and
                report["source_sha256"]==
                {p:e10.digest(e10.source(root,p))
                 for p in e10.TARGETS[role]},
                "E11_REPORT_SOURCE_HASH_NOT_BOUND_TO_REAL_GIT_BYTES:"+role)
    return deny("E11_NINE_LOCAL_REPORTS_REBOUND_TO_REAL_SOURCE_ONLY",
                exact_source_head=head,actual_source_rebound_roles=9,
                external_independent_authentication=False)

def rewrite_source_and_rehash_manifest(folder,role="ATLAS"):
    """Actual on-disk coherent attack; only in generated disposable lab."""
    require(role in e10.ROLES,"E11_UNKNOWN_ROLE")
    folder=Path(folder)
    raw=e10.source(folder,role+".json")
    d=e10.sealed_report(raw)
    path=next(iter(e10.TARGETS[role]))
    original=d["source_sha256"][path]
    fake="f"*64 if original!="f"*64 else "0"*64
    d["source_sha256"][path]=fake
    changed=e10.canonical(d)
    require(changed!=raw,"E11_NO_ACTUAL_REPORT_REWRITE")
    (folder/(role+".json")).write_bytes(changed)
    m=e3.strictly_load(e10.source(folder,"MANIFEST.json"))
    matches=[r for r in m["reports"] if r["role"]==role]
    require(len(matches)==1,"E11_MANIFEST_ROLE_NOT_UNIQUE")
    matches[0]["report_sha256"]=e10.digest(changed)
    (folder/"MANIFEST.json").write_bytes(e10.canonical(m))
    return {"role":role,"source_path":path,"old_sha":original,
            "forged_sha":fake,"report_sha":e10.digest(changed),
            "manifest_sha":e10.digest(e10.canonical(m))}

def exercise(root,head):
    audit(root,head)
    with tempfile.TemporaryDirectory(prefix="nordoi-e11-real-rehash-") as tmp:
        folder=Path(tmp)/"e10_bundle"
        baseline=e10.exercise(root,head,folder)
        require(baseline["actual_distinct_child_processes"]==9 and
                baseline["generated_report_files"]==9,
                "E11_GENUINE_E10_NINE_PROCESS_BUNDLE_NOT_CREATED")
        good=rebind_actual_repo_sources(root,folder,head)
        require(good["actual_source_rebound_roles"]==9,
                "E11_CLEAN_NINE_REAL_ROLE_REBIND_NOT_VERIFIED")
        original_report=e10.source(folder,"ATLAS.json")
        original_manifest=e10.source(folder,"MANIFEST.json")
        attack=rewrite_source_and_rehash_manifest(folder)
        old=e10.validate_dossier(folder,head)
        require(old["machine_function_count"]==9 and
                old["formal_independent_reviews"]==0,
                "E11_OLD_E10_LOCAL_VALIDATOR_FALSE_ACCEPTANCE_NOT_REPRODUCED")
        caught=False
        try:rebind_actual_repo_sources(root,folder,head)
        except Blocked as err:
            caught=str(err).startswith(
                "E11_REPORT_SOURCE_HASH_NOT_BOUND_TO_REAL_GIT_BYTES:")
        require(caught,"E11_NEW_GUARD_DID_NOT_CATCH_REHASHED_REPORT")
        (folder/"ATLAS.json").write_bytes(original_report)
        (folder/"MANIFEST.json").write_bytes(original_manifest)
        restored=rebind_actual_repo_sources(root,folder,head)
        require(restored["actual_source_rebound_roles"]==9,
                "E11_ORIGINAL_REPORTS_NO_LONGER_VALIDATE")
        require(e10.head_at(root)==head,
                "E11_RESEARCH_MUTATED_SOURCE_HEAD")
    return deny("E11_REAL_NINE_REPORT_COHERENT_REWRITE_DETECTED_LOCAL_ONLY",
                exact_source_head=head,actual_e10_role_processes=9,
                actual_e10_report_files=9,
                old_e10_validator_accepted_coherently_rewritten_report=True,
                attack_recomputed_report_and_manifest_hashes=True,
                new_source_rebound_guard_rejected_coherent_rewrite=True,
                restored_original_source_reports_passed=True,
                real_source_file_changed=False,
                external_custody_proven=False,
                human_testing_performed=False,
                attack_role=attack["role"])

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    mode=ap.add_mutually_exclusive_group(required=True)
    mode.add_argument("--audit",action="store_true")
    mode.add_argument("--exercise",action="store_true")
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--root",default=str(ROOT))
    a=ap.parse_args(argv)
    try:
        d=audit(Path(a.root),a.expected_head) if a.audit else exercise(
            Path(a.root),a.expected_head)
        code=2
    except (Blocked,e10.Blocked,e10.e9.Blocked,e3.Rejected,
            KeyError,TypeError,ValueError,IndexError,OSError,
            subprocess.SubprocessError) as err:
        d=deny("E11_REAL_COHERENT_TAMPER_EXPERIMENT_NOT_QUALIFIED",
               error_type=type(err).__name__)
        code=3
    sys.stdout.buffer.write(e10.canonical(d))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":raise SystemExit(main())
