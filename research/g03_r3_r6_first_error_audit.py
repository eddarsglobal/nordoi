#!/usr/bin/env python3
"""NORDOI G0.3 R3-R6: first-use diagnostic read-only audit, NOT a novice study."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile

REMOTE = "https://github.com/eddarsglobal/nordoi.git"
MAIN = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
PROD = "e2898bf7c511e53636ff28c9fd3efc93165a0215"
PINS = {
    "src/bin/nordoi.rs": "e126a3f8ffe5a45ac33c23323ef3de2ccdfbc35d",
    "examples/p2_1_console/hello.noi": "e1b74cdabb63716744832c518e9e81e41aa3f14b",
    "docs/NORDOI_G02_FIRST_USE_QUICKSTART.md": "0aa6b2af0d67c55b9d70d89d9b8b0fb5d970646e",
}
HELLO = b'module app.main;\n\neffect ConsoleWrite;\n\nentry main emits "Hello from NORDOI P2.1!\\n";\n'
UNDECLARED = b'module app.main; entry main emits "NO";'
MISSPELLED = b'module app.main; effect ConsoleWrite; entry main emtis "NO";'
EXPECTED = b"Hello from NORDOI P2.1!\n"
MISSING = "NORDOI_R3_R6_MISSING_DIRECTORY/hello.noi"

CASES = [
    ("valid_run", ["run","-","--grant-console"], HELLO, 0, None, None),
    ("missing_grant_console_run", ["console-run","-"], HELLO, 5, "console-run", "capability denied"),
    ("undeclared_effect", ["run","-","--grant-console"], UNDECLARED, 4, "console-run", "requires explicit"),
    ("misspelled_emits", ["run","-","--grant-console"], MISSPELLED, 4, "console-run", "expected contextual 'emits'"),
    ("invalid_option", ["run","-","--unknown-option"], None, 2, "usage", "Usage:"),
    ("missing_argument", ["run"], None, 2, "usage", "Usage:"),
    ("unknown_command", ["rnu","-"], None, 2, "usage", "unknown command"),
    ("missing_source_file", ["run",MISSING,"--grant-console"], None, 3, "io", None),
    ("invalid_utf8", ["run","-","--grant-console"], b"\xff\xfe", 3, "input", "not valid UTF-8"),
    ("legacy_plain_run", ["run","-"], HELLO, None, None, None),
]

SAFE_ACTIONS = {
    "valid_run": "None; receipt on stderr is separate from program stdout.",
    "missing_grant_console_run": "Explain explicit grant; never grant automatically.",
    "undeclared_effect": "Explain source declaration is different from a CLI grant.",
    "misspelled_emits": "Correct source spelling only; do not expand permissions.",
    "invalid_option": "Check the supported options shown in help.",
    "missing_argument": "Supply a path or '-' for stdin.",
    "unknown_command": "Check the command spelling.",
    "missing_source_file": "Check the file path relative to the working directory.",
    "invalid_utf8": "Use UTF-8 encoded source.",
    "legacy_plain_run": "Explain the old HALT-only route; do not conflate with denied console-run.",
}

def blob(data):
    return hashlib.sha1(b"blob "+str(len(data)).encode()+b"\0"+data).hexdigest()

def git(*args):
    env = dict(os.environ, GIT_TERMINAL_PROMPT="0")
    return subprocess.check_output(["git",*args],env=env,stderr=subprocess.PIPE,timeout=120).decode().strip()

def selftest():
    assert len(HELLO)==86 and blob(HELLO)==PINS["examples/p2_1_console/hello.noi"]
    assert len(CASES)==10 and len(set(x[0] for x in CASES))==10
    assert set(SAFE_ACTIONS)=={x[0] for x in CASES}
    assert [x[3] for x in CASES].count(2)==3
    print("G03_R3_R6_SELFTEST=PASS",flush=True)

def check():
    for ref, pin in (("main",MAIN),("production/p0.1",PROD)):
        s=git("ls-remote","--exit-code",REMOTE,"refs/heads/"+ref)
        if not s or s.split()[0]!=pin:
            raise RuntimeError("REMOTE_DRIFT_"+ref)
    print("G03_R3_R6_REMOTE_SOURCE_AND_PRODUCTION=PASS",flush=True)

def execute(binary,repo,env,args,data):
    p=subprocess.run([str(binary)]+args,cwd=repo,env=env,input=data,
                     stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=40)
    if len(p.stdout)>100000 or len(p.stderr)>100000:
        raise RuntimeError("ABNORMAL_OUTPUT_SIZE")
    return p

def audit(output):
    if output.is_symlink() or output.exists() or output.parent.is_symlink() or not output.parent.is_dir():
        raise RuntimeError("OUTPUT_EXISTS_OR_PARENT_INVALID")
    check()
    with tempfile.TemporaryDirectory(prefix="nordoi-g03-r3-r6-") as tmp:
        root=Path(tmp)
        repo=root/"repo"
        git("clone","--quiet","--depth","1","--single-branch","--branch","main",REMOTE,str(repo))
        if git("-C",str(repo),"rev-parse","HEAD")!=MAIN:
            raise RuntimeError("CLONED_COMMIT_DRIFT")
        for path,pin in PINS.items():
            p=repo/path
            if not p.is_file() or p.is_symlink() or blob(p.read_bytes())!=pin:
                raise RuntimeError("SOURCE_BLOB_DRIFT_"+path)
        print("G03_R3_R6_PINNED_SOURCE_BLOBS=3/3_PASS",flush=True)
        env=dict(os.environ,CARGO_NET_OFFLINE="true",CARGO_INCREMENTAL="0",
                 CARGO_TARGET_DIR=str(root/"target"))
        build=subprocess.run(["cargo","build","--offline","--release","--bin","nordoi"],
                             cwd=repo,env=env,capture_output=True,timeout=900)
        if build.returncode:
            print(build.stderr.decode(errors="replace")[-2500:],flush=True)
            raise RuntimeError("ISOLATED_RELEASE_BUILD_FAILED")
        binary=root/"target"/"release"/("nordoi.exe" if os.name=="nt" else "nordoi")
        if not binary.is_file() or binary.is_symlink():
            raise RuntimeError("COMPILED_BINARY_MISSING")
        print("G03_R3_R6_RELEASE_BUILD=PASS",flush=True)
        events=[]
        for name,args,data,expected_code,expected_stage,needle in CASES:
            p=execute(binary,repo,env,args,data)
            stderr=p.stderr.decode("utf-8",errors="replace")
            stdout=p.stdout.decode("utf-8",errors="replace")
            stage_match=re.search(r"^error\[([^\]]+)\]",stderr,re.M)
            stage=stage_match.group(1) if stage_match else None
            if name=="valid_run":
                if p.returncode!=0 or p.stdout!=EXPECTED or "authority=EXPLICIT" not in stderr or "ambient-authority=NONE" not in stderr:
                    raise RuntimeError("SUCCESS_OUTPUT_OR_AUTHORITY_CONTRACT_DRIFT")
            elif name=="legacy_plain_run":
                if p.returncode==0 or p.stdout or "capability denied" in stderr:
                    raise RuntimeError("LEGACY_RUN_CONTRACT_DRIFT")
            else:
                if p.returncode!=expected_code or p.stdout or stage!=expected_stage or (needle and needle not in stderr):
                    raise RuntimeError("ERROR_CONTRACT_DRIFT_"+name)
            events.append({"id":name,"args":args,"exit_code":p.returncode,
                           "error_stage":stage,"stderr_text":stderr,"stdout_text":stdout,
                           "stderr_bytes":len(p.stderr),"stdout_bytes":len(p.stdout),
                           "safe_action_from_research":SAFE_ACTIONS[name]})
            print("G03_R3_R6_CASE_"+name.upper()+"=PASS",flush=True)
        result={"schema":"NORDOI-G03-R3-R6-FIRST-ERROR-AUDIT-v1",
                "source_sha":MAIN,"production_sha":PROD,
                "host_system":platform.system(),"host_arch":platform.machine(),
                "case_count":len(events),"passed":len(events),
                "human_comprehension":"NOT_MEASURED",
                "diagnostic_rewrites":"NONE",
                "source_modifications":"NONE","github_push":"NONE",
                "production":"NOT_AUTHORIZED","events":events}
        fd=os.open(str(output),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,"wb") as f:
            f.write((json.dumps(result,indent=2,ensure_ascii=False,sort_keys=True)+"\n").encode())
        print("G03_R3_R6_CASES=10/10_PASS",flush=True)
        print("G03_R3_R6_REPORT="+str(output),flush=True)
        print("PRODUCTION=NOT_AUTHORIZED",flush=True)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode",choices=["selftest","check","run"])
    p.add_argument("--output",type=Path)
    args=p.parse_args()
    selftest()
    if args.mode=="selftest": return
    if args.mode=="check":return check()
    if args.output is None:p.error("run requires --output")
    audit(args.output)

if __name__=="__main__":
    try:main()
    except (AssertionError,OSError,RuntimeError,subprocess.SubprocessError) as e:
        print("G03_R3_R6_ERROR="+str(e),flush=True)
        raise SystemExit(1)
