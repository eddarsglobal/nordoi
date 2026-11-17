#!/usr/bin/env python3
"""BUREAU-001 E4: actual coherent tampering of real machine evidence copies.

This is a boundary test. An E3 verifier cannot tell a pair of coherently
modified log+manifest files from unmodified originals when the same actor
can write both. Show this on copies of actual 9 machine logs, with the
exact source HEAD still pinned and no changes to the source or genuine logs.

An E4 local immutable-in-the-process prior snapshot detects divergence if
it is not altered. This is NOT an independently protected external witness,
cryptographic operator identity, global antirollback or production admission.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0,str(Path(__file__).resolve().parent))
import bureau_001_e3_exact_head_provenance as e3
import bureau_001_e2_machine_only as e2

SCHEMA="NORDOI-G03-R6F-BUREAU001-E4-COHERENT-EVIDENCE-TAMPERING-V1"
POLICY="governance/NORDOI_R6F_BUREAU001_E4_COHERENT_EVIDENCE_TAMPERING.json"
PARENT_E3="655ea42e5d4c7a62fb1506243c15706044b6ae29"
TARGET_ROLE="ATLAS"
MARK=b"E4_COORDINATED_EDIT_TO_REAL_MACHINE_LOG_NOT_A_NEW_TEST\n"
ROLES=e3.ROLES

class Blocked(ValueError):pass

def need(v,reason):
    if not v:raise Blocked(reason)

def sha(raw):return hashlib.sha256(raw).hexdigest()

def audit(root,expected_head):
    root=Path(root).resolve(strict=True)
    e3data=e3.audit(root,expected_head)
    policy=e3.strictly_load(e3.real_file_bytes(root/POLICY))
    obs=policy["observed_e3"]
    need(policy["schema"]==SCHEMA and
         policy["project"]=="eddarsglobal/nordoi" and
         policy["source_parent_e3_head"]==PARENT_E3 and
         policy["hypothesis"].startswith("In a same-admin mutable evidence directory") and
         policy["attack_target"]=="ATLAS.log" and
         policy["attack_marker"]==MARK.decode().strip() and
         obs["exact_candidate_head"]==PARENT_E3 and
         obs["actual_machine_nine_suites"]=="9/9_PASS_ON_EXACT_HEAD" and
         obs["machine_job_id"]==133588850999 and
         obs["machine_artifact_id"]==14266742126 and
         obs["reported_manifest_sha256"]==
             "4e910cfa9fc1f8aad64027148b68dc621abeec3915207e30877af5daac039e4a" and
         obs["four_platform_source_jobs"]=={
             "macos_intel":"SUCCESS","windows":"SUCCESS",
             "ubuntu":"SUCCESS","macos_arm":"CANCELLED_NOT_PASS"} and
         obs["general_rust_jobs"]=="5/5_SUCCESS",
         "E4_WRONG_HISTORICAL_OR_NONPARENT_SOURCE")
    need(policy["e3_expected_result_for_tampered_coherent_copy"]==
             "ACCEPTS_WHEN_ALL_MUTABLE_FIELDS_ARE_UPDATED" and
         policy["e4_expected_with_unchanged_prior_anchor"]==
             "REJECTS_COUPLED_TAMPER" and
         policy["e4_expected_with_rewritten_anchor"]==
             "LIMIT_UNRESOLVED_SAME_ADMIN_CAN_REWRITE_ANCHOR" and
         policy["anchor_authenticity"]=="UNAUTHENTICATED_SAME_ADMIN_LOCAL" and
         policy["actual_external_trust_verified"] is False and
         policy["no_human_testers"] is True and
         policy["independent_specialist_reviews"]==0 and
         policy["owner_oob_root"]=="NOT_AUTHENTICATED" and
         policy["global_monotonic_antirollback"]=="NOT_VERIFIED" and
         policy["r6f"]=="IN_PROGRESS_UNCERTIFIED" and
         policy["r6g"]=="LOCKED_BY_SEQUENCE" and
         policy["native_admission_gates_unmet"]==12 and
         policy["production"]=="NOT_AUTHORIZED" and
         policy["no_privileged_transition"] is True,
         "E4_FALSE_EXTERNAL_CERTIFICATION_OR_NON_NEGATIVE_OPERATION")
    return dict(
        result="E4_SOURCE_AUDIT_BOUNDARY_PASS",
        exact_pr_head=e3data["checked_out_head"],
        prior_e3_exact_head=PARENT_E3,
        apple_silicon_original="CANCELLED_NOT_PASS",
        scope="SAME_ADMIN_COHERENT_MUTATION_OF_REAL_MACHINE_EVIDENCE_COPIES",
        external_anchor_independent=False,
        human_testers=0,independent_specialist_reviews=0,
        native_admission_gates_unmet=12,r6g="LOCKED_BY_SEQUENCE",
        production="NOT_AUTHORIZED")

def get_raw_evidence(directory):
    directory=Path(directory).resolve(strict=True)
    need(directory.is_dir(),"E4_NOT_DIRECTORY")
    man=e3.real_file_bytes(directory/"E2_MACHINE_EVIDENCE.json")
    logs={role:e3.real_file_bytes(directory/(role+".log")) for role in ROLES}
    return man,logs

def local_anchor(manifest_bytes,logs,source_head):
    """A PRIOR local digest snapshot, NOT a signer/custodian or external floor."""
    need(type(source_head) is str and bool(e3.HEX40.fullmatch(source_head)),
         "E4_INVALID_HEAD")
    need(type(logs) is dict and tuple(logs)==ROLES,"E4_BAD_ROLE_SET")
    return dict(
        schema="NORDOI-E4-LOCAL-PRECOMMIT-SNAPSHOT-NO-EXTERNAL-AUTHORITY-V1",
        source_head=source_head,manifest_sha256=sha(manifest_bytes),
        nine_log_sha256={role:sha(logs[role]) for role in ROLES},
        signer=None,external_custodian=False,
        same_admin_mutable=True)

def against_anchor(anchor,man,logs,head):
    need(anchor["schema"]==
          "NORDOI-E4-LOCAL-PRECOMMIT-SNAPSHOT-NO-EXTERNAL-AUTHORITY-V1" and
         anchor["source_head"]==head and
         anchor["signer"] is None and
         anchor["external_custodian"] is False and
         anchor["same_admin_mutable"] is True,
         "E4_UNVERIFIED_OR_FICTITIOUS_EXTERNAL_WITNESS")
    need(tuple(anchor["nine_log_sha256"])==ROLES,
         "E4_MISSING_ANCHOR_LOG_DIGESTS")
    need(sha(man)==anchor["manifest_sha256"],
         "E4_PRIOR_MANIFEST_DIGEST_MISMATCH")
    for role in ROLES:
        need(sha(logs[role])==anchor["nine_log_sha256"][role],
             "E4_PRIOR_LOG_DIGEST_MISMATCH:"+role)
    return True

def write_atomic_file(path,data):
    # Exercise only on an isolated copy directory, not the original evidence.
    path=Path(path)
    tmp=path.parent/(path.name+".E4_staged")
    need(not tmp.exists(),"E4_STAGED_FILE_ALREADY_EXISTS")
    with tmp.open("xb") as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp,path)

def copy_actual(directory,target):
    """Copy from true evidence; no original files modified or test faked."""
    source=Path(directory).resolve(strict=True)
    dest=Path(target)
    need(not dest.exists(),"E4_TARGET_EXISTS")
    dest.mkdir(mode=0o700)
    for name in ["E2_MACHINE_EVIDENCE.json"]+[r+".log" for r in ROLES]:
        contents=e3.real_file_bytes(source/name)
        (dest/name).write_bytes(contents)
    return dest

def coherent_edit(directory):
    """Coherent same-admin mutation of a *copy* of an actual prior test log."""
    directory=Path(directory)
    original=e3.real_file_bytes(directory/(TARGET_ROLE+".log"))
    need(b"test result: ok." in original and
         b"10 passed; 0 failed; 0 ignored" in original and
         MARK not in original,
         "E4_TRUE_ORIGINAL_NATIVE_ATLAS_TEST_LOG_NOT_PRESENT")
    doc=e3.strictly_load(e3.real_file_bytes(directory/"E2_MACHINE_EVIDENCE.json"))
    records=doc["machine_results"]
    need(type(records) is list and
         tuple(x["role"] for x in records)==ROLES,"E4_NINE_ROLE_SET_CHANGED")
    altered=MARK+original
    write_atomic_file(directory/(TARGET_ROLE+".log"),altered)
    records[0]["log_sha256"]=sha(altered)
    write_atomic_file(directory/"E2_MACHINE_EVIDENCE.json",
                      (json.dumps(doc,sort_keys=True,indent=2)+"\n").encode())
    return original,altered

def e3_separate_process(root,head,evidence_dir):
    p=subprocess.run(
       [sys.executable,"-B",str(Path(root)/"scripts/bureau_001_e3_exact_head_provenance.py"),
        "--verify","--root",str(root),"--expected-head",head,
        "--evidence-dir",str(evidence_dir)],
       cwd=root,capture_output=True,text=True,timeout=120)
    try:value=json.loads(p.stdout)
    except (ValueError,UnicodeError) as ex:
        raise Blocked("E4_E3_REAL_SEPARATE_PROCESS_OUTPUT_MISSING") from ex
    need(p.returncode==0 and value.get("role_logs_sha256_verified")==9,
         "E4_EXPECTED_REAL_E3_ACCEPTANCE_NOT_OBSERVED")
    return value

def exercise(root,expected_head,evidence_dir):
    root=Path(root).resolve(strict=True)
    baseline=audit(root,expected_head)
    p=Path(evidence_dir).resolve(strict=True)
    need(p.is_dir() and p!=root and root not in p.parents,
         "E4_REAL_EVIDENCE_NOT_OUTSIDE_REPOSITORY")
    # Check actual source-bound evidence first, using real prior E3 verifier.
    original=e3_separate_process(root,expected_head,p)
    raw_man,raw_logs=get_raw_evidence(p)
    anchor=local_anchor(raw_man,raw_logs,expected_head)
    need(against_anchor(anchor,raw_man,raw_logs,expected_head),
         "E4_PRIOR_LOCAL_ANCHOR_INCONSISTENT_BEFORE_TAMPER")
    with tempfile.TemporaryDirectory(prefix="nordoi_e4_actual_coherent_") as d:
        dest=copy_actual(p,Path(d)/"isolated_copy")
        saved_original,tampered=coherent_edit(dest)
        need(sha(tampered)!=sha(saved_original),
             "E4_REAL_LOG_CONTENT_NOT_CHANGED")
        forged_man,forged_logs=get_raw_evidence(dest)
        # A genuinely new process runs original E3 on the actual altered files:
        # it ACCEPTS coherent forged content, despite correct commit/source.
        e3_accepts=e3_separate_process(root,expected_head,dest)
        need(e3_accepts["exact_source_head"]==expected_head,
             "E4_FORGED_COHERENT_HEAD_NOT_EXACT")
        try:against_anchor(anchor,forged_man,forged_logs,expected_head)
        except Blocked as ex:
            need(str(ex)=="E4_PRIOR_MANIFEST_DIGEST_MISMATCH",
                 "E4_UNEXPECTED_LOCAL_ANCHOR_REJECTION")
            prior_anchor_detected=True
        else:raise Blocked("E4_TAMPER_ESCAPED_UNALTERED_PRIOR_ANCHOR")
        # Illustrate the impossibility of a same-admin *independent* anchor:
        # rewriting the local snapshot against forged bytes also yields PASS.
        co_modified_anchor=local_anchor(forged_man,forged_logs,expected_head)
        need(against_anchor(co_modified_anchor,forged_man,forged_logs,
                            expected_head),
             "E4_EXPECTED_REWRITTEN_SAME_ADMIN_ANCHOR_REPRO_NOT_SEEN")
        # Verify real originals remain untouched.
        after_man,after_logs=get_raw_evidence(p)
        need(after_man==raw_man and after_logs==raw_logs,
             "E4_ORIGINAL_MACHINE_LOGS_CHANGED")
        need(e3.check_head(root,expected_head)==expected_head,
             "E4_GIT_SOURCE_CHANGED_DURING_LAB")
    return dict(
       result="E4_REAL_NINE_MACHINE_EVIDENCE_COHERENT_TAMPER_REPRODUCED",
       exact_source_head=expected_head,
       real_original_e3_nine_logs_verified=original["role_logs_sha256_verified"],
       forged_copy_e3_accepted_in_new_process=True,
       forged_copy_kept_original_machine_test_counts=True,
       same_admin_log_and_manifest_both_modified=True,
       unchanged_prior_local_anchor_detected=prior_anchor_detected,
       rewritten_same_admin_anchor_passes_again=True,
       true_independent_anchor_exists=False,
       original_machine_evidence_untouched=True,
       experimental_scope="REAL_FILESYSTEM_OS_PROCESSES_AND_ACTUAL_NINE_MACHINE_LOGS",
       real_human_testers=0,independent_specialist_reviews=0,
       real_external_custodian="NOT_ENROLLED",
       owner_oob_root="NOT_AUTHENTICATED",
       global_monotonic_antirollback="NOT_VERIFIED",
       formal_r6f_closure="HOLD_MISSING_INDEPENDENT_EVIDENCE",
       r6f="IN_PROGRESS_UNCERTIFIED",r6g="LOCKED_BY_SEQUENCE",
       native_admission_gates_unmet=12,production="NOT_AUTHORIZED",
       source_audit=baseline["result"])

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--audit",action="store_true")
    group.add_argument("--exercise",action="store_true")
    parser.add_argument("--root",type=Path,
                        default=Path(__file__).resolve().parents[1])
    parser.add_argument("--expected-head",required=True)
    parser.add_argument("--evidence-dir",type=Path)
    x=parser.parse_args(argv)
    try:
        if x.audit:res=audit(x.root,x.expected_head)
        else:
            need(x.evidence_dir is not None,
                 "E4_ACTUAL_MACHINE_EVIDENCE_DIR_REQUIRED")
            res=exercise(x.root,x.expected_head,x.evidence_dir)
        print(json.dumps(res,sort_keys=True))
        return 0
    except (Blocked,e3.Rejected,e2.EvidenceError,ValueError,TypeError,
            KeyError,OSError,subprocess.SubprocessError) as ex:
        print(json.dumps({
            "result":"E4_REMEDIATE_EVIDENCE_OR_LAB_NOT_VERIFIED",
            "reason":str(ex) if isinstance(ex,(Blocked,e3.Rejected,e2.EvidenceError))
                     else type(ex).__name__,
            "human_testers":0,"real_external_custodian":"NOT_ENROLLED",
            "r6g":"LOCKED_BY_SEQUENCE","native_admission_gates_unmet":12,
            "production":"NOT_AUTHORIZED"},sort_keys=True))
        return 3

if __name__=="__main__":
    raise SystemExit(main())
