#!/usr/bin/env python3
"""NORDOI G0.3 R5K: actual concurrent signed recovery intents, same-admin TOCTOU.

Uses real OpenSSH Ed25519 temporary keys + real R5H/R5I SQLite files.
No independent guardian. No permission grant, recovery or production route.
Every assessment and CLI output is BLOCK / NOT_AUTHORIZED.
"""
import argparse
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import g03_r5h_signed_revocation_lab as rev
import g03_r5i_revocation_floor as floor
import g03_r5j_signed_recovery_quarantine as rec

PROTOCOL="NORDOI-G03-R5K-CONCURRENT-RECOVERY-QUARANTINE-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="b348fac65dbe91dec958878f868fe47118a68f44"
MAX_BYTES=28000
FIELDS=frozenset(("protocol","project","parent_commit","intents"))
ITEM_FIELDS=frozenset(("role","intent_b64","intent_sha256"))
ROLES=("candidate_a","candidate_b")

class Refused(ValueError): pass

def hashbytes(value):return hashlib.sha256(value).hexdigest()

def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True,
                       allow_nan=False)+"\n").encode("ascii")

def unique(pairs):
    result={}
    for k,v in pairs:
        if k in result:raise Refused("DUPLICATE_JSON_KEY")
        result[k]=v
    return result

def no_constant(_):raise Refused("NONFINITE_JSON_NUMBER")

def envelope(packet_a,packet_b):
    def item(role,packet):
        return {"role":role,"intent_b64":base64.b64encode(packet).decode("ascii"),
                "intent_sha256":hashbytes(packet)}
    return canonical({"protocol":PROTOCOL,"project":PROJECT,"parent_commit":PARENT,
                      "intents":[item("candidate_a",packet_a),item("candidate_b",packet_b)]})

def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_BYTES:
        raise Refused("MISSING_OR_OVERSIZED_BATCH")
    try:
        data=json.loads(raw.decode("utf-8"),object_pairs_hook=unique,
                        parse_constant=no_constant)
    except (UnicodeError,json.JSONDecodeError) as exc:
        raise Refused("INVALID_UTF8_JSON") from exc
    if type(data) is not dict or data.keys()!=FIELDS or canonical(data)!=raw:
        raise Refused("NONCANONICAL_OR_WRONG_BATCH_FIELDS")
    if data["protocol"]!=PROTOCOL or data["project"]!=PROJECT or data["parent_commit"]!=PARENT:
        raise Refused("BATCH_OR_SOURCE_IDENTITY_INVALID")
    items=data["intents"]
    if type(items) is not list or len(items)!=2:
        raise Refused("EXACTLY_TWO_SIGNED_INTENTS_REQUIRED")
    packets=[]
    for i,item in enumerate(items):
        if (type(item) is not dict or item.keys()!=ITEM_FIELDS or
            item.get("role")!=ROLES[i]):
            raise Refused("ROLE_IDENTITY_OR_ORDER_INVALID")
        encoded=item["intent_b64"]
        if type(encoded) is not str or not 40<len(encoded)<18000:
            raise Refused("SIGNED_INTENT_BASE64_INVALID")
        try:raw_intent=base64.b64decode(encoded,validate=True)
        except (ValueError,binascii.Error) as exc:
            raise Refused("SIGNED_INTENT_BASE64_INVALID") from exc
        if (base64.b64encode(raw_intent).decode("ascii")!=encoded or
                hashbytes(raw_intent)!=item["intent_sha256"]):
            raise Refused("SIGNED_INTENT_DIGEST_MISMATCH")
        try:rec.decode(raw_intent)
        except (rec.Invalid,ValueError,TypeError) as exc:
            raise Refused("INNER_RECOVERY_INTENT_INVALID") from exc
        packets.append(raw_intent)
    return packets

def _block(reason,detail=None,**fields):
    result={"status":"BLOCK","admission":"NOT_AUTHORIZED",
            "reason":reason,"independent_guardian":"NOT_ENROLLED",
            "owner_authenticated_oob_root":"NOT_ENROLLED",
            "protected_global_recovery_ledger":"NOT_IMPLEMENTED"}
    if detail is not None:result["detail"]=detail
    result.update(fields)
    return result

