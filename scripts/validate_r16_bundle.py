#!/usr/bin/env python3
"""R0.16 bounded offline research-byte validator. No external trust authorization."""
from __future__ import annotations

import csv
import hashlib
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = '3158ee20a894461df5402335b7db5509db9abd02'
PACK_SHA = 'f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393'
NEW = {
    'README_R0_16.md',
    'docs/NORDOI_R0_16_OFFLINE_BUNDLE_SPEC.md',
    'governance/r16_offline_bundle_manifest_v1.tsv',
    'governance/r16_bundle_witnesses_v1.tsv',
    'research/R0_16_OFFLINE_BYTE_BOUNDARY_DECISION.md',
    'research/fixtures/r16_synthetic_bundle_v1.bin',
    'scripts/validate_r16_bundle.py',
    'tests/resource_task_r16.rs',
}
AREAS = (
    'LIFECYCLE_SEMANTICS', 'EXECUTOR_SCHEDULING', 'MEMORY_OWNERSHIP',
    'CAPABILITY_AUTHORITY', 'CANCELLATION_CLEANUP', 'EFFECT_IO_BOUNDARY',
    'CRASH_RECOVERY', 'ADVERSARIAL_TESTING', 'CROSS_PLATFORM',
    'COMPILER_NAIR_BINDING', 'RESOURCE_LIMITS', 'INDEPENDENT_REVIEW',
)
MANIFEST_HEADER = (
    'id', 'gate_id', 'evidence_id', 'integrity_id', 'authentication_id', 'verification_id',
    'area', 'baseline_sha', 'admitted_status', 'scope', 'offset', 'length',
    'fixture_sha256', 'mock_producer', 'signature_ref', 'trust_root', 'limitation',
)


def tsv(path: str, header: tuple[str,...]) -> list[dict[str,str]]:
    with (ROOT / path).open(encoding='utf-8',newline='') as f:
        reader = csv.DictReader(f, delimiter='\t')
        if tuple(reader.fieldnames or ()) != header:
            raise RuntimeError(f'Invalid tabular schema: {path}')
        rows = list(reader)
    if any(set(row) != set(header) or any(v is None or not v.strip() or '\r' in v for v in row.values()) for row in rows):
        raise RuntimeError(f'Invalid tabular records: {path}')
    return rows


def check_legacy() -> None:
    g = tsv('governance/r11_native_readiness_gates_v1.tsv', ('id','area','status','evidence_scope','acceptance_criterion','limitation'))
    e = tsv('governance/r12_evidence_ledger_v1.tsv', ('id','gate_id','area','baseline_sha','status','evidence_scope','artifact_ref','reviewer_ref','limitation'))
    i = tsv('governance/r13_integrity_chain_v1.tsv', ('id','gate_id','evidence_id','area','baseline_sha','status','scope','artifact_sha256','previous_chain_sha256','chain_sha256','producer_ref','reviewer_ref','signature_ref','trust_root','revocation','limitation'))
    a = tsv('governance/r14_authentication_registry_v1.tsv', ('id','gate_id','evidence_id','integrity_id','area','baseline_sha','status','scope','producer_key','reviewer_key','trust_root','authentication','revocation','limitation'))
    v = tsv('governance/r15_trust_policy_registry_v1.tsv', ('id','gate_id','evidence_id','integrity_id','authentication_id','area','baseline_sha','status','scope','policy','verifier_ref','trust_root','limitation'))
    if any(len(r) != 12 for r in (g,e,i,a,v)):
        raise RuntimeError('Frozen chain must contain exactly 12 records per phase')
    for n, (rg,re_,ri,ra,rv) in enumerate(zip(g,e,i,a,v),1):
        area = AREAS[n-1]
        if not (
            rg['id'] == re_['gate_id'] == ri['gate_id'] == ra['gate_id'] == rv['gate_id'] == f'R11-G{n:02}'
            and re_['id'] == ri['evidence_id'] == ra['evidence_id'] == rv['evidence_id'] == f'R12-E{n:02}'
            and ri['id'] == ra['integrity_id'] == rv['integrity_id'] == f'R13-L{n:02}'
            and ra['id'] == rv['authentication_id'] == f'R14-A{n:02}'
            and rv['id'] == f'R15-V{n:02}'
            and all(record['area'] == area for record in (rg,re_,ri,ra,rv))
            and rg['status'] == 'UNMET' and rg['evidence_scope'] == 'RESEARCH_ONLY'
            and re_['status'] == 'ABSENT' and re_['evidence_scope'] == 'RESEARCH_ONLY'
            and re_['baseline_sha'] == 'd61487bd2392890b3aaaa6ef33fce4371cfb6850'
            and re_['artifact_ref'] == re_['reviewer_ref'] == 'NONE'
            and ri['status'] == 'ABSENT' and ri['scope'] == 'RESEARCH_ONLY'
            and ri['baseline_sha'] == '851d978c36f4c12fc16c115e89a7774e33ce361b'
            and all(ri[k] == 'NONE' for k in ('artifact_sha256','previous_chain_sha256','chain_sha256','producer_ref','reviewer_ref','signature_ref','trust_root','revocation'))
            and ra['status'] == 'ABSENT' and ra['scope'] == 'RESEARCH_ONLY'
            and ra['baseline_sha'] == '35e048dd83f2c072a6c454494ac099906156dc38'
            and all(ra[k] == 'NONE' for k in ('producer_key','reviewer_key','trust_root','authentication','revocation'))
            and rv['status'] == 'ABSENT' and rv['scope'] == 'RESEARCH_ONLY'
            and rv['baseline_sha'] == '86fc90a79f5d0cef83c730bcd7c02175307fffd3'
            and rv['policy'] == 'DUAL_SYNTHETIC_REVIEW' and rv['verifier_ref'] == rv['trust_root'] == 'NONE'
        ):
            raise RuntimeError(f'Frozen historical lineage mismatch at {n}')


