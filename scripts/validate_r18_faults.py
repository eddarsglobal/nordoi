#!/usr/bin/env python3
"""NORDOI R0.18: bounded deterministic adversarial fault replay static contract.
This validator never performs real external authentication or native execution.
"""
from pathlib import Path
import csv
import hashlib
import io
import os
import re
import runpy
import struct
import subprocess

ROOT = Path(__file__).resolve().parent.parent
BASE = 'c5f76bc2e393f95f38552778f870fac4be8206c3'
PACK_SHA = 'f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393'
TRANS_SHA = '1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab'
MAN_SHA = 'f4dfbb95a61e1a9184ca459f7279a0c686166de10411554c8364445cb75dcb24'
REG_SHA = 'c0d649990dbeb318408a8e82af34478fc7cf1f50834e8f69a05d0464d1e2e242'
CAMPAIGN_SHA = '281ea03f893b54e2c03d12e7df1236da249ff1d8a61cbe0a8f5b4fb18d26b783'
RESULT_SHA = '9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293'
NEW = {
    'README_R0_18.md',
    'docs/NORDOI_R0_18_DIFFERENTIAL_FAULT_SPEC.md',
    'governance/r18_fault_campaign_v1.tsv',
    'governance/r18_fault_witnesses_v1.tsv',
    'research/R0_18_ADVERSARIAL_BOUNDARY_DECISION.md',
    'research/fixtures/r18_differential_fault_results_v1.bin',
    'scripts/validate_r18_faults.py',
    'tests/resource_task_r18.rs',
}
C18 = 'id\tdomain\toperation\tposition\targument\treader_a\treader_b\tmutated_sha256\texpected\tscope\tlimitation'
W18 = 'id\twitness\tclassification\tlimitation'
sha=lambda b:hashlib.sha256(b).hexdigest()

def normalized(path: str) -> bytes:
    source=(ROOT/path).read_bytes()
    output=source.replace(b'\r\n',b'\n')
    if b'\r' in output:
        raise RuntimeError(f'Unexpected embedded CR in {path}')
    return output

def tsv(path: str, header: str) -> list[dict[str,str]]:
    value=normalized(path).decode('utf-8')
    if value.splitlines()[0] != header:
        raise RuntimeError(f'Incorrect header {path}')
    rows=list(csv.DictReader(io.StringIO(value),delimiter='\t'))
    if not rows or any(None in r or any(v is None or not v.strip() for v in r.values()) for r in rows):
        raise RuntimeError(f'Incomplete TSV {path}')
    if any(len(r)!=len(header.split('\t')) for r in rows):
        raise RuntimeError(f'Invalid TSV width {path}')
    return rows

def reference_legacy() -> None:
    namespace=runpy.run_path(str(ROOT/'scripts/validate_r17_replay.py'),run_name='frozen_r017_reference')
    namespace['check_static']()

def decoder_a(blob:bytes):
    if len(blob)<10 or blob[:8]!=b'NDR16PK1' or blob[8:10]!=b'\x0c\x00':return None
    result=[];cursor=10
    for index in range(1,13):
        if cursor+2>len(blob):return None
        size=int.from_bytes(blob[cursor:cursor+2],'little');cursor+=2
        if not 1<=size<=256 or cursor+size>len(blob):return None
        result.append((index,cursor,size,sha(blob[cursor:cursor+size])));cursor+=size
    return result if cursor==len(blob) else None

def decoder_b(blob:bytes,manifest:bytes):
    if sha(manifest)!=MAN_SHA or blob[:8]!=b'NDR16PK1' or blob[8:10]!=b'\x0c\x00':return None
    entries=list(csv.DictReader(io.StringIO(manifest.decode('ascii')),delimiter='\t'))
    if len(entries)!=12:return None
    result=[];end=10
    for index,entry in enumerate(entries,1):
        try:offset=int(entry['offset']);size=int(entry['length'])
        except (KeyError,TypeError,ValueError):return None
        if not 1<=size<=256 or offset!=end+2 or end+2>len(blob):return None
        if int.from_bytes(blob[end:end+2],'little')!=size or offset+size>len(blob):return None
        result.append((index,offset,size,sha(blob[offset:offset+size])));end=offset+size
    return result if end==len(blob) else None