def decide(raw,expected_keys,primary_dir,floor_dir,now,after_first_read=None):
    """Check two real SSHSIGs; reread actual SQLite floor before final BLOCK.

    after_first_read is an experiment-only synchronous hook inserting a
    deterministic real OS writer between two observations.
    This is not a production atomicity boundary.
    """
    try:
        packets=parse(raw)
        if type(expected_keys) is not dict or set(expected_keys)!=set(ROLES):
            raise Refused("TWO_FROZEN_FIXTURE_KEY_PINS_REQUIRED")
        if type(now) is not int:raise Refused("TIME_NOT_INTEGER")
        signed=[]
        for role,packet in zip(ROLES,packets):
            value,msg,sig=rec.decode(packet)
            pl=value["payload"]
            key=value["fixture_public_key"]
            if type(expected_keys[role]) is not str or key!=expected_keys[role]:
                raise Refused("SIGNING_KEY_SUBSTITUTION_"+role.upper())
            if not pl["issued_unix"]<=now<=pl["expires_unix"]:
                raise Refused("STALE_SIGNED_RECOVERY_INTENT_"+role.upper())
            if not rec.verify(msg,sig,key):
                raise Refused("REAL_SSHSIG_VERIFICATION_FAILED_"+role.upper())
            signed.append((value,hashbytes(packet)))
        first=rec.observed(primary_dir,floor_dir)
        if after_first_read is not None:
            if not callable(after_first_read):raise Refused("INVALID_LAB_HOOK")
            after_first_read()
        second=rec.observed(primary_dir,floor_dir)
        if first!=second:
            raise Refused("CHECKPOINT_CHANGED_DURING_VALIDATION")
        for value,_ in signed:
            for field in ("floor_height","floor_epoch","floor_head","primary_head"):
                if value["payload"][field]!=second[field]:
                    raise Refused("SIGNED_INTENT_CHECKPOINT_STALE_"+field.upper())
        if signed[0][1]==signed[1][1]:
            reason="REPLAYED_IDENTICAL_SIGNED_INTENT_REFUSED"
        elif signed[0][0]["fixture_public_key"]==signed[1][0]["fixture_public_key"]:
            reason="SAME_FIXTURE_SIGNER_COLLISION_REFUSED"
        elif signed[0][0]["payload"]["nonce"]==signed[1][0]["payload"]["nonce"]:
            reason="CONCURRENT_SIGNED_NONCE_COLLISION_REFUSED"
        else:
            reason="CONCURRENT_VALID_SIGNED_INTENTS_NO_EXTERNAL_AUTHORITY"
        return _block(reason,real_native_signatures=2,
                      verified_fixture_only=True,local_checkpoint_height=second["floor_height"])
    except (Refused,rec.Invalid,rev.Denied,floor.Refused,OSError,
            ValueError,TypeError,subprocess.SubprocessError) as exc:
        return _block("CONCURRENT_RECOVERY_EVIDENCE_QUARANTINED",
                      detail=str(exc) if isinstance(exc,(Refused,rec.Invalid,rev.Denied,floor.Refused)) else type(exc).__name__)

def _child_restore(sources_and_targets):
    """Real distinct OS process: copy disposable SQLite snapshots only."""
    program="import shutil,sys; a=sys.argv[1:]; assert len(a)%2==0; [shutil.copy2(a[i],a[i+1]) for i in range(0,len(a),2)]"
    args=[sys.executable,"-B","-c",program]
    for source,target in sources_and_targets:
        args.extend([str(source),str(target)])
    r=subprocess.run(args,capture_output=True,text=True,timeout=25,check=False)
    if r.returncode:
        raise Refused("ACTUAL_SEPARATE_PROCESS_RESTORE_FAILED")

