#!/usr/bin/env python3
"""NORDOI G0.3 R3-R2: prebuilt CLI vs Cargo frontend, NOT language-engine speed."""
import argparse, hashlib, json, os, random, statistics, subprocess, tempfile, time
from pathlib import Path
from math import comb

REMOTE="https://github.com/eddarsglobal/nordoi.git"
MAIN="7442b06238dfa9351ae932c0cc636c0c99c3dfea"
PROD="e2898bf7c511e53636ff28c9fd3efc93165a0215"
PINS={"examples/p2_1_console/hello.noi":"e1b74cdabb63716744832c518e9e81e41aa3f14b",
"docs/NORDOI_G02_FIRST_USE_QUICKSTART.md":"0aa6b2af0d67c55b9d70d89d9b8b0fb5d970646e"}
HELLO="examples/p2_1_console/hello.noi"
EXPECTED=b"Hello from NORDOI P2.1!\n"
SEED=20261107

def blob(data):
    return hashlib.sha1(b"blob "+str(len(data)).encode()+b"\0"+data).hexdigest()

def git(*args):
    env=dict(os.environ,GIT_TERMINAL_PROMPT="0")
    return subprocess.check_output(["git",*args],env=env,stderr=subprocess.PIPE,timeout=120).decode().strip()

def schedule():
    rng=random.Random(SEED)
    blocks=[]
    for _ in range(8):
        block=["cargo_first"]*6+["direct_first"]*6
        rng.shuffle(block)
        blocks.append(block)
    return blocks

def infer(deltas):
    rng=random.Random(SEED^0xB00)
    samples=sorted(statistics.median([rng.choice(deltas) for _ in deltas]) for _ in range(10000))
    nonzero=[d for d in deltas if d!=0]
    n=len(nonzero)
    k=min(sum(d>0 for d in nonzero),sum(d<0 for d in nonzero)) if n else 0
    p=min(1.0,2*sum(comb(n,i) for i in range(k+1))/2**n) if n else 1.0
    return [samples[250],samples[9750]],p

def selftest():
    hello=b'module app.main;\n\neffect ConsoleWrite;\n\nentry main emits "Hello from NORDOI P2.1!\\n";\n'
    assert len(hello)==86 and blob(hello)==PINS[HELLO]
    assert all(x.count("cargo_first")==6 and x.count("direct_first")==6 for x in schedule())
    assert infer([1.0]*8)[1]==0.0078125
    assert infer([2.0]*8)[0]==[2.0,2.0]
    print("G03_R3_R2_SELFTEST=PASS",flush=True)

def check():
    for branch,expected in [("main",MAIN),("production/p0.1",PROD)]:
        result=git("ls-remote","--exit-code",REMOTE,"refs/heads/"+branch)
        if result.split()[0]!=expected:
            raise RuntimeError("REMOTE_PIN_DRIFT_"+branch)
    print("G03_R3_R2_REMOTE_PINS=PASS",flush=True)

def timed(cmd,repo,env,receipt=None):
    start=time.perf_counter_ns()
    r=subprocess.run(cmd,cwd=repo,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=40)
    elapsed=time.perf_counter_ns()-start
    if r.returncode!=0 or r.stdout!=EXPECTED:
        raise RuntimeError("OUTPUT_OR_EXIT_PARITY_FAILED")
    if b"authority=EXPLICIT" not in r.stderr or b"ambient-authority=NONE" not in r.stderr:
        raise RuntimeError("AUTHORITY_RECEIPT_FAILED")
    if receipt is not None and r.stderr!=receipt:
        raise RuntimeError("STDERR_RECEIPT_PARITY_FAILED")
    return elapsed,r.stderr

