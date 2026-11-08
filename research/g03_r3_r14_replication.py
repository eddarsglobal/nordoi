#!/usr/bin/env python3
"""NORDOI G0.3 R3-R14: three separate owner-triggered, order-balanced Mac replications.

Research engineering only. No compiler mutation, no external human gate, no auto-merge.
Loads R3-R13 runner ONLY after pinning its exact Git blob; every real run reclones both
verified source trees and repeats the in-process R3-R13 probes using AB/BA balance.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import secrets
import shutil
import stat
import statistics
import sys
import tempfile
import time
import types

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CANDIDATE_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
R13_BLOB = "dac0109959e10a15764ff54a4da91bf190deff59"
R13_PROBE_SHA256 = "be6bbd0b74ca8ff12035ee2e8529961285767251f26074d2ce1b4663d723204a"
R13_SCHEMA = "NORDOI-G03-R3-R13-PAIRED-ENGINE-v1"
SCHEMA = "NORDOI-G03-R3-R14-REPLICATION-v1"
PROTOCOL_SCHEMA = "NORDOI-G03-R3-R14-PROTOCOL-v1"
ANALYSIS_SCHEMA = "NORDOI-G03-R3-R14-ANALYSIS-v1"
SLOTS = ("S1", "S2", "S3")
CASES = ("noop_set", "changed_set_flush", "dependency_set_flush",
         "noop_transaction", "denied_capability", "nair_validate",
         "nair_execute_with_validation")
BLOCKS = 8
ITERATIONS = 30000
MAX_FILE = 300_000


class Blocked(Exception):
    pass


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()


def file_read(path, *, private=False, limit=MAX_FILE):
    p = Path(path)
    try:
        st = p.lstat()
    except OSError as e:
        raise Blocked("required file missing") from e
    if not stat.S_ISREG(st.st_mode) or st.st_nlink != 1 or st.st_size > limit:
        raise Blocked("unsafe file kind or size")
    if private and os.name == "posix" and st.st_mode & 0o077:
        raise Blocked("private permissions too broad")
    try:
        data = p.read_bytes()
    except OSError as e:
        raise Blocked("file unreadable") from e
    if len(data) > limit:
        raise Blocked("oversize input")
    return data


def read_json(path, private=True):
    try:
        val = json.loads(file_read(path, private=private).decode("utf-8"))
    except (UnicodeError, ValueError) as e:
        raise Blocked("malformed JSON") from e
    if not isinstance(val, dict):
        raise Blocked("wrong JSON kind")
    return val


def pinned_r13(path):
    data = file_read(path, limit=100_000)
    if git_blob(data) != R13_BLOB:
        raise Blocked("R3-R13 tool Git blob mismatch")
    mod = types.ModuleType("pinned_nordoi_r313")
    mod.__file__ = str(path)
    # Execute only the exact Git-pinned, reviewed R3-R13 Python script.
    exec(compile(data, str(path), "exec"), mod.__dict__)
    if (mod.BASE_SHA != BASE_SHA or mod.PROBE_SCHEMA != "NORDOI-G03-R3-R13-PROBE-v1"
            or mod.SCHEMA != R13_SCHEMA or tuple(mod.CASES) != CASES):
        raise Blocked("pinned runner API contract drift")
    return mod


def study_dir(path):
    p = Path(path)
    try:
        st = p.lstat()
    except OSError as e:
        raise Blocked("study folder unavailable") from e
    if not stat.S_ISDIR(st.st_mode):
        raise Blocked("unsafe study folder")
    if os.name == "posix" and st.st_mode & 0o077:
        raise Blocked("study folder permissions too broad")
    return p


def private_write(path, payload):
    p = Path(path)
    if p.exists() or p.is_symlink() or not p.parent.is_dir():
        raise Blocked("output path exists or unsafe")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        fd = os.open(str(p), flags, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as f:
            json.dump(payload, f, sort_keys=True, indent=2)
            f.write("\n")
    except OSError as e:
        raise Blocked("cannot create exclusive private file") from e
    if os.name == "posix" and p.stat().st_mode & 0o077:
        raise Blocked("bad output permissions")


def protocol():
    return {"schema": PROTOCOL_SCHEMA, "source_sha": BASE_SHA,
            "candidate_sha": CANDIDATE_SHA, "r13_git_blob": R13_BLOB,
            "r13_probe_sha256": R13_PROBE_SHA256,
            "sessions": list(SLOTS), "blocks": BLOCKS, "iterations": ITERATIONS,
            "orders_per_case_per_session": {"baseline_first": 4, "candidate_first": 4},
            "independence_proven_by_software": False,
            "historical_r13_pilot": "EXCLUDED_FROM_REPLICATION_DECISION",
            "automatic_merge": False, "human_evaluation_gate": "CLOSED_NOT_REQUIRED",
            "production": "NOT_AUTHORIZED"}


def validate_protocol(p):
    if read_json(p / "protocol.json") != protocol():
        raise Blocked("preregistered protocol changed")


def initialize(path):
    root = Path(path)
    if root.exists() or root.is_symlink() or not root.parent.is_dir():
        raise Blocked("study folder already exists or parent unavailable")
    try:
        root.mkdir(mode=0o700, parents=False, exist_ok=False)
    except OSError as e:
        raise Blocked("private study folder creation refused") from e
    if os.name == "posix":
        root.chmod(0o700)
    private_write(root / "protocol.json", protocol())
    print("G03_R3_R14_PRIVATE_PROTOCOL=PREPARED")
    print("SESSIONS_REQUIRED=3_SEPARATE_OWNER_RUNS")
    print("R13_PILOT=EXCLUDED_FROM_REPLICATION")
    print("PRODUCTION=NOT_AUTHORIZED")


def validate_sample(rows, count=BLOCKS):
    if not isinstance(rows, list) or len(rows) != count:
        raise Blocked("session blocks missing")
    orders=[]
    normalized=[]
    for index, row in enumerate(rows, start=1):
        if (not isinstance(row, dict) or set(row) != {"block", "candidate_first", "baseline_ns", "candidate_ns"}
                or type(row["block"]) is not int or row["block"] != index
                or type(row["candidate_first"]) is not bool
                or type(row["baseline_ns"]) is not int or type(row["candidate_ns"]) is not int
                or not 0 < row["baseline_ns"] < 120_000_000_000
                or not 0 < row["candidate_ns"] < 120_000_000_000):
            raise Blocked("invalid raw sample")
        orders.append(row["candidate_first"])
        normalized.append((row["baseline_ns"] / ITERATIONS,
                           row["candidate_ns"] / ITERATIONS))
    if sum(orders) != BLOCKS // 2:
        raise Blocked("A/B invocation order imbalance")
    base = [x for x, _ in normalized]
    cand = [x for _, x in normalized]
    diffs = [a - b for a, b in normalized]
    return {"baseline_median_ns_per_op": round(statistics.median(base), 3),
            "candidate_median_ns_per_op": round(statistics.median(cand), 3),
            "paired_delta_median_ns_per_op": round(statistics.median(diffs), 3),
            "paired_delta_percent_of_baseline": round(100 * statistics.median(diffs) / statistics.median(base), 3),
            "candidate_faster_blocks": sum(d > 0 for d in diffs),
            "candidate_slower_blocks": sum(d < 0 for d in diffs),
            "equal_blocks": sum(d == 0 for d in diffs),
            "candidate_first": BLOCKS // 2,
            "baseline_first": BLOCKS // 2}


def validate_session(root, session):
    report = read_json(root / (session + ".json"))
    if (report.get("schema") != SCHEMA or report.get("session") != session
            or report.get("source_sha") != BASE_SHA
            or report.get("candidate_sha") != CANDIDATE_SHA
            or report.get("r13_git_blob") != R13_BLOB
            or report.get("probe_sha256") != R13_PROBE_SHA256
            or report.get("iterations") != ITERATIONS or report.get("blocks") != BLOCKS
            or report.get("whole_process_cli_excluded") is not True
            or report.get("independence_proven_by_software") is not False
            or report.get("human_evaluation_gate") != "CLOSED_NOT_REQUIRED"
            or report.get("production") != "NOT_AUTHORIZED"
            or not isinstance(report.get("nonce"), str) or len(report["nonce"]) != 32
            or not all(c in "0123456789abcdef" for c in report["nonce"])
            or not isinstance(report.get("host"), dict)
            or report["host"].get("system") != "Darwin"
            or report["host"].get("machine") != "x86_64"
            or not isinstance(report.get("started_unix_s"), int)
            or not isinstance(report.get("completed_unix_s"), int)
            or report["completed_unix_s"] < report["started_unix_s"]
            or not isinstance(report.get("source_guard"), dict)
            or report["source_guard"].get("source_delta") != "EXACT_DUPLICATE_EXISTENCE_LOOKUP_ONLY"
            or not isinstance(report.get("build"), dict)
            or report["build"].get("offline") is not True
            or report["build"].get("release") is not True
            or not isinstance(report.get("cases"), dict)
            or set(report["cases"]) != set(CASES)):
        raise Blocked("invalid session provenance or structure")
    summaries = {name:validate_sample(report["cases"][name]) for name in CASES}
    return report, summaries


def validate_prior(root, slot):
    idx = SLOTS.index(slot)
    for prev in SLOTS[:idx]:
        if not (root / (prev + ".json")).exists():
            raise Blocked("complete prior session first")
        validate_session(root, prev)
    if (root / (slot + ".json")).exists() or (root / (slot + ".json")).is_symlink():
        raise Blocked("session already recorded; no overwrite")


def run_session(root, slot, r13_path):
    if platform.system() != "Darwin" or platform.machine() != "x86_64":
        raise Blocked("actual run only validated for owner's Mac Intel")
    root=study_dir(root)
    validate_protocol(root)
    if slot not in SLOTS:
        raise Blocked("unknown session slot")
    validate_prior(root,slot)
    runner = pinned_r13(r13_path)
    if any(shutil.which(x) is None for x in ("git", "cargo", "rustc")):
        raise Blocked("toolchain unavailable")
    started=int(time.time())
    with tempfile.TemporaryDirectory(prefix="NORDOI_R314_MEASURE_") as folder:
        temp=Path(folder)
        baseline=temp/"baseline"; candidate=temp/"candidate"
        runner.clone(baseline,"main",BASE_SHA)
        runner.clone(candidate,runner.BRANCH,CANDIDATE_SHA)
        guard=runner.verify_delta(baseline/"src/kernel.rs",candidate/"src/kernel.rs")
        allowed={"src/kernel.rs","research/g03_r3_r13_kernel_probe.rs",
                 "research/g03_r3_r13_paired_mac.py",
                 "research/G03_R3_R13_ENGINEERING.md",
                 "tests/g03_r3_r13_redundant_lookup.rs",
                 ".github/workflows/g03_r3_r13_paired.yml"}
        old=set(runner.run(["git","ls-files"],cwd=baseline).decode().splitlines())
        new=set(runner.run(["git","ls-files"],cwd=candidate).decode().splitlines())
        if not (old ^ new).issubset(allowed):
            raise Blocked("tracked file inventory changed")
        for name in old & new - {"src/kernel.rs"}:
            if (baseline/name).read_bytes() != (candidate/name).read_bytes():
                raise Blocked("unrelated tracked content changed")
        probe=candidate/"research/g03_r3_r13_kernel_probe.rs"
        if hashlib.sha256(file_read(probe, limit=20_000)).hexdigest()!=R13_PROBE_SHA256:
            raise Blocked("R13 Rust probe altered")
        a, a_build, a_binary=runner.compile_probe(baseline,temp/"a",probe)
        b, b_build, b_binary=runner.compile_probe(candidate,temp/"b",probe)
        for name in CASES:
            runner.one(a,name,1000)
            runner.one(b,name,1000)
        schedules = {}
        srng=secrets.SystemRandom()
        for name in CASES:
            schedule=[False]*(BLOCKS//2)+[True]*(BLOCKS//2)
            srng.shuffle(schedule)
            schedules[name]=schedule
        cases={name:[] for name in CASES}
        for block in range(BLOCKS):
            for offset in range(len(CASES)):
                name=CASES[(block+offset)%len(CASES)]
                candidate_first=schedules[name][block]
                order=((("candidate",b),("baseline",a)) if candidate_first
                       else (("baseline",a),("candidate",b)))
                row={"block":block+1,"candidate_first":candidate_first}
                for arm,executable in order:
                    row[arm+"_ns"]=runner.one(executable,name,ITERATIONS)
                cases[name].append(row)
        # Structural verification before any persistent write.
        for name in CASES: validate_sample(cases[name])
        data={"schema":SCHEMA,"session":slot,"nonce":secrets.token_hex(16),
              "source_sha":BASE_SHA,"candidate_sha":CANDIDATE_SHA,
              "r13_git_blob":R13_BLOB,"probe_sha256":R13_PROBE_SHA256,
              "source_guard":guard,"started_unix_s":started,"completed_unix_s":int(time.time()),
              "host":{"system":platform.system(),"machine":platform.machine(),
                      "python":platform.python_version(),"cpu_count":os.cpu_count()},
              "build":{"offline":True,"release":True,"baseline_seconds":a_build,
                       "candidate_seconds":b_build},
              "binaries":{"baseline_sha256":a_binary,"candidate_sha256":b_binary},
              "blocks":BLOCKS,"iterations":ITERATIONS,"cases":cases,
              "whole_process_cli_excluded":True,
              "independence_proven_by_software":False,
              "human_evaluation_gate":"CLOSED_NOT_REQUIRED",
              "production":"NOT_AUTHORIZED"}
        private_write(root/(slot+".json"),data)
    print("G03_R3_R14_"+slot+"_SESSION=PASS")
    print("ORDER_BALANCE=4_AB_4_BA_PER_CASE")
    print("SOURCE_GUARD=PASS")
    summary=validate_session(root,slot)[1]["noop_set"]
    print("NOOP_SET_PAIRED_DELTA_NS="+str(summary["paired_delta_median_ns_per_op"]))
    print("NOOP_SET_FASTER_BLOCKS="+str(summary["candidate_faster_blocks"])+"/8")
    print("INDEPENDENT_SESSION_PROVENANCE=NOT_VERIFIED_BY_SOFTWARE")
    print("REPORT_PRIVATE="+str(root/(slot+".json")))
    print("PRODUCTION=NOT_AUTHORIZED")


def analyze(root):
    root=study_dir(root)
    validate_protocol(root)
    present=[slot for slot in SLOTS if (root/(slot+".json")).exists()]
    if present != list(SLOTS):
        print("G03_R3_R14_REPLICATION=INCOMPLETE")
        print("COMPLETED_SLOTS="+",".join(present))
        print("INDEPENDENT_SESSION_PROVENANCE=NOT_VERIFIED_BY_SOFTWARE")
        print("PRODUCTION=NOT_AUTHORIZED")
        return 0
    reports=[]; per_case={name:[] for name in CASES}
    for slot in SLOTS:
        report,stats=validate_session(root,slot)
        reports.append(report)
        for name in CASES: per_case[name].append(stats[name])
    if len({r["nonce"] for r in reports}) != len(SLOTS):
        raise Blocked("duplicate session nonce")
    if any(reports[i]["started_unix_s"] < reports[i-1]["completed_unix_s"] for i in range(1,3)):
        raise Blocked("chronological sessions overlap")
    key=per_case["noop_set"]
    direction=sum(x["paired_delta_median_ns_per_op"]>0 for x in key)
    strong=sum(x["paired_delta_percent_of_baseline"]>=5 and x["candidate_faster_blocks"]>=5 for x in key)
    assessment="ENGINEERING_REVIEW_ONLY" if direction==3 and strong==3 else "MIXED_OR_WEAK_REMEASURE"
    print("G03_R3_R14_REPLICATION=3_SESSIONS_STRUCTURALLY_VERIFIED")
    for slot, stats in zip(SLOTS,key):
        print(slot+"_NOOP_SET_PAIRED_PERCENT="+str(stats["paired_delta_percent_of_baseline"]))
        print(slot+"_NOOP_SET_FASTER_BLOCKS="+str(stats["candidate_faster_blocks"])+"/8")
    print("NOOP_SET_POSITIVE_SESSION_COUNT="+str(direction)+"/3")
    print("G03_R3_R14_ENGINEERING_ASSESSMENT="+assessment)
    print("INDEPENDENT_SESSION_PROVENANCE=NOT_VERIFIED_BY_SOFTWARE")
    print("MERGE_AUTHORIZATION=NONE")
    print("PRODUCTION=NOT_AUTHORIZED")
    return 0


def selftest():
    assert git_blob(b"123") == "d800886d9c86731ae5c4a62b0b77c437015e00d2"
    sample=[{"block":i+1,"candidate_first":(i%2==0),
             "baseline_ns":100*ITERATIONS,"candidate_ns":80*ITERATIONS} for i in range(BLOCKS)]
    z=validate_sample(sample)
    assert z["candidate_first"]==4 and z["candidate_faster_blocks"]==8
    assert z["paired_delta_percent_of_baseline"]==20
    bad=[dict(item) for item in sample];bad[0]["candidate_first"]=False
    try: validate_sample(bad)
    except Blocked: pass
    else: raise AssertionError("unbalanced invocation order accepted")
    bad=[dict(item) for item in sample];bad[0]["candidate_ns"]=-1
    try: validate_sample(bad)
    except Blocked: pass
    else: raise AssertionError("negative duration accepted")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R314_SYNTHETIC_") as d:
        root=Path(d)/"study"
        initialize(root)
        assert read_json(root/"protocol.json")==protocol()
        assert analyze(root)==0  # No fabricated sessions.
        try: initialize(root)
        except Blocked: pass
        else: raise AssertionError("duplicate initialization accepted")
        bad_p=dict(protocol(), blocks=9)
        (root/"protocol.json").write_text(json.dumps(bad_p),encoding="utf-8")
        try: validate_protocol(root)
        except Blocked: pass
        else: raise AssertionError("protocol alteration accepted")
    print("G03_R3_R14_SELFTEST=SYNTHETIC_PASS")
    print("REAL_MAC_SESSIONS=0_IN_SELFTEST")
    print("HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED")
    print("PRODUCTION=NOT_AUTHORIZED")


def main():
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    sub.add_parser("selftest")
    i=sub.add_parser("init");i.add_argument("--out",required=True)
    s=sub.add_parser("run-session");s.add_argument("--study-dir",required=True)
    s.add_argument("--slot",choices=SLOTS,required=True)
    s.add_argument("--r13-tool",required=True)
    a=sub.add_parser("analyze");a.add_argument("--study-dir",required=True)
    args=p.parse_args()
    try:
        if args.action=="selftest": selftest()
        elif args.action=="init":initialize(args.out)
        elif args.action=="run-session":run_session(args.study_dir,args.slot,args.r13_tool)
        else: analyze(args.study_dir)
        return 0
    except (Blocked,ValueError,OSError,KeyError,TypeError,AssertionError,UnicodeError,AttributeError):
        print("G03_R3_R14_GATE=BLOCKED",file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2


if __name__=="__main__":
    sys.exit(main())
