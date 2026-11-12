#!/usr/bin/env python3
"""NORDOI G0.3 R5V: real OS identity evidence vs independent custody.

Two actual Python processes and two genuine Ed25519 signers under the same
effective administrator do NOT prove distinct trust domains. Claims about
administrative roles, home directories or distinct PIDs cannot make an
independent OOB trust root or globally monotonic revocation history.
All outputs BLOCK / NOT_AUTHORIZED. No real-user switching or production.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

import g03_r5l_crash_restart_denial_journal as denial
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as rotate

PROTOCOL="NORDOI-G03-R5V-ADMIN-SEPARATION-EVIDENCE-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="e2138af4c1a2dd3964d5ebdd9bf669129b63cfb1"
NAMESPACE="nordoi-r5v"
FIELDS=frozenset(("protocol","project","parent","role","claimed_domain",
                   "pid","ppid","uid","euid","gid","egid","key_sha256",
                   "key_epoch","history_head","nonce"))
ROLES=("owner_proxy","custodian_proxy")
MAX_REPORT=2400

class Refused(ValueError):pass

def canonical(obj):
    return (json.dumps(obj,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")
def pairs(items):
    d={}
    for k,v in items:
        if k in d:raise Refused("DUPLICATE_OS_IDENTITY_FIELD")
        d[k]=v
    return d
def no_nan(_):raise Refused("NONFINITE_OS_IDENTITY_EVIDENCE")
def block(reason,**kwargs):
    return denial.block(reason,native_authority_granted=False,
                        independent_admin_custody="NOT_PROVEN",
                        authenticated_external_guardian="NOT_ENROLLED",
                        owner_oob_root="NOT_ENROLLED",
                        global_monotonic_key_history="NOT_VERIFIED",**kwargs)
def issha(x):return rotate.check_sha(x)
def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_REPORT:
        raise Refused("OS_IDENTITY_EVIDENCE_MISSING_OR_OVERSIZED")
    try:
        obj=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs,
                       parse_constant=no_nan)
    except (UnicodeError,json.JSONDecodeError) as e:
        raise Refused("OS_IDENTITY_NOT_UTF8_JSON") from e
    if type(obj) is not dict or obj.keys()!=FIELDS or canonical(obj)!=raw:
        raise Refused("OS_IDENTITY_FIELDS_OR_CANONICAL_INVALID")
    if (obj["protocol"],obj["project"],obj["parent"])!=(
        PROTOCOL,PROJECT,PARENT):
        raise Refused("UNPINNED_IDENTITY_SOURCE")
    if obj["role"] not in ROLES:
        raise Refused("INVALID_ACTUAL_PROXY_ROLE")
    if type(obj["claimed_domain"]) is not str or \
       re.fullmatch(r"lab-[a-z][a-z0-9-]{2,39}",obj["claimed_domain"]) is None:
        raise Refused("UNTRUSTED_DOMAIN_LABEL_INVALID")
    for k in ("pid","ppid"):
        if type(obj[k]) is not int or obj[k]<0 or (k=="pid" and obj[k]==0):
            raise Refused("INVALID_PROCESS_ID_"+k)
    for k in ("uid","euid","gid","egid"):
        if obj[k] is not None and (type(obj[k]) is not int or obj[k]<0):
            raise Refused("INVALID_OPERATING_SYSTEM_ID_"+k)
    if any(not issha(obj[k]) for k in ("key_sha256","history_head","nonce")):
        raise Refused("OS_IDENTITY_INVALID_SHA_OR_NONCE")
    if type(obj["key_epoch"]) is not int or obj["key_epoch"]<0:
        raise Refused("OS_IDENTITY_INVALID_KEY_EPOCH")
    return obj

def observe(role,claimed_domain,key_sha,epoch,head,nonce):
    if role not in ROLES:raise Refused("INVALID_PROXY_ROLE")
    values={
        "protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
        "role":role,"claimed_domain":claimed_domain,
        "pid":os.getpid(),"ppid":os.getppid(),
        "uid":os.getuid() if hasattr(os,"getuid") else None,
        "euid":os.geteuid() if hasattr(os,"geteuid") else None,
        "gid":os.getgid() if hasattr(os,"getgid") else None,
        "egid":os.getegid() if hasattr(os,"getegid") else None,
        "key_sha256":key_sha,"key_epoch":epoch,
        "history_head":head,"nonce":nonce,
    }
    return parse(canonical(values))

def child_observe(role,domain,pin,epoch,head,nonce):
    cmd=[sys.executable,"-B",str(Path(__file__).resolve()),
         "--observe","--role",role,"--claimed-domain",domain,
         "--key-sha256",pin,"--epoch",str(epoch),
         "--head",head,"--nonce",nonce]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=30,check=False)
    if p.returncode!=2:
        raise Refused("ACTUAL_CHILD_PROCESS_DID_NOT_FAIL_CLOSED")
    try:return parse(p.stdout.encode("ascii"))
    except (UnicodeError,ValueError) as e:
        raise Refused("ACTUAL_CHILD_REPORT_INVALID") from e

def assess(owner,candidate):
    """Real identity comparison is a necessary disqualifier, never a grant."""
    try:
        o=parse(canonical(owner));c=parse(canonical(candidate))
        if o["role"]!="owner_proxy" or c["role"]!="custodian_proxy":
            raise Refused("REPORTED_ROLES_NOT_BOUND")
        if o["nonce"]!=c["nonce"] or o["history_head"]!=c["history_head"] or \
           o["key_epoch"]!=c["key_epoch"]:
            raise Refused("OBSERVED_CHALLENGE_OR_HISTORY_MISMATCH")
        if o["pid"]==c["pid"]:
            raise Refused("SAME_OS_PROCESS_NOT_SEPARATE")
        if o["key_sha256"]==c["key_sha256"]:
            raise Refused("SAME_KEY_NOT_DISTINCT_SIGNATURE_CUSTODY")
        if o["uid"] is None or c["uid"] is None:
            return block("OS_USER_ID_SEPARATION_NOT_OBSERVABLE_ON_PLATFORM",
                         os_identity_observation_complete=False)
        if o["uid"]==c["uid"] or o["euid"]==c["euid"]:
            return block("ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED",
                         os_identity_observation_complete=True,
                         actual_distinct_processes=True,
                         distinct_local_key_pins=True,
                         claimed_different_domains_not_proof=True,
                         actual_euid_equal=o["euid"]==c["euid"])
        # A different OS uid is only an observation. Proving independent
        # control requires independently OOB authenticated custody and keys.
        return block("DIFFERENT_OS_UID_STILL_NO_INDEPENDENT_CUSTODY_PROOF",
                     os_identity_observation_complete=True,
                     independent_owner_root_authenticated=False)
    except (Refused,TypeError,ValueError) as e:
        return block("ADMINISTRATIVE_SEPARATION_EVIDENCE_QUARANTINED",
                     detail=str(e) if isinstance(e,Refused) else type(e).__name__)

def signature(path,key):
    sig=Path(str(path)+".sig")
    if sig.exists():raise Refused("SIGNED_OS_REPORT_ALREADY_EXISTS")
    if signed.run_ssh(["-Y","sign","-f",str(key),"-n",NAMESPACE,str(path)])!=0 \
       or not sig.is_file():raise Refused("REAL_OS_IDENTITY_SSHSIG_SIGN_FAILED")
    if os.name!="nt":os.chmod(sig,0o600)
    return sig

def verify(path,signature_path,allowed,pub,expected):
    try:
        with fdgate.OpenSnapshot(Path(path)) as held:
            raw=Path(path).read_bytes()
            obj=parse(raw)
            held.check()
        if obj["key_sha256"]!=rotate.public_sha(pub) or \
           obj["key_sha256"]!=expected:
            raise Refused("OS_REPORT_KEY_NOT_BOUND_TO_REAL_PUBLIC_KEY")
        with fdgate.OpenSnapshot(Path(signature_path)) as sig:
            if signed.run_ssh(["-Y","verify","-f",str(allowed),
                               "-I",signed.IDENTITY,"-n",NAMESPACE,
                               "-s",str(signature_path)],raw)!=0:
                raise Refused("REAL_OS_REPORT_ED25519_SIGNATURE_INVALID")
            sig.check()
        return block("REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL",
                     signature_valid_for_local_key=True,
                     independently_authenticated_signer=False)
    except (Refused,fdgate.Refused,OSError,ValueError,TypeError) as e:
        return block("OS_REPORT_SIGNATURE_QUARANTINED",
                     detail=str(e) if isinstance(e,(Refused,fdgate.Refused))
                     else type(e).__name__)

def native_exercise():
    if os.name=="nt":return block("POSIX_OS_IDENTITY_LAB_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5v-native-") as tmp:
        root=Path(tmp);os.chmod(root,0o700)
        a,b,j=history.fixture(root)
        bundle=root/"bundle.zip"
        handoff.prepare(a,b,j,"a"*64,1000,1200,bundle)
        receipt=root/"receipt.json"
        body=signed.prepare_receipt(bundle,"a"*64,1100)
        receipt.write_bytes(body);os.chmod(receipt,0o600)
        old_key,old_allow=signed.create_same_admin_key(root,"owner_key")
        next_key,next_allow=signed.create_same_admin_key(root,"candidate_key")
        old_pub=root/"owner_key.pub";new_pub=root/"candidate_key.pub"
        old_pin=rotate.public_sha(old_pub);new_pin=rotate.public_sha(new_pub)
        state=rotate.bootstrap(root,old_pin)
        owner=child_observe("owner_proxy","lab-owner-space",old_pin,0,state["head"],"a"*64)
        candidate=child_observe("custodian_proxy","lab-candidate-space",new_pin,0,state["head"],"a"*64)
        if owner["pid"]==candidate["pid"] or owner["claimed_domain"]==candidate["claimed_domain"]:
            raise Refused("TWO_REAL_PROCESSES_NOT_DISTINCT")
        if owner["euid"]!=candidate["euid"] or owner["uid"]!=candidate["uid"]:
            raise Refused("SAME_ADMIN_NEGATIVE_CONTROL_NOT_REPRODUCIBLE")
        comparison=assess(owner,candidate)
        if comparison["reason"]!="ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED":
            raise Refused("SAME_REAL_UID_FALSELY_INDEPENDENT")
        owner_file=root/"owner_report.json"
        cand_file=root/"custodian_report.json"
        owner_file.write_bytes(canonical(owner));cand_file.write_bytes(canonical(candidate))
        os.chmod(owner_file,0o600);os.chmod(cand_file,0o600)
        owner_sig=signature(owner_file,old_key)
        candidate_sig=signature(cand_file,next_key)
        for p,sig,allow,pub,pin in ((owner_file,owner_sig,old_allow,old_pub,old_pin),
                                     (cand_file,candidate_sig,next_allow,new_pub,new_pin)):
            if verify(p,sig,allow,pub,pin)["reason"]!="REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL":
                raise Refused("GENUINE_SIGNED_REAL_OS_REPORT_FAILED")
        # Real valid key-change / revocation from R5U, not a dummy JSON status.
        rev=root/"rotate.json"
        payload=rotate.intent(rotate.ROTATE,1,old_pin,new_pin,
                              rotate.sha(body),"1"*64)
        rev.write_bytes(payload);os.chmod(rev,0o600)
        osig=rotate.sign_intent(rev,old_key)
        other=root/"rotate_next.json";other.write_bytes(payload)
        nsig=rotate.sign_intent(other,next_key)
        out=rotate.append(root,payload,osig,old_allow,nsig,next_allow)
        if out["reason"]!="REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED":
            raise Refused("GENUINE_TWO_SIG_ROTATION_NOT_RECORDED")
        # Preserve height-one ledger; sign and commit genuine successor revocation.
        snapshot=root/"before_revocation.sqlite3"
        shutil.copy2(rotate.ledger_path(root),snapshot)
        revpath=root/"revoke.json"
        revoke=rotate.intent(rotate.REVOKE,2,new_pin,new_pin,
                             rotate.sha(body),"2"*64)
        revpath.write_bytes(revoke)
        rsig=rotate.sign_intent(revpath,next_key)
        revoked=rotate.append(root,revoke,rsig,next_allow)
        if revoked["reason"]!="REAL_LOCAL_SIGNED_REVOCATION_DENIED":
            raise Refused("REAL_SIGNED_REVOCATION_NOT_COMMITTED")
        # Local cryptographic signature survives revocation, but policy is
        # evaluated from the locally controlled SQLite key epoch.
        if verify(cand_file,candidate_sig,next_allow,new_pub,new_pin)["reason"]!=\
                "REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL":
            raise Refused("SIGNATURE_ALGORITHMIC_VALIDITY_NOT_STABLE")
        if not rotate.audit(root)["revoked"]:
            raise Refused("REVOCATION_NOT_RECORDED")
        # Real second process replaces SQLite file, still same admin control.
        fdgate.real_atomic_replace(snapshot,rotate.ledger_path(root))
        if rotate.audit(root)["revoked"]:
            raise Refused("LOCAL_REVOCATION_ROLLBACK_NOT_REPRODUCED")
        restored=assess(owner,candidate)
        if restored["reason"]!="ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED":
            raise Refused("REVOKED_CUSTODY_NOT_DISQUALIFIED_AFTER_ROLLBACK")
        # Byte-identical signature substitution while held FD is open:
        dup=root/"same_signature.sig"
        shutil.copy2(candidate_sig,dup);os.chmod(dup,0o600)
        with fdgate.OpenSnapshot(candidate_sig) as held:
            fdgate.real_atomic_replace(dup,candidate_sig)
            try:held.check()
            except fdgate.Refused:swapped=True
            else:swapped=False
        if not swapped:raise Refused("REAL_SIGNATURE_INODE_SWAP_NOT_DETECTED")
        return block("REAL_OS_ADMIN_CUSTODY_DISQUALIFICATION_QUALIFIED",
                     real_child_processes="DIFFERENT_PIDS_SAME_UID_EUID",
                     distinct_ed25519_local_keypairs="VALID_KEY_POSSESSION_ONLY",
                     owner_and_custodian_claimed_domains="DIFFERENT_LABELS_NOT_AUTHORITY",
                     real_os_uid_euid_separation="NOT_OBSERVED",
                     true_local_two_key_rotation="BLOCK_ONLY",
                     real_signed_successor_revocation="LOCAL_BLOCK",
                     cryptographic_signatures_after_revocation="STILL_MATHEMATICALLY_VALID",
                     local_revocation_after_second_process_rollback="LOST_LOCALLY",
                     real_second_process_signature_inode_swap="QUARANTINED",
                     actual_independent_guardian="NOT_ENROLLED",
                     native_admission="BLOCK")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument("--observe",action="store_true")
    g.add_argument("--exercise",action="store_true")
    for arg in ("role","claimed-domain","key-sha256","head","nonce"):
        p.add_argument("--"+arg)
    p.add_argument("--epoch",type=int,default=0)
    args=p.parse_args(argv)
    if args.observe:
        try:out=canonical(observe(args.role,args.claimed_domain,args.key_sha256,
                                  args.epoch,args.head,args.nonce)).decode("ascii")
        except (Refused,TypeError,ValueError) as e:
            out=canonical({"status":"BLOCK","reason":"OS_OBSERVATION_REFUSED",
                           "detail":str(e)}).decode("ascii")
        sys.stdout.write(out)
        return 2
    try:out=native_exercise()
    except (Refused,rotate.Refused,handoff.Refused,history.Refused,fdgate.Refused,
            OSError,ValueError,TypeError,subprocess.SubprocessError) as e:
        out=block("REAL_ADMIN_SEPARATION_LAB_REFUSED",
                  detail=str(e) if isinstance(e,(Refused,rotate.Refused,
                      handoff.Refused,history.Refused,fdgate.Refused))
                  else type(e).__name__)
    print(json.dumps(out,sort_keys=True,separators=(",",":")))
    return 2
if __name__=="__main__":sys.exit(main())
