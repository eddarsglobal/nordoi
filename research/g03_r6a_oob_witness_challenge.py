#!/usr/bin/env python3
"""NORDOI G0.3 R6A — offline external historical-witness challenge preflight.

Creates a fresh cryptographically random challenge tied to an *actual* R5Z
LOCAL signed-witness checkpoint. It may verify genuine OpenSSH Ed25519
candidate receipts, but cannot authenticate signer administrative independence
or a remotely protected global high-water mark. All outcomes BLOCK.
NO network, external enrollment, privilege change, native grant or production.
"""
import argparse
import json
import os
from pathlib import Path
import re
import secrets
import sys
import tempfile

import g03_r5z_preserved_witness_rollback as witness
import g03_r5y_signed_nonce_rollback as ledger
import g03_r5x_signed_external_provenance as receipts
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as identity
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6A-EXTERNAL-WITNESS-CHALLENGE-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "381d0fd785ab7c572f133355cc482ca34e1a0a7f"
OPERATION = "VERIFY_EXTERNAL_WITNESS_CANDIDATE_DENY_ONLY"
NAMESPACE = "nordoi-r6a"
MAX_BYTES = 4096
MAX_TTL_SECONDS = 300
LAB_PREFIX = "nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-r5y-r5z-r6a-"
FIELDS = frozenset((
    "protocol", "project", "parent", "operation", "challenge",
    "source_witness_sha256", "source_witness_height",
    "source_nonce_set_sha256", "source_local_revocation_floor",
    "source_receipt_sha256", "candidate_signer_key_sha256",
    "asserted_owner_oob_root_sha256", "asserted_guardian_admin_domain",
    "asserted_external_history_anchor_sha256",
    "asserted_external_revocation_floor", "asserted_external_spent_nonce_floor",
    "issued_at_local", "expires_at_local",
))

class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(items):
    d = {}
    for k, v in items:
        if k in d:
            raise Refused("DUPLICATE_EXTERNAL_CHALLENGE_FIELD")
        d[k] = v
    return d

def finite(_):
    raise Refused("NONFINITE_EXTERNAL_CHALLENGE")

