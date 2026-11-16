#!/usr/bin/env python3
"""NORDOI G0.3 R6F BUREAU-001 R6 — genuine local keys vs external custody.

DENY-ONLY research feasibility gate: an exact, genuinely signed local dossier
can authenticate possession of two actual Ed25519 private keys. Even if it
claims an independent external guardian, an owner OOB fingerprint, a quorum
or a global monotonic checkpoint, the local signer cannot authenticate those
claims. Replays the authentic R5Q three-file coherent rollback counterexample.

No external connection, enrollment, independent reviewer signature, secret
export, production, native admission or user authority is performed.
"""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

import g03_r6e_oob_origin_intake as r6e
import g03_r6f_r4_fd_race as r4
import g03_r6f_r3_signer_binding as r3
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5x_signed_external_provenance as signer
import g03_r5q_coherent_local_rollback as history
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6F-R6-EXTERNAL-CUSTODY-FEASIBILITY-NO-ADMISSION-V1"
PARENT = "4d3bd3cbff8c67cef98481750270a2c6c0ce7b37"
PROJECT = "eddarsglobal/nordoi"
OPERATION = "RESEARCH_EXTERNAL_CUSTODY_EVIDENCE_INTAKE_DENY_ONLY"
NAMESPACE = "nordoi-r6f-r6-feasibility"
FIELDS = frozenset((
    "protocol","parent","project","operation","r6e_receipt_sha256",
    "r6e_challenge","owner_key_sha256","guardian_key_sha256",
    "locally_issued_at","locally_expires_at",
    "claimed_owner_oob_fingerprint_confirmed","claimed_external_operator_id",
    "claimed_separate_administrative_domain","claimed_remote_monotonic_head",
    "claimed_external_nonce_floor","claimed_external_revocation_floor",
    "claimed_independent_quorum","claimed_bureau_certification",
))
CLAIMS = (
    "claimed_owner_oob_fingerprint_confirmed","claimed_external_operator_id",
    "claimed_separate_administrative_domain","claimed_remote_monotonic_head",
    "claimed_external_nonce_floor","claimed_external_revocation_floor",
    "claimed_independent_quorum","claimed_bureau_certification",
)
MAX_PACKET_BYTES = 3200
MAX_LOCAL_TTL = 300
HEX64 = re.compile(r"[0-9a-f]{64}\Z")

class Refused(ValueError):
    pass

def deny(reason, **kwargs):
    return r6e.deny(reason, **kwargs)

