#!/usr/bin/env python3
"""R5E actual second local SQLite witness process, always DENY authority.

Two real durable files may detect publisher-only rollback while witness survives.
Both remain controlled by the same OS account: neither can prove globally
monotonic external custody or authenticate an independent remote witness.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import stat
import sys

import g03_r5d_local_monotonic_ledger as primary

PROTOCOL = "NORDOI-G03-R5E-SECOND-LOCAL-WITNESS-LAB-V1"
PROJECT = "eddarsglobal/nordoi"
R5D_HEAD = "f870bc0a869d0dcf478a02c7ad1df87122135125"
GENESIS = hashlib.sha256((PROTOCOL + ":GENESIS").encode("ascii")).hexdigest()
FILE = "r5e_witness.sqlite3"

class Denied(ValueError):
    pass

def canon(obj):
    return (json.dumps(obj, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")

def witness_hash(previous, seq, epoch, nonce, local_prev, local_head):
    data = dict(protocol=PROTOCOL, project=PROJECT, r5d_source=R5D_HEAD,
                seq=seq, epoch=epoch, nonce=nonce, local_prev=local_prev,
                local_head=local_head, witness_prev=previous)
    return hashlib.sha256(previous.encode("ascii") + b"\n" + canon(data)).hexdigest()

def safe_roots(primary_dir, witness_dir):
    a, b = Path(primary_dir), Path(witness_dir)
    for p in (a, b):
        if not p.is_absolute() or p.is_symlink() or not p.is_dir():
            raise Denied("DIRECTORY_MUST_BE_EXISTING_ABSOLUTE_NOT_SYMLINK")
        if os.name != "nt" and p.stat().st_mode & 0o077:
            raise Denied("INSECURE_LAB_DIRECTORY_PERMISSIONS")
    aa, bb = a.resolve(strict=True), b.resolve(strict=True)
    if aa == bb or aa in bb.parents or bb in aa.parents:
        raise Denied("LAB_STORES_MUST_USE_DISTINCT_DIRECTORY_TREES")
    target = b / FILE
    if target.is_symlink() or (target.exists() and not target.is_file()):
        raise Denied("WITNESS_DB_NOT_REGULAR")
    if os.name != "nt" and target.exists() and target.stat().st_mode & 0o077:
        raise Denied("WITNESS_DB_EXCESSIVE_ACCESS")
    return aa, target

class Witness:
    def __init__(self, primary_dir, witness_dir):
        self.primary_dir, self.path = safe_roots(primary_dir, witness_dir)
        self.db = sqlite3.connect(str(self.path), timeout=8.0, isolation_level=None)
        try:
            self.db.execute("PRAGMA busy_timeout=8000")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1), height INTEGER NOT NULL, epoch INTEGER NOT NULL, local_head TEXT NOT NULL, witness_head TEXT NOT NULL)")
            self.db.execute("CREATE TABLE IF NOT EXISTS checkpoints (height INTEGER PRIMARY KEY, epoch INTEGER NOT NULL, nonce TEXT NOT NULL UNIQUE, local_prev TEXT NOT NULL, local_head TEXT NOT NULL, witness_prev TEXT NOT NULL, witness_head TEXT NOT NULL)")
            self.db.execute("INSERT OR IGNORE INTO state VALUES(1,0,4,?,?)", (primary.GENESIS, GENESIS))
            self._verify_witness()
            self.db.execute("COMMIT")
            os.chmod(self.path, 0o600)
        except BaseException:
            if self.db.in_transaction: self.db.execute("ROLLBACK")
            self.db.close()
            raise

    def __enter__(self): return self
    def __exit__(self, *_): self.db.close()

    def _verify_witness(self):
        if self.db.execute("PRAGMA quick_check").fetchone() != ("ok",):
            raise Denied("WITNESS_SQLITE_INTEGRITY_FAILED")
        meta = self.db.execute("SELECT height,epoch,local_head,witness_head FROM state WHERE id=1").fetchone()
        if meta is None or self.db.execute("SELECT count(*) FROM state").fetchone()[0] != 1:
            raise Denied("WITNESS_STATE_INCORRECT")
        height, epoch, local_head, witness_head = 0, 4, primary.GENESIS, GENESIS
        for h, e, nonce, local_prev, this_local, witness_prev, this_witness in self.db.execute(
            "SELECT height,epoch,nonce,local_prev,local_head,witness_prev,witness_head FROM checkpoints ORDER BY height").fetchall():
            if (type(h) is not int or h != height+1 or type(e) is not int or e < epoch or
                not primary.is_sha(nonce) or not primary.is_sha(this_local) or
                local_prev != local_head or witness_prev != witness_head or
                this_witness != witness_hash(witness_head,h,e,nonce,local_prev,this_local)):
                raise Denied("WITNESS_CHAIN_INVALID_OR_FORKED")
            height, epoch, local_head, witness_head = h, e, this_local, this_witness
        if meta != (height,epoch,local_head,witness_head):
            raise Denied("WITNESS_METADATA_MISMATCH")
        return meta

    def _current_primary(self):
        # Real R5D on-disk SQLite data, verified inside one SQLite write lock.
        with primary.LocalLedger(self.primary_dir) as local:
            local.db.execute("BEGIN IMMEDIATE")
            try:
                height,epoch,head = local._verify()
                event = local.db.execute(
                    "SELECT nonce,prev,head FROM events WHERE seq=?",(height,)).fetchone() if height else None
                local.db.execute("COMMIT")
            except BaseException:
                if local.db.in_transaction: local.db.execute("ROLLBACK")
                raise
        return height,epoch,head,event

    def checkpoint(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,e,last_local,last_witness = self._verify_witness()
            ph,pe,phead,event = self._current_primary()
            if ph != h+1 or event is None:
                raise Denied("WITNESS_REPLAY_GAP_OR_STALE_PRIMARY")
            nonce,local_prev,event_head = event
            if pe < e or local_prev != last_local or event_head != phead:
                raise Denied("WITNESS_PREDECESSOR_EPOCH_OR_HEAD_FORK")
            new = witness_hash(last_witness,ph,pe,nonce,local_prev,phead)
            self.db.execute("INSERT INTO checkpoints VALUES(?,?,?,?,?,?,?)",
                            (ph,pe,nonce,local_prev,phead,last_witness,new))
            self.db.execute("UPDATE state SET height=?,epoch=?,local_head=?,witness_head=? WHERE id=1",
                            (ph,pe,phead,new))
            self._verify_witness()
            self.db.execute("COMMIT")
            return dict(status="LAB_WITNESS_COMMIT", reason="ACTUAL_SECOND_SQLITE_STORE_LOCAL_ONLY",
                        admission="NOT_AUTHORIZED", witness_height=ph,
                        witness_head_sha256=new, global_antirollback="NOT_IMPLEMENTED",
                        remote_independent_custodian="NOT_ENROLLED")
        except BaseException:
            if self.db.in_transaction: self.db.execute("ROLLBACK")
            raise

    def audit(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,e,local_head,witness_head = self._verify_witness()
            ph,pe,phead,_ = self._current_primary()
            self.db.execute("COMMIT")
            if ph < h: reason="PRIMARY_ROLLBACK_DETECTED_BY_PRESERVED_SECOND_STORE"
            elif ph > h: reason="PRIMARY_AHEAD_OF_SECOND_STORE_UNCHECKPOINTED"
            elif phead != local_head or pe != e: reason="WITNESS_PRIMARY_EQUIVOCATION"
            else: reason="LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT"
            return dict(status="BLOCK", reason=reason, admission="NOT_AUTHORIZED",
                        local_height=ph,witness_height=h,
                        global_antirollback="NOT_IMPLEMENTED",
                        remote_independent_custodian="NOT_ENROLLED")
        except BaseException:
            if self.db.in_transaction: self.db.execute("ROLLBACK")
            raise

def execute(primary_dir,witness_dir,operation):
    try:
        with Witness(primary_dir,witness_dir) as w:
            return w.checkpoint() if operation=="checkpoint" else w.audit() if operation=="audit" else (_ for _ in ()).throw(Denied("UNKNOWN_OPERATION"))
    except (Denied,primary.Denied,sqlite3.Error,OSError,TypeError,ValueError) as err:
        return dict(status="BLOCK",reason="SECOND_STORE_OPERATION_REFUSED",detail=str(err),
                    admission="NOT_AUTHORIZED",global_antirollback="NOT_IMPLEMENTED")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--primary-directory",required=True)
    p.add_argument("--witness-directory",required=True)
    p.add_argument("--operation",required=True,choices=("checkpoint","audit"))
    args=p.parse_args(argv)
    print(json.dumps(execute(args.primary_directory,args.witness_directory,args.operation),
                     sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__": sys.exit(main())
