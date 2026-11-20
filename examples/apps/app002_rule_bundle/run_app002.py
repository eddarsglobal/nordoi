#!/usr/bin/env python3
"""APP-002: launch native NORDOI V1.3-module -> P2.7 bridge, verify real files.

The Rust engine itself, not Python, executes imported .noi rules and
selects a publication .noi branch. Python chooses only a unique destination,
provides explicit authorization, and independently checks returned evidence.
"""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
MODULE_ROOT=HERE/"rules"/"src"
APPROVED=HERE/"approved.noi"
REJECTED=HERE/"rejected.noi"
THRESHOLD=60

def expected(score):
    accepted=score>=THRESHOLD
    case="approved" if accepted else "rejected"
    return {
      "decision.txt":f"approved={str(accepted).lower()}, score={score}, next={score+1}\n",
      "report.txt":f"module-decision={case}, input={score}, next={score+1}\n",
    }

def run(args):
    if args.score<0 or args.score>100:
        print("APP002_DENIED=INVALID_SCORE_EXPECTED_0_TO_100",file=sys.stderr)
        return 2
    parent=Path(args.output_parent) if args.output_parent else Path.home()/"Downloads"
    if not parent.is_dir() or parent.is_symlink():
        print("APP002_DENIED=OUTPUT_PARENT_NOT_A_REAL_DIRECTORY",file=sys.stderr)
        return 2
    parent=parent.resolve(strict=True)
    if args.engine:
        exe=Path(args.engine).resolve(strict=True)
        if not exe.is_file():
            print("APP002_DENIED=ENGINE_NOT_A_FILE",file=sys.stderr)
            return 2
        command=[str(exe)]
    else:
        if shutil.which("cargo") is None:
            print("APP002_DENIED=CARGO_NOT_FOUND",file=sys.stderr)
            return 2
        command=["cargo","run","--offline","--quiet","--bin","nordoi","--"]
    destination=Path(tempfile.mkdtemp(prefix="NORDOI_APP002_",dir=parent))
    command+=["module-bundle-write",str(MODULE_ROOT),"app.main",
             str(APPROVED),str(REJECTED),str(args.score),
             "--grant-output-dir",str(destination)]
    p=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=180)
    if p.returncode:
        print("APP002_REAL_ENGINE=FAIL",file=sys.stderr)
        print(p.stdout,file=sys.stderr)
        print(p.stderr,file=sys.stderr)
        return 1
    selected="approved" if args.score>=THRESHOLD else "rejected"
    claims=[
       "module-bundle-write","modules=2",f"original-key-code={args.score}",
       f"rule-result=INT({1 if selected=='approved' else 0})",
       f'selected="{selected}"', "bridge=V13_RULE_SELECTS_P27_BRANCH",
       "status=COMMITTED","authority=EXPLICIT","grant-count=2",
       "ambient-authority=NONE","rule-receipt-hex=",
       "bundle-receipt-sha256=","crash-durability=UNCLAIMED",
    ]
    output_claims=["status=COMMITTED","authority=EXPLICIT","grant-count=2",
       "ambient-authority=NONE","overwrite=DENIED",
       "partial-final-state=FORBIDDEN","publication=DIRECTORY-RENAME",
       "crash-durability=UNCLAIMED"]
    if not all(x in p.stdout for x in claims) or not all(
        x in p.stderr for x in output_claims):
        print("APP002_REAL_NATIVE_BRIDGE_RECEIPTS_INCOMPLETE",file=sys.stderr)
        return 1
    final=destination/"quality-decision"
    if not final.is_dir() or final.is_symlink() or {
        x.name for x in final.iterdir()}!=set(expected(args.score)):
        print("APP002_FILES_MISSING_OR_EXTRA",file=sys.stderr)
        return 1
    for name,value in expected(args.score).items():
        path=final/name
        if path.is_symlink() or not path.is_file() or (
              path.read_bytes()!=value.encode("utf-8")):
            print("APP002_INVALID_REAL_FILE:"+name,file=sys.stderr)
            return 1
    print("NORDOI_APP002=PASS")
    print("SCORE="+str(args.score))
    print("RULE_MODULES=2")
    print("RULE_RESULT="+("1" if selected=="approved" else "0"))
    print("SELECTED_BRANCH="+selected)
    print("NORDOI_REAL_BUNDLE="+str(final))
    for name,value in expected(args.score).items():
        print(name+"="+value.rstrip("\n"))
    print("NORDOI_APP002_BRIDGE=V13_RULE_SELECTS_P27_BRANCH")
    print("NORDOI_AUTHORITY=EXPLICIT_TWO_EXACT_FILE_CAPABILITIES")
    print("CRASH_DURABILITY=UNCLAIMED")
    return 0

def main():
    a=argparse.ArgumentParser(description=__doc__)
    a.add_argument("--score",type=int,required=True)
    a.add_argument("--output-parent",default=None)
    a.add_argument("--engine",default=None,
        help="Optional already-built real NORDOI Rust binary")
    args=a.parse_args()
    try:return run(args)
    except (OSError,ValueError,subprocess.SubprocessError):
        print("APP002_DENIED=EXECUTION_OR_FILE_ERROR",file=sys.stderr)
        return 1

if __name__=="__main__":
    raise SystemExit(main())
