#!/usr/bin/env python3
"""NORDOI G0.3 R5X: genuine SSHSIG candidate provenance, without external trust.

The cryptographic verifier checks a real locally signed receipt against exact
R5W/R5V source/history/nonce. Even a valid signature, local OOB-looking pin,
claimed host, valid timebox or nonce ledger never proves independent custody.
Every result is BLOCK; no enrollment, external credential, or production.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as osgate
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as rotate
import g03_r5o_fd_identity_quarantine as fdgate

PROJECT = "eddarsglobal/nordoi"
PARENT = "38fcadb6b659e9774c8677a14a7fd27df328f4b3"
PROTOCOL = "NORDOI-G03-R5X-SIGNED-CUSTODY-PROVENANCE-V1"
OPERATION = "VERIFY_CANDIDATE_SIGNED_RECEIPT_DENY_ONLY"
NAMESPACE = "nordoi-r5x"
MAX_BYTES = 4096
MAX_WINDOW = 900
FIELDS = frozenset((
    "protocol", "project", "parent", "operation", "dossier_sha256", "challenge",
    "history_head", "key_epoch", "guardian_key_sha256",
    "claimed_admin_domain", "claimed_host_attestation_sha256",
    "claimed_owner_oob_root_sha256", "claimed_remote_revocation_floor",
    "claimed_remote_spent_nonce_floor", "issued_at", "expires_at",
))

class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def no_duplicates(pairs):
    d = {}
    for k, v in pairs:
        if k in d:
            raise Refused("DUPLICATE_SIGNED_PROVENANCE_FIELD")
        d[k] = v
    return d

def no_nan(_):
    raise Refused("NONFINITE_PROVENANCE_NUMBER")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("SIGNATURE_RECEIPT_BYTES_MISSING_OR_OVERSIZED")
    try:
        x = json.loads(raw.decode("utf-8"), object_pairs_hook=no_duplicates,
                       parse_constant=no_nan)
    except (ValueError, UnicodeError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("INVALID_UTF8_OR_JSON") from exc
    if type(x) is not dict or x.keys() != FIELDS or canonical(x) != raw:
        raise Refused("PROVENANCE_SCHEMA_OR_CANONICAL_REJECTED")
    if (x["protocol"], x["project"], x["parent"], x["operation"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION
    ):
        raise Refused("UNPINNED_SOURCE_OR_FORBIDDEN_GRANT_OPERATION")
    for k in ("dossier_sha256", "challenge", "history_head",
              "guardian_key_sha256"):
        if not intake.check_pin(x[k]):
            raise Refused("INVALID_REQUIRED_SHA256_" + k)
    for k in ("claimed_host_attestation_sha256", "claimed_owner_oob_root_sha256"):
        if x[k] is not None and not intake.check_pin(x[k]):
            raise Refused("INVALID_CLAIMED_EXTERNAL_PIN_" + k)
    if type(x["claimed_admin_domain"]) is not str or re.fullmatch(
        r"lab-[a-z][a-z0-9-]{2,39}", x["claimed_admin_domain"]
    ) is None:
        raise Refused("UNTRUSTED_CLAIMED_ADMIN_DOMAIN_INVALID")
    for k in ("key_epoch", "claimed_remote_revocation_floor",
              "claimed_remote_spent_nonce_floor", "issued_at", "expires_at"):
        if type(x[k]) is not int or x[k] < 0:
            raise Refused("INVALID_COUNTER_OR_LOCAL_TIME_" + k)
    if x["expires_at"] < x["issued_at"] or (
        x["expires_at"] - x["issued_at"] > MAX_WINDOW
    ):
        raise Refused("EXCESSIVE_OR_BACKWARDS_UNTRUSTED_TIMEBOX")
    return x

def deny(reason, **extra):
    return dict(
        status="BLOCK", admission="NOT_AUTHORIZED",
        native_authority_granted=False, production="NOT_AUTHORIZED",
        signed_receipt_is_external_authority=False,
        external_guardian="NOT_ENROLLED", owner_oob_root="NOT_ENROLLED",
        independent_admin_custody="NOT_PROVEN",
        global_monotonic_revocation="NOT_VERIFIED",
        native_admission_gates_unmet=12, reason=reason, **extra,
    )

def make_candidate(dossier_raw, signer_pin, issue_time, window=300):
    """Only a canonical lab proposal; issuer pin is not a trust anchor."""
    d = intake.parse(dossier_raw)
    if not intake.check_pin(signer_pin):
        raise Refused("CANDIDATE_SIGNER_SHA_INVALID")
    if (type(issue_time) is not int or issue_time < 0 or
        type(window) is not int or not 0 < window <= MAX_WINDOW):
        raise Refused("CANDIDATE_TIMEBOX_INVALID")
    x = dict(protocol=PROTOCOL, project=PROJECT, parent=PARENT,
             operation=OPERATION,
             dossier_sha256=intake.digest(dossier_raw),
             challenge=d["challenge"], history_head=d["history_head"],
             key_epoch=d["key_epoch"], guardian_key_sha256=signer_pin,
             claimed_admin_domain="lab-candidate-guardian",
             claimed_host_attestation_sha256=None,
             claimed_owner_oob_root_sha256=None,
             claimed_remote_revocation_floor=0,
             claimed_remote_spent_nonce_floor=0,
             issued_at=issue_time, expires_at=issue_time + window)
    return parse(canonical(x))

def verify(raw, dossier_raw, owner_raw, candidate_raw, now,
           signature_path=None, allowed_signers=None, public_key=None,
           local_spent_challenges=(), local_revocation_floor=None):
    """Verify a candidate receipt; NEVER promote a local signer to authority.

    Local used-nonce and high-water input is evidence for negative tests only,
    not proof of an externally durable, tamperproof monotonic floor.
    """
    try:
        x = parse(raw)
        if type(now) is not int or now < 0:
            raise Refused("UNTRUSTED_CURRENT_TIME_INVALID")
        d = intake.parse(dossier_raw)
        prior = intake.inspect(dossier_raw, owner_raw, candidate_raw)
        if prior["status"] != "BLOCK" or not prior.get(
            "structural_binding_verified", False
        ):
            raise Refused("PARENT_R5W_EVIDENCE_UNVERIFIED")
        if (x["dossier_sha256"], x["challenge"], x["history_head"],
            x["key_epoch"]) != (
            intake.digest(dossier_raw), d["challenge"],
            d["history_head"], d["key_epoch"]
        ):
            raise Refused("EXACT_DOSSIER_OR_CHALLENGE_HISTORY_MISMATCH")
        if not x["issued_at"] <= now <= x["expires_at"]:
            return deny("UNTRUSTED_CLOCK_TIMEBOX_STALE_OR_FUTURE")
        if (type(local_revocation_floor) is not int and
            local_revocation_floor is not None) or (
            local_revocation_floor is not None and local_revocation_floor < 0
        ):
            raise Refused("LOCAL_REVOCATION_FLOOR_INVALID")
        if local_revocation_floor is not None and (
            x["claimed_remote_revocation_floor"] < local_revocation_floor
        ):
            return deny("LOCAL_REVOCATION_FLOOR_REGRESSION",
                        negative_local_floor_comparison=True)
        if type(local_spent_challenges) not in (tuple, frozenset, set):
            raise Refused("INVALID_UNTRUSTED_LOCAL_SPENT_NONCE_INPUT")
        if any(not intake.check_pin(v) for v in local_spent_challenges):
            raise Refused("INVALID_LOCAL_NONCE_REGISTER")
        if x["challenge"] in local_spent_challenges:
            return deny("LOCAL_NONCE_REPLAY_BLOCKED",
                        global_spent_nonce_proven=False)
        if any(p is None for p in (signature_path, allowed_signers, public_key)):
            return deny("NO_AUTHENTICATED_EXTERNAL_SIGNATURE_OR_ROOT")
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("NATIVE_POSIX_CRYPTO_NOT_AVAILABLE_NO_ASSERTION")
        if x["guardian_key_sha256"] != rotate.public_sha(public_key):
            raise Refused("SIGNED_RECEIPT_KEY_NOT_BOUND_TO_PUBLIC_KEY")
        for p in (signature_path, allowed_signers, public_key):
            pp = Path(p)
            if pp.is_symlink() or not pp.is_file():
                raise Refused("CRYPTO_EVIDENCE_FILE_MISSING_OR_SYMLINK")
        # Genuine OpenSSH SSHSIG verification with source-specific namespace.
        # A local allowlist is a cryptographic test fixture, never OOB trust.
        with fdgate.OpenSnapshot(Path(signature_path)) as snapshot:
            if signed.run_ssh([
                "-Y", "verify", "-f", str(allowed_signers),
                "-I", signed.IDENTITY, "-n", NAMESPACE,
                "-s", str(signature_path)
            ], raw) != 0:
                raise Refused("REAL_ED25519_SSHSIG_INVALID_OR_WRONG_NAMESPACE")
            snapshot.check()
        return deny("REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN",
                    cryptographic_signature_valid=True,
                    signing_key_possession_only=True,
                    local_clock_only=True,
                    locally_declared_root_not_independently_authenticated=True,
                    claimed_host_attestation_not_independent=True)
    except (Refused, intake.Refused, osgate.Refused, rotate.Refused,
            signed.Refused, fdgate.Refused, ValueError, TypeError, OSError) as e:
        return deny("SIGNED_PROVENANCE_EVIDENCE_QUARANTINED",
                    detail=str(e) if isinstance(e, (Refused, intake.Refused,
                        osgate.Refused, rotate.Refused, signed.Refused,
                        fdgate.Refused)) else type(e).__name__)

def sign_local(path, key, namespace=NAMESPACE):
    """Genuine local research SSHSIG, no external operator impersonation."""
    sig = Path(str(path) + ".sig")
    if sig.exists() or signed.run_ssh([
        "-Y", "sign", "-f", str(key), "-n", namespace, str(path)
    ]) != 0 or not sig.is_file():
        raise Refused("REAL_LOCAL_SIGNING_FAILED")
    if os.name != "nt":
        os.chmod(sig, 0o600)
    return sig

def native_exercise():
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return deny("REAL_POSIX_OPENSSH_NOT_AVAILABLE")
    with tempfile.TemporaryDirectory(
        prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-native-"
    ) as name:
        root = Path(name)
        os.chmod(root, 0o700)
        ok, _ = signed.create_same_admin_key(root, "r5x_owner")
        guardian, allowed = signed.create_same_admin_key(root, "r5x_guardian")
        own_pin = rotate.public_sha(root / "r5x_owner.pub")
        guardian_pub = root / "r5x_guardian.pub"
        guardian_pin = rotate.public_sha(guardian_pub)
        a = osgate.child_observe("owner_proxy", "lab-owner-domain",
                                 own_pin, 0, "b" * 64, "c" * 64)
        b = osgate.child_observe("custodian_proxy", "lab-guardian-domain",
                                 guardian_pin, 0, "b" * 64, "c" * 64)
        if a["pid"] == b["pid"] or a["uid"] != b["uid"]:
            raise Refused("SAME_ADMIN_LAB_OS_IDENTITIES_NOT_REPRODUCED")
        ar, br = osgate.canonical(a), osgate.canonical(b)
        d = intake.canonical(intake.candidate(ar, br))
        x = make_candidate(d, guardian_pin, 1000)
        path = root / "r5x_signed_receipt.json"
        raw = canonical(x)
        path.write_bytes(raw)
        os.chmod(path, 0o600)
        sig = sign_local(path, guardian)
        valid = verify(raw, d, ar, br, 1100, sig, allowed, guardian_pub)
        if valid["reason"] != "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN":
            raise Refused("VALID_LOCAL_SIGNATURE_WAS_NOT_DENIED")
        old = verify(raw, d, ar, br, 1100, sig, allowed, guardian_pub,
                     local_spent_challenges=frozenset([x["challenge"]]))
        if old["reason"] != "LOCAL_NONCE_REPLAY_BLOCKED":
            raise Refused("REPLAY_WAS_NOT_REFUSED")
        local_rollback = verify(raw, d, ar, br, 1100, sig, allowed, guardian_pub,
                                local_revocation_floor=1)
        if local_rollback["reason"] != "LOCAL_REVOCATION_FLOOR_REGRESSION":
            raise Refused("LOCAL_FLOOR_REGRESSION_WAS_NOT_REFUSED")
        return deny("REAL_SIGNED_SAME_ADMIN_PROVENANCE_NEGATIVE_CONTROLS_PASS",
                    two_os_processes="DISTINCT_PIDS_SAME_UID_EUID",
                    real_ed25519_ssh_signature="VALID_SAME_ADMIN_KEY_POSSESSION",
                    challenge_bound_to_frozen_r5w="PASS_LOCAL_ONLY",
                    real_signed_local_nonce_replay="BLOCK",
                    real_signed_local_floor_regression="BLOCK",
                    out_of_band_trust_root="NONE",
                    independent_custodian="NOT_ENROLLED",
                    native_admission="BLOCK")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise", action="store_true", required=True)
    p.parse_args(argv)
    try:
        out = native_exercise()
    except (Refused, OSError, ValueError, TypeError, signed.Refused,
            osgate.Refused, rotate.Refused, intake.Refused) as e:
        out = deny("R5X_NATIVE_SIGNED_EXERCISE_FAILED",
                   detail=str(e) if isinstance(e, (Refused, signed.Refused,
                       osgate.Refused, rotate.Refused, intake.Refused))
                       else type(e).__name__)
    print(json.dumps(out, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
