#!/usr/bin/env python3
"""R3-R16 engineering-only: separately built A/A, B/B, A/B and safety probe.

All real timings come from user-triggered Mac Intel only; this script never
merges a PR or changes original sources. Synthetic selftest is not evidence
of actual speed. A/A and B/B compare *independently built* executable files.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import secrets
import shutil
import statistics
import subprocess
import sys
import tempfile
import types

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CAND_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
R13_BLOB = "dac0109959e10a15764ff54a4da91bf190deff59"
R13_PROBE_SHA = "be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a"
SECURITY_BLOB = "4080350aab5c3b2f377f7c22bca54d49c50e6df7"
BASE_KERNEL_SHA256 = "dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2"
BRANCH = "candidate/g03-r3-r13-eliminate-redundant-atom-check-20261108"
SCHEMA = "NORDOI-G03-R3-R16-INDEPENDENT-CONTROLS-v1"
CASES = ("noop_set", "changed_set_flush", "denied_capability")
PAIRS = ("AA", "BB", "AB")

class Blocked(Exception):
    pass

def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()

def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def pinned_file(path, blob, limit=100000):
    p = Path(path)
    if p.is_symlink() or not p.is_file() or p.stat().st_size <= 0 or p.stat().st_size > limit:
        raise Blocked("unsafe or missing pinned file")
    data = p.read_bytes()
    if git_blob(data) != blob:
        raise Blocked("Git blob identity mismatch")
    return data

def load_r13(path):
    data = pinned_file(path,R13_BLOB)
    module = types.ModuleType("verified_nordoi_r313")
    module.__file__ = str(path)
    exec(compile(data,str(path),"exec"),module.__dict__)
    if (module.BASE_SHA != BASE_SHA or module.BRANCH != BRANCH
            or module.PROBE_SCHEMA != "NORDOI-G03-R3-R13-PROBE-v1"):
        raise Blocked("R13 runner changed")
    return module

def tracked_identity(runner,a,b):
    fa = set(runner.run(["git","ls-files"],cwd=a).decode().splitlines())
    fb = set(runner.run(["git","ls-files"],cwd=b).decode().splitlines())
    if fa != fb or any((a/f).read_bytes() != (b/f).read_bytes() for f in fa):
        raise Blocked("same-revision tracked sources differ")
    return len(fa)

def summarize(rows, iterations):
    if not rows or iterations < 1 or len(rows) % 2 != 0:
        raise Blocked("missing rows or invalid bound")
    first = [r["right_first"] for r in rows]
    if any(type(x) is not bool for x in first) or sum(first) * 2 != len(rows):
        raise Blocked("unbalanced arm ordering")
    for i,row in enumerate(rows,1):
        if (set(row) != {"block","right_first","left_ns","right_ns"}
                or type(row["block"]) is not int or row["block"] != i
                or type(row["left_ns"]) is not int or type(row["right_ns"]) is not int
                or row["left_ns"] <= 0 or row["right_ns"] <= 0):
            raise Blocked("malformed raw block")
    a = [r["left_ns"]/iterations for r in rows]
    b = [r["right_ns"]/iterations for r in rows]
    d = [x-y for x,y in zip(a,b)]
    baseline = statistics.median(a)
    return {
        "left_median_ns_op":round(baseline,4),
        "right_median_ns_op":round(statistics.median(b),4),
        "paired_delta_ns_op":round(statistics.median(d),4),
        "paired_percent":round(statistics.median(d)*100/baseline,4),
        "positive_blocks":sum(x>0 for x in d),
        "negative_blocks":sum(x<0 for x in d),
        "median_abs_delta_ns_op":round(statistics.median(abs(x) for x in d),4),
        "max_abs_delta_ns_op":round(max(abs(x) for x in d),4),
    }

def pair_once(runner,left,right,case,iterations,right_first):
    order = (("right",right),("left",left)) if right_first else (
        ("left",left),("right",right))
    measured = {}
    for label,exe in order:
        measured[label+"_ns"] = runner.one(exe,case,iterations)
    return {"right_first":right_first,**measured}

def security_probe(runner,root,security_rs):
    deps = root/"target"/"release"/"deps"
    libs = list(deps.glob("libnordoi_kernel-*.rlib"))
    if len(libs) != 1:
        raise Blocked("release library ambiguous for safety probe")
    exe = root/("safety.exe" if os.name=="nt" else "safety")
    runner.run(["rustc","--edition=2021","-C","opt-level=3",
        "-C","codegen-units=1","-L","dependency="+str(deps),
        "--extern","nordoi_kernel="+str(libs[0]),str(security_rs),"-o",str(exe)])
    receipt = runner.run([str(exe)],timeout=45)
    if receipt != b"G03_R3_R16_SECURITY_PROBE=PASS\nPRODUCTION=NOT_AUTHORIZED\n":
        raise Blocked("safety contract receipt not exact")
    return {"sha256":sha256(exe),"bytes":exe.stat().st_size,
            "contract":"PASS"}

def code_section_fingerprint(exe):
    # A fingerprint of Mach-O __TEXT,__text bytes as dumped by otool.
    # Whole-section equality/difference does NOT prove require() equality.
    otool=shutil.which("otool")
    if not otool:
        return {"status":"OTOOL_UNAVAILABLE","interpretation":"NOT_ESTABLISHED"}
    p=subprocess.run([otool,"-t",str(exe)],stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,timeout=120)
    if p.returncode or not p.stdout or len(p.stdout)>25_000_000:
        raise Blocked("Mach-O text-section inspection failed")
    lines=p.stdout.splitlines()
    if len(lines)<3 or b"(__TEXT,__text)" not in p.stdout:
        raise Blocked("unexpected otool output")
    # Ignore the executable path printed as the first line.
    normalized=b"\n".join(lines[1:])
    return {"status":"CODE_SECTION_FINGERPRINT_ONLY",
        "otool_text_sha256":hashlib.sha256(normalized).hexdigest(),
        "dump_bytes":len(p.stdout),
        "interpretation":"NOT_FUNCTION_LEVEL_EQUIVALENCE"}

def file_identity(exe):
    p=Path(exe)
    return {"sha256":sha256(p),"bytes":p.stat().st_size}

def selftest():
    rows = [{"block":i+1,"right_first":bool(i%2),
        "left_ns":10000,"right_ns":8500} for i in range(12)]
    assert summarize(rows,100)["paired_percent"]==15.0
    assert summarize([dict(r,right_ns=10000) for r in rows],100)["paired_percent"]==0
    bad = [dict(r) for r in rows]
    bad[0]["right_first"] = False if bad[0]["right_first"] else True
    try:
        summarize(bad,100)
    except Blocked:
        pass
    else:
        raise AssertionError("unbalanced schedule accepted")
    bad = [dict(r) for r in rows]
    bad[0]["right_ns"]=-1
    try:
        summarize(bad,100)
    except Blocked:
        pass
    else:
        raise AssertionError("negative duration accepted")
    assert git_blob(b"123") == "d800886d9c86731ae5c4a62b0b77c437015e00d2"
    print("G03_R3_R16_SELFTEST=SYNTHETIC_PASS")
    print("REAL_MAC_BENCHMARK=NOT_RUN_IN_SELFTEST")
    print("PRODUCTION=NOT_AUTHORIZED")

def smoke(source,security_rs):
    # Cross-platform CI: correctness test only, no benchmark or Mac performance claim.
    source=Path(source).resolve()
    safety=Path(security_rs).resolve()
    pinned_file(safety,SECURITY_BLOB,12000)
    # Verify canonical committed bytes, not platform-translated checkout CRLF.
    git_kernel=subprocess.run(["git","show","HEAD:src/kernel.rs"],
        cwd=str(source),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60)
    if git_kernel.returncode or hashlib.sha256(git_kernel.stdout).hexdigest()!=BASE_KERNEL_SHA256:
        raise Blocked("CI committed source kernel differs from frozen main")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R316_CI_") as folder:
        root=Path(folder)
        env=dict(os.environ,CARGO_NET_OFFLINE="true",
            CARGO_TARGET_DIR=str(root/"target"))
        runner=subprocess.run(["cargo","build","--offline","--release","--lib"],
            cwd=str(source),env=env,stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,timeout=900)
        if runner.returncode!=0:
            raise Blocked("offline Rust library build failed")
        dummy=types.SimpleNamespace(run=lambda argv,timeout=900:
            shell_checked(argv,timeout=timeout))
        security_probe(dummy,root,safety)
    print("G03_R3_R16_CI_SECURITY_SMOKE=PASS")
    print("REAL_MAC_BENCHMARK=NOT_RUN_IN_CI")
    print("PRODUCTION=NOT_AUTHORIZED")

def shell_checked(argv,timeout=900):
    p=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
    if p.returncode:
        raise Blocked("smoke subprocess failed")
    return p.stdout

def measure(r13_tool,security_rs,output,iterations,blocks):
    if platform.system()!="Darwin" or platform.machine()!="x86_64":
        raise Blocked("real benchmark requires Mac Intel")
    if not 5000<=iterations<=150000 or not 8<=blocks<=24 or blocks%2:
        raise Blocked("out of bounds benchmark")
    if any(shutil.which(x) is None for x in ("git","cargo","rustc")):
        raise Blocked("git/Rust missing")
    runner=load_r13(r13_tool)
    pinned_file(security_rs,SECURITY_BLOB,12000)
    out=Path(output)
    if out.exists() or out.is_symlink() or not out.parent.is_dir():
        raise Blocked("unsafe output path")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R316_REBUILDS_") as tmp:
        root=Path(tmp)
        sources={"base_a":root/"base_a","base_b":root/"base_b",
                 "cand_a":root/"cand_a","cand_b":root/"cand_b"}
        for name,path in sources.items():
            runner.clone(path,"main" if name.startswith("base") else BRANCH,
                BASE_SHA if name.startswith("base") else CAND_SHA)
        tracked_identity(runner,sources["base_a"],sources["base_b"])
        tracked_identity(runner,sources["cand_a"],sources["cand_b"])
        source_guard=runner.verify_delta(
            sources["base_a"]/"src/kernel.rs",
            sources["cand_a"]/"src/kernel.rs")
        allowed={"src/kernel.rs","research/g03_r3_r13_kernel_probe.rs",
            "research/g03_r3_r13_paired_mac.py",
            "research/G03_R3_R13_ENGINEERING.md",
            "tests/g03_r3_r13_redundant_lookup.rs",
            ".github/workflows/g03_r3_r13_paired.yml"}
        old=set(runner.run(["git","ls-files"],cwd=sources["base_a"]).decode().splitlines())
        new=set(runner.run(["git","ls-files"],cwd=sources["cand_a"]).decode().splitlines())
        if not (old ^ new).issubset(allowed):
            raise Blocked("unexpected source inventory")
        for rel in old & new - {"src/kernel.rs"}:
            if (sources["base_a"]/rel).read_bytes() != (
                    sources["cand_a"]/rel).read_bytes():
                raise Blocked("unrelated source change")
        probe=sources["cand_a"]/"research/g03_r3_r13_kernel_probe.rs"
        if sha256(probe)!=R13_PROBE_SHA:
            raise Blocked("R13 Rust probe identity changed")
        binaries={}
        artifacts={}
        for name,source in sources.items():
            build_root=root/("build_"+name)
            exe,build_seconds,binary_hash=runner.compile_probe(
                source,build_root,probe)
            contract=security_probe(runner,build_root,security_rs)
            binaries[name]=exe
            artifacts[name]={"executable":file_identity(exe),
                "text_section":code_section_fingerprint(exe),
                "build_seconds":build_seconds,
                "security_test":contract}
            if artifacts[name]["executable"]["sha256"]!=binary_hash:
                raise Blocked("binary hash inconsistency")
        for case in CASES:
            for binary in binaries.values():
                runner.one(binary,case,1000)
        cases={case:{pair:[] for pair in PAIRS} for case in CASES}
        rng=secrets.SystemRandom()
        pair_map={"AA":("base_a","base_b"),
                  "BB":("cand_a","cand_b"),
                  "AB":("base_a","cand_a")}
        schedules={}
        for case in CASES:
            schedules[case]={}
            for pair in PAIRS:
                order=[False]*(blocks//2)+[True]*(blocks//2)
                rng.shuffle(order)
                schedules[case][pair]=order
        group_orders={}
        for block in range(blocks):
            for offset in range(len(CASES)):
                case=CASES[(block+offset)%len(CASES)]
                order=list(PAIRS)
                # Each pair occupies first/middle/last position equally.
                shift=(block+offset)%len(PAIRS)
                order=order[shift:]+order[:shift]
                group_orders[case+"_"+str(block+1)]=order
                for pair in order:
                    left,right=pair_map[pair]
                    row=pair_once(runner,binaries[left],binaries[right],
                        case,iterations,schedules[case][pair][block])
                    row["block"]=block+1
                    cases[case][pair].append(row)
        summaries={}
        for case in CASES:
            summaries[case]={pair:summarize(cases[case][pair],iterations)
                             for pair in PAIRS}
        binary_pair_hashes={
            "baseline_two_builds_equal":artifacts["base_a"]["executable"]["sha256"]==
                artifacts["base_b"]["executable"]["sha256"],
            "candidate_two_builds_equal":artifacts["cand_a"]["executable"]["sha256"]==
                artifacts["cand_b"]["executable"]["sha256"],
        }
        report={"schema":SCHEMA,"source_sha":BASE_SHA,"candidate_sha":CAND_SHA,
            "r13_tool_blob":R13_BLOB,"r13_probe_sha256":R13_PROBE_SHA,
            "security_probe_git_blob":SECURITY_BLOB,
            "source_guard":source_guard,"builds":artifacts,
            "build_hash_equivalence":binary_pair_hashes,
            "host":{"machine":platform.machine(),"system":platform.system(),
                    "python":platform.python_version(),"cpu_count":os.cpu_count()},
            "compiler":{"rustc":runner.run(["rustc","--version"]).decode().strip(),
                        "cargo":runner.run(["cargo","--version"]).decode().strip()},
            "cases":summaries,"raw":cases,"group_orders":group_orders,
            "iterations":iterations,"blocks":blocks,
            "control_method":"INDEPENDENT_BASELINE_AND_CANDIDATE_RELEASE_BUILDS",
            "workload":"R13_PINNED_EXTERNAL_RUST_API_PROBE",
            "statistical_decision":"ENGINEERING_REVIEW_ONLY",
            "machine_code_identity":"NOT_ESTABLISHED_BY_BINARY_HASH",
            "merge_authorization":"NONE","human_evaluation_gate":"CLOSED_NOT_REQUIRED",
            "production":"NOT_AUTHORIZED","original_repo_modified":False}
        runner.private_receipt(out,report)
    print("G03_R3_R16_MAC_SECURITY=4_OF_4_PASS")
    print("G03_R3_R16_INDEPENDENT_BUILDS=4_PASS")
    print("SOURCE_GUARD=PASS")
    print("ORDER_BALANCE=HALF_FIRST_PER_CASE_AND_COMPARISON")
    print("BASELINE_BINARY_HASH_EQUAL="+str(binary_pair_hashes["baseline_two_builds_equal"]).upper())
    print("CANDIDATE_BINARY_HASH_EQUAL="+str(binary_pair_hashes["candidate_two_builds_equal"]).upper())
    for case in CASES:
        for pair in PAIRS:
            s=summaries[case][pair]
            print(case.upper()+"_"+pair+"_PAIRED_PERCENT="+str(s["paired_percent"]))
            print(case.upper()+"_"+pair+"_PAIRED_DELTA_NS="+str(s["paired_delta_ns_op"]))
            print(case.upper()+"_"+pair+"_FASTER_BLOCKS="+str(s["positive_blocks"])+"/"+str(blocks))
    print("G03_R3_R16_ENGINEERING_DECISION=REVIEW_ONLY")
    print("MERGE_AUTHORIZATION=NONE")
    print("REPORT_PRIVATE="+str(out))
    print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    sub.add_parser("selftest")
    t=sub.add_parser("smoke")
    t.add_argument("--source",required=True)
    t.add_argument("--security-rust",required=True)
    m=sub.add_parser("measure")
    m.add_argument("--r13-tool",required=True)
    m.add_argument("--security-rust",required=True)
    m.add_argument("--out",required=True)
    m.add_argument("--iterations",type=int,default=30000)
    m.add_argument("--blocks",type=int,default=12)
    args=p.parse_args()
    try:
        if args.action=="selftest": selftest()
        elif args.action=="smoke":smoke(args.source,args.security_rust)
        else:measure(args.r13_tool,args.security_rust,args.out,
                     args.iterations,args.blocks)
        return 0
    except (Blocked,OSError,ValueError,TypeError,KeyError,UnicodeError,
            AssertionError,subprocess.TimeoutExpired) as ex:
        print("G03_R3_R16_GATE=BLOCKED",file=sys.stderr)
        print("REASON="+str(ex)[:160],file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
