#!/usr/bin/env python3
"""APP-022: live SSHSIG freshness witness + APP-021 real native agent replay.

A witness on 127.0.0.1 runs as a separate process and signs a fresh random
challenge. The receiver pins an independent witness public key, verifies the
signed witness's exact APP-021 authority chain and current journal HEAD, and
requires witness epoch, sequence, generation and counter floors. For native
restore/resume it obtains a SECOND live attestation after real Rust replay,
before any effect. Fail closed if the witness or the signed current state
changes.

The CI laboratory uses SAME-ADMIN local keys and storage. It does NOT prove
separate custody, externally authenticated identity, persistent independent
monotonicity across process restart, host compromise resistance, global
antirollback, TPM/TEE, third-party certification or production readiness.
"""
from __future__ import annotations

import argparse
import base64
import binascii
import http.client
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"app021_authority_rotation"))
import receiver_app021 as previous
from witness_app022 import (CHALLENGE_SCHEMA,CHALLENGE_FIELDS,
    ATTEST_SCHEMA,RESPONSE_SCHEMA,NAMESPACE)

anchor=previous.anchor
roster=previous.roster
portable=previous.portable
pinned=previous.pinned
signed=previous.signed
audit=previous.audit
Denied=previous.Denied
Unknown=previous.Unknown
require=previous.require

RESULT_SCHEMA="NORDOI_APP022_LIVE_WITNESS_RECEIVER_RESULT_V1"
ACTIONS=previous.ACTIONS
URI=re.compile(r"http://127\.0\.0\.1:([0-9]{1,5})/attest\Z",re.ASCII)
PIN=re.compile(r"[a-f0-9]{64}\Z",re.ASCII)
STATEMENT_FIELDS=frozenset(("schema","nonce","authority_id","receiver_id",
 "chain_sha256","head_sha256","epoch","minimum_sequence",
 "minimum_generation","counter","release","production"))
RESPONSE_FIELDS=frozenset(("schema","statement","signature_b64","production"))


def witness_pubkey(args,root):
    require(type(args.expected_witness_key_sha256) is str
            and PIN.fullmatch(args.expected_witness_key_sha256) is not None,
            "WITNESS_EXPECTED_PUBLIC_KEY_PIN_INVALID")
    require(args.witness_public_key is not None,
            "WITNESS_PUBLIC_KEY_REQUIRED")
    path=Path(args.witness_public_key)
    package=Path(args.portable_dir) if args.portable_dir else None
    dest=Path(args.restore_root) if args.restore_root else None
    signed.nonoverlap(root,package,dest,[path])
    pub=audit.bounded_regular_file(
        path,signed.MAX_PUBLIC_KEY,"WITNESS_PUBLIC_KEY_NOT_REGULAR_BOUNDED")
    require(anchor.digest(pub)==args.expected_witness_key_sha256,
            "WITNESS_PUBLIC_KEY_PIN_MISMATCH")
    signed.check_pubkey(path,args.expected_witness_key_sha256)
    return pub


