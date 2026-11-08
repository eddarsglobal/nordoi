#!/usr/bin/env python3
"""NORDOI G0.3 R3-R18: pinned CapabilitySet::contains and call-layout audit.

Engineering-only function disassembly. No new speed measurements, merges,
kernel modifications, production permissions or external human study.
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
R13_BLOB = "dac0109959e10a15764ff54a4da91bf190deff59"
R17_BLOB = "713f50c217aa9cc2f0604e24089ed8749af07a08"
RUST_BLOB = "599f2353777e86de4a85ce6ba4e1cbc951a4249f"
BASE_KERNEL_SHA = "dcb0689dc86cd2646a5f4ffb3c8b0c5eaa72c59e0cdea1307aeba2e86f8de7e2"
SCHEMA = "NORDOI-G03-R3-R18-CONTAINS-CALL-LAYOUT-v1"
RECEIPT = b"G03_R3_R17_REQUIRE_WITNESS=PASS\nPRODUCTION=NOT_AUTHORIZED\n"
FUNCTIONS = ("witness", "require", "contains")
TARGET_BRANCH = re.compile(r"^(j[a-z]+)\s+(0x[0-9a-fA-F]+)$")

class Blocked(Exception):
    pass

def sha(b):
    return hashlib.sha256(b).hexdigest()

def blob(b):
    return hashlib.sha1(b"blob " + str(len(b)).encode("ascii") + b"\0" + b).hexdigest()

def pin(path, expected, limit=100000):
    p=Path(path)
    if p.is_symlink() or not p.is_file():
        raise Blocked("missing or linked pinned input")
    stat=p.stat()
    if stat.st_size<=0 or stat.st_size>limit or stat.st_nlink!=1:
        raise Blocked("pinned input file unsafe")
    payload=p.read_bytes()
    if blob(payload)!=expected:
        raise Blocked("pinned Git blob identity mismatch")
    return payload

def load_r17(path):
    payload=pin(path,R17_BLOB)
    mod=types.ModuleType("verified_nordoi_r317_fix1")
    mod.__file__=str(path)
    exec(compile(payload,str(path),"exec"),mod.__dict__)
    if (mod.BASE_SHA!=BASE_SHA or mod.CAND_SHA!=CAND_SHA or
            mod.R13_BLOB!=R13_BLOB or mod.R17_RUST_BLOB!=RUST_BLOB or
            mod.SCHEMA!="NORDOI-G03-R3-R17-FIX1-FUNCTION-CODE-AUDIT-v1"):
        raise Blocked("pinned R17 API or source contract changed")
    return mod

def canonical_assembly(function):
    start=int(function["start_hex"],16)
    upper=start+int(function["size_upper_bound_bytes"])
    result=[]
    for row in function["disassembly"]:
        asm=row["asm"]
        match=TARGET_BRANCH.fullmatch(asm.strip())
        if match:
            target=int(match.group(2),16)
            if start<=target<upper:
                asm=match.group(1)+"\tLOCAL+0x"+format(target-start,"x")
        result.append(asm)
    return result

def summarize_function(data):
    normalized=canonical_assembly(data)
    instructions=data["disassembly"]
    if not normalized or len(normalized)!=data["instruction_lines"]:
        raise Blocked("malformed function disassembly")
    begin=int(data["start_hex"],16)
    call_edges=[]
    for asm in normalized:
        if asm.startswith(("callq\t", "call\t")):
            call_edges.append(asm.split(None,1)[1])
    return {
        "symbol":data["name"],"address":data["start_hex"],
        "address_int":begin,
        "alignment_mod_16":begin%16,"alignment_mod_32":begin%32,
        "alignment_mod_64":begin%64,"alignment_mod_128":begin%128,
        "instruction_count":len(normalized),
        "size_upper_bound_bytes":data["size_upper_bound_bytes"],
        "raw_asm_sha256":data["asm_sha256"],
        "normalized_asm_sha256":sha("\n".join(normalized).encode()),
        "normalized_assembly":normalized,
        "raw_disassembly":instructions,
        "direct_calls":call_edges
    }

def named_symbols(symbols):
    def matches(name, kind):
        if kind=="witness":
            return name.lstrip("_")=="nordoi_r17_require_witness"
        if kind=="require":
            return "AtomicKernel7require" in name and "nordoi_kernel" in name
        if kind=="contains":
            return "CapabilitySet8contains" in name and "nordoi_kernel" in name
        return False
    selected={}
    for kind in FUNCTIONS:
        choices=[s["name"] for s in symbols if matches(s["name"],kind)]
        selected[kind]=choices[0] if len(choices)==1 else None
    return selected

def inspect_macho(exe,r17):
    text=r17.external_run(["nm","-an",str(exe)],timeout=100).decode("utf-8","replace")
    syms=r17.symbols_from_nm(text)
    chosen=named_symbols(syms)
    if chosen["witness"] is None or chosen["require"] is None:
        raise Blocked("unique witness/require function symbol missing")
    asm=r17.external_run(["otool","-tvV",str(exe)],timeout=150)
    if len(asm)>30_000_000:
        raise Blocked("Mach-O disassembly unexpectedly large")
    decoded=asm.decode("utf-8","replace")
    functions={}
    for kind, symbol in chosen.items():
        if symbol is None:
            functions[kind]={"status":"NOT_LINKER_VISIBLE_OR_INLINED"}
            continue
        try:
            raw=r17.disassembly_for_symbol(syms,decoded,symbol,limit=1200)
            functions[kind]={"status":"EXTRACTED",**summarize_function(raw)}
        except r17.Blocked:
            if kind in ("witness","require"):
                raise Blocked("required witness/require disassembly unavailable")
            functions[kind]={"status":"PRESENT_BUT_DISASSEMBLY_BLOCKED"}
    require=functions["require"]
    edges=require["direct_calls"]
    call_contains=any("CapabilitySet8contains" in edge for edge in edges)
    return {
        "symbol_names":chosen,
        "functions":functions,
        "require_calls_contains":call_contains,
        "nm_symbol_count":len(syms),
        "binary_sha256":sha(Path(exe).read_bytes()),
        "binary_bytes":Path(exe).stat().st_size
    }

def cross_compare(left,right):
    compared={}
    for kind in FUNCTIONS:
        a=left["functions"][kind]
        b=right["functions"][kind]
        if a["status"]!="EXTRACTED" or b["status"]!="EXTRACTED":
            compared[kind]={"status":"NOT_COMPARABLE",
               "baseline_status":a["status"],"candidate_status":b["status"]}
            continue
        identical=a["normalized_assembly"]==b["normalized_assembly"]
        differing=[i for i,(x,y) in enumerate(zip(
            a["normalized_assembly"],b["normalized_assembly"])) if x!=y]
        shift=b["address_int"]-a["address_int"]
        compared[kind]={
          "status":"COMPARABLE",
          "normalized_text_identical":identical,
          "instruction_count_baseline":a["instruction_count"],
          "instruction_count_candidate":b["instruction_count"],
          "first_different_instruction_indices":differing[:12],
          "candidate_minus_baseline_address_bytes":shift,
          "alignment_mod_16_changed":a["alignment_mod_16"]!=b["alignment_mod_16"],
          "alignment_mod_32_changed":a["alignment_mod_32"]!=b["alignment_mod_32"],
          "alignment_mod_64_changed":a["alignment_mod_64"]!=b["alignment_mod_64"],
          "baseline_direct_calls":a["direct_calls"],
          "candidate_direct_calls":b["direct_calls"]
        }
    return compared

def external_run(argv,cwd=None,env=None,timeout=900):
    try:
        p=subprocess.run(argv,cwd=cwd,env=env,timeout=timeout,
          stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    except (OSError,subprocess.TimeoutExpired) as exc:
        raise Blocked("subprocess unavailable or timeout") from exc
    if p.returncode:
        raise Blocked("subprocess failed "+str(argv[0]))
    return p.stdout

def security_build(source,work,rust_file):
    env=dict(os.environ,CARGO_NET_OFFLINE="true",CARGO_TARGET_DIR=str(work/"target"))
    external_run(["cargo","build","--offline","--release","--lib"],cwd=source,env=env)
    libs=list((work/"target"/"release"/"deps").glob("libnordoi_kernel-*.rlib"))
    if len(libs)!=1:
        raise Blocked("offline release library ambiguous")
    exe=work/("witness.exe" if os.name=="nt" else "witness")
    external_run(["rustc","--edition=2021","-C","opt-level=3","-C","codegen-units=1",
        "-L","dependency="+str(work/"target"/"release"/"deps"),
        "--extern","nordoi_kernel="+str(libs[0]),str(rust_file),"-o",str(exe)])
    if external_run([str(exe)],timeout=45)!=RECEIPT:
        raise Blocked("Rust witness safety behavior differs")
    return exe

def smoke(source):
    source=Path(source).resolve()
    if sha(external_run(["git","show","HEAD:src/kernel.rs"],cwd=source,timeout=50))!=BASE_KERNEL_SHA:
        raise Blocked("CI baseline kernel SHA mismatch")
    rust=external_run(["git","show","HEAD:research/g03_r3_r18_require_witness.rs"],
                       cwd=source,timeout=50)
    if blob(rust)!=RUST_BLOB:
        raise Blocked("CI witness Git blob mismatch")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R318_CI_") as root:
        work=Path(root)
        witness=work/"witness.rs"
        witness.write_bytes(rust)
        security_build(source,work,witness)
    print("G03_R3_R18_RUST_SECURITY_SMOKE=PASS")
    print("MAC_INTEL_ASSEMBLY=NOT_EXAMINED_IN_CI")
    print("PRODUCTION=NOT_AUTHORIZED")

def selftest():
    base=[
      {"address":"0x100001000","asm":"pushq\t%rbp"},
      {"address":"0x100001001","asm":"je\t0x100001008"},
      {"address":"0x100001003","asm":"callq\t_foo"},
      {"address":"0x100001008","asm":"retq"}]
    shifted=[
      {"address":"0x100002000","asm":"pushq\t%rbp"},
      {"address":"0x100002001","asm":"je\t0x100002008"},
      {"address":"0x100002003","asm":"callq\t_foo"},
      {"address":"0x100002008","asm":"retq"}]
    def make(start,items):
        return {"start_hex":start,"size_upper_bound_bytes":16,
          "disassembly":items,"instruction_lines":len(items),
          "asm_sha256":"synthetic-only"}
    x=summarize_function(make("0x100001000",base))
    y=summarize_function(make("0x100002000",shifted))
    assert x["normalized_assembly"]==y["normalized_assembly"]
    assert x["alignment_mod_16"]==y["alignment_mod_16"]
    bad=[dict(item) for item in shifted]
    bad[0]["asm"]="popq\t%rbp"
    z=summarize_function(make("0x100002000",bad))
    assert x["normalized_assembly"]!=z["normalized_assembly"]
    samples=[{"name":"__RNvMs_nordoi_kernel6kernelAtomicKernel7require"},
      {"name":"__RNvMNt_nordoi_kernel10capabilityCapabilitySet8contains"},
      {"name":"_nordoi_r17_require_witness"}]
    found=named_symbols(samples)
    assert all(found.values())
    assert named_symbols(samples+[samples[1]])["contains"] is None
    assert blob(b"123")=="d800886d9c86731ae5c4a62b0b77c437015e00d2"
    print("G03_R3_R18_SYMBOL_AND_BRANCH_SELFTEST=SYNTHETIC_PASS")
    print("REAL_MAC_ASM_INSPECTION=NONE_IN_SELFTEST")
    print("PRODUCTION=NOT_AUTHORIZED")

def audit(r13_path,r17_path,rust_path,output):
    if platform.system()!="Darwin" or platform.machine()!="x86_64":
        raise Blocked("actual codegen audit requires Mac Intel")
    if any(shutil.which(n) is None for n in ("git","cargo","rustc","nm","otool")):
        raise Blocked("Mac Rust/Apple tools unavailable")
    r17=load_r17(r17_path)
    r13=r17.load_r13(r13_path)
    rust=pin(rust_path,RUST_BLOB,12000)
    output=Path(output)
    if output.exists() or output.is_symlink() or not output.parent.is_dir():
        raise Blocked("private receipt path exists or invalid")
    with tempfile.TemporaryDirectory(prefix="NORDOI_R318_DISASM_") as d:
        root=Path(d)
        sources={"baseline":root/"sources"/"base",
                 "candidate":root/"sources"/"candidate"}
        (root/"sources").mkdir()
        for label,source in sources.items():
            r13.clone(source,"main" if label=="baseline" else r13.BRANCH,
                      BASE_SHA if label=="baseline" else CAND_SHA)
        guard=r17.verify_source(r13,sources["baseline"],sources["candidate"])
        results={}
        (root/"builds").mkdir()
        for label,source in sources.items():
            work=root/"builds"/label
            work.mkdir()
            rust_file=work/"witness.rs"
            rust_file.write_bytes(rust)
            exe=r17.build_witness(source,work,rust_file)
            results[label]=inspect_macho(exe,r17)
            results[label]["default_deny_explicit_grant"]="PASS"
        comparisons=cross_compare(results["baseline"],results["candidate"])
        callcheck=all(results[k]["require_calls_contains"] for k in results)
        report={
            "schema":SCHEMA,"baseline_source_sha":BASE_SHA,"candidate_source_sha":CAND_SHA,
            "r13_tool_git_blob":R13_BLOB,"r17_tool_git_blob":R17_BLOB,
            "witness_git_blob":RUST_BLOB,
            "source_guard":guard,
            "host":{"system":platform.system(),"machine":platform.machine(),
                    "python":platform.python_version(),"cpu_count":os.cpu_count()},
            "compiler":{"rustc":external_run(["rustc","--version"]).decode().strip(),
                        "cargo":external_run(["cargo","--version"]).decode().strip()},
            "builds":results,"comparisons":comparisons,
            "call_edge_require_contains_present_both":callcheck,
            "security_contract":"PASS_BOTH_DEFAULT_DENY_AND_EXPLICIT_GRANT",
            "full_path_machine_byte_equivalence":"NOT_PROVEN",
            "layout_causes_denied_latency":"NOT_ESTABLISHED",
            "new_speed_measurements":"NONE",
            "original_repo_modified":False,
            "merge_authorization":"NONE","production":"NOT_AUTHORIZED",
            "human_evaluation_gate":"CLOSED_NOT_REQUIRED"}
        r17.private_output(output,report)
    print("G03_R3_R18_MAC_MACHINE_CODE_AUDIT=PASS")
    print("SOURCE_GUARD=PASS")
    print("RUST_DENY_GRANT_SECURITY=2_OF_2_PASS")
    print("REQUIRE_CALLS_CONTAINS_BOTH="+str(callcheck).upper())
    for kind in FUNCTIONS:
        data=comparisons[kind]
        print(kind.upper()+"_COMPARISON="+data["status"])
        if data["status"]=="COMPARABLE":
            print(kind.upper()+"_NORMALIZED_TEXT_EQUAL="+
                  str(data["normalized_text_identical"]).upper())
            print(kind.upper()+"_ADDRESS_SHIFT_BYTES="+
                  str(data["candidate_minus_baseline_address_bytes"]))
            print(kind.upper()+"_ALIGNMENT_64_CHANGED="+
                  str(data["alignment_mod_64_changed"]).upper())
    print("DENIED_CAPABILITY_ROOT_CAUSE=NOT_ESTABLISHED")
    print("MERGE_AUTHORIZATION=NONE")
    print("REPORT_PRIVATE="+str(output))
    print("PRODUCTION=NOT_AUTHORIZED")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="action",required=True)
    sub.add_parser("selftest")
    s=sub.add_parser("smoke");s.add_argument("--source",required=True)
    a=sub.add_parser("audit")
    a.add_argument("--r13-tool",required=True)
    a.add_argument("--r17-tool",required=True)
    a.add_argument("--rust-witness",required=True)
    a.add_argument("--out",required=True)
    args=p.parse_args()
    try:
        if args.action=="selftest":selftest()
        elif args.action=="smoke":smoke(args.source)
        else:audit(args.r13_tool,args.r17_tool,args.rust_witness,args.out)
        return 0
    except (Blocked,OSError,ValueError,UnicodeError,TypeError,KeyError,AttributeError,
            AssertionError,subprocess.TimeoutExpired) as e:
        print("G03_R3_R18_GATE=BLOCKED",file=sys.stderr)
        print("REASON="+str(e)[:220],file=sys.stderr)
        print("PRODUCTION=NOT_AUTHORIZED",file=sys.stderr)
        return 2

if __name__=="__main__":
    sys.exit(main())
