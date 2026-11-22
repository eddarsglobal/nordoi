#!/usr/bin/env python3
"""APP-018 — local receiver-owned monotone freshness journal for APP-017.

Receiver bootstraps a private LOCAL anchor from an independently supplied
roster SHA-256. Each advance creates an exclusive, SHA-256 linked, immutable
journal entry, then replaces a canonical HEAD with fsync attempts. Every
process restart reconstructs and checks the WHOLE chain, latest HEAD and
current roster before any APP-017/016/015 native replay or filesystem effects.

An old roster together with its authentic old hash no longer suffices while
this specific trusted local anchor remains intact and is always consulted.
This does NOT survive malicious replacement/deletion of the whole anchor
root, dishonest selection of a different root, compromised host/filesystem,
crash during an in-flight native effect, or unverified electrical power loss.
No remote authority, signature, TPM, sandbox or production admission.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "app017_receiver_revocation"))
import roster_app017 as roster

portable = roster.portable
pinned = roster.pinned
audit = roster.audit

SCHEMA = "NORDOI_RECEIVER_LOCAL_FRESHNESS_JOURNAL_V1"
HEAD_SCHEMA = "NORDOI_RECEIVER_FRESHNESS_HEAD_V1"
RESULT = "NORDOI_RECEIVER_LOCAL_FRESHNESS_RESULT_V1"
MAX_ENTRIES = 128
MAX_ENTRY_BYTES = 8192
MAX_HEAD_BYTES = 4096
EVENT_FIELDS = frozenset((
    "schema","sequence","roster_generation","roster_sha256",
    "roster_state","active_policy_sha256","revoked_policy_sha256",
    "previous_entry_sha256","transition","release","signature",
    "host_adversary_protection","production",
))
HEAD_FIELDS = frozenset((
    "schema","sequence","roster_generation","roster_sha256",
    "roster_state","active_policy_sha256","latest_entry_sha256",
    "local_durability","power_loss_certification",
    "host_adversary_protection","production",
))
LOCAL_DURABILITY = "FILE_FSYNC_AND_DIRECTORY_FSYNC_WHEN_SUPPORTED"
MAX_ROSTER_GENERATION = 1_000_000


class Denied(Exception):
    def __init__(self, code: str):
        self.code=code
        super().__init__(code)


class Unknown(Exception):
    def __init__(self, code: str):
        self.code=code
        super().__init__(code)


def require(condition: bool, code: str):
    if not condition:
        raise Denied(code)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(data: dict) -> bytes:
    return json.dumps(data, sort_keys=True, ensure_ascii=True,
                      separators=(",",":")).encode("ascii")+b"\n"


def emit(row: dict):
    sys.stdout.buffer.write(canonical(row))
    sys.stdout.buffer.flush()


def decode_strict(raw: bytes, code: str) -> dict:
    try:
        value=portable.independent.decode_json(raw,code)
    except (audit.AuditRejected,ValueError):
        raise Denied(code) from None
    require(type(value) is dict and canonical(value)==raw,
            code)
    return value


def private_dir(path: Path, code: str):
    require(path.is_dir() and not path.is_symlink(),code)
    if os.name=="posix":
        bits=stat.S_IMODE(path.lstat().st_mode)
        require(not (bits & 0o077), "ANCHOR_DIRECTORY_NOT_PRIVATE")


def external_root(root: Path, *paths: Path | None):
    a=root.resolve(strict=True)
    for p in paths:
        if p is None or not p.exists():
            continue
        b=p.resolve(strict=True)
        require(a!=b and a not in b.parents and b not in a.parents,
                "ANCHOR_NOT_SEPARATE_FROM_UNTRUSTED_INPUT")


@contextmanager
def locked(root: Path):
    private_dir(root,"ANCHOR_ROOT_MISSING_OR_SYMLINKED")
    lock=root/".lock"
    try:
        lock.mkdir(mode=0o700)
    except FileExistsError:
        raise Denied("ANCHOR_BUSY_OR_STALE_LOCK") from None
    except OSError:
        raise Denied("ANCHOR_LOCK_UNAVAILABLE") from None
    try:
        yield
    finally:
        try:
            lock.rmdir()
        except OSError:
            # Fail-closed: a stale lock refuses ALL later operations.
            pass


def directory_fsync(path: Path):
    try:
        fd=os.open(path,os.O_RDONLY|getattr(os,"O_DIRECTORY",0))
    except OSError:
        if os.name=="nt":
            return
        raise Unknown("ANCHOR_DIRECTORY_FSYNC_UNCERTAIN") from None
    try:
        os.fsync(fd)
    except OSError:
        if os.name!="nt":
            raise Unknown("ANCHOR_DIRECTORY_FSYNC_UNCERTAIN") from None
    finally:
        os.close(fd)


def exclusive_file(path: Path, content: bytes):
    try:
        fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    except FileExistsError:
        raise Denied("ANCHOR_IMMUTABLE_JOURNAL_ENTRY_EXISTS") from None
    except OSError:
        raise Unknown("ANCHOR_JOURNAL_PUBLICATION_UNCERTAIN") from None
    try:
        with os.fdopen(fd,"wb") as f:
            f.write(content)
            f.flush()
            os.fsync(f.fileno())
    except OSError:
        raise Unknown("ANCHOR_JOURNAL_PUBLICATION_UNCERTAIN") from None


def atomic_head(root: Path, data: bytes):
    fd=None
    temporary=None
    try:
        fd,name=tempfile.mkstemp(prefix=".head-pending-",dir=root)
        temporary=Path(name)
        if os.name=="posix":
            os.fchmod(fd,0o600)
        with os.fdopen(fd,"wb") as f:
            fd=None
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
        os.replace(temporary,root/"HEAD.json")
        directory_fsync(root)
    except OSError:
        raise Unknown("ANCHOR_HEAD_PUBLICATION_UNCERTAIN") from None
    finally:
        if fd is not None:
            os.close(fd)
        if temporary is not None:
            try:
                temporary.unlink(missing_ok=True)
            except OSError:
                pass


def record(roster_obj: dict, roster_sha: str,
           sequence: int, previous_sha: str | None) -> dict:
    return {
        "schema":SCHEMA,
        "sequence":sequence,
        "roster_generation":roster_obj["generation"],
        "roster_sha256":roster_sha,
        "roster_state":roster_obj["state"],
        "active_policy_sha256":roster_obj["active_policy_sha256"],
        "revoked_policy_sha256":roster_obj["revoked_policy_sha256"],
        "previous_entry_sha256":previous_sha,
        "transition":"BOOTSTRAP" if sequence==1 else "ADVANCE",
        "release":"RESEARCH_ONLY",
        "signature":"NOT_PRESENT",
        "host_adversary_protection":"NOT_PROVIDED",
        "production":"NOT_AUTHORIZED",
    }


def head_for(entry: dict, entry_sha: str) -> dict:
    return {
        "schema":HEAD_SCHEMA,
        "sequence":entry["sequence"],
        "roster_generation":entry["roster_generation"],
        "roster_sha256":entry["roster_sha256"],
        "roster_state":entry["roster_state"],
        "active_policy_sha256":entry["active_policy_sha256"],
        "latest_entry_sha256":entry_sha,
        "local_durability":LOCAL_DURABILITY,
        "power_loss_certification":"NOT_PROVIDED",
        "host_adversary_protection":"NOT_PROVIDED",
        "production":"NOT_AUTHORIZED",
    }


def verify_transition(previous: dict | None, current: dict, retired: set[str]):
    require(type(current) is dict and set(current)==EVENT_FIELDS
            and current.get("schema")==SCHEMA,
            "ANCHOR_ENTRY_SCHEMA_INVALID")
    seq=current["sequence"]
    require(type(seq) is int and 1<=seq<=MAX_ENTRIES,
            "ANCHOR_SEQUENCE_INVALID")
    require(type(current["roster_generation"]) is int
            and 1<=current["roster_generation"]<=MAX_ROSTER_GENERATION,
            "ANCHOR_GENERATION_INVALID")
    require(current["roster_state"] in ("ACTIVE","FROZEN")
            and type(current["roster_state"]) is str,
            "ANCHOR_STATE_INVALID")
    for key in ("roster_sha256","previous_entry_sha256"):
        value=current[key]
        if value is not None:
            require(type(value) is str
                    and pinned.PIN.fullmatch(value) is not None,
                    "ANCHOR_PIN_FORMAT_INVALID")
    revoked=current["revoked_policy_sha256"]
    require(type(revoked) is list
            and len(revoked)<=roster.MAX_REVOKED
            and all(type(x) is str
                    and pinned.PIN.fullmatch(x) is not None for x in revoked)
            and revoked==sorted(set(revoked)),
            "ANCHOR_REVOKED_LIST_INVALID")
    active=current["active_policy_sha256"]
    if current["roster_state"]=="ACTIVE":
        require(type(active) is str and pinned.PIN.fullmatch(active) is not None,
                "ANCHOR_ACTIVE_POLICY_INVALID")
        require(active not in retired and active not in revoked,
                "ANCHOR_PREVIOUSLY_REVOKED_POLICY_READMITTED")
    else:
        require(active is None,"ANCHOR_FROZEN_HAS_ACTIVE_POLICY")
    require(current["release"]=="RESEARCH_ONLY"
            and current["signature"]=="NOT_PRESENT"
            and current["host_adversary_protection"]=="NOT_PROVIDED"
            and current["production"]=="NOT_AUTHORIZED",
            "ANCHOR_FALSE_AUTHORITY_CLAIM")
    if previous is None:
        require(seq==1 and current["previous_entry_sha256"] is None
                and current["transition"]=="BOOTSTRAP",
                "ANCHOR_BOOTSTRAP_CHAIN_INVALID")
    else:
        require(seq==previous["sequence"]+1
                and current["roster_generation"]>previous["roster_generation"]
                and current["roster_sha256"]!=previous["roster_sha256"]
                and current["transition"]=="ADVANCE",
                "ANCHOR_NONMONOTONE_JOURNAL")
        old=previous["active_policy_sha256"]
        if old is not None and old!=active:
            require(old in revoked,
                    "ANCHOR_OLD_POLICY_NOT_EXPLICITLY_REVOKED")
    retired.update(revoked)


def read_locked(root: Path) -> tuple[dict, list[dict]]:
    private_dir(root,"ANCHOR_ROOT_MISSING_OR_SYMLINKED")
    contents={p.name for p in root.iterdir()}
    require(contents=={"HEAD.json","history",".lock"},
            "ANCHOR_ROOT_FILES_UNEXPECTED_OR_INCOMPLETE")
    history=root/"history"
    private_dir(history,"ANCHOR_HISTORY_NOT_PRIVATE_DIRECTORY")
    records=sorted(p.name for p in history.iterdir())
    require(1<=len(records)<=MAX_ENTRIES,
            "ANCHOR_HISTORY_COUNT_INVALID")
    expected=[f"entry-{i:08d}.json" for i in range(1,len(records)+1)]
    require(records==expected,"ANCHOR_HISTORY_NONCONTIGUOUS_OR_EXTRA")
    prior=None
    last_sha=None
    revoked_all=set()
    events=[]
    for name in expected:
        raw=audit.bounded_regular_file(
            history/name,MAX_ENTRY_BYTES,"ANCHOR_ENTRY_NOT_REGULAR_BOUNDED")
        entry=decode_strict(raw,"ANCHOR_ENTRY_NOT_CANONICAL_JSON")
        verify_transition(prior,entry,revoked_all)
        require(entry["previous_entry_sha256"]==last_sha,
                "ANCHOR_CHAIN_SHA256_MISMATCH")
        last_sha=digest(raw)
        prior=entry
        events.append(entry)
    head_raw=audit.bounded_regular_file(
        root/"HEAD.json",MAX_HEAD_BYTES,"ANCHOR_HEAD_NOT_REGULAR_BOUNDED")
    head=decode_strict(head_raw,"ANCHOR_HEAD_NOT_CANONICAL_JSON")
    require(set(head)==HEAD_FIELDS and head.get("schema")==HEAD_SCHEMA,
            "ANCHOR_HEAD_SCHEMA_INVALID")
    require(head==head_for(events[-1],last_sha),
            "ANCHOR_HEAD_HISTORY_MISMATCH")
    return head,events


def append_locked(root: Path, entry: dict):
    history=root/"history"
    number=entry["sequence"]
    require(number<=MAX_ENTRIES,"ANCHOR_JOURNAL_CAPACITY_EXHAUSTED")
    entry_raw=canonical(entry)
    entry_sha=digest(entry_raw)
    exclusive_file(history/f"entry-{number:08d}.json",entry_raw)
    directory_fsync(history)
    atomic_head(root,canonical(head_for(entry,entry_sha)))


def load_new_roster(roster_file: Path, expected_sha: str,
                    minimum_generation: int):
    return roster.load_roster(roster_file,expected_sha,minimum_generation)


def bootstrap(root: Path, roster_file: Path,
              roster_sha: str, min_generation: int) -> dict:
    check=load_new_roster(roster_file,roster_sha,min_generation)
    require(not root.exists() and not root.is_symlink(),
            "ANCHOR_BOOTSTRAP_ROOT_ALREADY_EXISTS")
    try:
        root.mkdir(mode=0o700)
        (root/"history").mkdir(mode=0o700)
    except FileExistsError:
        raise Denied("ANCHOR_BOOTSTRAP_ROOT_ALREADY_EXISTS") from None
    except OSError:
        raise Unknown("ANCHOR_BOOTSTRAP_DIRECTORY_UNCERTAIN") from None
    external_root(root,roster_file)
    with locked(root):
        entry=record(check,roster_sha,1,None)
        verify_transition(None,entry,set())
        append_locked(root,entry)
        result,_=read_locked(root)
        return result


def advance(root: Path, roster_file: Path,
            roster_sha: str, receiver_minimum: int) -> dict:
    with locked(root):
        current,events=read_locked(root)
        external_root(root,roster_file)
        minimum=max(receiver_minimum,current["roster_generation"]+1)
        next_roster=load_new_roster(roster_file,roster_sha,minimum)
        next_entry=record(next_roster,roster_sha,len(events)+1,
                          current["latest_entry_sha256"])
        retired={value for x in events for value in x["revoked_policy_sha256"]}
        verify_transition(events[-1],next_entry,retired)
        append_locked(root,next_entry)
        result,_=read_locked(root)
        return result


def active_locked(root: Path, roster_file: Path, action: str,
                  policy_file: Path, package: Path,
                  native: Path | None) -> tuple[dict,dict,dict,dict]:
    head,history=read_locked(root)
    external_root(root,roster_file,policy_file,package,native)
    current,policy,manifest=roster.authorize(
        roster_file,head["roster_sha256"],head["roster_generation"],
        policy_file,package,action,native)
    require(current["generation"]==head["roster_generation"]
            and current["state"]==head["roster_state"]
            and current["active_policy_sha256"]==head["active_policy_sha256"],
            "ANCHOR_ROSTER_METADATA_CHANGED")
    return head,current,policy,manifest


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action",choices=(
        "bootstrap","advance","status","inspect","verify","restore","resume"))
    parser.add_argument("--anchor-root",required=True)
    parser.add_argument("--receiver-roster")
    parser.add_argument("--expected-roster-sha256")
    parser.add_argument("--receiver-minimum-generation",type=int,default=1)
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args=parser.parse_args(argv)
    root=Path(args.anchor_root)
    try:
        if args.action in ("bootstrap","advance"):
            require(args.receiver_roster is not None
                    and args.expected_roster_sha256 is not None,
                    "ANCHOR_NEW_ROSTER_AND_EXTERNAL_PIN_REQUIRED")
            roster_file=Path(args.receiver_roster)
            if args.action=="bootstrap":
                head=bootstrap(root,roster_file,args.expected_roster_sha256,
                               args.receiver_minimum_generation)
            else:
                head=advance(root,roster_file,args.expected_roster_sha256,
                             args.receiver_minimum_generation)
            emit({"schema":RESULT,
                  "status":"BOOTSTRAPPED" if args.action=="bootstrap" else "ADVANCED",
                  "sequence":head["sequence"],
                  "roster_generation":head["roster_generation"],
                  "roster_sha256":head["roster_sha256"],
                  "roster_state":head["roster_state"],
                  "local_durability":LOCAL_DURABILITY,
                  "power_loss_certification":"NOT_PROVIDED",
                  "host_adversary_protection":"NOT_PROVIDED",
                  "production":"NOT_AUTHORIZED"})
            return 0
        with locked(root):
            head,_=read_locked(root)
            if args.action=="status":
                emit({"schema":RESULT,"status":head["roster_state"],
                      "sequence":head["sequence"],
                      "roster_generation":head["roster_generation"],
                      "roster_sha256":head["roster_sha256"],
                      "local_journal":"SHA256_CHAIN_VALID",
                      "native_engine":"NOT_EXECUTED",
                      "power_loss_certification":"NOT_PROVIDED",
                      "host_adversary_protection":"NOT_PROVIDED",
                      "production":"NOT_AUTHORIZED"})
                return 0
            require(args.receiver_roster is not None
                    and args.receiver_policy is not None
                    and args.portable_dir is not None,
                    "ANCHOR_INPUTS_REQUIRED")
            policy_file=Path(args.receiver_policy)
            package=Path(args.portable_dir)
            roster_file=Path(args.receiver_roster)
            native=Path(args.restore_root) if args.restore_root else None
            if args.action in ("restore","resume"):
                require(native is not None,"ANCHOR_NATIVE_ROOT_REQUIRED")
            checked,roster_obj,policy,manifest=active_locked(
                root,roster_file,args.action,policy_file,package,native)
            if args.action=="inspect":
                emit({"schema":RESULT,"status":"POLICY_MATCH",
                      "roster_generation":checked["roster_generation"],
                      "roster_sha256":checked["roster_sha256"],
                      "policy_id":policy["policy_id"],
                      "native_jobs_held":manifest["committed"],
                      "native_engine":"NOT_EXECUTED",
                      "production":"NOT_AUTHORIZED"})
                return 0
            require(args.engine is not None and args.source_root is not None,
                    "ANCHOR_ENGINE_SOURCE_REQUIRED")
            engine,source=Path(args.engine),Path(args.source_root)
            proof=roster.previous.admitted_replay(policy,package,engine,source)
            if args.action=="verify":
                emit({"schema":RESULT,"status":"PASS",
                      "roster_generation":checked["roster_generation"],
                      "journal_sequence":checked["sequence"],
                      "native_jobs_recomputed":proof["native_jobs_recomputed"],
                      "native_files_bytewise_equal":
                          proof["native_files_bytewise_equal"],
                      "native_receipts_exactly_equal":
                          proof["native_receipts_exactly_equal"],
                      "local_journal":"SHA256_CHAIN_VALID",
                      "signature":"NOT_PRESENT",
                      "power_loss_certification":"NOT_PROVIDED",
                      "host_adversary_protection":"NOT_PROVIDED",
                      "production":"NOT_AUTHORIZED"})
                return 0

            # Lock held across replay, policy recheck, and real native effects:
            # another APP-018 process cannot rotate the live local anchor.
            after,new_roster,new_policy,_=active_locked(
                root,roster_file,args.action,policy_file,package,native)
            require(after==checked and new_policy==policy
                    and new_roster==roster_obj,
                    "ANCHOR_OR_POLICY_CHANGED_DURING_REPLAY")
            if args.action=="restore":
                receipt=portable.restore(package,policy["portable_sha256"],native)
                receipt["journal_sequence"]=checked["sequence"]
                receipt["roster_generation"]=checked["roster_generation"]
                receipt["local_freshness"]="VERIFIED_BEFORE_RESTORE"
                emit(receipt)
                return 0
            return portable.resume_native(
                package,policy["portable_sha256"],native,engine,source,
                policy["engine_sha256"],policy["source_tree_sha256"])
    except (Unknown,roster.previous.replay.Uncertain,
            portable.Unknown) as e:
        emit({"schema":RESULT,"status":"INCONCLUSIVE",
              "error_code":e.code,"publication":"UNKNOWN"})
        return 3
    except (Denied,roster.Denied,roster.previous.Denied,
            roster.previous.replay.Reject,portable.Refused,
            portable.inner.Refused,pinned.Denied,audit.AuditRejected,
            ValueError,OSError) as e:
        emit({"schema":RESULT,"status":"REJECTED",
              "error_code":getattr(e,"code","ANCHOR_INPUT_REJECTED"),
              "publication":"NOT_STARTED"})
        return 2


if __name__=="__main__":
    raise SystemExit(main())
