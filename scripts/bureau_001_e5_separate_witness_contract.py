#!/usr/bin/env python3
"""NORDOI R6F BUREAU-001 E5 — separate witness PROTOCOL DESIGN / no admission.

This source-bound intake rejects any local self-attested witness/guardian
certificate. A plausible JSON packet, even with multiple genuine SAME-ADMIN
OpenSSH Ed25519 signatures, does not prove separation of administration,
owner-authenticated OOB trust or independently retained monotonic history.

--audit: exact source-bound proof obligation audit (intentional BLOCK exit 2).
--exercise: genuine two-key local SSHSIG + three-file R5Q rollback (POSIX only)
and false self-certified witness claim (intentional BLOCK exit 2).
No real external operator contacted, no network calls, no human testers.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
sys.path.insert(0,str(ROOT/"research"))
import bureau_001_e4_coherent_evidence_tampering as e4
import bureau_001_e3_exact_head_provenance as e3
import roadmap_guardian as guardian
import g03_r6f_r6_external_custody_feasibility as r6

CONTRACT="governance/NORDOI_R6F_BUREAU001_E5_SEPARATE_WITNESS_CONTRACT.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E5-SEPARATE-WITNESS-TRUST-CONTRACT-V1"
PROTOCOL="NORDOI-R6F-E5-UNTRUSTED-WITNESS-PROPOSAL-NO-ADMISSION-V1"
PARENT="0c4a4f5b1b349edb445871c4980daba57e936a59"
PROJECT="eddarsglobal/nordoi"
FIELDS=frozenset((
    "protocol","project","source_head","producer_manifest_sha256",
    "previous_witness_receipt_sha256","challenge_nonce_sha256",
    "declared_key_epoch","declared_revocation_floor",
    "declared_spent_nonce_floor","locally_claimed_separate_admin",
    "locally_claimed_oob_verified","locally_claimed_external_quorum",
    "locally_claimed_authorization"))
CLAIMS=(
    "locally_claimed_separate_admin","locally_claimed_oob_verified",
    "locally_claimed_external_quorum","locally_claimed_authorization")
NUMBERS=("declared_key_epoch","declared_revocation_floor",
         "declared_spent_nonce_floor")
HEX40=re.compile(r"[0-9a-f]{40}\Z")
HEX64=re.compile(r"[0-9a-f]{64}\Z")
EXPECTED_OBLIGATIONS=(
    "independent_root","operator_domain","freshness",
    "external_monotonicity","threshold_and_recovery","formal_approval")
EXPECTED_STATES=(
    "NOT_AUTHENTICATED","NOT_ENROLLED","NOT_VERIFIED",
    "NOT_VERIFIED","NOT_PROVEN","NOT_DECIDED")

class Blocked(ValueError):
    pass

def require(ok,reason):
    if not ok:raise Blocked(reason)

def unique(pairs):
    d={}
    for k,v in pairs:
        if k in d:raise Blocked("E5_DUPLICATE_UNTRUSTED_CLAIM")
        d[k]=v
    return d

def nonfinite(_):
    raise Blocked("E5_NONFINITE_CLAIM")

def canonical(obj):
    return (json.dumps(obj,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def parse(raw,expected_head):
    require(type(raw) is bytes and 0<len(raw)<=4096,
            "E5_WRONG_PACKET_SIZE_OR_TYPE")
    require(type(expected_head) is str and HEX40.fullmatch(expected_head),
            "E5_BAD_EXACT_SOURCE_HEAD")
    try:
        p=json.loads(raw.decode("utf-8"),
                     object_pairs_hook=unique,parse_constant=nonfinite)
    except (ValueError,UnicodeError) as e:
        if isinstance(e,Blocked):raise
        raise Blocked("E5_BAD_CLAIM_JSON") from e
    require(type(p) is dict and p.keys()==FIELDS and
            canonical(p)==raw,"E5_NONCANONICAL_OR_EXTRA_AUTHORITY_FIELDS")
    require(p["protocol"]==PROTOCOL and p["project"]==PROJECT and
            p["source_head"]==expected_head,
            "E5_WRONG_PROTOCOL_OR_SOURCE_HEAD")
    for key in ("producer_manifest_sha256","previous_witness_receipt_sha256",
                "challenge_nonce_sha256"):
        require(type(p[key]) is str and HEX64.fullmatch(p[key]),
                "E5_INVALID_SHA256_FIELD_"+key)
    for key in CLAIMS:
        require(type(p[key]) is bool,"E5_FALSE_BOOLEAN_AUTHORITY_CLAIM_"+key)
    for key in NUMBERS:
        require(type(p[key]) is int and 0<=p[key]<=2**63-1,
                "E5_INVALID_UNTRUSTED_MONOTONIC_NUMBER_"+key)
    return p

def local_claim(head,**changes):
    p=dict(
        protocol=PROTOCOL,project=PROJECT,source_head=head,
        producer_manifest_sha256=
           "099a23953ba7815556d82b393555a7ab475aa4b946684325e85c9e7e2392b77e",
        previous_witness_receipt_sha256="0"*64,
        challenge_nonce_sha256=hashlib.sha256(
           ("E5_UNTRUSTED_LOCAL_TEST_NONCE:"+head).encode()).hexdigest(),
        declared_key_epoch=1,declared_revocation_floor=0,
        declared_spent_nonce_floor=0,
        locally_claimed_separate_admin=False,
        locally_claimed_oob_verified=False,
        locally_claimed_external_quorum=False,
        locally_claimed_authorization=False)
    require(not (set(changes)-FIELDS),"E5_UNKNOWN_FIELD_IN_LOCAL_CLAIM_FACTORY")
    p.update(changes)
    return canonical(p)

def refusal(reason,**details):
    return dict(
        result="BLOCK",reason=reason,
        observed_local_signatures_verified=details.get("observed_local_signatures_verified",0),
        proved_independent_custodians=0,
        verified_external_monotonic_ledger=False,
        owner_oob_root="NOT_AUTHENTICATED",
        external_custodian="NOT_ENROLLED",
        independent_quorum=False,
        authorized_native_gates=0,
        native_admission_gates_unmet=12,
        human_testers=0,independent_specialist_reviews=0,
        r6f="IN_PROGRESS_UNCERTIFIED",
        r6g="LOCKED_BY_SEQUENCE",
        production="NOT_AUTHORIZED",
        **{k:v for k,v in details.items()
           if k!="observed_local_signatures_verified"})

def assess(raw,expected_head):
    try:
        p=parse(raw,expected_head)
        asserted=any(p[k] for k in CLAIMS)
        return refusal("E5_SELF_DECLARED_EXTERNAL_WITNESS_NOT_PROVEN"
                      if asserted else
                      "E5_NO_INDEPENDENT_EXTERNAL_WITNESS_EVIDENCE",
                      local_asserted_authority_count=sum(bool(p[k]) for k in CLAIMS),
                      locally_declared_epoch=p["declared_key_epoch"],
                      locally_declared_nonce_floor=p["declared_spent_nonce_floor"],
                      self_asserted_flags_cannot_grant_authority=True)
    except (Blocked,ValueError,TypeError,KeyError) as e:
        return refusal("E5_INVALID_WITNESS_PROPOSAL_QUARANTINED",
                       detail=str(e) if isinstance(e,Blocked)
                              else type(e).__name__)

def audit(root,expected_head):
    root=Path(root).resolve(strict=True)
    baseline=e4.audit(root,expected_head)
    p=e3.strictly_load(e3.real_file_bytes(root/CONTRACT))
    require(p["schema"]==SCHEMA and p["project"]==PROJECT and
            p["parent_e4_exact_head"]==PARENT and
            p["mandate"]==
            "DESIGN_AND_NEGATIVE_AUDIT_OF_SEPARATE_EVIDENCE_WITNESS_NOT_ENROLLMENT",
            "E5_SOURCE_OR_MANDATE_NOT_PINNED")
    obs=p["prior_e4_observed"]
    require(obs["actual_mac_intel_machine_suites"]=="9/9_PASS" and
            obs["coherent_same_admin_log_and_manifest_forgery"]=="REPRODUCED" and
            obs["prior_unmodified_local_anchor_detects"] is True and
            obs["rewritten_same_admin_anchor_fails_to_protect"] is True and
            obs["github_ci_specialized_run"]==44023579163 and
            obs["general_rust_run"]==44023579257 and
            obs["github_ci"]=="10/10_SUCCESS" and
            obs["real_machine_manifest_sha256"]==
              "099a23953ba7815556d82b393555a7ab475aa4b946684325e85c9e7e2392b77e" and
            obs["real_exercise_sha256"]==
              "6beba105ff9df2c6b3980f5c09e4846c677dc51afa0b73953874a8d85be4d61f",
            "E5_E4_REAL_MACHINE_FINDING_MISATTRIBUTED")
    domains=p["trust_domains"]
    require([d["id"] for d in domains]==[
           "producer","local_signer","candidate_witness",
           "owner_oob_anchor","external_floor"] and
           all(d["independent_from_local"] is False for d in domains[:2]) and
           all(d["independent_from_local"] is None for d in domains[2:]),
           "E5_FALSE_INDEPENDENT_ADMIN_DOMAIN")
    ps=p["proof_obligations"]
    require(tuple(x["id"] for x in ps)==EXPECTED_OBLIGATIONS and
            tuple(x["status"] for x in ps)==EXPECTED_STATES and
            all(x["real_evidence_sha256"] is None and
                x["cryptographically_authenticated_external"] is False and
                len(x["proof_obligation"])>50 for x in ps),
            "E5_FAKE_EXTERNAL_WITNESS_PROOF")
    require(set(p["required_protocol_fields"])==FIELDS and
            p["claim_semantics"]=="UNTRUSTED_LOCAL_ASSERTIONS_ONLY" and
            p["witness_receipts_received"]==0 and
            p["independent_witness_keys_authenticated"]==0 and
            p["no_human_testers"] is True and
            p["external_network_called"] is False and
            p["operator_contacted"] is False and
            p["owner_authorization_to_enroll"] is False and
            p["local_cryptographic_signatures_are_external_proof"] is False,
            "E5_FORGED_PROOF_OR_EXTERNAL_OPERATOR_CONTACT")
    g=p["exact_native_admission_gates"]
    require(tuple(x["id"] for x in g)==guardian.GATE_IDS and
            all(x["state"]==guardian.GATE_UNMET and x["promoted"] is False
                for x in g) and
            p["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
            p["r6g"]=="LOCKED_BY_SEQUENCE" and
            p["native_admission_gates_unmet"]==12 and
            p["independent_specialist_reviews"]==0 and
            p["owner_oob_root"]=="NOT_AUTHENTICATED" and
            p["independent_operator"]=="NOT_ENROLLED" and
            p["global_monotonic_antirollback"]=="NOT_VERIFIED" and
            p["production"]=="NOT_AUTHORIZED" and
            p["research_only"] is True and
            p["decision"]=="HOLD_INDEPENDENT_WITNESS_CONTRACT_NOT_ACTUAL_PROOF",
            "E5_NATIVE_GATES_OR_GOVERNANCE_PROMOTED")
    return refusal("E5_WITNESS_PROOF_OBLIGATIONS_CLASSIFIED_AND_UNMET",
                   source_head=baseline["exact_pr_head"],
                   witness_proof_obligations=6,
                   actual_external_receipts=0,
                   prior_real_coherent_log_manifest_forgery_reproduced=True)

def exercise(root,head):
    baseline=audit(root,head)
    require(r6.r4.is_posix_ssh(),"E5_TRUE_POSIX_OPENSSH_REQUIRED")
    # Actual new OS processes, genuine OpenSSH Ed25519 SSHSIG verification,
    # and real R5Q SQLite/JSON 3-file coherent history rollback, not simulated
    # external parties. Reuse original already qualified R6 native lab.
    original,exit_code=r6.native_exercise()
    require(exit_code==2 and original["reason"]==
            "R6_REAL_LOCAL_TWO_KEY_OOB_CLAIMS_AND_CROSS_EPOCH_ROLLBACK_NO_EXTERNAL_AUTHORITY"
            and original["true_local_ssh_signatures_verified"]==2 and
            original["malicious_same_admin_oob_claim_genuinely_signed"] is True and
            original["real_r5q_three_file_coherent_rollback_reproduced"] is True,
            "E5_ACTUAL_TWO_LOCAL_SSHSIG_AND_R5Q_NOT_CONFIRMED")
    forged=local_claim(head,locally_claimed_separate_admin=True,
        locally_claimed_oob_verified=True,
        locally_claimed_external_quorum=True,
        locally_claimed_authorization=True,
        declared_key_epoch=12,
        declared_revocation_floor=7,
        declared_spent_nonce_floor=999)
    denied=assess(forged,head)
    require(denied["reason"]=="E5_SELF_DECLARED_EXTERNAL_WITNESS_NOT_PROVEN"
            and denied["local_asserted_authority_count"]==4
            and denied["authorized_native_gates"]==0,
            "E5_SELF_SIGNED_FALSE_EXTERNAL_PROOF_ACCEPTED")
    return refusal(
        "E5_REAL_LOCAL_SSHSIG_FALSE_WITNESS_AND_R5Q_ROLLBACK_STILL_NO_EXTERNAL_AUTHORITY",
        observed_local_signatures_verified=2,
        genuine_r5q_cross_epoch_rollback_reproduced=True,
        claimed_quorum_and_owner_oob_root_refused=True,
        actual_independent_witness_receipts=0,
        actual_external_monotonic_history=False,
        six_proof_obligations_unmet=True,
        source_head=baseline["source_head"])

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    modes=p.add_mutually_exclusive_group(required=True)
    modes.add_argument("--audit",action="store_true")
    modes.add_argument("--exercise",action="store_true")
    p.add_argument("--root",type=Path,default=ROOT)
    p.add_argument("--expected-head",required=True)
    a=p.parse_args(argv)
    try:
        value=audit(a.root,a.expected_head) if a.audit else exercise(a.root,a.expected_head)
        code=2  # Intentional admission DENIAL is successful negative result.
    except (Blocked,e4.Blocked,e3.Rejected,
            ValueError,TypeError,KeyError,OSError,subprocess.SubprocessError) as ex:
        value=refusal("E5_SOURCE_OR_REAL_CRYPTOGRAPHIC_EXERCISE_QUARANTINED",
                 detail=str(ex) if isinstance(ex,(Blocked,e4.Blocked,e3.Rejected))
                        else type(ex).__name__)
        code=3
    print(json.dumps(value,sort_keys=True))
    return code

if __name__=="__main__":
    raise SystemExit(main())
