#!/usr/bin/env python3
"""NORDOI R0.22 synthetic witness fork isolation: no real trust or native authority."""
from __future__ import annotations
import csv
import hashlib
import io
from pathlib import Path
import re
import runpy
import struct
import subprocess

ROOT=Path(__file__).resolve().parent.parent
BASE='a34bc552b31661570f270c6c75700849b261084c'
R21_SHA='cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba'
POLICY_SHA='31a2f2ef5fbadf9d3a026e570db45e9da0c0d990ffafb52a3577bc267b14155f'
TRANSCRIPT_SHA='bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da'
LIMITATION='SYNTHETIC_ONLY_NO_REAL_SIGNING_VERIFICATION_PRODUCTION_OR_NATIVE_AUTHORITY'
SCENARIOS=['REGISTER_A', 'REGISTER_B', 'ADVANCE_A', 'ADVANCE_B', 'FORK_A_EQUAL_HEIGHT', 'REPLAY_A', 'STALE_PARENT_A', 'CROSS_ROOT_PARENT', 'ADVANCE_B_3', 'REVOKE_A', 'RESURRECT_A', 'ADVANCE_B_AFTER_REVOKE', 'FORGED_EXTERNAL', 'TAMPER_R21_LINEAGE', 'NATIVE_ESCALATION']
HEADER=['id', 'gate_id', 'area', 'scenario', 'baseline_sha', 'evidence_sha256', 'actor', 'target', 'root_claim', 'epoch_claim', 'height_claim', 'parent_claim', 'witness_claim', 'producer', 'scope', 'signature', 'verifier', 'ci', 'review', 'native', 'before_state', 'after_state', 'expected', 'limitation']
WITNESS_HEADER=['id','witness','classification','limitation']
NEW={
 'README_R0_22.md',
 'docs/NORDOI_R0_22_WITNESS_FORK_ISOLATION_SPEC.md',
 'governance/r22_witness_forks_v1.tsv',
 'governance/r22_witness_fork_witnesses_v1.tsv',
 'research/R0_22_WITNESS_FORK_BOUNDARY_DECISION.md',
 'research/fixtures/r22_witness_forks_v1.bin',
 'scripts/validate_r22_witness_forks.py',
 'tests/resource_task_r22.rs',
}

def sha(data:bytes)->str:return hashlib.sha256(data).hexdigest()
def normalized(path):
    data=(ROOT/path).read_bytes().replace(b'\r\n',b'\n')
    if b'\r' in data:raise RuntimeError('Bare CR in '+path)
    return data

def read_tsv(path,header):
    content=normalized(path).decode('utf-8')
    if not content.startswith('\t'.join(header)+'\n'):raise RuntimeError('TSV header mismatch: '+path)
    rows=list(csv.DictReader(io.StringIO(content),delimiter='\t'))
    if not rows or any(set(r)!=set(header) or None in r or any(v is None or not v for v in r.values()) for r in rows):
        raise RuntimeError('Malformed TSV: '+path)
    return rows

def root_name(g,l,e):return f'MOCK-G{g:02}-{l}-E{e}'
def initial():return {l:{'epoch':0,'root':'NONE','height':0,'head':'0'*64,'revoked':False} for l in 'AB'}
def state_text(s):return '|'.join(f'{l}={s[l]["epoch"]}:{s[l]["root"]}:{s[l]["height"]}:{s[l]["head"]}:{int(s[l]["revoked"])}' for l in 'AB')
def witness(g,l,e,h,parent,r,action):return sha(f'R22-WITNESS|{g:02}|{l}|{e}|{h}|{parent}|{r}|{action}'.encode())

