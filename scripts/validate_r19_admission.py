#!/usr/bin/env python3
"""R0.19 — bounded offline synthetic admission gate contract (no real trust authority)."""
from pathlib import Path
import csv
import hashlib
import io
import re
import runpy
import struct
import subprocess

ROOT = Path(__file__).resolve().parent.parent
BASE = '2839c377e78e3c0e58693bc5aa8e668d8e0e436f'
PACK_SHA = 'f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393'
REPLAY_SHA = '1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab'
FAULT_SHA = '9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293'
POLICY_SHA = '7e7f3eccc8e06f6acc0bbb811ddd82b7813654b2b7a676764a5341a81643a3e2'
TRANSCRIPT_SHA = 'ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33'
HEADER = 'id\tgate_id\tarea\tscenario\tbaseline_sha\tpack_sha256\treplay_sha256\tfault_sha256\tproducer_key\tsignature_ref\tverifier_ref\trevocation\tscope\tci\treview\tnative_request\texpected\tlimitation'
WITNESS_HEADER = 'id\twitness\tclassification\tlimitation'
SCENARIOS = ['CLEAN_MOCK', 'TAMPER_ROOT', 'FORGED_SIGNATURE', 'FORGED_VERIFIER', 'REVOKED_ROOT', 'MISSING_CI', 'MISSING_REVIEW', 'NATIVE_ESCALATION']
NEW = {
    'README_R0_19.md',
    'docs/NORDOI_R0_19_EVIDENCE_ADMISSION_GATE_SPEC.md',
    'governance/r19_admission_attempts_v1.tsv',
    'governance/r19_admission_witnesses_v1.tsv',
    'research/R0_19_ADMISSION_BOUNDARY_DECISION.md',
    'research/fixtures/r19_admission_attempts_v1.bin',
    'scripts/validate_r19_admission.py',
    'tests/resource_task_r19.rs',
}
sha = lambda b: hashlib.sha256(b).hexdigest()

def normalized(path):
    data = (ROOT/path).read_bytes().replace(b'\r\n',b'\n')
    if b'\r' in data: raise RuntimeError(f'Unmatched CR in {path}')
    return data

def rows(path, header):
    data = normalized(path).decode('utf-8')
    if not data.startswith(header+'\n'):raise RuntimeError('Header mismatch: '+path)
    entries=list(csv.DictReader(io.StringIO(data),delimiter='\t'))
    if not entries or any(len(e)!=len(header.split('\t')) or None in e or any(not val or not val.strip() for val in e.values()) for e in entries):
        raise RuntimeError('Malformed TSV: '+path)
    return entries

def decide(e):
    if e['pack_sha256']!=PACK_SHA or e['replay_sha256']!=REPLAY_SHA or e['fault_sha256']!=FAULT_SHA:
        return 'INVALID'
    if e['scope']!='RESEARCH_ONLY' or e['signature_ref']!='NONE' or e['verifier_ref']!='NONE' or e['revocation']!='UNKNOWN' or e['native_request']!='NO':
        return 'BLOCKED'
    if e['ci']!='YES' or e['review']!='YES' or not e['producer_key'].startswith('MOCK-PRODUCER-'):
        return 'BLOCKED'
    return 'RESEARCH_REVIEW_ONLY'

