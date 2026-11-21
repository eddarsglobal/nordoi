#!/usr/bin/env python3
"""APP-004: agent-first JSON client for the real NORDOI Rust runtime.

Rust executes the imported NOI rule, selects the P2.7 arm and emits one JSON
object. Python never decides the branch or generates the JSON success data.
This adapter only bounds input, provides a fresh explicit grant directory and
checks the bytes of the actual files against the native JSON result.
"""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE_ROOT=REPO/"examples"/"apps"/"app003_integrated_quality"/"src"
SCHEMA="NORDOI_AGENT_BUNDLE_RESULT_V1"

def failure(code, reason, publication="UNKNOWN"):
    return {"schema":SCHEMA,"status":"ERROR","error_code":reason,
            "exit_code":code,"publication":publication}

def emit(data):
    sys.stdout.write(json.dumps(data,ensure_ascii=True,
                                separators=(",",":"),sort_keys=True)+"\n")

def exact_native_response(output):
    if not output.endswith("\n") or output.count("\n") != 1:
        raise ValueError("NATIVE_JSON_NOT_ONE_LINE")
    val=json.loads(output)
    if not isinstance(val,dict) or val.get("schema")!=SCHEMA:
        raise ValueError("NATIVE_JSON_SCHEMA_MISMATCH")
    return val

def verify_actual_files(out,score,protocol,native_stderr):
    if protocol.get("status")!="COMMITTED":
        raise ValueError("NATIVE_COMMIT_NOT_CONFIRMED")
    if protocol.get("key_code")!=score or protocol.get("entry_module")!="app.main":
        raise ValueError("NATIVE_INPUT_OR_MODULE_MISMATCH")
    if protocol.get("module_count")!=2:
        raise ValueError("NATIVE_MODULE_COUNT_WRONG")
    decision=protocol.get("decision")
    if decision not in (0,1) or type(decision) is not int:
        raise ValueError("NATIVE_DECISION_INVALID")
    if protocol.get("selected")!=("approved" if decision==1 else "rejected"):
        raise ValueError("NATIVE_SELECTED_ARM_INVALID")
    if protocol.get("bundle")!="quality-decision":
        raise ValueError("NATIVE_BUNDLE_NAME_INVALID")
    if protocol.get("publication")!="DIRECTORY_RENAME":
        raise ValueError("NATIVE_PUBLICATION_UNQUALIFIED")
    if protocol.get("partial_final_state")!="FORBIDDEN":
        raise ValueError("NATIVE_PARTIAL_STATE_NOT_FORBIDDEN")
    if protocol.get("crash_durability")!="UNCLAIMED":
        raise ValueError("NATIVE_CRASH_CLAIM_CHANGED")
    if protocol.get("authority")!={"mode":"EXPLICIT","grant_count":2,
            "ambient":"NONE","overwrite":"DENIED"}:
        raise ValueError("NATIVE_AUTHORITY_INVALID")
    for key,fmt in (("rule_receipt_hex",r"[a-f0-9]{40,}"),
                    ("bundle_receipt_sha256",r"[a-f0-9]{64}")):
        if re.fullmatch(fmt,str(protocol.get(key,""))) is None:
            raise ValueError("NATIVE_PROVENANCE_MISSING_"+key)
    if ("receipt-sha256="+protocol["bundle_receipt_sha256"]) not in native_stderr:
        raise ValueError("REAL_P27_RECEIPT_DISAGREES")
    files=protocol.get("files")
    if not isinstance(files,list) or len(files)!=2:
        raise ValueError("NATIVE_FILE_COUNT_INVALID")
    dest=out/"quality-decision"
    if not dest.is_dir() or dest.is_symlink():
        raise ValueError("NATIVE_REAL_BUNDLE_MISSING")
    expected_names={"decision.txt","report.txt"}
    if {x.name for x in dest.iterdir()}!=expected_names:
        raise ValueError("NATIVE_WRONG_FINAL_FILE_SET")
    observed=set()
    for file in files:
        if not isinstance(file,dict) or set(file)!={"name","bytes","content","proof_sha256"}:
            raise ValueError("NATIVE_FILE_REPORT_INVALID")
        name=file["name"]
        if name not in expected_names or name in observed:
            raise ValueError("NATIVE_FILE_TARGET_INVALID")
        observed.add(name)
        content=file["content"]
        if not isinstance(content,str) or type(file["bytes"]) is not int:
            raise ValueError("NATIVE_FILE_BYTES_INVALID")
        raw=content.encode("utf-8")
        if len(raw)!=file["bytes"]:
            raise ValueError("NATIVE_FILE_LENGTH_MISMATCH")
        if not re.fullmatch(r"[0-9a-f]{64}",str(file["proof_sha256"])):
            raise ValueError("NATIVE_FILE_PROOF_MISSING")
        final=dest/name
        if final.is_symlink() or not final.is_file() or final.read_bytes()!=raw:
            raise ValueError("REAL_NORDOI_OUTPUT_BYTES_MISMATCH")
    if observed!=expected_names:
        raise ValueError("NATIVE_FILE_NAMES_INCOMPLETE")

def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--score",type=int,required=True)
    p.add_argument("--output-parent",default=None)
    p.add_argument("--engine",default=None,
                   help="Already-built real NORDOI binary; else Cargo --offline")
    args=p.parse_args(argv)
    if not 0<=args.score<=100:
        emit(failure(2,"INVALID_SCORE_EXPECTED_0_TO_100","NOT_ATTEMPTED"))
        return 2
    parent=Path(args.output_parent) if args.output_parent else Path.home()/"Downloads"
    if not parent.is_dir() or parent.is_symlink():
        emit(failure(2,"OUTPUT_PARENT_NOT_EXISTING_REAL_DIRECTORY","NOT_ATTEMPTED"))
        return 2
    if args.engine:
        exe=Path(args.engine).resolve(strict=False)
        if not exe.is_file():
            emit(failure(2,"NATIVE_ENGINE_MISSING","NOT_ATTEMPTED"))
            return 2
        cmd=[str(exe)]
    else:
        if shutil.which("cargo") is None:
            emit(failure(2,"CARGO_NOT_INSTALLED","NOT_ATTEMPTED"))
            return 2
        cmd=["cargo","run","--offline","--quiet","--bin","nordoi","--"]
    try:
        destination=Path(tempfile.mkdtemp(prefix="NORDOI_APP004_",dir=parent))
        cmd+=["agent-bundle-json",str(SOURCE_ROOT),"app.main",str(args.score),
              "--grant-output-dir",str(destination)]
        outcome=subprocess.run(cmd,cwd=REPO,capture_output=True,text=True,timeout=180)
        native=exact_native_response(outcome.stdout)
        if outcome.returncode!=0:
            if native.get("status")!="ERROR":
                emit(failure(outcome.returncode,"NATIVE_ERROR_ENVELOPE_INVALID"))
                return 1
            sys.stdout.write(outcome.stdout)
            return outcome.returncode
        verify_actual_files(destination,args.score,native,outcome.stderr)
        sys.stdout.write(outcome.stdout)
        return 0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        emit(failure(1,"NATIVE_PROTOCOL_OR_REAL_FILE_VERIFICATION_FAILED"))
        print("APP004_VALIDATION_FAILURE="+str(error),file=sys.stderr)
        return 1

if __name__=="__main__":
    raise SystemExit(main())
