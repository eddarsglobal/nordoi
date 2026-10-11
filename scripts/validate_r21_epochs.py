#!/usr/bin/env python3
"""R0.21 deterministic cross-root epoch isolation, mock data ONLY; no real authority."""
from __future__ import annotations
import csv
import hashlib
import io
from pathlib import Path
import re
import runpy
import struct
import subprocess

ROOT = Path(__file__).resolve().parent.parent
BASE = 'bd54ef97523fc9f415ce0d6eaa41830b7105b7bf'
R20_SHA = '2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817'
POLICY_SHA = '179b9e332d9562d90df7bdaec8c46297b9a467cfd04b728c47750b0aec7a9a57'
TRANSCRIPT_SHA = 'cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba'
LIMITATION = 'SYNTHETIC_ONLY_NO_REAL_SIGNATURE_EXTERNAL_VERIFIER_OR_NATIVE_AUTHORITY'
HEADER = 'id gate_id area scenario baseline_sha evidence_sha256 actor target root_claim epoch_claim producer scope signature verifier ci review native before_state after_state expected limitation'.split()
WITNESS_HEADER = ['id', 'witness', 'classification', 'limitation']
SCENARIOS = [
    'REGISTER_A', 'REGISTER_B', 'ROTATE_A', 'CROSS_ROOT_WRITE', 'STALE_A',
    'ROTATE_B', 'EPOCH_COLLISION', 'REVOKE_A', 'RESURRECT_A', 'REPLAY_A',
    'B_ROTATES_AFTER_A_REVOKED', 'TAMPER_LINEAGE', 'NATIVE_ESCALATION',
]
NEW = {
    'README_R0_21.md',
    'docs/NORDOI_R0_21_CROSS_ROOT_EPOCH_SPEC.md',
    'governance/r21_cross_root_epochs_v1.tsv',
    'governance/r21_cross_root_witnesses_v1.tsv',
    'research/R0_21_CROSS_ROOT_BOUNDARY_DECISION.md',
    'research/fixtures/r21_cross_root_transcript_v1.bin',
    'scripts/validate_r21_epochs.py',
    'tests/resource_task_r21.rs',
}

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def normalized(path: str) -> bytes:
    data = (ROOT / path).read_bytes().replace(b'\r\n', b'\n')
    if b'\r' in data:
        raise RuntimeError('Unexpected bare CR in ' + path)
    return data

def read_tsv(path: str, header: list[str]) -> list[dict[str,str]]:
    data = normalized(path).decode('utf-8')
    if not data.startswith('\t'.join(header) + '\n'):
        raise RuntimeError('Unexpected TSV schema: ' + path)
    reader = list(csv.DictReader(io.StringIO(data), delimiter='\t'))
    if not reader or any(set(row) != set(header) or None in row or
                         any(value is None or not value for value in row.values()) for row in reader):
        raise RuntimeError('Corrupted or incomplete TSV: '+path)
    return reader

def initial():
    return {'A': {'epoch':0, 'root':'NONE', 'revoked':False},
            'B': {'epoch':0, 'root':'NONE', 'revoked':False}}

def state_text(s):
    return '|'.join(f'{label}={s[label]["epoch"]}:{s[label]["root"]}:{int(s[label]["revoked"])}' for label in 'AB')

def root_name(gate, lane, epoch):
    return f'MOCK-G{gate:02}-{lane}-E{epoch}'

