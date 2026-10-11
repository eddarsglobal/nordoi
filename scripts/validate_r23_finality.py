#!/usr/bin/env python3
"""R0.23 deterministic research finality validation; no production authority."""
from pathlib import Path
import hashlib,csv,io,struct,re,runpy,subprocess
ROOT=Path(__file__).resolve().parent.parent
BASE='9d0bbdbb5153e5d38ff0f387ee381ad806c790d9'
R22_SHA='bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da'
LIMIT='SYNTHETIC_ONLY_NO_REAL_SIGNING_VERIFICATION_PRODUCTION_OR_NATIVE_AUTHORITY'
SCENARIOS=['REGISTER_A','REGISTER_B','ADVANCE_A','ADVANCE_B','FINALIZE_A','EQUIVOCATE_A_HEIGHT3','REWRITE_FINAL_A','REPLAY_A_PRE_FINAL','CROSS_ROOT_FINALITY','ADVANCE_B_3','FINALIZE_B','EQUIVOCATE_B_HEIGHT4','REVOKE_A','ADVANCE_B_POST_REVOKE','RESURRECT_A','FORGED_EXTERNAL','TAMPER_R22_LINEAGE','NATIVE_ESCALATION']
GOOD={0,1,2,3,4,9,10,12,13}; INVALID=16
AREAS=['LIFECYCLE_SEMANTICS','EXECUTOR_SCHEDULING','MEMORY_OWNERSHIP','CAPABILITY_AUTHORITY','CANCELLATION_CLEANUP','EFFECT_IO_BOUNDARY','CRASH_RECOVERY','ADVERSARIAL_TESTING','CROSS_PLATFORM','COMPILER_NAIR_BINDING','RESOURCE_LIMITS','INDEPENDENT_REVIEW']
HEADER=['id','gate_id','area','scenario','baseline_sha','evidence_sha256','actor','target','root_claim','epoch_claim','height_claim','parent_claim','witness_claim','producer','scope','signature','verifier','ci','review','native','before_state','after_state','expected','limitation']
WITNESS_HEADER=['id','witness','classification','limitation']
def sha(d):return hashlib.sha256(d).hexdigest()
def root(g,l,e):return f'MOCK-G{g:02}-{l}-E{e}'
def initial():return {l:{'epoch':0,'root':'NONE','height':0,'head':'0'*64,'revoked':False,'final_height':0,'final_head':'0'*64} for l in 'AB'}
def state_text(s):return '|'.join(f'{l}={s[l]["epoch"]}:{s[l]["root"]}:{s[l]["height"]}:{s[l]["head"]}:{int(s[l]["revoked"])}:{s[l]["final_height"]}:{s[l]["final_head"]}' for l in 'AB')
def witness(g,l,e,h,p,r,a):return sha(f'R23-WITNESS|{g:02}|{l}|{e}|{h}|{p}|{r}|{a}'.encode())
def decide(row,s,g):
    if row['baseline_sha']!=BASE or row['evidence_sha256']!=R22_SHA:return 'INVALID'
    try:e=int(row['epoch_claim']);h=int(row['height_claim'])
    except ValueError:return 'INVALID'
    if not 0<=e<=255 or not 0<=h<=255:return 'INVALID'
    for k,v in {'producer':'MOCK_R23','scope':'RESEARCH_ONLY','signature':'NONE','verifier':'NONE','ci':'YES','review':'YES','native':'NO'}.items():
        if row[k]!=v:return 'BLOCKED'
    if row['actor']!=row['target'] or row['target'] not in 'AB':return 'BLOCKED'
    l=row['target'];lane=s[l];action=row['scenario']
    if lane['revoked']:return 'BLOCKED'
    if row['witness_claim']!=witness(g,l,e,h,row['parent_claim'],row['root_claim'],action):return 'BLOCKED'
    if row['parent_claim']!=lane['head']:return 'BLOCKED'
    if action=='REVOKE_A':
        if l!='A' or lane['height']!=3 or lane['final_height']!=3 or e!=lane['epoch'] or h!=lane['height'] or row['root_claim']!=lane['root'] or s['B']['final_height']!=4:return 'BLOCKED'
        lane['revoked']=True # preserve finality and canonical head
        return 'RESEARCH_REVIEW_ONLY'
    if h!=lane['height']+1 or e<lane['epoch']:return 'BLOCKED'
    valid=False;seal=False
    if action in ('REGISTER_A','REGISTER_B') and action[-1]==l:
        valid=lane['height']==0 and h==1 and e==1 and row['root_claim']==root(g,l,1)
    elif action in ('ADVANCE_A','ADVANCE_B') and action[-1]==l:
        valid=lane['height']==1 and h==2 and e==2 and row['root_claim']==root(g,l,2)
    elif action=='FINALIZE_A' and l=='A':
        valid=lane['height']==2 and lane['final_height']==0 and e==2 and h==3 and row['root_claim']==lane['root'];seal=valid
    elif action=='ADVANCE_B_3' and l=='B':
        valid=lane['height']==2 and h==3 and e==3 and row['root_claim']==root(g,l,3)
    elif action=='FINALIZE_B' and l=='B':
        valid=lane['height']==3 and lane['final_height']==0 and h==4 and e==3 and row['root_claim']==lane['root'];seal=valid
    elif action=='ADVANCE_B_POST_REVOKE' and l=='B':
        valid=s['A']['revoked'] and lane['height']==4 and lane['final_height']==4 and h==5 and e==4 and row['root_claim']==root(g,l,4)
    if not valid:return 'BLOCKED'
    lane.update(epoch=e,root=row['root_claim'],height=h,head=row['witness_claim'])
    if seal:lane.update(final_height=h,final_head=row['witness_claim'])
    return 'RESEARCH_REVIEW_ONLY'

