#!/usr/bin/env python3
"""NORDOI G0.3 R3-R10: independent novice research operator kit.

Standard-library only. Never creates human responses or certifies human identity.
No CLI, language, runtime, permissions or project-source modifications.
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

R8_BLOB = '696dc7089f20b449aa77f7a553f7973a2558282f'
R9_BLOB = '57c0f6b2c0cc47f207222b8d602ba58bbb3cae9e'
SOURCE = '98da03b2ea1efcf665d205189284aa8b66fda5d3'
PRODUCTION = 'e2898bf7c511e53636ff28c9fd3efc93165a0215'
SCHEMA = 'NORDOI-G03-R3-R10-SESSION-LOG-v1'
SLOTS = tuple('P%03d' % n for n in range(1, 21))
MAX_BYTES = 262144


class Blocked(Exception):
    """Fail closed without leaking participant data or local paths."""


def git_blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode('ascii') + b'\0' + data).hexdigest()


def load_r9(path):
    p = Path(path)
    try:
        s = p.lstat()
        if not stat.S_ISREG(s.st_mode) or s.st_nlink != 1 or s.st_size > 200000:
            raise Blocked('R9 file unsafe')
        raw = p.read_bytes()
    except OSError as exc:
        raise Blocked('R9 unavailable') from exc
    if git_blob(raw) != R9_BLOB:
        # Checkout may convert LF to CRLF on Windows. Compare the exact canonical Git blob.
        if b'\r\n' not in raw or b'\r' in raw.replace(b'\r\n', b'\n'):
            raise Blocked('R9 source pin mismatch')
        raw = raw.replace(b'\r\n', b'\n')
        if git_blob(raw) != R9_BLOB:
            raise Blocked('R9 source pin mismatch')
    m = types.ModuleType('nordoi_r9_pinned')
    m.__file__ = str(p)
    # Executable Python is accepted ONLY after exact Git blob verification.
    exec(compile(raw, str(p), 'exec'), m.__dict__)
    if (m.R8_GIT_BLOB != R8_BLOB or m.SOURCE != SOURCE or
            m.PRODUCTION != PRODUCTION or m.SLOTS != SLOTS):
        raise Blocked('R9 research contract mismatch')
    return m


def verify(r9_path, r8_path, study_path, audit_path):
    r9 = load_r9(r9_path)
    try:
        r8 = r9.read_r8(r8_path)
        root = r9.study_dir(study_path)
        audit = r9.read_json(audit_path)
        allocation = r9.validate_packet_set(r8, root, audit)
    except (r9.Blocked, ValueError, KeyError, TypeError) as exc:
        raise Blocked('prior study evidence invalid') from exc
    if len(allocation['sessions']) != 20:
        raise Blocked('invalid participant slots')
    return allocation


def slot_log_template(allocation):
    return {'schema': SCHEMA, 'phase': 'EMPTY_TEMPLATE_NOT_HUMAN_EVIDENCE',
            'slots': [{'slot': entry['slot'],
                       'novice_screened': None,
                       'voluntary_consent_before_packet': None,
                       'independent_recruitment_reviewed': None,
                       'single_session_observed': None,
                       'six_trials_completed': None,
                       'manual_timing_observed': None,
                       'withdrawn': None,
                       'protocol_deviation': None}
                      for entry in allocation['sessions']]}


CONSENT = '''# NORDOI R3-R10 — Voluntary participant information and consent script

**Research draft; not a deployed consent platform.** Read to each adult volunteer **before** showing any NORDOI diagnostic packet. Preserve one uniform English version for both study arms.

We are studying whether short explanations help people understand six software error messages. You will read six static examples and choose an error type and a next action for each. **Do not run code, change permissions or grant access.** Participation is entirely voluntary. You can decline any item or stop at any time without penalty. The study is a small research pilot, not a test of your intelligence, skill or performance.

Expected activity: six short questions with a stopwatch used by the operator. We record only the coded slot, choices, elapsed seconds and non-identifying operator checks. **Do not supply** your name, contact information, employer, accounts, code, logs, screen recording, or any personal story. Any public release will be aggregated only, after review; privacy cannot be guaranteed absolutely.

Please ask questions before starting. The researcher should ask: **Do you consent voluntarily to this study?** Record the answer **privately**. If no, do not present the packet or record task responses. Never pre-fill consent, independence, expertise or completion fields.

Recruitment: independent, 18+ novice who has never worked on NORDOI and has not seen these exact diagnostics or study materials. No project team members, close collaborators or assistants acting as humans; do not pressure anyone to participate. Record any exclusions privately as aggregate counts only, never identity data.

After consent, the researcher distributes only the sequentially allocated packet for that slot, reads the same instructions to each participant, and records each task time with a manual timer. No extra hints. Condition A and B packets must not be swapped. The tool cannot certify any of these human actions.
'''

OPERATOR = '''# NORDOI R3-R10 — Session operator checklist (private)

**Research / NO HUMAN EVIDENCE YET / NO PRODUCTION.** This folder is an operator aid and **not** a study result. It does not contain participant answers or copied NORDOI packets.

1. Use only genuine independent consenting adult novices; avoid collaborators and exposure to the NORDOI exercises. The person running this study must independently check eligibility. Do not record names, emails, addresses, IPs, age or employers; eligibility is a human check.
2. Keep the existing R3-R8 private folder with `allocation.json` and P001–P020.md unchanged. Assign the next *unused* sequential slot on actual enrollment, with no re-randomization and no substitution after results are known. Never pre-create participants from slot numbers.
3. **Before** displaying the packet: read `CONSENT_SCRIPT.md`; record refusal as no response and stop. `session_log_TEMPLATE.json` is deliberately blank. Make a separate PRIVATE copy `session_log_ACTUAL.json` only when genuine sessions are conducted.
4. Present exactly the six scenarios in the packet. A: exact original stderr; B: identical stderr + hint. The packet already holds the condition. Do not edit stderr, hide full usage text, offer extra correction advice, run CLI commands or grant permissions.
5. Record category, safe action, and elapsed seconds per task in the pre-existing private `responses_ACTUAL.json`, using R3-R8's original response schema and study answer choices. This R3-R10 tool never creates participant responses. Keep it off GitHub, CI and public chats.
6. A withdrawal, incomplete session, refused consent, protocol deviation or unmatched independent-recruitment record BLOCKS a claim of completed 20-person pilot. Do not fill gaps with AI agents or synthetic records. Preserve the original protocol; document deviations separately, non-identifying.
7. `review-log` checks fields structurally only; a boolean `true` is **not** external proof of participant identity, consent, timing or independence. Retain private operator context and seek independent research review.
8. No powered efficacy inference or adoption decision is permitted from this 20-person feasibility pilot. A separate unseen transfer study and authority-safety assessment are required before any CLI diagnostic change.
'''


def private_file(path, content):
    flags = os.O_CREAT | os.O_EXCL | os.O_WRONLY
    if hasattr(os, 'O_NOFOLLOW'):
        flags |= os.O_NOFOLLOW
    fd = os.open(str(path), flags, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)


def make_kit(output, allocation):
    root = Path(output)
    if root.exists() or root.is_symlink():
        raise Blocked('kit destination exists')
    root.mkdir(mode=0o700, parents=False, exist_ok=False)
    private_file(root / 'CONSENT_SCRIPT.md', CONSENT)
    private_file(root / 'OPERATOR_CHECKLIST.md', OPERATOR)
    private_file(root / 'session_log_TEMPLATE.json',
                 json.dumps(slot_log_template(allocation), indent=2) + '\n')
    return root


def read_log(path):
    p = Path(path)
    try:
        s = p.lstat()
        if not stat.S_ISREG(s.st_mode) or s.st_nlink != 1 or s.st_size > MAX_BYTES:
            raise Blocked('unsafe log file')
        if os.name == 'posix' and s.st_mode & 0o077:
            raise Blocked('private log permissions too broad')
        return json.loads(p.read_text(encoding='utf-8'))
    except (OSError, ValueError, UnicodeError) as exc:
        raise Blocked('invalid log') from exc


def validate_log(data, allocation):
    if not isinstance(data, dict) or set(data) != {'schema', 'phase', 'slots'}:
        raise Blocked('log fields invalid')
    if data.get('schema') != SCHEMA or data.get('phase') != 'ACTUAL_MANUALLY_RECORDED':
        raise Blocked('log remains template or unknown phase')
    rows = data.get('slots')
    if not isinstance(rows, list) or len(rows) != 20:
        raise Blocked('log size invalid')
    for row, assigned in zip(rows, allocation['sessions']):
        if not isinstance(row, dict) or set(row) != {
                'slot','novice_screened','voluntary_consent_before_packet',
                'independent_recruitment_reviewed','single_session_observed',
                'six_trials_completed','manual_timing_observed','withdrawn',
                'protocol_deviation'}:
            raise Blocked('log row invalid')
        if row['slot'] != assigned['slot']:
            raise Blocked('slot order changed')
        if any(row[k] is not True for k in (
                'novice_screened','voluntary_consent_before_packet',
                'independent_recruitment_reviewed','single_session_observed',
                'six_trials_completed','manual_timing_observed')):
            raise Blocked('required human gate missing')
        if row['withdrawn'] is not False or row['protocol_deviation'] != 'none':
            raise Blocked('withdrawal or protocol deviation')
    return True


def selftest():
    allocation = {'sessions': [{'slot': name} for name in SLOTS]}
    empty = slot_log_template(allocation)
    assert len(empty['slots']) == 20 and all(
        row['voluntary_consent_before_packet'] is None for row in empty['slots'])
    for invalid in (empty, {'schema': SCHEMA, 'phase': 'ACTUAL_MANUALLY_RECORDED', 'slots': []}):
        try:
            validate_log(invalid, allocation)
        except Blocked:
            pass
        else:
            raise AssertionError('blank/invalid participant data passed')
    rows=[]
    for name in SLOTS:
        rows.append({'slot': name, 'novice_screened': True,
                    'voluntary_consent_before_packet': True,
                    'independent_recruitment_reviewed': True,
                    'single_session_observed': True,
                    'six_trials_completed': True, 'manual_timing_observed': True,
                    'withdrawn': False, 'protocol_deviation': 'none'})
    synthetic = {'schema': SCHEMA, 'phase': 'ACTUAL_MANUALLY_RECORDED', 'slots': rows}
    assert validate_log(synthetic, allocation)
    for field,value in [('independent_recruitment_reviewed',None),
                        ('withdrawn',True),('protocol_deviation','assistance')]:
        broken = json.loads(json.dumps(synthetic))
        broken['slots'][0][field] = value
        try:
            validate_log(broken, allocation)
        except Blocked:
            pass
        else:
            raise AssertionError('unsafe or incomplete log was accepted')
    with tempfile.TemporaryDirectory(prefix='NORDOI_R10_SYNTHETIC_') as tmp:
        root = Path(tmp) / 'kit'
        make_kit(root, allocation)
        assert (root/'CONSENT_SCRIPT.md').exists()
        assert (root/'OPERATOR_CHECKLIST.md').exists()
        assert read_log(root/'session_log_TEMPLATE.json')['phase'] == 'EMPTY_TEMPLATE_NOT_HUMAN_EVIDENCE'
        try:
            make_kit(root, allocation)
        except Blocked:
            pass
        else:
            raise AssertionError('existing directory overwritten')
    print('G03_R3_R10_SELFTEST=SYNTHETIC_PASS')
    print('HUMAN_TRIALS=0_IN_SELFTEST')
    print('PRODUCTION=NOT_AUTHORIZED')


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    sub=ap.add_subparsers(dest='command',required=True)
    sub.add_parser('selftest')
    for mode in ('check','prepare','review-log'):
        p=sub.add_parser(mode)
        p.add_argument('--r8-tool',required=True)
        p.add_argument('--r9-tool',required=True)
        p.add_argument('--study-dir',required=True)
        p.add_argument('--report',required=True)
        if mode=='prepare': p.add_argument('--out',required=True)
        if mode=='review-log':
            p.add_argument('--operator-dir',required=True)
            p.add_argument('--log',required=True)
    args=ap.parse_args()
    try:
        if args.command=='selftest':
            selftest()
            return 0
        alloc=verify(args.r9_tool,args.r8_tool,args.study_dir,args.report)
        print('G03_R3_R10_R8_R9_INTEGRITY=PASS')
        print('G03_R3_R10_PACKET_SET=20/20_VERIFIED')
        if args.command=='prepare':
            make_kit(args.out,alloc)
            print('G03_R3_R10_OPERATOR_KIT=PREPARED_PRIVATE')
            print('CONSENT_AND_INDEPENDENCE=NOT_ATTESTED')
        elif args.command=='review-log':
            r=Path(args.operator_dir)
            s=r.lstat()
            if not stat.S_ISDIR(s.st_mode) or (os.name=='posix' and s.st_mode & 0o077):
                raise Blocked('operator directory unsafe')
            l=Path(args.log)
            if l.parent.resolve()!=r.resolve() or l.name!='session_log_ACTUAL.json':
                raise Blocked('noncanonical session log')
            validate_log(read_log(l),alloc)
            print('G03_R3_R10_LOG_STRUCTURAL_GATE=PASS')
            print('HUMAN_PROVENANCE=UNVERIFIED_BY_SOFTWARE')
            print('PEDAGOGICAL_SUPERIORITY=NOT_AUTHORIZED')
        else:
            print('G03_R3_R10_OPERATOR_READINESS=PASS')
            print('HUMAN_TRIALS=NOT_MEASURED_BY_READINESS_CHECK')
        print('STUDY_ORIGINAL_FILES=UNMODIFIED')
        print('GITHUB_PUSH=NONE')
        print('PRODUCTION=NOT_AUTHORIZED')
        return 0
    except (Blocked, ValueError, OSError, TypeError, KeyError, AssertionError, UnicodeError):
        print('G03_R3_R10_GATE=BLOCKED',file=sys.stderr)
        print('PRODUCTION=NOT_AUTHORIZED',file=sys.stderr)
        return 2


if __name__=='__main__':
    sys.exit(main())
