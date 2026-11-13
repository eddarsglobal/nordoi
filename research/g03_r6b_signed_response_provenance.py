#!/usr/bin/env python3
"""NORDOI G0.3 R6B — exact-challenge signed response provenance, deny only.

A genuine SSHSIG verifies possession of a same-admin laboratory Ed25519 key,
not external custodial independence. R6B binds a candidate response to the
byte-exact R6A random-challenge request and R5Z witness history; it rejects
cross-request replay, stale local clocks, altered payloads and unsupported
external root claims. All results BLOCK, no network/enrollment/native grants.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

import g03_r6a_oob_witness_challenge as challenge
import g03_r5z_preserved_witness_rollback as witness
import g03_r5y_signed_nonce_rollback as ledger
import g03_r5x_signed_external_provenance as receipts
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as identity
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6B-SIGNED-RESPONSE-PROVENANCE-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "b5ce4f793b6d1a5699955998478bb93bb968c711"
OPERATION = "CHECK_SIGNED_EXTERNAL_RESPONSE_DENY_ONLY"
NAMESPACE = "nordoi-r6b"
MAX_BYTES = 4096
MAX_WINDOW = 300
LAB_PREFIX = "nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-r5y-r5z-r6a-r6b-"
FIELDS = frozenset((
    "protocol", "project", "parent", "operation", "request_sha256",
    "challenge", "witness_sha256", "witness_height", "nonce_set_sha256",
    "local_revocation_floor", "candidate_key_sha256",
    "claimed_external_operator_id", "claimed_admin_domain",
    "claimed_owner_oob_root_sha256", "claimed_external_history_sha256",
    "claimed_external_revocation_floor", "claimed_external_spent_nonce_floor",
    "issued_at_local", "expires_at_local",
))

class Refused(ValueError):
    pass

def canonical(d):
    return (json.dumps(d, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(pairs):
    d = {}
    for key, value in pairs:
        if key in d:
            raise Refused("DUPLICATE_RESPONSE_JSON_FIELD")
        d[key] = value
    return d

def finite(_):
    raise Refused("NONFINITE_RESPONSE_FIELD")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("RESPONSE_MISSING_OR_OVERSIZED")
    try:
        d = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=finite)
    except (UnicodeError, ValueError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("RESPONSE_NOT_CANONICAL_UTF8_JSON") from exc
    if type(d) is not dict or d.keys() != FIELDS or canonical(d) != raw:
        raise Refused("RESPONSE_SCHEMA_OR_CANONICAL_MISMATCH")
    if (d["protocol"], d["project"], d["parent"], d["operation"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION
    ):
        raise Refused("RESPONSE_UNPINNED_SOURCE_OR_PRIVILEGED_OPERATION")
    for key in ("request_sha256", "challenge", "witness_sha256",
                "nonce_set_sha256", "candidate_key_sha256"):
        if not challenge.pin(d[key]):
            raise Refused("RESPONSE_REQUIRED_PIN_INVALID_" + key)
    for key in ("claimed_owner_oob_root_sha256", "claimed_external_history_sha256"):
        if d[key] is not None and not challenge.pin(d[key]):
            raise Refused("RESPONSE_FALSE_EXTERNAL_PIN_INVALID_" + key)
    for key in ("claimed_external_operator_id", "claimed_admin_domain"):
        if d[key] is not None and (
            type(d[key]) is not str or
            re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}", d[key]) is None
        ):
            raise Refused("RESPONSE_EXTERNAL_IDENTITY_CLAIM_INVALID_" + key)
    for key in ("witness_height", "local_revocation_floor",
                "claimed_external_revocation_floor",
                "claimed_external_spent_nonce_floor",
                "issued_at_local", "expires_at_local"):
        if d[key] is not None and (type(d[key]) is not int or d[key] < 0):
            raise Refused("RESPONSE_COUNTER_INVALID_" + key)
    if type(d["witness_height"]) is not int or d["witness_height"] < 1:
        raise Refused("RESPONSE_WITNESS_HEIGHT_MISSING")
    for key in ("local_revocation_floor", "issued_at_local", "expires_at_local"):
        if type(d[key]) is not int:
            raise Refused("RESPONSE_REQUIRED_COUNTER_MISSING_" + key)
    if (d["expires_at_local"] < d["issued_at_local"] or
        d["expires_at_local"] - d["issued_at_local"] > MAX_WINDOW):
        raise Refused("RESPONSE_LOCAL_TIMEBOX_INVALID")
    return d

def deny(reason, **additional):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                independent_admin_custody="NOT_PROVEN",
                external_guardian="NOT_ENROLLED",
                owner_authenticated_oob_root="NOT_ENROLLED",
                globally_protected_monotonic_history="NOT_VERIFIED",
                native_admission_gates_unmet=12, reason=reason, **additional)

def make_response(request_raw, now, ttl=180):
    """Unsigned proposal; signing cannot make the authority independent."""
    req = challenge.parse(request_raw)
    if (type(now) is not int or now < 0 or type(ttl) is not int
        or not 1 <= ttl <= MAX_WINDOW):
        raise Refused("RESPONSE_INVALID_ISSUED_TIME_OR_TTL")
    d = dict(
        protocol=PROTOCOL, project=PROJECT, parent=PARENT, operation=OPERATION,
        request_sha256=ledger.sha(request_raw),
        challenge=req["challenge"],
        witness_sha256=req["source_witness_sha256"],
        witness_height=req["source_witness_height"],
        nonce_set_sha256=req["source_nonce_set_sha256"],
        local_revocation_floor=req["source_local_revocation_floor"],
        candidate_key_sha256=req["candidate_signer_key_sha256"],
        claimed_external_operator_id=None,
        claimed_admin_domain=None, claimed_owner_oob_root_sha256=None,
        claimed_external_history_sha256=None,
        claimed_external_revocation_floor=None,
        claimed_external_spent_nonce_floor=None,
        issued_at_local=now, expires_at_local=now + ttl,
    )
    return parse(canonical(d))

def assess(response_raw, request_raw, local_witness_raw, now,
           signature=None, allowed_signers=None, signer_public_key=None,
           local_spent=(), locally_revoked_keys=()):
    """Cryptographic evaluation of candidate provenance, never admission.

    The allowlist and signer key are locally controlled. They cannot be used
    as an owner-authenticated out-of-band root or as proof of a real operator.
    """
    try:
        response = parse(response_raw)
        req = challenge.parse(request_raw)
        w = witness.parse(local_witness_raw)
        if type(now) is not int or now < 0:
            raise Refused("CURRENT_LOCAL_CLOCK_INVALID")
        if (req["source_witness_sha256"], req["source_witness_height"],
            req["source_nonce_set_sha256"], req["source_local_revocation_floor"],
            req["source_receipt_sha256"]) != (
            ledger.sha(local_witness_raw), w["local_height"],
            w["nonce_set_sha256"], w["local_revocation_floor"],
            w["source_receipt_sha256"]
        ):
            raise Refused("R6A_REQUEST_NOT_BOUND_TO_ACTUAL_R5Z_WITNESS")
        if (response["request_sha256"], response["challenge"],
            response["witness_sha256"], response["witness_height"],
            response["nonce_set_sha256"], response["local_revocation_floor"],
            response["candidate_key_sha256"]) != (
            ledger.sha(request_raw), req["challenge"],
            req["source_witness_sha256"], req["source_witness_height"],
            req["source_nonce_set_sha256"], req["source_local_revocation_floor"],
            req["candidate_signer_key_sha256"]
        ):
            raise Refused("RESPONSE_WRONG_REQUEST_NONCE_SOURCE_OR_SIGNER")
        if not (req["issued_at_local"] <= now <= req["expires_at_local"]
                and response["issued_at_local"] <= now <= response["expires_at_local"]):
            return deny("LOCAL_TIMEBOX_STALE_OR_FUTURE_NOT_EXTERNAL_FRESHNESS")
        for keyset in (local_spent, locally_revoked_keys):
            if type(keyset) not in (tuple, frozenset, set) or any(
                not challenge.pin(v) for v in keyset
            ):
                raise Refused("INVALID_LOCAL_SPENT_OR_REVOKED_SET")
        if req["challenge"] in local_spent:
            return deny("LOCALLY_REPLAYED_CHALLENGE_REFUSED",
                        independent_nonce_floor=False)
        if response["candidate_key_sha256"] in locally_revoked_keys:
            return deny("LOCALLY_REVOKED_CANDIDATE_REFUSED",
                        independent_revocation_floor=False)
        claimed = (
            "claimed_external_operator_id", "claimed_admin_domain",
            "claimed_owner_oob_root_sha256",
            "claimed_external_history_sha256",
            "claimed_external_revocation_floor",
            "claimed_external_spent_nonce_floor",
        )
        if any(response[k] is not None for k in claimed):
            return deny("SELF_DECLARED_SIGNED_EXTERNAL_PROVENANCE_UNTRUSTED",
                        claims_not_independently_authenticated=True)
        if any(p is None for p in (signature, allowed_signers, signer_public_key)):
            return deny("MISSING_VERIFIABLE_LOCAL_SIGNER_NOT_EXTERNAL_AUTHORITY")
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("GENUINE_OPENSSH_OS_CRYPTO_UNAVAILABLE")
        for p in (signature, allowed_signers, signer_public_key):
            f = Path(p)
            if f.is_symlink() or not f.is_file():
                raise Refused("NONREGULAR_OR_SYMLINK_SIGNER_PROOF")
        if keys.public_sha(signer_public_key) != response["candidate_key_sha256"]:
            raise Refused("CANDIDATE_SIGNER_PIN_DOES_NOT_MATCH_PUBLIC_KEY")
        with fdgate.OpenSnapshot(Path(signature)) as frozen:
            if ssh.run_ssh([
                "-Y", "verify", "-f", str(allowed_signers), "-I", ssh.IDENTITY,
                "-n", NAMESPACE, "-s", str(signature)
            ], response_raw) != 0:
                raise Refused("REAL_ED25519_SSHSIG_INVALID")
            frozen.check()
        return deny("VALID_SSHSIG_LOCAL_SIGNER_NOT_INDEPENDENT_CUSTODIAN",
                    cryptographic_signature_valid=True,
                    exact_r6a_request_bound=True,
                    signer_independence_proven=False,
                    remote_monotonic_state_proven=False)
    except (Refused, challenge.Refused, witness.Refused, ledger.Refused,
            keys.Refused, fdgate.Refused, ValueError, TypeError, OSError) as exc:
        return deny("SIGNED_RESPONSE_PROVENANCE_QUARANTINED",
                    detail=str(exc) if isinstance(exc, (Refused,
                        challenge.Refused, witness.Refused, ledger.Refused,
                        keys.Refused, fdgate.Refused)) else type(exc).__name__)

def sign_local(path, key, namespace=NAMESPACE):
    return receipts.sign_local(path, key, namespace)

def make_lab():
    """Real same-admin R5V->R6A chain; only disposable private local files."""
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        raise Refused("POSIX_OPENSSH_REQUIRED_FOR_GENUINE_NATIVE_EXPERIMENT")
    home = tempfile.TemporaryDirectory(prefix=LAB_PREFIX)
    try:
        root = Path(home.name)
        os.chmod(root, 0o700)
        owner_key, owner_allowed = ssh.create_same_admin_key(root, "r6b_owner")
        candidate_key, candidate_allowed = ssh.create_same_admin_key(
            root, "r6b_candidate")
        op = keys.public_sha(root / "r6b_owner.pub")
        cp = keys.public_sha(root / "r6b_candidate.pub")
        if op == cp:
            raise Refused("IDENTICAL_LOCAL_KEYPAIRS")
        a = identity.child_observe("owner_proxy", "lab-owner-domain",
                                   op, 0, "b"*64, "c"*64)
        b = identity.child_observe("custodian_proxy", "lab-candidate-domain",
                                   cp, 0, "b"*64, "c"*64)
        if a["pid"] == b["pid"] or a["uid"] != b["uid"] or a["euid"] != b["euid"]:
            raise Refused("REAL_SEPARATE_PROCESSES_SAME_ADMIN_NOT_OBSERVED")
        ar, br = identity.canonical(a), identity.canonical(b)
        d = intake.canonical(intake.candidate(ar, br))
        receipt = receipts.canonical(receipts.make_candidate(d, cp, 1000))
        rp = root / "r6b_source_receipt.json"
        rp.write_bytes(receipt)
        os.chmod(rp, 0o600)
        rp_sig = receipts.sign_local(rp, candidate_key)
        verified = receipts.verify(receipt, d, ar, br, 1100, rp_sig,
                                   candidate_allowed, root / "r6b_candidate.pub")
        if verified["reason"] != "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN":
            raise Refused("R5X_SIGNED_RECEIPT_NOT_GENUINE")
        ledger.bootstrap(root)
        if ledger.commit_local_denial(root, receipt, 1100, 1)["reason"] != (
            "LOCAL_ONLY_SPENT_NONCE_COMMITTED"
        ):
            raise Refused("R5Y_LOCAL_HISTORY_COMMIT_FAILED")
        wraw = witness.canonical(witness.candidate(root, receipt, 1100))
        wp = witness.save_witness(root, wraw, witness.WITNESS)
        ws = witness.sign_local(wp, candidate_key)
        if witness.verify_local(wraw, ws, candidate_allowed)["reason"] != (
            "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL"
        ):
            raise Refused("R5Z_GENUINE_WITNESS_NOT_VERIFIED")
        req = challenge.canonical(challenge.create_request(wraw, cp, 1100))
        qpath = root / "r6b_request.json"
        qpath.write_bytes(req)
        os.chmod(qpath, 0o600)
        qsig = challenge.sign_local(qpath, candidate_key)
        qout = challenge.assess(req, wraw, 1200, qsig, candidate_allowed,
                                root / "r6b_candidate.pub")
        if qout["reason"] != "REAL_SSHSIG_VALID_LOCAL_ONLY_NO_EXTERNAL_CUSTODY":
            raise Refused("R6A_REAL_CHALLENGE_NOT_VERIFIED")
        response = canonical(make_response(req, 1100))
        path = root / "r6b_candidate_response.json"
        path.write_bytes(response)
        os.chmod(path, 0o600)
        sig = sign_local(path, candidate_key)
        return home, dict(
            root=root, owner_key=owner_key, owner_allowed=owner_allowed,
            candidate_key=candidate_key, allowed=candidate_allowed,
            public=root/"r6b_candidate.pub", request=req, witness=wraw,
            response=response, signature=sig, owner_pin=op,
            candidate_pin=cp, owner_pid=a["pid"], candidate_pid=b["pid"],
            owner_uid=a["uid"], candidate_uid=b["uid"])

    except Exception:
        home.cleanup()
        raise

def native_exercise():
    home, lab = make_lab()
    try:
        args = (lab["response"], lab["request"], lab["witness"], 1200,
                lab["signature"], lab["allowed"], lab["public"])
        valid = assess(*args)
        if valid["reason"] != "VALID_SSHSIG_LOCAL_SIGNER_NOT_INDEPENDENT_CUSTODIAN":
            raise Refused("VALID_LOCAL_SSHSIG_INCORRECTLY_QUALIFIED")
        modified = dict(parse(lab["response"]),
                        claimed_external_operator_id="lab-claimed-guardian")
        bogus = assess(canonical(modified), lab["request"], lab["witness"], 1200)
        if bogus["reason"] != "SELF_DECLARED_SIGNED_EXTERNAL_PROVENANCE_UNTRUSTED":
            raise Refused("BOGUS_EXTERNAL_OPERATOR_CLAIM_NOT_REFUSED")
        prior = assess(*args, local_spent=frozenset([parse(lab["response"])["challenge"]]))
        if prior["reason"] != "LOCALLY_REPLAYED_CHALLENGE_REFUSED":
            raise Refused("SIGNED_RESPONSE_LOCAL_REPLAY_NOT_BLOCKED")
        return deny("REAL_SIGNED_R6A_RESPONSE_PROVENANCE_NEGATIVE_CONTROLS_PASS",
                    genuine_os_subprocesses="DISTINCT_PIDS_SAME_UID_EUID",
                    same_admin_valid_ed25519="PASS_KEY_POSSESSION_ONLY",
                    exact_r6a_challenge_response="PASS_LOCAL_BINDING_ONLY",
                    fake_external_operator="DENIED",
                    signed_nonce_reuse="DENIED_LOCAL_ONLY",
                    real_independent_signer="NOT_ENROLLED",
                    external_monotonic_history="NOT_VERIFIED",
                    native_admission="BLOCK")
    finally:
        home.cleanup()

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise", required=True, action="store_true")
    parser.parse_args(argv)
    try:
        result = native_exercise()
    except (Refused, challenge.Refused, witness.Refused, ledger.Refused,
            receipts.Refused, identity.Refused, keys.Refused, ssh.Refused,
            fdgate.Refused, ValueError, OSError) as exc:
        result = deny("R6B_NATIVE_RESEARCH_EXERCISE_FAILED",
                      detail=str(exc) if isinstance(exc, (Refused,
                          challenge.Refused, witness.Refused, ledger.Refused,
                          receipts.Refused, identity.Refused, keys.Refused,
                          ssh.Refused, fdgate.Refused)) else type(exc).__name__)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