POLICY_SHA='6da4eb212e7a8e8ed1c297808ad6c7beaae14e6f2cb665b12c0d5d9b6c56dba8'
TRANSCRIPT_SHA='26fd97b314f458458b4f89cbdde5b55d41403c5dc5ea89c061a057b5ecf494b7'
NEW={
 'README_R0_23.md',
 'docs/NORDOI_R0_23_FINALITY_SPEC.md',
 'governance/r23_finality_equivocation_v1.tsv',
 'governance/r23_finality_witnesses_v1.tsv',
 'research/R0_23_FINALITY_BOUNDARY_DECISION.md',
 'research/fixtures/r23_finality_equivocation_v1.bin',
 'scripts/validate_r23_finality.py',
 'tests/resource_task_r23.rs',
}
def normalized(path):
    raw=(ROOT/path).read_bytes().replace(b'\r\n',b'\n')
    if b'\r' in raw:raise RuntimeError('Bare CR in '+path)
    return raw

def read_tsv(path,header):
    data=normalized(path).decode('utf-8')
    if not data.startswith('\t'.join(header)+'\n'):raise RuntimeError('Wrong TSV header: '+path)
    out=list(csv.DictReader(io.StringIO(data),delimiter='\t'))
    if not out or any(set(r)!=set(header) or None in r or any(x is None or not x for x in r.values()) for r in out):
        raise RuntimeError('Invalid TSV row: '+path)
    return out

