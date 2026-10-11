#!/usr/bin/env python3
"""R0.20: deterministic offline mock trust-root lifecycle boundary, never production admission."""
import csv
import hashlib
import io
from pathlib import Path
import re
import runpy
import struct
import subprocess

ROOT = Path(__file__).resolve().parent.parent
BASE = '834c20d68e86f0ed6ab14141e0f9f9e1df241901'
EVIDENCE_SHA = 'ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33'
POLICY_SHA = '1a296c9adb4b828fd504169a3579b0c8addbdbc7add1c8b0fafeedd6352f5e2e'
TRANSCRIPT_SHA = '2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817'
HEADER = 'id gate_id area scenario baseline_sha evidence_sha256 root_claim epoch_claim scope signature verifier ci review native before_epoch after_epoch before_root after_root before_revoked after_revoked expected limitation'.split()
WITNESS_HEADER = ['id', 'witness', 'classification', 'limitation']
SCENARIOS = [
    'REGISTER_MOCK', 'ROTATE_MOCK', 'STALE_EPOCH', 'FOREIGN_SCOPE',
    'FORGED_SIGNATURE', 'FORGED_VERIFIER', 'TAMPER_LINEAGE', 'REVOKE_MOCK',
    'REPLAY_REVOKED', 'ROLLBACK_EPOCH', 'NATIVE_ESCALATION',
]
NEW = {
    'README_R0_20.md',
    'docs/NORDOI_R0_20_TRUST_ROOT_LIFECYCLE_SPEC.md',
    'governance/r20_trust_root_lifecycle_v1.tsv',
    'governance/r20_root_witnesses_v1.tsv',
    'research/R0_20_TRUST_ROOT_BOUNDARY_DECISION.md',
    'research/fixtures/r20_trust_root_transitions_v1.bin',
    'scripts/validate_r20_trust_roots.py',
    'tests/resource_task_r20.rs',
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def normalized(path):
    data = (ROOT / path).read_bytes().replace(b'\r\n', b'\n')
    if b'\r' in data:
        raise RuntimeError('Bare CR in '+path)
    return data


def read_tsv(path, names):
    data = normalized(path).decode('utf-8')
    if not data.startswith('\t'.join(names)+'\n'):
        raise RuntimeError('Header mismatch: '+path)
    parsed = list(csv.DictReader(io.StringIO(data), delimiter='\t'))
    if not parsed or any(len(row) != len(names) or None in row or any(not v for v in row.values()) for row in parsed):
        raise RuntimeError('Malformed TSV: '+path)
    return parsed


def initial():
    return {'epoch': 0, 'root': 'NONE', 'revoked': False}


def root_name(gate, suffix):
    return f'MOCK-ROOT-{gate:02}-{suffix}'


def decide(row, state, gate):
    if row['evidence_sha256'] != EVIDENCE_SHA or row['baseline_sha'] != BASE:
        return 'INVALID'
    try:
        epoch = int(row['epoch_claim'])
    except ValueError:
        return 'INVALID'
    if row['scope'] != 'RESEARCH_ONLY' or row['signature'] != 'NONE' or row['verifier'] != 'NONE' or row['ci'] != 'YES' or row['review'] != 'YES' or row['native'] != 'NO':
        return 'BLOCKED'
    action = row['scenario']
    if action == 'REGISTER_MOCK' and state == initial() and row['root_claim'] == root_name(gate, 'A') and epoch == 1:
        state.update(epoch=1, root=row['root_claim'])
        return 'RESEARCH_REVIEW_ONLY'
    if action == 'ROTATE_MOCK' and state == {'epoch':1, 'root':root_name(gate,'A'), 'revoked':False} and row['root_claim'] == root_name(gate,'B') and epoch == 2:
        state.update(epoch=2, root=row['root_claim'])
        return 'RESEARCH_REVIEW_ONLY'
    if action == 'REVOKE_MOCK' and state == {'epoch':2, 'root':root_name(gate,'B'), 'revoked':False} and row['root_claim'] == state['root'] and epoch == 2:
        state['revoked'] = True
        return 'RESEARCH_REVIEW_ONLY'
    return 'BLOCKED'


def static():
    # Execute ONLY the already certified R0.19 static checks. The R0.19 Git
    # boundary is intentionally not invoked: it predates the new R0.20 paths.
    r19 = runpy.run_path(str(ROOT / 'scripts/validate_r19_admission.py'), run_name='r20_frozen_reference')
    reference_gates = read_tsv('governance/r11_native_readiness_gates_v1.tsv',
                              normalized('governance/r11_native_readiness_gates_v1.tsv').decode().splitlines()[0].split('\t'))
    areas = [gate['area'] for gate in reference_gates]
    if len(areas) != 12 or any(gate['status'] != 'UNMET' for gate in reference_gates):
        raise RuntimeError('Frozen 12-gate native readiness mismatch')
    r19['check_static'].__globals__['areas'] = areas
    r19['check_static']()
    if sha((ROOT/'research/fixtures/r19_admission_attempts_v1.bin').read_bytes()) != EVIDENCE_SHA:
        raise RuntimeError('R0.19 certified binary evidence root mismatch')
    registry = read_tsv('governance/r20_trust_root_lifecycle_v1.tsv', HEADER)
    if sha(normalized('governance/r20_trust_root_lifecycle_v1.tsv')) != POLICY_SHA or len(registry) != 132:
        raise RuntimeError('R0.20 registry size/hash mismatch')
    output = bytearray(b'NDR20TR1' + struct.pack('<H',132))
    counts = {'INVALID':0, 'BLOCKED':0, 'RESEARCH_REVIEW_ONLY':0}
    state = initial()
    for idx, row in enumerate(registry):
        gate = idx // 11 + 1
        scenario_index = idx % 11
        scenario = SCENARIOS[scenario_index]
        if scenario_index == 0:
            state = initial()
        if row['id'] != f'R20-L{idx+1:03}' or row['gate_id'] != f'R11-G{gate:02}' or row['area'] != areas[gate-1] or row['scenario'] != scenario or row['baseline_sha'] != BASE or row['limitation'] != 'SYNTHETIC_ONLY_NO_REAL_SIGNATURE_VERIFIER_OR_NATIVE_AUTHORITY':
            raise RuntimeError('Wrong canonical row identity: '+str(idx))
        before = dict(state)
        if row['before_epoch'] != str(before['epoch']) or row['before_root'] != before['root'] or row['before_revoked'] != ('YES' if before['revoked'] else 'NO'):
            raise RuntimeError('Incorrect before-state: '+str(idx))
        decision = decide(row, state, gate)
        canonical = 'RESEARCH_REVIEW_ONLY' if scenario_index in (0, 1, 7) else ('INVALID' if scenario_index == 6 else 'BLOCKED')
        if decision != canonical or row['expected'] != decision or row['after_epoch'] != str(state['epoch']) or row['after_root'] != state['root'] or row['after_revoked'] != ('YES' if state['revoked'] else 'NO'):
            raise RuntimeError('Decision or state mismatch: '+str(idx))
        counts[decision] += 1
        fields = '\t'.join(row[name] for name in HEADER)+'\n'
        output.extend(struct.pack('<HBBBBB',idx+1,scenario_index,{'INVALID':0,'BLOCKED':1,'RESEARCH_REVIEW_ONLY':2}[decision],before['epoch'],state['epoch'],int(state['revoked'])))
        output.extend(hashlib.sha256(fields.encode()).digest())
    output.extend(hashlib.sha256(output).digest())
    if counts != {'INVALID':12,'BLOCKED':84,'RESEARCH_REVIEW_ONLY':36}:
        raise RuntimeError('Decision distribution drift')
    golden = (ROOT/'research/fixtures/r20_trust_root_transitions_v1.bin').read_bytes()
    if len(output) != 5190 or golden != output or sha(golden) != TRANSCRIPT_SHA:
        raise RuntimeError('Golden offline transcript mismatch')
    rust = (ROOT/'tests/resource_task_r20.rs').read_text()
    tests = re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses = read_tsv('governance/r20_root_witnesses_v1.tsv', WITNESS_HEADER)
    if len(tests) != 36 or len(set(tests)) != 36 or len(witnesses) != len(tests) or [w['witness'] for w in witnesses] != tests:
        raise RuntimeError('Rust witness mapping or count mismatch')
    for idx, row in enumerate(witnesses, 1):
        if row['id'] != f'R20-P{idx:02}' or row['classification'] not in {'FROZEN_LINEAGE','LIFECYCLE_TRANSITION','TRUST_BOUNDARY'} or len(row['limitation']) < 90:
            raise RuntimeError('Governance witness schema mismatch')
    for token in [BASE,EVIDENCE_SHA,POLICY_SHA,TRANSCRIPT_SHA,'fn golden_bytes(', 'fn apply(', 'fn normalized_lf(', 'R20_WITNESS','ResearchReviewOnly']:
        if token not in rust:
            raise RuntimeError('Missing Rust contract symbol: '+token)
    for banned in ['unsafe {', '#[allow(', 'std::process::Command', 'ProductionAuthorized', 'NativeAuthorized']:
        if banned in rust:
            raise RuntimeError('Unsafe authority or warning bypass in research test: '+banned)
    docs = '\n'.join((ROOT/p).read_text() for p in NEW if p.endswith('.md'))
    for token in [BASE,POLICY_SHA,TRANSCRIPT_SHA,'TEST-ONLY','RESEARCH_ONLY','UNMET','ABSENT','NOT']:
        if token not in docs:
            raise RuntimeError('Missing limitation in documentation: '+token)
    print('R0.20 static contract: PASS')
    print('Certified input: r0.19 / '+BASE)
    print('Trust-root synthetic lifecycle cases: 132/132; INVALID=12, BLOCKED=84, RESEARCH_REVIEW_ONLY=36')
    print('Golden transcript: 5190 bytes; SHA-256 '+TRANSCRIPT_SHA)
    print('Rust witnesses: 36/36; plus two inherited SHA-256 vector tests (expected 38 total)')
    print('Historical native gates: 12/12 UNMET; real external verifiers: 12/12 ABSENT')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; no real keys, signature verification, native authority')


def boundary():
    def git(*args):
        return subprocess.run(['git',*args], cwd=ROOT, text=True, capture_output=True, check=False)
    top=git('rev-parse','--show-toplevel')
    if top.returncode or Path(top.stdout.strip()).resolve() != ROOT.resolve():
        print('R0.19 Git boundary: NOT CHECKED (reconstructed non-Git tree)')
        return
    tag=git('rev-parse','r0.19^{commit}')
    if tag.returncode or tag.stdout.strip()!=BASE:
        raise RuntimeError('Certified tag r0.19 does not bind exact commit')
    historic=git('ls-tree','-r','--name-only','r0.19')
    if historic.returncode:raise RuntimeError('Cannot inspect certified history')
    frozen=set(historic.stdout.splitlines())
    if frozen & NEW:raise RuntimeError('R0.20 overwrites a certified path')
    diff=git('diff','--name-only','r0.19','--','.')
    if diff.returncode or set(diff.stdout.splitlines())-NEW:raise RuntimeError('Unexpected changed historic path')
    staged=git('diff','--cached','--name-only')
    if staged.returncode or set(staged.stdout.splitlines())-NEW:raise RuntimeError('Unexpected staged historic path')
    new=git('ls-files','--others','--exclude-standard')
    if new.returncode or set(new.stdout.splitlines())-NEW:raise RuntimeError('Unexpected untracked path')
    if any(not (ROOT/p).is_file() for p in frozen):raise RuntimeError('Historic tracked file removed')
    print('R0.19 Git boundary: PASS (exactly eight additive paths, no frozen history altered)')

if __name__=='__main__':
    static()
    boundary()
    print('Rust compilation, Clippy, full Release Gate, GitHub CI and real external audit: NOT RUN by this validator')
