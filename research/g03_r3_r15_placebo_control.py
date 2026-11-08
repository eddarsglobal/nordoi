#!/usr/bin/env python3
"""G0.3 R3-R15: pinned A/A placebo vs A/B Rust kernel cost control.

A/A executes the *same* baseline binary twice (two timing labels);
A/B executes baseline vs single-lookup candidate. Each arm self-checks.
Synthetic selftest is NOT a real benchmark. No source or production writes.
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
import sys
import tempfile
import time
import types

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CANDIDATE_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
R13_TOOL_BLOB = "dac0109959e10a15764ff54a4da91bf190deff59"
R13_PROBE_SHA256 = "be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a"
SCHEMA = "NORDOI-G03-R3-R15-PLACEBO-v1"
CASES = ("noop_set", "changed_set_flush", "denied_capability")
BRANCH = "candidate/g03-r3-r13-eliminate-redundant-atom-check-20261108"

class Blocked(Exception):
    pass

def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()

def load_r13(path):
    p = Path(path)
    if p.is_symlink() or not p.is_file() or p.stat().st_size > 100000:
        raise Blocked("R13 tool missing or unsafe")
    source = p.read_bytes()
    if git_blob(source) != R13_TOOL_BLOB:
        raise Blocked("R13 tool blob mismatch")
    module = types.ModuleType("nordoi_pinned_r13")
    module.__file__ = str(p)
    exec(compile(source, str(p), "exec"), module.__dict__)
    if (module.BASE_SHA != BASE_SHA or module.BRANCH != BRANCH or
        module.PROBE_SCHEMA != "NORDOI-G03-R3-R13-PROBE-v1"):
        raise Blocked("R13 runner contract drift")
    return module

def summarize(rows, iterations):
    if not rows or not isinstance(iterations,int) or iterations < 1:
        raise Blocked("empty/invalid samples")
    a = []
    b = []
    orders = []
    for row in rows:
        if (set(row) != {"first_right","left_ns","right_ns"} or
            type(row["first_right"]) is not bool or
            type(row["left_ns"]) is not int or type(row["right_ns"]) is not int or
            row["left_ns"] <= 0 or row["right_ns"] <= 0):
            raise Blocked("invalid row")
        a.append(row["left_ns"] / iterations)
        b.append(row["right_ns"] / iterations)
        orders.append(row["first_right"])
    if 2 * sum(orders) != len(rows):
        raise Blocked("order imbalance")
    delta = [x-y for x,y in zip(a,b)]
    med_base = statistics.median(a)
    med_diff = statistics.median(delta)
    return {
        "baseline_median_ns_per_op": round(med_base, 4),
        "right_median_ns_per_op": round(statistics.median(b), 4),
        "paired_delta_ns_per_op": round(med_diff, 4),
        "paired_percent": round(100 * med_diff / med_base, 4),
        "positive_blocks": sum(d > 0 for d in delta),
        "negative_blocks": sum(d < 0 for d in delta),
        "raw_delta_ns_per_op": [round(d,4) for d in delta],
        "min_delta_ns_per_op": round(min(delta),4),
        "max_delta_ns_per_op": round(max(delta),4),
    }

def run_pair(runner, left, right, case, iterations, first_right):
    order = ((right, "right"), (left, "left")) if first_right else (
        (left, "left"), (right, "right"))
    times = {}
    for exe, label in order:
        times[label] = runner.one(exe, case, iterations)
    return {"first_right": first_right, "left_ns": times["left"],
            "right_ns": times["right"]}

def selftest():
    fake = [{"first_right": bool(i % 2),
             "left_ns": 10000, "right_ns": 10000} for i in range(12)]
    z = summarize(fake,100)
    assert z["paired_delta_ns_per_op"] == 0.0 and z["positive_blocks"] == 0
    faster = [dict(row,right_ns=8500) for row in fake]
    assert summarize(faster,100)["paired_percent"] == 15.0
    bad = [dict(x) for x in fake]
    bad[0]["first_right"] = not bad[0]["first_right"]
    try:
        summarize(bad,100)
    except Blocked:
        pass
    else:
        raise AssertionError("imbalance accepted")
    bad = [dict(x) for x in fake]
    bad[0]["right_ns"] = -1
    try:
        summarize(bad,100)
    except Blocked:
        pass
    else:
        raise AssertionError("negative duration accepted")
    print("G03_R3_R15_SELFTEST=SYNTHETIC_PASS")
    print("REAL_MAC_MEASUREMENTS=NOT_RUN_IN_SELFTEST")
    print("HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED")
    print("PRODUCTION=NOT_AUTHORIZED")

def measure(r13_tool, output, iterations, blocks):
    if platform.system() != "Darwin" or platform.machine() != "x86_64":
        raise Blocked("real benchmark scoped to Mac Intel")
    if not 5000 <= iterations <= 150000 or not 8 <= blocks <= 24 or blocks % 2:
        raise Blocked("invalid sampling bounds")
    if any(shutil.which(x) is None for x in ("git","cargo","rustc")):
        raise Blocked("missing Rust/git toolchain")
    runner = load_r13(r13_tool)
    with tempfile.TemporaryDirectory(prefix="NORDOI_R315_ISOLATED_") as folder:
        root = Path(folder)
        base, cand = root/"base", root/"candidate"
        runner.clone(base,"main",BASE_SHA)
        runner.clone(cand,BRANCH,CANDIDATE_SHA)
        guard = runner.verify_delta(base/"src/kernel.rs",cand/"src/kernel.rs")
        allowed = {"src/kernel.rs","research/g03_r3_r13_kernel_probe.rs",
                   "research/g03_r3_r13_paired_mac.py",
                   "research/G03_R3_R13_ENGINEERING.md",
                   "tests/g03_r3_r13_redundant_lookup.rs",
                   ".github/workflows/g03_r3_r13_paired.yml"}
        original = set(runner.run(["git","ls-files"],cwd=base).decode().splitlines())
        revised = set(runner.run(["git","ls-files"],cwd=cand).decode().splitlines())
        if not (original ^ revised).issubset(allowed):
            raise Blocked("candidate tracked file inventory drift")
        for file in original & revised - {"src/kernel.rs"}:
            if (base/file).read_bytes() != (cand/file).read_bytes():
                raise Blocked("unrelated tracked file changed")
        probe = cand/"research/g03_r3_r13_kernel_probe.rs"
        if hashlib.sha256(probe.read_bytes()).hexdigest() != R13_PROBE_SHA256:
            raise Blocked("probe hash drift")
        a, build_a, hash_a = runner.compile_probe(base, root/"a", probe)
        b, build_b, hash_b = runner.compile_probe(cand, root/"b", probe)
        for case in CASES:
            runner.one(a,case,1000)
            runner.one(b,case,1000)
        rng = secrets.SystemRandom()
        schedules = {}
        for case in CASES:
            aa = [False]*(blocks//2)+[True]*(blocks//2)
            ab = [False]*(blocks//2)+[True]*(blocks//2)
            mix = [False]*(blocks//2)+[True]*(blocks//2)
            rng.shuffle(aa); rng.shuffle(ab); rng.shuffle(mix)
            schedules[case] = (aa,ab,mix)
        raw = {case:{"AA":[],"AB":[]} for case in CASES}
        for block in range(blocks):
            for offset in range(len(CASES)):
                case = CASES[(block+offset)%len(CASES)]
                aa_schedule,ab_schedule,mix = schedules[case]
                jobs = (("AB",a,b,ab_schedule[block]),
                        ("AA",a,a,aa_schedule[block]))
                if not mix[block]:
                    jobs = jobs[::-1]
                for label,l,r,order in jobs:
                    row = run_pair(runner,l,r,case,iterations,order)
                    row["block"] = block+1
                    raw[case][label].append(row)
        totals = {}
        for case in CASES:
            totals[case] = {}
            for label in ("AA","AB"):
                rows = raw[case][label]
                if [x["block"] for x in rows] != list(range(1,blocks+1)):
                    raise Blocked("lost blocks")
                totals[case][label] = summarize(
                    [{k:v for k,v in row.items() if k != "block"} for row in rows],
                    iterations)
        report = {
            "schema":SCHEMA,"source_sha":BASE_SHA,"candidate_sha":CANDIDATE_SHA,
            "r13_tool_git_blob":R13_TOOL_BLOB,"probe_sha256":R13_PROBE_SHA256,
            "source_guard":guard,"host":{"system":platform.system(),
                 "machine":platform.machine(),"python":platform.python_version(),
                 "cpu_count":os.cpu_count()},
            "build":{"offline":True,"release":True,"baseline_s":build_a,
                     "candidate_s":build_b},
            "binary_sha256":{"baseline":hash_a,"candidate":hash_b},
            "iterations":iterations,"blocks":blocks,"cases":totals,
            "raw":raw,"AA_control":"EXACT_SAME_BASELINE_BINARY_EXECUTED_TWICE",
            "AB_comparison":"BASELINE_VS_R13_CANDIDATE",
            "balanced_order":True,"merge_authorization":"NONE",
            "human_evaluation_gate":"CLOSED_NOT_REQUIRED",
            "production":"NOT_AUTHORIZED","original_repo_modified":False,
            "measurement_limit":"single Mac Intel host; microbenchmark jitter",
            "run_start_or_end":"not a proof of independent CPU conditions",
        }
        runner.private_receipt(output,report)
        print("G03_R3_R15_AA_AB_MAC=PASS")
        print("SOURCE_GUARD=PASS")
        print("ORDER_BALANCE=HALF_EACH_PER_CASE_AND_CONTROL")
        for case in CASES:
            a0,b0=totals[case]["AA"],totals[case]["AB"]
            print(case.upper()+"_AA_PAIRED_PERCENT="+str(a0["paired_percent"]))
            print(case.upper()+"_AB_PAIRED_PERCENT="+str(b0["paired_percent"]))
            print(case.upper()+"_AB_FASTER_BLOCKS="+str(b0["positive_blocks"])+"/"+str(blocks))
        print("ENGINEERING_DECISION=REVIEW_ONLY")
        print("MERGE_AUTHORIZATION=NONE")
        print("REPORT_PRIVATE="+str(output))
        print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="task",required=True)
    sub.add_parser("selftest")
    m=sub.add_parser("measure")
    m.add_argument("--r13-tool",required=True)
    m.add_argument("--out",required=True)
    m.add_argument("--iterations",type=int,default=30000)
    m.add_argument("--blocks",type=int,default=12)
    args=p.parse_args()
    try:
        if args.task == "selftest":
            selftest()
        else:
            measure(args.r13_tool,args.out,args.iterations,args.blocks)
        return 0
    except (Blocked,OSError,KeyError,ValueError,TypeError,UnicodeError,AssertionError) as e:
        print("G03_R3_R15_GATE=BLOCKED",file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__ == "__main__":
    sys.exit(main())
