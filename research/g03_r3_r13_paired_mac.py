#!/usr/bin/env python3
"""R3-R13: pinned, paired in-process Rust baseline vs candidate, no engine claims."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
BRANCH = "candidate/g03-r3-r13-eliminate-redundant-atom-check-20261108"
REPO = "https://github.com/eddarsglobal/nordoi.git"
SCHEMA = "NORDOI-G03-R3-R13-PAIRED-ENGINE-v1"
PROBE_SCHEMA = "NORDOI-G03-R3-R13-PROBE-v1"
CASES = ("noop_set", "changed_set_flush", "dependency_set_flush",
         "noop_transaction", "denied_capability", "nair_validate",
         "nair_execute_with_validation")
OLD = (b"    fn apply_single(&mut self, id: AtomId, next: Value) -> AtomicResult<bool> {\n"
       b"        self.ensure_exists(id)?;\n\n"
       b"        // Preserve the No Work Without Effect law: an identical write exits")
NEW = (b"    fn apply_single(&mut self, id: AtomId, next: Value) -> AtomicResult<bool> {\n"
       b"        // self.get(id)? below already rejects UnknownAtom. A second\n"
       b"        // ensure_exists HashMap probe here is redundant on every write.\n"
       b"        // Ownership is still checked in set_owned before this function.\n"
       b"        // Preserve the No Work Without Effect law: an identical write exits")

class Blocked(Exception):
    pass

def run(argv, cwd=None, env=None, timeout=900):
    try:
        v = subprocess.run(argv, cwd=cwd, env=env, timeout=timeout,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except (OSError, subprocess.TimeoutExpired) as e:
        raise Blocked("unavailable or timed-out subprocess") from e
    if v.returncode:
        raise Blocked("failed verification/build/subprocess")
    return v.stdout

def hash256(content):
    return hashlib.sha256(content).hexdigest()

def verify_delta(a, b):
    old = Path(a).read_bytes()
    new = Path(b).read_bytes()
    if old.count(OLD) != 1 or new != old.replace(OLD, NEW):
        raise Blocked("unexpected candidate kernel source delta")
    return {"baseline_sha256": hash256(old), "candidate_sha256": hash256(new),
            "source_delta": "EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY"}

def clone(destination, ref, expected):
    run(["git", "clone", "--quiet", "--depth", "1", "--branch", ref, REPO, str(destination)],
        timeout=180)
    observed = run(["git", "rev-parse", "HEAD"], cwd=destination, timeout=20).decode().strip()
    if observed != expected:
        raise Blocked("source head mismatch")

def compile_probe(source, root, probe):
    target = root / "target"
    env = dict(os.environ, CARGO_NET_OFFLINE="true", CARGO_TARGET_DIR=str(target))
    started = time.perf_counter_ns()
    run(["cargo", "build", "--offline", "--release", "--lib"], cwd=source, env=env)
    build_s = round((time.perf_counter_ns()-started) / 1e9, 4)
    libs = list((target / "release" / "deps").glob("libnordoi_kernel-*.rlib"))
    if len(libs) != 1:
        raise Blocked("ambiguous release library")
    exe = root / ("probe.exe" if os.name == "nt" else "probe")
    run(["rustc", "--edition=2021", "-C", "opt-level=3", "-C", "codegen-units=1",
         "-L", "dependency="+str(target / "release" / "deps"),
         "--extern", "nordoi_kernel="+str(libs[0]), str(probe), "-o", str(exe)])
    if b"G03_R3_R13_RUST_SELFTEST=PASS" not in run([str(exe), "selftest"], timeout=45):
        raise Blocked("probe functional selftest failed")
    return exe, build_s, hash256(exe.read_bytes())

def one(exe, case, iterations):
    raw = run([str(exe), "one", case, str(iterations)], timeout=120)
    if len(raw) > 8192:
        raise Blocked("oversized probe output")
    try:
        d = json.loads(raw)
    except (ValueError, UnicodeError) as e:
        raise Blocked("invalid probe output") from e
    if (not isinstance(d, dict) or d.get("schema") != PROBE_SCHEMA or
        d.get("case") != case or d.get("iterations") != iterations or
        d.get("invariants") != "PASS" or type(d.get("total_ns")) is not int or
        d["total_ns"] <= 0):
        raise Blocked("probe invariant/identity violation")
    return d["total_ns"]

def summary(rows, iterations):
    a = [r["baseline_ns"]/iterations for r in rows]
    b = [r["candidate_ns"]/iterations for r in rows]
    delta = [x-y for x,y in zip(a,b)]
    return {"baseline_median_ns_per_op": round(statistics.median(a),3),
            "candidate_median_ns_per_op": round(statistics.median(b),3),
            "paired_delta_median_ns_per_op": round(statistics.median(delta),3),
            "candidate_faster_blocks": sum(x>0 for x in delta),
            "candidate_slower_blocks": sum(x<0 for x in delta),
            "raw_blocks": rows}

def private_receipt(path, report):
    p=Path(path)
    if p.exists() or p.is_symlink() or not p.parent.is_dir():
        raise Blocked("report path unsafe or exists")
    flags=os.O_EXCL|os.O_CREAT|os.O_WRONLY
    if hasattr(os,"O_NOFOLLOW"): flags |= os.O_NOFOLLOW
    fd=os.open(str(p),flags,0o600)
    with os.fdopen(fd,"w",encoding="utf-8",newline="\n") as f:
        json.dump(report,f,sort_keys=True,indent=2)
        f.write("\n")
    if os.name=="posix" and (p.stat().st_mode & 0o077):
        raise Blocked("private file permissions too broad")

def selftest():
    orig=b"header\n"+OLD+b"\nfooter\n"
    with tempfile.TemporaryDirectory(prefix="NORDOI_R313_TEST_") as tmp:
        a,b=Path(tmp)/"base",Path(tmp)/"candidate"
        a.write_bytes(orig)
        b.write_bytes(orig.replace(OLD,NEW))
        assert verify_delta(a,b)["source_delta"]=="EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY"
        b.write_bytes(orig)
        try: verify_delta(a,b)
        except Blocked: pass
        else: raise AssertionError("invalid source delta accepted")
        report=Path(tmp)/"report.json"
        private_receipt(report,{"synthetic":True})
        try: private_receipt(report,{})
        except Blocked: pass
        else: raise AssertionError("overwritten report")
    assert summary([{"baseline_ns":600,"candidate_ns":500},
                    {"baseline_ns":400,"candidate_ns":500},
                    {"baseline_ns":700,"candidate_ns":400}],10)["candidate_faster_blocks"]==2
    print("G03_R3_R13_PYTHON_SELFTEST=SYNTHETIC_PASS")
    print("REAL_PERFORMANCE=NOT_MEASURED_IN_SELFTEST")
    print("HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED")
    print("PRODUCTION=NOT_AUTHORIZED")

def smoke(source):
    source=Path(source).resolve()
    probe=source/"research"/"g03_r3_r13_kernel_probe.rs"
    if not probe.is_file(): raise Blocked("candidate probe missing")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R313_CI_") as tmp:
        root=Path(tmp)
        env=dict(os.environ,CARGO_TARGET_DIR=str(root/"target"),CARGO_NET_OFFLINE="true")
        run(["cargo","test","--offline","--release","--test","g03_r3_r13_redundant_lookup"],
            cwd=source,env=env)
        exe,_,_=compile_probe(source,root,probe)
        for case in CASES: one(exe,case,200)
    print("G03_R3_R13_SMOKE=PASS")
    print("REAL_BENCHMARK=NOT_RUN_IN_CI")

def measure(sha, out, iterations, blocks):
    if (len(sha)!=40 or any(x not in "0123456789abcdef" for x in sha) or
        not 5000<=iterations<=200000 or not 5<=blocks<=15):
        raise Blocked("invalid candidate pin or sample bounds")
    if any(shutil.which(exe) is None for exe in ("git","cargo","rustc")):
        raise Blocked("required toolchain missing")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R313_PAIRED_") as tmp:
        root=Path(tmp)
        base,cand=root/"base",root/"candidate"
        clone(base,"main",BASE_SHA)
        clone(cand,BRANCH,sha)
        delta=verify_delta(base/"src"/"kernel.rs",cand/"src"/"kernel.rs")
        allowed={"src/kernel.rs","research/g03_r3_r13_kernel_probe.rs",
                 "research/g03_r3_r13_paired_mac.py",
                 "research/G03_R3_R13_ENGINEERING.md",
                 "tests/g03_r3_r13_redundant_lookup.rs",
                 ".github/workflows/g03_r3_r13_paired.yml"}
        orig=set(run(["git","ls-files"],cwd=base).decode().splitlines())
        revised=set(run(["git","ls-files"],cwd=cand).decode().splitlines())
        if not (orig^revised).issubset(allowed):
            raise Blocked("unexpected tracked-file inventory")
        for file in orig&revised-{"src/kernel.rs"}:
            if (base/file).read_bytes() != (cand/file).read_bytes():
                raise Blocked("unrelated tracked source changed")
        probe=cand/"research"/"g03_r3_r13_kernel_probe.rs"
        if not probe.is_file(): raise Blocked("missing fixed probe")
        exe_a, secs_a, hash_a=compile_probe(base,root/"a",probe)
        exe_b, secs_b, hash_b=compile_probe(cand,root/"b",probe)
        samples={name:[] for name in CASES}
        for case in CASES:
            one(exe_a,case,1000)
            one(exe_b,case,1000)
        for block in range(blocks):
            for offset in range(len(CASES)):
                case=CASES[(block+offset)%len(CASES)]
                reverse=(block+offset)%2 != 0
                order=(("candidate",exe_b),("baseline",exe_a)) if reverse else (
                    ("baseline",exe_a),("candidate",exe_b))
                row={"block":block+1,"candidate_first":reverse}
                for arm,exe in order: row[arm+"_ns"]=one(exe,case,iterations)
                samples[case].append(row)
        result={"schema":SCHEMA,"source_sha":BASE_SHA,"candidate_sha":sha,
                "source_guard":delta,"probe_sha256":hash256(probe.read_bytes()),
                "host":{"system":platform.system(),"machine":platform.machine(),
                        "cpu_count":os.cpu_count(),"python":platform.python_version()},
                "build":{"offline":True,"release":True,"baseline_seconds":secs_a,
                         "candidate_seconds":secs_b},
                "executables":{"baseline_sha256":hash_a,"candidate_sha256":hash_b},
                "samples_per_case":blocks,"iterations_per_sample":iterations,
                "measurements":{name:summary(samples[name],iterations) for name in CASES},
                "nair_execute_includes_validation":True,
                "whole_process_cli_excluded":True,
                "cross_language_superiority":"NOT_ESTABLISHED",
                "performance_gain":"REQUIRES_REVIEW",
                "human_evaluation_gate":"CLOSED_NOT_REQUIRED",
                "production":"NOT_AUTHORIZED","original_repo_modified":False}
        private_receipt(out,result)
        print("G03_R3_R13_PAIRED_MAC=PASS")
        print("BASELINE_SHA="+BASE_SHA)
        print("CANDIDATE_SHA="+sha)
        for name in CASES:
            d=result["measurements"][name]
            print(name.upper()+"_BASELINE_MEDIAN_NS="+str(d["baseline_median_ns_per_op"]))
            print(name.upper()+"_CANDIDATE_MEDIAN_NS="+str(d["candidate_median_ns_per_op"]))
            print(name.upper()+"_PAIRED_DELTA_MEDIAN_NS="+str(d["paired_delta_median_ns_per_op"]))
        print("PERFORMANCE_GAIN=REQUIRES_ENGINEERING_REVIEW")
        print("REPORT_PRIVATE="+str(out))
        print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    sub.add_parser("selftest")
    s=sub.add_parser("smoke");s.add_argument("--source",required=True)
    m=sub.add_parser("measure");m.add_argument("--candidate-sha",required=True)
    m.add_argument("--out",required=True)
    m.add_argument("--iterations",type=int,default=30000)
    m.add_argument("--blocks",type=int,default=7)
    args=p.parse_args()
    try:
        if args.action=="selftest": selftest()
        elif args.action=="smoke": smoke(args.source)
        else: measure(args.candidate_sha,args.out,args.iterations,args.blocks)
        return 0
    except (Blocked, OSError, ValueError, KeyError, TypeError, UnicodeError, AssertionError):
        print("G03_R3_R13_GATE=BLOCKED",file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