def decide(row,s,g):
    if row['baseline_sha']!=BASE or row['evidence_sha256']!=R21_SHA:return 'INVALID'
    try:e=int(row['epoch_claim']);h=int(row['height_claim'])
    except ValueError:return 'INVALID'
    if not 0<=e<=255 or not 0<=h<=255:return 'INVALID'
    for k,v in {'producer':'MOCK_R22','scope':'RESEARCH_ONLY','signature':'NONE','verifier':'NONE','ci':'YES','review':'YES','native':'NO'}.items():
        if row[k]!=v:return 'BLOCKED'
    if row['actor']!=row['target'] or row['target'] not in ('A','B'):return 'BLOCKED'
    l=row['target'];lane=s[l];action=row['scenario']
    if lane['revoked']:return 'BLOCKED'
    if row['witness_claim']!=witness(g,l,e,h,row['parent_claim'],row['root_claim'],action):return 'BLOCKED'
    if row['parent_claim']!=lane['head'] or h!=lane['height']+1:return 'BLOCKED'
    okay=False;revoke=False
    if action in ('REGISTER_A','REGISTER_B') and action[-1]==l:
        okay=lane['height']==0 and e==1 and h==1 and row['root_claim']==root_name(g,l,1)
    elif action in ('ADVANCE_A','ADVANCE_B') and action[-1]==l:
        okay=lane['height']==1 and e==2 and h==2 and row['root_claim']==root_name(g,l,2)
    elif action=='ADVANCE_B_3' and l=='B':
        okay=lane['height']==2 and e==3 and h==3 and row['root_claim']==root_name(g,l,3)
    elif action=='REVOKE_A' and l=='A':
        okay=lane['height']==2 and e==2 and h==3 and row['root_claim']==lane['root'] and s['B']['height']==3
        revoke=okay
    elif action=='ADVANCE_B_AFTER_REVOKE' and l=='B':
        okay=lane['height']==3 and s['A']['revoked'] and e==4 and h==4 and row['root_claim']==root_name(g,l,4)
    if not okay:return 'BLOCKED'
    lane.update(epoch=e,root=row['root_claim'],height=h,head=row['witness_claim'],revoked=revoke)
    return 'RESEARCH_REVIEW_ONLY'

