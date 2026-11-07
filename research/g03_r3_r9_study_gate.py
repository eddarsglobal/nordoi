#!/usr/bin/env python3
"""NORDOI G0.3 R3-R9: audit a prepared study; score only actual supplied responses.

Research-only, read-only, standard library; nothing here can certify a human identity.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import types

R8_GIT_BLOB = '696dc7089f20b449aa77f7a553f7973a2558282f'
R8_STUDY = 'NORDOI-G03-R3-R8-PILOT-v1'
R8_RESPONSES = 'NORDOI-G03-R3-R8-RESPONSES-v1'
R9_STATUS = 'NORDOI-G03-R3-R9-READONLY-GATE-v1'
SOURCE = '98da03b2ea1efcf665d205189284aa8b66fda5d3'
PRODUCTION = 'e2898bf7c511e53636ff28c9fd3efc93165a0215'
SLOTS = tuple(f'P{i:03d}' for i in range(1, 21))
MAX_INPUT_BYTES = 1_048_576


class Blocked(Exception):
    """Intentionally hides sensitive paths or participant material."""


def check_regular(path, *, limit=MAX_INPUT_BYTES, private=False):
    p = Path(path)
    try:
        s = p.lstat()
    except OSError as exc:
        raise Blocked('unavailable input') from exc
    if not stat.S_ISREG(s.st_mode) or s.st_nlink != 1 or s.st_size > limit:
        raise Blocked('unsafe file type or size')
    if private and os.name == 'posix' and s.st_mode & 0o077:
        raise Blocked('private file permissions too broad')
    return p


def read_json(path, *, private=False):
    p = check_regular(path, private=private)
    try:
        with p.open('rb') as f:
            raw = f.read(MAX_INPUT_BYTES + 1)
        if len(raw) > MAX_INPUT_BYTES:
            raise Blocked('oversized JSON')
        value = json.loads(raw.decode('utf-8'))
    except (UnicodeError, json.JSONDecodeError, OSError) as exc:
        raise Blocked('invalid or unreadable JSON') from exc
    if not isinstance(value, dict):
        raise Blocked('wrong JSON shape')
    return value


def read_r8(path):
    p = check_regular(path, limit=200_000)
    try:
        blob = p.read_bytes()
    except OSError as exc:
        raise Blocked('pinned tool unreadable') from exc
    git_sha = hashlib.sha1(b'blob ' + str(len(blob)).encode('ascii') + b'\0' + blob).hexdigest()
    if git_sha != R8_GIT_BLOB:
        raise Blocked('R8 Git blob pin mismatch')
    # This exact, reviewed file is executable Python. NEVER execute an unpinned file.
    module = types.ModuleType('nordoi_g03_r3_r8_pinned')
    module.__file__ = str(p)
    exec(compile(blob, str(p), 'exec'), module.__dict__)
    if (module.SOURCE != SOURCE or module.PRODUCTION != PRODUCTION or
            module.STUDY_SCHEMA != R8_STUDY or module.RESPONSE_SCHEMA != R8_RESPONSES or
            module.SLOTS != 20 or len(module.CASES) != 6):
        raise Blocked('pinned R8 contract mismatch')
    return module


def study_dir(path):
    p = Path(path)
    try:
        s = p.lstat()
    except OSError as exc:
        raise Blocked('study directory missing') from exc
    if not stat.S_ISDIR(s.st_mode):
        raise Blocked('unsafe study directory')
    if os.name == 'posix' and s.st_mode & 0o077:
        raise Blocked('study directory not private')
    return p


def validate_packet_set(r8, root, audit):
    p = study_dir(root)
    raw = read_json(p / 'allocation.json', private=True)
    if raw.get('schema') != R8_STUDY or raw.get('source_sha') != SOURCE or raw.get('production_sha') != PRODUCTION:
        raise Blocked('allocation provenance mismatch')
    if raw.get('design') != 'parallel-randomized-blocks-of-4':
        raise Blocked('allocation design mismatch')
    sessions = raw.get('sessions')
    if not isinstance(sessions, list) or len(sessions) != 20:
        raise Blocked('allocation size mismatch')
    try:
        validated = r8.validate_audit(audit)
    except (ValueError, KeyError, TypeError) as exc:
        raise Blocked('R3-R6 evidence mismatch') from exc
    arms = []
    for i, entry in enumerate(sessions):
        slot = SLOTS[i]
        if not isinstance(entry, dict) or entry != {'slot': slot, 'arm': entry.get('arm'), 'packet': slot + '.md'}:
            raise Blocked('allocation order or field mismatch')
        arm = entry['arm']
        if arm not in ('A', 'B'):
            raise Blocked('unexpected arm')
        arms.append(arm)
        path = check_regular(p / (slot + '.md'), limit=300_000, private=True)
        expected = r8.packet(slot, arm, validated).encode('utf-8')
        try:
            actual = path.read_bytes()
        except OSError as exc:
            raise Blocked('packet unreadable') from exc
        if actual != expected:
            raise Blocked('packet does not match original R8 instrument')
        if actual.count(b'Additional research hint') != (6 if arm == 'B' else 0):
            raise Blocked('A/B hint contamination')
    if sorted(arms) != ['A'] * 10 + ['B'] * 10:
        raise Blocked('allocation not balanced')
    for offset in range(0, 20, 4):
        if arms[offset:offset + 4].count('A') != 2:
            raise Blocked('randomized block mismatch')
    template = read_json(p / 'responses_TEMPLATE.json', private=True)
    if template.get('schema') != R8_RESPONSES or not isinstance(template.get('entries'), list):
        raise Blocked('response template missing')
    if [x.get('slot') for x in template['entries'] if isinstance(x, dict)] != list(SLOTS):
        raise Blocked('template slots mismatch')
    for row in template['entries']:
        if any(row.get(field) is not None for field in ('novice', 'consented', 'independent')):
            raise Blocked('pre-populated human evidence in template')
        trials = row.get('trials')
        if not isinstance(trials, list) or [x.get('case') for x in trials if isinstance(x, dict)] != r8.CASE_IDS:
            raise Blocked('template trial mismatch')
        if any(any(t.get(k) is not None for k in ('diagnosis', 'action', 'seconds')) for t in trials):
            raise Blocked('pre-populated human trial in template')
    return raw


def analyze(r8, allocation, responses):
    if responses.get('schema') != R8_RESPONSES:
        raise Blocked('responses schema mismatch')
    try:
        aggregates = r8.score(allocation, responses)
    except (ValueError, TypeError, KeyError, ZeroDivisionError) as exc:
        raise Blocked('incomplete, unsafe or invalid responses') from exc
    if aggregates['A']['n'] != 10 or aggregates['B']['n'] != 10:
        raise Blocked('unexpected population')
    return aggregates


def run_selftest(r8):
    # Entirely synthetic, unpublishable fixture. Never make or claim human records.
    source_events = [
        ('missing_grant_console_run', 5, 'console-run', 'capability denied'),
        ('undeclared_effect', 4, 'console-run', 'effect ConsoleWrite;'),
        ('misspelled_emits', 4, 'console-run', "expected contextual 'emits'"),
        ('invalid_option', 2, 'usage', 'error[usage]'),
        ('missing_source_file', 3, 'io', 'error[io]'),
        ('legacy_plain_run', 4, 'run', 'error[run]'),
    ]
    events = []
    for name, code, stage, marker in source_events:
        events.append({'id': name, 'exit_code': code, 'error_stage': stage,
                       'stderr_text': marker, 'stdout_text': '', 'stderr_bytes': len(marker.encode()),
                       'stdout_bytes': 0, 'args': ['console-run', '-'] if name == 'missing_grant_console_run' else ['run', '-']})
    for i in range(4):
        events.append({'id': 'other_' + str(i)})
    report = {'schema': r8.REPORT_SCHEMA, 'source_sha': SOURCE, 'production_sha': PRODUCTION,
              'case_count': 10, 'passed': 10, 'source_modifications': 'NONE',
              'diagnostic_rewrites': 'NONE', 'production': 'NOT_AUTHORIZED', 'events': events}
    with tempfile.TemporaryDirectory(prefix='NORDOI_R9_SYNTHETIC_') as folder:
        root = Path(folder)
        output = root / 'study'
        r8.prepare(report, output)
        alloc = validate_packet_set(r8, output, report)
        assert len(alloc['sessions']) == 20
        # Report tampering or packet altering must fail closed.
        bad = dict(report, passed=9)
        try:
            validate_packet_set(r8, output, bad)
        except Blocked:
            pass
        else:
            raise AssertionError('invalid source report was accepted')
        pkt = output / 'P001.md'
        pkt.write_bytes(pkt.read_bytes() + b'altered\n')
        try:
            validate_packet_set(r8, output, report)
        except Blocked:
            pass
        else:
            raise AssertionError('tampered packet accepted')
        # Synthetic outcome data never entered in files, and not considered people.
        incomplete = read_json(output / 'responses_TEMPLATE.json', private=True)
        try:
            analyze(r8, alloc, incomplete)
        except Blocked:
            pass
        else:
            raise AssertionError('blank responses accepted')
    print('G03_R3_R9_SELFTEST=SYNTHETIC_PASS')
    print('HUMAN_TRIALS=0_IN_SELFTEST')
    print('PRODUCTION=NOT_AUTHORIZED')


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    commands = ap.add_subparsers(dest='command', required=True)
    for name in ('selftest', 'check', 'analyze'):
        c = commands.add_parser(name)
        c.add_argument('--r8-tool', required=True)
        if name != 'selftest':
            c.add_argument('--study-dir', required=True)
            c.add_argument('--report', required=True)
        if name == 'analyze':
            c.add_argument('--responses', required=True)
    args = ap.parse_args()
    try:
        r8 = read_r8(args.r8_tool)
        if args.command == 'selftest':
            run_selftest(r8)
            return 0
        root = study_dir(args.study_dir)
        audit = read_json(args.report)
        alloc = validate_packet_set(r8, root, audit)
        print('G03_R3_R9_INTEGRITY=PASS')
        print('G03_R3_R9_PACKET_SET=20/20_VERIFIED')
        print('G03_R3_R9_ARM_COUNTS=A10_B10')
        print('G03_R3_R9_BLOCKS=5/5_BALANCED')
        if args.command == 'check':
            status = 'PRESENT_UNVERIFIED' if (root / 'responses_ACTUAL.json').exists() else 'NOT_PRESENT'
            print('HUMAN_RESPONSE_FILE=' + status)
            print('HUMAN_COMPREHENSION=UNMEASURED_BY_THIS_CHECK')
        else:
            response = check_regular(args.responses, private=True)
            if response.parent.resolve() != root.resolve() or response.name != 'responses_ACTUAL.json':
                raise Blocked('responses not canonical private study data')
            data = read_json(response, private=True)
            aggregates = analyze(r8, alloc, data)
            # Descriptive outcomes from user-supplied fields; NOT independent human evidence.
            print('G03_R3_R9_DESCRIPTIVE_SELF_REPORT=' + json.dumps(aggregates, sort_keys=True))
            print('INDEPENDENT_HUMAN_PROVENANCE=UNVERIFIED_BY_SOFTWARE')
            print('PEDAGOGICAL_SUPERIORITY=NOT_AUTHORIZED')
        print('STUDY_ORIGINAL_FILES=UNMODIFIED')
        print('GITHUB_PUSH=NONE')
        print('PRODUCTION=NOT_AUTHORIZED')
        return 0
    except (Blocked, ValueError, OSError, KeyError, UnicodeError, TypeError, AssertionError):
        # No exception message: avoid leaking private packet text, paths or responses.
        print('G03_R3_R9_GATE=BLOCKED', file=sys.stderr)
        print('PRODUCTION=NOT_AUTHORIZED', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
