#!/usr/bin/env python3
"""NORDOI G0.3 R5D: ACTUAL local SQLite durability/replay research, DENY ONLY.

A local SQLite transaction is not independent external monotonicity. A publisher
can restore an older database snapshot. This code never admits production.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import stat
import sys

PROTOCOL = "NORDOI-G03-R5D-LOCAL-MONOTONIC-LAB-V1"
PROJECT = "eddarsglobal/nordoi"
PARENT_SHA = "639272ce393f6005a93cf08da95e76859851e0f4"
MIN_EPOCH = 4
GENESIS = hashlib.sha256((PROTOCOL + ":GENESIS").encode("ascii")).hexdigest()
HEX = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
DATABASE = "r5d_lab.sqlite3"

class Denied(ValueError):
    pass

def is_sha(value):
    return type(value) is str and bool(HEX.fullmatch(value)) and value != "0" * 64

def canonical(value):
    return (json.dumps(value, ensure_ascii=True, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("ascii")

def receipt_hash(prev, seq, epoch, nonce, payload):
    record = dict(protocol=PROTOCOL, project=PROJECT, r5c_head=PARENT_SHA,
                  previous=prev, sequence=seq, epoch=epoch, nonce=nonce,
                  payload_sha256=payload)
    return hashlib.sha256(prev.encode("ascii") + b"\n" + canonical(record)).hexdigest()

def private_lab_path(root):
    """Path hygiene only: does NOT create an externally protected boundary."""
    root = Path(root)
    if root.is_symlink() or not root.is_dir():
        raise Denied("LAB_ROOT_MUST_BE_A_REAL_DIRECTORY")
    if os.name != "nt" and root.stat().st_mode & 0o077:
        raise Denied("LAB_DIRECTORY_MUST_NOT_ALLOW_OTHER_USERS")
    if not root.is_absolute():
        raise Denied("LAB_DIRECTORY_MUST_BE_ABSOLUTE")
    db = root / DATABASE
    if db.is_symlink() or (db.exists() and not db.is_file()):
        raise Denied("DB_SYMLINK_OR_NONREGULAR_FORBIDDEN")
    if os.name != "nt" and db.exists() and db.stat().st_mode & 0o077:
        raise Denied("DB_FILE_PERMISSIONS_INSECURE")
    return db

class LocalLedger:
    """Local, reversible storage; never a global trust authority."""
    def __init__(self, scratch_root):
        self.path = private_lab_path(scratch_root)
        self.db = sqlite3.connect(str(self.path), isolation_level=None, timeout=8.0)
        try:
            self.db.execute("PRAGMA busy_timeout=8000")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            self.db.execute("PRAGMA temp_store=MEMORY")
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("CREATE TABLE IF NOT EXISTS metadata (id INTEGER PRIMARY KEY CHECK(id=1), height INTEGER NOT NULL, epoch INTEGER NOT NULL, head TEXT NOT NULL)")
            self.db.execute("CREATE TABLE IF NOT EXISTS events (seq INTEGER PRIMARY KEY, epoch INTEGER NOT NULL, nonce TEXT NOT NULL UNIQUE, payload_sha TEXT NOT NULL, prev TEXT NOT NULL, head TEXT NOT NULL)")
            self.db.execute("INSERT OR IGNORE INTO metadata(id, height, epoch, head) VALUES(1, 0, 4, ?)", (GENESIS,))
            self._verify()
            self.db.execute("COMMIT")
            os.chmod(self.path, 0o600)
        except BaseException:
            if self.db.in_transaction:
                self.db.execute("ROLLBACK")
            self.db.close()
            raise

    def close(self):
        self.db.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def _verify(self):
        quick = self.db.execute("PRAGMA quick_check").fetchone()
        if quick != ("ok",):
            raise Denied("SQLITE_INTEGRITY_CHECK_FAILED")
        meta = self.db.execute("SELECT height,epoch,head FROM metadata WHERE id=1").fetchone()
        if meta is None or self.db.execute("SELECT count(*) FROM metadata").fetchone()[0] != 1:
            raise Denied("METADATA_MISSING_OR_DUPLICATE")
        height, epoch, head = meta
        expected_head, expected_epoch, expected_height = GENESIS, MIN_EPOCH, 0
        records = self.db.execute("SELECT seq,epoch,nonce,payload_sha,prev,head FROM events ORDER BY seq").fetchall()
        for seq, event_epoch, nonce, payload, prev, event_head in records:
            if (type(seq) is not int or seq != expected_height + 1 or
                type(event_epoch) is not int or event_epoch < expected_epoch or
                not is_sha(nonce) or not is_sha(payload) or prev != expected_head or
                event_head != receipt_hash(prev, seq, event_epoch, nonce, payload)):
                raise Denied("LOCAL_LEDGER_CHAIN_CORRUPTED_OR_FORKED")
            expected_head = event_head
            expected_epoch = event_epoch
            expected_height = seq
        if (height,epoch,head) != (expected_height,expected_epoch,expected_head):
            raise Denied("METADATA_DOES_NOT_MATCH_CHAIN")
        return expected_height, expected_epoch, expected_head

    def inspect(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            height, epoch, head = self._verify()
            self.db.execute("COMMIT")
            return dict(status="BLOCK", reason="LOCAL_STATE_HAS_NO_INDEPENDENT_ANCHOR",
                        admission="NOT_AUTHORIZED", local_height=height,
                        local_epoch=epoch, local_head_sha256=head,
                        external_highwater="NOT_ENROLLED",
                        global_antirollback="NOT_IMPLEMENTED",
                        durable_external_nonce_ledger="NOT_IMPLEMENTED")
        except BaseException:
            if self.db.in_transaction:
                self.db.execute("ROLLBACK")
            raise

    def append(self, *, nonce, payload_sha256, epoch, expected_prev):
        if not is_sha(nonce) or not is_sha(payload_sha256) or not is_sha(expected_prev):
            raise Denied("INVALID_NONCE_PAYLOAD_OR_PREVIOUS_HEAD")
        if type(epoch) is not int or epoch < MIN_EPOCH:
            raise Denied("INVALID_OR_DOWNGRADED_EPOCH")
        self.db.execute("BEGIN IMMEDIATE")
        try:
            height, floor, prev = self._verify()
            if epoch < floor:
                raise Denied("EPOCH_ROLLBACK")
            if expected_prev != prev:
                raise Denied("STALE_OR_FORKED_PREVIOUS_HEAD")
            if self.db.execute("SELECT 1 FROM events WHERE nonce=?",(nonce,)).fetchone():
                raise Denied("CHALLENGE_ALREADY_CONSUMED_LOCALLY")
            new_head = receipt_hash(prev, height+1, epoch, nonce, payload_sha256)
            self.db.execute("INSERT INTO events(seq,epoch,nonce,payload_sha,prev,head) VALUES(?,?,?,?,?,?)",
                            (height+1, epoch, nonce, payload_sha256, prev, new_head))
            self.db.execute("UPDATE metadata SET height=?,epoch=?,head=? WHERE id=1",
                            (height+1, epoch, new_head))
            self._verify()
            self.db.execute("COMMIT")
            return dict(status="LAB_LOCAL_COMMIT", reason="ACTUAL_SQLITE_FULL_SYNC_LOCAL_ONLY",
                        admission="NOT_AUTHORIZED", local_height=height+1,
                        local_epoch=epoch, local_head_sha256=new_head,
                        external_highwater="NOT_ENROLLED",
                        global_antirollback="NOT_IMPLEMENTED")
        except BaseException:
            if self.db.in_transaction:
                self.db.execute("ROLLBACK")
            raise

def execute(root, op, *, nonce=None, payload=None, epoch=None, expected_prev=None):
    try:
        with LocalLedger(root) as lab:
            if op == "inspect":
                return lab.inspect()
            if op == "append":
                return lab.append(nonce=nonce, payload_sha256=payload,
                                  epoch=epoch, expected_prev=expected_prev)
            raise Denied("UNKNOWN_LOCAL_LAB_OPERATION")
    except (Denied, sqlite3.Error, OSError, ValueError, TypeError, OverflowError) as exc:
        return dict(status="BLOCK", reason="LOCAL_LEDGER_OPERATION_REFUSED",
                    detail=str(exc), admission="NOT_AUTHORIZED",
                    global_antirollback="NOT_IMPLEMENTED")

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--lab-directory", required=True,
                   help="OWNER-EXPLICIT PRIVATE TEMP DIR ONLY. Never original repo or production")
    p.add_argument("--operation", required=True, choices=("append","inspect"))
    p.add_argument("--nonce")
    p.add_argument("--payload-sha256")
    p.add_argument("--epoch", type=int)
    p.add_argument("--expected-prev")
    args = p.parse_args(argv)
    result = execute(args.lab_directory, args.operation,
                     nonce=args.nonce,payload=args.payload_sha256,epoch=args.epoch,
                     expected_prev=args.expected_prev)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 2  # EVEN A REAL LOCAL COMMIT NEVER GRANTS OUTSIDE CUSTODY

if __name__ == "__main__":
    sys.exit(main())
