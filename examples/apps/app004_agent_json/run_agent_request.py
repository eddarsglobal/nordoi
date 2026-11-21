#!/usr/bin/env python3
"""APP-004-R2 agent request client: machine JSON stdin -> genuine Rust NORDOI JSON stdout.

The native engine parses the request; the client never selects approved or
rejected. The source root and FileWrite authority remain separate CLI args.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE_ROOT=REPO/"examples/apps/app003_integrated_quality/src"
sys.path.insert(0,str(HERE))
import run_app004

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--score",type=int,required=True)
    parser.add_argument("--output-parent")
    parser.add_argument("--engine")
    options=parser.parse_args(argv)
    if not 0<=options.score<=100:
        run_app004.emit(run_app004.failure(2,"INVALID_SCORE_EXPECTED_0_TO_100","NOT_ATTEMPTED"))
        return 2
    parent=Path(options.output_parent) if options.output_parent else Path.home()/"Downloads"
    if not parent.is_dir() or parent.is_symlink():
        run_app004.emit(run_app004.failure(2,"OUTPUT_PARENT_NOT_EXISTING_REAL_DIRECTORY","NOT_ATTEMPTED"))
        return 2
    if options.engine:
        engine=Path(options.engine).resolve(strict=False)
        if not engine.is_file():
            run_app004.emit(run_app004.failure(2,"NATIVE_ENGINE_MISSING","NOT_ATTEMPTED"))
            return 2
        cmd=[str(engine)]
    else:
        if shutil.which("cargo") is None:
            run_app004.emit(run_app004.failure(2,"CARGO_NOT_INSTALLED","NOT_ATTEMPTED"))
            return 2
        cmd=["cargo","run","--offline","--quiet","--bin","nordoi","--"]
    try:
        out=Path(tempfile.mkdtemp(prefix="NORDOI_AGENT_REQUEST_",dir=parent))
        cmd+=["agent-request-json",str(SOURCE_ROOT),"app.main",
              "--grant-output-dir",str(out)]
        request={"schema":"NORDOI_AGENT_REQUEST_V1","key_code":options.score}
        p=subprocess.run(
            cmd,cwd=REPO,input=json.dumps(request,separators=(",",":")),
            capture_output=True,text=True,encoding="utf-8",errors="strict",
            timeout=180,
        )
        response=run_app004.exact_native_response(p.stdout)
        if p.returncode!=0:
            if response.get("status")!="ERROR":
                run_app004.emit(run_app004.failure(1,"NATIVE_ERROR_ENVELOPE_INVALID"))
                return 1
            sys.stdout.buffer.write(p.stdout.encode("utf-8"))
            sys.stdout.buffer.flush()
            return p.returncode
        run_app004.verify_actual_files(out,options.score,response,p.stderr)
        sys.stdout.buffer.write(p.stdout.encode("utf-8"))
        sys.stdout.buffer.flush()
        return 0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        run_app004.emit(run_app004.failure(1,"NATIVE_REQUEST_OR_FILES_VERIFICATION_FAILED"))
        print("APP004_R2_VALIDATION_FAILURE="+str(error),file=sys.stderr)
        return 1

if __name__=="__main__":
    raise SystemExit(main())