def live_attestation(args,root,head,chain,event,public):
    require(type(args.witness_url) is str and URI.fullmatch(args.witness_url)
            is not None,"WITNESS_URL_LOCAL_LOOPBACK_REQUIRED")
    port=int(URI.fullmatch(args.witness_url).group(1))
    require(1<=port<65536,"WITNESS_PORT_INVALID")
    require(type(args.minimum_witness_counter) is int
            and 1<=args.minimum_witness_counter<=2**31-1,
            "WITNESS_COUNTER_RECEIVER_FLOOR_INVALID")
    pub=witness_pubkey(args,root)
    nonce=os.urandom(32).hex()
    challenge={"schema":CHALLENGE_SCHEMA,"nonce":nonce,
               "authority_id":args.authority_id,"receiver_id":args.receiver_id}
    conn=http.client.HTTPConnection("127.0.0.1",port,timeout=4)
    try:
        conn.request("POST","/attest",body=anchor.canonical(challenge),
                     headers={"Content-Type":"application/json",
                              "Cache-Control":"no-store"})
        answer=conn.getresponse()
        raw=answer.read(65537)
        require(answer.status==200,"WITNESS_LIVE_SIGNER_REFUSED")
        require(0<len(raw)<=65536,"WITNESS_LIVE_RESPONSE_SIZE_INVALID")
    except (OSError, TimeoutError, http.client.HTTPException):
        raise Denied("WITNESS_LIVE_SIGNER_UNAVAILABLE") from None
    finally:
        conn.close()
    response=anchor.decode_strict(raw,"WITNESS_RESPONSE_NOT_CANONICAL")
    require(type(response) is dict and set(response)==RESPONSE_FIELDS
            and response.get("schema")==RESPONSE_SCHEMA
            and response.get("production")=="NOT_AUTHORIZED"
            and type(response.get("statement")) is dict
            and set(response["statement"])==STATEMENT_FIELDS,
            "WITNESS_SIGNED_RESPONSE_SCHEMA_INVALID")
    statement=response["statement"]
    require(statement.get("schema")==ATTEST_SCHEMA
            and statement.get("release")=="RESEARCH_ONLY"
            and statement.get("production")=="NOT_AUTHORIZED"
            and statement.get("nonce")==nonce
            and statement.get("authority_id")==args.authority_id
            and statement.get("receiver_id")==args.receiver_id,
            "WITNESS_FRESH_CHALLENGE_OR_SCOPE_INVALID")
    require(all(type(statement.get(k)) is str and
                PIN.fullmatch(statement[k]) is not None
                for k in ("chain_sha256","head_sha256")),
            "WITNESS_SIGNED_HASH_INVALID")
    require(all(type(statement.get(k)) is int and
                1<=statement[k]<=2**31-1
                for k in ("epoch","minimum_sequence",
                          "minimum_generation","counter")),
            "WITNESS_SIGNED_FLOOR_INVALID")
    signature=response["signature_b64"]
    require(type(signature) is str and len(signature)<=16384,
            "WITNESS_SIGNATURE_ENCODING_INVALID")
    try:
        sig=base64.b64decode(signature,validate=True)
    except (ValueError,binascii.Error):
        raise Denied("WITNESS_SIGNATURE_ENCODING_INVALID") from None
    require(base64.b64encode(sig).decode("ascii")==signature
            and sig.startswith(b"-----BEGIN SSH SIGNATURE-----\n")
            and sig.rstrip().endswith(b"-----END SSH SIGNATURE-----"),
            "WITNESS_SIGNATURE_ENCODING_INVALID")
    binary=shutil.which("ssh-keygen")
    require(binary is not None,"WITNESS_SSHSIG_VERIFIER_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-witness-verify-") as tmp:
        rootdir=Path(tmp)
        signers=rootdir/"allowed_signers"
        file=rootdir/"witness.sshsig"
        signers.write_bytes(args.authority_id.encode("ascii")+b" "+pub)
        file.write_bytes(sig)
        try:
            verified=subprocess.run(
                [binary,"-Y","verify","-f",str(signers),
                 "-I",args.authority_id,"-n",NAMESPACE,
                 "-s",str(file)],input=anchor.canonical(statement),
                capture_output=True,timeout=20)
        except (OSError,subprocess.TimeoutExpired):
            raise Denied("WITNESS_SSHSIG_VERIFIER_UNAVAILABLE") from None
        require(verified.returncode==0,
                "WITNESS_SSHSIG_ED25519_VERIFICATION_FAILED")
    require(statement["chain_sha256"]==args.expected_chain_sha256
            and statement["head_sha256"]==anchor.digest(anchor.canonical(head)),
            "WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH")
    require(statement["epoch"]==event["epoch"]
            and statement["minimum_sequence"]>=max(
                event["minimum_sequence"],args.minimum_sequence)
            and statement["minimum_generation"]>=max(
                event["minimum_generation"],args.minimum_generation)
            and statement["minimum_sequence"]<=head["sequence"]
            and statement["minimum_generation"]<=head["roster_generation"]
            and statement["counter"]>=args.minimum_witness_counter,
            "WITNESS_CURRENT_FRESHNESS_FLOOR_MISMATCH")
    return statement


