#!/usr/bin/env python3
"""NORDOI G0.3 R5W: independent administrative custody *intake* preflight.

External authority cannot be created by two local signatures, a claimed
trust-root hash, a third-party label, distinct PIDs or untrusted certificate
files. This parser binds candidate reports to the exact R5V history, while
every outcome remains BLOCK. No enrollment, secret import, privileges or grant.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile

import g03_r5v_admin_separation_evidence as r5v
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as rotate

PROTOCOL = "NORDOI-G03-R5W-EXTERNAL-ADMIN-INTAKE-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "5f39b6f3553f3e3c0578c978f8b5da7a22aaeba3"
OPERATION = "QUALIFY_EXTERNAL_CUSTODY_EVIDENCE_DENY_ONLY"
MAX_BYTES = 4608
EXTERNAL = ("owner_oob_root_pin", "guardian_oob_key_pin",
            "independent_os_attestation_sha256", "external_history_anchor_sha256",
            "quorum_attestation_sha256")
FLOORS = ("external_key_epoch_floor", "external_revocation_floor",
          "external_spent_nonce_floor")
FIELDS = frozenset(("protocol", "project", "parent", "operation", "challenge",
                    "history_head", "key_epoch", "owner_report_sha256",
                    "custodian_report_sha256", "owner_key_sha256",
                    "custodian_key_sha256") + EXTERNAL + FLOORS)
class Refused(ValueError):
    pass

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def canonical(record):
    return (json.dumps(record, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique_pairs(items):
    out = {}
    for k, v in items:
        if k in out:
            raise Refused("DUPLICATE_EXTERNAL_INTAKE_FIELD")
        out[k] = v
    return out

def nonfinite(_):
    raise Refused("NONFINITE_EXTERNAL_INTAKE")

def check_pin(x):
    return type(x) is str and re.fullmatch(r"[a-f0-9]{64}", x) is not None

def block(reason, **other):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                independent_administrative_custody="NOT_PROVEN",
                external_guardian="NOT_ENROLLED",
                owner_authenticated_oob_root="NOT_ENROLLED",
                global_monotonic_antirollback="NOT_VERIFIED",
                native_admission_gates_unmet=12, reason=reason, **other)

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= MAX_BYTES:
        raise Refused("MISSING_OR_OVERSIZED_INTAKE")
    try:
        d = json.loads(raw.decode("utf-8"), object_pairs_hook=unique_pairs,
                       parse_constant=nonfinite)
    except (UnicodeError, json.JSONDecodeError) as ex:
        raise Refused("INTAKE_NOT_VALID_UTF8_JSON") from ex
    if type(d) is not dict or d.keys() != FIELDS or canonical(d) != raw:
        raise Refused("INTAKE_SCHEMA_OR_CANONICAL_MISMATCH")
    if (d["protocol"], d["project"], d["parent"], d["operation"]) != (
        PROTOCOL, PROJECT, PARENT, OPERATION
    ):
        raise Refused("UNPINNED_OR_FORBIDDEN_OPERATION")
    for key in ("challenge", "history_head", "owner_report_sha256",
                "custodian_report_sha256", "owner_key_sha256",
                "custodian_key_sha256"):
        if not check_pin(d[key]):
            raise Refused("INVALID_INTAKE_DIGEST_" + key)
    if type(d["key_epoch"]) is not int or d["key_epoch"] < 0:
        raise Refused("INVALID_KEY_EPOCH")
    for key in EXTERNAL:
        if d[key] is not None and not check_pin(d[key]):
            raise Refused("UNTRUSTED_EXTERNAL_PIN_MALFORMED_" + key)
    for key in FLOORS:
        if d[key] is not None and (type(d[key]) is not int or d[key] < 0):
            raise Refused("UNTRUSTED_EXTERNAL_FLOOR_INVALID_" + key)
    return d

def candidate(owner_raw, custodian_raw):
    """Untrusted local *proposal*, never proof of an external operator."""
    a = r5v.parse(owner_raw)
    b = r5v.parse(custodian_raw)
    if a["role"] != "owner_proxy" or b["role"] != "custodian_proxy":
        raise Refused("R5V_SIGNER_ROLES_NOT_BOUND")
    if (a["nonce"], a["history_head"], a["key_epoch"]) != (
        b["nonce"], b["history_head"], b["key_epoch"]
    ):
        raise Refused("R5V_OWNER_AND_CUSTODIAN_HISTORY_DISAGREE")
    d = dict(protocol=PROTOCOL, project=PROJECT, parent=PARENT,
             operation=OPERATION, challenge=a["nonce"],
             history_head=a["history_head"], key_epoch=a["key_epoch"],
             owner_report_sha256=digest(owner_raw),
             custodian_report_sha256=digest(custodian_raw),
             owner_key_sha256=a["key_sha256"],
             custodian_key_sha256=b["key_sha256"])
    d.update({key: None for key in EXTERNAL + FLOORS})
    return parse(canonical(d))

def inspect(dossier_raw, owner_raw, custodian_raw):
    """Structural verification, not autonomous external certification."""
    try:
        d = parse(dossier_raw)
        a = r5v.parse(owner_raw)
        b = r5v.parse(custodian_raw)
        if (d["owner_report_sha256"], d["custodian_report_sha256"]) != (
            digest(owner_raw), digest(custodian_raw)
        ):
            raise Refused("OS_REPORT_BYTES_NOT_BOUND_TO_INTAKE")
        if (d["challenge"], d["history_head"], d["key_epoch"],
            d["owner_key_sha256"], d["custodian_key_sha256"]) != (
            a["nonce"], a["history_head"], a["key_epoch"],
            a["key_sha256"], b["key_sha256"]
        ) or (b["nonce"], b["history_head"], b["key_epoch"]) != (
            a["nonce"], a["history_head"], a["key_epoch"]
        ):
            raise Refused("INTAKE_CHALLENGE_EPOCH_HISTORY_OR_KEY_MISMATCH")
        if a["role"] != "owner_proxy" or b["role"] != "custodian_proxy":
            raise Refused("ROLES_NOT_ANCHORED")
        if a["pid"] == b["pid"] or a["key_sha256"] == b["key_sha256"]:
            raise Refused("DISTINCT_LOCAL_PROCESS_AND_KEY_REQUIRED")
        prior = r5v.assess(a, b)
        if prior["status"] != "BLOCK" or prior["admission"] != "NOT_AUTHORIZED":
            raise Refused("R5V_NEGATIVE_EVIDENCE_NOT_ENFORCED")
        if any(d[k] is not None for k in EXTERNAL + FLOORS):
            return block("SELF_ASSERTED_EXTERNAL_PINS_AND_FLOORS_NOT_AUTHORITY",
                         structural_binding_verified=True,
                         local_os_boundary=prior["reason"],
                         self_declared_external_fields_quarantined=True)
        return block("NO_INDEPENDENT_EXTERNALLY_AUTHENTICATED_EVIDENCE",
                     structural_binding_verified=True,
                     local_os_boundary=prior["reason"],
                     self_declared_external_fields_quarantined=False)
    except (Refused, r5v.Refused, TypeError, ValueError) as e:
        return block("EXTERNAL_INTAKE_QUARANTINED",
                     detail=str(e) if isinstance(e, (Refused, r5v.Refused))
                     else type(e).__name__)

def native_exercise():
    """Two real locally signed reports; external trust remains unproven."""
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return block("POSIX_NATIVE_SIGNED_INTAKE_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5w-") as home:
        root = Path(home)
        os.chmod(root, 0o700)
        ok, oa = signed.create_same_admin_key(root, "r5w_owner")
        ck, ca = signed.create_same_admin_key(root, "r5w_candidate")
        op = rotate.public_sha(root / "r5w_owner.pub")
        cp = rotate.public_sha(root / "r5w_candidate.pub")
        if op == cp:
            raise Refused("NON_DISTINCT_LOCAL_REAL_KEYS")
        owner = r5v.child_observe("owner_proxy", "lab-owner-domain", op, 0,
                                   "b" * 64, "c" * 64)
        cand = r5v.child_observe("custodian_proxy", "lab-claimed-guardian", cp,
                                  0, "b" * 64, "c" * 64)
        if owner["pid"] == cand["pid"] or owner["uid"] != cand["uid"]:
            raise Refused("LOCAL_SAME_ADMIN_NEGATIVE_CONTROL_UNREPRODUCIBLE")
        ofile, cfile = root / "owner.json", root / "guardian.json"
        oraw, craw = r5v.canonical(owner), r5v.canonical(cand)
        ofile.write_bytes(oraw)
        cfile.write_bytes(craw)
        os.chmod(ofile, 0o600)
        os.chmod(cfile, 0o600)
        osig = r5v.signature(ofile, ok)
        csig = r5v.signature(cfile, ck)
        for path, signature, allow, pub, pin in (
            (ofile, osig, oa, root / "r5w_owner.pub", op),
            (cfile, csig, ca, root / "r5w_candidate.pub", cp),
        ):
            result = r5v.verify(path, signature, allow, pub, pin)
            if result["reason"] != "REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL":
                raise Refused("REAL_OPENSSH_SIGNATURE_TEST_FAILED")
        d = candidate(oraw, craw)
        base = inspect(canonical(d), oraw, craw)
        if base["reason"] != "NO_INDEPENDENT_EXTERNALLY_AUTHENTICATED_EVIDENCE":
            raise Refused("MISSING_EXTERNAL_CUSTODY_FALSELY_ACCEPTED")
        forged = dict(d, guardian_oob_key_pin=cp, external_revocation_floor=100)
        attempt = inspect(canonical(forged), oraw, craw)
        if attempt["reason"] != "SELF_ASSERTED_EXTERNAL_PINS_AND_FLOORS_NOT_AUTHORITY":
            raise Refused("SELF_ASSERTED_EXTERNAL_ROOT_FALSELY_ACCEPTED")
        return block("REAL_SIGNED_LOCAL_CUSTODY_EXTERNAL_INTAKE_DENIED",
                     genuine_local_ssh_ed25519="TWO_VALID_SAME_ADMIN_SIGNATURES",
                     local_processes="DIFFERENT_PIDS_SAME_OS_USER",
                     unsigned_external_trust_anchor="NONE",
                     claimed_external_pin="SELF_ASSERTED_QUARANTINED",
                     external_nonce_and_revocation_protection="NOT_VERIFIED",
                     r5v_admin_evidence="RETAINS_DENIAL",
                     native_admission="BLOCK")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise", action="store_true", required=True)
    args = p.parse_args(argv)
    try:
        report = native_exercise()
    except (Refused, r5v.Refused, OSError, ValueError) as exc:
        report = block("R5W_NATIVE_INTAKE_EXERCISE_FAILED",
                       detail=str(exc) if isinstance(exc, (Refused, r5v.Refused))
                       else type(exc).__name__)
    print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