def apply_fault(f:dict[str,str], sources:dict[str,bytes]) -> bytes:
    if f['domain'] not in sources or f['operation'] not in {'XOR','ZERO','CUT','ADD','SWAP'}:
        raise RuntimeError('Unrecognized mutation operation')
    src=sources[f['domain']]
    if not 0<=int(f['position'])<=len(src):raise RuntimeError('Out-of-bounds fault')
    pos=int(f['position']);arg=int(f['argument']);out=bytearray(src)
    if f['operation']=='XOR':out[pos]^=arg
    if f['operation']=='ZERO':out[pos]=0
    if f['operation']=='CUT':del out[pos:]
    if f['operation']=='ADD':out.append(arg)
    if f['operation']=='SWAP':out[pos],out[arg]=out[arg],out[pos]
    if bytes(out)==src:raise RuntimeError('Vacuous fault, untrusted')
    return bytes(out)

def static() -> None:
    for item in NEW:
        p=ROOT/item
        if not p.is_file() or p.is_symlink():raise RuntimeError(f'Absent or symlinked R0.18 file: {item}')
    reference_legacy()
    sources={
        'PACK':(ROOT/'research/fixtures/r16_synthetic_bundle_v1.bin').read_bytes(),
        'TRANSCRIPT':(ROOT/'research/fixtures/r17_independent_replay_transcript_v1.bin').read_bytes(),
        'MANIFEST':normalized('governance/r16_offline_bundle_manifest_v1.tsv'),
        'REGISTRY':normalized('governance/r17_replay_registry_v1.tsv'),
    }
    for name,root in [('PACK',PACK_SHA),('TRANSCRIPT',TRANS_SHA),('MANIFEST',MAN_SHA),('REGISTRY',REG_SHA)]:
        if sha(sources[name])!=root:raise RuntimeError(f'R0.18 certified input root mismatch {name}')
    if decoder_a(sources['PACK']) != decoder_b(sources['PACK'],sources['MANIFEST']):
        raise RuntimeError('Frozen baseline decoders disagree')
    campaign=tsv('governance/r18_fault_campaign_v1.tsv',C18)
    witnesses=tsv('governance/r18_fault_witnesses_v1.tsv',W18)
    if len(campaign)!=60 or len(witnesses)!=38 or sha(normalized('governance/r18_fault_campaign_v1.tsv'))!=CAMPAIGN_SHA:
        raise RuntimeError('Frozen 60-case campaign or 38-witness contract invalid')
    output=bytearray(b'NDR18FLT1'+BASE.encode()+struct.pack('<H',60)+bytes.fromhex(PACK_SHA)+bytes.fromhex(TRANS_SHA))
    a_yes=b_no=disagree=0
    for n,f in enumerate(campaign,1):
        changed=apply_fault(f,sources)
        if f['id']!=f'R18-F{n:02}' or f['mutated_sha256'] != sha(changed) or f['expected']!='INVALID' or f['scope']!='RESEARCH_ONLY' or len(f['limitation'])<65:
            raise RuntimeError(f'Fault witness {n} invalid')
        pack=changed if f['domain']=='PACK' else sources['PACK']
        manifest=changed if f['domain']=='MANIFEST' else sources['MANIFEST']
        a=decoder_a(pack);b=decoder_b(pack,manifest)
        ap='PASS' if a is not None else 'REJECT';bp='PASS' if b is not None else 'REJECT'
        if f['reader_a']!=ap or f['reader_b']!=bp:raise RuntimeError(f'Differential classification {n} changed')
        a_yes+=(a is not None and b is not None)
        b_no+=(a is None)
        disagree+=(ap!=bp)
        tp=changed if f['domain']=='TRANSCRIPT' else sources['TRANSCRIPT']
        rg=changed if f['domain']=='REGISTRY' else sources['REGISTRY']
        invalid=(sha(pack)!=PACK_SHA or sha(manifest)!=MAN_SHA or sha(tp)!=TRANS_SHA or sha(rg)!=REG_SHA or a!=b)
        if not invalid:raise RuntimeError(f'Corruption accepted despite pinned root: {n}')
        kind=['PACK','TRANSCRIPT','MANIFEST','REGISTRY'].index(f['domain'])
        mask=int(a is not None)+2*int(b is not None)
        output.extend(struct.pack('<HBB',n,kind,mask))
        output.extend(hashlib.sha256(changed).digest())
    output.extend(hashlib.sha256(output).digest())
    golden=(ROOT/'research/fixtures/r18_differential_fault_results_v1.bin').read_bytes()
    if bytes(output)!=golden or len(golden)!=2307 or sha(golden)!=RESULT_SHA:
        raise RuntimeError('Python independent golden result mismatch')
    if a_yes<12 or b_no<15 or disagree<3:
        raise RuntimeError('Insufficient structural differential coverage')
    rust=(ROOT/'tests/resource_task_r18.rs').read_text(encoding='utf-8')
    tests=re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    if len(tests)!=38 or len(set(tests))!=38 or [w['witness'] for w in witnesses]!=tests:
        raise RuntimeError('Rust tests do not match 38 witnesses')
    if any(w['id']!=f'R18-P{n:02}' or w['classification'] not in {'FROZEN_BASELINE','DIFFERENTIAL_FAULT','TRUST_BOUNDARY'} or len(w['limitation'])<65 for n,w in enumerate(witnesses,1)):
        raise RuntimeError('Rust witness integrity invalid')
    needed=(BASE,PACK_SHA,TRANS_SHA,MAN_SHA,REG_SHA,CAMPAIGN_SHA,RESULT_SHA,'fn decode_a(','fn decode_b(',
            'fn mutation(','fn campaign(','fn assess(','fn normalized_lf(', 'fn frozen_status(',
            'R18_WITNESS version=1', 'ResearchReviewOnly','#[path = "../src/effect_audit/hash.rs"]')
    if any(x not in rust for x in needed) or any(bad in rust for bad in ('unsafe {','#[ignore]','#[should_panic]','std::process::Command','ProductionAuthorized','NativeAuthorized')):
        raise RuntimeError('Unsafe or missing R0.18 Rust boundary')
    docs='\n'.join((ROOT/item).read_text() for item in NEW if item.endswith('.md'))
    if any(x not in docs for x in (BASE,PACK_SHA,RESULT_SHA,'UNMET','ABSENT','TEST-ONLY','RESEARCH_ONLY','60','40','NOT')):
        raise RuntimeError('Required research boundary disclosure absent')
    print('R0.18 static contract: PASS')
    print(f'Certified baseline: r0.17 / {BASE}')
    print('Synthetic deterministic faults: 60/60; structural differential outcomes: REPRODUCIBLE')
    print(f'Decoder structural pass-both: {a_yes}; A rejections: {b_no}; disagreements: {disagree}')
    print(f'Golden transcript SHA-256: {RESULT_SHA} / 2307 bytes')
    print('Governance witnesses: 38/38; external verifiers 12/12 ABSENT; native readiness 12/12 UNMET')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; no real identity authentication or native authority')

