#!/usr/bin/env python3
"""R0.17 reproducible local replay. Synthetic bytes; absolutely no native authority."""
from __future__ import annotations

import csv
import hashlib
import runpy
import re
import struct
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = 'a5d7e9669a9b6d4be6a47bed17756946df24c9bb'
PACK_SHA = 'f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393'
TRANSCRIPT_SHA = '1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab'
NEW = {
    'README_R0_17.md',
    'docs/NORDOI_R0_17_INDEPENDENT_REPLAY_SPEC.md',
    'governance/r17_replay_registry_v1.tsv',
    'governance/r17_replay_witnesses_v1.tsv',
    'research/R0_17_REPLAY_BOUNDARY_DECISION.md',
    'research/fixtures/r17_independent_replay_transcript_v1.bin',
    'scripts/validate_r17_replay.py',
    'tests/resource_task_r17.rs',
}
AREAS = (
    'LIFECYCLE_SEMANTICS', 'EXECUTOR_SCHEDULING', 'MEMORY_OWNERSHIP',
    'CAPABILITY_AUTHORITY', 'CANCELLATION_CLEANUP', 'EFFECT_IO_BOUNDARY',
    'CRASH_RECOVERY', 'ADVERSARIAL_TESTING', 'CROSS_PLATFORM',
    'COMPILER_NAIR_BINDING', 'RESOURCE_LIMITS', 'INDEPENDENT_REVIEW',
)
M16 = ('id','gate_id','evidence_id','integrity_id','authentication_id','verification_id','area',
       'baseline_sha','admitted_status','scope','offset','length','fixture_sha256','mock_producer',
       'signature_ref','trust_root','limitation')
M17 = ('id','bundle_id','gate_id','area','baseline_sha','status','scope','replay_mode',
       'payload_offset','payload_length','payload_sha256','transcript_sha256','external_verifier',
       'trust_root','limitation')
W17 = ('id','witness','classification','limitation')


def tsv(path: str, header: tuple[str, ...]) -> list[dict[str, str]]:
    with (ROOT / path).open(encoding='utf-8', newline='') as source:
        reader = csv.DictReader(source, delimiter='\t')
        if tuple(reader.fieldnames or ()) != header:
            raise RuntimeError(f'Unexpected TSV header: {path}')
        rows = list(reader)
    if any(set(row) != set(header) or any(value is None or not value.strip() or '\r' in value
                                   for value in row.values()) for row in rows):
        raise RuntimeError(f'Invalid TSV data: {path}')
    return rows


def cursor_decode(blob: bytes) -> list[tuple[int, int, bytes]]:
    if len(blob) < 10 or blob[:8] != b'NDR16PK1' or struct.unpack_from('<H',blob,8)[0] != 12:
        raise RuntimeError('Cursor reader rejected R0.16 header')
    pos = 10
    records = []
    for index in range(1,13):
        if pos + 2 > len(blob):
            raise RuntimeError('Cursor reader found missing length prefix')
        length = struct.unpack_from('<H',blob,pos)[0]
        if not 1 <= length <= 256 or pos + 2 + length > len(blob):
            raise RuntimeError('Cursor reader found invalid bounded length')
        pos += 2
        records.append((index,pos,bytes(blob[pos:pos+length])))
        pos += length
    if pos != len(blob):
        raise RuntimeError('Cursor reader found trailing bytes')
    return records


def indexed_decode(blob: bytes, manifest: list[dict[str,str]]) -> list[tuple[int,int,bytes]]:
    if blob[:8] != b'NDR16PK1' or len(blob) < 10 or int.from_bytes(blob[8:10],'little') != 12:
        raise RuntimeError('Indexed reader rejected header')
    if len(manifest) != 12:
        raise RuntimeError('Indexed reader requires 12 records')
    result = []
    previous = 10
    for index, row in enumerate(manifest,1):
        offset = int(row['offset']); length = int(row['length'])
        if not 1 <= length <= 256 or offset != previous + 2 or offset > len(blob):
            raise RuntimeError('Indexed reader rejected span')
        # Independent direct lookup at manifest offset; never reuse cursor decoder results.
        if int.from_bytes(blob[previous:offset], 'little') != length:
            raise RuntimeError('Indexed reader found length disagreement')
        payload = blob[offset:offset+length]
        if len(payload) != length or hashlib.sha256(payload).hexdigest() != row['fixture_sha256']:
            raise RuntimeError('Indexed reader found digest disagreement')
        if not (row['id'] == f'R16-B{index:02}' and row['gate_id'] == f'R11-G{index:02}'
                and row['evidence_id'] == f'R12-E{index:02}' and row['integrity_id'] == f'R13-L{index:02}'
                and row['authentication_id'] == f'R14-A{index:02}' and row['verification_id'] == f'R15-V{index:02}'
                and row['area'] == AREAS[index-1] and row['baseline_sha'] == '3158ee20a894461df5402335b7db5509db9abd02'
                and row['admitted_status'] == 'ABSENT' and row['scope'] == 'RESEARCH_ONLY'
                and row['mock_producer'] == f'MOCK-PRODUCER-{index:02}'
                and row['signature_ref'] == row['trust_root'] == 'NONE' and len(row['limitation']) >= 65):
            raise RuntimeError(f'Indexed reader rejected frozen lineage {index}')
        result.append((index,offset,bytes(payload)))
        previous = offset + length
    if previous != len(blob):
        raise RuntimeError('Indexed reader detected trailing bytes')
    return result