def static():
    r22=runpy.run_path(str(ROOT/'scripts/validate_r22_witness_forks.py'),run_name='r23_parent')
    r22['static']() # inherited static checks: R0.16 through R0.22
    old=(ROOT/'research/fixtures/r22_witness_forks_v1.bin').read_bytes()
    if len(old)!=7654 or sha(old)!=R22_SHA:raise RuntimeError('Parent binary digest differs')
    gates_path='governance/r11_native_readiness_gates_v1.tsv'
    native=read_tsv(gates_path,normalized(gates_path).decode().splitlines()[0].split('\t'))
    ext_path='governance/r15_trust_policy_registry_v1.tsv'
    ext=read_tsv(ext_path,normalized(ext_path).decode().splitlines()[0].split('\t'))
    if len(native)!=12 or len(ext)!=12 or any(r['status']!='UNMET' for r in native) or any(r['status']!='ABSENT' for r in ext):
        raise RuntimeError('Frozen production gate statuses have changed')
    if [r['area'] for r in native]!=AREAS:raise RuntimeError('Certified gate order changed')
    rows=read_tsv('governance/r23_finality_equivocation_v1.tsv',HEADER)
    if len(rows)!=216 or sha(normalized('governance/r23_finality_equivocation_v1.tsv'))!=POLICY_SHA:
        raise RuntimeError('R0.23 frozen policy registry mismatch')
    binary=bytearray(b'NDR23FN1'+struct.pack('<H',216)+bytes.fromhex(R22_SHA)+bytes.fromhex(BASE))
    counts={'INVALID':0,'BLOCKED':0,'RESEARCH_REVIEW_ONLY':0}
    state=initial()
    for idx,r in enumerate(rows):
        gate=idx//18+1
        scenario=idx%18
        if scenario==0:state=initial()
        if (r['id']!=f'R23-X{idx+1:03}' or r['gate_id']!=f'R11-G{gate:02}' or r['area']!=AREAS[gate-1]
            or r['scenario']!=SCENARIOS[scenario] or r['baseline_sha']!=BASE or r['limitation']!=LIMIT):
            raise RuntimeError('Scenario order, scope or parent mismatch at '+str(idx+1))
        if r['before_state']!=state_text(state):raise RuntimeError('Wrong before state at '+str(idx+1))
        a=state['A']['height'];b=state['B']['height']
        outcome=decide(r,state,gate)
        expected='INVALID' if scenario==16 else 'RESEARCH_REVIEW_ONLY' if scenario in GOOD else 'BLOCKED'
        if outcome!=expected or outcome!=r['expected'] or r['after_state']!=state_text(state):
            raise RuntimeError('Invalid state transition at '+str(idx+1))
        counts[outcome]+=1
        binary.extend(struct.pack('<H10B',idx+1,scenario,{'INVALID':0,'BLOCKED':1,'RESEARCH_REVIEW_ONLY':2}[outcome],a,state['A']['height'],b,state['B']['height'],int(state['A']['revoked']),int(state['B']['revoked']),state['A']['final_height'],state['B']['final_height']))
        binary.extend(hashlib.sha256(('\t'.join(r[c] for c in HEADER)+'\n').encode()).digest())
    binary.extend(hashlib.sha256(binary).digest())
    gold=(ROOT/'research/fixtures/r23_finality_equivocation_v1.bin').read_bytes()
    if len(gold)!=9598 or bytes(binary)!=gold or sha(gold)!=TRANSCRIPT_SHA:raise RuntimeError('Golden transcript mismatch')
    if counts!={'INVALID':12,'BLOCKED':96,'RESEARCH_REVIEW_ONLY':108}:raise RuntimeError('Histogram mismatch')
    rust=normalized('tests/resource_task_r23.rs').decode()
    tests=re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses=read_tsv('governance/r23_finality_witnesses_v1.tsv',WITNESS_HEADER)
    if len(tests)!=48 or len(set(tests))!=48 or len(witnesses)!=48 or [w['witness'] for w in witnesses]!=tests:
        raise RuntimeError('Rust witnesses must exactly match the 48-item inventory')
    if any(w['id']!=f'R23-P{i:02}' or w['classification'] not in {'FROZEN_LINEAGE','FINALITY_EQUIVOCATION','TRUST_BOUNDARY'} or w['limitation']!=LIMIT for i,w in enumerate(witnesses,1)):
        raise RuntimeError('Rust witness classification invalid')
    for marker in [BASE,R22_SHA,POLICY_SHA,TRANSCRIPT_SHA,'fn apply(', 'fn rebuild(', 'fn normalized_lf(', 'R23_WITNESS', 'ResearchReviewOnly','final_head','final_height']:
        if marker not in rust:raise RuntimeError('Rust contract lacks '+marker)
    for forbidden in ['unsafe {','#[allow(', 'std::process::Command','ProductionAuthorized','NativeAuthorized']:
        if forbidden in rust:raise RuntimeError('Unacceptable trust or exception in Rust: '+forbidden)
    docs='\n'.join(normalized(p).decode() for p in NEW if p.endswith('.md'))
    for marker in [BASE,R22_SHA,POLICY_SHA,TRANSCRIPT_SHA,'TEST-ONLY','RESEARCH_ONLY','UNMET','ABSENT','production']:
        if marker not in docs:raise RuntimeError('Missing required statement: '+marker)
    print('R0.23 static contract: PASS')
    print('Certified input: r0.22 / '+BASE)
    print('Synthetic finality/equivocation cases: 216/216; INVALID=12, BLOCKED=96, RESEARCH_REVIEW_ONLY=108')
    print('Golden transcript: 9598 bytes; SHA-256 '+TRANSCRIPT_SHA)
    print('Rust witnesses: 48/48; two inherited SHA-256 vector tests (expected total 50)')
    print('Native gates: 12/12 UNMET; real external verifiers: 12/12 ABSENT')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; NO production authentication or native admission')

def check_git_boundary():
    def git(*args):return subprocess.run(['git',*args],cwd=ROOT,text=True,capture_output=True,check=False)
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve()!=ROOT.resolve():
        print('R0.22 Git boundary: NOT CHECKED (synthetic validation tree)')
        return
    tag=git('rev-parse','r0.22^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:raise RuntimeError('Certified r0.22 tag mismatch')
    tree=git('ls-tree','-r','--name-only','r0.22')
    if tree.returncode:raise RuntimeError('Cannot inspect frozen git tree')
    frozen=set(tree.stdout.splitlines())
    if frozen & NEW:raise RuntimeError('New path collides with certified version')
    for args in [('diff','--name-only','r0.22','--','.'),('diff','--cached','--name-only'),('ls-files','--others','--exclude-standard')]:
        r=git(*args)
        if r.returncode or set(r.stdout.splitlines())-NEW:raise RuntimeError('Forbidden modified path')
    if any(not (ROOT/p).is_file() for p in frozen):raise RuntimeError('Frozen file missing')
    if any(not (ROOT/p).is_file() for p in NEW):raise RuntimeError('Required new file missing')
    print('R0.22 Git boundary: PASS (exactly eight additive paths; zero frozen edits)')

if __name__=='__main__':
    static()
    check_git_boundary()
    print('Rust compilation, Clippy, release gate, GitHub CI and external audits: NOT RUN by Python validator')