def git(*args:str):
    return subprocess.run(['git',*args],cwd=ROOT,capture_output=True,text=True,check=False)

def boundary() -> None:
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve()!=ROOT.resolve():
        print('R0.17 Git boundary: NOT CHECKED (reconstructed non-Git tree)')
        return
    tag=git('rev-parse','r0.17^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:raise RuntimeError('Certified r0.17 tag absent or mismatched')
    frozen=git('ls-tree','-r','--name-only','r0.17')
    if frozen.returncode:raise RuntimeError('Cannot inspect historical tracked tree')
    frozen_paths=set(frozen.stdout.splitlines())
    if frozen_paths & NEW:raise RuntimeError('R0.18 overlaps previously certified paths')
    changes=git('diff','--name-only','r0.17','--','.')
    if changes.returncode or set(changes.stdout.splitlines())-NEW:raise RuntimeError('Frozen paths changed')
    staged=git('diff','--cached','--name-only')
    if staged.returncode or set(staged.stdout.splitlines())-NEW:raise RuntimeError('Unexpected staged modifications')
    others=git('ls-files','--others','--exclude-standard')
    if others.returncode or set(others.stdout.splitlines())-NEW:raise RuntimeError('Unexpected untracked paths')
    if any(not (ROOT/item).is_file() for item in frozen_paths):raise RuntimeError('Historical file deleted')
    print('R0.17 Git boundary: PASS (exactly 8 allowed additive paths, no frozen changes)')

if __name__=='__main__':
    static()
    boundary()
    print('Rust tests, Clippy, Release Gate, independent external audit and GitHub CI: NOT RUN here')
