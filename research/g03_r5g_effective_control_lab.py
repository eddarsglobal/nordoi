#!/usr/bin/env python3
"""R5G actual same-UID second-process control counterexample and untrusted claims.

No live external guardian, owner authenticated OOB root, or admission route exists.
The CLI --run controls ONLY randomly-created temporary R5D/R5E databases.
The --actor helper requires that ephemeral directory's secret token and paths.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile

import g03_r5d_local_monotonic_ledger as publisher
import g03_r5e_second_store_witness as witness

PROTOCOL = "NORDOI-G03-R5G-EFFECTIVE-CONTROL-RESEARCH-V1"
PROJECT = "eddarsglobal/nordoi"
R5F_HEAD = "b84f49ce1c993f2c1f770a8d9882bb38b188aaae"
FIELDS = frozenset(("protocol", "project", "source_commit", "publisher_controller",
                    "guardian_controller_claim", "witness_controller_claim",
                    "owner_oob_root_sha256_claim", "guardian_receipt_sha256_claim",
                    "witness_highwater_sha256_claim", "durable_nonce_ledger_sha256_claim",
                    "recovery_controller_claim", "provenance_channel_claim"))
SHA = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
CTRL = re.compile(r"[a-z0-9][a-z0-9._/-]{0,79}\Z", re.ASCII)
MAX_BYTES = 8192

class Unsafe(ValueError):
    pass

def canonical(obj):
    return (json.dumps(obj, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def unique(pairs):
    out = {}
    for key, value in pairs:
        if key in out: raise Unsafe("DUPLICATE_KEY")
        out[key] = value
    return out

def nonfinite(_):
    raise Unsafe("NONFINITE_NUMBER")

def parse_claim(raw):
    if not isinstance(raw, bytes) or not 0 < len(raw) <= MAX_BYTES:
        raise Unsafe("CLAIM_MISSING_OR_OVERSIZED")
    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                           parse_constant=nonfinite)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise Unsafe("INVALID_JSON") from exc
    if type(value) is not dict or value.keys() != FIELDS or canonical(value) != raw:
        raise Unsafe("NONCANONICAL_OR_FIELDS_MISMATCH")
    if (value["protocol"] != PROTOCOL or value["project"] != PROJECT or
            value["source_commit"] != R5F_HEAD):
        raise Unsafe("WRONG_FROZEN_SOURCE")
    for key in ("publisher_controller", "guardian_controller_claim",
                "witness_controller_claim", "recovery_controller_claim"):
        if not isinstance(value[key], str) or not CTRL.fullmatch(value[key]):
            raise Unsafe("CONTROLLER_CLAIM_INVALID")
    for key in ("owner_oob_root_sha256_claim", "guardian_receipt_sha256_claim",
                "witness_highwater_sha256_claim", "durable_nonce_ledger_sha256_claim"):
        if type(value[key]) is not str or not SHA.fullmatch(value[key]) or value[key] == "0" * 64:
            raise Unsafe("HASH_CLAIM_INVALID")
    if value["provenance_channel_claim"] not in ("DECLARED_BY_PUBLISHER", "CLAIMED_OFFLINE"):
        raise Unsafe("CLAIMED_CHANNEL_NOT_RECOGNIZED")
    return value

def assess_claim(raw):
    try:
        value = parse_claim(raw)
    except (Unsafe, ValueError, TypeError, OverflowError) as exc:
        return {"status": "BLOCK", "admission": "NOT_AUTHORIZED",
                "reason": "INVALID_UNTRUSTED_CONTROL_CLAIMS", "detail": str(exc)}
    controllers = (value["publisher_controller"], value["guardian_controller_claim"],
                   value["witness_controller_claim"], value["recovery_controller_claim"])
    if len(set(controllers)) != 4:
        reason = "SHARED_CONTROLLER_CLAIM_DISQUALIFIES"
    else:
        reason = "NAMES_AND_DIGESTS_CANNOT_AUTHENTICATE_EXTERNAL_CONTROL"
    return {"status": "BLOCK", "admission": "NOT_AUTHORIZED", "reason": reason,
            "cryptographic_identity": "NOT_VERIFIED",
            "actual_oob_channel": "NOT_ATTESTED",
            "external_protected_highwater": "NOT_ENROLLED",
            "administrator_independence": "NOT_VERIFIED"}

def _inside_ephemeral_root(root):
    p = Path(root)
    if (not p.is_absolute() or p.is_symlink() or not p.is_dir() or
            not p.name.startswith("nordoi-r5g-live-")):
        raise Unsafe("ONLY_SELF_CREATED_EPHEMERAL_LAB_ROOT")
    if os.name != "nt" and p.stat().st_mode & 0o077:
        raise Unsafe("EPHEMERAL_LAB_NOT_PRIVATE")
    marker = p / "one-use-control-token"
    if marker.is_symlink() or not marker.is_file():
        raise Unsafe("NO_PRIVATE_LAB_MARKER")
    return p, marker

def _actor(root, token):
    """Actual separate process modifies only verified disposable two-DB snapshot."""
    p, marker = _inside_ephemeral_root(root)
    try:
        saved = marker.read_text("ascii")
    except (OSError, UnicodeError) as exc:
        raise Unsafe("INVALID_EPHEMERAL_TOKEN") from exc
    if len(saved) != 64 or saved != token or not SHA.fullmatch(token):
        raise Unsafe("EPHEMERAL_TOKEN_MISMATCH")
    a, b = p / "publisher", p / "witness"
    if (a.is_symlink() or b.is_symlink() or
            not a.is_dir() or not b.is_dir()):
        raise Unsafe("EPHEMERAL_STORE_DIRECTORY_INVALID")
    primary = a / publisher.DATABASE
    second = b / witness.FILE
    first = p / "saved-publisher.sqlite3"
    other = p / "saved-witness.sqlite3"
    for path in (primary, second, first, other):
        if path.is_symlink() or not path.is_file() or not stat.S_ISREG(path.lstat().st_mode):
            raise Unsafe("EPHEMERAL_SQLITE_PATH_INVALID")
        if path.read_bytes()[:16] != b"SQLite format 3\x00":
            raise Unsafe("EPHEMERAL_SQLITE_MAGIC_INVALID")
        if os.name != "nt" and path.stat().st_uid != os.geteuid():
            raise Unsafe("EPHEMERAL_FILES_NOT_SAME_UID")
    second_before = stat.S_IMODE(second.stat().st_mode)
    if os.name != "nt" and second_before != 0o400:
        raise Unsafe("SECOND_STORE_NOT_READONLY_BEFORE_ACTOR")
    os.chmod(second, 0o600)
    shutil.copyfile(first, primary)
    shutil.copyfile(other, second)
    return {"status": "BLOCK", "admission": "NOT_AUTHORIZED",
            "reason": "SAME_ADMIN_SECOND_PROCESS_RESTORED_BOTH_STORES",
            "actor_pid_differs": True, "witness_mode_before": second_before,
            "witness_mode_after": stat.S_IMODE(second.stat().st_mode),
            "external_operator": "NOT_ENROLLED"}

def exercise():
    """Real SQLite + real separate OS process, no outside side effects."""
    if os.name == "nt":
        return {"status": "BLOCK", "admission": "NOT_AUTHORIZED",
                "reason": "WINDOWS_ACL_ADMIN_CONTROL_NOT_ATTESTED",
                "external_operator": "NOT_ENROLLED"}
    with tempfile.TemporaryDirectory(prefix="nordoi-r5g-live-") as root_text:
        p = Path(root_text)
        os.chmod(p, 0o700)
        pub, wit = p / "publisher", p / "witness"
        pub.mkdir(mode=0o700)
        wit.mkdir(mode=0o700)
        os.chmod(pub, 0o700)
        os.chmod(wit, 0o700)
        token = hashlib.sha256(os.urandom(32)).hexdigest()
        marker = p / "one-use-control-token"
        marker.write_text(token, encoding="ascii")
        os.chmod(marker, 0o600)
        event1 = publisher.execute(pub,"append",nonce="a"*64,payload="b"*64,
                                   epoch=4,expected_prev=publisher.GENESIS)
        if event1.get("status") != "LAB_LOCAL_COMMIT": raise Unsafe("FIRST_LOCAL_EVENT_FAILED")
        witness1 = witness.execute(pub,wit,"checkpoint")
        if witness1.get("status") != "LAB_WITNESS_COMMIT": raise Unsafe("FIRST_WITNESS_FAILED")
        shutil.copy2(pub / publisher.DATABASE, p / "saved-publisher.sqlite3")
        shutil.copy2(wit / witness.FILE, p / "saved-witness.sqlite3")
        event2 = publisher.execute(pub,"append",nonce="c"*64,payload="b"*64,
                                   epoch=5,expected_prev=event1["local_head_sha256"])
        if event2.get("status") != "LAB_LOCAL_COMMIT": raise Unsafe("SECOND_LOCAL_EVENT_FAILED")
        witness2 = witness.execute(pub,wit,"checkpoint")
        if witness2.get("status") != "LAB_WITNESS_COMMIT": raise Unsafe("SECOND_WITNESS_FAILED")
        if witness.execute(pub,wit,"audit").get("witness_height") != 2:
            raise Unsafe("TWO_CHECKPOINTS_NOT_VERIFIED")
        os.chmod(wit / witness.FILE, 0o400)
        before = stat.S_IMODE((wit / witness.FILE).stat().st_mode)
        child = subprocess.run([sys.executable,"-B",str(Path(__file__).resolve()),
                                "--actor","--root",str(p),"--token",token],
                               capture_output=True,text=True,timeout=25,check=False)
        if child.returncode != 2: raise Unsafe("SEPARATE_ACTOR_CLI_FAILED")
        try: actor = json.loads(child.stdout)
        except ValueError as exc: raise Unsafe("ACTOR_RECEIPT_NOT_JSON") from exc
        if actor.get("reason") != "SAME_ADMIN_SECOND_PROCESS_RESTORED_BOTH_STORES":
            raise Unsafe("SEPARATE_ACTOR_NOT_CONFIRMED")
        after = stat.S_IMODE((wit / witness.FILE).stat().st_mode)
        audit = witness.execute(pub,wit,"audit")
        if (audit.get("reason") != "LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT" or
                audit.get("witness_height") != 1 or audit.get("local_height") != 1):
            raise Unsafe("JOINT_SNAPSHOT_ROLLBACK_COUNTEREXAMPLE_FAILED")
        return {"status": "BLOCK", "admission": "NOT_AUTHORIZED",
                "reason": "SAME_UID_CAN_RESTORE_READONLY_LOCAL_WITNESS",
                "two_actual_sqlite_databases": True,
                "witness_before_restore_height": 2,
                "witness_after_restore_height": 1,
                "witness_mode_before_actor": before,
                "witness_mode_after_actor": after,
                "separate_real_actor_process": True,
                "same_uid_control_weakness": "CONFIRMED_LOCAL_ONLY",
                "both_store_rollback_detections": "NO",
                "owner_oob_root": "NOT_ENROLLED",
                "independent_external_operator": "NOT_ENROLLED",
                "global_monotonic_antirollback": "NOT_IMPLEMENTED"}

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    modes = p.add_mutually_exclusive_group(required=True)
    modes.add_argument("--run",action="store_true")
    modes.add_argument("--actor",action="store_true")
    p.add_argument("--root")
    p.add_argument("--token")
    args = p.parse_args(argv)
    try:
        if args.run:
            if args.root or args.token: raise Unsafe("RUN_NEVER_ACCEPTS_USER_STORAGE_PATHS")
            result = exercise()
        else:
            if not args.root or not args.token: raise Unsafe("ACTOR_REQUIRES_EPHEMERAL_TOKEN")
            result = _actor(args.root,args.token)
    except (Unsafe, OSError, ValueError, TypeError, subprocess.SubprocessError) as exc:
        result = {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                  "reason":"LOCAL_CONTROL_EXPERIMENT_REFUSED",
                  "detail":str(exc) if isinstance(exc,Unsafe) else type(exc).__name__}
    print(json.dumps(result,sort_keys=True,separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
