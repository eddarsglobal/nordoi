#!/usr/bin/env python3
"""NORDOI G0.3 R6C: two genuine keys, one administrator, NO independent quorum.

Qualify a real R6B signed candidate response bound to exact R6A/R5Z
sources and additionally signed by distinct local owner/candidate SSHSIG
keys. Two true keys, two OS PIDs, two domain labels or two signatures under
one UID/EUID never establish independently authenticated custodians.
Everything is DENY ONLY. No network, grant, enrollment, or production.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys

import g03_r6b_signed_response_provenance as response
import g03_r6a_oob_witness_challenge as request
import g03_r5z_preserved_witness_rollback as witness
import g03_r5y_signed_nonce_rollback as ledger
import g03_r5x_signed_external_provenance as receipts
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R6C-TWO-KEY-SAME-ADMIN-QUORUM-NO-AUTHORITY-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "7a6c22bf6b092488bae23361497f8645141b70a9"
OPERATION = "CHECK_TWO_REAL_SIGNATURES_DENY_NATIVE_ADMISSION"
OWNER_NS = "nordoi-r6c-owner"
GUARDIAN_NS = "nordoi-r6c-guardian"
MAX_BYTES = 4096
MAX_TTL = 300
FIELDS = frozenset((
    "protocol", "project", "parent", "operation", "response_sha256",
    "request_sha256", "challenge", "witness_sha256", "witness_height",
    "nonce_set_sha256", "revocation_floor", "owner_key_sha256",
    "candidate_key_sha256", "owner_role", "candidate_role",
    "claimed_owner_oob_root_sha256", "claimed_independent_admin_domain",
    "claimed_external_anchor_sha256", "claimed_remote_revocation_floor",
    "claimed_remote_spent_nonce_floor", "claimed_quorum_size",
    "issued_at_local", "expires_at_local",
))

class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(pairs):
    out = {}
    for key, value in pairs:
        if key in out:
            raise Refused("DUPLICATE_DUAL_WITNESS_FIELD")
        out[key] = value
    return out

def finite(_):
    raise Refused("NONFINITE_DUAL_WITNESS_NUMBER")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("DUAL_WITNESS_MISSING_OR_OVERSIZED")
    try:
        obj = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                         parse_constant=finite)
    except (UnicodeError, ValueError) as exc:
        if isinstance(exc, Refused):
            raise
        raise Refused("DUAL_WITNESS_INVALID_UTF8_JSON") from exc
    if type(obj) is not dict or obj.keys() != FIELDS or canonical(obj) != raw:
        raise Refused("DUAL_WITNESS_SCHEMA_OR_CANONICAL_MISMATCH")
    if (obj["protocol"], obj["project"], obj["parent"], obj["operation"],
        obj["owner_role"], obj["candidate_role"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION, "owner_proxy", "custodian_proxy"
    ):
        raise Refused("DUAL_WITNESS_WRONG_SOURCE_OPERATION_OR_ROLE")
    for k in ("response_sha256", "request_sha256", "challenge",
              "witness_sha256", "nonce_set_sha256", "owner_key_sha256",
              "candidate_key_sha256"):
        if not request.pin(obj[k]):
            raise Refused("DUAL_WITNESS_INVALID_PIN_" + k)
    if obj["owner_key_sha256"] == obj["candidate_key_sha256"]:
        raise Refused("DUPLICATE_SIGNER_KEY_NOT_TWO_REAL_KEYS")
    for k in ("claimed_owner_oob_root_sha256", "claimed_external_anchor_sha256"):
        if obj[k] is not None and not request.pin(obj[k]):
            raise Refused("UNTRUSTED_CLAIM_PIN_INVALID_" + k)
    if obj["claimed_independent_admin_domain"] is not None and (
        type(obj["claimed_independent_admin_domain"]) is not str or
        re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}",
                     obj["claimed_independent_admin_domain"]) is None
    ):
        raise Refused("UNTRUSTED_ADMIN_DOMAIN_LABEL_INVALID")
    for k in ("witness_height", "revocation_floor",
              "claimed_remote_revocation_floor", "claimed_remote_spent_nonce_floor",
              "claimed_quorum_size", "issued_at_local", "expires_at_local"):
        if obj[k] is not None and (type(obj[k]) is not int or obj[k] < 0):
            raise Refused("DUAL_WITNESS_INVALID_COUNTER_" + k)
    if type(obj["witness_height"]) is not int or obj["witness_height"] < 1:
        raise Refused("DUAL_WITNESS_NONPOSITIVE_HEIGHT")
    for k in ("revocation_floor", "issued_at_local", "expires_at_local"):
        if type(obj[k]) is not int:
            raise Refused("DUAL_WITNESS_MISSING_REQUIRED_COUNTER_" + k)
    if (obj["expires_at_local"] < obj["issued_at_local"] or
        obj["expires_at_local"] - obj["issued_at_local"] > MAX_TTL):
        raise Refused("DUAL_WITNESS_TIMEBOX_INVALID")
    return obj

def deny(reason, **extra):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                external_guardian="NOT_ENROLLED", owner_oob_root="NOT_ENROLLED",
                independent_admin_custody="NOT_PROVEN",
                independently_protected_global_history="NOT_VERIFIED",
                independent_quorum="NOT_PROVEN",
                native_admission_gates_unmet=12, reason=reason, **extra)

def exact_source(reply_raw, request_raw, witness_raw):
    reply = response.parse(reply_raw)
    req = request.parse(request_raw)
    historic = witness.parse(witness_raw)
    if (reply["request_sha256"], reply["challenge"], reply["witness_sha256"],
        reply["witness_height"], reply["nonce_set_sha256"],
        reply["local_revocation_floor"], reply["candidate_key_sha256"]) != (
        ledger.sha(request_raw), req["challenge"], ledger.sha(witness_raw),
        historic["local_height"], historic["nonce_set_sha256"],
        historic["local_revocation_floor"], req["candidate_signer_key_sha256"]
    ):
        raise Refused("SIGNED_R6B_RESPONSE_DOES_NOT_MATCH_R6A_R5Z")
    if (req["source_witness_sha256"], req["source_witness_height"],
        req["source_nonce_set_sha256"], req["source_local_revocation_floor"],
        req["source_receipt_sha256"]) != (
        ledger.sha(witness_raw), historic["local_height"],
        historic["nonce_set_sha256"], historic["local_revocation_floor"],
        historic["source_receipt_sha256"]
    ):
        raise Refused("R6A_REQUEST_NOT_BOUND_TO_R5Z_HISTORY")
    return reply, req, historic

def proposal(reply_raw, request_raw, witness_raw, owner_pin, now, ttl=180):
    reply, req, historic = exact_source(reply_raw, request_raw, witness_raw)
    if not request.pin(owner_pin) or owner_pin == reply["candidate_key_sha256"]:
        raise Refused("INVALID_OR_SAME_OWNER_AND_GUARDIAN_KEY")
    if (type(now) is not int or now < 0 or type(ttl) is not int or
        not 1 <= ttl <= MAX_TTL):
        raise Refused("INVALID_ISSUED_TIME_OR_TTL")
    obj = dict(
        protocol=PROTOCOL, project=PROJECT, parent=PARENT, operation=OPERATION,
        response_sha256=ledger.sha(reply_raw),
        request_sha256=ledger.sha(request_raw),
        challenge=req["challenge"],
        witness_sha256=ledger.sha(witness_raw),
        witness_height=historic["local_height"],
        nonce_set_sha256=historic["nonce_set_sha256"],
        revocation_floor=historic["local_revocation_floor"],
        owner_key_sha256=owner_pin,
        candidate_key_sha256=reply["candidate_key_sha256"],
        owner_role="owner_proxy", candidate_role="custodian_proxy",
        claimed_owner_oob_root_sha256=None,
        claimed_independent_admin_domain=None,
        claimed_external_anchor_sha256=None,
        claimed_remote_revocation_floor=None,
        claimed_remote_spent_nonce_floor=None,
        claimed_quorum_size=None,
        issued_at_local=now, expires_at_local=now + ttl,
    )
    return parse(canonical(obj))

def inspect(raw, reply_raw, request_raw, witness_raw, now, *,
            owner_signature=None, owner_allowed=None, owner_public=None,
            guardian_signature=None, guardian_allowed=None, guardian_public=None,
            response_signature=None, response_allowed=None, response_public=None,
            local_spent=(), local_revoked=()):
    """Signed 2/2 same-admin candidate checks, NEVER an authority grant."""
    try:
        obj = parse(raw)
        reply, req, historic = exact_source(reply_raw, request_raw, witness_raw)
        if type(now) is not int or now < 0:
            raise Refused("UNTRUSTED_LOCAL_CLOCK_INVALID")
        if (obj["response_sha256"], obj["request_sha256"], obj["challenge"],
            obj["witness_sha256"], obj["witness_height"],
            obj["nonce_set_sha256"], obj["revocation_floor"],
            obj["candidate_key_sha256"]) != (
            ledger.sha(reply_raw), ledger.sha(request_raw), req["challenge"],
            ledger.sha(witness_raw), historic["local_height"],
            historic["nonce_set_sha256"], historic["local_revocation_floor"],
            reply["candidate_key_sha256"]
        ):
            raise Refused("DUAL_SIGNATURE_RESPONSE_SOURCE_OR_NONCE_MISMATCH")
        if not (req["issued_at_local"] <= now <= req["expires_at_local"] and
                reply["issued_at_local"] <= now <= reply["expires_at_local"] and
                obj["issued_at_local"] <= now <= obj["expires_at_local"]):
            return deny("LOCAL_CLOCK_STALE_OR_FUTURE_NOT_EXTERNAL_FRESHNESS")
        for keys_set in (local_spent, local_revoked):
            if type(keys_set) not in (tuple, frozenset, set) or any(
                not request.pin(p) for p in keys_set
            ):
                raise Refused("INVALID_LOCAL_NONCE_OR_REVOKED_KEY_SET")
        if req["challenge"] in local_spent:
            return deny("LOCALLY_USED_CHALLENGE_NOT_GLOBAL_ANTIROLLBACK",
                        local_replay_refused=True)
        if (obj["owner_key_sha256"] in local_revoked or
            obj["candidate_key_sha256"] in local_revoked):
            return deny("LOCALLY_REVOKED_DUAL_SIGNER_REFUSED",
                        global_revocation_proven=False)
        claims = ("claimed_owner_oob_root_sha256",
                  "claimed_independent_admin_domain",
                  "claimed_external_anchor_sha256",
                  "claimed_remote_revocation_floor",
                  "claimed_remote_spent_nonce_floor",
                  "claimed_quorum_size")
        if any(obj[c] is not None for c in claims):
            return deny("SIGNED_SELF_ASSERTED_INDEPENDENCE_NOT_AUTHORITY",
                        external_claims_not_independently_authenticated=True)
        signer_paths = (owner_signature, owner_allowed, owner_public,
                        guardian_signature, guardian_allowed, guardian_public,
                        response_signature, response_allowed, response_public)
        if any(p is None for p in signer_paths):
            return deny("REAL_SIGNATURE_EVIDENCE_MISSING_NO_QUORUM")
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("NATIVE_POSIX_OPENSSH_NOT_AVAILABLE")
        for path in signer_paths:
            f = Path(path)
            if f.is_symlink() or not f.is_file():
                raise Refused("SIGNATURE_EVIDENCE_FILE_MISSING_OR_SYMLINK")
        if (keys.public_sha(owner_public) != obj["owner_key_sha256"] or
            keys.public_sha(guardian_public) != obj["candidate_key_sha256"] or
            keys.public_sha(response_public) != obj["candidate_key_sha256"]):
            raise Refused("OWNER_OR_GUARDIAN_PUBLIC_KEY_HASH_MISMATCH")
        parent = response.assess(
            reply_raw, request_raw, witness_raw, now,
            response_signature, response_allowed, response_public
        )
        if parent["reason"] != "VALID_SSHSIG_LOCAL_SIGNER_NOT_INDEPENDENT_CUSTODIAN":
            raise Refused("R6B_RESPONSE_NOT_VERIFIED_WITH_REAL_SSHSIG")
        for signature, allowlist, namespace in (
            (owner_signature, owner_allowed, OWNER_NS),
            (guardian_signature, guardian_allowed, GUARDIAN_NS),
        ):
            with fdgate.OpenSnapshot(Path(signature)) as snapshot:
                if ssh.run_ssh([
                    "-Y", "verify", "-f", str(allowlist),
                    "-I", ssh.IDENTITY, "-n", namespace,
                    "-s", str(signature)
                ], raw) != 0:
                    raise Refused("GENUINE_DISTINCT_ED25519_SSHSIG_INVALID_" + namespace)
                snapshot.check()
        return deny("TWO_REAL_SSHSIG_KEYS_SAME_ADMIN_NOT_INDEPENDENT_QUORUM",
                    genuine_signatures_verified=3,
                    distinct_local_key_hashes=True,
                    same_administrator_independence_proven=False,
                    exact_r6a_r6b_r5z_source_binding=True,
                    independent_quorum_proven=False)
    except (Refused, response.Refused, request.Refused, witness.Refused,
            ledger.Refused, keys.Refused, fdgate.Refused, OSError, ValueError,
            TypeError) as exc:
        return deny("DUAL_WITNESS_PROVENANCE_QUARANTINED",
                    detail=str(exc) if isinstance(exc, (Refused,
                        response.Refused, request.Refused, witness.Refused,
                        ledger.Refused, keys.Refused, fdgate.Refused))
                    else type(exc).__name__)

def sign_local(path, secret, namespace):
    return receipts.sign_local(path, secret, namespace)

def create_lab():
    """Real R6B local lab; add two independent-KEY, same-admin SSHSIGs."""
    home, base = response.make_lab()
    try:
        root = base["root"]
        candidate = proposal(base["response"], base["request"], base["witness"],
                             base["owner_pin"], 1100)
        raw = canonical(candidate)
        f = root / "r6c_two_key_candidate.json"
        f.write_bytes(raw)
        os.chmod(f, 0o600)
        owner_sig = sign_local(f, base["owner_key"], OWNER_NS)
        # Real OpenSSH refuses to overwrite an existing .sig: give each
        # signer a distinct identical-byte file in a private directory.
        other = root / "r6c_guardian_same_bytes.json"
        other.write_bytes(raw)
        os.chmod(other, 0o600)
        guardian_sig = sign_local(other, base["candidate_key"], GUARDIAN_NS)
        bundle = dict(base, body=raw, owner_signature=owner_sig,
                      guardian_signature=guardian_sig,
                      owner_public=root/"r6b_owner.pub",
                      guardian_public=root/"r6b_candidate.pub",
                      parent_response_sig=base["signature"],
                      owner_allowed=base["owner_allowed"],
                      guardian_allowed=base["allowed"])
        return home, bundle
    except Exception:
        home.cleanup()
        raise

def native_exercise():
    home, lab = create_lab()
    try:
        def valid(**kw):
            args = dict(owner_signature=lab["owner_signature"],
                        owner_allowed=lab["owner_allowed"],
                        owner_public=lab["owner_public"],
                        guardian_signature=lab["guardian_signature"],
                        guardian_allowed=lab["guardian_allowed"],
                        guardian_public=lab["guardian_public"],
                        response_signature=lab["parent_response_sig"],
                        response_allowed=lab["allowed"],
                        response_public=lab["public"])
            args.update(kw)
            return inspect(lab["body"], lab["response"], lab["request"],
                           lab["witness"], 1200, **args)
        result = valid()
        if result["reason"] != "TWO_REAL_SSHSIG_KEYS_SAME_ADMIN_NOT_INDEPENDENT_QUORUM":
            raise Refused("VALID_REAL_LOCAL_TWO_KEY_PROOF_WAS_NOT_DENIED")
        spoofed = dict(parse(lab["body"]),
                       claimed_quorum_size=2,
                       claimed_owner_oob_root_sha256=lab["owner_pin"])
        forged_raw = canonical(spoofed)
        f = lab["root"] / "r6c_two_signature_false_quorum.json"
        f.write_bytes(forged_raw)
        os.chmod(f, 0o600)
        sig = sign_local(f, lab["owner_key"], OWNER_NS)
        if ssh.run_ssh(["-Y", "verify", "-f", str(lab["owner_allowed"]),
                        "-I", ssh.IDENTITY, "-n", OWNER_NS,
                        "-s", str(sig)], forged_raw) != 0:
            raise Refused("FALSE_QUORUM_LOCAL_SIGNATURE_TEST_INVALID")
        fake = inspect(forged_raw, lab["response"], lab["request"],
                       lab["witness"], 1200)
        if fake["reason"] != "SIGNED_SELF_ASSERTED_INDEPENDENCE_NOT_AUTHORITY":
            raise Refused("SIGNED_FAKE_OWNER_ROOT_NOT_QUARANTINED")
        replay = valid(local_spent=frozenset([parse(lab["body"])["challenge"]]))
        if replay["reason"] != "LOCALLY_USED_CHALLENGE_NOT_GLOBAL_ANTIROLLBACK":
            raise Refused("SIGNED_LOCAL_REPLAY_NOT_BLOCKED")
        return deny("REAL_TWO_KEY_SAME_ADMIN_QUORUM_NEGATIVE_CONTROLS_PASS",
                    authentic_owner_ed25519="PASS_LOCAL_ONLY",
                    authentic_candidate_ed25519="PASS_LOCAL_ONLY",
                    authentic_r6b_response_signature="PASS_LOCAL_ONLY",
                    two_different_os_pids="PASS_SAME_UID_EUID",
                    same_admin_false_quorum_claim="BLOCK",
                    local_replayed_random_challenge="BLOCK",
                    externally_authenticated_admin_domains=0,
                    protected_remote_nonce_revocation_histories=0,
                    native_admission="BLOCK")
    finally:
        home.cleanup()

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise", action="store_true", required=True)
    p.parse_args(argv)
    try:
        out = native_exercise()
    except (Refused, response.Refused, request.Refused, witness.Refused,
            ledger.Refused, receipts.Refused, keys.Refused, fdgate.Refused,
            OSError, ValueError, TypeError) as exc:
        out = deny("R6C_REAL_NATIVE_QUORUM_NEGATIVE_EXPERIMENT_FAILED",
                   detail=str(exc) if isinstance(exc, (Refused,
                       response.Refused, request.Refused, witness.Refused,
                       ledger.Refused, receipts.Refused, keys.Refused,
                       fdgate.Refused)) else type(exc).__name__)
    print(json.dumps(out, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