def decide(row, state, gate):
    if row['baseline_sha'] != BASE or row['evidence_sha256'] != R20_SHA:
        return 'INVALID'
    try:
        epoch = int(row['epoch_claim'])
    except ValueError:
        return 'INVALID'
    if not 0 <= epoch <= 255:
        return 'INVALID'
    needed = {'producer':'MOCK_R21', 'scope':'RESEARCH_ONLY', 'signature':'NONE',
              'verifier':'NONE', 'ci':'YES', 'review':'YES', 'native':'NO'}
    if any(row[k] != value for k,value in needed.items()):
        return 'BLOCKED'
    actor,target = row['actor'],row['target']
    if actor not in ('A','B') or target not in ('A','B') or actor != target:
        return 'BLOCKED'
    lane = state[target]
    if lane['revoked']:
        return 'BLOCKED'
    action = row['scenario']
    if action in ('REGISTER_A','REGISTER_B') and action[-1] == target and lane == {'epoch':0,'root':'NONE','revoked':False} and epoch == 1 and row['root_claim'] == root_name(gate,target,1):
        lane.update(epoch=1,root=row['root_claim'])
        return 'RESEARCH_REVIEW_ONLY'
    if action in ('ROTATE_A','ROTATE_B') and action[-1] == target and lane == {'epoch':1,'root':root_name(gate,target,1),'revoked':False} and epoch == 2 and row['root_claim'] == root_name(gate,target,2):
        lane.update(epoch=2,root=row['root_claim'])
        return 'RESEARCH_REVIEW_ONLY'
    if action == 'REVOKE_A' and target == 'A' and lane == {'epoch':2,'root':root_name(gate,'A',2),'revoked':False} and epoch == 2 and row['root_claim'] == lane['root']:
        lane['revoked'] = True
        return 'RESEARCH_REVIEW_ONLY'
    if action == 'B_ROTATES_AFTER_A_REVOKED' and target == 'B' and state['A']['revoked'] and lane == {'epoch':2,'root':root_name(gate,'B',2),'revoked':False} and epoch == 3 and row['root_claim'] == root_name(gate,'B',3):
        lane.update(epoch=3,root=row['root_claim'])
        return 'RESEARCH_REVIEW_ONLY'
    return 'BLOCKED'

def run_static():
    # Historical r20 checks run unchanged; their old Git boundary must not be rerun.
    r20 = runpy.run_path(str(ROOT/'scripts/validate_r20_trust_roots.py'),run_name='r21_frozen_reference')
    r20['static']()
    evidence = (ROOT/'research/fixtures/r20_trust_root_transitions_v1.bin').read_bytes()
    if len(evidence) != 5190 or sha(evidence) != R20_SHA:
        raise RuntimeError('Certified R0.20 input transcript mismatch')
    gates_file = 'governance/r11_native_readiness_gates_v1.tsv'
    gates = read_tsv(gates_file,normalized(gates_file).decode().splitlines()[0].split('\t'))
    areas = [row['area'] for row in gates]
    if len(areas) != 12 or any(row['status'] != 'UNMET' for row in gates):
        raise RuntimeError('Historical native gates are not 12/12 UNMET')
    ext_file = 'governance/r15_trust_policy_registry_v1.tsv'
    ext = read_tsv(ext_file,normalized(ext_file).decode().splitlines()[0].split('\t'))
    if len(ext) != 12 or any(row['status'] != 'ABSENT' for row in ext):
        raise RuntimeError('Historical real verifiers are not 12/12 ABSENT')
    records = read_tsv('governance/r21_cross_root_epochs_v1.tsv',HEADER)
    if len(records) != 156 or sha(normalized('governance/r21_cross_root_epochs_v1.tsv')) != POLICY_SHA:
        raise RuntimeError('Canonical R0.21 registry size or hash differs')
    result = bytearray(b'NDR21XC1' + struct.pack('<H',156) + bytes.fromhex(R20_SHA) + bytes.fromhex(BASE))
    counts = {'INVALID':0,'BLOCKED':0,'RESEARCH_REVIEW_ONLY':0}
    state = initial()
    for idx,row in enumerate(records):
        gate=idx//13+1
        scenario_index=idx%13
        if scenario_index == 0:state=initial()
        if row['id']!=f'R21-X{idx+1:03}' or row['gate_id'] != f'R11-G{gate:02}' or row['area'] != areas[gate-1] or row['scenario'] != SCENARIOS[scenario_index] or row['baseline_sha'] != BASE or row['limitation'] != LIMITATION:
            raise RuntimeError('Incorrect registry ordering or lineage at row '+str(idx+1))
        before=state_text(state)
        if row['before_state']!=before:
            raise RuntimeError('Incorrect before-state at row '+str(idx+1))
        a_before=state['A']['epoch']; b_before=state['B']['epoch']
        verdict=decide(row,state,gate)
        expected='INVALID' if scenario_index==11 else 'RESEARCH_REVIEW_ONLY' if scenario_index in (0,1,2,5,7,10) else 'BLOCKED'
        if verdict != expected or verdict != row['expected'] or row['after_state'] != state_text(state):
            raise RuntimeError('Decision/isolated state mismatch at row '+str(idx+1))
        counts[verdict]+=1
        result.extend(struct.pack('<HBBBBBBBB',idx+1,scenario_index,
               {'INVALID':0,'BLOCKED':1,'RESEARCH_REVIEW_ONLY':2}[verdict],
               a_before,state['A']['epoch'],b_before,state['B']['epoch'],
               int(state['A']['revoked']),int(state['B']['revoked'])))
        result.extend(hashlib.sha256(('\t'.join(row[field] for field in HEADER)+'\n').encode()).digest())
    result.extend(hashlib.sha256(result).digest())
    if counts != {'INVALID':12,'BLOCKED':72,'RESEARCH_REVIEW_ONLY':72}:
        raise RuntimeError('Unexpected admission histogram')
    golden = (ROOT/'research/fixtures/r21_cross_root_transcript_v1.bin').read_bytes()
    if len(result)!=6646 or result != golden or sha(golden) != TRANSCRIPT_SHA:
        raise RuntimeError('Golden transcript does not reproduce R0.21 decisions')
    rust = (ROOT/'tests/resource_task_r21.rs').read_text(encoding='utf8')
    tests = re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses = read_tsv('governance/r21_cross_root_witnesses_v1.tsv',WITNESS_HEADER)
    if len(tests)!=48 or len(set(tests))!=48 or len(witnesses)!=48 or [w['witness'] for w in witnesses]!=tests:
        raise RuntimeError('R0.21 Rust witness list not 48 unique exact tests')
    for idx,row in enumerate(witnesses,1):
        if row['id']!=f'R21-P{idx:02}' or row['classification'] not in {'FROZEN_LINEAGE','EPOCH_REPLAY','CROSS_ROOT_ISOLATION','TRUST_BOUNDARY'} or len(row['limitation']) < 90:
            raise RuntimeError('Witness classification is incomplete')
    for symbol in [BASE,R20_SHA,POLICY_SHA,TRANSCRIPT_SHA,'fn rebuild(', 'fn apply(', 'fn normalized_lf(', 'R21_WITNESS','ResearchReviewOnly']:
        if symbol not in rust:raise RuntimeError('Missing essential Rust contract '+symbol)
    for banned in ['unsafe {','#[allow(', 'std::process::Command','ProductionAuthorized','NativeAuthorized']:
        if banned in rust:raise RuntimeError('Research scope violation: '+banned)
    docs = '\n'.join((ROOT/path).read_text() for path in NEW if path.endswith('.md'))
    for word in [BASE,POLICY_SHA,TRANSCRIPT_SHA,'TEST-ONLY','RESEARCH_ONLY','UNMET','ABSENT','NOT']:
        if word not in docs:raise RuntimeError('Missing limitation or hash in docs '+word)
    print('R0.21 static contract: PASS')
    print('Certified baseline: r0.20 / '+BASE)
    print('Synthetic cross-root epochs: 156/156; INVALID=12, BLOCKED=72, RESEARCH_REVIEW_ONLY=72')
    print('Two isolated mock lanes A/B per gate; revocation and replay deny by default')
    print('Golden binary transcript: 6646 bytes; SHA-256 '+TRANSCRIPT_SHA)
    print('Rust witnesses: 48/48; two inherited SHA-256 tests (expected total 50)')
    print('Native readiness 12/12 UNMET; real external verifiers 12/12 ABSENT')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; no real keys, signatures, or native authority')

