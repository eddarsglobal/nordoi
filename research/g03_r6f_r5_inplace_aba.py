#!/usr/bin/env python3
"""NORDOI G0.3 R6F BUREAU-001 R5 — real local in-place/ABA/multifile audit.

The R4 held-FD guard rejects actual same-admin in-place mutation+restore and
same-inode path ABA under observed POSIX metadata, while the independent
R5Q experiment proves a *new* process cannot detect coherent historical
rollback using only locally restored bytes. These are three different claims.

All experiments use genuine OS file writes, rename, subprocesses and existing
OpenSSH signatures, exclusively in a disposable private research fixture.
No mock signature, external custodian, privileged operation or production.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import g03_r6f_r4_fd_race as r4
import g03_r6f_r3_signer_binding as r3
import g03_r6e_oob_origin_intake as r6e
import g03_r5q_coherent_local_rollback as r5q

PROTOCOL = "NORDOI-G03-R6F-R5-INPLACE-ABA-MULTIFILE-DENY-V1"
PARENT = "4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad"
EXPECTED_QUARANTINE = "R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED"

class Refused(ValueError):
    pass

def deny(reason, **details):
    return r4.deny(reason, **details)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def check_research_path(root, path):
    """Never write outside an exact disposable laboratory root."""
    root = Path(root).resolve(strict=True)
    if not root.name.startswith("nordoi-r5l-r5q-r5s-r5t-"):
        raise Refused("R5_NOT_A_NORDOI_DISPOSABLE_LAB_ROOT")
    path = Path(path)
    if (path.parent != root or path.is_symlink()
            or not path.is_file()):
        raise Refused("R5_PATH_NOT_AN_ORDINARY_FILE_IN_PRIVATE_LAB")
    if os.name != "nt":
        if root.stat().st_mode & 0o077 or path.stat().st_uid != os.getuid():
            raise Refused("R5_DISPOSABLE_LAB_NOT_PRIVATE_OR_NOT_OWNED")
    return path

def second_process_inplace_restore(root, pairs):
    """Actual child writes *modified* bytes then restores original bytes.

    Source and substitute are existing ordinary disposable fixture files.
    The two-file case changes both the candidate .pub and allowed_signers,
    then restores both. The child restores mtime where supported but cannot
    independently authenticate or roll back kernel-managed ctime.
    """
    if not r4.is_posix_ssh():
        raise Refused("R5_REAL_POSIX_FILE_EXERCISE_REQUIRED")
    root = Path(root).resolve(strict=True)
    if type(pairs) not in (list, tuple) or not 1 <= len(pairs) <= 3:
        raise Refused("R5_INVALID_PAIR_COUNT")
    checked = []
    for target, alternate in pairs:
        p = check_research_path(root, target)
        q = check_research_path(root, alternate)
        if p == q or p in (a for a,b in checked):
            raise Refused("R5_REPEATED_OR_IDENTICAL_PAIR")
        if p.read_bytes() == q.read_bytes():
            raise Refused("R5_NO_REAL_ALTERNATE_BYTES")
        checked.append((p, q))
    program = (
        "import json,os,sys,time,hashlib;"
        "pairs=json.loads(sys.argv[1]);"
        "before=[(os.stat(p),open(p,'rb').read()) for p,q in pairs];"
        "assert all(os.stat(p).st_ino!=0 for p,q in pairs);"
        "exec('for p,q in pairs:\\n"
        " with open(p,\"wb\") as f:\\n"
        "  f.write(open(q,\"rb\").read());f.flush();os.fsync(f.fileno())');"
        "time.sleep(.02);"
        "exec('for (p,q),(st,raw) in zip(pairs,before):\\n"
        " with open(p,\"wb\") as f:\\n"
        "  f.write(raw);f.flush();os.fsync(f.fileno())\\n"
        " os.utime(p,ns=(st.st_atime_ns,st.st_mtime_ns))');"
        "after=[(os.stat(p),open(p,'rb').read()) for p,q in pairs];"
        "result={'actual_os_file_writes':True,"
        "'all_original_bytes_restored':all(x[1]==y[1] for x,y in zip(before,after)),"
        "'all_original_inodes_restored':all(x[0].st_ino==y[0].st_ino for x,y in zip(before,after)),"
        "'all_original_mtime_restored':all(x[0].st_mtime_ns==y[0].st_mtime_ns for x,y in zip(before,after)),"
        "'kernel_ctime_changed':any(x[0].st_ctime_ns!=y[0].st_ctime_ns for x,y in zip(before,after))};"
        "print(json.dumps(result,sort_keys=True))"
    )
    p = subprocess.run([sys.executable, "-B", "-c", program,
                        json.dumps([[str(a), str(b)] for a,b in checked])],
                       text=True, capture_output=True, check=False, timeout=35)
    if p.returncode:
        raise Refused("R5_SECOND_PROCESS_INPLACE_RESTORE_FAILED:"
                      + p.stderr[-600:])
    result = json.loads(p.stdout)
    if not (result["actual_os_file_writes"] and
            result["all_original_bytes_restored"] and
            result["all_original_inodes_restored"]):
        raise Refused("R5_INPLACE_RESTORE_DID_NOT_PRESERVE_ORIGINAL_BYTES")
    return result

def second_process_path_aba(root, target, alternate):
    """Real A->B->A *path* ABA, old inode restored, no synthetic swap."""
    if not r4.is_posix_ssh():
        raise Refused("R5_REAL_POSIX_FILE_EXERCISE_REQUIRED")
    root = Path(root).resolve(strict=True)
    p = check_research_path(root, target)
    q = check_research_path(root, alternate)
    if p == q or p.read_bytes() == q.read_bytes():
        raise Refused("R5_ABA_REQUIRES_DIFFERENT_FILES")
    hold = root / "nordoi-r5-aba-held-original"
    if hold.exists() or hold.is_symlink():
        raise Refused("R5_ABA_HOLD_PATH_ALREADY_USED")
    program = (
        "import os,sys,json,hashlib;"
        "a,b,hold=sys.argv[1:4];"
        "pre=os.stat(a);raw=open(a,'rb').read();"
        "os.rename(a,hold);os.replace(b,a);os.rename(a,b);os.rename(hold,a);"
        "post=os.stat(a);"
        "print(json.dumps({'real_os_rename_sequence':True,"
        "'original_inode_returned':pre.st_dev==post.st_dev and pre.st_ino==post.st_ino,"
        "'original_bytes_returned':open(a,'rb').read()==raw,"
        "'kernel_ctime_changed':pre.st_ctime_ns!=post.st_ctime_ns},sort_keys=True))"
    )
    out = subprocess.run([sys.executable, "-B", "-c", program,
                          str(p), str(q), str(hold)],
                         capture_output=True, text=True, timeout=35,
                         check=False)
    if out.returncode:
        raise Refused("R5_TRUE_SECOND_PROCESS_ABA_RENAME_FAILED:"+out.stderr[-600:])
    result = json.loads(out.stdout)
    if not (result["real_os_rename_sequence"] and
            result["original_inode_returned"] and
            result["original_bytes_returned"]):
        raise Refused("R5_REAL_PATH_ABA_NOT_COMPLETE")
    return result

def run_guarded_interleaving(data, kind):
    """Actual R4 native verification, with deterministic real child mutation."""
    if kind not in ("single_inplace", "paired_inplace", "path_aba"):
        raise Refused("R5_UNKNOWN_REAL_INTERLEAVING")
    root = data["root"]
    rogue_signature, rogue_allowed, rogue_pub = r3.create_rogue_signed_receipt(data)
    victim = root / ("nordoi-r5-victim-"+kind+".allowed")
    r4.copy_private(data["guardian_allowed"], victim)
    if kind == "paired_inplace":
        fake_pub = root / "nordoi-r5-rogue-pub-inplace"
        r4.copy_private(rogue_pub, fake_pub)
    seen = []
    def actual_write_between_check_and_use(command):
        if seen:
            return
        if kind == "single_inplace":
            x = second_process_inplace_restore(root, [(victim, rogue_allowed)])
        elif kind == "paired_inplace":
            x = second_process_inplace_restore(
                root, [(victim, rogue_allowed),
                       (data["guardian_public"], fake_pub)]
            )
        else:
            alt = root / "nordoi-r5-aba-alternate"
            r4.copy_private(rogue_allowed, alt)
            x = second_process_path_aba(root, victim, alt)
        seen.append(x)
    # Use the valid original candidate signature: the second process changes
    # and restores same-admin evidence during the actual six-signature path.
    result = r4.assess_lab(
        data, local_allowed=victim,
        before_verify=actual_write_between_check_and_use
    )
    if not seen:
        raise Refused("R5_TRUE_INTERLEAVING_NOT_REACHED")
    if result["reason"] != EXPECTED_QUARANTINE:
        raise Refused("R5_HISTORIC_R4_GUARD_DID_NOT_QUARANTINE_REAL_ABA:"
                      + str(result.get("reason")))
    return result, seen[0]

def inspect_cross_epoch():
    """Existing real R5Q experiment: after coherent rollback, new process is blind."""
    result = r5q.native_exercise()
    if result.get("reason") != "REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED":
        raise Refused("R5Q_COHERENT_LOCAL_ROLLBACK_LAB_DID_NOT_COMPLETE")
    for key in ("same_admin_all_three_restored",
                "fresh_os_process_observed_old_local_history",
                "globally_spent_nonce_reissued_after_full_rollback"):
        if result.get(key) is not True:
            raise Refused("R5Q_REQUIRED_REAL_CROSS_EPOCH_FINDING_MISSING:"+key)
    if result.get("protected_independent_high_water") is not False:
        raise Refused("R5Q_FALSE_EXTERNAL_AUTHORITY_CLAIM")
    return result

def exercise():
    if not r4.is_posix_ssh():
        return deny("R5_REAL_POSIX_OPENSSH_OR_FD_UNAVAILABLE"), 2
    reports = {}
    for kind in ("single_inplace", "paired_inplace", "path_aba"):
        home, data = r6e.create_lab()
        try:
            result, process_report = run_guarded_interleaving(data, kind)
            reports[kind] = dict(
                original_bytes_restored=process_report[
                    "all_original_bytes_restored" if kind!="path_aba"
                    else "original_bytes_returned"],
                actual_child_process=True,
                os_metadata_changed=process_report["kernel_ctime_changed"],
                r4_reason=result["reason"],
            )
        finally:
            home.cleanup()
    historical = inspect_cross_epoch()
    if any(not v["os_metadata_changed"] for v in reports.values()):
        raise Refused("R5_FILESYSTEM_DID_NOT_EXPOSE_EXPECTED_CTIME_CHANGE")
    return deny(
        "R5_REAL_INPLACE_ABA_MULTIFILE_DETECTED_BUT_CROSS_EPOCH_CUSTODY_ABSENT",
        actual_child_inplace_mutation_and_restore=True,
        actual_child_paired_multifile_mutation_and_restore=True,
        actual_child_atomic_same_inode_path_aba=True,
        original_bytes_restored_in_each_case=True,
        fd_metadata_detected_changes_under_actual_os=True,
        prior_r4_guard_refused_all_three_interleavings=True,
        existing_r5q_real_coherent_local_rollback_replayed=True,
        coherent_local_multi_file_rollback_invisible_to_new_examiner=True,
        independent_external_monotonic_custody_proven=False,
        same_admin_memory_or_kernel_control_proven_safe=False,
    ), 2

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exercise", action="store_true", required=True)
    parser.parse_args(argv)
    try:
        result, status = exercise()
    except (Refused, r4.Refused, r3.Refused, r6e.Refused,
            OSError, TypeError, ValueError, KeyError,
            subprocess.SubprocessError) as exc:
        result, status = deny(
            "R5_REAL_ABA_MULTIFILE_EXERCISE_QUARANTINED",
            detail=str(exc) if isinstance(exc,(Refused,r4.Refused,
                                                r3.Refused,r6e.Refused))
                   else type(exc).__name__), 3
    print(json.dumps(result, sort_keys=True))
    return status

if __name__ == "__main__":
    sys.exit(main())
