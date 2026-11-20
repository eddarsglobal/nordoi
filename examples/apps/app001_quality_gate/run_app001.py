#!/usr/bin/env python3
"""APP-001: run a real bounded NORDOI quality-gate P2.7 program.

This is a demonstration for noncritical 0..100 scores, NOT a production
quality/safety certification. Two-file source is evaluated by the NORDOI
binary; this Python program only handles input, fresh directory selection,
explicit host permission, and final byte-exact independent output checking.
"""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE=HERE/"quality_gate.noi"
THRESHOLD=60

def expected(score):
    passed="true" if score>=THRESHOLD else "false"
    rejected="false" if score>=THRESHOLD else "true"
    return {
      "decision.txt":f"approved={passed}, rejected={rejected}, score={score}\n",
      "report.txt":f"score={score}, next={score+1}, meets_threshold={passed}\n",
    }

def run(args):
    if not (0<=args.score<=100):
        print("APP001_DENIED=INVALID_SCORE_EXPECTED_0_TO_100",file=sys.stderr)
        return 2
    if args.output_parent is None:
        base=Path.home()/"Downloads"
        if not base.is_dir():base=Path(tempfile.gettempdir())
    else:
        base=Path(args.output_parent)
    if base.is_symlink() or not base.is_dir():
        print("APP001_DENIED=OUTPUT_PARENT_MUST_BE_EXISTING_REAL_DIRECTORY",file=sys.stderr)
        return 2
    base=base.resolve(strict=True)
    # Always create a fresh private directory: never write to old program
    # output or write into project source. User can select parent directory.
    new_dir=Path(tempfile.mkdtemp(prefix="NORDOI_APP001_",dir=base))
    if args.engine:
        exe=Path(args.engine).resolve(strict=True)
        if not exe.is_file():
            print("APP001_DENIED=ENGINE_NOT_A_FILE",file=sys.stderr)
            return 2
        command=[str(exe),"bundle-write",str(SOURCE),str(args.score),
                 "--grant-output-dir",str(new_dir)]
    else:
        if shutil.which("cargo") is None:
            print("APP001_DENIED=CARGO_UNAVAILABLE",file=sys.stderr)
            return 2
        command=["cargo","run","--offline","--quiet","--bin","nordoi","--",
                 "bundle-write",str(SOURCE),str(args.score),
                 "--grant-output-dir",str(new_dir)]
    p=subprocess.run(command,cwd=REPO,capture_output=True,text=True,timeout=180)
    if p.returncode!=0:
        print("APP001_ENGINE=FAILED",file=sys.stderr)
        print(p.stderr,file=sys.stderr)
        return 1
    # Never infer success from a mere exit 0. Require all receipt claims and
    # actual file bytes, including absence of unexpected final-bundle files.
    receipt=p.stderr
    if not all(token in receipt for token in (
         "status=COMMITTED","authority=EXPLICIT","grant-count=2",
         "ambient-authority=NONE","overwrite=DENIED",
         "partial-final-state=FORBIDDEN","crash-durability=UNCLAIMED")):
        print("APP001_RECEIPT=UNQUALIFIED",file=sys.stderr)
        return 1
    dest=new_dir/"quality-gate"
    exp=expected(args.score)
    if (not dest.is_dir() or dest.is_symlink()
        or {x.name for x in dest.iterdir()}!=set(exp)):
        print("APP001_OUTPUT=INVALID_FILE_SET",file=sys.stderr)
        return 1
    for name,value in exp.items():
        target=dest/name
        if target.is_symlink() or not target.is_file() or (
             target.read_bytes()!=value.encode("utf-8")):
            print("APP001_OUTPUT=INCORRECT_FILE_CONTENT:"+name,file=sys.stderr)
            return 1
    print("NORDOI_APP001=PASS")
    print("SCORE="+str(args.score))
    print("THRESHOLD="+str(THRESHOLD))
    print("APPROVED="+("true" if args.score>=THRESHOLD else "false"))
    print("NORDOI_REAL_BUNDLE="+str(dest))
    for name in ("decision.txt","report.txt"):
        print(name+"="+exp[name].rstrip("\n"))
    print("NORDOI_AUTHORITY=EXPLICIT_TWO_FILE_CAPABILITIES")
    print("NORDOI_CRASH_DURABILITY=UNCLAIMED")
    return 0

def main(argv=None):
    a=argparse.ArgumentParser(description=__doc__)
    a.add_argument("--score",required=True,type=int,
                   help="Bounded demonstration score, integer from 0 to 100")
    a.add_argument("--output-parent",
                   help="Existing output parent; creates a fresh unique subdirectory")
    a.add_argument("--engine",help="Optional already-built nordoi executable")
    return run(a.parse_args(argv))

if __name__=="__main__":
    try:raise SystemExit(main())
    except (OSError,subprocess.SubprocessError,ValueError) as e:
        print("APP001_DENIED=EXECUTION_OR_FILESYSTEM_ERROR",file=sys.stderr)
        raise SystemExit(1)
