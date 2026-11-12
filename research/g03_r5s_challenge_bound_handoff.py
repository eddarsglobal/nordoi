#!/usr/bin/env python3
"""G0.3 R5S: genuine offline evidence handoff envelope, deny-only.

Content-addressed, challenge-bound archival transfer to a second real process
plus local spent-challenge refusal. An authentic *independent* custodian, owner
OOB root, secure clock and external monotonic high-water do NOT exist here.
Even valid transfer verification NEVER grants authority.
"""
import argparse
from contextlib import closing
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
import zipfile

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5m_file_integrity_quarantine as witness
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history

PROTOCOL="NORDOI-G03-R5S-CHALLENGE-BOUND-HANDOFF-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="e9b10fbc0ec51f243399ea636c5d2b3fbcc3c6eb"
OPERATION="OFFLINE_EVIDENCE_TRANSPORT_DENY_ONLY"
ASSETS=("evidence/primary.sqlite3","evidence/local_witness.json",
        "evidence/spent_nonces.sqlite3")
ENTRIES=("manifest.json",)+ASSETS
MAX_ZIP_BYTES=2*1024*1024
MAX_PAYLOAD_BYTES=512*1024
MAX_MANIFEST_BYTES=4096
MAX_TTL_SECONDS=300
MANIFEST_FIELDS=frozenset(("protocol","project","parent","operation",
                           "challenge","created_at","expires_at",
                           "projection_sha256","asset_sha256"))

class Refused(ValueError):pass

def sha(b):return hashlib.sha256(b).hexdigest()
def issha(s):return type(s) is str and re.fullmatch(r"[0-9a-f]{64}",s) is not None
def canonical(doc):
    return (json.dumps(doc,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")
def pairs(p):
    d={}
    for k,v in p:
        if k in d:raise Refused("DUPLICATE_MANIFEST_FIELD")
        d[k]=v
    return d
def nonfinite(_):raise Refused("NONFINITE_MANIFEST")
def block(reason,**kw):
    return denial.block(reason,native_authority_granted=False,
                        independent_custodian="NOT_ENROLLED",
                        owner_out_of_band_root="NOT_ENROLLED",
                        external_monotonic_history="NOT_PROVEN",
                        **kw)

def manifest_parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_MANIFEST_BYTES:
        raise Refused("MANIFEST_MISSING_OR_OVERSIZED")
    try:
        obj=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs,
                       parse_constant=nonfinite)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("MANIFEST_UTF8_OR_JSON_INVALID") from exc
    if type(obj) is not dict or obj.keys()!=MANIFEST_FIELDS or canonical(obj)!=raw:
        raise Refused("MANIFEST_NOT_CANONICAL_OR_FIELDS")
    if (obj["protocol"],obj["project"],obj["parent"],obj["operation"])!=(
        PROTOCOL,PROJECT,PARENT,OPERATION):
        raise Refused("MANIFEST_LINEAGE_OR_OPERATION_INVALID")
    if not issha(obj["challenge"]) or not issha(obj["projection_sha256"]):
        raise Refused("CHALLENGE_OR_PROJECTION_NOT_SHA256")
    hashes=obj["asset_sha256"]
    if type(hashes) is not dict or hashes.keys()!=set(ASSETS) or not all(issha(v) for v in hashes.values()):
        raise Refused("EXACT_THREE_ASSET_HASHES_REQUIRED")
    for name in ("created_at","expires_at"):
        if type(obj[name]) is not int or obj[name]<0:
            raise Refused("INVALID_TIME_FIELD_"+name)
    if not 1<=obj["expires_at"]-obj["created_at"]<=MAX_TTL_SECONDS:
        raise Refused("UNSAFE_LOCAL_CLAIMED_TTL")
    return obj

def evidence_bytes(a,b,j):
    current=history.projection(a,b,j)
    sources=history.paths(a,b,j)
    blobs={}
    for src,key in zip(sources,ASSETS):
        with fdgate.OpenSnapshot(src) as f:
            data=src.read_bytes()
            if len(data)>MAX_PAYLOAD_BYTES or sha(data)!=f.digest:
                raise Refused("LIVE_EVIDENCE_CHANGED_OR_OVERSIZED_"+key)
            f.check()
            blobs[key]=data
    newer=history.projection(a,b,j)
    if newer!=current:raise Refused("LIVE_EVIDENCE_CHANGED_BETWEEN_PROJECTIONS")
    if sha(blobs[ASSETS[0]])!=current["primary_db_sha256"] or \
       sha(blobs[ASSETS[1]])!=current["witness_bytes_sha256"]:
        raise Refused("PRIMARY_WITNESS_HASHES_DISAGREE")
    return current,blobs

def bytes_zip(manifest,blobs):
    mem=io.BytesIO()
    with zipfile.ZipFile(mem,"w") as z:
        for name,data in (("manifest.json",canonical(manifest)),)+tuple(
                (key,blobs[key]) for key in ASSETS):
            info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0))
            info.compress_type=zipfile.ZIP_STORED
            info.external_attr=0o600<<16
            z.writestr(info,data)
    result=mem.getvalue()
    if len(result)>MAX_ZIP_BYTES:raise Refused("TRANSFER_ENVELOPE_TOO_LARGE")
    return result