def pin(v):
    return type(v) is str and re.fullmatch(r"[a-f0-9]{64}", v) is not None

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("MISSING_OR_OVERSIZED_EXTERNAL_CHALLENGE")
    try:
        x = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=finite)
    except (ValueError, UnicodeError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("EXTERNAL_CHALLENGE_INVALID_UTF8_JSON") from exc
    if type(x) is not dict or x.keys() != FIELDS or canonical(x) != raw:
        raise Refused("EXTERNAL_CHALLENGE_FIELDS_OR_CANONICAL_INVALID")
    if (x["protocol"], x["project"], x["parent"], x["operation"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION
    ):
        raise Refused("WRONG_SOURCE_OR_PRIVILEGED_OPERATION")
    for k in ("challenge", "source_witness_sha256",
              "source_nonce_set_sha256", "source_receipt_sha256",
              "candidate_signer_key_sha256"):
        if not pin(x[k]):
            raise Refused("INVALID_PIN_" + k)
    for k in ("asserted_owner_oob_root_sha256",
              "asserted_external_history_anchor_sha256"):
        if x[k] is not None and not pin(x[k]):
            raise Refused("INVALID_ASSERTED_EXTERNAL_PIN_" + k)
    domain = x["asserted_guardian_admin_domain"]
    if domain is not None and (type(domain) is not str or
                               re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}", domain) is None):
        raise Refused("INVALID_UNTRUSTED_DOMAIN_CLAIM")
    for k in ("source_witness_height", "source_local_revocation_floor",
              "asserted_external_revocation_floor",
              "asserted_external_spent_nonce_floor", "issued_at_local",
              "expires_at_local"):
        if x[k] is not None and (type(x[k]) is not int or x[k] < 0):
            raise Refused("INVALID_UNTRUSTED_COUNTER_" + k)
    if type(x["source_witness_height"]) is not int or x["source_witness_height"] < 1:
        raise Refused("SOURCE_WITNESS_HEIGHT_NOT_POSITIVE")
    if type(x["source_local_revocation_floor"]) is not int:
        raise Refused("SOURCE_LOCAL_REVOCATION_FLOOR_MISSING")
    if type(x["issued_at_local"]) is not int or type(x["expires_at_local"]) is not int:
        raise Refused("LOCAL_CLOCK_FIELDS_MISSING")
    if (x["expires_at_local"] < x["issued_at_local"] or
        x["expires_at_local"] - x["issued_at_local"] > MAX_TTL_SECONDS):
        raise Refused("UNTRUSTED_LOCAL_TIMEBOX_INVALID")
    return x

def deny(reason, **other):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                independent_administrative_custody="NOT_PROVEN",
                authenticated_owner_oob_root="NOT_ENROLLED",
                external_guardian="NOT_ENROLLED",
                independently_protected_monotonic_history="NOT_VERIFIED",
                native_admission_gates_unmet=12, reason=reason, **other)

def create_request(witness_raw, signer_pin, now, ttl=180, challenge=None):
    """Challenge is 256 bits OS randomness unless supplied by a unit test."""
    w = witness.parse(witness_raw)
    if not pin(signer_pin):
        raise Refused("SIGNER_KEY_PIN_INVALID")
    if (type(now) is not int or now < 0 or type(ttl) is not int or
        not 1 <= ttl <= MAX_TTL_SECONDS):
        raise Refused("INVALID_LOCAL_TIMEBOX_ARGUMENTS")
    challenge = secrets.token_hex(32) if challenge is None else challenge
    if not pin(challenge):
        raise Refused("INVALID_FRESH_CHALLENGE")
    x = {
        "protocol": PROTOCOL, "project": PROJECT, "parent": PARENT,
        "operation": OPERATION, "challenge": challenge,
        "source_witness_sha256": ledger.sha(witness_raw),
        "source_witness_height": w["local_height"],
        "source_nonce_set_sha256": w["nonce_set_sha256"],
        "source_local_revocation_floor": w["local_revocation_floor"],
        "source_receipt_sha256": w["source_receipt_sha256"],
        "candidate_signer_key_sha256": signer_pin,
        "asserted_owner_oob_root_sha256": None,
        "asserted_guardian_admin_domain": None,
        "asserted_external_history_anchor_sha256": None,
        "asserted_external_revocation_floor": None,
        "asserted_external_spent_nonce_floor": None,
        "issued_at_local": now, "expires_at_local": now + ttl,
    }
    return parse(canonical(x))

def assess(raw, witness_raw, now, signature=None, allowed=None, public_key=None,
           locally_spent=()):
    """Identity-of-key verification, NEVER independent-custody verification."""
    try:
        x = parse(raw)
        w = witness.parse(witness_raw)
        if type(now) is not int or now < 0:
            raise Refused("INVALID_CALLER_LOCAL_CLOCK")
        if (x["source_witness_sha256"], x["source_witness_height"],
            x["source_nonce_set_sha256"], x["source_local_revocation_floor"],
            x["source_receipt_sha256"]) != (
            ledger.sha(witness_raw), w["local_height"],
            w["nonce_set_sha256"], w["local_revocation_floor"],
            w["source_receipt_sha256"]
        ):
            raise Refused("EXACT_SOURCE_WITNESS_OR_HISTORY_BINDING_MISMATCH")
        if not x["issued_at_local"] <= now <= x["expires_at_local"]:
            return deny("LOCAL_UNTRUSTED_CHALLENGE_TIMEBOX_STALE_OR_FUTURE")
        if type(locally_spent) not in (tuple, frozenset, set) or any(
            not pin(v) for v in locally_spent
        ):
            raise Refused("MALFORMED_LOCAL_SPENT_CHALLENGE_REGISTER")
        if x["challenge"] in locally_spent:
            return deny("LOCALLY_SPENT_CHALLENGE_REPLAY_BLOCKED",
                        globally_protected_spent_nonce=False)
        if any(x[k] is not None for k in (
            "asserted_owner_oob_root_sha256",
            "asserted_guardian_admin_domain",
            "asserted_external_history_anchor_sha256",
            "asserted_external_revocation_floor",
            "asserted_external_spent_nonce_floor",
        )):
            return deny("SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_AUTHENTICATED",
                        structural_witness_binding=True,
                        claimed_external_fields_quarantined=True)
        if any(p is None for p in (signature, allowed, public_key)):
            return deny("EXTERNAL_WITNESS_SIGNATURE_OR_ROOT_MISSING",
                        structural_witness_binding=True)
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("NATIVE_OPENSSH_NOT_AVAILABLE")
        if x["candidate_signer_key_sha256"] != keys.public_sha(public_key):
            raise Refused("CANDIDATE_KEY_NOT_BOUND_TO_PUBLIC_KEY")
        for p in (signature, allowed, public_key):
            f = Path(p)
            if not f.is_file() or f.is_symlink():
                raise Refused("SIGNATURE_ALLOWLIST_OR_PUBLIC_KEY_NOT_REGULAR_FILE")
        # Uses actual system OpenSSH; allowed_signers is a SAME-ADMIN fixture,
        # never a trusted independent OOB root or external witness operator.
        with fdgate.OpenSnapshot(Path(signature)) as held:
            if ssh.run_ssh(["-Y", "verify", "-f", str(allowed),
                            "-I", ssh.IDENTITY, "-n", NAMESPACE,
                            "-s", str(signature)], raw) != 0:
                raise Refused("ACTUAL_OPENSSH_SIGNED_CANDIDATE_INVALID")
            held.check()
        return deny("REAL_SSHSIG_VALID_LOCAL_ONLY_NO_EXTERNAL_CUSTODY",
                    cryptographic_signature_valid=True,
                    signer_is_independently_authenticated=False,
                    externally_durable_nonce_floor=False,
                    local_clock_is_external_freshness=False)
    except (Refused, witness.Refused, ledger.Refused, keys.Refused, fdgate.Refused,
            OSError, ValueError, TypeError) as exc:
        return deny("EXTERNAL_WITNESS_CHALLENGE_QUARANTINED",
                    detail=str(exc) if isinstance(exc, (Refused, witness.Refused,
                        ledger.Refused, keys.Refused, fdgate.Refused))
                    else type(exc).__name__)

def sign_local(path, key, namespace=NAMESPACE):
    return receipts.sign_local(path, key, namespace)

def native_exercise():
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return deny("GENUINE_POSIX_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=LAB_PREFIX) as path:
        root = Path(path)
        os.chmod(root, 0o700)
        owner, _ = ssh.create_same_admin_key(root, "r6a_owner")
        signer, allowed = ssh.create_same_admin_key(root, "r6a_same_admin_candidate")
        op = keys.public_sha(root / "r6a_owner.pub")
        sp = keys.public_sha(root / "r6a_same_admin_candidate.pub")
        if op == sp:
            raise Refused("DISTINCT_ED25519_LOCAL_KEYPAIRS_NOT_OBSERVED")
        a = identity.child_observe("owner_proxy", "lab-owner-domain",
                                   op, 0, "b" * 64, "c" * 64)
        b = identity.child_observe("custodian_proxy", "lab-guardian-domain",
                                   sp, 0, "b" * 64, "c" * 64)
        if a["pid"] == b["pid"] or a["uid"] != b["uid"] or a["euid"] != b["euid"]:
            raise Refused("REAL_SAME_ADMIN_OS_IDENTITIES_NOT_OBSERVED")
        ar, br = identity.canonical(a), identity.canonical(b)
        dossier = intake.canonical(intake.candidate(ar, br))
        receipt = receipts.canonical(receipts.make_candidate(dossier, sp, 1000))
        local_receipt = root / "r6a_local_receipt.json"
        local_receipt.write_bytes(receipt)
        os.chmod(local_receipt, 0o600)
        receipt_sig = receipts.sign_local(local_receipt, signer)
        check = receipts.verify(receipt, dossier, ar, br, 1100, receipt_sig,
                                allowed, root / "r6a_same_admin_candidate.pub")
        if check["reason"] != "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN":
            raise Refused("REAL_SIGNED_R5X_RECEIPT_NOT_VERIFIED")
        ledger.bootstrap(root)
        if ledger.commit_local_denial(root, receipt, 1100, 1)["reason"] != (
            "LOCAL_ONLY_SPENT_NONCE_COMMITTED"
        ):
            raise Refused("REAL_R5Y_SQLITE_CHECKPOINT_NOT_ESTABLISHED")
        wraw = witness.canonical(witness.candidate(root, receipt, 1100))
        local_witness = witness.save_witness(root, wraw, witness.WITNESS)
        wsignature = witness.sign_local(local_witness, signer)
        if witness.verify_local(wraw, wsignature, allowed)["reason"] != (
            "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL"
        ):
            raise Refused("REAL_R5Z_SIGNED_WITNESS_INVALID")
        request = create_request(wraw, sp, 1100)
        raw = canonical(request)
        path = root / "r6a_external_request.json"
        path.write_bytes(raw)
        os.chmod(path, 0o600)
        signature = sign_local(path, signer)
        valid = assess(raw, wraw, 1200, signature, allowed,
                       root / "r6a_same_admin_candidate.pub")
        if valid["reason"] != "REAL_SSHSIG_VALID_LOCAL_ONLY_NO_EXTERNAL_CUSTODY":
            raise Refused("REAL_CANDIDATE_SIGNATURE_FALSELY_AUTHORIZED")
        fake = dict(request, asserted_owner_oob_root_sha256=op,
                    asserted_external_history_anchor_sha256=sp,
                    asserted_external_revocation_floor=999)
        fake_result = assess(canonical(fake), wraw, 1200)
        if fake_result["reason"] != "SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_AUTHENTICATED":
            raise Refused("LOCAL_SELF_SIGNED_EXTERNAL_CLAIM_FALSELY_ACCEPTED")
        replay = assess(raw, wraw, 1200, signature, allowed,
                        root / "r6a_same_admin_candidate.pub",
                        locally_spent=frozenset([request["challenge"]]))
        if replay["reason"] != "LOCALLY_SPENT_CHALLENGE_REPLAY_BLOCKED":
            raise Refused("SPENT_LOCAL_CHALLENGE_NOT_BLOCKED")
        return deny("REAL_R5Z_WITNESS_OOB_CHALLENGE_NEGATIVE_CONTROL_QUALIFIED",
                    genuine_local_receipt_ssh_ed25519="PASS_KEY_POSSESSION_ONLY",
                    actual_local_signed_r5z_witness="PASS_LOCAL_ONLY",
                    actual_fresh_os_random_challenge="256_BIT_VALUE",
                    same_admin_valid_candidate_signature="BLOCK",
                    self_asserted_oob_claim="BLOCK",
                    local_spent_challenge="BLOCK",
                    independently_verified_custodian="NONE",
                    independently_protected_external_history="NONE",
                    native_admission="BLOCK")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise", action="store_true", required=True)
    p.parse_args(argv)
    try:
        result = native_exercise()
    except (Refused, witness.Refused, ledger.Refused, identity.Refused,
            receipts.Refused, keys.Refused, ssh.Refused, fdgate.Refused,
            OSError, ValueError) as exc:
        result = deny("R6A_REAL_NATIVE_EXTERNAL_CHALLENGE_FAILED",
                      detail=str(exc) if isinstance(exc, (Refused, witness.Refused,
                          ledger.Refused, identity.Refused, receipts.Refused,
                          keys.Refused, ssh.Refused, fdgate.Refused))
                      else type(exc).__name__)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