def canonical(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=True,
                       separators=(",",":"), allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    obj={}
    for key,value in pairs:
        if key in obj:
            raise Refused("DUPLICATE_CUSTODY_FIELD")
        obj[key]=value
    return obj

def nonfinite(_):
    raise Refused("NONFINITE_EXTERNAL_PROVENANCE_ASSERTION")

def pin(value):
    return type(value) is str and HEX64.fullmatch(value) is not None

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_PACKET_BYTES:
        raise Refused("INVALID_SIZE_OR_NONBYTES_CUSTODY_PACKET")
    try:
        obj=json.loads(raw.decode("utf-8"),
                       object_pairs_hook=unique,parse_constant=nonfinite)
    except (UnicodeError,json.JSONDecodeError) as e:
        raise Refused("INVALID_CANONICAL_CUSTODY_JSON") from e
    if type(obj) is not dict or obj.keys()!=FIELDS or canonical(obj)!=raw:
        raise Refused("WRONG_FIELDS_OR_NONCANONICAL_CUSTODY_PACKET")
    if (obj["protocol"],obj["project"],obj["parent"],obj["operation"])!=(
        PROTOCOL,PROJECT,PARENT,OPERATION
    ):
        raise Refused("WRONG_SOURCE_PROTOCOL_OR_PRIVILEGED_OPERATION")
    for key in ("r6e_receipt_sha256","r6e_challenge",
                "owner_key_sha256","guardian_key_sha256"):
        if not pin(obj[key]):
            raise Refused("INVALID_LOCAL_EVIDENCE_SHA256_"+key)
    if obj["owner_key_sha256"]==obj["guardian_key_sha256"]:
        raise Refused("SINGLE_KEY_CANNOT_BE_TWO_INDEPENDENT_KEYS")
    for key in ("locally_issued_at","locally_expires_at"):
        if type(obj[key]) is not int or obj[key]<0:
            raise Refused("INVALID_LOCAL_CLOCK_"+key)
    if not 0 < obj["locally_expires_at"]-obj["locally_issued_at"] <= MAX_LOCAL_TTL:
        raise Refused("INVALID_UNTRUSTED_LOCAL_TIMEBOX")
    for key in CLAIMS:
        value=obj[key]
        if value is not None and (
            type(value) not in (str,int,bool)
            or (type(value) is str and (not 1 <= len(value) <= 128 or "\x00" in value))
            or (type(value) is int and not 0<=value<=1000000000)
        ):
            raise Refused("BAD_SELF_REPORTED_AUTHORITY_CLAIM_"+key)
    return obj

def make(local_r6e_raw, issued=1200, ttl=180, **claims):
    """Bind local packet to *actual parsed* original R6E evidence bytes."""
    receipt=r6e.parse(local_r6e_raw)
    if set(claims)-set(CLAIMS):
        raise Refused("UNRECOGNIZED_EXTERNAL_AUTHORITY_CLAIM")
    record=dict(protocol=PROTOCOL,parent=PARENT,project=PROJECT,
                operation=OPERATION,
                r6e_receipt_sha256=hashlib.sha256(local_r6e_raw).hexdigest(),
                r6e_challenge=receipt["challenge"],
                owner_key_sha256=receipt["owner_key_sha256"],
                guardian_key_sha256=receipt["candidate_key_sha256"],
                locally_issued_at=issued,locally_expires_at=issued+ttl)
    record.update({name:claims.get(name) for name in CLAIMS})
    return canonical(parse(canonical(record)))

def assess(raw, local_r6e_raw, now, data, *, owner_signature=None,
           guardian_signature=None, owner_allowed=None, guardian_allowed=None):
    """Verify actual local SSHSIGs, never authenticate external provenance."""
    try:
        obj=parse(raw)
        orig=r6e.parse(local_r6e_raw)
        if type(now) is not int or now < 0:
            raise Refused("INVALID_LOCAL_CLOCK")
        if (obj["r6e_receipt_sha256"]!=hashlib.sha256(local_r6e_raw).hexdigest()
            or obj["r6e_challenge"]!=orig["challenge"]
            or obj["owner_key_sha256"]!=orig["owner_key_sha256"]
            or obj["guardian_key_sha256"]!=orig["candidate_key_sha256"]):
            raise Refused("CUSTODY_CLAIM_NOT_BOUND_TO_FROZEN_R6E_EVIDENCE")
        if not obj["locally_issued_at"] <= now <= obj["locally_expires_at"]:
            return deny("UNTRUSTED_LOCAL_CUSTODY_TIMEBOX_EXPIRED")
        if not r4.is_posix_ssh():
            return deny("GENUINE_POSIX_OPENSSH_NOT_AVAILABLE")
        if type(data) is not dict:
            raise Refused("R6_LOCAL_RESEARCH_FIXTURE_REQUIRED")
        owner_signature=(data.get("r6_owner_signature") if owner_signature is None
                         else owner_signature)
        guardian_signature=(data.get("r6_guardian_signature") if guardian_signature is None
                            else guardian_signature)
        owner_allowed=(data["owner_allowed"] if owner_allowed is None
                       else owner_allowed)
        guardian_allowed=(data["guardian_allowed"] if guardian_allowed is None
                         else guardian_allowed)
        for file in (owner_signature,guardian_signature):
            if file is None or Path(file).is_symlink() or not Path(file).is_file():
                raise Refused("EXACT_LOCAL_SIGNATURE_FILE_MISSING_OR_SYMLINK")
        # Real code-identity and SSHSIG chain, as previously established in R4.
        with ExitStack() as stack:
            owner, allowed=r4.freeze_evidence(
                stack, data, guardian_allowed,owner_allowed,guardian_allowed
            )
            for public, keyhash in (
                (data["owner_public"],obj["owner_key_sha256"]),
                (data["guardian_public"],obj["guardian_key_sha256"]),
            ):
                key=owner[str(Path(public))]
                if hashlib.sha256(r3.parse_ed25519_public(key.data)+b"\n").hexdigest()!=keyhash:
                    raise Refused("LOCAL_DECLARED_PUBLIC_KEY_PIN_NOT_MATCHED")
            signatures = (
                (owner_signature,owner_allowed),
                (guardian_signature,guardian_allowed)
            )
            for path, allow in signatures:
                with fdgate.OpenSnapshot(Path(path)) as sig:
                    cmd=["-Y","verify","-f",str(allow),
                         "-I",ssh.IDENTITY,"-n",NAMESPACE,"-s",str(path)]
                    if r4.genuine_fd_verify(cmd,raw,allowed) != 0:
                        raise Refused("ACTUAL_ED25519_SSHSIG_INVALID_OR_WRONG_NAMESPACE")
                    sig.check()
            for f in owner.values():
                f.check()
            for f in allowed.values():
                f.check()
        if any(obj[key] is not None for key in CLAIMS):
            return deny(
                "TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS",
                true_local_ed25519_signatures_verified=2,
                signer_keys_distinct=True,
                same_administrator_provenance_not_external=True,
                externally_confirmed_owner_oob_root=False,
                independent_custodian_not_authenticated=True,
                external_monotonic_custody_not_verified=True,
            )
        return deny(
            "TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE",
            true_local_ed25519_signatures_verified=2,
            signer_keys_distinct=True,
            independent_operator_attested=False,
            trusted_global_nonce_floor=False,
            trusted_recovery_audit=False,
        )
    except (Refused,r4.Refused,r3.Refused,r6e.Refused,fdgate.Refused,
            ValueError,TypeError,KeyError,OSError,subprocess.SubprocessError) as e:
        return deny("R6_LOCAL_CUSTODY_FEASIBILITY_PROOF_QUARANTINED",
                    detail=(str(e) if isinstance(e, (
                        Refused,r4.Refused,r3.Refused,r6e.Refused,fdgate.Refused
                    )) else type(e).__name__))

def sign_lab(data, raw, *, filename="r6_owner_local_evidence.json"):
    """Generate genuine two-key local fixtures; never an external identity."""
    path=data["root"]/filename
    if path.exists():
        raise Refused("R6_DISPOSABLE_LOCAL_EVIDENCE_ALREADY_EXISTS")
    path.write_bytes(raw)
    if os.name!="nt":
        os.chmod(path,0o600)
    owner=signer.sign_local(path,data["owner_key"],NAMESPACE)
    # Each real signature is over the *same* original canonical packet.
    second=data["root"]/("r6_guardian_"+filename)
    second.write_bytes(raw)
    if os.name!="nt":
        os.chmod(second,0o600)
    guardian=signer.sign_local(second,data["candidate_key"],NAMESPACE)
    return owner,guardian

def local_lab(claims=None, filename="r6_owner_local_evidence.json"):
    home,data=r6e.create_lab()
    try:
        raw=make(data["r6e_body"],**(claims or {}))
        a,b=sign_lab(data,raw,filename=filename)
        return home,dict(data,r6_owner_signature=a,
                         r6_guardian_signature=b,r6_claim_raw=raw)
    except Exception:
        home.cleanup()
        raise

def inspect_real_rollback():
    result=history.native_exercise()
    if result.get("reason")!="REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED":
        raise Refused("HISTORIC_R5Q_REAL_COHERENT_ROLLBACK_NOT_REPRODUCED")
    if not (result["same_admin_all_three_restored"]
            and result["fresh_os_process_observed_old_local_history"]
            and result["globally_spent_nonce_reissued_after_full_rollback"]
            and result["protected_independent_high_water"] is False):
        raise Refused("HISTORIC_R5Q_LOCAL_GLOBAL_HISTORY_CONTRADICTION_MISSING")
    return result

def native_exercise():
    if not r4.is_posix_ssh():
        return deny("R6_GENUINE_POSIX_OPENSSH_REQUIRED"),2
    local_home,local=local_lab()
    try:
        r4_result=r4.assess_lab(local)
        if r4_result["reason"]!=(
            "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY"
        ):
            raise Refused("R4_ORIGINAL_REAL_SIGNATURE_CHAIN_MISSING")
        valid=assess(local["r6_claim_raw"],local["r6e_body"],1200,local)
        if valid["reason"]!="TWO_REAL_LOCAL_SSHSIGS_NO_INDEPENDENT_CUSTODY_EVIDENCE":
            raise Refused("TWO_REAL_GENUINE_LOCAL_SSHSIGS_NOT_QUALIFIED")
    finally:
        local_home.cleanup()
    claims={
        "claimed_owner_oob_fingerprint_confirmed":True,
        "claimed_external_operator_id":"local-lab-pretending-independent",
        "claimed_separate_administrative_domain":True,
        "claimed_remote_monotonic_head":"f"*64,
        "claimed_external_nonce_floor":10,
        "claimed_external_revocation_floor":10,
        "claimed_independent_quorum":2,
        "claimed_bureau_certification":True,
    }
    home,data=local_lab(claims=claims, filename="r6_false_custodian_claim.json")
    try:
        spoof=assess(data["r6_claim_raw"],data["r6e_body"],1200,data)
        if spoof["reason"]!="TWO_REAL_LOCAL_SSHSIGS_SIGNED_UNTRUSTED_EXTERNAL_CLAIMS":
            raise Refused("GENUINELY_SIGNED_SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_REFUSED")
        if spoof.get("true_local_ed25519_signatures_verified")!=2:
            raise Refused("REAL_TWO_SSHSIGS_NOT_CRYPTOGRAPHICALLY_QUALIFIED")
    finally:
        home.cleanup()
    inspect_real_rollback()
    return deny(
        "R6_REAL_LOCAL_TWO_KEY_OOB_CLAIMS_AND_CROSS_EPOCH_ROLLBACK_NO_EXTERNAL_AUTHORITY",
        true_local_ssh_signatures_verified=2,
        malicious_same_admin_oob_claim_genuinely_signed=True,
        false_external_guardian_operator_statement_refused=True,
        original_r4_held_fd_signature_chain_valid=True,
        real_r5q_three_file_coherent_rollback_reproduced=True,
        independent_owner_out_of_band_root_authenticated=False,
        independent_custodian_enrolled=False,
        global_monotonic_antirollback_established=False,
        independent_quorum_established=False,
    ),2

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise",action="store_true",required=True)
    parser.parse_args(argv)
    try:
        outcome,code=native_exercise()
    except (Refused,r4.Refused,r3.Refused,r6e.Refused,
            ValueError,TypeError,KeyError,OSError,subprocess.SubprocessError) as e:
        outcome,code=deny(
            "R6_REAL_EXTERNAL_CUSTODY_FEASIBILITY_EXERCISE_QUARANTINED",
            detail=(str(e) if isinstance(e,(Refused,r4.Refused,
                                            r3.Refused,r6e.Refused))
                   else type(e).__name__)
        ),3
    print(json.dumps(outcome,sort_keys=True))
    return code

if __name__=="__main__":
    sys.exit(main())