def static() -> None:
    for path in NEW:
        target = ROOT/path
        if not target.is_file() or target.is_symlink():
            raise RuntimeError(f'Missing R0.16 path or symlink: {path}')
    check_legacy()
    manifest = tsv('governance/r16_offline_bundle_manifest_v1.tsv', MANIFEST_HEADER)
    blob = (ROOT/'research/fixtures/r16_synthetic_bundle_v1.bin').read_bytes()
    if hashlib.sha256(blob).hexdigest() != PACK_SHA:
        raise RuntimeError('Synthetic byte fixture root SHA-256 mismatch')
    if len(blob) < 10 or blob[:8] != b'NDR16PK1' or int.from_bytes(blob[8:10],'little') != 12 or len(manifest) != 12:
        raise RuntimeError('Invalid binary header/count or manifest size')
    pos=10
    digest_set=set()
    for idx,row in enumerate(manifest,1):
        if pos+2 > len(blob):
            raise RuntimeError('Truncated record length')
        size=int.from_bytes(blob[pos:pos+2], 'little')
        pos += 2
        if not 1 <= size <= 256 or pos+size > len(blob):
            raise RuntimeError('Invalid bounded payload size')
        payload=blob[pos:pos+size]
        digest=hashlib.sha256(payload).hexdigest()
        if not (
            row['id'] == f'R16-B{idx:02}'
            and row['gate_id'] == f'R11-G{idx:02}'
            and row['evidence_id'] == f'R12-E{idx:02}'
            and row['integrity_id'] == f'R13-L{idx:02}'
            and row['authentication_id'] == f'R14-A{idx:02}'
            and row['verification_id'] == f'R15-V{idx:02}'
            and row['area'] == AREAS[idx-1]
            and row['baseline_sha'] == BASE
            and row['admitted_status'] == 'ABSENT'
            and row['scope'] == 'RESEARCH_ONLY'
            and row['offset'] == str(pos)
            and row['length'] == str(size)
            and row['fixture_sha256'] == digest
            and row['mock_producer'] == f'MOCK-PRODUCER-{idx:02}'
            and row['signature_ref'] == row['trust_root'] == 'NONE'
            and len(row['limitation']) >= 65
            and digest not in digest_set
        ):
            raise RuntimeError(f'Malformed research bundle admission row {idx}')
        digest_set.add(digest)
        pos += size
    if pos != len(blob) or not b'\xff\x80\r\n' in blob or not b'\x00' in blob:
        raise RuntimeError('Trailing byte, missing non-UTF-8 or missing NUL test content')
    src=(ROOT/'tests/resource_task_r16.rs').read_text(encoding='utf-8')
    tests=re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',src)
    witnesses=tsv('governance/r16_bundle_witnesses_v1.tsv', ('id','witness','classification','limitation'))
    if len(tests) != 44 or len(set(tests)) != 44 or len(witnesses) != 44 or [w['witness'] for w in witnesses] != tests:
        raise RuntimeError('R0.16 witness/test mismatch')
    allowed={'FROZEN_BASELINE','BUNDLE_LAYOUT','FAIL_CLOSED','TRUST_BOUNDARY'}
    if any(w['id'] != f'R16-P{idx:02}' or w['classification'] not in allowed or len(w['limitation']) < 65 for idx,w in enumerate(witnesses,1)):
        raise RuntimeError('Malformed R0.16 witness metadata')
    mandatory=['#[path = "../src/effect_audit/hash.rs"]',
               'include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin")',
               'include_str!("../governance/r15_trust_policy_registry_v1.tsv")',
               'fn decode_pack(', 'fn assess(', 'fn frozen_contract(',
               'R16_WITNESS version=1', 'ResearchReviewOnly', PACK_SHA]
    if any(x not in src for x in mandatory):
        raise RuntimeError('Mandatory R0.16 binary/hash/frozen guard missing')
    if any(x in src for x in ('unsafe {', 'std::thread::spawn(', 'tokio::spawn(', '#[ignore]', '#[should_panic]', 'ProductionAuthorized', 'NativeAuthorized')):
        raise RuntimeError('Forbidden R0.16 native capability test or test skipping')
    for rel in ('src/lib.rs','Cargo.toml'):
        p=ROOT/rel
        if p.exists() and ('resource_task_r16' in p.read_text(encoding='utf-8') or 'r16_offline_bundle_manifest' in p.read_text(encoding='utf-8')):
            raise RuntimeError('Attempted R0.16 production export')
    docs='\n'.join((ROOT/p).read_text(encoding='utf-8') for p in NEW if p.endswith('.md'))
    if any(word not in docs for word in ('r0.15',BASE,'TEST-ONLY','RESEARCH_ONLY','UNMET','ABSENT','SHA-256','NOT','44','46')):
        raise RuntimeError('Mandatory limitations omitted from R0.16 documentation')
    print('R0.16 static contract: PASS')
    print(f'Certified baseline: r0.15 / {BASE}')
    print('Native readiness: 12/12 UNMET; real external evidence R0.12-R0.16: 12/12 ABSENT')
    print('Offline synthetic binary bundle: 12/12 bounded payloads; 44/44 witnesses')
    print(f'Fixture SHA-256: {PACK_SHA}')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; real signatures and external verification NOT performed')


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(['git',*args],cwd=ROOT,capture_output=True,text=True,check=False)


