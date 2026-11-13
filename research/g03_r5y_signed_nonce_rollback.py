#!/usr/bin/env python3
"""NORDOI G0.3 R5Y — real local receipt freshness, nonce and rollback lab.

Genuine R5X SSHSIG is only local key possession. A committed SQLite
spent-challenge journal and locally monotonic heights cannot survive a
coherent rollback under the SAME administrator. Every result DENIES native
admission. No enrollment, user switching, external credentials, or grants.
"""
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import stat
import subprocess
import sys
import tempfile
from urllib.parse import quote

import g03_r5x_signed_external_provenance as provenance
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as identities
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as keys

PROTOCOL = "NORDOI-G03-R5Y-LOCAL-FRESHNESS-ROLLBACK-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT = "fbef91f4dccc22d3a73a8cb4f466e738d3bedbdf"
LEDGER = "r5y_local_nonce_history.sqlite3"
PREFIX = "nordoi-r5l-r5q-r5s-r5t-r5u-r5w-r5x-r5y-"
STATUS = "DENY_ONLY"
PROOF_FIELDS = frozenset((
    "protocol", "project", "parent", "receipt_sha256", "challenge",
    "history_head", "key_epoch", "local_height", "observed_at",
    "local_revocation_floor", "status",
))

class Refused(ValueError):
    pass

def canonical(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(rows):
    result = {}
    for key, value in rows:
        if key in result:
            raise Refused("DUPLICATE_LOCAL_EVIDENCE_FIELD")
        result[key] = value
    return result

def finite(_):
    raise Refused("NONFINITE_LOCAL_EVIDENCE")

def parse_proof(raw):
    if type(raw) is not bytes or not 0 < len(raw) <= 2400:
        raise Refused("PROOF_MISSING_OR_OVERSIZED")
    try:
        x = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=finite)
    except (ValueError, UnicodeError) as err:
        if isinstance(err, Refused):
            raise
        raise Refused("PROOF_INVALID_UTF8_OR_JSON") from err
    if type(x) is not dict or x.keys() != PROOF_FIELDS or canonical(x) != raw:
        raise Refused("PROOF_NONCANONICAL_OR_WRONG_FIELDS")
    if (x["protocol"], x["project"], x["parent"], x["status"]) != (
        PROTOCOL, PROJECT, PARENT, STATUS
    ):
        raise Refused("PROOF_UNPINNED_OR_PRIVILEGED")
    for k in ("receipt_sha256", "challenge", "history_head"):
        if not intake.check_pin(x[k]):
            raise Refused("INVALID_PROOF_DIGEST_" + k)
    for k in ("key_epoch", "local_height", "observed_at",
              "local_revocation_floor"):
        if type(x[k]) is not int or x[k] < 0:
            raise Refused("INVALID_LOCAL_PROOF_COUNTER_" + k)
    if x["local_height"] < 1:
        raise Refused("LOCAL_HEIGHT_MUST_BE_POSITIVE")
    return x

def deny(reason, **additional):
    return dict(
        status="BLOCK", admission="NOT_AUTHORIZED",
        native_authority_granted=False, production="NOT_AUTHORIZED",
        independent_custodian="NOT_ENROLLED", owner_oob_root="NOT_ENROLLED",
        external_monotonicity="NOT_VERIFIED", independent_quorum="NONE",
        native_admission_gates_unmet=12, reason=reason, **additional,
    )

def private_root(path):
    path = Path(path)
    if not path.is_absolute() or path.is_symlink() or not path.is_dir():
        raise Refused("LAB_DIRECTORY_NOT_REAL_PRIVATE_ABSOLUTE")
    if not path.name.startswith(PREFIX):
        raise Refused("ONLY_DISPOSABLE_R5Y_LAB_ALLOWED")
    if os.name != "nt" and (path.stat().st_mode & 0o077):
        raise Refused("LAB_DIRECTORY_ACCESS_NOT_PRIVATE")
    return path.resolve(strict=True)

