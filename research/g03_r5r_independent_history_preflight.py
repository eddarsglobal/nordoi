#!/usr/bin/env python3
"""NORDOI G0.3 R5R: real independent-history evidence preflight, DENY ONLY.

This research verifier distinguishes a locally self-asserted evidence bundle
from a genuinely independently authenticated, externally monotonic custodian.
No amount of plausible JSON, same-UID signatures, or current local SQLite
history can attest to ownership, operator separation or global high-water.
It intentionally contains NO native admission, enrollment or production path.
"""
import argparse
import hashlib
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
import g03_r5q_coherent_local_rollback as histories

PROTOCOL="NORDOI-G03-R5R-INDEPENDENT-HISTORY-PREFLIGHT-V1"
PROJECT="eddarsglobal/nordoi"
PARENT="8d15f9661d97a8cc30a1eca66322be28ca500abc"
OPERATION="OBSERVE_ONLY_NEVER_ENROLL"
MAX_BYTES=6144
GATES=(
    "owner_oob_root_authenticated",
    "guardian_legal_identity_established",
    "guardian_administrative_control_separated",
    "guardian_key_pinned_through_owner_oob",
    "live_guardian_signature_independently_verified",
    "signed_fresh_owner_challenge_bound",
    "signed_checkpoint_chain_history_bound",
    "external_monotonic_height_protected",
    "external_revocation_floor_protected",
    "external_spent_nonce_ledger_protected",
    "independent_quorum_proven",
    "independent_recovery_rotation_revocation_proven",
)
FIELDS=frozenset((
    "protocol","project","parent","operation","nonce",
    "local_projection_sha256","claimed_guardian_id","claimed_owner_root_sha256",
    "claimed_external_height","claimed_external_spent_nonces",
    "self_declared_controls",
))

class Refused(ValueError): pass

def canonical(x):
    return (json.dumps(x,sort_keys=True,separators=(",",":"),
                       ensure_ascii=True,allow_nan=False)+"\n").encode("ascii")

def pairs_unique(pairs):
    d={}
    for k,v in pairs:
        if k in d:raise Refused("DUPLICATE_LOCAL_ASSERTION_KEY")
        d[k]=v
    return d

def no_constant(_):raise Refused("NONFINITE_LOCAL_ASSERTION")

def issha(x):
    return type(x) is str and re.fullmatch(r"[0-9a-f]{64}",x) is not None

def parse(raw):
    if type(raw) is not bytes or not 0<len(raw)<=MAX_BYTES:
        raise Refused("MISSING_OR_OVERSIZED_ASSERTION")
    try:
        doc=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs_unique,
                       parse_constant=no_constant)
    except (json.JSONDecodeError,UnicodeError) as e:
        raise Refused("LOCAL_ASSERTION_NOT_VALID_UTF8_JSON") from e
    if type(doc) is not dict or doc.keys()!=FIELDS or canonical(doc)!=raw:
        raise Refused("LOCAL_ASSERTION_FIELDS_OR_CANONICAL_FAILURE")
    if (doc["protocol"],doc["project"],doc["parent"],doc["operation"])!=(
        PROTOCOL,PROJECT,PARENT,OPERATION):
        raise Refused("CANDIDATE_SOURCE_OR_OPERATION_NOT_PINNED")
    for k in ("nonce","local_projection_sha256","claimed_owner_root_sha256"):
        if not issha(doc[k]):raise Refused("CANDIDATE_INVALID_HEX_"+k)
    if (type(doc["claimed_guardian_id"]) is not str or
        re.fullmatch(r"[a-z][a-z0-9.-]{2,79}",doc["claimed_guardian_id"]) is None):
        raise Refused("GUARDIAN_NAME_NOT_CANONICAL")
    for k in ("claimed_external_height","claimed_external_spent_nonces"):
        if type(doc[k]) is not int or doc[k]<0:
            raise Refused("INVALID_SELF_ASSERTED_COUNTER_"+k)
    declared=doc["self_declared_controls"]
    if type(declared) is not dict or set(declared)!=set(GATES):
        raise Refused("MISSING_OR_UNKNOWN_REQUIRED_EXTERNAL_CONTROL")
    for name in GATES:
        if type(declared[name]) is not bool:
            raise Refused("NOT_A_BOOL_EXTERNAL_CONTROL_"+name)
    return doc

def create_claim(local_projection,nonce="b"*64,all_claims=True):
    if not issha(nonce):raise Refused("BAD_RESEARCH_NONCE")
    return canonical({
        "protocol":PROTOCOL,"project":PROJECT,"parent":PARENT,
        "operation":OPERATION,"nonce":nonce,
        "local_projection_sha256":histories.digest_projection(local_projection),
        "claimed_guardian_id":"self-controlled-guardian",
        "claimed_owner_root_sha256":"c"*64,
        "claimed_external_height":local_projection["primary_height"],
        "claimed_external_spent_nonces":local_projection["local_spent_nonce_height"],
        "self_declared_controls":{name:bool(all_claims) for name in GATES},
    })

