#!/usr/bin/env python3
"""NORDOI G0.3 R5Z — preserved same-admin witness and coherent rollback.

A preserved signed LOCAL witness detects a restored primary ledger if the
witness is retained, but the same administrator can restore BOTH histories.
Genuine OpenSSH Ed25519 is key possession only, never an independent
guardian. No native grants, privileged user switches, or production writes.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import g03_r5y_signed_nonce_rollback as nonce
import g03_r5x_signed_external_provenance as receipts
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as identities
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5o_fd_identity_quarantine as fdgate

PROTOCOL = "NORDOI-G03-R5Z-PRESERVED-LOCAL-WITNESS-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "fb70bef860353895285e62d14e2f236402999cf7"
NAMESPACE = "nordoi-r5z"
PREFIX = "nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-r5y-r5z-"
WITNESS = "retained_witness.json"
OLD_WITNESS = "before_second_witness.json"
FIELDS = frozenset(("protocol", "project", "parent", "custody",
                    "source_receipt_sha256", "local_height",
                    "nonce_set_sha256", "local_revocation_floor",
                    "observed_at", "owner_oob_root", "external_guardian",
                    "admission"))

class Refused(ValueError):
    pass

def canonical(d):
    return (json.dumps(d, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def distinct(rows):
    result = {}
    for k, v in rows:
        if k in result:
            raise Refused("DUPLICATE_WITNESS_EVIDENCE_KEY")
        result[k] = v
    return result

def finite(_):
    raise Refused("NONFINITE_WITNESS_EVIDENCE")

def parse(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= 2048:
        raise Refused("WITNESS_MISSING_OR_OVERSIZED")
    try:
        d = json.loads(raw.decode("utf-8"), object_pairs_hook=distinct,
                       parse_constant=finite)
    except (ValueError, UnicodeError) as e:
        if isinstance(e, Refused):
            raise
        raise Refused("WITNESS_INVALID_UTF8_OR_JSON") from e
    if type(d) is not dict or d.keys() != FIELDS or canonical(d) != raw:
        raise Refused("WITNESS_SCHEMA_OR_CANONICAL_MISMATCH")
    if (d["protocol"], d["project"], d["parent"], d["custody"],
        d["owner_oob_root"], d["external_guardian"], d["admission"]) != (
        PROTOCOL, PROJECT, PARENT, "SAME_ADMIN_LOCAL_ONLY", None, None,
        "NOT_AUTHORIZED"
    ):
        raise Refused("UNPINNED_OR_FALSELY_INDEPENDENT_WITNESS")
    for field in ("source_receipt_sha256", "nonce_set_sha256"):
        if not intake.check_pin(d[field]):
            raise Refused("WITNESS_HASH_INVALID_" + field)
    for field in ("local_height", "local_revocation_floor", "observed_at"):
        if type(d[field]) is not int or d[field] < 0:
            raise Refused("WITNESS_COUNTER_INVALID_" + field)
    if d["local_height"] == 0:
        raise Refused("EMPTY_WITNESS_NOT_QUALIFIED")
    return d

def deny(reason, **additional):
    return dict(status="BLOCK", admission="NOT_AUTHORIZED",
                native_authority_granted=False, production="NOT_AUTHORIZED",
                independent_custody="NOT_PROVEN",
                external_guardian="NOT_ENROLLED",
                owner_oob_root="NOT_ENROLLED",
                globally_monotonic_antirollback="NOT_VERIFIED",
                native_admission_gates_unmet=12, reason=reason, **additional)

def lab_root(path):
    root = nonce.private_root(path)
    if not root.name.startswith(PREFIX):
        raise Refused("R5Z_DISPOSABLE_LAB_PREFIX_REQUIRED")
    return root

def nonce_digest(values):
    return nonce.sha(canonical(list(values)))

def candidate(root, receipt_raw, observed_at):
    """Unsigned portable structural witness; signature verified separately."""
    local = nonce.audit(lab_root(root))
    receipt = receipts.parse(receipt_raw)
    if (type(observed_at) is not int or observed_at < 0 or
        local["height"] < 1 or receipt["challenge"] not in local["challenges"]):
        raise Refused("WITNESS_REQUIRES_ACTUAL_SPENT_LOCAL_CHALLENGE")
    d = dict(protocol=PROTOCOL, project=PROJECT, parent=PARENT,
             custody="SAME_ADMIN_LOCAL_ONLY",
             source_receipt_sha256=nonce.sha(receipt_raw),
             local_height=local["height"],
             nonce_set_sha256=nonce_digest(local["challenges"]),
             local_revocation_floor=local["revocation_floor"],
             observed_at=observed_at, owner_oob_root=None,
             external_guardian=None, admission="NOT_AUTHORIZED")
    return parse(canonical(d))

def compare(root, witness_raw):
    """Preserved local copy can disqualify; never grant, even on match."""
    try:
        d = parse(witness_raw)
        local = nonce.audit(lab_root(root))
        if local["height"] < d["local_height"] or (
            local["revocation_floor"] < d["local_revocation_floor"]
        ):
            return deny("PRESERVED_LOCAL_WITNESS_DETECTS_PRIMARY_ROLLBACK",
                        preserved_local_copy_only=True,
                        primary_height=local["height"],
                        witness_height=d["local_height"])
        if local["height"] > d["local_height"]:
            return deny("PRIMARY_AHEAD_OF_OLDER_LOCAL_WITNESS",
                        preserved_local_copy_only=True)
        if (nonce_digest(local["challenges"]) != d["nonce_set_sha256"] or
            local["revocation_floor"] != d["local_revocation_floor"]):
            return deny("SAME_HEIGHT_OR_FLOOR_HISTORY_MISMATCH",
                        preserved_local_copy_only=True)
        return deny("COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL",
                    local_consistency_only=True,
                    globally_protected_witness=False)
    except (Refused, nonce.Refused, OSError, ValueError, TypeError) as e:
        return deny("LOCAL_WITNESS_EVIDENCE_QUARANTINED",
                    detail=str(e) if isinstance(e, (Refused, nonce.Refused))
                    else type(e).__name__)

def sign_local(path, key):
    return receipts.sign_local(path, key, NAMESPACE)

def verify_local(witness_raw, signature, allowed_signers):
    """Real cryptographic verification, with an explicitly local allowlist."""
    try:
        parse(witness_raw)
        if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
            return deny("GENUINE_POSIX_SSHSIG_NOT_AVAILABLE")
        for rawpath in (signature, allowed_signers):
            p = Path(rawpath)
            if not p.is_file() or p.is_symlink():
                raise Refused("WITNESS_SIGNATURE_FILE_INVALID")
        with fdgate.OpenSnapshot(Path(signature)) as held:
            if ssh.run_ssh(["-Y", "verify", "-f", str(allowed_signers),
                            "-I", ssh.IDENTITY, "-n", NAMESPACE,
                            "-s", str(signature)], witness_raw) != 0:
                raise Refused("REAL_LOCAL_WITNESS_SSHSIG_INVALID")
            held.check()
        return deny("REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL",
                    cryptographic_signature_valid=True,
                    local_same_admin_signer=True)
    except (Refused, fdgate.Refused, OSError, ValueError, TypeError) as e:
        return deny("SIGNED_LOCAL_WITNESS_QUARANTINED",
                    detail=str(e) if isinstance(e, (Refused, fdgate.Refused))
                    else type(e).__name__)

def save_witness(root, raw, name):
    if name not in (WITNESS, OLD_WITNESS):
        raise Refused("ONLY_FIXED_RESEARCH_WITNESS_PATHS")
    root = lab_root(root)
    parse(raw)
    path = root / name
    fd = os.open(str(path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(raw)
            handle.flush()
            os.fsync(handle.fileno())
    except Exception:
        path.unlink(missing_ok=True)
        raise
    return path

def restore_witness_lab(root_path, old_path):
    """OS helper may only restore fixed disposable local witness+signature."""
    root = lab_root(root_path)
    old = Path(old_path)
    if (not old.is_absolute() or old.is_symlink() or
        old.name != OLD_WITNESS or
        old.parent.resolve(strict=True) != root or not old.is_file()):
        raise Refused("WITNESS_RESTORE_SOURCE_OUTSIDE_LAB")
    dest = root / WITNESS
    oldsig = Path(str(old) + ".sig")
    destsig = Path(str(dest) + ".sig")
    if any(p.is_symlink() or not p.is_file() or
           (os.name != "nt" and p.stat().st_mode & 0o077)
           for p in (old, oldsig, dest, destsig)):
        raise Refused("RESTORE_WITNESS_OR_SIGNATURE_INVALID")
    oldraw = old.read_bytes()
    parse(oldraw)
    prepared = root / "prepared_witness.json"
    prepared_sig = root / "prepared_witness.json.sig"
    if prepared.exists() or prepared_sig.exists():
        raise Refused("RESTORE_TEMP_ALREADY_EXISTS")
    shutil.copy2(old, prepared)
    shutil.copy2(oldsig, prepared_sig)
    if os.name != "nt":
        os.chmod(prepared, 0o600)
        os.chmod(prepared_sig, 0o600)
    os.replace(prepared, dest)
    os.replace(prepared_sig, destsig)
    return dict(pid=os.getpid(), ppid=os.getppid(),
                uid=os.getuid() if hasattr(os, "getuid") else None,
                euid=os.geteuid() if hasattr(os, "geteuid") else None,
                action="SAME_ADMIN_WITNESS_AND_SIGNATURE_RESTORED")

def child_restore_witness(root, source):
    p = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()),
                        "--restore-witness", str(root), str(source)],
                       capture_output=True, text=True, timeout=30, check=False)
    if p.returncode != 0:
        raise Refused("TRUE_SECOND_OS_WITNESS_RESTORE_FAILED")
    try:
        d = json.loads(p.stdout)
    except json.JSONDecodeError as e:
        raise Refused("OS_WITNESS_RESTORE_REPORT_INVALID") from e
    if (d.get("pid") == os.getpid() or d.get("action") !=
        "SAME_ADMIN_WITNESS_AND_SIGNATURE_RESTORED"):
        raise Refused("WITNESS_RESTORE_NOT_SECOND_OS_PROCESS")
    return d

def native_exercise():
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return deny("POSIX_OS_AND_REAL_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=PREFIX) as name:
        root = Path(name)
        os.chmod(root, 0o700)
        owner_key, _ = ssh.create_same_admin_key(root, "r5z_owner")
        guardian_key, allowed = ssh.create_same_admin_key(root, "r5z_guardian")
        owner_pin = keys.public_sha(root / "r5z_owner.pub")
        guardian_pin = keys.public_sha(root / "r5z_guardian.pub")
        if owner_pin == guardian_pin:
            raise Refused("SAME_ED25519_KEY_NOT_VALID_NEGATIVE_CONTROL")
        def actual_receipt(challenge, label):
            a = identities.child_observe("owner_proxy", "lab-owner-domain",
                                         owner_pin, 0, "b" * 64, challenge)
            b = identities.child_observe("custodian_proxy", "lab-guardian-domain",
                                         guardian_pin, 0, "b" * 64, challenge)
            if a["pid"] == b["pid"] or a["uid"] != b["uid"] or a["euid"] != b["euid"]:
                raise Refused("REAL_SAME_ADMIN_OS_PROCESSES_NOT_OBSERVED")
            ar, br = identities.canonical(a), identities.canonical(b)
            dossier = intake.canonical(intake.candidate(ar, br))
            obj = receipts.make_candidate(dossier, guardian_pin, 1000)
            raw = receipts.canonical(obj)
            path = root / (label + ".json")
            path.write_bytes(raw)
            os.chmod(path, 0o600)
            sig = receipts.sign_local(path, guardian_key)
            check = receipts.verify(raw, dossier, ar, br, 1100, sig,
                                    allowed, root / "r5z_guardian.pub")
            if check["reason"] != "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN":
                raise Refused("REAL_RECEIPT_SIGNING_FAILED")
            return raw
        first = actual_receipt("c" * 64, "receipt_one")
        second = actual_receipt("d" * 64, "receipt_two")
        nonce.bootstrap(root)
        if nonce.commit_local_denial(root, first, 1100, 1)["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("FIRST_LOCAL_SPENT_CHALLENGE_COMMIT_FAILED")
        old_db = root / "before_second.sqlite3"
        shutil.copy2(nonce.db_path(root), old_db)
        os.chmod(old_db, 0o600)
        h1 = canonical(candidate(root, first, 1100))
        old_witness = save_witness(root, h1, OLD_WITNESS)
        old_sig = sign_local(old_witness, guardian_key)
        if nonce.commit_local_denial(root, second, 1100, 2)["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("SECOND_LOCAL_SPENT_CHALLENGE_COMMIT_FAILED")
        h2 = canonical(candidate(root, second, 1100))
        retained = save_witness(root, h2, WITNESS)
        signed_retained = sign_local(retained, guardian_key)
        if verify_local(h2, signed_retained, allowed)["reason"] != "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL":
            raise Refused("RETAINED_REAL_SSHSIG_INVALID")
        if compare(root, h2)["reason"] != "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL":
            raise Refused("CURRENT_WITNESS_NOT_MATCHING_LOCAL_PRIMARY")
        primary_child = nonce.child_restore(root, old_db)
        if primary_child["uid"] != os.getuid():
            raise Refused("SAME_ADMIN_PRIMARY_RESTORE_NOT_OBSERVED")
        if compare(root, h2)["reason"] != "PRESERVED_LOCAL_WITNESS_DETECTS_PRIMARY_ROLLBACK":
            raise Refused("PRESERVED_LOCAL_WITNESS_DID_NOT_FLAG_OLD_PRIMARY")
        child = child_restore_witness(root, old_witness)
        if child["uid"] != os.getuid():
            raise Refused("SAME_ADMIN_WITNESS_RESTORE_NOT_OBSERVED")
        restored = retained.read_bytes()
        if restored != h1 or compare(root, restored)["reason"] != "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL":
            raise Refused("COHERENT_SAME_ADMIN_ROLLBACK_COUNTEREXAMPLE_NOT_REPRODUCED")
        if verify_local(restored, signed_retained, allowed)["reason"] != "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL":
            raise Refused("OLD_SIGNED_WITNESS_NOT_CRYPTOGRAPHICALLY_VALID")
        if nonce.commit_local_denial(root, second, 1100, 2)["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("OLD_SPENT_NONCE_NOT_REISSUED_AFTER_COORDINATED_RESTORE")
        return deny("REAL_SIGNED_PRESERVED_VS_COHERENT_ROLLBACK_QUALIFIED",
                    true_second_process_primary_restore="PASS",
                    true_second_process_signed_witness_restore="PASS",
                    retained_local_witness_detects_primary_rollback="PASS",
                    coherent_both_history_restore="LOCALLY_INDISTINGUISHABLE",
                    old_local_ed25519_signature="STILL_VALID",
                    locally_spent_nonce_after_double_restore="REISSUED",
                    external_guardian_enrollment="NONE",
                    global_monotonicity="NOT_IMPLEMENTED",
                    native_admission="BLOCK")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    group = p.add_mutually_exclusive_group(required=True)
    group.add_argument("--exercise", action="store_true")
    group.add_argument("--restore-witness", nargs=2, metavar=("ROOT", "OLD"))
    args = p.parse_args(argv)
    if args.restore_witness:
        try:
            print(json.dumps(restore_witness_lab(*args.restore_witness),
                             sort_keys=True, separators=(",", ":")))
            return 0
        except (Refused, OSError, nonce.Refused):
            return 1
    try:
        report = native_exercise()
    except (Refused, nonce.Refused, identities.Refused, receipts.Refused,
            keys.Refused, ssh.Refused, fdgate.Refused, OSError, ValueError) as e:
        report = deny("R5Z_REAL_NATIVE_WITNESS_EXPERIMENT_FAILED",
                      detail=str(e) if isinstance(e, (Refused, nonce.Refused,
                          identities.Refused, receipts.Refused, keys.Refused,
                          ssh.Refused, fdgate.Refused)) else type(e).__name__)
    print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