def prepare(a,b,j,challenge,created_at,expiry,output):
    """Export only to fresh private disposable R5S lab path; no network use."""
    if not issha(challenge):raise Refused("INVALID_EXPORT_CHALLENGE")
    base=Path(output)
    if not base.is_absolute() or base.is_symlink() or base.exists() or \
       not any(x.startswith("nordoi-r5l-r5s-") for x in base.parts) or \
       not base.parent.is_dir():
        raise Refused("ONLY_NEW_DISPOSABLE_LAB_EXPORT_ALLOWED")
    if os.name!="nt" and base.parent.stat().st_mode&0o077:
        raise Refused("TRANSFER_FOLDER_NOT_PRIVATE")
    if type(created_at) is not int or type(expiry) is not int:
        raise Refused("UNTRUSTED_CLOCK_TYPE")
    p,assets=evidence_bytes(a,b,j)
    m={"protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
       "operation":OPERATION,"challenge":challenge,"created_at":created_at,
       "expires_at":expiry,"projection_sha256":history.digest_projection(p),
       "asset_sha256":{k:sha(v) for k,v in assets.items()}}
    manifest_parse(canonical(m))
    encoded=bytes_zip(m,assets)
    # Exclusive new output only; never replace real user files or credentials.
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL|getattr(os,"O_CLOEXEC",0)
    fd=os.open(str(base),flags,0o600)
    try:
        with os.fdopen(fd,"wb") as stream:
            stream.write(encoded)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        base.unlink(missing_ok=True)
        raise
    return block("LOCAL_HANDOFF_ENVELOPE_EXPORTED_NOT_CUSTODY",
                 envelope_sha256=sha(encoded),
                 projection_sha256=m["projection_sha256"],
                 local_export_only=True)

def decode_package(binary,expected_challenge,now):
    if not issha(expected_challenge):raise Refused("RECEIVER_CHALLENGE_INVALID")
    if type(now) is not int or now<0:raise Refused("UNTRUSTED_CLOCK_INPUT")
    if not 0<len(binary)<=MAX_ZIP_BYTES:raise Refused("BAD_TRANSFER_ENVELOPE_SIZE")
    try:
        with zipfile.ZipFile(io.BytesIO(binary),"r") as z:
            items=z.infolist()
            if len(items)!=len(ENTRIES) or {x.filename for x in items}!=set(ENTRIES):
                raise Refused("ARCHIVE_MUST_CONTAIN_EXACT_FOUR_FIXED_NAMES")
            if any(x.filename not in ENTRIES or x.is_dir() or
                   x.compress_type!=zipfile.ZIP_STORED or
                   x.flag_bits&1 or x.file_size>MAX_PAYLOAD_BYTES for x in items):
                raise Refused("MALFORMED_OR_COMPRESSED_ARCHIVE_ENTRY")
            manifest=manifest_parse(z.read("manifest.json"))
            blobs={name:z.read(name) for name in ASSETS}
    except (zipfile.BadZipFile,RuntimeError,EOFError,KeyError) as exc:
        raise Refused("TRANSFER_ZIP_INVALID:"+type(exc).__name__) from exc
    if manifest["challenge"]!=expected_challenge:
        raise Refused("SIGNED_RECEIVER_CHALLENGE_MISMATCH")
    if not manifest["created_at"]<=now<=manifest["expires_at"]:
        raise Refused("LOCAL_UNTRUSTED_TIMEBOX_EXPIRED_OR_FUTURE")
    if any(sha(data)!=manifest["asset_sha256"][key] for key,data in blobs.items()):
        raise Refused("ENCLOSED_FILE_SHA256_MISMATCH")
    # Verify real SQL data from the received bytes in a fresh isolated private
    # directory; no extraction into the actual app checkout or owner state.
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-receiver-") as tmp:
        root=Path(tmp)
        if os.name!="nt":os.chmod(root,0o700)
        a,b,j=[root/n for n in ("primary","witness","nonce")]
        for d in (a,b,j):d.mkdir(mode=0o700)
        for path,data in zip((a/denial.NAME,b/witness.WITNESS,j/denial.NAME),
                             (blobs[k] for k in ASSETS)):
            path.write_bytes(data)
            if os.name!="nt":os.chmod(path,0o600)
        try:p=history.projection(a,b,j)
        except (history.Refused,witness.Refused,denial.Refused,
                sqlite3.Error,OSError) as exc:
            raise Refused("RECEIVED_SQLITE_JSON_HISTORY_INVALID:"+type(exc).__name__) from exc
    if history.digest_projection(p)!=manifest["projection_sha256"]:
        raise Refused("RECEIVED_APPLICATION_HISTORY_PROJECTION_MISMATCH")
    return manifest,sha(binary),p