def encode_replay(records: list[tuple[int,int,bytes]], pack_digest: str) -> bytes:
    if len(records) != 12:
        raise RuntimeError('Wrong record count for transcript')
    out = bytearray(b'NDR17RPL1' + BASE.encode('ascii'))
    out.extend(struct.pack('<H',12))
    out.extend(bytes.fromhex(pack_digest))
    for expected,(index,offset,payload) in enumerate(records,1):
        if index != expected or not 1 <= len(payload) <= 256 or offset > 65535:
            raise RuntimeError('Invalid bounded canonical replay entry')
        out.extend(struct.pack('<HIH',index,offset,len(payload)))
        out.extend(hashlib.sha256(payload).digest())
    out.extend(hashlib.sha256(out).digest())
    return bytes(out)


def reference_legacy() -> None:
    # Call the certified R0.16 static contract without invoking its Git clean-tree guard.
    module_path = ROOT / 'scripts/validate_r16_bundle.py'
    # runpy executes the frozen validator in memory, without writing __pycache__.
    namespace = runpy.run_path(str(module_path), run_name='frozen_r016_contract')
    namespace['static']()


def check_static() -> None:
    for rel in NEW:
        path = ROOT / rel
        if not path.is_file() or path.is_symlink():
            raise RuntimeError(f'Missing or symlinked new R0.17 file: {rel}')
    reference_legacy()
    manifest = tsv('governance/r16_offline_bundle_manifest_v1.tsv',M16)
    registry = tsv('governance/r17_replay_registry_v1.tsv',M17)
    blob = (ROOT/'research/fixtures/r16_synthetic_bundle_v1.bin').read_bytes()
    transcript = (ROOT/'research/fixtures/r17_independent_replay_transcript_v1.bin').read_bytes()
    if hashlib.sha256(blob).hexdigest() != PACK_SHA:
        raise RuntimeError('Frozen R0.16 fixture SHA-256 mismatch')
    if hashlib.sha256(transcript).hexdigest() != TRANSCRIPT_SHA or len(transcript) != 595:
        raise RuntimeError('Reproducible transcript root SHA-256 or size mismatch')
    sequence = cursor_decode(blob)
    indexed = indexed_decode(blob,manifest)
    if sequence != indexed:
        raise RuntimeError('Two local decoders disagree')
    if encode_replay(sequence, PACK_SHA) != transcript or encode_replay(indexed,PACK_SHA) != transcript:
        raise RuntimeError('Python canonical dual transcript mismatch')
    if transcript[-32:] != hashlib.sha256(transcript[:-32]).digest():
        raise RuntimeError('Bad terminal SHA-256')
    if len(registry) != 12:
        raise RuntimeError('Exactly twelve R0.17 registry rows required')
    for index,(record, row) in enumerate(zip(sequence,registry),1):
        _,offset,payload = record
        if not (row['id'] == f'R17-R{index:02}' and row['bundle_id'] == f'R16-B{index:02}'
                and row['gate_id'] == f'R11-G{index:02}' and row['area'] == AREAS[index-1]
                and row['baseline_sha'] == BASE and row['status'] == 'ABSENT'
                and row['scope'] == 'RESEARCH_ONLY' and row['replay_mode'] == 'DUAL_LOCAL_DECODE'
                and row['payload_offset'] == str(offset) and row['payload_length'] == str(len(payload))
                and row['payload_sha256'] == hashlib.sha256(payload).hexdigest()
                and row['transcript_sha256'] == TRANSCRIPT_SHA
                and row['external_verifier'] == row['trust_root'] == 'NONE'
                and len(row['limitation']) >= 65):
            raise RuntimeError(f'Untrustworthy R0.17 registry row {index}')
    rust = (ROOT/'tests/resource_task_r17.rs').read_text(encoding='utf-8')
    tests = re.findall(r'#\[test\]\s*fn\s+([a-z][a-z0-9_]*)\s*\(',rust)
    witnesses = tsv('governance/r17_replay_witnesses_v1.tsv',W17)
    if len(tests) != 44 or len(set(tests)) != 44 or len(witnesses) != 44:
        raise RuntimeError('R0.17 requires 44 unique Rust tests + governance witnesses')
    for index,(test,witness) in enumerate(zip(tests,witnesses),1):
        if not (witness['id'] == f'R17-P{index:02}' and witness['witness'] == test
                and witness['classification'] in {'FROZEN_BASELINE','DUAL_REPLAY','FAIL_CLOSED','TRUST_BOUNDARY'}
                and len(witness['limitation']) >= 65):
            raise RuntimeError(f'Test/witness contract violation: {index}')
    required = ('#[path = "../src/effect_audit/hash.rs"]',
                'include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin")',
                'include_bytes!("../research/fixtures/r17_independent_replay_transcript_v1.bin")',
                'fn cursor_decode(', 'fn indexed_decode(', 'fn transcript_from(',
                'fn assess(', 'fn historic_boundaries(', 'R17_WITNESS version=1', 'ResearchReviewOnly',
                BASE, PACK_SHA, TRANSCRIPT_SHA)
    if any(fragment not in rust for fragment in required):
        raise RuntimeError('Mandatory Rust cross-runtime replay boundary missing')
    if any(fragment in rust for fragment in ('unsafe {','#[ignore]','#[should_panic]','std::thread::spawn(',
                                            'ProductionAuthorized','NativeAuthorized','std::process::Command')):
        raise RuntimeError('Forbidden Rust source in test-only replay')
    for rel in ('Cargo.toml','src/lib.rs'):
        path = ROOT/rel
        if path.exists() and ('resource_task_r17' in path.read_text() or 'r17_replay_registry' in path.read_text()):
            raise RuntimeError('R0.17 exported to production')
    texts = '\n'.join((ROOT/name).read_text(encoding='utf-8') for name in NEW if name.endswith('.md'))
    if any(fragment not in texts for fragment in (BASE,'r0.16','TEST-ONLY','RESEARCH_ONLY','UNMET',
                                                  'ABSENT','SHA-256','NOT','44','46')):
        raise RuntimeError('Missing boundary disclosure in R0.17 documentation')
    # Actual negative controls (including forged trailer repair) must fail reproducibility.
    modified = bytearray(transcript); modified[90] ^= 1
    modified[-32:] = hashlib.sha256(modified[:-32]).digest()
    if hashlib.sha256(modified).hexdigest() == TRANSCRIPT_SHA or modified == encode_replay(sequence,PACK_SHA):
        raise RuntimeError('Forged but resealed transcript was admitted')
    bad = bytearray(blob); bad[0] ^= 1
    try:
        cursor_decode(bytes(bad))
    except RuntimeError:
        pass
    else:
        raise RuntimeError('Corrupt R0.16 packet header was admitted')
    print('R0.17 static contract: PASS')
    print(f'Certified input: r0.16 / {BASE}')
    print('Two local decoders: 12/12 concordant; Python hashlib transcript: 595 bytes')
    print(f'Transcript SHA-256: {TRANSCRIPT_SHA}')
    print('Governance witnesses: 44/44; 12/12 external verifiers ABSENT, 12/12 native gates UNMET')
    print('Scope: TEST-ONLY / RESEARCH_ONLY; no independent external identity verification or native authority')


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(['git',*args],cwd=ROOT,capture_output=True,text=True,check=False)