def db_path(root):
    db = private_root(root) / LEDGER
    if db.is_symlink():
        raise Refused("SYMLINK_LOCAL_HISTORY_FORBIDDEN")
    return db

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def bootstrap(root):
    db = db_path(root)
    fd = os.open(str(db), os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(fd)
    try:
        with closing(sqlite3.connect(str(db), timeout=10)) as con:
            con.execute("PRAGMA journal_mode=DELETE")
            con.execute("PRAGMA synchronous=FULL")
            con.execute("""CREATE TABLE used(
                height INTEGER PRIMARY KEY,
                challenge TEXT NOT NULL UNIQUE,
                receipt_hash TEXT NOT NULL,
                history_head TEXT NOT NULL,
                key_epoch INTEGER NOT NULL,
                local_revocation_floor INTEGER NOT NULL,
                status TEXT NOT NULL
            )""")
            con.commit()
        if os.name != "nt":
            os.chmod(db, 0o600)
    except Exception:
        db.unlink(missing_ok=True)
        raise
    return audit(root)

def audit(root):
    db = db_path(root)
    if not db.is_file() or db.is_symlink():
        raise Refused("LOCAL_SQLITE_MISSING")
    if os.name != "nt" and db.stat().st_mode & 0o077:
        raise Refused("LOCAL_SQLITE_WORLD_ACCESSIBLE")
    uri = "file:" + quote(str(db), safe="/") + "?mode=ro"
    try:
        with closing(sqlite3.connect(uri, uri=True, timeout=10)) as con:
            con.execute("PRAGMA query_only=ON")
            con.execute("BEGIN")
            if con.execute("PRAGMA quick_check").fetchone() != ("ok",):
                raise Refused("LOCAL_SQLITE_CORRUPT")
            rows = con.execute("""SELECT height,challenge,receipt_hash,history_head,
                key_epoch,local_revocation_floor,status FROM used ORDER BY height""").fetchall()
            con.execute("COMMIT")
    except sqlite3.Error as exc:
        raise Refused("LOCAL_SQLITE_SCHEMA_OR_DATA_INVALID") from exc
    for i, row in enumerate(rows, 1):
        height, challenge, rhash, hist, epoch, floor, status = row
        if (height != i or not intake.check_pin(challenge) or
            not intake.check_pin(rhash) or not intake.check_pin(hist) or
            type(epoch) is not int or epoch < 0 or type(floor) is not int or
            floor < 0 or status != STATUS):
            raise Refused("LOCAL_SQLITE_AUDIT_INVALID")
    if len({r[1] for r in rows}) != len(rows):
        raise Refused("LOCAL_NONCE_LEDGER_DUPLICATE")
    return dict(height=len(rows), challenges=tuple(r[1] for r in rows),
                revocation_floor=max((r[5] for r in rows), default=0),
                global_antirollback="NOT_IMPLEMENTED")

def commit_local_denial(root, signed_receipt_raw, observed_at, floor=0):
    """Durable SQLite local negative evidence; never an external authority."""
    receipt = provenance.parse(signed_receipt_raw)
    if (type(observed_at) is not int or observed_at < 0 or
        type(floor) is not int or floor < 0):
        raise Refused("INVALID_UNTRUSTED_LOCAL_CLOCK_OR_FLOOR")
    db = db_path(root)
    state = audit(root)
    if receipt["challenge"] in state["challenges"]:
        return deny("LOCAL_SIGNED_NONCE_ALREADY_SPENT",
                    local_durable_replay_block=True,
                    local_height=state["height"])
    if floor < state["revocation_floor"]:
        return deny("LOCAL_REVOCATION_FLOOR_REGRESSION",
                    local_durable_floor_block=True)
    if not receipt["issued_at"] <= observed_at <= receipt["expires_at"]:
        return deny("LOCAL_UNTRUSTED_TIMEBOX_INVALID")
    try:
        with closing(sqlite3.connect(str(db), timeout=10)) as con:
            con.execute("PRAGMA synchronous=FULL")
            con.execute("BEGIN IMMEDIATE")
            current = con.execute(
                "SELECT COUNT(*),MAX(local_revocation_floor) FROM used"
            ).fetchone()
            count, previous_floor = current[0], current[1] or 0
            if floor < previous_floor:
                con.execute("ROLLBACK")
                return deny("LOCAL_REVOCATION_FLOOR_REGRESSION",
                            local_durable_floor_block=True)
            if con.execute(
                "SELECT 1 FROM used WHERE challenge=?",
                (receipt["challenge"],)
            ).fetchone():
                con.execute("ROLLBACK")
                return deny("LOCAL_SIGNED_NONCE_ALREADY_SPENT",
                            local_durable_replay_block=True)
            con.execute("""INSERT INTO used VALUES(?,?,?,?,?,?,?)""",
                        (count + 1, receipt["challenge"], sha(signed_receipt_raw),
                         receipt["history_head"], receipt["key_epoch"], floor,
                         STATUS))
            con.commit()
    except sqlite3.Error as exc:
        raise Refused("LOCAL_SQLITE_COMMIT_INVALID") from exc
    proof = dict(protocol=PROTOCOL, project=PROJECT, parent=PARENT,
                 status=STATUS, receipt_sha256=sha(signed_receipt_raw),
                 challenge=receipt["challenge"], history_head=receipt["history_head"],
                 key_epoch=receipt["key_epoch"], local_height=count + 1,
                 observed_at=observed_at, local_revocation_floor=floor)
    parse_proof(canonical(proof))
    return deny("LOCAL_ONLY_SPENT_NONCE_COMMITTED",
                local_durable_replay_block=True,
                local_height=count + 1, canonical_local_proof_sha256=sha(canonical(proof)))

def restore_lab_snapshot(root, source):
    """Controlled real OS child helper: restore ONLY a disposable R5Y DB."""
    root = private_root(root)
    snapshot = Path(source)
    if (not snapshot.is_absolute() or snapshot.is_symlink() or
        snapshot.parent != root or snapshot.name != "before_second.sqlite3" or
        not snapshot.is_file() or snapshot.stat().st_mode & 0o077):
        raise Refused("RESTORE_SOURCE_OUTSIDE_LAB_FORBIDDEN")
    target = db_path(root)
    if not target.is_file() or target.is_symlink():
        raise Refused("RESTORE_TARGET_INVALID")
    spare = root / "prepared_restore.sqlite3"
    if spare.exists() or spare.is_symlink():
        raise Refused("RESTORE_SPARE_ALREADY_EXISTS")
    shutil.copy2(snapshot, spare)
    os.chmod(spare, 0o600)
    os.replace(spare, target)
    return dict(pid=os.getpid(), ppid=os.getppid(),
                uid=os.getuid() if hasattr(os, "getuid") else None,
                euid=os.geteuid() if hasattr(os, "geteuid") else None,
                event="REAL_SECOND_PROCESS_ATOMIC_SAME_ADMIN_RESTORE")

def child_restore(root, snapshot):
    proc = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()),
                           "--restore-lab", str(root), str(snapshot)],
                          capture_output=True, text=True, timeout=30, check=False)
    if proc.returncode != 0:
        raise Refused("REAL_RESTORE_HELPER_FAILED")
    try:
        d = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise Refused("REAL_RESTORE_HELPER_INVALID_OS_REPORT") from exc
    if (d.get("pid") == os.getpid() or d.get("event") !=
        "REAL_SECOND_PROCESS_ATOMIC_SAME_ADMIN_RESTORE"):
        raise Refused("REAL_CHILD_PROCESS_NOT_OBSERVED")
    return d