def inspect(path,expected_challenge,now,ledger_dir=None,after_open=None):
    try:
        p=Path(path)
        with fdgate.OpenSnapshot(p) as f:
            raw=p.read_bytes()
            if sha(raw)!=f.digest:
                raise Refused("TRANSFER_FILE_CHANGED_WHILE_READING")
            if after_open is not None:
                if not callable(after_open):raise Refused("INVALID_INTERLEAVE_HOOK")
                after_open()
            f.check()
        manifest,package_sha,state=decode_package(raw,expected_challenge,now)
        if ledger_dir is not None:
            existing=denial.private_journal_dir(ledger_dir)
            if not existing.is_file():
                raise Refused("RECEIVER_LOCAL_NONCE_LEDGER_MISSING_NO_SILENT_GENESIS")
            used=denial.execute(ledger_dir,"record",manifest["challenge"],
                                package_sha,"candidate_a")
            if used.get("reason")!="LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION":
                raise Refused("RECEIVER_CHALLENGE_ALREADY_SPENT_OR_LEDGER_REFUSED:"+
                              str(used.get("detail",used.get("reason"))))
            height=used["local_height"]
        else:height=None
        return block("HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY",
                     received_archive_sha256=package_sha,
                     received_history_sha256=manifest["projection_sha256"],
                     received_height=state["primary_height"],
                     receiver_local_denial_height=height,
                     external_guardian_accepted=False,
                     external_time_authority_verified=False,
                     independent_custody_verified=False)
    except (Refused,history.Refused,fdgate.Refused,witness.Refused,
            denial.Refused,OSError,TypeError,ValueError,sqlite3.Error) as exc:
        return block("HANDOFF_TRANSFER_QUARANTINED",
                     detail=str(exc) if isinstance(exc,(Refused,history.Refused,
                             fdgate.Refused,witness.Refused,denial.Refused))
                     else type(exc).__name__,
                     independent_custody_verified=False)

def second_process_inspection(path,nonce,now,ledger_dir=None):
    cmd=[sys.executable,"-B",str(Path(__file__).resolve()),"--inspect",
         "--bundle",str(path),"--challenge",nonce,"--now",str(now)]
    if ledger_dir is not None:cmd+=["--ledger-dir",str(ledger_dir)]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=45,check=False)
    if p.returncode!=2:raise Refused("SECOND_PROCESS_NOT_FAIL_CLOSED")
    return json.loads(p.stdout)

