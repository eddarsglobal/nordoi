#!/usr/bin/env python3
"""NORDOI G0.3 R3-R20: strict pinned integration-readiness audit.

This is NOT a merge, source change, benchmark, production approval or human
signature gate. Both original and R3-R13 candidate are tested, separately.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CAND_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
CAND_BRANCH = "candidate/g03-r3-r13-eliminate-redundant-atom-check-20261108"
REPO = "https://github.com/eddarsglobal/nordoi.git"
SCHEMA = "NORDOI-G03-R3-R20-INTEGRATION-READINESS-v1"
BASE_KERNEL_HASH = "dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2"
CAND_KERNEL_HASH = "35194549eee9d6798de2c9afb091a668c2d491a40c548693ee6e87a3aa29aba0"
SAFETY_GIT_BLOB = "4080350aab5c3b2f377f7c22bca54d49c50e6df7"
SAFETY_RECEIPT = b"G03_R3_R16_SECURITY_PROBE=PASS\nPRODUCTION=NOT_AUTHORIZED\n"
NEW_FILES = {
    ".github/workflows/g03_r3_r13_paired.yml",
    "research/G03_R3_R13_ENGINEERING.md",
    "research/g03_r3_r13_kernel_probe.rs",
    "research/g03_r3_r13_paired_mac.py",
    "tests/g03_r3_r13_redundant_lookup.rs",
}
OLD = (b"    fn apply_single(&mut self, id: AtomId, next: Value) -> AtomicResult<bool> {\n"
       b"        self.ensure_exists(id)?;\n\n"
       b"        // Preserve the No Work Without Effect law: an identical write exits")
NEW = (b"    fn apply_single(&mut self, id: AtomId, next: Value) -> AtomicResult<bool> {\n"
       b"        // self.get(id)? below already rejects UnknownAtom. A second\n"
       b"        // ensure_exists HashMap probe here is redundant on every write.\n"
       b"        // Ownership is still checked in set_owned before this function.\n"
       b"        // Preserve the No Work Without Effect law: an identical write exits")
TEST_LINE = re.compile(r"test result: ok\.\s+(\d+) passed;\s+(\d+) failed;")

class Blocked(Exception):
    pass

def sha256(payload):
    return hashlib.sha256(payload).hexdigest()

def git_blob(payload):
    return hashlib.sha1(b"blob "+str(len(payload)).encode("ascii")+b"\0"+payload).hexdigest()

def patch_exact(base,cand):
    if base.count(OLD)!=1 or base.replace(OLD,NEW)!=cand:
        raise Blocked("candidate kernel change outside exact single-lookup replacement")
    return True

def verify_kernel(base, cand):
    if sha256(base)!=BASE_KERNEL_HASH or sha256(cand)!=CAND_KERNEL_HASH:
        raise Blocked("frozen kernel content changed")
    patch_exact(base,cand)
    return {"source_delta":"EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY",
            "baseline_sha256":sha256(base),"candidate_sha256":sha256(cand)}

def require_expected_inventory(old,new):
    if old-new or new-old!=NEW_FILES or "src/kernel.rs" not in (old&new):
        raise Blocked("tracked file inventory outside exact R3-R13 candidate PR")
    return True

def tracked_inventory(base, cand):
    old=set(subprocess_checked(["git","ls-files","-z"],cwd=base).decode().split("\0"))-{""}
    new=set(subprocess_checked(["git","ls-files","-z"],cwd=cand).decode().split("\0"))-{""}
    require_expected_inventory(old,new)
    for item in sorted(old&new-{"src/kernel.rs"}):
        if (base/item).read_bytes()!=(cand/item).read_bytes():
            raise Blocked("non-kernel shared tracked file differs: "+item)
    return {"only_candidate_added_files":sorted(new-old),
            "removed_files":[],"all_other_common_files_byte_identical":True,
            "common_tracked_files":len(old&new),
            "candidate_added_files":len(new-old)}

def subprocess_checked(command,cwd=None,env=None,timeout=900):
    try:
        r=subprocess.run(command,cwd=cwd,env=env,timeout=timeout,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    except (OSError,subprocess.TimeoutExpired) as e:
        raise Blocked("unavailable/timed out: "+str(command[0])) from e
    if r.returncode:
        message=(r.stdout+b"\n"+r.stderr).decode("utf-8","replace")[-1800:]
        raise Blocked("command rejected: "+str(command[0])+" / "+message)
    return r.stdout

def clone_pinned(to,branch,sha):
    subprocess_checked(["git","-c","core.autocrlf=false","-c","core.eol=lf",
        "clone","--quiet","--depth","1","--branch",branch,REPO,str(to)],timeout=240)
    observed=subprocess_checked(["git","rev-parse","HEAD"],cwd=to,timeout=30).decode().strip()
    if observed!=sha:
        raise Blocked("frozen branch tip drift")
    return observed

def local_safety_source(source):
    p=Path(source)
    if p.is_symlink() or not p.is_file() or p.stat().st_size>12000 or p.stat().st_size<1:
        raise Blocked("security probe file missing/unsafe")
    payload=p.read_bytes()
    # Windows checkout can translate LF to CRLF. Accept only the exact
    # Git-pinned bytes after a line-ending-only restoration; no other change.
    if git_blob(payload)!=SAFETY_GIT_BLOB:
        if b"\r\n" not in payload or b"\r" in payload.replace(b"\r\n",b""):
            raise Blocked("security probe content changed beyond CRLF")
        payload=payload.replace(b"\r\n",b"\n")
        if git_blob(payload)!=SAFETY_GIT_BLOB:
            raise Blocked("security probe Git-pinned LF bytes mismatch")
    return payload

def test_summary(stdout):
    raw=stdout.decode("utf-8","replace")
    found=TEST_LINE.findall(raw)
    if not found or any(int(nfailed)!=0 for _,nfailed in found):
        raise Blocked("Rust all-target test result missing/failed")
    return {"test_result_sections":len(found),
            "passed":sum(int(npassed) for npassed,_ in found),
            "failed":0,
            "test_stdout_sha256":sha256(stdout)}

def run_suite(source,work,safety):
    env=dict(os.environ,CARGO_NET_OFFLINE="true",CARGO_TARGET_DIR=str(work/"target"))
    source=str(source)
    fmt=subprocess_checked(["cargo","fmt","--all","--","--check"],cwd=source,env=env)
    started=time.monotonic()
    tests=subprocess_checked(["cargo","test","--offline","--release","--all-targets"],
        cwd=source,env=env,timeout=1800)
    test_seconds=round(time.monotonic()-started,3)
    count=test_summary(tests)
    subprocess_checked(["cargo","clippy","--offline","--all-targets","--",
         "-D","warnings"],cwd=source,env=env,timeout=1800)
    subprocess_checked(["cargo","build","--offline","--release","--lib"],
         cwd=source,env=env,timeout=900)
    deps=work/"target"/"release"/"deps"
    libs=list(deps.glob("libnordoi_kernel-*.rlib"))
    if len(libs)!=1:
        raise Blocked("Rust release static library missing/ambiguous")
    script=work/"pinned_security.rs"
    script.write_bytes(safety)
    executable=work/("security.exe" if os.name=="nt" else "security")
    subprocess_checked(["rustc","--edition=2021","-C","opt-level=3",
        "-C","codegen-units=1","-L","dependency="+str(deps),
        "--extern","nordoi_kernel="+str(libs[0]),str(script),
        "-o",str(executable)],timeout=900)
    verdict=subprocess_checked([str(executable)],timeout=60).replace(b"\r\n",b"\n")
    if verdict!=SAFETY_RECEIPT:
        raise Blocked("security/atomicity/ownership witness contract not exact")
    return {
        "cargo_fmt":"PASS","cargo_fmt_stdout_sha256":sha256(fmt),
        "cargo_test_release_all_targets":"PASS",
        "cargo_test_result":count,"cargo_test_elapsed_s_not_a_benchmark":test_seconds,
        "cargo_clippy_all_targets_deny_warnings":"PASS",
        "external_security_contract":"PASS",
        "external_security_sha256":sha256(executable.read_bytes()),
        "external_security_checks":["default deny","explicit allow","fresh-kernel deny",
            "unknown atom","ownership denial","noop zero work","transaction conflict"],
        "real_speed_measurements":"NONE"
    }

def save_private(destination,report):
    p=Path(destination)
    if p.exists() or p.is_symlink() or not p.parent.is_dir():
        raise Blocked("private receipt already exists or invalid directory")
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL
    if hasattr(os,"O_NOFOLLOW"):flags|=os.O_NOFOLLOW
    fd=os.open(p,flags,0o600)
    with os.fdopen(fd,"w",encoding="utf-8",newline="\n") as output:
        json.dump(report,output,indent=2,sort_keys=True)
        output.write("\n")
    if os.name=="posix" and (p.stat().st_mode&0o077):
        raise Blocked("report is not private")

def selftest():
    sample=b"header\n"+OLD+b"\nfooter\n"
    # Real source SHA pins are enforced in two-revision check().
    assert patch_exact(sample,sample.replace(OLD,NEW))
    for bad in (sample,sample.replace(OLD,NEW)+b"\nEXTRA\n"):
        try:patch_exact(sample,bad)
        except Blocked:pass
        else:raise AssertionError("non-exact patch accepted")
    assert sha256(sample)!=sha256(sample.replace(OLD,NEW))
    assert git_blob(b"123")=="d800886d9c86731ae5c4a62b0b77c437015e00d2"
    old={"src/kernel.rs","Cargo.toml"}
    good=old|NEW_FILES
    assert require_expected_inventory(old,good)
    for bad in (good|{"src/extra.rs"},old|{"src/kernel.rs"}):
        try:require_expected_inventory(old,bad)
        except Blocked:pass
        else:raise AssertionError("unauthorized tracked inventory accepted")
    assert len(TEST_LINE.findall("test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured;"))==1
    assert not TEST_LINE.findall("error: test failed")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R320_NEGATIVE_") as d:
        root=Path(d)
        base=root/"base";cand=root/"cand"
        base.mkdir();cand.mkdir()
        (base/"src").mkdir();(cand/"src").mkdir()
        (base/"src"/"kernel.rs").write_bytes(sample)
        (cand/"src"/"kernel.rs").write_bytes(sample.replace(OLD,NEW))
        assert (cand/"src"/"kernel.rs").read_bytes()!=sample
        (cand/"src"/"kernel.rs").write_bytes(sample.replace(OLD,NEW)+b"\nTAMPERED\n")
        assert (cand/"src"/"kernel.rs").read_bytes()!=sample.replace(OLD,NEW)
        try:patch_exact(sample,(cand/"src"/"kernel.rs").read_bytes())
        except Blocked:pass
        else:raise AssertionError("tampered candidate passed")
        report=root/"report.json"
        save_private(report,{"synthetic":True})
        try:save_private(report,{})
        except Blocked:pass
        else:raise AssertionError("overwrite protection failed")
    print("G03_R3_R20_SYNTHETIC_NEGATIVE_TESTS=PASS")
    print("REAL_SOURCE_TESTS=NOT_RUN_IN_SELFTEST")
    print("MERGE_AUTHORIZATION=NONE")
    print("PRODUCTION=NOT_AUTHORIZED")

def check(safety_source,out):
    if any(shutil.which(x) is None for x in ("git","cargo","rustc","rustfmt")):
        raise Blocked("required Git/Rust/rustfmt toolchain not present")
    safety=local_safety_source(safety_source)
    with tempfile.TemporaryDirectory(prefix="NORDOI_R320_READINESS_") as folder:
        root=Path(folder)
        orig=root/"original";cand=root/"candidate"
        clone_pinned(orig,"main",BASE_SHA)
        clone_pinned(cand,CAND_BRANCH,CAND_SHA)
        delta=verify_kernel((orig/"src"/"kernel.rs").read_bytes(),
                            (cand/"src"/"kernel.rs").read_bytes())
        inventory=tracked_inventory(orig,cand)
        results={}
        for label,source in (("baseline",orig),("candidate",cand)):
            work=root/("work_"+label)
            work.mkdir()
            results[label]=run_suite(source,work,safety)
        if results["baseline"]["external_security_contract"]!="PASS" or \
           results["candidate"]["external_security_contract"]!="PASS":
            raise Blocked("security checks incomplete")
        report={
            "schema":SCHEMA,"frozen_base_sha":BASE_SHA,"frozen_candidate_sha":CAND_SHA,
            "r13_candidate_pr":40,
            "source_guard":delta,"tracked_inventory":inventory,
            "host":{"system":platform.system(),"machine":platform.machine(),
                    "python":platform.python_version(),"cpu_count":os.cpu_count()},
            "toolchain":{"cargo":subprocess_checked(["cargo","--version"]).decode().strip(),
                         "rustc":subprocess_checked(["rustc","--version"]).decode().strip()},
            "verification":results,
            "historical_performance_evidence":{
                "scope":"OWNER_MAC_R3_R16_FIX1_MICROBENCHMARK_NOT_RERUN",
                "noop_set_ab_percent":23.1812,
                "changed_set_flush_ab_percent":3.9061,
                "denied_capability_ab_percent":-1.3758,
                "denied_capability_ab_delta_ns_per_op":-0.17,
                "denied_capability_root_cause":"NOT_ESTABLISHED",
                "disposition":"EXPLICIT_PERFORMANCE_EXCEPTION_REQUIRES_ENGINEERING_DECISION"},
            "decision":"READY_FOR_GOVERNED_MERGE_DECISION_WITH_PERFORMANCE_EXCEPTION",
            "no_new_performance_measurements":True,
            "human_evaluation_gate":"CLOSED_NOT_REQUIRED",
            "merge_authorization":"NONE","production":"NOT_AUTHORIZED",
            "original_repo_modified":False
        }
        if out:save_private(out,report)
        print("G03_R3_R20_SOURCE_DELTA=PASS")
        print("G03_R3_R20_TRACKED_INVENTORY=PASS")
        for label in ("baseline","candidate"):
            x=results[label]
            prefix=label.upper()
            print(prefix+"_CARGO_FMT=PASS")
            print(prefix+"_CARGO_TEST_ALL_TARGETS=PASS")
            print(prefix+"_TESTS_PASSED="+str(x["cargo_test_result"]["passed"]))
            print(prefix+"_CARGO_CLIPPY=PASS")
            print(prefix+"_SECURITY_CONTRACT=PASS")
        print("DENIED_CAPABILITY_PERFORMANCE_EXCEPTION=OPEN")
        print("INTEGRATION_READINESS="+report["decision"])
        print("MERGE_AUTHORIZATION=NONE")
        print("REPORT_PRIVATE="+str(out) if out else "REPORT_PRIVATE=NONE_CI")
        print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    s=p.add_subparsers(dest="task",required=True)
    s.add_parser("selftest")
    x=s.add_parser("check")
    x.add_argument("--safety-source",required=True)
    x.add_argument("--out")
    args=p.parse_args()
    try:
        if args.task=="selftest":selftest()
        else:check(args.safety_source,args.out)
        return 0
    except (Blocked,OSError,ValueError,KeyError,UnicodeError,TypeError,
            AssertionError,subprocess.TimeoutExpired) as e:
        print("G03_R3_R20_GATE=BLOCKED",file=sys.stderr)
        print("REASON="+str(e)[:650],file=sys.stderr)
        print("MERGE_AUTHORIZATION=NONE",file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