def git_boundary() -> None:
    g=git('rev-parse','--show-toplevel')
    if g.returncode or Path(g.stdout.strip()).resolve() != ROOT.resolve():
        print('R0.15 Git boundary: NOT CHECKED (reconstructed repository)')
        return
    tag=git('rev-parse','r0.15^{commit}')
    if tag.returncode or tag.stdout.strip() != BASE:
        raise RuntimeError('Certified tag r0.15 is missing or mismatched')
    tree=git('ls-tree','-r','--name-only','r0.15')
    if tree.returncode or set(tree.stdout.splitlines()) & NEW:
        raise RuntimeError('R0.16 overlaps r0.15 certified files')
    tracked=git('diff','--name-only','r0.15','--','.')
    if tracked.returncode or set(tracked.stdout.splitlines())-NEW:
        raise RuntimeError('Frozen r0.15 path changed')
    untracked=git('ls-files','--others','--exclude-standard')
    if untracked.returncode or set(untracked.stdout.splitlines()) - NEW:
        raise RuntimeError('Unexpected untracked path outside R0.16')
    if any(not (ROOT/name).is_file() for name in tree.stdout.splitlines()):
        raise RuntimeError('Certified r0.15 file was deleted')
    print('R0.15 Git boundary: PASS (eight additive paths; zero frozen changes)')


if __name__ == '__main__':
    static()
    git_boundary()
    print('Rust tests, Clippy, full Release Gate and GitHub CI: NOT RUN by this validator')