def check_git_boundary() -> None:
    g = git('rev-parse','--show-toplevel')
    if g.returncode or Path(g.stdout.strip()).resolve() != ROOT.resolve():
        print('R0.16 Git boundary: NOT CHECKED (reconstructed repository without .git)')
        return
    tag = git('rev-parse','r0.16^{commit}')
    if tag.returncode or tag.stdout.strip() != BASE:
        raise RuntimeError('Certified r0.16 tag missing or mismatched')
    prior = git('ls-tree','-r','--name-only','r0.16')
    if prior.returncode:
        raise RuntimeError('Cannot inspect frozen tree')
    prior_paths = set(prior.stdout.splitlines())
    if prior_paths & NEW:
        raise RuntimeError('R0.17 overlaps certified r0.16')
    changed = git('diff','--name-only','r0.16','--','.')
    if changed.returncode or set(changed.stdout.splitlines()) - NEW:
        raise RuntimeError('Frozen paths modified since r0.16')
    untracked = git('ls-files','--others','--exclude-standard')
    if untracked.returncode or set(untracked.stdout.splitlines()) - NEW:
        raise RuntimeError('Unexpected untracked paths outside R0.17')
    if any(not (ROOT/path).is_file() for path in prior_paths):
        raise RuntimeError('Frozen file deleted')
    print('R0.16 Git boundary: PASS (exactly eight permitted additive paths; frozen source untouched)')


if __name__ == '__main__':
    check_static()
    check_git_boundary()
    print('Rust compilation, Clippy, full Release Gate, GitHub CI and real external audit: NOT RUN here')