def run(output):
    if output.exists() or output.is_symlink() or not output.parent.is_dir() or output.parent.is_symlink():
        raise RuntimeError("OUTPUT_EXISTS_OR_INVALID_DIRECTORY")
    with tempfile.TemporaryDirectory(prefix="nordoi-r3-r2-") as tmp:
        root=Path(tmp); repo=root/"repo"
        git("clone","--quiet","--depth","1","--branch","main","--single-branch",REMOTE,str(repo))
        if git("-C",str(repo),"rev-parse","HEAD")!=MAIN:
            raise RuntimeError("ISOLATED_CLONE_SHA_MISMATCH")
        for rel,pin in PINS.items():
            file=repo/rel
            if file.is_symlink() or blob(file.read_bytes())!=pin:
                raise RuntimeError("ISOLATED_SOURCE_BLOB_MISMATCH_"+rel)
        print("G03_R3_R2_SOURCE_PINS=PASS",flush=True)
        env=dict(os.environ,CARGO_TARGET_DIR=str(root/"target"),
                 CARGO_INCREMENTAL="0",CARGO_NET_OFFLINE="true")
        start=time.perf_counter_ns()
        build=subprocess.run(["cargo","build","--offline","--release","--bin","nordoi"],
            cwd=repo,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=600)
        build_ns=time.perf_counter_ns()-start
        if build.returncode:
            print(build.stderr.decode(errors="replace")[-3500:])
            raise RuntimeError("CARGO_RELEASE_BUILD_FAILED")
        binary=root/"target"/"release"/"nordoi"
        if not binary.is_file() or binary.is_symlink():
            raise RuntimeError("RELEASE_BINARY_MISSING")
        print("G03_R3_R2_RELEASE_BUILD=PASS",flush=True)
        cargo=["cargo","run","--quiet","--offline","--release","--bin","nordoi",
               "--","run",HELLO,"--grant-console"]
        direct=[str(binary),"run",HELLO,"--grant-console"]
        _,receipt=timed(direct,repo,env)
        timed(cargo,repo,env,receipt)
        deny=subprocess.run([str(binary),"console-run",HELLO],cwd=repo,
             env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
        if deny.returncode!=5 or deny.stdout:
            raise RuntimeError("DENIAL_FAIL_CLOSED_REGRESSION")
        print("G03_R3_R2_RECEIPT_AND_DENIAL=PASS",flush=True)
        for _ in range(4):
            timed(cargo,repo,env,receipt);timed(direct,repo,env,receipt)
        raw=[]
        for index,orders in enumerate(schedule(),1):
            pairs=[]
            for order in orders:
                if order=="cargo_first":
                    a,_=timed(cargo,repo,env,receipt);b,_=timed(direct,repo,env,receipt)
                else:
                    b,_=timed(direct,repo,env,receipt);a,_=timed(cargo,repo,env,receipt)
                pairs.append({"order":order,"cargo_ns":a,"direct_ns":b,"paired_delta_ns":a-b})
            raw.append({"block":index,"pairs":pairs})
            med=statistics.median(p["paired_delta_ns"]/1e6 for p in pairs)
            print(f"G03_R3_R2_BLOCK_{index:02d}_CARGO_MINUS_DIRECT_MS={med:.6f}",flush=True)
            if index<8:time.sleep(0.15)
        deltas=[statistics.median(p["paired_delta_ns"]/1e6 for p in block["pairs"])
                for block in raw]
        bounds,p_value=infer(deltas)
        report={"schema":"NORDOI-G03-R3-R2-INVOCATION-v1","main_sha":MAIN,
            "production_unchanged_sha":PROD,
            "scope":"WHOLE_PROCESS_CARGO_FRONTEND_VS_DIRECT_PREBUILT_EXECUTABLE",
            "not_a_measure_of":"INTRINSIC_NORDOI_RUNTIME_SPEED_OR_HUMAN_USABILITY",
            "one_time_release_build_ms":build_ns/1e6,"seed":SEED,
            "blocks":8,"pairs_per_block":12,"warmups_per_mode":4,
            "block_paired_deltas_ms":deltas,
            "median_block_paired_delta_ms":statistics.median(deltas),
            "block_bootstrap_95pct_ci_ms":bounds,"block_sign_p_two_sided":p_value,
            "cargo_global_median_ms":statistics.median(p["cargo_ns"]/1e6 for b in raw for p in b["pairs"]),
            "direct_global_median_ms":statistics.median(p["direct_ns"]/1e6 for b in raw for p in b["pairs"]),
            "raw_blocks":raw,"output_and_receipt_parity":"PASS",
            "explicit_denial":"PASS","original_source_modifications":"NONE",
            "github_push":"NONE","production":"NOT_AUTHORIZED"}
        data=(json.dumps(report,indent=2,sort_keys=True)+"\n").encode()
        fd=os.open(str(output),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,"wb") as f:f.write(data)
        print("G03_R3_R2_MEDIAN_PAIRED_CARGO_OVERHEAD_MS="+
              str(report["median_block_paired_delta_ms"]))
        print("G03_R3_R2_CI95_MS="+str(bounds))
        print("G03_R3_R2_REPORT="+str(output))
        print("G03_R3_R2_ENGINE_SPEED=NOT_MEASURED")
        print("G03_R3_R2_HUMAN_LEARNING=UNMEASURED")
        print("ORIGINAL_REPO=UNTOUCHED")
        print("GITHUB_PUSH=NONE")
        print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode",choices=["selftest","check","run"])
    p.add_argument("--output",type=Path)
    a=p.parse_args()
    selftest()
    if a.mode=="selftest":return
    check()
    if a.mode=="check":return
    if not a.output:p.error("run requires --output")
    run(a.output)

if __name__=="__main__":
    try:main()
    except Exception as exc:
        print("G03_R3_R2_ERROR="+str(exc))
        raise SystemExit(1)
