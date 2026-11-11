#!/usr/bin/env python3
"""R5I: actual second SQLite revocation checkpoint; no independent authority.

A preserved local checkpoint detects loss of a signed R5H REVOKE.
Both same-admin SQLite snapshots can still be restored together. Every
outcome blocks native admission and no outside guardian is enrolled.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import stat
import sys

import g03_r5h_signed_revocation_lab as rev

PROTOCOL="NORDOI-G03-R5I-LOCAL-REVOCATION-CHECKPOINT-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="43192354c023cf7f952f0d09a6f7c68e97a7f03f"
NAME="r5i_local_revocation_floor.sqlite3"
GENESIS=hashlib.sha256((PROTOCOL+":GENESIS").encode("ascii")).hexdigest()

class Refused(ValueError):pass

def digest(prev,seq,epoch,nonce,packet_sha,local_prev,local_head):
    row={"protocol":PROTOCOL,"project":PROJECT,"r5h_parent":PARENT,
         "height":seq,"epoch":epoch,"nonce":nonce,"packet_sha":packet_sha,
         "primary_prev":local_prev,"primary_head":local_head,
         "floor_prev":prev}
    return hashlib.sha256(prev.encode("ascii")+b"\n"+rev.canonical(row)).hexdigest()

def private_dirs(primary_dir,checkpoint_dir):
    a,b=Path(primary_dir),Path(checkpoint_dir)
    for p in (a,b):
        if not p.is_absolute() or p.is_symlink() or not p.is_dir():
            raise Refused("STORE_DIRECTORY_NOT_PRIVATE_ABSOLUTE")
        if os.name!="nt" and p.stat().st_mode & 0o077:
            raise Refused("STORE_DIRECTORY_PERMISSIONS")
    x,y=a.resolve(strict=True),b.resolve(strict=True)
    if x==y or x in y.parents or y in x.parents:
        raise Refused("STORE_ROOT_OVERLAP")
    for db in (x/rev.DB_NAME,y/NAME):
        if db.is_symlink() or (db.exists() and not db.is_file()):
            raise Refused("SQLITE_SYMLINK_OR_NONREGULAR")
        if os.name!="nt" and db.exists() and db.stat().st_mode & 0o077:
            raise Refused("SQLITE_PERMISSIONS")
    return x,y/NAME

class Checkpoint:
    def __init__(self,primary_dir,checkpoint_dir):
        self.primary,self.path=private_dirs(primary_dir,checkpoint_dir)
        # Reopening an existing, corrupted floor must NEVER silently create new genesis.
        creating_new=not self.path.exists()
        self.db=sqlite3.connect(str(self.path),isolation_level=None,timeout=10.0)
        try:
            self.db.execute("PRAGMA busy_timeout=10000")
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1), height INTEGER NOT NULL, epoch INTEGER NOT NULL, primary_head TEXT NOT NULL, floor_head TEXT NOT NULL)")
            self.db.execute("CREATE TABLE IF NOT EXISTS checkpoints (height INTEGER PRIMARY KEY,epoch INTEGER NOT NULL,nonce TEXT NOT NULL UNIQUE,packet_sha TEXT NOT NULL,primary_prev TEXT NOT NULL,primary_head TEXT NOT NULL,floor_prev TEXT NOT NULL,floor_head TEXT NOT NULL)")
            if creating_new:
                self.db.execute("INSERT INTO state VALUES(1,0,4,?,?)",(rev.GENESIS,GENESIS))
            self._verify()
            self.db.execute("COMMIT")
            os.chmod(self.path,0o600)
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            self.db.close()
            raise
    def __enter__(self):return self
    def __exit__(self,*_):self.db.close()
    def _verify(self):
        if self.db.execute("PRAGMA quick_check").fetchone()!=("ok",):
            raise Refused("FLOOR_SQLITE_CORRUPTED")
        meta=self.db.execute("SELECT height,epoch,primary_head,floor_head FROM state WHERE id=1").fetchone()
        if meta is None or self.db.execute("SELECT count(*) FROM state").fetchone()[0]!=1:
            raise Refused("FLOOR_METADATA_MISSING")
        h,epoch,local,head=0,4,rev.GENESIS,GENESIS
        rows=self.db.execute("SELECT height,epoch,nonce,packet_sha,primary_prev,primary_head,floor_prev,floor_head FROM checkpoints ORDER BY height").fetchall()
        for seq,e,nonce,packet_sha,prev,local_head,floor_prev,floor_head in rows:
            if (type(seq) is not int or seq!=h+1 or type(e) is not int or e<epoch or
                not rev.is_sha(nonce) or not rev.is_sha(packet_sha) or
                not rev.is_sha(local_head) or prev!=local or floor_prev!=head or
                floor_head!=digest(head,seq,e,nonce,packet_sha,prev,local_head)):
                raise Refused("FLOOR_CHAIN_INVALID")
            h,epoch,local,head=seq,e,local_head,floor_head
        if meta!=(h,epoch,local,head):raise Refused("FLOOR_METADATA_MISMATCH")
        return meta
    def _primary(self):
        if not (self.primary/rev.DB_NAME).is_file():
            raise Refused("PRIMARY_SIGNED_REVOCATION_DB_MISSING")
        with rev.RevocationLedger(self.primary) as local:
            local.db.execute("BEGIN IMMEDIATE")
            try:
                h,head,epoch,revoked=local._verify()
                event=(local.db.execute("SELECT epoch,nonce,packet_sha,prev,head FROM events WHERE seq=?",(h,)).fetchone()
                       if h else None)
                local.db.execute("COMMIT")
            except BaseException:
                if local.db.in_transaction:local.db.execute("ROLLBACK")
                raise
        return h,head,epoch,revoked,event
    def checkpoint(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,e,local,head=self._verify()
            ph,phead,pe,revoked,event=self._primary()
            if not revoked or ph!=h+1 or event is None:
                raise Refused("NO_NEXT_SIGNED_REVOCATION_OR_CHECKPOINT_REPLAY")
            event_e,nonce,packet_sha,prev,this_head=event
            if (event_e!=pe or pe<e or prev!=local or this_head!=phead):
                raise Refused("PRIMARY_FORK_OR_EPOCH_ROLLBACK")
            next_head=digest(head,ph,pe,nonce,packet_sha,prev,phead)
            self.db.execute("INSERT INTO checkpoints VALUES(?,?,?,?,?,?,?,?)",
                            (ph,pe,nonce,packet_sha,prev,phead,head,next_head))
            self.db.execute("UPDATE state SET height=?,epoch=?,primary_head=?,floor_head=? WHERE id=1",
                            (ph,pe,phead,next_head))
            self._verify()
            self.db.execute("COMMIT")
            return {"status":"LAB_LOCAL_CHECKPOINT_COMMIT","admission":"NOT_AUTHORIZED",
                    "reason":"PRESERVED_SECOND_LOCAL_STORE_ONLY",
                    "floor_height":ph,"floor_head":next_head,
                    "independent_guardian":"NOT_ENROLLED","global_antirollback":"NOT_IMPLEMENTED"}
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            raise
    def audit(self):
        self.db.execute("BEGIN IMMEDIATE")
        try:
            h,e,local,head=self._verify()
            ph,phead,pe,revoked,_=self._primary()
            self.db.execute("COMMIT")
            if ph<h:
                reason="SIGNED_REVOCATION_LOSS_DETECTED_WITH_PRESERVED_LOCAL_FLOOR"
            elif ph>h:
                reason="PRIMARY_REVOCATION_AHEAD_OF_UNCHECKPOINTED_LOCAL_FLOOR"
            elif phead!=local or pe!=e or (h>0 and not revoked):
                reason="REVOCATION_FLOOR_PRIMARY_FORK_OR_EQUIVOCATION"
            else:
                reason="LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT"
            return {"status":"BLOCK","admission":"NOT_AUTHORIZED","reason":reason,
                    "primary_height":ph,"floor_height":h,
                    "local_revoked":bool(revoked),
                    "independent_guardian":"NOT_ENROLLED",
                    "external_protected_floor":"NOT_IMPLEMENTED",
                    "global_antirollback":"NOT_IMPLEMENTED"}
        except BaseException:
            if self.db.in_transaction:self.db.execute("ROLLBACK")
            raise

def execute(primary_dir,checkpoint_dir,action):
    try:
        with Checkpoint(primary_dir,checkpoint_dir) as lab:
            return lab.checkpoint() if action=="checkpoint" else lab.audit() if action=="audit" else (_ for _ in ()).throw(Refused("UNKNOWN_ACTION"))
    except (Refused,rev.Denied,sqlite3.Error,OSError,ValueError,TypeError) as e:
        return {"status":"BLOCK","admission":"NOT_AUTHORIZED",
                "reason":"LOCAL_FLOOR_OPERATION_REFUSED",
                "detail":str(e) if isinstance(e,(Refused,rev.Denied)) else type(e).__name__,
                "independent_guardian":"NOT_ENROLLED",
                "global_antirollback":"NOT_IMPLEMENTED"}

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--primary-directory",required=True)
    p.add_argument("--floor-directory",required=True)
    p.add_argument("--action",required=True,choices=("audit","checkpoint"))
    a=p.parse_args(argv)
    print(json.dumps(execute(a.primary_directory,a.floor_directory,a.action),
                     sort_keys=True,separators=(",",":")))
    return 2
if __name__=="__main__":sys.exit(main())