def native_exercise():
    if os.name == "nt" or not Path("/usr/bin/ssh-keygen").is_file():
        return deny("REAL_POSIX_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(prefix=PREFIX) as path:
        root = Path(path)
        os.chmod(root, 0o700)
        owner, _ = signed.create_same_admin_key(root, "r5y_owner")
        guardian, allowed = signed.create_same_admin_key(root, "r5y_guardian")
        op = keys.public_sha(root / "r5y_owner.pub")
        gp = keys.public_sha(root / "r5y_guardian.pub")
        if op == gp:
            raise Refused("REAL_LOCAL_ED25519_KEYS_IDENTICAL")
        def receipt_for(challenge):
            a = identities.child_observe("owner_proxy", "lab-owner-domain",
                                         op, 0, "b" * 64, challenge)
            b = identities.child_observe("custodian_proxy", "lab-guardian-domain",
                                         gp, 0, "b" * 64, challenge)
            if a["pid"] == b["pid"] or a["uid"] != b["uid"] or a["euid"] != b["euid"]:
                raise Refused("SAME_ADMIN_OS_NEGATIVE_CONTROL_NOT_REPRODUCED")
            ar, br = identities.canonical(a), identities.canonical(b)
            dossier = intake.canonical(intake.candidate(ar, br))
            doc = provenance.make_candidate(dossier, gp, 1000)
            file = root / ("receipt_" + challenge[:8] + ".json")
            raw = provenance.canonical(doc)
            file.write_bytes(raw)
            os.chmod(file, 0o600)
            sig = provenance.sign_local(file, guardian)
            out = provenance.verify(raw, dossier, ar, br, 1100,
                                    sig, allowed, root / "r5y_guardian.pub")
            if out["reason"] != "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN":
                raise Refused("GENUINE_SSHSIG_NEGATIVE_CONTROL_FAILED")
            return raw, doc
        first, a = receipt_for("c" * 64)
        second, b = receipt_for("d" * 64)
        bootstrap(root)
        if commit_local_denial(root, first, 1100, 1)["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("FIRST_DENY_ONLY_COMMIT_FAILED")
        snap = root / "before_second.sqlite3"
        shutil.copy2(db_path(root), snap)
        os.chmod(snap, 0o600)
        if commit_local_denial(root, second, 1100, 2)["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("SECOND_DENY_ONLY_COMMIT_FAILED")
        if audit(root)["height"] != 2:
            raise Refused("SECOND_DURABLE_HEIGHT_NOT_TWO")
        if commit_local_denial(root, second, 1100, 2)["reason"] != "LOCAL_SIGNED_NONCE_ALREADY_SPENT":
            raise Refused("SIGNED_NONCE_REPLAY_NOT_REFUSED")
        child = child_restore(root, snap)
        if child["uid"] != os.getuid() or audit(root)["height"] != 1:
            raise Refused("REAL_COHERENT_ROLLBACK_NOT_OBSERVED")
        replay = commit_local_denial(root, second, 1100, 2)
        if replay["reason"] != "LOCAL_ONLY_SPENT_NONCE_COMMITTED":
            raise Refused("LOCAL_ROLLBACK_DID_NOT_REISSUE_SPENT_NONCE")
        return deny("REAL_SSHSIG_NONCE_ROLLBACK_COUNTEREXAMPLE_QUALIFIED",
                    authentic_ssh_ed25519="SAME_ADMIN_SIGNATURE_VALID",
                    genuine_second_process_restore="PASS",
                    signed_nonce_replay_before_restore="BLOCK",
                    old_spent_nonce_after_coherent_restore="REISSUED_LOCALLY",
                    protected_external_revocation_floor="ABSENT",
                    real_independent_guardian="NOT_ENROLLED",
                    native_admission="BLOCK")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise", action="store_true")
    g.add_argument("--restore-lab", nargs=2, metavar=("ROOT", "SNAPSHOT"))
    args = p.parse_args(argv)
    if args.restore_lab:
        try:
            print(json.dumps(restore_lab_snapshot(*args.restore_lab),
                             sort_keys=True, separators=(",", ":")))
            return 0
        except (Refused, OSError):
            return 1
    try:
        out = native_exercise()
    except (Refused, provenance.Refused, identities.Refused, intake.Refused,
            keys.Refused, signed.Refused, OSError, ValueError, TypeError) as exc:
        out = deny("R5Y_REAL_NATIVE_COUNTEREXAMPLE_FAILED",
                   detail=str(exc) if isinstance(exc, (Refused, provenance.Refused,
                       identities.Refused, intake.Refused, keys.Refused,
                       signed.Refused)) else type(exc).__name__)
    print(json.dumps(out, sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
