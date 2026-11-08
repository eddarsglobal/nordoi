#!/usr/bin/env python3
"""G0.3 R3-R17: pinned, function-symbol machine-code audit, engineering only.

The external Rust witness checks default denial and explicit grant. On Mac Intel,
nm and otool collect disassembly of its named non-inlined wrapper, and the
nordoi_kernel require symbol when independently visible. This is NOT proof of
cause for fractional-ns performance differences, and does not authorize merge.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import types

BASE_SHA = "98da03b2ea1efcf665d205189284aa8b66fda5d3"
CAND_SHA = "a1ae15326eaa837f8994a75fa7cfbe2eccc44976"
BRANCH = "candidate/g03-r3-r13-eliminate-redundant-atom-check-20261108"
R13_BLOB = "dac0109959e10a15764ff54a4da91bf190deff59"
R17_RUST_BLOB = "599f2353777e86de4a85ce6ba4e1cbc951a4249f"
SCHEMA = "NORDOI-G03-R3-R17-FIX1-FUNCTION-CODE-AUDIT-v1"
WITNESS = "nordoi_r17_require_witness"
RECEIPT = b"G03_R3_R17_REQUIRE_WITNESS=PASS\nPRODUCTION=NOT_AUTHORIZED\n"
BASE_KERNEL_SHA256 = "dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2"

class Blocked(Exception):
    pass

def git_blob(b):
    return hashlib.sha1(b"blob "+str(len(b)).encode("ascii")+b"\0"+b).hexdigest()

def h256(b):
    return hashlib.sha256(b).hexdigest()

def check_pinned(path, expected_blob, max_size=110000):
    p=Path(path)
    if p.is_symlink() or not p.is_file() or not 0 < p.stat().st_size <= max_size:
        raise Blocked("pinned tool missing/unsafe")
    b=p.read_bytes()
    if git_blob(b)!=expected_blob:
        raise Blocked("pinned Git blob mismatch")
    return b

def load_r13(path):
    data=check_pinned(path,R13_BLOB)
    module=types.ModuleType("nordoi_r13_pinned")
    module.__file__=str(path)
    exec(compile(data,str(path),"exec"),module.__dict__)
    if (module.BASE_SHA!=BASE_SHA or module.BRANCH!=BRANCH
            or module.PROBE_SCHEMA!="NORDOI-G03-R3-R13-PROBE-v1"):
        raise Blocked("R13 contract drift")
    return module

def external_run(args, cwd=None, env=None, timeout=900):
    try:
        p=subprocess.run(args,cwd=cwd,env=env,timeout=timeout,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    except (OSError,subprocess.TimeoutExpired) as e:
        raise Blocked("subprocess unavailable or timed out") from e
    if p.returncode:
        raise Blocked("subprocess rejected: "+str(args[0]))
    return p.stdout

def build_witness(source, work, rust_source):
    target=work/"target"
    env=dict(os.environ,CARGO_NET_OFFLINE="true",CARGO_TARGET_DIR=str(target))
    external_run(["cargo","build","--offline","--release","--lib"],cwd=source,env=env)
    deps=target/"release"/"deps"
    lib=list(deps.glob("libnordoi_kernel-*.rlib"))
    if len(lib)!=1:
        raise Blocked("ambiguous linked Rust release library")
    exe=work/("witness.exe" if os.name=="nt" else "witness")
    external_run(["rustc","--edition=2021","-C","opt-level=3",
        "-C","codegen-units=1","-L","dependency="+str(deps),
        "--extern","nordoi_kernel="+str(lib[0]),
        str(rust_source),"-o",str(exe)])
    result=external_run([str(exe)],timeout=45)
    if result!=RECEIPT:
        raise Blocked("functional witness did not print exact PASS")
    return exe

NM_LINE = re.compile(r"^\s*([0-9a-fA-F]{8,16})\s+([tT])\s+(\S+)\s*$")
ASM_LINE = re.compile(r"^\s*([0-9a-fA-F]{8,16})\s+(.+?)\s*$")

def symbols_from_nm(decoded):
    found=[]
    for line in decoded.splitlines():
        m=NM_LINE.match(line)
        if m:
            found.append({"address":int(m.group(1),16),"name":m.group(3)})
    if not found:
        raise Blocked("no Mach-O text symbols")
    return sorted(found,key=lambda x:x["address"])

def disassembly_for_symbol(symbols, otool, symbol, limit=140):
    matches=[x for x in symbols if x["name"]==symbol]
    if len(matches)!=1:
        raise Blocked("witness symbol not unique or missing")
    start=matches[0]["address"]
    later=sorted({x["address"] for x in symbols if x["address"]>start})
    end=later[0] if later else start+2048
    if end <= start or end-start>32768:
        raise Blocked("symbol span invalid")
    instructions=[]
    for line in otool.splitlines():
        m=ASM_LINE.match(line)
        if not m:
            continue
        address=int(m.group(1),16)
        if start <= address < end:
            instructions.append({"address":"0x"+format(address,"x"),
                                 "asm":m.group(2)[:250]})
            if len(instructions)>limit:
                raise Blocked("symbol instruction count exceeds expected bounds")
    if not instructions:
        raise Blocked("symbol not disassembled")
    normalized="\n".join(x["asm"] for x in instructions)
    return {"name":symbol,"start_hex":hex(start),"size_upper_bound_bytes":end-start,
        "instruction_lines":len(instructions),"asm_sha256":h256(normalized.encode()),
        "disassembly":instructions}

def inspect_macho(exe):
    if platform.system()!="Darwin" or platform.machine()!="x86_64":
        return {"status":"NOT_SUPPORTED_OUTSIDE_MAC_INTEL"}
    nm=shutil.which("nm")
    otool=shutil.which("otool")
    if not nm or not otool:
        raise Blocked("nm/otool required for exact Mac audit")
    symbol_text=external_run([nm,"-an",str(exe)],timeout=100).decode("utf-8","replace")
    sym=symbols_from_nm(symbol_text)
    witness_names=[x["name"] for x in sym
        if x["name"].lstrip("_")==WITNESS]
    if len(witness_names)!=1:
        raise Blocked("named no-inline witness missing")
    asm=external_run([otool,"-tvV",str(exe)],timeout=150)
    if len(asm)>30_000_000:
        raise Blocked("otool output too large")
    asm_text=asm.decode("utf-8","replace")
    witness=disassembly_for_symbol(sym,asm_text,witness_names[0])
    candidates=[x["name"] for x in sym
        if ("require" in x["name"].lower() and
            ("nordoi_kernel" in x["name"].lower() or "AtomicKernel" in x["name"]))]
    # Candidate symbol existence and symbol-level disassembly are evidence;
    # inlining and Rust mangling may prevent a direct require function listing.
    selected=[]
    for name in sorted(set(candidates))[:8]:
        try:
            selected.append(disassembly_for_symbol(sym,asm_text,name,limit=300))
        except Blocked:
            continue
    return {"status":"MACHO_X86_64_INSPECTED",
        "witness":witness,"require_related_symbols":selected,
        "require_symbol_status":"EXTRACTED" if selected else "NOT_LINKER_VISIBLE_OR_INLINED",
        "nm_text_symbol_count":len(sym),
        "whole_otool_dump_sha256":h256(asm),
        "attribution":"CODE_LAYOUT_CAUSALITY_NOT_PROVEN"}

def verify_source(runner,base,cand):
    guard=runner.verify_delta(base/"src"/"kernel.rs",cand/"src"/"kernel.rs")
    allowed={"src/kernel.rs","research/g03_r3_r13_kernel_probe.rs",
         "research/g03_r3_r13_paired_mac.py",
         "research/G03_R3_R13_ENGINEERING.md",
         "tests/g03_r3_r13_redundant_lookup.rs",
         ".github/workflows/g03_r3_r13_paired.yml"}
    old=set(runner.run(["git","ls-files"],cwd=base).decode().splitlines())
    new=set(runner.run(["git","ls-files"],cwd=cand).decode().splitlines())
    if not (old^new).issubset(allowed):
        raise Blocked("unexpected tracked-file inventory")
    for file in old&new-{"src/kernel.rs"}:
        if (base/file).read_bytes()!=(cand/file).read_bytes():
            raise Blocked("unrelated tracked source change")
    return guard

def build_dir(root,label):
    # FIX1: never reuse the checked-out "candidate" source directory as
    # an output/build directory for the candidate witness binary.
    if label not in ("baseline","candidate"):
        raise Blocked("invalid build role")
    root=Path(root)
    return root/"builds"/label

def private_output(path, report):
    p=Path(path)
    if p.exists() or p.is_symlink() or not p.parent.is_dir():
        raise Blocked("output path unsafe or already exists")
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL
    if hasattr(os,"O_NOFOLLOW"):
        flags|=os.O_NOFOLLOW
    fd=os.open(p,flags,0o600)
    with os.fdopen(fd,"w",encoding="utf-8") as f:
        json.dump(report,f,sort_keys=True,indent=2)
        f.write("\n")
    if os.name=="posix" and p.stat().st_mode & 0o077:
        raise Blocked("private output permissions unsafe")

def selftest():
    assert git_blob(b"123")=="d800886d9c86731ae5c4a62b0b77c437015e00d2"
    synthetic_nm="\n".join([
        "0000000100001000 T _nordoi_r17_require_witness",
        "0000000100001010 T _next_function",
    ])
    sample=symbols_from_nm(synthetic_nm)
    synthetic_asm="\n".join([
        "0000000100001000 pushq %rbp",
        "0000000100001004 callq 0x100001012",
        "000000010000100a popq %rbp",
        "0000000100001010 retq",
    ])
    witness=disassembly_for_symbol(sample,synthetic_asm,
        "_nordoi_r17_require_witness")
    assert witness["instruction_lines"]==3
    assert witness["size_upper_bound_bytes"]==16
    try:
        disassembly_for_symbol(sample,synthetic_asm,"_invalid")
    except Blocked:
        pass
    else:
        raise AssertionError("unknown symbol passed")
    try:
        disassembly_for_symbol(sample,"garbage","_nordoi_r17_require_witness")
    except Blocked:
        pass
    else:
        raise AssertionError("missing instructions passed")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R317_FIX1_DIRTEST_") as tmp:
        root=Path(tmp)
        (root/"candidate").mkdir()
        (root/"base").mkdir()
        b=build_dir(root,"baseline")
        c=build_dir(root,"candidate")
        assert b!=c and b!=root/"base" and c!=root/"candidate"
        assert c.parent==b.parent==root/"builds"
        b.parent.mkdir()
        b.mkdir()
        c.mkdir()
        assert (root/"candidate").is_dir()
        try:
            build_dir(root,"unknown")
        except Blocked:
            pass
        else:
            raise AssertionError("unexpected build role accepted")
    print("G03_R3_R17_FIX1_BUILD_DIRECTORY_SELFTEST=PASS")
    print("G03_R3_R17_SELFTEST=SYNTHETIC_PASS")
    print("REAL_MAC_MACHINE_CODE=NOT_INSPECTED_IN_SELFTEST")
    print("PRODUCTION=NOT_AUTHORIZED")

def smoke(source):
    source=Path(source).resolve()
    git_kernel=external_run(["git","show","HEAD:src/kernel.rs"],cwd=source,timeout=30)
    if h256(git_kernel)!=BASE_KERNEL_SHA256:
        raise Blocked("CI main kernel source not pinned")
    rust=external_run(["git","show","HEAD:research/g03_r3_r17_require_witness.rs"],
        cwd=source,timeout=30)
    if git_blob(rust)!=R17_RUST_BLOB:
        raise Blocked("CI witness source not pinned")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R317_CI_") as tmp:
        temp=Path(tmp)
        witness=temp/"witness.rs"
        witness.write_bytes(rust)
        exe=build_witness(source,temp,witness)
        if not exe.is_file():
            raise Blocked("CI witness missing")
    print("G03_R3_R17_CI_RUST_SECURITY_SMOKE=PASS")
    print("REAL_MAC_DISASSEMBLY=NOT_RUN_IN_CI")
    print("PRODUCTION=NOT_AUTHORIZED")

def measure(r13tool,rust_source,out):
    if platform.system()!="Darwin" or platform.machine()!="x86_64":
        raise Blocked("Mac Intel required for machine-code disassembly")
    if any(shutil.which(x) is None for x in ("git","cargo","rustc","nm","otool")):
        raise Blocked("required toolchain missing")
    runner=load_r13(r13tool)
    rust=check_pinned(rust_source,R17_RUST_BLOB,max_size=12000)
    with tempfile.TemporaryDirectory(prefix="NORDOI_R317_ISOLATED_") as tmp:
        root=Path(tmp)
        base=root/"base"; cand=root/"candidate"
        runner.clone(base,"main",BASE_SHA)
        runner.clone(cand,BRANCH,CAND_SHA)
        guard=verify_source(runner,base,cand)
        results={}
        (root/"builds").mkdir()
        for label,source in (("baseline",base),("candidate",cand)):
            work=build_dir(root,label)
            work.mkdir()
            pinned=work/"r17_witness.rs"
            pinned.write_bytes(rust)
            exe=build_witness(source,work,pinned)
            results[label]={
                "exe_sha256":h256(exe.read_bytes()),
                "exe_bytes":exe.stat().st_size,
                "code":inspect_macho(exe),
                "security_default_deny_and_grant":"PASS"
            }
        a=results["baseline"]["code"]
        b=results["candidate"]["code"]
        witness_equal=(a["witness"]["asm_sha256"]==b["witness"]["asm_sha256"])
        report={
            "schema":SCHEMA,"baseline_source_sha":BASE_SHA,
            "candidate_source_sha":CAND_SHA,"r13_git_blob":R13_BLOB,
            "witness_git_blob":R17_RUST_BLOB,"source_guard":guard,
            "compiler":{"rustc":external_run(["rustc","--version"]).decode().strip(),
                "cargo":external_run(["cargo","--version"]).decode().strip()},
            "host":{"system":platform.system(),"machine":platform.machine(),
                "cpu_count":os.cpu_count(),"python":platform.python_version()},
            "builds":results,
            "witness_asm_equal":witness_equal,
            "witness_asm_comparison":"TEXTUAL_ASM_WITHOUT_CAUSAL_ATTRIBUTION",
            "require_machine_code_equivalence":"NOT_ESTABLISHED_WITHOUT_FUNCTION_BOUNDARY_PROOF",
            "denied_capability_slowdown_cause":"NOT_ESTABLISHED",
            "measurement":"DISASSEMBLY_ONLY_NO_NEW_BENCHMARK",
            "security": "DEFAULT_DENY_EXPLICIT_GRANT_FRESH_KERNEL_PASS_BOTH",
            "merge_authorization":"NONE","production":"NOT_AUTHORIZED",
            "human_evaluation_gate":"CLOSED_NOT_REQUIRED",
            "original_repo_modified":False
        }
        private_output(out,report)
    print("G03_R3_R17_FIX1_BUILD_DIRECTORIES=ISOLATED")
    print("G03_R3_R17_MAC_MACHINE_CODE_AUDIT=PASS")
    print("G03_R3_R17_REQUIRE_SECURITY=2_OF_2_PASS")
    print("SOURCE_GUARD=PASS")
    print("WRAPPER_ASM_IDENTICAL="+str(witness_equal).upper())
    print("BASE_REQUIRE_SYMBOL="+a["require_symbol_status"])
    print("CANDIDATE_REQUIRE_SYMBOL="+b["require_symbol_status"])
    print("DENIED_CAPABILITY_ROOT_CAUSE=NOT_ESTABLISHED")
    print("MERGE_AUTHORIZATION=NONE")
    print("REPORT_PRIVATE="+str(out))
    print("PRODUCTION=NOT_AUTHORIZED")

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    sub=parser.add_subparsers(dest="action",required=True)
    sub.add_parser("selftest")
    sm=sub.add_parser("smoke");sm.add_argument("--source",required=True)
    m=sub.add_parser("measure")
    m.add_argument("--r13-tool",required=True)
    m.add_argument("--rust-witness",required=True)
    m.add_argument("--out",required=True)
    args=parser.parse_args()
    try:
        if args.action=="selftest": selftest()
        elif args.action=="smoke":smoke(args.source)
        else:measure(args.r13_tool,args.rust_witness,args.out)
        return 0
    except (Blocked,OSError,ValueError,UnicodeError,TypeError,KeyError,
            AssertionError,subprocess.TimeoutExpired) as e:
        print("G03_R3_R17_GATE=BLOCKED",file=sys.stderr)
        print("REASON="+str(e)[:160],file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
