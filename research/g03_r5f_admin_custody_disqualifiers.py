#!/usr/bin/env python3
"""G0.3 R5F: read-only OS custody disqualifiers, not guardian enrollment.

Observes actual file/directory ownership for real R5D and R5E SQLite stores.
Same local UID or a common hardlink disqualifies independent custody.
Different OS UIDs and different devices still DO NOT prove independent control.
All outcomes BLOCK and no production trust root is ever enrolled.
"""
import argparse
import json
import os
from pathlib import Path
import stat
import sys

PROJECT = "eddarsglobal/nordoi"
R5E_HEAD = "04e5670cdfe9682e2d467c9f3678ba2b069afd54"
PROTOCOL = "NORDOI-G03-R5F-ADMIN-CUSTODY-NEGATIVE-CONTROL-V1"
PRIMARY_FILE = "r5d_lab.sqlite3"
WITNESS_FILE = "r5e_witness.sqlite3"
SQLITE_MAGIC = b"SQLite format 3\x00"

class UnsafeEvidence(ValueError):
    pass

def check_directory(value):
    path = Path(value)
    if not path.is_absolute() or path.is_symlink():
        raise UnsafeEvidence("DIRECTORY_NOT_ABSOLUTE_OR_SYMLINK")
    try:
        info = path.lstat()
    except OSError as error:
        raise UnsafeEvidence("DIRECTORY_NOT_ACCESSIBLE") from error
    if not stat.S_ISDIR(info.st_mode):
        raise UnsafeEvidence("NOT_A_DIRECTORY")
    if os.name != "nt" and info.st_mode & 0o077:
        raise UnsafeEvidence("DIRECTORY_NOT_PRIVATE")
    return path, info

def check_sqlite(path):
    if path.is_symlink():
        raise UnsafeEvidence("SQLITE_FILE_SYMLINK")
    try:
        info = path.lstat()
    except OSError as error:
        raise UnsafeEvidence("SQLITE_FILE_MISSING") from error
    if not stat.S_ISREG(info.st_mode):
        raise UnsafeEvidence("SQLITE_FILE_NONREGULAR")
    if os.name != "nt" and info.st_mode & 0o077:
        raise UnsafeEvidence("SQLITE_FILE_NOT_PRIVATE")
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
                 | getattr(os, "O_NONBLOCK", 0))
    try:
        live = os.fstat(fd)
        if not stat.S_ISREG(live.st_mode) or (info.st_dev, info.st_ino) != (live.st_dev, live.st_ino):
            raise UnsafeEvidence("SQLITE_FILE_SWAP")
        if os.read(fd, len(SQLITE_MAGIC)) != SQLITE_MAGIC:
            raise UnsafeEvidence("SQLITE_FILE_HEADER_INVALID")
        return live
    finally:
        os.close(fd)

def inspect(primary_directory, witness_directory):
    """Metadata only, read-only. Never return an identity or admission grant."""
    a, a_dir = check_directory(primary_directory)
    b, b_dir = check_directory(witness_directory)
    ar = a.resolve(strict=True)
    br = b.resolve(strict=True)
    if ar == br or ar in br.parents or br in ar.parents:
        raise UnsafeEvidence("SAME_OR_NESTED_STORE_DIRECTORY")
    a_file = check_sqlite(a / PRIMARY_FILE)
    b_file = check_sqlite(b / WITNESS_FILE)
    if (a_file.st_dev, a_file.st_ino) == (b_file.st_dev, b_file.st_ino):
        raise UnsafeEvidence("SAME_FILE_HARDLINK_OR_ALIAS")
    if os.name == "nt":
        finding = "NT_OWNER_ACL_AND_ADMIN_DOMAIN_NOT_ATTESTED"
    elif a_dir.st_uid == b_dir.st_uid and a_file.st_uid == b_file.st_uid:
        finding = "SAME_OS_UID_CONTROLS_BOTH_SQLITE_STORES"
    elif a_file.st_uid == b_file.st_uid or a_dir.st_uid == b_dir.st_uid:
        finding = "SHARED_OS_OWNER_IN_CUSTODY_PATH"
    else:
        finding = "DIFFERENT_UIDS_STILL_NO_EXTERNAL_CUSTODY_PROOF"
    return {
        "status": "BLOCK",
        "admission": "NOT_AUTHORIZED",
        "finding": finding,
        "distinct_file_inodes": True,
        "same_storage_device": a_file.st_dev == b_file.st_dev,
        "external_identity": "NOT_VERIFIED",
        "owner_oob_key_root": "NOT_ENROLLED",
        "external_protected_highwater": "NOT_IMPLEMENTED",
        "remote_durable_nonce_ledger": "NOT_IMPLEMENTED",
        "administrator_independence": "NOT_ATTESTED",
        "production": "NOT_AUTHORIZED",
    }

def assess(primary_directory, witness_directory):
    try:
        return inspect(primary_directory, witness_directory)
    except (UnsafeEvidence, OSError, ValueError, TypeError) as error:
        return {
            "status": "BLOCK",
            "admission": "NOT_AUTHORIZED",
            "finding": "UNSAFE_OR_MISSING_CUSTODY_EVIDENCE",
            "detail": str(error) if isinstance(error, UnsafeEvidence) else type(error).__name__,
            "external_identity": "NOT_VERIFIED",
            "administrator_independence": "NOT_ATTESTED",
            "production": "NOT_AUTHORIZED",
        }

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--primary-directory", required=True)
    parser.add_argument("--witness-directory", required=True)
    args = parser.parse_args(argv)
    print(json.dumps(assess(args.primary_directory, args.witness_directory),
                     sort_keys=True, separators=(",", ":")))
    return 2

if __name__ == "__main__":
    sys.exit(main())
