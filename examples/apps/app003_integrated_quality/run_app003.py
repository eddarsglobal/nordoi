#!/usr/bin/env python3
"""APP-003 launcher: one real integrated NOI entry imports rules and writes files.

The native Rust NORDOI binary tokenizes and compiles the single source
program. Python does not select branches or generate NOI source. It only
bounds user input, creates an isolated output directory, and independently
checks genuine files and native receipts after execution.
"""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
MODULE_ROOT=HERE/"src"
THRESHOLD=60

def expected(score):
    accepted=score>=THRESHOLD
    case="approved" if accepted else "rejected"
    return {
      "decision.txt":f"approved={str(accepted).lower()}, score={score}, next={score+1}\n",
      "report.txt":f"module-decision={case}, input={score}, next={score+1}\n",
    }

def run(args):
    if not 0<=args.score<=100:
        print("APP003_DENIED=INVALID_SCORE_EXPECTED_0_TO_100",file=sys.stderr)
        return 2
    parent=Path(args.output_parent) if args.output_parent else Path.home()/"Downloads"
    if parent.is_symlink() or not parent.is_dir():
        print("APP003_DENIED=OUTPUT_PARENT_NOT_EXISTING_REAL_DIRECTORY",file=sys.stderr)
        return 2
    parent=parent.resolve(strict=True)
    if args.engine:
        binary=Path(args.engine).resolve(strict=True)
        if not binary.is_file():
            print("APP003_DENIED=INVALID_NATIVE_BINARY",file=sys.stderr)
            return 2
        command=[str(binary)]
    else:
        if shutil.which("cargo") is None:
            print("APP003_DENIED=CARGO_UNAVAILABLE",file=sys.stderr)
            return 2
        command=["cargo","run","--offline","--quiet","--bin","nordoi","--"]
    out=Path(tempfile.mkdtemp(prefix="NORDOI_APP003_",dir=parent))
    command+=["integrated-bundle-run",str(MODULE_ROOT),"app.main",str(args.score),
              "--grant-output-dir",str(out)]
    native=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=180)
    if native.returncode:
        print("APP003_NATIVE_EXECUTION=FAIL",file=sys.stderr)
        print(native.stdout,file=sys.stderr)
        print(native.stderr,file=sys.stderr)
        return 1
    decision="approved" if args.score>=THRESHOLD else "rejected"
    claim=[
       "integrated-bundle-run","source-profile=APP003_SINGLE_NOI_IMPORTS_AND_EFFECTS",
       "modules=2",f"original-key-code={args.score}",
       f"rule-result=INT({1 if decision=='approved' else 0})",
       f'selected="{decision}"', "bridge=V13_PURE_RULE_TO_P27_EFFECT",
       "status=COMMITTED","authority=EXPLICIT","grant-count=2",
       "ambient-authority=NONE","rule-receipt-hex=",
       "bundle-receipt-sha256=","crash-durability=UNCLAIMED",
    ]
    receipt=[
       "status=COMMITTED","authority=EXPLICIT","grant-count=2",
       "ambient-authority=NONE","overwrite=DENIED",
       "partial-final-state=FORBIDDEN","publication=DIRECTORY-RENAME",
       "crash-durability=UNCLAIMED",
    ]
    if not all(x in native.stdout for x in claim) or not all(
        x in native.stderr for x in receipt):
        print("APP003_NATIVE_RECEIPTS=INCOMPLETE",file=sys.stderr)
        return 1
    bundle=out/"quality-decision"
    expected_files=expected(args.score)
    if (not bundle.is_dir() or bundle.is_symlink()
        or {p.name for p in bundle.iterdir()}!=set(expected_files)):
        print("APP003_OUTPUT=WRONG_FILE_SET",file=sys.stderr)
        return 1
    for name,body in expected_files.items():
        target=bundle/name
        if target.is_symlink() or not target.is_file() or (
            target.read_bytes()!=body.encode("utf-8")):
            print("APP003_OUTPUT=WRONG_BYTES:"+name,file=sys.stderr)
            return 1
    print("NORDOI_APP003=PASS")
    print("SOURCE_PROFILE=ONE_NOI_ENTRY_IMPORTS_PURE_RULE")
    print("SCORE="+str(args.score))
    print("RULE_MODULES=2")
    print("RULE_RESULT="+("1" if decision=="approved" else "0"))
    print("SELECTED_BRANCH="+decision)
    print("NORDOI_REAL_BUNDLE="+str(bundle))
    for name,body in expected_files.items():
        print(name+"="+body.rstrip("\n"))
    print("APP003_NATIVE_COMPILER=V13_TO_P27_SINGLE_ENTRY")
    print("APP003_AUTHORITY=EXPLICIT_TWO_FILE_CAPABILITIES")
    print("CRASH_DURABILITY=UNCLAIMED")
    return 0

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--score",type=int,required=True)
    p.add_argument("--output-parent",default=None)
    p.add_argument("--engine",default=None)
    args=p.parse_args()
    try:return run(args)
    except (OSError,ValueError,subprocess.SubprocessError):
        print("APP003_DENIED=EXECUTION_OR_OUTPUT_ERROR",file=sys.stderr)
        return 1

if __name__=="__main__":
    raise SystemExit(main())
