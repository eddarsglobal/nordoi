#!/usr/bin/env python3
"""NORDOI E6: real local signed receipt binding != independent witness authority.

--audit: bound E5/R1 source/proof status (expected exit 2 BLOCK).
--exercise: genuine POSIX OpenSSH Ed25519 signing, re-signing, real second
OS process verification, local nonce replay and original real R5Q rollback.
Expected exit 2 BLOCK, errors exit 3. No humans, network, enrollments, keys
exported, independent signatures, R6G or production transition.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e5_separate_witness_contract as e5
import bureau_001_e3_exact_head_provenance as e3
import g03_r6f_r6_external_custody_feasibility as r6
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5x_signed_external_provenance as signer
import g03_r6f_r4_fd_race as r4

CONTRACT="governance/NORDOI_R6F_BUREAU001_E6_SIGNED_RECEIPT_BOUNDARY.json"
SCHEMA="NORDOI-G03-R6F-BUREAU001-E6-LOCAL-SIGNED-RECEIPT-REBOUNDARY-V1"
PROTOCOL="NORDOI-R6F-E6-LOCAL-SIGNED-WITNESS-RECEIPT-DENY-ONLY-V1"
NAMESPACE="nordoi-r6f-e6-local-witness-receipt"
PROJECT="eddarsglobal/nordoi"
PARENT="ccad109eff3931b7c0417bf5e25e3291617bd5c7"
IDENTITY=ssh.IDENTITY
FIELDS=frozenset((
  "protocol","project","source_head","manifest_sha256",
  "previous_receipt_sha256","challenge_nonce_sha256",
  "candidate_public_sha256","key_epoch","revocation_floor",
  "spent_nonce_floor","issued_at","expires_at",
  "claimed_external_custody"))
SHA=re.compile(r"[0-9a-f]{64}\Z")
SHA40=re.compile(r"[0-9a-f]{40}\Z")

class Blocked(ValueError):pass

def need(x,why):
    if not x:raise Blocked(why)

def unique(pairs):
    d={}
    for k,v in pairs:
        if k in d:raise Blocked("E6_DUPLICATE_SIGNED_RECEIPT_FIELD")
        d[k]=v
    return d

def nonfinite(_):
    raise Blocked("E6_NONFINITE_LOCAL_RECEIPT_NUMBER")

def canonical(obj):
    return (json.dumps(obj,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def parse(raw,expected_head):
    need(type(raw) is bytes and 0<len(raw)<=4096,"E6_RECEIPT_SIZE_OR_TYPE")
    need(type(expected_head) is str and SHA40.fullmatch(expected_head),
         "E6_EXPECTED_HEAD_MALFORMED")
    try:
        d=json.loads(raw.decode("utf-8"),
            object_pairs_hook=unique,parse_constant=nonfinite)
    except (UnicodeError,json.JSONDecodeError) as e:
        raise Blocked("E6_INVALID_SIGNED_RECEIPT_JSON") from e
    need(type(d) is dict and d.keys()==FIELDS and canonical(d)==raw,
         "E6_WRONG_FIELDS_OR_NONCANONICAL_SIGNED_RECEIPT")
    need(d["protocol"]==PROTOCOL and d["project"]==PROJECT and
         d["source_head"]==expected_head,"E6_RECEIPT_WRONG_SOURCE_OR_PROTOCOL")
    for k in ("manifest_sha256","previous_receipt_sha256",
              "challenge_nonce_sha256","candidate_public_sha256"):
        need(type(d[k]) is str and SHA.fullmatch(d[k]),
             "E6_INVALID_SHA256_"+k)
    for k in ("key_epoch","revocation_floor","spent_nonce_floor",
              "issued_at","expires_at"):
        need(type(d[k]) is int and 0<=d[k]<=2**63-1,
             "E6_BAD_LOCAL_NUMBER_"+k)
    need(d["issued_at"] < d["expires_at"] <= d["issued_at"]+300,
         "E6_INVALID_LOCAL_RECEIPT_TIMEBOX")
    need(type(d["claimed_external_custody"]) is bool,
         "E6_FALSE_EXTERNAL_AUTHORITY_TYPE")
    return d

def make(head,manifest_sha,previous_sha,nonce_sha,pub_sha,now=1200,
         **changes):
    p=dict(
        protocol=PROTOCOL,project=PROJECT,source_head=head,
        manifest_sha256=manifest_sha,
        previous_receipt_sha256=previous_sha,
        challenge_nonce_sha256=nonce_sha,
        candidate_public_sha256=pub_sha,
        key_epoch=1,revocation_floor=0,spent_nonce_floor=0,
        issued_at=now,expires_at=now+120,
        claimed_external_custody=False)
    need(not (set(changes)-FIELDS),"E6_UNRECOGNIZED_LOCAL_PACKET_FIELD")
    p.update(changes)
    return canonical(parse(canonical(p),head))

def refuse(reason,**info):
    d=dict(
        result="BLOCK",reason=reason,
        cryptographic_signature_valid=info.get("cryptographic_signature_valid",False),
        signed_source_head=info.get("signed_source_head"),
        local_spent_nonce_detected=info.get("local_spent_nonce_detected",False),
        independently_authenticated_guardian=False,
        authenticated_external_witness_receipts=0,
        independent_monotonic_history=False,
        native_admission_gates_unmet=12,
        human_testers=0,independent_specialist_reviews=0,
        real_external_custodian="NOT_ENROLLED",
        owner_oob_root="NOT_AUTHENTICATED",
        global_monotonic_antirollback="NOT_VERIFIED",
        r6f="IN_PROGRESS_UNCERTIFIED",
        r6g="LOCKED_BY_SEQUENCE",production="NOT_AUTHORIZED")
    # Preserve measured negative-lab details but never permit a caller to
    # override the fixed BLOCK / native-lock / independent-authority fields.
    d.update({k:v for k,v in info.items() if k not in d})
    return d

def verify_local(raw,signature,allowed,public,head,manifest,previous,
                 challenge,now,spent_local=(),min_local_epoch=0):
    """True SSHSIG verification of exact bytes. NEVER external witness approval."""
    try:
        d=parse(raw,head)
        need(all(type(v) is str and SHA.fullmatch(v)
                 for v in (manifest,previous,challenge)),
             "E6_EXPECTED_CONTEXT_HASH_INVALID")
        need(d["manifest_sha256"]==manifest and
             d["previous_receipt_sha256"]==previous and
             d["challenge_nonce_sha256"]==challenge,
             "E6_SIGNED_RECEIPT_WRONG_CONTEXT_BINDING")
        need(type(now) is int and d["issued_at"]<=now<=d["expires_at"],
             "E6_LOCAL_CLOCK_OR_EXPIRY_INVALID")
        need(type(min_local_epoch) is int and
             d["key_epoch"]>=min_local_epoch,
             "E6_LOCAL_EPOCH_REGRESSION")
        need(type(spent_local) in (set,frozenset,tuple,list) and
             all(type(x) is str and SHA.fullmatch(x) for x in spent_local),
             "E6_SPENT_LOCAL_INPUT_MALFORMED")
        if challenge in spent_local:
            return refuse("E6_LOCAL_SPENT_NONCE_REPLAY_DETECTED",
                          local_spent_nonce_detected=True,
                          signed_source_head=head)
        need(r4.is_posix_ssh(),"E6_GENUINE_POSIX_OPENSSH_REQUIRED")
        pub=Path(public); allow=Path(allowed); sig=Path(signature)
        need(all(p.is_file() and not p.is_symlink() for p in (pub,allow,sig)),
             "E6_PUBLIC_ALLOWLIST_OR_SIGNATURE_PATH_REPLACED")
        # Exact allowlist is locally derived from this exact local public key.
        # This proves same-admin key possession, not OOB-verified control.
        pub_bytes=e3.real_file_bytes(pub,limit=2048)
        allowed_bytes=e3.real_file_bytes(allow,limit=4096)
        need(digest(pub_bytes)==d["candidate_public_sha256"],
             "E6_CANDIDATE_PUBLIC_KEY_NOT_SIGNED_PACKET_PIN")
        need(allowed_bytes==IDENTITY.encode()+b" "+pub_bytes,
             "E6_LOCAL_ALLOWED_SIGNERS_NOT_BOUND_TO_PUBLIC")
        sig_before=digest(e3.real_file_bytes(sig,limit=16384))
        cmd=["-Y","verify","-f",str(allow),"-I",IDENTITY,
             "-n",NAMESPACE,"-s",str(sig)]
        ok=ssh.run_ssh(cmd,raw)==0
        need(sig_before==digest(e3.real_file_bytes(sig,limit=16384)),
             "E6_SIGNATURE_CHANGED_DURING_VERIFICATION")
        need(ok,"E6_REAL_ED25519_SIGNATURE_OR_NAMESPACE_INVALID")
        return refuse("E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS",
                      cryptographic_signature_valid=True,
                      signed_source_head=head)
    except (Blocked,e3.Rejected,ssh.Refused,ValueError,TypeError,KeyError,
            OSError,subprocess.SubprocessError) as ex:
        return refuse("E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED")

def audit(root,head):
    root=Path(root).resolve(strict=True)
    p=e3.strictly_load(e3.real_file_bytes(root/CONTRACT))
    base=e5.audit(root,head)
    need(p["schema"]==SCHEMA and p["project"]==PROJECT and
         p["parent_e5_exact_head"]==PARENT,
         "E6_INCORRECT_FROZEN_E5_PARENT")
    need(p["predecessor_e5_cis"]=={
         "specialized_run_id":44025229325,"specialized_success":4,
         "general_run_id":44025229249,"general_success":5},
         "E6_OLD_GREEN_GATES_NOT_VERIFIED")
    need(p["receipt_schema"]==PROTOCOL and
         p["ssh_namespace"]==NAMESPACE and
         p["receipt_identity"]==IDENTITY and
         set(p["required_fields"])==FIELDS,
         "E6_RECEIPT_CONTRACT_FIELDS_MISMATCH")
    need(len(p["proof_boundaries"])==6 and
         all(x["status"]!="VERIFIED_EXTERNAL" for x in p["proof_boundaries"])
         and p["real_external_signer_authenticated"] is False
         and p["real_external_receipts_received"]==0
         and p["true_external_monotonic_ledger_verified"] is False
         and p["external_capabilities_implemented"] is False
         and p["human_testers_allowed"] is False
         and p["operator_contacted"] is False
         and p["independent_specialist_reviews"]==0
         and p["owner_oob_root"]=="NOT_AUTHENTICATED"
         and p["external_custodian"]=="NOT_ENROLLED"
         and p["global_monotonic_antirollback"]=="NOT_VERIFIED"
         and p["r6f"]=="IN_PROGRESS_UNCERTIFIED"
         and p["r6g"]=="LOCKED_BY_SEQUENCE"
         and p["native_admission_gates_unmet"]==12
         and p["production"]=="NOT_AUTHORIZED"
         and p["formal_closure"]=="HOLD_MISSING_INDEPENDENT_EVIDENCE",
         "E6_FALSE_EXTERNAL_AUTHORITY_OR_PRIVILEGED_TRANSITION")
    return refuse("E6_LOCAL_RECEIPT_CRYPTO_PROVENANCE_CONTRACT_UNMET",
                  signed_source_head=base["source_head"])

def local_crypto_fixture(root,head):
    root=Path(root)
    key,allowed=ssh.create_same_admin_key(root,"e6_local_candidate")
    pub=root/"e6_local_candidate.pub"
    manifest="099a23953ba7815556d82b393555a7ab475aa4b946684325e85c9e7e2392b77e"
    prev="0"*64
    nonce=digest(os.urandom(32))
    raw=make(head,manifest,prev,nonce,digest(pub.read_bytes()))
    packet=root/"e6_receipt.json";packet.write_bytes(raw)
    os.chmod(packet,0o600)
    signature=signer.sign_local(packet,key,namespace=NAMESPACE)
    return dict(key=key,allowed=allowed,public=pub,receipt=packet,
                signature=signature,raw=raw,manifest=manifest,
                previous=prev,nonce=nonce,head=head)

def check_second_process(data,**overrides):
    root=Path(__file__).resolve().parents[1]
    opts={
        "--expected-head" : data["head"],
        "--receipt" : str(data["receipt"]),
        "--signature" : str(data["signature"]),
        "--allowed" : str(data["allowed"]),
        "--public" : str(data["public"]),
        "--manifest-sha" : data["manifest"],
        "--previous-sha" : data["previous"],
        "--challenge-sha" : data["nonce"],
        "--now" : "1200"}
    opts.update(overrides)
    cmd=[sys.executable,"-B",str(Path(__file__).resolve()),
         "--verify-local"]
    for key,v in opts.items():cmd.extend([key,v])
    p=subprocess.run(cmd,cwd=root,capture_output=True,text=True,timeout=60)
    need(p.returncode==2,"E6_SECOND_OS_PROCESS_NOT_DENY_ONLY")
    d=json.loads(p.stdout)
    return d

def exercise(root,head):
    pre=audit(root,head)
    need(r4.is_posix_ssh(),"E6_REAL_POSIX_OPENSSH_REQUIRED")
    with tempfile.TemporaryDirectory(
         prefix="nordoi-r5l-r5q-r5s-r5t-e6-native-") as tmp:
        rootdir=Path(tmp);os.chmod(rootdir,0o700)
        d=local_crypto_fixture(rootdir,head)
        valid=check_second_process(d)
        need(valid["reason"]=="E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS"
             and valid["cryptographic_signature_valid"] is True,
             "E6_GENUINE_LOCAL_RECEIPT_NOT_VERIFIED")
        # A separate receiver process with no retained spent nonce sees the
        # exact same cryptographically valid signature as valid again.
        duplicate=check_second_process(d)
        need(duplicate["reason"]==valid["reason"],
             "E6_CROSS_PROCESS_LOCAL_REPLAY_NOT_OBSERVED")
        blocked=verify_local(d["raw"],d["signature"],d["allowed"],
            d["public"],head,d["manifest"],d["previous"],d["nonce"],1200,
            spent_local=frozenset([d["nonce"]]))
        need(blocked["reason"]=="E6_LOCAL_SPENT_NONCE_REPLAY_DETECTED",
             "E6_LOCAL_NONCE_REPLAY_NOT_REFUSED")
        wrong=check_second_process(d,**{"--challenge-sha":"f"*64})
        need(wrong["reason"]=="E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED",
             "E6_WRONG_CHALLENGE_NOT_REFUSED")
        bad_raw=d["raw"].replace(b'"key_epoch":1',b'"key_epoch":2')
        tampered=verify_local(bad_raw,d["signature"],d["allowed"],
                    d["public"],head,d["manifest"],d["previous"],d["nonce"],1200)
        need(tampered["reason"]=="E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED",
             "E6_INVALID_SIGNATURE_ON_REWRITTEN_RECEIPT_NOT_REFUSED")
        # The SAME admin can sign a different history binding with the SAME
        # actual Ed25519 key. Crypto verifies, but no external history exists.
        new_manifest=digest(b"E6_NEW_LOCAL_MANIFEST_NOT_AN_EXTERNAL_HISTORY")
        rebound=make(head,new_manifest,d["previous"],d["nonce"],
                     digest(d["public"].read_bytes()))
        rebound_path=rootdir/"e6_resigned_local_history.json"
        rebound_path.write_bytes(rebound);os.chmod(rebound_path,0o600)
        rebound_sig=signer.sign_local(rebound_path,d["key"],namespace=NAMESPACE)
        res=check_second_process(dict(d,receipt=rebound_path,signature=rebound_sig,
                                      manifest=new_manifest))
        need(res["reason"]=="E6_REAL_LOCAL_SSHSIG_VALID_NO_INDEPENDENT_WITNESS",
             "E6_SAME_ADMIN_RE_SIGNED_HISTORY_NOT_ACCEPTED_AS_LOCAL_ONLY")
        # Wrong namespace also signed with genuine key but rejected in E6.
        wrong_path=rootdir/"e6_wrong_namespace.json"
        wrong_path.write_bytes(d["raw"]);os.chmod(wrong_path,0o600)
        wrong_sig=signer.sign_local(wrong_path,d["key"],
                                   namespace="nordoi-r6f-e6-wrong-context")
        badns=verify_local(d["raw"],wrong_sig,d["allowed"],d["public"],
                           head,d["manifest"],d["previous"],d["nonce"],1200)
        need(badns["reason"]=="E6_INVALID_OR_CONTEXT_MISMATCHED_RECEIPT_DENIED",
             "E6_WRONG_SSHSIG_NAMESPACE_NOT_REFUSED")
        rollback=r6.inspect_real_rollback()
        need(rollback["same_admin_all_three_restored"] and
             rollback["globally_spent_nonce_reissued_after_full_rollback"] and
             rollback["protected_independent_high_water"] is False,
             "E6_REAL_R5Q_LOCAL_ONLY_FLOOR_LIMIT_NOT_REPRODUCED")
    need(e3.check_head(root,head)==head,"E6_SOURCE_CHANGED")
    return refuse("E6_TWO_REAL_LOCAL_REBOUND_RECEIPTS_AND_CROSS_PROCESS_REPLAY_DENIED",
        signed_source_head=head,
        cryptographic_signature_valid=True,
        real_signed_local_receipts=2,
        first_signature_verified_in_real_second_process=True,
        untracked_replay_accepted_in_new_process=True,
        local_spent_set_detected=True,
        changed_bytes_without_signature_refused=True,
        different_manifest_resigned_by_same_admin=True,
        wrong_signature_namespace_refused=True,
        original_r5q_three_file_rollback_reproduced=True)

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    modes=p.add_mutually_exclusive_group(required=True)
    modes.add_argument("--audit",action="store_true")
    modes.add_argument("--exercise",action="store_true")
    modes.add_argument("--verify-local",action="store_true")
    p.add_argument("--expected-head",required=True)
    p.add_argument("--root",type=Path,default=ROOT)
    for a in ("receipt","signature","allowed","public","manifest-sha",
              "previous-sha","challenge-sha","now"):
        p.add_argument("--"+a)
    opts=p.parse_args(argv)
    try:
        if opts.audit:out=audit(opts.root,opts.expected_head)
        elif opts.exercise:out=exercise(opts.root,opts.expected_head)
        else:
            need(all(v is not None for v in (
              opts.receipt,opts.signature,opts.allowed,opts.public,
              opts.manifest_sha,opts.previous_sha,opts.challenge_sha,opts.now)),
              "E6_LOCAL_VERIFIER_ARGUMENTS_REQUIRED")
            raw=e3.real_file_bytes(opts.receipt,limit=4096)
            out=verify_local(raw,opts.signature,opts.allowed,opts.public,
                             opts.expected_head,opts.manifest_sha,
                             opts.previous_sha,opts.challenge_sha,int(opts.now))
        code=2
    except (Blocked,e5.Blocked,e3.Rejected,ValueError,KeyError,TypeError,
            OSError,subprocess.SubprocessError) as exc:
        out=refuse("E6_SOURCE_OR_REAL_LAB_NOT_QUALIFIED")
        code=3
    print(json.dumps(out,sort_keys=True))
    return code

if __name__=="__main__":raise SystemExit(main())