def native_exercise():
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5s-native-") as tmp:
        root=Path(tmp)
        if os.name!="nt":os.chmod(root,0o700)
        a,b,j=history.fixture(root)
        rec=root/"receiver_nonce"
        rec.mkdir(mode=0o700)
        with denial.DenialJournal(rec) as ledger:
            if ledger.audit()["local_height"]!=0:
                raise Refused("RECEIVER_LEDGER_GENESIS_FAILED")
        safe0=root/"genesis_nonce.sqlite3"
        shutil.copy2(rec/denial.NAME,safe0)
        path=root/"evidence_transfer.zip"
        sent=prepare(a,b,j,"a"*64,1000,1200,path)
        if sent["reason"]!="LOCAL_HANDOFF_ENVELOPE_EXPORTED_NOT_CUSTODY":
            raise Refused("REAL_ENVELOPE_EXPORT_FAILED")
        receipt=second_process_inspection(path,"a"*64,1100,rec)
        if receipt.get("reason")!="HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY":
            raise Refused("REAL_CHILD_RECEIVER_FAILED")
        replay=second_process_inspection(path,"a"*64,1100,rec)
        if "ALREADY_RECORDED_DENIAL_NONCE" not in replay.get("detail",""):
            raise Refused("SAME_LOCAL_CHALLENGE_REPLAY_NOT_DENIED")
        if "RECEIVER_CHALLENGE_MISMATCH" not in inspect(path,"b"*64,1100).get("detail",""):
            raise Refused("WRONG_CHALLENGE_NOT_REFUSED")
        if "TIMEBOX_EXPIRED" not in inspect(path,"a"*64,1300).get("detail",""):
            raise Refused("EXPIRED_UNTRUSTED_LOCAL_CLOCK_NOT_REFUSED")
        dup=root/"unchanged_copy.zip"
        shutil.copy2(path,dup)
        if os.name!="nt":os.chmod(dup,0o600)
        swap=inspect(path,"a"*64,1100,
                     after_open=lambda:fdgate.real_atomic_replace(dup,path))
        if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in swap.get("detail",""):
            raise Refused("REAL_SECOND_PROCESS_ATOMIC_BUNDLE_SWAP_NOT_BLOCKED")
        history.advance_world(a,b,j)
        old=second_process_inspection(path,"a"*64,1100)
        if old.get("reason")!="HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY":
            raise Refused("OLD_SELF_CONSISTENT_BUNDLE_MUST_EXPOSE_NO_HIGHWATER")
        if history.projection(a,b,j)["primary_height"]!=2 or old["received_height"]!=1:
            raise Refused("REAL_HISTORICAL_FORK_NOT_DEMONSTRATED")
        # Roll back local challenge ledger via a different true OS process.
        fdgate.real_atomic_replace(safe0,rec/denial.NAME)
        repeated=second_process_inspection(path,"a"*64,1100,rec)
        if repeated.get("reason")!="HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY":
            raise Refused("REAL_LOCAL_LEDGER_ROLLBACK_COUNTEREXAMPLE_NOT_OBSERVED")
        if repeated.get("receiver_local_denial_height")!=1:
            raise Refused("REPLAY_AFTER_ROLLBACK_NOT_LOCAL_DENIAL_ONLY")
        return block("REAL_TRANSFER_WITHOUT_INDEPENDENT_CUSTODY_QUALIFIED",
                     real_second_process_verified_integrity=True,
                     challenge_replay_same_local_ledger="QUARANTINED",
                     challenge_replay_after_local_ledger_rollback="LOCAL_DENIAL_AGAIN",
                     expired_local_timebox="QUARANTINED",
                     wrong_receiver_challenge="QUARANTINED",
                     real_second_process_bundle_inode_swap="QUARANTINED",
                     older_self_consistent_bundle_after_real_world_advance="STILL_LOCALLY_VERIFIABLE",
                     independent_receiver_custody="NOT_ENROLLED",
                     actual_external_signed_receipt="NONE",
                     global_protected_spent_nonce_floor="NOT_IMPLEMENTED")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise",action="store_true")
    g.add_argument("--inspect",action="store_true")
    p.add_argument("--bundle")
    p.add_argument("--challenge")
    p.add_argument("--now",type=int)
    p.add_argument("--ledger-dir")
    args=p.parse_args(argv)
    if args.exercise:
        try:out=native_exercise()
        except (Refused,history.Refused,fdgate.Refused,witness.Refused,denial.Refused,
                OSError,sqlite3.Error,subprocess.SubprocessError,ValueError) as exc:
            out=block("REAL_HANDOFF_EXERCISE_REFUSED",detail=str(exc))
    else:
        out=inspect(args.bundle,args.challenge,args.now,args.ledger_dir)
    print(json.dumps(out,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