def static():
    # Executes frozen research static contracts, never older Git boundary functions.
    r21=runpy.run_path(str(ROOT/'scripts/validate_r21_epochs.py'),run_name='r22_frozen_reference')
    r21['run_static']()
    old=(ROOT/'research/fixtures/r21_cross_root_transcript_v1.bin').read_bytes()
    if len(old)!=6646 or sha(old)!=R21_SHA:raise RuntimeError('R0.21 frozen transcript mismatch')
    gates_path='governance/r11_native_readiness_gates_v1.tsv'
    gates=read_tsv(gates_path,normalized(gates_path).decode().splitlines()[0].split('\t'))
    ext_path='governance/r15_trust_policy_registry_v1.tsv'
    ext=read_tsv(ext_path,normalized(ext_path).decode().splitlines()[0].split('\t'))
    areas=[r['area'] for r in gates]
    if len(areas)!=12 or len(ext)!=12 or any(r['status']!='UNMET' for r in gates) or any(r['status']!='ABSENT' for r in ext):
        raise RuntimeError('12/12 frozen admission boundary violated')
    rows=read_tsv('governance/r22_witness_forks_v1.tsv',HEADER)
    if len(rows)!=180 or sha(normalized('governance/r22_witness_forks_v1.tsv'))!=POLICY_SHA:
        raise RuntimeError('Canonical R0.22 witness registry modified')
    transcript=bytearray(b'NDR22FK1'+struct.pack('<H',180)+bytes.fromhex(R21_SHA)+bytes.fromhex(BASE))
    counts={'INVALID':0,'BLOCKED':0,'RESEARCH_REVIEW_ONLY':0}
    s=initial()
    for idx,row in enumerate(rows):
        g=idx//15+1;j=idx%15
        if j==0:s=initial()
        if (row['id']!=f'R22-X{idx+1:03}' or row['gate_id']!=f'R11-G{g:02}' or row['area']!=areas[g-1] or row['scenario']!=SCENARIOS[j] or row['limitation']!=LIMITATION or row['baseline_sha']!=BASE):
            raise RuntimeError('Unexpected sequence or lineage at '+str(idx+1))
        if row['before_state']!=state_text(s):raise RuntimeError('Incorrect before-state at '+str(idx+1))
        a=s['A']['height'];b=s['B']['height']
        outcome=decide(row,s,g)
        expected='INVALID' if j==13 else 'RESEARCH_REVIEW_ONLY' if j in (0,1,2,3,8,9,11) else 'BLOCKED'
        if outcome!=expected or outcome!=row['expected'] or row['after_state']!=state_text(s):
            raise RuntimeError('Invalid transition or state at '+str(idx+1))
        counts[outcome]+=1
        transcript.extend(struct.pack('<H8B',idx+1,j,{'INVALID':0,'BLOCKED':1,'RESEARCH_REVIEW_ONLY':2}[outcome],a,s['A']['height'],b,s['B']['height'],int(s['A']['revoked']),int(s['B']['revoked'])))
        transcript.extend(hashlib.sha256(('\t'.join(row[k] for k in HEADER)+'\n').encode()).digest())
    transcript.extend(hashlib.sha256(transcript).digest())
    golden=(ROOT/'research/fixtures/r22_witness_forks_v1.bin').read_bytes()
    if len(golden)!=7654 or transcript!=golden or sha(golden)!=TRANSCRIPT_SHA:raise RuntimeError('R0.22 transcript does not reproduce')
    if counts!={'INVALID':12,'BLOCKED':84,'RESEARCH_REVIEW_ONLY':84}:raise RuntimeError('Histogram mismatch')
    rust=(ROOT/'tests/resource_task_r22.rs').read_text(encoding='utf-8')
    tests=re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses=read_tsv('governance/r22_witness_fork_witnesses_v1.tsv',WITNESS_HEADER)
    if len(tests)!=48 or len(set(tests))!=48 or len(witnesses)!=48 or [x['witness'] for x in witnesses]!=tests:
        raise RuntimeError('R0.22 witness inventory does not equal 48 distinct Rust tests')
    if any(row['id']!=f'R22-P{i:02}' or row['classification'] not in {'FROZEN_LINEAGE','FORK_REPLAY','ROOT_ISOLATION','TRUST_BOUNDARY'} or len(row['limitation'])<70 for i,row in enumerate(witnesses,1)):
        raise RuntimeError('Invalid Rust witness classification')
    for requirement in [BASE,R21_SHA,POLICY_SHA,TRANSCRIPT_SHA,'fn rebuild(', 'fn apply(', 'fn normalized_lf(', 'R22_WITNESS','ResearchReviewOnly']:
        if requirement not in rust:raise RuntimeError('Rust witness missing '+requirement)
    for forbidden in ['unsafe {','#[allow(', 'std::process::Command','ProductionAuthorized','NativeAuthorized']:
        if forbidden in rust:raise RuntimeError('Inadmissible trust claim in Rust')
    docs='\n'.join((ROOT/p).read_text() for p in NEW if p.endswith('.md'))
    for required in [BASE,POLICY_SHA,TRANSCRIPT_SHA,'TEST-ONLY','RESEARCH_ONLY','UNMET','ABSENT','NOT']:
        if required not in docs:raise RuntimeError('Documentation omitted '+required)
    print('R0.22 static contract: PASS')
    print('Certified baseline: r0.21 / '+BASE)
    print('Witness fork/monotonic replay cases: 180/180; INVALID=12, BLOCKED=84, RESEARCH_REVIEW_ONLY=84')
    print('Golden transcript: 7654 bytes; SHA-256 '+TRANSCRIPT_SHA)
    print('Rust witnesses: 48/48; plus two inherited SHA-256 vector tests (expected 50 total)')
    print('Native gates: 12/12 UNMET; real external verifiers: 12/12 ABSENT')
    print('Scope: TEST-ONLY / RESEARCH_ONLY, no external signing or native authority')

def check_git_boundary():
    def git(*args):return subprocess.run(['git',*args],cwd=ROOT,text=True,capture_output=True,check=False)
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve()!=ROOT.resolve():
        print('R0.21 Git boundary: NOT CHECKED (reconstructed non-Git tree)')
        return
    tag=git('rev-parse','r0.21^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:raise RuntimeError('Certified r0.21 tag drift')
    historic=git('ls-tree','-r','--name-only','r0.21')
    if historic.returncode:raise RuntimeError('Cannot inspect frozen Git tree')
    frozen=set(historic.stdout.splitlines())
    if frozen & NEW:raise RuntimeError('Collision with certified path')
    for args in [('diff','--name-only','r0.21','--','.'),('diff','--cached','--name-only'),('ls-files','--others','--exclude-standard')]:
        r=git(*args)
        if r.returncode or set(r.stdout.splitlines())-NEW:raise RuntimeError('Unrelated file has changed')
    if any(not (ROOT/p).is_file() for p in frozen):raise RuntimeError('Frozen file missing')
    if any(not (ROOT/p).is_file() for p in NEW):raise RuntimeError('Required new file missing')
    print('R0.21 Git boundary: PASS (eight additive paths; zero frozen edits)')

if __name__=='__main__':
    static()
    check_git_boundary()
    print('Rust tests, Clippy, full Release Gate, GitHub CI and independent audit: NOT RUN by this validator')