def witness_scope(statement):
    return {k:v for k,v in statement.items() if k!="nonce"}


def result(status,head,event,args,key_pin,statement,**extra):
    return {**previous.status_output(status,head,event,args,key_pin),
            "schema":RESULT_SCHEMA,
            "live_witness":"NEW_NONCE_SSHSIG_ED25519_VERIFIED",
            "witness_counter":statement["counter"],
            "witness_epoch":statement["epoch"],
            "witness_process":"LOOPBACK_SEPARATE_PROCESS",
            "witness_key_custody":"SAME_ADMIN_LAB_NOT_INDEPENDENT",
            "persistent_global_antirollback":"NOT_PROVEN",
            "native_admission":"RESEARCH_ONLY",
            **extra}


def call_signed(args,root,key,key_pin,seq,gen,statement):
    with tempfile.TemporaryDirectory(prefix="nordoi-app022-active-") as tmp:
        public_file=Path(tmp)/"active.key"
        public_file.write_bytes(key)
        old=SimpleNamespace(**vars(args))
        old.trusted_public_key=str(public_file)
        old.expected_public_key_sha256=key_pin
        old.minimum_sequence=seq
        old.minimum_generation=gen
        head,payload=signed.verified_signed_head(old,root)
        if args.action=="status":
            anchor.emit(result(head["roster_state"],head,
                args._authority_event,args,key_pin,statement,
                crypto_signature="SSHSIG_ED25519_VERIFIED",
                native_engine="NOT_EXECUTED"))
            return 0
        require(head["roster_state"]=="ACTIVE",
                "WITNESS_FROZEN_NATIVE_ADMISSION_DENIED")
        if args.action in ("inspect","verify"):
            return signed.native(old,root,head,payload)
        require(args.receiver_roster and args.receiver_policy and
                args.portable_dir and args.restore_root and
                args.engine and args.source_root,
                "WITNESS_NATIVE_INPUTS_REQUIRED")
        roster_file=Path(args.receiver_roster)
        policy_file=Path(args.receiver_policy)
        package=Path(args.portable_dir)
        native_root=Path(args.restore_root)
        before,old_roster,old_policy,_=anchor.active_locked(
            root,roster_file,args.action,policy_file,package,native_root)
        require(before==head,"WITNESS_HEAD_CHANGED_BEFORE_REPLAY")
        engine,source=Path(args.engine),Path(args.source_root)
        roster.previous.admitted_replay(old_policy,package,engine,source)
        again,event,pub,again_pin=previous.chain_file(args,root)
        require(again==args._authority_chain and
                event==args._authority_event and
                pub==key and again_pin==key_pin,
                "WITNESS_AUTHORITY_CHANGED_DURING_NATIVE_REPLAY")
        fresh=live_attestation(args,root,head,again,event,key)
        require(witness_scope(fresh)==witness_scope(statement),
                "WITNESS_STATE_CHANGED_DURING_NATIVE_REPLAY")
        new_head,new_payload=signed.verified_signed_head(old,root)
        after,new_roster,new_policy,_=anchor.active_locked(
            root,roster_file,args.action,policy_file,package,native_root)
        require(new_head==head and new_payload==payload and after==head
                and new_roster==old_roster and new_policy==old_policy,
                "WITNESS_POLICY_CHANGED_BEFORE_EFFECT")
        if args.action=="restore":
            receipt=portable.restore(package,old_policy["portable_sha256"],native_root)
            receipt["witness_counter"]=fresh["counter"]
            receipt["fresh_witness_sshsig"]="VERIFIED_TWICE_BEFORE_NATIVE_EFFECT"
            receipt["witness_key_custody"]="SAME_ADMIN_LAB_NOT_INDEPENDENT"
            receipt["production"]="NOT_AUTHORIZED"
            anchor.emit(receipt)
            return 0
        return portable.resume_native(
            package,old_policy["portable_sha256"],native_root,
            engine,source,old_policy["engine_sha256"],
            old_policy["source_tree_sha256"])


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action",choices=ACTIONS)
    parser.add_argument("--anchor-root",required=True)
    parser.add_argument("--authority-chain-file",required=True)
    parser.add_argument("--expected-chain-sha256")
    parser.add_argument("--expected-root-key-sha256")
    parser.add_argument("--authority-id",required=True)
    parser.add_argument("--receiver-id",required=True)
    parser.add_argument("--minimum-epoch",type=int,required=True)
    parser.add_argument("--minimum-sequence",type=int,required=True)
    parser.add_argument("--minimum-generation",type=int,required=True)
    parser.add_argument("--witness-url",required=True)
    parser.add_argument("--witness-public-key",required=True)
    parser.add_argument("--expected-witness-key-sha256")
    parser.add_argument("--minimum-witness-counter",type=int,required=True)
    parser.add_argument("--signed-statement")
    parser.add_argument("--detached-signature")
    parser.add_argument("--receiver-roster")
    parser.add_argument("--receiver-policy")
    parser.add_argument("--portable-dir")
    parser.add_argument("--engine")
    parser.add_argument("--source-root")
    parser.add_argument("--restore-root")
    args=parser.parse_args(argv)
    root=Path(args.anchor_root)
    try:
        with anchor.locked(root):
            chain,event,key,key_pin=previous.chain_file(args,root)
            head,_=anchor.read_locked(root)
            require(head["sequence"]>=max(event["minimum_sequence"],
                                          args.minimum_sequence)
                    and head["roster_generation"]>=max(
                        event["minimum_generation"],args.minimum_generation),
                    "WITNESS_JOURNAL_BELOW_AUTHORITY_FLOOR")
            statement=live_attestation(args,root,head,chain,event,key)
            if event["state"]=="FROZEN":
                require(head["roster_state"]=="FROZEN",
                        "WITNESS_FROZEN_ROSTER_MISMATCH")
                if args.action!="status":
                    raise Denied("WITNESS_FROZEN_NATIVE_ADMISSION_DENIED")
                anchor.emit(result("FROZEN",head,event,args,None,statement,
                    crypto_signature="WITNESS_AND_AUTHORITY_SSHSIG_VERIFIED",
                    native_engine="NOT_EXECUTED"))
                return 0
            require(key is not None and key_pin is not None,
                    "WITNESS_ACTIVE_AUTHORITY_KEY_REQUIRED")
            args._authority_chain=chain
            args._authority_event=event
            return call_signed(args,root,key,key_pin,
                max(event["minimum_sequence"],args.minimum_sequence),
                max(event["minimum_generation"],args.minimum_generation),
                statement)
    except (Unknown,roster.previous.replay.Uncertain,portable.Unknown) as exc:
        anchor.emit({"schema":RESULT_SCHEMA,"status":"INCONCLUSIVE",
            "error_code":exc.code,"publication":"UNKNOWN",
            "production":"NOT_AUTHORIZED"})
        return 3
    except (Denied,roster.Denied,roster.previous.Denied,
            roster.previous.replay.Reject,portable.Refused,
            portable.inner.Refused,pinned.Denied,audit.AuditRejected,
            ValueError,OSError) as exc:
        anchor.emit({"schema":RESULT_SCHEMA,"status":"REJECTED",
            "error_code":getattr(exc,"code","WITNESS_INPUT_REJECTED"),
            "publication":"NOT_STARTED","production":"NOT_AUTHORIZED"})
        return 2


if __name__=="__main__":
    raise SystemExit(main())