def block(reason,**kw):
    return denial.block(reason,native_authority_granted=False,
                        independent_history_authority="NOT_ENROLLED",
                        external_monotonic_highwater="NOT_VERIFIED",
                        global_spent_nonce_history="NOT_VERIFIED",
                        **kw)

def examine(a,b,j,candidate_path=None,after_first_observation=None):
    """Checks real evidence, then refuses any locally supplied 'custody' proof.

    The supplied path is an untrusted source. Even a genuine local SSHSIG
    signature would prove local key possession, NOT independent custody.
    """
    try:
        state=histories.projection(a,b,j)
        local_digest=histories.digest_projection(state)
        if candidate_path is None:
            return block("REAL_EXTERNAL_CUSTODY_EVIDENCE_NOT_PROVIDED",
                         local_projection_sha256=local_digest,
                         external_provenance_verified=False)
        p=Path(candidate_path)
        with fdgate.OpenSnapshot(p) as held:
            doc=parse(p.read_bytes())
            if hashlib.sha256(p.read_bytes()).hexdigest()!=held.digest:
                raise Refused("LOCAL_ASSERTION_BYTES_CHANGED")
            held.check()
            if after_first_observation is not None:
                if not callable(after_first_observation):
                    raise Refused("INVALID_EVIDENCE_INTERLEAVING")
                after_first_observation()
            held.check()
        if doc["local_projection_sha256"]!=local_digest:
            raise Refused("LOCAL_ASSERTION_WRONG_HISTORY_DIGEST")
        if doc["claimed_external_height"]<state["primary_height"]:
            raise Refused("CLAIMED_EXTERNAL_HEIGHT_BEHIND_LOCAL")
        if doc["claimed_external_spent_nonces"]<state["local_spent_nonce_height"]:
            raise Refused("CLAIMED_GLOBAL_NONCE_FLOOR_BEHIND_LOCAL")
        if not all(doc["self_declared_controls"].values()):
            return block("REQUIRED_EXTERNAL_CONTROL_CLAIM_INCOMPLETE",
                         local_projection_sha256=local_digest,
                         external_provenance_verified=False)
        # Absolutely no locally supplied booleans or same-admin signed key
        # may cross this boundary. External OOB root and custodian not enrolled.
        return block("SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE",
                     local_projection_sha256=local_digest,
                     all_twelve_self_claimed=True,
                     independently_enrolled_owner_root=False,
                     external_provenance_verified=False)
    except (Refused,histories.Refused,fdgate.Refused,denial.Refused,
            OSError,ValueError,TypeError) as e:
        return block("UNTRUSTED_HISTORY_CUSTODY_EVIDENCE_QUARANTINED",
                     detail=str(e) if isinstance(e,(Refused,histories.Refused,
                                                   fdgate.Refused,denial.Refused))
                     else type(e).__name__,
                     external_provenance_verified=False)

def actual_same_user_sign_and_verify(root,doc):
    """Actual OpenSSH Ed25519 cryptography under one OS user; NOT custody."""
    base=Path(root)
    key=base/"self_key"
    path=base/"self_signed_claim.json"
    allowed=base/"allowed_signers"
    path.write_bytes(doc)
    if os.name!="nt":os.chmod(path,0o600)
    def run(args,input=None):
        x=subprocess.run(args,capture_output=True,input=input,timeout=35,check=False)
        return x
    keygen=run(["/usr/bin/ssh-keygen","-q","-t","ed25519","-N","","-f",str(key)])
    if keygen.returncode!=0:raise Refused("REAL_SAME_USER_SSH_KEYGEN_FAILED")
    allowed.write_text("self-fixture "+(base/"self_key.pub").read_text(),encoding="ascii")
    if os.name!="nt":os.chmod(allowed,0o600)
    signed=run(["/usr/bin/ssh-keygen","-Y","sign","-f",str(key),"-n",
                "nordoi-r5r",str(path)])
    signature=Path(str(path)+".sig")
    if signed.returncode!=0 or not signature.is_file():
        raise Refused("REAL_SAME_USER_ED25519_SIGN_FAILED")
    check=["/usr/bin/ssh-keygen","-Y","verify","-f",str(allowed),
           "-I","self-fixture","-n","nordoi-r5r","-s",str(signature)]
    good=run(check,input=doc)
    tampered=run(check,input=doc+b"tampered")
    return (good.returncode==0,tampered.returncode!=0)

