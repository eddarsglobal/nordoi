#!/usr/bin/env python3
"""R6F E12: AI-only, fail-closed intake for twelve real external native gates.

This is an automated research verifier, not an external authority, enrollment
procedure or attestation service. Real local SSHSIG is tested on POSIX. Success
returns BLOCK and exit 2; errors return exit 3. No human testers.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e11_rebound_machine_dossier as e11
import bureau_001_e6_signed_receipt_rebinding as e6
import bureau_001_e3_exact_head_provenance as e3
import roadmap_guardian as guardian

CONTRACT="governance/NORDOI_R6F_BUREAU001_E12_TWELVE_GATE_INTAKE.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E12-TWELVE-REAL-EVIDENCE-INTAKE-V1"
CLAIM_SCHEMA="NORDOI-R6F-E12-UNTRUSTED-OPERATOR-CLAIMS-V1"
PARENT="96e56f954721eb96f3526e529a27b2456bd2e8ce"
GATES=tuple(guardian.GATE_IDS)
KINDS=(
    "independently_authentic_owner_root",
    "independently_attested_custodian_identity",
    "separate_admin_root_of_control",
    "outside_channel_pinned_guardian_key",
    "independent_os_attestation_signature",
    "bound_fresh_challenge_signature_and_history",
    "independently_attested_separate_host",
    "externally_enforced_epoch_register",
    "externally_rollback_protected_revocation_floor",
    "externally_rollback_protected_nonce_floor",
    "separately_authorized_distinct_control_quorum",
    "independent_recovery_and_rotation_proof",
)
ROLES=tuple(guardian.ROLE_IDS)
UNMET="UNMET_INDEPENDENT_EXTERNAL_EVIDENCE"
class Blocked(ValueError):pass

def require(ok,reason):
    if not ok:raise Blocked(reason)

def deny(reason,**details):
    d=dict(result="BLOCK",reason=reason,
           native_admission_gates_unmet=12,
           native_gate_status={x:UNMET for x in GATES},
           authenticated_external_witness_receipts=0,
           enrolled_independent_custodians=0,
           separately_authenticated_owner_roots=0,
           actual_independent_bureau_reviews=0,
           human_testers=0,founder_manual_test_duty=False,
           r6f="IN_PROGRESS_UNCERTIFIED",
           r6g="LOCKED_BY_SEQUENCE",
           production="NOT_AUTHORIZED")
    d.update({k:v for k,v in details.items() if k not in d})
    return d

def audit(root,head):
    prev=e11.audit(root,head)
    require(prev["result"]=="BLOCK" and
            prev["native_admission_gates_unmet"]==12,
            "E12_E11_SOURCE_C20_NOT_PROTECTED")
    c=e3.strictly_load(e11.e10.source(root,CONTRACT))
    require(c["schema"]==SCHEMA and c["project"]=="eddarsglobal/nordoi"
            and c["parent_e11_exact_head"]==PARENT
            and c["prior_e11"]=={
                "specialized_run_id":44077394937,"specialized_success":4,
                "rust_run_id":44077394930,"rust_success":5},
            "E12_E11_GREEN_PARENT_NOT_PINNED")
    require(c["execution"]=="AI_AND_AUTOMATION_ONLY"
            and c["founder_role"]=="DIRECTOR_NO_MANUAL_TEST_DUTY"
            and c["human_testers_allowed"] is False
            and tuple(c["machine_bureau_roles"])==ROLES
            and c["real_independent_bureau_audits"]==0
            and c["external_trust_roots_enrolled"]==0
            and c["external_witness_receipts_authenticated"]==0,
            "E12_AI_ONLY_AND_REAL_TRUST_LIMITS_INVALID")
    rows=c["required_native_gates"]
    require(type(rows) is list and len(rows)==12
            and tuple(r["id"] for r in rows)==GATES
            and tuple(r["required_evidence_kind"] for r in rows)==KINDS
            and all(r["status"]==UNMET and
                    r["self_declaration_acceptable"] is False for r in rows),
            "E12_TWELVE_NATIVE_GATES_FALSE_PROMOTION")
    policy=c["provenance_trust_policy"]
    require(policy["accept_only_out_of_band_anchored_external_control"] is False
            and policy["self_declarations_never_satisfy_gates"] is True
            and policy["cryptographically_valid_same_admin_receipt_only_local"] is True
            and policy["machine_can_prepare_formalities"] is True
            and policy["machine_can_automatically_verify_when_real_trust_exists"] is True
            and policy["do_not_fabricate_external_authority"] is True,
            "E12_FAKE_EXTERNAL_TRUST_POLICY")
    limits=c["review_limits"]
    require(limits=={
        "formal_independent_reviews":"NOT_SUBMITTED",
        "owner_oob_root":"NOT_AUTHENTICATED",
        "external_custodian":"NOT_ENROLLED",
        "global_monotonic_antirollback":"NOT_VERIFIED",
        "independent_quorum":"NOT_PROVEN",
        "r6f":"IN_PROGRESS_UNCERTIFIED",
        "r6g":"LOCKED_BY_SEQUENCE",
        "native_admission_gates_unmet":12,
        "production":"NOT_AUTHORIZED",
        "merge_allowed":False},
        "E12_FORMAL_AUTHORITY_OR_GATES_FALSIFIED")
    return deny("E12_TWELVE_EXTERNAL_GATES_UNMET_INTAKE_AUDIT_ONLY",
                source_head=head,intake_gate_count=12,
                actual_external_anchor_count=0)

def asserted_claim(head,all_met=True,local_receipt_sha=None):
    require(isinstance(head,str) and e6.SHA40.fullmatch(head),
            "E12_MALFORMED_HEAD")
    if local_receipt_sha is not None:
        require(isinstance(local_receipt_sha,str) and e6.SHA.fullmatch(local_receipt_sha),
                "E12_INVALID_LOCAL_RECEIPT_HASH")
    return {
        "schema":CLAIM_SCHEMA,
        "project":"eddarsglobal/nordoi",
        "source_head":head,
        "claimed_domains_separate":True,
        "claimed_owner_oob_verified":True,
        "claimed_quorum_verified":True,
        "claimed_external_custodian_enrolled":True,
        "claimed_monotonic_history":True,
        "local_same_admin_receipt_sha256":local_receipt_sha,
        "claimed_native_gate_states":[{
            "id":id,"asserted_state":"MET" if all_met else UNMET
            } for id in GATES]
    }

def unsafe_naive_self_promotion(claim):
    """Insecure counterexample only: never called as a native gate."""
    return bool(claim["claimed_domains_separate"]
            and claim["claimed_owner_oob_verified"]
            and claim["claimed_quorum_verified"]
            and claim["claimed_external_custodian_enrolled"]
            and claim["claimed_monotonic_history"]
            and all(r["asserted_state"]=="MET"
                    for r in claim["claimed_native_gate_states"]))

def intake_untrusted_json(root,head,raw,local_crypto_valid=False):
    audit(root,head)
    c=e3.strictly_load(raw)
    require(set(c)=={
        "schema","project","source_head","claimed_domains_separate",
        "claimed_owner_oob_verified","claimed_quorum_verified",
        "claimed_external_custodian_enrolled","claimed_monotonic_history",
        "local_same_admin_receipt_sha256","claimed_native_gate_states"},
        "E12_SELF_CLAIM_SCHEMA_INVALID")
    require(c["schema"]==CLAIM_SCHEMA
            and c["project"]=="eddarsglobal/nordoi"
            and c["source_head"]==head,
            "E12_UNTRUSTED_CLAIM_WRONG_BINDING")
    names=("claimed_domains_separate","claimed_owner_oob_verified",
           "claimed_quorum_verified","claimed_external_custodian_enrolled",
           "claimed_monotonic_history")
    require(all(type(c[k]) is bool for k in names),
            "E12_MALFORMED_SELF_ASSERTED_BOOL")
    digest=c["local_same_admin_receipt_sha256"]
    require(digest is None or (type(digest) is str and
            e6.SHA.fullmatch(digest)),
            "E12_SELF_CLAIM_LOCAL_RECEIPT_HASH_INVALID")
    gates=c["claimed_native_gate_states"]
    require(type(gates) is list and len(gates)==12
            and all(type(row) is dict and
                    set(row)=={"id","asserted_state"}
                    for row in gates)
            and tuple(r["id"] for r in gates)==GATES
            and all(r["asserted_state"] in ("MET",UNMET) for r in gates),
            "E12_SELF_CLAIM_MISSING_DUPLICATE_OR_UNKNOWN_GATE")
    require(type(local_crypto_valid) is bool,
            "E12_CRYPTO_VALIDITY_NOT_MACHINE_BOOL")
    # Not even a real LOCAL SSHSIG can authenticate an external control root.
    # Provenance of local signature is supplied by actual e6 verify, never the JSON.
    return deny("E12_SELF_DECLARATIONS_AND_LOCAL_SSHSIG_CANNOT_UNLOCK_GATES",
        exact_source_head=head,
        untrusted_claims_count=12,
        self_declarations_all_met=unsafe_naive_self_promotion(c),
        naive_only_self_claim_gate_would_promote=unsafe_naive_self_promotion(c),
        actual_local_sshsig_verified=local_crypto_valid,
        verified_external_trust_anchors=0,
        external_issuer_authentication="NOT_VERIFIED",
        trusted_external_gate_admissions=0)

def exercise(root,head):
    audit(root,head)
    require(e6.r4.is_posix_ssh(),
            "E12_ACTUAL_POSIX_OPENSSH_REQUIRED_NOT_SIMULATED")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-e12-real-sshsig-") as home:
        p=Path(home);os.chmod(p,0o700)
        fixture=e6.local_crypto_fixture(p,head)
        actual=e6.check_second_process(fixture)
        require(actual["result"]=="BLOCK"
                and actual["cryptographic_signature_valid"] is True
                and actual["reason"]==
                "E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS",
                "E12_REAL_OPENSSH_RECEIPT_NOT_VERIFIED")
        receipt_hash=hashlib.sha256(fixture["raw"]).hexdigest()
        untrusted=asserted_claim(head,all_met=True,
                                 local_receipt_sha=receipt_hash)
        raw=e11.e10.canonical(untrusted)
        checked=intake_untrusted_json(root,head,raw,local_crypto_valid=True)
        require(checked["result"]=="BLOCK"
                and checked["naive_only_self_claim_gate_would_promote"] is True
                and checked["actual_local_sshsig_verified"] is True
                and checked["trusted_external_gate_admissions"]==0
                and checked["native_admission_gates_unmet"]==12,
                "E12_REAL_LOCAL_SIGNER_FALSE_NATIVE_AUTHORITY")
        # Real second OpenSSH validation of SAME receipt remains valid
        # for same-admin observer, not an independent administrative domain.
        replay=e6.check_second_process(fixture)
        require(replay["cryptographic_signature_valid"] is True
                and replay["result"]=="BLOCK",
                "E12_TRUE_REPLAY_LOCAL_SIGNER_EXPERIMENT_FAILED")
    require(e11.e10.head_at(root)==head,
            "E12_SOURCE_HEAD_CHANGED")
    return deny("E12_REAL_SSHSIG_SIGNED_SELF_CLAIMS_DENIED_ALL_TWELVE_GATES",
        exact_source_head=head,
        true_posix_local_ed25519_signatures_verified=2,
        naive_self_attestation_would_open_twelve=True,
        actual_external_trust_roots=0,
        actual_accepted_external_evidence=0,
        all_twelve_native_gates_remain_unmet=True,
        human_testing_performed=False,
        native_effect_performed=False)

def main(argv=None):
    ap=argparse.ArgumentParser(description=__doc__)
    action=ap.add_mutually_exclusive_group(required=True)
    action.add_argument("--audit",action="store_true")
    action.add_argument("--exercise",action="store_true")
    ap.add_argument("--expected-head",required=True)
    ap.add_argument("--root",default=str(ROOT))
    a=ap.parse_args(argv)
    try:
        root=Path(a.root).resolve(strict=True)
        d=audit(root,a.expected_head) if a.audit else exercise(root,a.expected_head)
        code=2
    except (Blocked,e11.Blocked,e11.e10.Blocked,e3.Rejected,
            e6.Blocked,e6.e5.Blocked,e6.ssh.Refused,e6.signer.Refused,guardian.Blocked,
            KeyError,TypeError,ValueError,OSError) as err:
        d=deny("E12_EVIDENCE_INTAKE_OR_CRYPTO_LAB_NOT_QUALIFIED",
               error_type=type(err).__name__)
        code=3
    sys.stdout.buffer.write(e11.e10.canonical(d))
    sys.stdout.buffer.flush()
    return code

if __name__=="__main__":raise SystemExit(main())
