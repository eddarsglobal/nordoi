#!/usr/bin/env python3
"""APP-005 native JSONL batch caller, no Python business decisions.

One actual NORDOI process evaluates multiple .noi inputs; all requests are
validated by the Rust parser before any file output. The caller only creates
a fresh private root and verifies genuine P2.7 file bytes from native
result records. No human test / network / separate AI runtime is required.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE=REPO/"examples/apps/app003_integrated_quality/src"
sys.path.insert(0,str(REPO/"examples/apps/app004_agent_json"))
import run_app004

def emit_error(reason,exit_code=2):
    run_app004.emit(run_app004.failure(exit_code,reason,"NOT_ATTEMPTED"))
    return exit_code

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scores",type=int,nargs="+",required=True)
    parser.add_argument("--output-parent")
    parser.add_argument("--engine",help="real built NORDOI binary, else offline Cargo")
    opts=parser.parse_args(argv)
    scores=opts.scores
    if not 1<=len(scores)<=8 or any(x<0 or x>100 for x in scores):
        return emit_error("INVALID_BATCH_SCORE_OR_COUNT")
    parent=Path(opts.output_parent) if opts.output_parent else Path.home()/"Downloads"
    if parent.is_symlink() or not parent.is_dir():
        return emit_error("INVALID_OUTPUT_PARENT")
    if opts.engine:
        binary=Path(opts.engine).resolve(strict=False)
        if not binary.is_file():
            return emit_error("NATIVE_ENGINE_MISSING")
        cmd=[str(binary)]
    else:
        if shutil.which("cargo") is None:
            return emit_error("CARGO_NOT_INSTALLED")
        cmd=["cargo","run","--offline","--quiet","--bin","nordoi","--"]
    try:
        output=Path(tempfile.mkdtemp(prefix="NORDOI_APP005_",dir=parent))
        cmd+=["agent-batch-jsonl",str(SOURCE),"app.main",
              "--grant-output-dir",str(output)]
        request="".join(json.dumps(
            {"schema":"NORDOI_AGENT_REQUEST_V1","key_code":x},
            separators=(",",":"),ensure_ascii=True)+"\n" for x in scores)
        p=subprocess.run(
            cmd,cwd=REPO,input=request,capture_output=True,
            text=True,encoding="utf-8",errors="strict",timeout=180,
        )
        lines=p.stdout.splitlines()
        if p.returncode:
            # A failure may be after earlier successful writes. Do not
            # disguise a partial batch as an all-or-nothing operation.
            if not lines:
                return emit_error("NATIVE_BATCH_ERROR_RECORD_MISSING",1)
            parsed=[json.loads(line) for line in lines]
            if parsed[-1].get("status")!="ERROR":
                return emit_error("NATIVE_BATCH_ERROR_RECORD_INVALID",1)
            sys.stdout.buffer.write(p.stdout.encode("utf-8"))
            sys.stdout.buffer.flush()
            return p.returncode
        if len(lines)!=len(scores):
            return emit_error("BATCH_NATIVE_RECEIPT_COUNT_MISMATCH",1)
        for index,(score,line) in enumerate(zip(scores,lines),1):
            data=json.loads(line)
            run_app004.verify_actual_files(
                output/f"job-{index:04}",score,data,p.stderr)
        sys.stdout.buffer.write(p.stdout.encode("utf-8"))
        sys.stdout.buffer.flush()
        return 0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        print("APP005_NATIVE_PROOF_FAILURE="+str(error),file=sys.stderr)
        return emit_error("NATIVE_BATCH_OR_REAL_FILES_VERIFICATION_FAILED",1)

if __name__=="__main__":
    raise SystemExit(main())