def exercise():
    if os.name=="nt" or not Path(rev.SSH).is_file():
        return _block("NATIVE_UNIX_OPENSSH_NOT_AVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5k-native-") as td:
        root=Path(td);os.chmod(root,0o700)
        primary=root/"primary";secondary=root/"checkpoint"
        primary.mkdir(mode=0o700);secondary.mkdir(mode=0o700)
        os.chmod(primary,0o700);os.chmod(secondary,0o700)
        p1=root/"key_a";p2=root/"key_b";p1.mkdir();p2.mkdir()
        os.chmod(p1,0o700);os.chmod(p2,0o700)
        priv_a,pub_a=rev.keygen(p1)
        priv_b,pub_b=rev.keygen(p2)
        with rev.RevocationLedger(primary) as ledger:
            if ledger.audit()["local_height"]!=0:raise Refused("GENESIS_PRIMARY_NOT_ZERO")
        if floor.execute(primary,secondary,"audit").get("floor_height")!=0:
            raise Refused("GENESIS_SECOND_FLOOR_NOT_ZERO")
        oldprimary=root/"oldprimary.sqlite3";oldfloor=root/"oldfloor.sqlite3"
        shutil.copy2(primary/rev.DB_NAME,oldprimary)
        shutil.copy2(secondary/floor.NAME,oldfloor)
        signed=rev.make_packet(priv_a,pub_a,p1)
        with rev.RevocationLedger(primary) as ledger:
            committed=ledger.revoke(signed,pub_a,1800000060)
        if committed.get("status")!="LAB_LOCAL_REVOKE_COMMIT":
            raise Refused("REAL_SIGNED_REVOCATION_NOT_COMMITTED")
        if floor.execute(primary,secondary,"checkpoint").get("status")!="LAB_LOCAL_CHECKPOINT_COMMIT":
            raise Refused("REAL_REVOCATION_FLOOR_NOT_CHECKPOINTED")
        current=rec.observed(primary,secondary)
        reqa=rec.intent(priv_a,pub_a,current,nonce="a"*64)
        reqb=rec.intent(priv_b,pub_b,current,nonce="b"*64)
        batch=envelope(reqa,reqb)
        keys={"candidate_a":pub_a,"candidate_b":pub_b}
        both=decide(batch,keys,primary,secondary,1800000060)
        if both.get("reason")!="CONCURRENT_VALID_SIGNED_INTENTS_NO_EXTERNAL_AUTHORITY":
            raise Refused("TWO_REAL_SIGNERS_NOT_QUARANTINED")
        duplicate=decide(envelope(reqa,reqa),
                         {"candidate_a":pub_a,"candidate_b":pub_a},
                         primary,secondary,1800000060)
        if duplicate.get("reason")!="REPLAYED_IDENTICAL_SIGNED_INTENT_REFUSED":
            raise Refused("REAL_DUPLICATE_NOT_QUARANTINED")
        primary_target=primary/rev.DB_NAME
        floor_target=secondary/floor.NAME
        changed=decide(batch,keys,primary,secondary,1800000060,
                       after_first_read=lambda:_child_restore([(oldprimary,primary_target)]))
        if (changed.get("reason")!="CONCURRENT_RECOVERY_EVIDENCE_QUARANTINED" or
                "STALE_DIVERGENT" not in changed.get("detail","")):
            raise Refused("ACTUAL_PRIMARY_RESTORE_NOT_QUARANTINED")
        # Restore disposable signed primary by re-applying the genuine signed test REVOKE.
        with rev.RevocationLedger(primary) as ledger:
            recommit=ledger.revoke(signed,pub_a,1800000060)
        if recommit.get("status")!="LAB_LOCAL_REVOKE_COMMIT":
            raise Refused("DISPOSABLE_PRIMARY_RECOMMIT_FAILED")
        if floor.execute(primary,secondary,"audit").get("reason")!="LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT":
            raise Refused("DISPOSABLE_HISTORY_NOT_REESTABLISHED")
        dual=decide(batch,keys,primary,secondary,1800000060,
                    after_first_read=lambda:_child_restore([(oldprimary,primary_target),(oldfloor,floor_target)]))
        if (dual.get("reason")!="CONCURRENT_RECOVERY_EVIDENCE_QUARANTINED" or
                "PRE_REVOCATION" not in dual.get("detail","")):
            raise Refused("ACTUAL_TWO_STORE_RESTORE_NOT_QUARANTINED")
        return _block("CONCURRENT_REAL_SIGNED_RECOVERY_ALWAYS_DENIED",
                      two_valid_disposable_signer_keys=True,
                      two_signed_intents_even_if_consistent="BLOCK",
                      duplicate_signed_intent="BLOCK",
                      actual_separate_process_primary_restore="BLOCK",
                      actual_separate_process_both_store_restore="BLOCK",
                      all_local_artifacts_rollback_without_external_anchor="NOT_GLOBALLY_DETECTABLE",
                      native_access_or_recovery_granted=False)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--exercise",action="store_true",required=True)
    p.parse_args(argv)
    try:report=exercise()
    except (Refused,rec.Invalid,rev.Denied,floor.Refused,OSError,ValueError,TypeError,
            subprocess.SubprocessError) as exc:
        report=_block("REAL_CONCURRENT_LAB_REFUSED",
                      detail=str(exc) if isinstance(exc,(Refused,rec.Invalid,rev.Denied,floor.Refused)) else type(exc).__name__)
    print(json.dumps(report,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":
    sys.exit(main())
