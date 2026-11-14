#!/usr/bin/env python3
"""NORDOI G0.3 R6D: genuine same-admin OS identity vs independent provenance.

This offline DENY-ONLY pre-enrollment lab verifies real R6B/R6C signatures
and two additional domain-separated SSHSIG signatures on an exact canonical
administrative-domain evidence envelope. Different keys/PIDs/domain labels,
or a locally signed assertion about an external operator/OOB root, cannot
authenticate a distinct administrator. No network, enrollment, grants,
credential exports or production modifications.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

import g03_r6c_two_key_quorum_negative as parent
import g03_r6b_signed_response_provenance as r6b
import g03_r6a_oob_witness_challenge as r6a
import g03_r5z_preserved_witness_rollback as r5z
import g03_r5y_signed_nonce_rollback as r5y
import g03_r5x_signed_external_provenance as receipts
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6D-ADMIN-DOMAIN-PROVENANCE-INTAKE-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "87bcd2bf968533e502e68dc7472b76c1d80ec3c8"
OPERATION = "CHECK_REAL_LOCAL_OS_ADMIN_PROVENANCE_DENY_ONLY"
OWNER_NS = "nordoi-r6d-owner"
GUARDIAN_NS = "nordoi-r6d-candidate"
MAX_BYTES = 4096
MAX_TTL = 300
FIELDS = frozenset((
    "protocol", "project", "parent", "operation",
    "r6c_envelope_sha256", "r6b_response_sha256", "r6a_request_sha256",
    "r5z_witness_sha256", "challenge", "owner_key_sha256",
    "candidate_key_sha256", "owner_pid", "candidate_pid", "owner_uid",
    "candidate_uid", "owner_role", "candidate_role",
    "claimed_owner_domain", "claimed_candidate_domain",
    "claimed_external_operator_id", "claimed_oob_owner_root_sha256",
    "claimed_external_admin_attestation_sha256",
    "claimed_external_history_sha256", "claimed_global_nonce_floor",
    "claimed_global_revocation_floor", "claimed_independent_quorum",
    "issued_at_local", "expires_at_local",
))
class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(pairs):
    result = {}
    for k, v in pairs:
        if k in result:
            raise Refused("DUPLICATE_ADMIN_PROVENANCE_FIELD")
        result[k] = v
    return result

def finite(_):
    raise Refused("NONFINITE_ADMIN_PROVENANCE_NUMBER")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("ADMIN_PROVENANCE_MISSING_OR_OVERSIZED")
    try:
        x = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=finite)
    except (UnicodeError, ValueError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("ADMIN_PROVENANCE_NOT_CANONICAL_JSON") from exc
    if type(x) is not dict or x.keys() != FIELDS or canonical(x) != raw:
        raise Refused("ADMIN_PROVENANCE_FIELDS_OR_CANONICAL_MISMATCH")
    if (x["protocol"], x["project"], x["parent"], x["operation"],
        x["owner_role"], x["candidate_role"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION, "owner_proxy", "custodian_proxy"
    ):
        raise Refused("ADMIN_PROVENANCE_WRONG_SOURCE_OR_AUTHORIZATION")
    for k in ("r6c_envelope_sha256", "r6b_response_sha256",
              "r6a_request_sha256", "r5z_witness_sha256", "challenge",
              "owner_key_sha256", "candidate_key_sha256"):
        if not r6a.pin(x[k]):
            raise Refused("ADMIN_PROVENANCE_INVALID_PIN_" + k)
    if x["owner_key_sha256"] == x["candidate_key_sha256"]:
        raise Refused("OWNER_AND_CANDIDATE_KEY_NOT_DISTINCT")
    for k in ("claimed_oob_owner_root_sha256",
              "claimed_external_admin_attestation_sha256",
              "claimed_external_history_sha256"):
        if x[k] is not None and not r6a.pin(x[k]):
            raise Refused("UNVERIFIED_EXTERNAL_PIN_INVALID_" + k)
    for k in ("claimed_owner_domain", "claimed_candidate_domain",
              "claimed_external_operator_id"):
        if x[k] is not None and (
            type(x[k]) is not str or
            re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}", x[k]) is None
        ):
            raise Refused("SELF_ASSERTED_DOMAIN_LABEL_INVALID_" + k)
    for k in ("owner_pid", "candidate_pid", "owner_uid", "candidate_uid",
              "claimed_global_nonce_floor", "claimed_global_revocation_floor",
              "issued_at_local", "expires_at_local"):
        if x[k] is not None and (type(x[k]) is not int or x[k] < 0):
            raise Refused("ADMIN_PROVENANCE_COUNTER_INVALID_" + k)
    for k in ("owner_pid", "candidate_pid", "owner_uid", "candidate_uid",
              "issued_at_local", "expires_at_local"):
        if type(x[k]) is not int:
            raise Refused("ADMIN_PROVENANCE_REQUIRED_COUNTER_MISSING_" + k)
    if not x["owner_pid"] or not x["candidate_pid"] or (
        x["owner_pid"] == x["candidate_pid"]
    ):
        raise Refused("ADMIN_PROVENANCE_NO_TWO_OBSERVED_PIDS")
    if x["claimed_independent_quorum"] is not None and (
        type(x["claimed_independent_quorum"]) is not int or
        x["claimed_independent_quorum"] < 0
    ):
        raise Refused("INVALID_SELF_ASSERTED_QUORUM")
    if (x["expires_at_local"] < x["issued_at_local"] or
        x["expires_at_local"] - x["issued_at_local"] > MAX_TTL):
        raise Refused("ADMIN_PROVENANCE_LOCAL_TIMEBOX_INVALID")
    return x

def deny(reason, **other):
    return dict(
        status="BLOCK", admission="NOT_AUTHORIZED",
        native_authority_granted=False, production="NOT_AUTHORIZED",
        owner_oob_root="NOT_ENROLLED", external_guardian="NOT_ENROLLED",
        independent_admin_custody="NOT_PROVEN",
        remote_monotonic_history="NOT_VERIFIED",
        independent_quorum="NOT_PROVEN",
        native_admission_gates_unmet=12, reason=reason, **other,
    )

def source(r6c_raw, r6b_raw, r6a_raw, r5z_raw):
    old = parent.parse(r6c_raw)
    reply, req, hist = parent.exact_source(r6b_raw, r6a_raw, r5z_raw)
    if (old["response_sha256"], old["request_sha256"], old["challenge"],
        old["witness_sha256"], old["witness_height"],
        old["nonce_set_sha256"], old["revocation_floor"],
        old["candidate_key_sha256"]) != (
        r5y.sha(r6b_raw), r5y.sha(r6a_raw), req["challenge"],
        r5y.sha(r5z_raw), hist["local_height"],
        hist["nonce_set_sha256"], hist["local_revocation_floor"],
        reply["candidate_key_sha256"]
    ):
        raise Refused("EXACT_R6C_TO_R6B_R6A_R5Z_SOURCE_MISMATCH")
    return old, reply, req

def make(r6c_raw, r6b_raw, r6a_raw, r5z_raw, owner_pid, candidate_pid,
         owner_uid, candidate_uid, now, ttl=180):
    old, _, _ = source(r6c_raw, r6b_raw, r6a_raw, r5z_raw)
    for v in (owner_pid, candidate_pid, owner_uid, candidate_uid, now, ttl):
        if type(v) is not int or v < 0:
            raise Refused("INVALID_UNAUTHENTICATED_OS_INT")
    if (owner_pid == 0 or candidate_pid == 0 or owner_pid == candidate_pid or
        not 1 <= ttl <= MAX_TTL):
        raise Refused("INVALID_OS_PAIR_OR_TTL")
    x = dict(
        protocol=PROTOCOL, project=PROJECT, parent=PARENT,
        operation=OPERATION, r6c_envelope_sha256=r5y.sha(r6c_raw),
        r6b_response_sha256=r5y.sha(r6b_raw),
        r6a_request_sha256=r5y.sha(r6a_raw),
        r5z_witness_sha256=r5y.sha(r5z_raw),
        challenge=old["challenge"], owner_key_sha256=old["owner_key_sha256"],
        candidate_key_sha256=old["candidate_key_sha256"],
        owner_pid=owner_pid, candidate_pid=candidate_pid,
        owner_uid=owner_uid, candidate_uid=candidate_uid,
        owner_role="owner_proxy", candidate_role="custodian_proxy",
        claimed_owner_domain="lab-owner-domain",
        claimed_candidate_domain="lab-candidate-domain",
        claimed_external_operator_id=None,
        claimed_oob_owner_root_sha256=None,
        claimed_external_admin_attestation_sha256=None,
        claimed_external_history_sha256=None,
        claimed_global_nonce_floor=None, claimed_global_revocation_floor=None,
        claimed_independent_quorum=None,
        issued_at_local=now, expires_at_local=now+ttl,
    )
    return parse(canonical(x))

def inspect(raw, r6c_raw, r6b_raw, r6a_raw, r5z_raw, now, *,
            original_owner_signature=None, original_guardian_signature=None,
            response_signature=None, r6d_owner_signature=None,
            r6d_candidate_signature=None, owner_allowed=None,
            candidate_allowed=None, owner_public=None, candidate_public=None,
            local_spent=(), local_revoked=()):
    try:
        x = parse(raw)
        older, reply, req = source(r6c_raw, r6b_raw, r6a_raw, r5z_raw)
        if type(now) is not int or now < 0:
            raise Refused("BAD_LOCAL_UNTRUSTED_CLOCK")
        if (x["r6c_envelope_sha256"], x["r6b_response_sha256"],
            x["r6a_request_sha256"], x["r5z_witness_sha256"], x["challenge"],
            x["owner_key_sha256"], x["candidate_key_sha256"]) != (
            r5y.sha(r6c_raw), r5y.sha(r6b_raw), r5y.sha(r6a_raw),
            r5y.sha(r5z_raw), older["challenge"],
            older["owner_key_sha256"], older["candidate_key_sha256"]
        ):
            raise Refused("ADMIN_PROVENANCE_SOURCE_HASH_OR_KEY_MISMATCH")
        if not (x["issued_at_local"] <= now <= x["expires_at_local"] and
                older["issued_at_local"] <= now <= older["expires_at_local"] and
                reply["issued_at_local"] <= now <= reply["expires_at_local"] and
                req["issued_at_local"] <= now <= req["expires_at_local"]):
            return deny("UNTRUSTED_LOCAL_CLOCK_EXPIRED_OR_FUTURE")
        for coll in (local_spent, local_revoked):
            if type(coll) not in (tuple, frozenset, set) or any(
                not r6a.pin(y) for y in coll
            ):
                raise Refused("MALFORMED_LOCAL_REPLAY_OR_REVOCATION_STATE")
        if x["challenge"] in local_spent:
            return deny("LOCAL_SPENT_NONCE_REJECTED_NO_GLOBAL_FLOOR")
        if (x["owner_key_sha256"] in local_revoked or
            x["candidate_key_sha256"] in local_revoked):
            return deny("LOCAL_REVOKED_KEY_REJECTED_NO_EXTERNAL_FLOOR")
        claims = ("claimed_external_operator_id",
                  "claimed_oob_owner_root_sha256",
                  "claimed_external_admin_attestation_sha256",
                  "claimed_external_history_sha256", "claimed_global_nonce_floor",
                  "claimed_global_revocation_floor", "claimed_independent_quorum")
        if any(x[f] is not None for f in claims):
            return deny("SAME_ADMIN_SIGNED_EXTERNAL_CLAIMS_NOT_AUTHENTICATED",
                        signed_claim_is_not_independence=True)
        if x["owner_uid"] != x["candidate_uid"]:
            return deny("DIFFERENT_CLAIMED_UID_NOT_EXTERNAL_ADMIN_ATTESTATION")
        if x["claimed_owner_domain"] == x["claimed_candidate_domain"]:
            return deny("SAME_DOMAIN_LABEL_NOT_SEPARATE_CUSTODY")
        proofs = (original_owner_signature, original_guardian_signature,
                  response_signature, r6d_owner_signature,
                  r6d_candidate_signature, owner_allowed,
                  candidate_allowed, owner_public, candidate_public)
        if any(p is None for p in proofs):
            return deny("MISSING_INDEPENDENTLY_VERIFIABLE_PROVENANCE")
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("REAL_POSIX_OPENSSH_REQUIRED_FOR_SIGNATURE_VERIFICATION")
        for p in proofs:
            f = Path(p)
            if f.is_symlink() or not f.is_file():
                raise Refused("MISSING_OR_SYMLINK_KEY_PROOF")
        if (keys.public_sha(owner_public) != x["owner_key_sha256"] or
            keys.public_sha(candidate_public) != x["candidate_key_sha256"]):
            raise Refused("REAL_OWNER_OR_GUARDIAN_PUBLIC_KEY_PIN_MISMATCH")
        prior = parent.inspect(
            r6c_raw, r6b_raw, r6a_raw, r5z_raw, now,
            owner_signature=original_owner_signature,
            owner_allowed=owner_allowed, owner_public=owner_public,
            guardian_signature=original_guardian_signature,
            guardian_allowed=candidate_allowed, guardian_public=candidate_public,
            response_signature=response_signature,
            response_allowed=candidate_allowed, response_public=candidate_public,
        )
        if prior["reason"] != "TWO_REAL_SSHSIG_KEYS_SAME_ADMIN_NOT_INDEPENDENT_QUORUM":
            raise Refused("SIGNED_R6C_PARENT_NOT_CRYPTOGRAPHICALLY_QUALIFIED")
        for sig, allowed, namespace in (
            (r6d_owner_signature, owner_allowed, OWNER_NS),
            (r6d_candidate_signature, candidate_allowed, GUARDIAN_NS),
        ):
            with fdgate.OpenSnapshot(Path(sig)) as frozen:
                if ssh.run_ssh([
                    "-Y", "verify", "-f", str(allowed), "-I", ssh.IDENTITY,
                    "-n", namespace, "-s", str(sig)
                ], raw) != 0:
                    raise Refused("REAL_ADMIN_DOMAIN_SSHSIG_INVALID_" + namespace)
                frozen.check()
        return deny("TRUE_LOCAL_OS_PAIR_AND_FIVE_SSHSIGS_NO_EXTERNAL_DOMAIN",
                    authenticated_local_signatures=5,
                    distinct_pids=True,
                    same_owner_uid=True,
                    local_domain_labels_are_assertions=True,
                    real_external_operator_authenticated=False,
                    independent_quorum_proven=False)
    except (Refused, parent.Refused, r6b.Refused, r6a.Refused,
            r5z.Refused, r5y.Refused, keys.Refused, fdgate.Refused,
            ValueError, OSError, TypeError) as exc:
        return deny("ADMIN_DOMAIN_PROVENANCE_QUARANTINED",
                    detail=str(exc) if isinstance(exc, (Refused, parent.Refused,
                        r6b.Refused, r6a.Refused, r5z.Refused, r5y.Refused,
                        keys.Refused, fdgate.Refused)) else type(exc).__name__)

def create_lab():
    """Use actual R6C same-admin key and OS-process observations."""
    home, earlier = parent.create_lab()
    try:
        x = make(earlier["body"], earlier["response"],
                 earlier["request"], earlier["witness"],
                 earlier["owner_pid"], earlier["candidate_pid"],
                 earlier["owner_uid"], earlier["candidate_uid"], 1100)
        raw = canonical(x)
        root = earlier["root"]
        a = root / "r6d_owner_domain.json"
        b = root / "r6d_candidate_domain.json"
        for p in (a,b):
            p.write_bytes(raw)
            os.chmod(p, 0o600)
        owner_sig = receipts.sign_local(a, earlier["owner_key"], OWNER_NS)
        candidate_sig = receipts.sign_local(b, earlier["candidate_key"], GUARDIAN_NS)
        return home, dict(earlier, r6d_body=raw,
                          r6d_owner_signature=owner_sig,
                          r6d_candidate_signature=candidate_sig)
    except Exception:
        home.cleanup()
        raise

def native_exercise():
    home, data = create_lab()
    try:
        def checked(raw, **extra):
            args = dict(
                original_owner_signature=data["owner_signature"],
                original_guardian_signature=data["guardian_signature"],
                response_signature=data["parent_response_sig"],
                r6d_owner_signature=data["r6d_owner_signature"],
                r6d_candidate_signature=data["r6d_candidate_signature"],
                owner_allowed=data["owner_allowed"],
                candidate_allowed=data["guardian_allowed"],
                owner_public=data["owner_public"],
                candidate_public=data["guardian_public"],
            )
            args.update(extra)
            return inspect(raw, data["body"], data["response"],
                           data["request"], data["witness"], 1200, **args)
        result = checked(data["r6d_body"])
        if result["reason"] != "TRUE_LOCAL_OS_PAIR_AND_FIVE_SSHSIGS_NO_EXTERNAL_DOMAIN":
            raise Refused("ACTUAL_FIVE_SIGNED_LOCAL_EVIDENCE_NOT_DENIED")
        fake = dict(parse(data["r6d_body"]),
                    claimed_external_operator_id="lab-fake-independent",
                    claimed_oob_owner_root_sha256=data["owner_pin"])
        forged = canonical(fake)
        p = data["root"] / "r6d_signed_false_external_identity.json"
        p.write_bytes(forged)
        os.chmod(p, 0o600)
        sig = receipts.sign_local(p, data["owner_key"], OWNER_NS)
        if ssh.run_ssh([
            "-Y", "verify", "-f", str(data["owner_allowed"]),
            "-I", ssh.IDENTITY, "-n", OWNER_NS, "-s", str(sig)
        ], forged) != 0:
            raise Refused("FALSE_INDEPENDENCE_REAL_SIGNATURE_NOT_VALID")
        if checked(forged)["reason"] != (
            "SAME_ADMIN_SIGNED_EXTERNAL_CLAIMS_NOT_AUTHENTICATED"
        ):
            raise Refused("SIGNED_FAKE_EXTERNAL_ADMIN_CLAIM_NOT_REJECTED")
        spent = checked(data["r6d_body"],
                        local_spent=frozenset([parse(data["r6d_body"])["challenge"]]))
        if spent["reason"] != "LOCAL_SPENT_NONCE_REJECTED_NO_GLOBAL_FLOOR":
            raise Refused("LOCAL_REPLAY_NEGATIVE_CONTROL_FAILED")
        return deny("R6D_REAL_ADMIN_DOMAIN_NEGATIVE_PROVENANCE_QUALIFIED",
                    two_observed_os_processes="DIFFERENT_PIDS_SAME_UID",
                    five_real_local_ssh_signatures="PASS_CRYPTO_ONLY",
                    signed_fake_external_operator="BLOCK",
                    locally_spent_challenge="BLOCK",
                    independently_authenticated_admin_domains=0,
                    independently_controlled_custodian_keys=0,
                    global_monotonic_witness="NOT_VERIFIED",
                    native_admission="BLOCK")
    finally:
        home.cleanup()

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise", action="store_true", required=True)
    parser.parse_args(argv)
    try:
        result = native_exercise()
    except (Refused, parent.Refused, r6b.Refused, r6a.Refused,
            r5z.Refused, r5y.Refused, receipts.Refused, keys.Refused,
            fdgate.Refused, ValueError, OSError) as exc:
        result = deny("R6D_NATIVE_ADMIN_DOMAIN_EXPERIMENT_FAILED",
                      detail=str(exc) if isinstance(exc, (Refused,
                          parent.Refused, r6b.Refused, r6a.Refused,
                          r5z.Refused, r5y.Refused, receipts.Refused,
                          keys.Refused, fdgate.Refused)) else type(exc).__name__)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
