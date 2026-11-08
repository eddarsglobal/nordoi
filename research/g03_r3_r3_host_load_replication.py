#!/usr/bin/env python3
"""NORDOI G0.3 R3-R3: Cargo frontend vs direct prebuilt executable, separate-session replication.
Measures process launch overhead only, NOT intrinsic language-engine speed or human usability.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import tempfile
import time

REMOTE = "https://github.com/eddarsglobal/nordoi.git"
MAIN = "7442b06238dfa9351ae932c0cc636c0c99c3dfea"
PROD = "e2898bf7c511e53636ff28c9fd3efc93165a0215"
HELLO = "examples/p2_1_console/hello.noi"
PINS = {
    HELLO: "e1b74cdabb63716744832c518e9e81e41aa3f14b",
    "docs/NORDOI_G02_FIRST_USE_QUICKSTART.md": "0aa6b2af0d67c55b9d70d89d9b8b0fb5d970646e",
}
EXPECTED = b"Hello from NORDOI P2.1!\n"
BLOCKS, PER_BLOCK, WARMUPS = 8, 12, 4
SEED, BOOTS = 20261107, 10000

def blob(data):
    return hashlib.sha1(b"blob "+str(len(data)).encode()+b"\0"+data).hexdigest()

def git(*args):
    e = dict(os.environ, GIT_TERMINAL_PROMPT="0")
    return subprocess.check_output(["git", *args], env=e, stderr=subprocess.PIPE, timeout=120).decode().strip()

def check():
    for ref, pin in [("main", MAIN), ("production/p0.1", PROD)]:
        value = git("ls-remote", "--exit-code", REMOTE, "refs/heads/"+ref)
        if not value or value.split()[0] != pin:
            raise RuntimeError("REMOTE_REF_DRIFT_"+ref)
    print("G03_R3_R3_REMOTE_MAIN_PRODUCTION=PASS", flush=True)

def orders(rep):
    rng = random.Random(SEED+rep*7919)
    blocks = []
    for _ in range(BLOCKS):
        seq = ["cargo_first"]*6+["direct_first"]*6
        rng.shuffle(seq)
        blocks.append(seq)
    return blocks

def sign_test(vals):
    from math import comb
    n = sum(v!=0 for v in vals)
    if not n: return 1.0
    k = min(sum(v>0 for v in vals), sum(v<0 for v in vals))
    return min(1.0, 2*sum(comb(n,i) for i in range(k+1))/2**n)

def stats(blocks, rep):
    if len(blocks)!=BLOCKS: raise ValueError("BAD_BLOCK_COUNT")
    meds, all_pairs = [], []
    expected = orders(rep)
    for idx, block in enumerate(blocks):
        pairs = block.get("pairs", [])
        if block.get("block")!=idx+1 or len(pairs)!=PER_BLOCK:
            raise ValueError("BAD_PAIR_COUNT_OR_ORDER")
        if [p.get("order") for p in pairs] != expected[idx]:
            raise ValueError("UNREGISTERED_ORDER")
        for p in pairs:
            a,b,d = (p.get(k) for k in ("cargo_ns","direct_ns","paired_delta_ns"))
            if type(a)!=int or type(b)!=int or type(d)!=int or a<=0 or b<=0 or a-b!=d:
                raise ValueError("BAD_RAW_DELTA")
            all_pairs.append(p)
        meds.append(statistics.median(p["paired_delta_ns"]/1e6 for p in pairs))
    rng = random.Random(SEED^(rep*32003))
    boot = sorted(statistics.median([rng.choice(meds) for _ in meds]) for _ in range(BOOTS))
    center=(BLOCKS-1)/2
    slope=sum((i-center)*(v-statistics.mean(meds)) for i,v in enumerate(meds))/sum((i-center)**2 for i in range(BLOCKS))
    return {
        "block_paired_medians_ms":meds,
        "primary_median_ms":statistics.median(meds),
        "block_bootstrap_95pct_ms":[boot[250], boot[9750]],
        "block_sign_p_two_sided":sign_test(meds),
        "global_cargo_median_ms":statistics.median(p["cargo_ns"]/1e6 for p in all_pairs),
        "global_direct_median_ms":statistics.median(p["direct_ns"]/1e6 for p in all_pairs),
        "all_96_pairs_cargo_slower":all(p["paired_delta_ns"]>0 for p in all_pairs),
        "first_to_last_block_drift_pct":100*(meds[-1]/meds[0]-1) if meds[0] else None,
        "block_delta_slope_ms_per_block":slope,
        "cargo_first_pairs":sum(p["order"]=="cargo_first" for p in all_pairs),
        "direct_first_pairs":sum(p["order"]=="direct_first" for p in all_pairs),
    }

def selftest():
    hello = b'module app.main;\n\neffect ConsoleWrite;\n\nentry main emits "Hello from NORDOI P2.1!\\n";\n'
    assert len(hello)==86 and blob(hello)==PINS[HELLO]
    assert orders(1)!=orders(2)
    for rep in (1,2):
        assert all(x.count("cargo_first")==6 and x.count("direct_first")==6 for x in orders(rep))
    assert sign_test([1]*8)==0.0078125
    fixture=[{"block":i,"pairs":[{"order":o,"cargo_ns":9000000,"direct_ns":1000000,"paired_delta_ns":8000000} for o in seq]} for i,seq in enumerate(orders(1),1)]
    s=stats(fixture,1)
    assert s["primary_median_ms"]==8 and s["cargo_first_pairs"]==48
    fixture[0]["pairs"][0]["paired_delta_ns"]+=1
    try: stats(fixture,1)
    except ValueError as e: assert str(e)=="BAD_RAW_DELTA"
    else: raise AssertionError("INVALID_DATA_WAS_ACCEPTED")
    print("G03_R3_R3_SELFTEST=PASS",flush=True)

def timing(command, repo, env, receipt=None):
    start=time.perf_counter_ns()
    p=subprocess.run(command,cwd=repo,env=env,capture_output=True,timeout=60)
    ns=time.perf_counter_ns()-start
    if p.returncode!=0 or p.stdout!=EXPECTED:
        raise RuntimeError("OUTPUT_OR_EXIT_MISMATCH")
    if b"authority=EXPLICIT" not in p.stderr or b"ambient-authority=NONE" not in p.stderr:
        raise RuntimeError("EXPLICIT_AUTHORITY_MISSING")
    if receipt is not None and p.stderr!=receipt:
        raise RuntimeError("STDERR_RECEIPT_PARITY_FAILED")
    return ns,p.stderr

def load():
    a,b,c=os.getloadavg()
    return {"1m":a, "5m":b, "15m":c}

def safe_write(path, value):
    if path.is_symlink() or path.exists() or not path.parent.is_dir() or path.parent.is_symlink():
        raise RuntimeError("REPORT_EXISTS_OR_PARENT_INVALID")
    fd=os.open(str(path),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
    with os.fdopen(fd,"wb") as f:
        f.write((json.dumps(value,sort_keys=True,indent=2)+"\n").encode("utf-8"))

def run(rep,output):
    if platform.system()!="Darwin" or platform.machine() not in ("x86_64","i386"):
        raise RuntimeError("MAC_INTEL_ONLY")
    if output.exists() or output.is_symlink() or not output.parent.is_dir() or output.parent.is_symlink():
        raise RuntimeError("REPORT_EXISTS_OR_PARENT_INVALID")
    check()
    started=datetime.datetime.now(datetime.timezone.utc).isoformat()
    with tempfile.TemporaryDirectory(prefix="nordoi-r3-r3-") as d:
        root=Path(d); repo=root/"repo"
        git("clone","--quiet","--depth","1","--single-branch","--branch","main",REMOTE,str(repo))
        if git("-C",str(repo),"rev-parse","HEAD")!=MAIN:
            raise RuntimeError("CLONE_PIN_DRIFT")
        for path,pin in PINS.items():
            f=repo/path
            if f.is_symlink() or not f.is_file() or blob(f.read_bytes())!=pin:
                raise RuntimeError("SOURCE_BLOB_DRIFT_"+path)
        print("G03_R3_R3_SOURCE_PINS=PASS",flush=True)
        env=dict(os.environ, CARGO_TARGET_DIR=str(root/"target"),
                 CARGO_INCREMENTAL="0", CARGO_NET_OFFLINE="true")
        start=time.perf_counter_ns()
        build=subprocess.run(["cargo","build","--offline","--release","--bin","nordoi"],
                             cwd=repo,env=env,capture_output=True,timeout=900)
        build_ns=time.perf_counter_ns()-start
        if build.returncode:
            print(build.stderr.decode(errors="replace")[-3000:],flush=True)
            raise RuntimeError("OFFLINE_RELEASE_BUILD_FAILED")
        binary=root/"target"/"release"/"nordoi"
        if binary.is_symlink() or not binary.is_file():
            raise RuntimeError("BINARY_NOT_FOUND")
        print("G03_R3_R3_RELEASE_BUILD=PASS",flush=True)
        direct=[str(binary),"run",HELLO,"--grant-console"]
        cargo=["cargo","run","--quiet","--offline","--release","--bin","nordoi",
               "--","run",HELLO,"--grant-console"]
        _,receipt=timing(direct,repo,env)
        timing(cargo,repo,env,receipt)
        denied=subprocess.run([str(binary),"console-run",HELLO],cwd=repo,env=env,
                              capture_output=True,timeout=30)
        if denied.returncode!=5 or denied.stdout:
            raise RuntimeError("UNGRANTED_DENIAL_FAILED")
        print("G03_R3_R3_RECEIPT_AND_DENIAL=PASS",flush=True)
        for i in range(WARMUPS):
            if i%2:
                timing(cargo,repo,env,receipt);timing(direct,repo,env,receipt)
            else:
                timing(direct,repo,env,receipt);timing(cargo,repo,env,receipt)
        blocks=[]
        for index,seq in enumerate(orders(rep),1):
            l0=load()
            pairs=[]
            for order in seq:
                if order=="cargo_first":
                    a,_=timing(cargo,repo,env,receipt)
                    b,_=timing(direct,repo,env,receipt)
                else:
                    b,_=timing(direct,repo,env,receipt)
                    a,_=timing(cargo,repo,env,receipt)
                pairs.append({"order":order,"cargo_ns":a,"direct_ns":b,"paired_delta_ns":a-b})
            blocks.append({"block":index,"pairs":pairs,"load_before":l0,"load_after":load()})
            med=statistics.median(p["paired_delta_ns"]/1e6 for p in pairs)
            print("G03_R3_R3_BLOCK_%02d_DELTA_MS=%.6f"%(index,med),flush=True)
            if index<BLOCKS: time.sleep(0.15)
        summary=stats(blocks,rep)
        report={
            "schema":"NORDOI-G03-R3-R3-REPLICATION-v1",
            "scope":"WHOLE_PROCESS_CARGO_FRONTEND_VS_DIRECT_PREBUILT_EXECUTABLE",
            "not_a_measure_of":"ENGINE_SPEED_OR_HUMAN_LEARNING",
            "replicate":rep,"seed":SEED+rep*7919,
            "started_utc":started,"finished_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "host_observations":{"os":"Darwin","arch":platform.machine(),
                  "logical_cpus":os.cpu_count(),"macos_version":platform.mac_ver()[0]},
            "main_sha":MAIN,"production_sha":PROD,"build_once_ms":build_ns/1e6,
            "blocks":BLOCKS,"pairs_per_block":PER_BLOCK,"warmups_per_mode":WARMUPS,
            "raw_blocks":blocks,"summary":summary,
            "assumptions":"ONE_MAC; LOAD_AVERAGE_CORRELATION_NOT_CAUSATION; BLOCKS_MAY_AUTOCORRELATE",
            "source_modifications":"NONE","github_push":"NONE","production":"NOT_AUTHORIZED",
        }
        safe_write(output,report)
        print("G03_R3_R3_SESSION_%d=PASS"%rep,flush=True)
        print("G03_R3_R3_PRIMARY_PAIRED_DELTA_MS=%.6f"%summary["primary_median_ms"],flush=True)
        print("G03_R3_R3_BLOCK_DRIFT_PCT=%.3f"%summary["first_to_last_block_drift_pct"],flush=True)
        print("G03_R3_R3_REPORT="+str(output),flush=True)
        print("ENGINE_SPEED=NOT_MEASURED",flush=True)
        print("HUMAN_LEARNING=NOT_MEASURED",flush=True)
        print("PRODUCTION=NOT_AUTHORIZED",flush=True)

def compare(baseline,first,second,output):
    docs=[]
    for path in (baseline,first,second):
        if path.is_symlink() or not path.is_file() or path.stat().st_size>1000000:
            raise ValueError("MISSING_OR_INVALID_INPUT")
        docs.append(json.loads(path.read_text(encoding="utf-8")))
    old,a,b=docs
    if old.get("schema")!="NORDOI-G03-R3-R2-INVOCATION-v1" or old.get("main_sha")!=MAIN or old.get("production_unchanged_sha")!=PROD:
        raise ValueError("BASELINE_PIN_MISMATCH")
    base=old.get("raw_blocks",[])
    if len(base)!=BLOCKS: raise ValueError("BASELINE_BLOCK_COUNT")
    olds=[]
    for x in base:
        pairs=x.get("pairs",[])
        if len(pairs)!=PER_BLOCK: raise ValueError("BASELINE_PAIR_COUNT")
        if sum(p["order"]=="cargo_first" for p in pairs)!=6: raise ValueError("BASELINE_ORDER_BALANCE")
        for p in pairs:
            if p["cargo_ns"]-p["direct_ns"]!=p["paired_delta_ns"]:
                raise ValueError("BASELINE_PAIR_DELTA")
        olds.append(statistics.median(p["paired_delta_ns"]/1e6 for p in pairs))
    if abs(statistics.median(olds)-old["median_block_paired_delta_ms"])>1e-6:
        raise ValueError("BASELINE_AGGREGATE_DRIFT")
    sums=[]
    for index,doc in ((1,a),(2,b)):
        if doc.get("schema")!="NORDOI-G03-R3-R3-REPLICATION-v1" or doc.get("replicate")!=index or doc.get("main_sha")!=MAIN or doc.get("production_sha")!=PROD:
            raise ValueError("SESSION_PIN_DRIFT")
        recomputed=stats(doc["raw_blocks"],index)
        if abs(doc["summary"]["primary_median_ms"]-recomputed["primary_median_ms"])>1e-6:
            raise ValueError("SESSION_AGGREGATE_DRIFT")
        sums.append(recomputed)
    result={
        "schema":"NORDOI-G03-R3-R3-COMPARISON-v1",
        "reference_r3_r2_median_delta_ms":statistics.median(olds),
        "replicate1_median_delta_ms":sums[0]["primary_median_ms"],
        "replicate2_median_delta_ms":sums[1]["primary_median_ms"],
        "replicate1_drift_pct":sums[0]["first_to_last_block_drift_pct"],
        "replicate2_drift_pct":sums[1]["first_to_last_block_drift_pct"],
        "all_new_pairs_positive":all(s["all_96_pairs_cargo_slower"] for s in sums),
        "input_sha256":{"baseline":hashlib.sha256(baseline.read_bytes()).hexdigest(),
             "replicate1":hashlib.sha256(first.read_bytes()).hexdigest(),
             "replicate2":hashlib.sha256(second.read_bytes()).hexdigest()},
        "conclusion_scope":"DESCRIPTIVE_REPEATABILITY_ONE_MAC_ONLY",
        "engine_speed":"NOT_MEASURED","human_learning":"NOT_MEASURED",
        "production":"NOT_AUTHORIZED",
    }
    safe_write(output,result)
    print("G03_R3_R3_COMPARISON=PASS",flush=True)
    print("G03_R3_R3_REPORT="+str(output),flush=True)
    print("PRODUCTION=NOT_AUTHORIZED",flush=True)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode", choices=["selftest","check","run","compare"])
    p.add_argument("--replicate",type=int,choices=[1,2])
    p.add_argument("--output",type=Path)
    p.add_argument("--baseline",type=Path)
    p.add_argument("--session1",type=Path)
    p.add_argument("--session2",type=Path)
    a=p.parse_args()
    selftest()
    if a.mode=="selftest": return
    if a.mode=="check": return check()
    if a.mode=="run":
        if a.replicate is None or a.output is None:
            p.error("run requires --replicate and --output")
        return run(a.replicate,a.output)
    if None in (a.baseline,a.session1,a.session2,a.output):
        p.error("compare requires --baseline, --session1, --session2, --output")
    compare(a.baseline,a.session1,a.session2,a.output)

if __name__=="__main__":
    try: main()
    except (RuntimeError,ValueError,AssertionError,subprocess.SubprocessError,OSError) as exc:
        print("G03_R3_R3_ERROR="+str(exc),flush=True)
        raise SystemExit(1)