def check_git_boundary():
    def git(*args):
        return subprocess.run(['git',*args],cwd=ROOT,text=True,capture_output=True,check=False)
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve() != ROOT.resolve():
        print('R0.20 Git boundary: NOT CHECKED (reconstructed non-Git tree)')
        return
    tag=git('rev-parse','r0.20^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:
        raise RuntimeError('r0.20 tagged commit differs from frozen certified baseline')
    historic=git('ls-tree','-r','--name-only','r0.20')
    if historic.returncode:raise RuntimeError('Cannot enumerate frozen Git tree')
    frozen=set(historic.stdout.splitlines())
    if frozen & NEW:raise RuntimeError('R0.21 would replace a frozen certified path')
    for action in [('diff','--name-only','r0.20','--','.'),('diff','--cached','--name-only'),('ls-files','--others','--exclude-standard')]:
        result=git(*action)
        if result.returncode or set(result.stdout.splitlines())-NEW:
            raise RuntimeError('Unexpected modified/staged/untracked file outside eight R0.21 paths')
    if any(not (ROOT/path).is_file() for path in frozen):
        raise RuntimeError('A historical certified file is missing')
    if any(not (ROOT/path).is_file() for path in NEW):
        raise RuntimeError('A required additive R0.21 file is missing')
    print('R0.20 Git boundary: PASS (exactly eight additive paths, zero historical edits)')

if __name__=='__main__':
    run_static()
    check_git_boundary()
    print('Rust compilation, Clippy, local Release Gate, GitHub CI and independent external audit: NOT RUN by this validator')
