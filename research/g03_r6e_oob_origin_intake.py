#!/usr/bin/env python3
"""NORDOI G0.3 R6E — genuine local 'OOB' receipt, NO real OOB trust.

This is a strictly offline, DENY-ONLY provenance-intake negative control.
It links a locally signed OOB-looking receipt to the exact R6D->R5Z
history and checks genuine SSHSIG while refusing any claim that the same
administrative domain authenticated an independent external custodian.
No external network, enrollment, privileged grant, merge or production.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys

import g03_r6d_admin_domain_provenance as r6d
import g03_r6c_two_key_quorum_negative as r6c
import g03_r6b_signed_response_provenance as r6b
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as r5x
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6E-OOB-ORIGIN-NEGATIVE-INTAKE-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "cc3816dac7bf677c3137df69adb34b3c44c23d19"
OPERATION = "REVIEW_UNAUTHENTICATED_OOB_INTAKE_DENY_ONLY"
NAMESPACE = "nordoi-r6e-claimed-oob"
MAX_BYTES = 4096
MAX_TTL = 300
FIELDS = frozenset((
    "protocol", "project", "parent", "operation",
    "r6d_evidence_sha256", "r6c_envelope_sha256",
    "r6b_response_sha256", "r6a_request_sha256", "r5z_witness_sha256",
    "challenge", "owner_key_sha256", "candidate_key_sha256",
    "local_claim_transport", "locally_issued_at", "locally_expires_at",
    "claimed_external_operator_id", "claimed_owner_oob_root_sha256",
    "claimed_guardian_public_key_sha256", "claimed_external_admin_domain",
    "claimed_external_history_sha256", "claimed_remote_nonce_floor",
    "claimed_remote_revocation_floor", "claimed_external_quorum",
))

class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(pairs):
    d = {}
    for key, value in pairs:
        if key in d:
            raise Refused("DUPLICATE_OOB_ORIGIN_FIELD")
        d[key] = value
    return d

def nonfinite(_):
    raise Refused("NONFINITE_OOB_ORIGIN_NUMBER")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("OOB_ORIGIN_MISSING_OR_OVERSIZED")
    try:
        x = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=nonfinite)
    except (ValueError, UnicodeError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("OOB_ORIGIN_INVALID_JSON") from exc
    if type(x) is not dict or x.keys() != FIELDS or canonical(x) != raw:
        raise Refused("OOB_ORIGIN_FIELDS_NOT_CANONICAL")
    if (x["protocol"], x["project"], x["parent"], x["operation"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION
    ):
        raise Refused("OOB_ORIGIN_WRONG_SOURCE_OR_PRIVILEGED_VERB")
    for key in ("r6d_evidence_sha256", "r6c_envelope_sha256",
                "r6b_response_sha256", "r6a_request_sha256",
                "r5z_witness_sha256", "challenge", "owner_key_sha256",
                "candidate_key_sha256"):
        if not r6a.pin(x[key]):
            raise Refused("OOB_ORIGIN_INVALID_PIN_" + key)
    if x["owner_key_sha256"] == x["candidate_key_sha256"]:
        raise Refused("OOB_ORIGIN_DUPLICATE_KEY_IDS")
    for key in ("claimed_owner_oob_root_sha256",
                "claimed_guardian_public_key_sha256",
                "claimed_external_history_sha256"):
        if x[key] is not None and not r6a.pin(x[key]):
            raise Refused("OOB_ORIGIN_CLAIMED_PIN_INVALID_" + key)
    for key in ("claimed_external_operator_id",
                "claimed_external_admin_domain"):
        if x[key] is not None and (
            type(x[key]) is not str or
            re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}", x[key]) is None
        ):
            raise Refused("OOB_ORIGIN_CLAIMED_ID_INVALID_" + key)
    if x["local_claim_transport"] != "same-admin-local-file":
        raise Refused("OOB_ORIGIN_TRANSPORT_NOT_ACTUAL_LOCAL_FILE")
    for key in ("locally_issued_at", "locally_expires_at",
                "claimed_remote_nonce_floor",
                "claimed_remote_revocation_floor", "claimed_external_quorum"):
        if x[key] is not None and (type(x[key]) is not int or x[key] < 0):
            raise Refused("OOB_ORIGIN_COUNTER_INVALID_" + key)
    if (type(x["locally_issued_at"]) is not int or
        type(x["locally_expires_at"]) is not int):
        raise Refused("OOB_ORIGIN_LOCAL_TIME_MISSING")
    if (x["locally_expires_at"] < x["locally_issued_at"] or
        x["locally_expires_at"] - x["locally_issued_at"] > MAX_TTL):
        raise Refused("OOB_ORIGIN_UNTRUSTED_TIMEBOX_INVALID")
    return x

def deny(reason, **extras):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False,
                production="NOT_AUTHORIZED",
                independent_external_custodian="NOT_ENROLLED",
                owner_oob_root="NOT_AUTHENTICATED",
                external_global_monotonic_history="NOT_VERIFIED",
                independent_admin_quorum="NOT_PROVEN",
                native_admission_gates_unmet=12,
                reason=reason, **extras)

def source(r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw):
    d = r6d.parse(r6d_raw)
    c, b, a = r6d.source(r6c_raw, r6b_raw, r6a_raw, r5z_raw)
    if (d["r6c_envelope_sha256"], d["r6b_response_sha256"],
        d["r6a_request_sha256"], d["r5z_witness_sha256"], d["challenge"],
        d["owner_key_sha256"], d["candidate_key_sha256"]) != (
        r5y.sha(r6c_raw), r5y.sha(r6b_raw),
        r5y.sha(r6a_raw), r5y.sha(r5z_raw), c["challenge"],
        c["owner_key_sha256"], c["candidate_key_sha256"]
    ):
        raise Refused("R6D_ENVELOPE_SOURCE_NOT_EXACT")
    return d, c, b, a

def make(r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw, now, ttl=180):
    d, _, _, _ = source(r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw)
    if (type(now) is not int or now < 0 or type(ttl) is not int or
        not 1 <= ttl <= MAX_TTL):
        raise Refused("INVALID_LOCAL_INTAKE_TIMEBOX")
    return parse(canonical(dict(
        protocol=PROTOCOL, project=PROJECT, parent=PARENT, operation=OPERATION,
        r6d_evidence_sha256=r5y.sha(r6d_raw),
        r6c_envelope_sha256=r5y.sha(r6c_raw),
        r6b_response_sha256=r5y.sha(r6b_raw),
        r6a_request_sha256=r5y.sha(r6a_raw),
        r5z_witness_sha256=r5y.sha(r5z_raw),
        challenge=d["challenge"],
        owner_key_sha256=d["owner_key_sha256"],
        candidate_key_sha256=d["candidate_key_sha256"],
        local_claim_transport="same-admin-local-file",
        locally_issued_at=now, locally_expires_at=now + ttl,
        claimed_external_operator_id=None,
        claimed_owner_oob_root_sha256=None,
        claimed_guardian_public_key_sha256=None,
        claimed_external_admin_domain=None,
        claimed_external_history_sha256=None,
        claimed_remote_nonce_floor=None,
        claimed_remote_revocation_floor=None,
        claimed_external_quorum=None,
    )))

def assess(raw, r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw, now, *,
           local_signature=None, local_allowed=None, local_public=None,
           original_owner_signature=None, original_guardian_signature=None,
           parent_response_signature=None, r6d_owner_signature=None,
           r6d_candidate_signature=None, owner_allowed=None,
           candidate_allowed=None, owner_public=None, candidate_public=None,
           local_spent=(), local_revoked=()):
    """Actual signature key possession only; never validates OOB origin."""
    try:
        receipt = parse(raw)
        d, c, b, a = source(r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw)
        if type(now) is not int or now < 0:
            raise Refused("LOCAL_CLOCK_INVALID")
        if (receipt["r6d_evidence_sha256"], receipt["r6c_envelope_sha256"],
            receipt["r6b_response_sha256"], receipt["r6a_request_sha256"],
            receipt["r5z_witness_sha256"], receipt["challenge"],
            receipt["owner_key_sha256"], receipt["candidate_key_sha256"]) != (
            r5y.sha(r6d_raw), r5y.sha(r6c_raw), r5y.sha(r6b_raw),
            r5y.sha(r6a_raw), r5y.sha(r5z_raw), d["challenge"],
            d["owner_key_sha256"], d["candidate_key_sha256"]
        ):
            raise Refused("CLAIMED_OOB_PACKET_NOT_EXACT_PARENT_PROVENANCE")
        if not (receipt["locally_issued_at"] <= now <= receipt["locally_expires_at"]
                and d["issued_at_local"] <= now <= d["expires_at_local"]
                and c["issued_at_local"] <= now <= c["expires_at_local"]
                and b["issued_at_local"] <= now <= b["expires_at_local"]
                and a["issued_at_local"] <= now <= a["expires_at_local"]):
            return deny("UNTRUSTED_LOCAL_TIMEBOX_STALE_OR_FUTURE")
        for collection in (local_spent, local_revoked):
            if type(collection) not in (tuple, set, frozenset) or any(
                not r6a.pin(v) for v in collection
            ):
                raise Refused("OOB_ORIGIN_MALFORMED_LOCAL_SET")
        if receipt["challenge"] in local_spent:
            return deny("LOCAL_SPENT_CHALLENGE_NOT_EXTERNAL_NONCE_FLOOR")
        if (receipt["owner_key_sha256"] in local_revoked or
            receipt["candidate_key_sha256"] in local_revoked):
            return deny("LOCAL_REVOKED_SIGNER_NOT_EXTERNAL_REVOCATION_FLOOR")
        claimed = (
            "claimed_external_operator_id", "claimed_owner_oob_root_sha256",
            "claimed_guardian_public_key_sha256",
            "claimed_external_admin_domain", "claimed_external_history_sha256",
            "claimed_remote_nonce_floor", "claimed_remote_revocation_floor",
            "claimed_external_quorum",
        )
        if any(receipt[key] is not None for key in claimed):
            return deny("SELF_ASSERTED_OOB_ROOT_OR_EXTERNAL_CUSTODY_NOT_AUTHENTICATED",
                        asserted_external_evidence_is_untrusted=True)
        paths = (local_signature, local_allowed, local_public,
                 original_owner_signature, original_guardian_signature,
                 parent_response_signature, r6d_owner_signature,
                 r6d_candidate_signature, owner_allowed, candidate_allowed,
                 owner_public, candidate_public)
        if any(x is None for x in paths):
            return deny("EXTERNAL_OOB_OR_SIGNED_EVIDENCE_MISSING")
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("GENUINE_OPENSSH_NOT_AVAILABLE_ON_THIS_PLATFORM")
        for path in paths:
            p = Path(path)
            if p.is_symlink() or not p.is_file():
                raise Refused("OOB_INTAKE_SIGNATURE_PROOF_IS_SYMLINK_OR_MISSING")
        if (keys.public_sha(local_public) != receipt["candidate_key_sha256"]
            or keys.public_sha(owner_public) != receipt["owner_key_sha256"]
            or keys.public_sha(candidate_public) != receipt["candidate_key_sha256"]):
            raise Refused("OOB_INTAKE_REAL_PUBLIC_KEY_PIN_MISMATCH")
        parent_result = r6d.inspect(
            r6d_raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw, now,
            original_owner_signature=original_owner_signature,
            original_guardian_signature=original_guardian_signature,
            response_signature=parent_response_signature,
            r6d_owner_signature=r6d_owner_signature,
            r6d_candidate_signature=r6d_candidate_signature,
            owner_allowed=owner_allowed, candidate_allowed=candidate_allowed,
            owner_public=owner_public, candidate_public=candidate_public,
        )
        if parent_result["reason"] != "TRUE_LOCAL_OS_PAIR_AND_FIVE_SSHSIGS_NO_EXTERNAL_DOMAIN":
            raise Refused("PARENT_R6D_REAL_FIVE_SIGNATURE_CHAIN_NOT_VALID")
        with fdgate.OpenSnapshot(Path(local_signature)) as frozen:
            if ssh.run_ssh(["-Y", "verify", "-f", str(local_allowed),
                            "-I", ssh.IDENTITY, "-n", NAMESPACE,
                            "-s", str(local_signature)], raw) != 0:
                raise Refused("GENUINE_LOCAL_OOB_LOOKING_SSHSIG_INVALID")
            frozen.check()
        return deny("SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT",
                    genuine_signatures_verified=6,
                    exact_r6d_to_r5z_history_bound=True,
                    separately_authenticated_owner_channel=False,
                    external_operator_independently_verified=False,
                    externally_protected_nonce_revocation_floors=False)
    except (Refused, r6d.Refused, r6c.Refused, r6b.Refused, r6a.Refused,
            r5z.Refused, r5y.Refused, keys.Refused, fdgate.Refused,
            TypeError, OSError, ValueError) as exc:
        return deny("OOB_ORIGIN_EVIDENCE_QUARANTINED",
                    detail=str(exc) if isinstance(exc, (Refused,
                        r6d.Refused, r6c.Refused, r6b.Refused, r6a.Refused,
                        r5z.Refused, r5y.Refused, keys.Refused,
                        fdgate.Refused)) else type(exc).__name__)

def create_lab():
    home, data = r6d.create_lab()
    try:
        root = data["root"]
        receipt = canonical(make(
            data["r6d_body"], data["body"], data["response"],
            data["request"], data["witness"], 1100
        ))
        path = root / "r6e_same_admin_oob_looking_receipt.json"
        path.write_bytes(receipt)
        os.chmod(path, 0o600)
        signature = r5x.sign_local(path, data["candidate_key"], NAMESPACE)
        return home, dict(data, r6e_body=receipt,
                          r6e_local_signature=signature)
    except Exception:
        home.cleanup()
        raise

def assess_lab(data, *, raw=None, request_raw=None, now=1200, **kwargs):
    args = dict(
        local_signature=data["r6e_local_signature"],
        local_allowed=data["guardian_allowed"], local_public=data["guardian_public"],
        original_owner_signature=data["owner_signature"],
        original_guardian_signature=data["guardian_signature"],
        parent_response_signature=data["parent_response_sig"],
        r6d_owner_signature=data["r6d_owner_signature"],
        r6d_candidate_signature=data["r6d_candidate_signature"],
        owner_allowed=data["owner_allowed"],
        candidate_allowed=data["guardian_allowed"],
        owner_public=data["owner_public"],
        candidate_public=data["guardian_public"],
    )
    args.update(kwargs)
    return assess(data["r6e_body"] if raw is None else raw,
                  data["r6d_body"], data["body"], data["response"],
                  data["request"] if request_raw is None else request_raw,
                  data["witness"], now, **args)

def native_exercise():
    home, data = create_lab()
    try:
        result = assess_lab(data)
        if result["reason"] != "SIX_VALID_LOCAL_SIGNATURES_NOT_OWNER_OOB_ENROLLMENT":
            raise Refused("Genuine_local_OOB_receipt_false_admission")
        fake = dict(parse(data["r6e_body"]),
                    claimed_owner_oob_root_sha256=data["owner_pin"],
                    claimed_external_operator_id="lab-false-guardian")
        fake_raw = canonical(fake)
        path = data["root"] / "r6e_false_oob_signed.json"
        path.write_bytes(fake_raw)
        os.chmod(path, 0o600)
        fake_sig = r5x.sign_local(path, data["candidate_key"], NAMESPACE)
        if ssh.run_ssh(["-Y", "verify", "-f", str(data["guardian_allowed"]),
                        "-I", ssh.IDENTITY, "-n", NAMESPACE,
                        "-s", str(fake_sig)], fake_raw) != 0:
            raise Refused("REAL_SIGNED_FALSE_OOB_CLAIM_NOT_VERIFIABLE")
        if assess_lab(data, raw=fake_raw, local_signature=fake_sig)["reason"] != (
            "SELF_ASSERTED_OOB_ROOT_OR_EXTERNAL_CUSTODY_NOT_AUTHENTICATED"
        ):
            raise Refused("SIGNED_FALSE_OOB_CLAIM_NOT_REFUSED")
        if assess_lab(data, local_spent=frozenset([parse(data["r6e_body"])["challenge"]]))[
            "reason"
        ] != "LOCAL_SPENT_CHALLENGE_NOT_EXTERNAL_NONCE_FLOOR":
            raise Refused("LOCAL_REPLAY_NEGATIVE_CONTROL_FAILED")
        return deny("R6E_GENUINE_LOCAL_OOB_ORIGIN_NEGATIVE_CONTROL_QUALIFIED",
                    real_local_signatures="SIX_PASS_KEY_POSSESSION_ONLY",
                    independently_observed_owner_oob_channel="NONE",
                    signed_false_external_operator="BLOCK",
                    local_challenge_replay="BLOCK",
                    independently_authenticated_custodian="NONE",
                    independent_remote_monotonic_history="NONE")
    finally:
        home.cleanup()

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise", action="store_true", required=True)
    p.parse_args(argv)
    try:
        out = native_exercise()
    except (Refused, r6d.Refused, r6c.Refused, r6b.Refused, r6a.Refused,
            r5z.Refused, r5y.Refused, keys.Refused, fdgate.Refused,
            OSError, ValueError) as exc:
        out = deny("R6E_NATIVE_EVIDENCE_EXERCISE_FAILED",
                   detail=str(exc) if isinstance(exc, (Refused, r6d.Refused,
                       r6c.Refused, r6b.Refused, r6a.Refused, r5z.Refused,
                       r5y.Refused, keys.Refused, fdgate.Refused))
                   else type(exc).__name__)
    print(json.dumps(out, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