def check_static():
    # historical checks, but must not claim native readiness from these synthetic fixtures
    runpy.run_path(str(ROOT/'scripts/validate_r18_faults.py'),run_name='r19_frozen_reference')['static']()
    inputs={
        'research/fixtures/r16_synthetic_bundle_v1.bin':PACK_SHA,
        'research/fixtures/r17_independent_replay_transcript_v1.bin':REPLAY_SHA,
        'research/fixtures/r18_differential_fault_results_v1.bin':FAULT_SHA,
    }
    for name,expected in inputs.items():
        if sha((ROOT/name).read_bytes())!=expected:raise RuntimeError('Frozen byte input mismatch: '+name)
    for num, (path, col, expected) in enumerate([
        ('governance/r11_native_readiness_gates_v1.tsv','status','UNMET'),
        ('governance/r12_evidence_ledger_v1.tsv','status','ABSENT'),
        ('governance/r13_integrity_chain_v1.tsv','status','ABSENT'),
        ('governance/r14_authentication_registry_v1.tsv','status','ABSENT'),
        ('governance/r15_trust_policy_registry_v1.tsv','status','ABSENT'),
    ]):
        data=normalized(path).decode('utf-8')
        parsed=list(csv.DictReader(io.StringIO(data),delimiter='\t'))
        if len(parsed)!=12 or any(r[col]!=expected or r['area']!=areas[i] for i,r in enumerate(parsed)):
            raise RuntimeError(f'Historical state violation: {path}')
    entries=rows('governance/r19_admission_attempts_v1.tsv',HEADER)
    if len(entries)!=96 or sha(normalized('governance/r19_admission_attempts_v1.tsv'))!=POLICY_SHA:
        raise RuntimeError('R0.19 canonical policy registry root mismatch')
    transcript=bytearray(b'NDR19AD1'+struct.pack('<H',96))
    counts={'INVALID':0,'BLOCKED':0,'RESEARCH_REVIEW_ONLY':0}
    for n,e in enumerate(entries,1):
        gate=(n-1)//8+1
        scenario=SCENARIOS[(n-1)%8]
        if e['id']!=f'R19-A{n:03}' or e['gate_id']!=f'R11-G{gate:02}' or e['area']!=areas[gate-1] or e['scenario']!=scenario or e['baseline_sha']!=BASE or e['producer_key']!=f'MOCK-PRODUCER-{gate:02}':
            raise RuntimeError('Wrong row order/identity')
        decision=decide(e)
        if e['expected']!=decision or (scenario=='CLEAN_MOCK')!=(decision=='RESEARCH_REVIEW_ONLY'):
            raise RuntimeError(f'Admission decision mismatch {n}')
        counts[decision]+=1
        line='\t'.join(e[k] for k in HEADER.split('\t'))+'\n'
        transcript.extend(struct.pack('<HBB',n,(n-1)%8,{'INVALID':0,'BLOCKED':1,'RESEARCH_REVIEW_ONLY':2}[decision]))
        transcript.extend(hashlib.sha256(line.encode('utf-8')).digest())
    transcript.extend(hashlib.sha256(transcript).digest())
    fixture=(ROOT/'research/fixtures/r19_admission_attempts_v1.bin').read_bytes()
    if len(fixture)!=3498 or fixture!=transcript or sha(fixture)!=TRANSCRIPT_SHA:
        raise RuntimeError('R0.19 independent transcript mismatch')
    if counts!={'INVALID':12,'BLOCKED':72,'RESEARCH_REVIEW_ONLY':12}:
        raise RuntimeError('R0.19 decision counts unexpected')
    rust=(ROOT/'tests/resource_task_r19.rs').read_text()
    tests=re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses=rows('governance/r19_admission_witnesses_v1.tsv',WITNESS_HEADER)
    if len(tests)!=len(set(tests)) or len(witnesses)!=len(tests) or [w['witness'] for w in witnesses]!=tests:
        raise RuntimeError('Rust test count or witness mapping mismatch')
    for i,w in enumerate(witnesses,1):
        if w['id']!=f'R19-P{i:02}' or w['classification'] not in {'FROZEN_LINEAGE','POLICY_ENFORCEMENT','TRUST_BOUNDARY'} or len(w['limitation'])<70:
            raise RuntimeError('Invalid witness metadata')
    for text in [BASE,PACK_SHA,REPLAY_SHA,FAULT_SHA,POLICY_SHA,TRANSCRIPT_SHA,'R19_WITNESS','ResearchReviewOnly','fn decide(','fn parse_rows(','fn expected_bytes(']:
        if text not in rust:raise RuntimeError('Missing Rust boundary '+text)
    if any(needle in rust for needle in ['unsafe {','#[allow(', 'ProductionAuthorized','NativeAuthorized','std::process::Command']):
        raise RuntimeError('Unexpected production authority or warning suppression')
    docs='\n'.join((ROOT/p).read_text() for p in NEW if p.endswith('.md'))
    for text in [BASE,TRANSCRIPT_SHA,'UNMET','ABSENT','TEST-ONLY','RESEARCH_ONLY','96','NOT']:
        if text not in docs:raise RuntimeError('Incomplete limits documentation '+text)
    print('R0.19 static contract: PASS')
    print('Certified input: r0.18 / '+BASE)
    print('Synthetic admission cases: 96/96; INVALID=12, BLOCKED=72, RESEARCH_REVIEW_ONLY=12')
    print('Historical statuses: 12/12 UNMET, 12/12 real external evidence ABSENT')
    print('Transcript: 3498 bytes; SHA-256 '+TRANSCRIPT_SHA)
    print('Rust witnesses: '+str(len(tests))+'/'+str(len(tests))+'; two inherited SHA-256 vector tests additional')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; real signatures, verifiers and native authority absent')

def boundary():
    def git(*args): return subprocess.run(['git',*args],cwd=ROOT,text=True,capture_output=True,check=False)
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve()!=ROOT.resolve():
        print('R0.18 Git boundary: NOT CHECKED (reconstructed non-Git tree)');return
    tag=git('rev-parse','r0.18^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:raise RuntimeError('Certified baseline tag mismatch')
    historical=git('ls-tree','-r','--name-only','r0.18')
    if historical.returncode:raise RuntimeError('Cannot inspect certified tree')
    files=set(historical.stdout.splitlines())
    if files&NEW:raise RuntimeError('Previously certified path overwritten')
    diff=git('diff','--name-only','r0.18','--','.')
    if diff.returncode or set(diff.stdout.splitlines())-NEW:raise RuntimeError('Historical tracked change')
    staged=git('diff','--cached','--name-only')
    if staged.returncode or set(staged.stdout.splitlines())-NEW:raise RuntimeError('Unexpected staged change')
    untracked=git('ls-files','--others','--exclude-standard')
    if untracked.returncode or set(untracked.stdout.splitlines())-NEW:raise RuntimeError('Unexpected untracked change')
    if any(not (ROOT/item).is_file() for item in files):raise RuntimeError('Historical tracked file missing')
    print('R0.18 Git boundary: PASS (eight additive paths; frozen tree unchanged)')

# define area list via canonical registry, not untrusted new test rows
areas = [r['area'] for r in list(csv.DictReader(io.StringIO(normalized('governance/r11_native_readiness_gates_v1.tsv').decode()), delimiter='\t'))] if __name__=='__main__' else []
if __name__=='__main__':
    check_static()
    boundary()
    print('Rust tests, Clippy, full Release Gate, GitHub CI and external audit: NOT RUN by this validator')