def native_exercise():
    if os.name=="nt":return block("POSIX_REAL_EXTERNAL_DISQUALIFIER_LAB_UNAVAILABLE")
    with tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5r-native-") as tmp:
        root=Path(tmp);os.chmod(root,0o700)
        a,b,j=histories.fixture(root)
        state_a=histories.projection(a,b,j)
        path=root/"untrusted_claim.json"
        claim_a=create_claim(state_a)
        path.write_bytes(claim_a);os.chmod(path,0o600)
        claimed=examine(a,b,j,path)
        if claimed.get("reason")!="SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE":
            raise Refused("SELF_ASSERTED_EXTERNAL_AUTHORITY_MUST_BE_BLOCKED")
        real_crypto,modified_rejected=actual_same_user_sign_and_verify(root,claim_a)
        if not real_crypto or not modified_rejected:
            raise Refused("REAL_OPENSSH_SIGNATURE_NEGATIVE_CONTROL_FAILED")
        if examine(a,b,j,path).get("reason")!="SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE":
            raise Refused("VALID_LOCAL_SIGNATURE_CANNOT_PROVE_INDEPENDENCE")
        snap=histories.file_snapshots(root,a,b,j,"history_a")
        histories.advance_world(a,b,j)
        # Candidate truly bound to old A must NOT match current real B.
        stale=examine(a,b,j,path)
        if "LOCAL_ASSERTION_WRONG_HISTORY_DIGEST" not in stale.get("detail",""):
            raise Refused("OLD_PROVENANCE_NOT_REJECTED_AFTER_REAL_HISTORY_ADVANCE")
        # Copy all old evidence with actual second OS process; local old claim
        # is again consistent but still does not establish an external root.
        histories.actual_second_process_restore(root,snap,a,b,j)
        again=examine(a,b,j,path)
        if again.get("reason")!="SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE":
            raise Refused("RESTORED_HISTORY_NOT_RECOGNIZED_AS_ONLY_LOCAL")
        # And an actual second OS process swapping identical bundle bytes
        # while its original FD remains held MUST be quarantined.
        copy=root/"same_bytes_candidate.json"
        shutil.copy2(path,copy)
        os.chmod(copy,0o600)
        swapped=examine(a,b,j,path,
                        after_first_observation=lambda:fdgate.real_atomic_replace(copy,path))
        if "OPEN_FD_AND_CURRENT_PATH_DIVERGED" not in swapped.get("detail",""):
            raise Refused("SWAPPED_BUNDLE_NOT_QUARANTINED")
        # New real OS process cannot infer a genuine independent custodian.
        cmd=[sys.executable,"-B",str(Path(__file__).resolve()),"--assess",
             "--primary-dir",str(a),"--witness-dir",str(b),"--journal-dir",str(j),
             "--candidate-file",str(path)]
        new=subprocess.run(cmd,text=True,capture_output=True,timeout=35,check=False)
        if new.returncode!=2:raise Refused("NEW_PROCESS_NOT_DENY_ONLY")
        result=json.loads(new.stdout)
        if result.get("reason")!="SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE":
            raise Refused("NEW_PROCESS_FALSELY_ENROLLED_EXTERNAL_GUARDIAN")
        return block("REAL_EXTERNAL_HISTORY_AUTHORITY_ABSENT_CONFIRMED",
                     real_same_uid_ed25519_signature="VALID_KEY_POSSESSION_ONLY",
                     altered_local_signed_claim="SIGNATURE_REJECTED",
                     all_twelve_claims_without_real_oob_root="BLOCK",
                     stale_claim_after_real_history_advance="QUARANTINED",
                     old_local_history_after_joint_rollback="SELF_ASSERTED_ONLY",
                     real_second_process_identical_bundle_swap="QUARANTINED",
                     fresh_os_process_external_enrollment="NOT_ESTABLISHED",
                     verified_external_legal_custodian=False,
                     verified_external_monotonic_height=False,
                     native_admission="BLOCK")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    g=p.add_mutually_exclusive_group(required=True)
    g.add_argument("--exercise",action="store_true")
    g.add_argument("--assess",action="store_true")
    p.add_argument("--primary-dir")
    p.add_argument("--witness-dir")
    p.add_argument("--journal-dir")
    p.add_argument("--candidate-file")
    args=p.parse_args(argv)
    try:
        if args.exercise:result=native_exercise()
        else:result=examine(args.primary_dir,args.witness_dir,args.journal_dir,
                            args.candidate_file)
    except (Refused,histories.Refused,fdgate.Refused,denial.Refused,
            OSError,ValueError,TypeError,subprocess.SubprocessError) as e:
        result=block("EXTERNAL_HISTORY_PREFLIGHT_EXERCISE_REFUSED",
                     detail=str(e) if isinstance(e,(Refused,histories.Refused,
                                                  fdgate.Refused,denial.Refused))
                     else type(e).__name__)
    print(json.dumps(result,sort_keys=True,separators=(",",":")))
    return 2

if __name__=="__main__":sys.exit(main())
