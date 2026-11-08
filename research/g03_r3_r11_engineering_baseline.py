#!/usr/bin/env python3
"""NORDOI G0.3 R3-R11: pinned, isolated, end-to-end CLI performance baseline.

No human subjects, no claims about competing languages, no modifications to original repo.
Selftest is synthetic; only 'measure' runs real NORDOI in a disposable clone.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

SOURCE = '98da03b2ea1efcf665d205189284aa8b66fda5d3'
REPOSITORY = 'https://github.com/eddarsglobal/nordoi.git'
SCHEMA = 'NORDOI-G03-R3-R11-CLI-BASELINE-v1'
VERSION = b'nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n'
HELLO = b'Hello from NORDOI P2.1!\n'
CORE = b'fn add(a,b){a+b} entry main returns add(20,22);'
CONSOLE = (b'module app.main;\n\neffect ConsoleWrite;\n\n'
           b'entry main emits "Hello from NORDOI P2.1!\\n";\n')
MODES = ('startup', 'pure_core', 'console_explicit', 'console_denied')


class Blocked(Exception):
    pass


def run_command(command, *, cwd=None, input_bytes=None, timeout=45, env=None):
    try:
        return subprocess.run(command, input=input_bytes, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, cwd=cwd, env=env,
                              timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise Blocked('process failed or timed out') from exc


def validate(mode, result):
    if mode == 'startup':
        ok = result.returncode == 0 and result.stdout == VERSION and not result.stderr
    elif mode == 'pure_core':
        ok = (result.returncode == 0 and b'result=INT(42)' in result.stdout and
              b'nair-instructions=2' in result.stdout and
              b'runtime-calls=0' in result.stdout and not result.stderr)
    elif mode == 'console_explicit':
        ok = (result.returncode == 0 and result.stdout == HELLO and
              b'authority=EXPLICIT' in result.stderr and
              b'ambient-authority=NONE' in result.stderr)
    elif mode == 'console_denied':
        ok = (result.returncode == 5 and result.stdout == b'' and
              b'capability denied' in result.stderr and
              b'ConsoleWrite' in result.stderr)
    else:
        raise Blocked('unknown experiment mode')
    if not ok:
        raise Blocked('observed output or authority contract mismatch')


def describe(values):
    if not values or any(not isinstance(v, int) or v <= 0 for v in values):
        raise Blocked('invalid timing input')
    a = sorted(values)
    return {
        'n': len(a),
        'median_ms': round(statistics.median(a) / 1_000_000, 6),
        'p95_ms': round(a[max(0, math.ceil(0.95 * len(a)) - 1)] / 1_000_000, 6),
        'min_ms': round(a[0] / 1_000_000, 6),
        'max_ms': round(a[-1] / 1_000_000, 6),
    }


def write_private_exclusive(path, payload):
    p = Path(path)
    if p.exists() or p.is_symlink() or not p.parent.is_dir():
        raise Blocked('output exists or directory missing')
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, 'O_NOFOLLOW'):
        flags |= os.O_NOFOLLOW
    try:
        fd = os.open(str(p), flags, 0o600)
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            json.dump(payload, stream, indent=2, sort_keys=True)
            stream.write('\n')
    except OSError as exc:
        raise Blocked('private receipt write refused') from exc
    if os.name == 'posix' and p.stat().st_mode & 0o077:
        raise Blocked('output permissions unexpectedly broad')


def exercise(binary, mode, *, cwd=None):
    args = {
        'startup': ['--version'],
        'pure_core': ['core-run', '-'],
        'console_explicit': ['run', '-', '--grant-console'],
        'console_denied': ['console-run', '-'],
    }[mode]
    inp = {'startup': None, 'pure_core': CORE,
           'console_explicit': CONSOLE, 'console_denied': CONSOLE}[mode]
    t0 = time.perf_counter_ns()
    r = run_command([str(binary)] + args, input_bytes=inp, cwd=cwd, timeout=15)
    elapsed = time.perf_counter_ns() - t0
    validate(mode, r)
    return elapsed


def tool_version(cmd):
    p = run_command(cmd, timeout=10)
    if p.returncode != 0:
        raise Blocked('tool version unavailable')
    return p.stdout.decode('utf-8', errors='replace').splitlines()[0][:160]


def measure(path, *, samples, warmups):
    if not shutil.which('git') or not shutil.which('cargo') or not shutil.which('rustc'):
        raise Blocked('git/cargo/rustc unavailable')
    versions = {'git': tool_version(['git', '--version']),
                'cargo': tool_version(['cargo', '--version']),
                'rustc': tool_version(['rustc', '--version'])}
    with tempfile.TemporaryDirectory(prefix='NORDOI_R3_R11_ISOLATED_') as temp:
        root = Path(temp)
        work = root / 'source'
        clone = run_command(['git', 'clone', '--quiet', '--single-branch', '--branch', 'main',
                             '--depth', '1', REPOSITORY, str(work)], timeout=180)
        if clone.returncode != 0:
            raise Blocked('pinned source clone unavailable')
        result = run_command(['git', 'rev-parse', 'HEAD'], cwd=work)
        if result.returncode != 0 or result.stdout.strip().decode('ascii', errors='replace') != SOURCE:
            raise Blocked('main source SHA moved: fail closed without benchmarking')
        env = dict(os.environ)
        env['CARGO_NET_OFFLINE'] = 'true'
        env['CARGO_TARGET_DIR'] = str(root / 'target')
        t0 = time.perf_counter_ns()
        locked = (work / 'Cargo.lock').is_file()
        cargo_args = ['cargo', 'build', '--offline']
        if locked:
            cargo_args.append('--locked')
        cargo_args.extend(['--release', '--bin', 'nordoi'])
        build = run_command(cargo_args, cwd=work, env=env, timeout=900)
        build_ns = time.perf_counter_ns() - t0
        if build.returncode != 0:
            raise Blocked('offline pinned build failed')
        binary = root / 'target' / 'release' / ('nordoi.exe' if os.name == 'nt' else 'nordoi')
        if not binary.is_file() or binary.is_symlink():
            raise Blocked('release executable missing')
        # Every timed mode has a fixed observable contract. No automatically granted permissions.
        series = {name: [] for name in MODES}
        for _ in range(warmups):
            for mode in MODES:
                exercise(binary, mode, cwd=work)
        # Interleaved fixed rotation lowers some systematic ordering bias without
        # implying that process launch time is kernel-only runtime speed.
        for i in range(samples):
            for j in range(len(MODES)):
                mode = MODES[(i + j) % len(MODES)]
                series[mode].append(exercise(binary, mode, cwd=work))
        report = {
            'schema': SCHEMA, 'source_sha': SOURCE, 'tool_versions': versions,
            'host': {'system': platform.system(), 'machine': platform.machine(),
                     'python': platform.python_version(), 'cpu_count': os.cpu_count()},
            'build': {'offline': True, 'locked': locked, 'release': True,
                      'seconds': round(build_ns / 1e9, 4)},
            'executable': {'bytes': binary.stat().st_size,
                           'sha256': hashlib.sha256(binary.read_bytes()).hexdigest()},
            'workload': 'WHOLE_PROCESS_CLI_ONLY_NOT_KERNEL_SPEED_OR_CROSS_LANGUAGE',
            'warmups': warmups, 'samples_per_mode': samples,
            'summary': {m: describe(series[m]) for m in MODES},
            'raw_ns': series, 'valid_authority_contracts': True,
            'human_evaluation_gate': 'CLOSED_NOT_REQUIRED',
            'claims_of_fastest_or_smallest': 'NOT_ESTABLISHED',
            'production': 'NOT_AUTHORIZED',
            'original_repo_modified': False,
        }
        write_private_exclusive(path, report)
        print('G03_R3_R11_ENGINEERING_BASELINE=PASS')
        print('PINNED_SOURCE_SHA=' + SOURCE)
        print('OFFLINE_BUILD=PASS')
        print('EXECUTABLE_BYTES=' + str(binary.stat().st_size))
        print('BUILD_SECONDS=' + str(report['build']['seconds']))
        for name in MODES:
            print(name.upper() + '_MEDIAN_MS=' + str(report['summary'][name]['median_ms']))
        print('REPORT_PRIVATE=' + str(path))
        print('COMPARISON_TO_OTHER_LANGUAGES=NOT_YET_MEASURED')
        print('PRODUCTION=NOT_AUTHORIZED')


def selftest():
    assert describe([1_000_000, 2_000_000, 3_000_000])['median_ms'] == 2.0
    assert describe([1_000_000, 2_000_000, 3_000_000])['p95_ms'] == 3.0
    for bad in ([], [0], [1.2], [-2]):
        try:
            describe(bad)
        except Blocked:
            pass
        else:
            raise AssertionError('unsafe timing accepted')
    class Result:
        def __init__(self, code, out, err=b''):
            self.returncode, self.stdout, self.stderr = code, out, err
    validate('startup', Result(0, VERSION))
    validate('pure_core', Result(0, b'result=INT(42) nair-instructions=2 runtime-calls=0'))
    validate('console_explicit', Result(0, HELLO, b'authority=EXPLICIT ambient-authority=NONE'))
    validate('console_denied', Result(5, b'', b'capability denied ConsoleWrite'))
    for mode, r in [('console_explicit', Result(0, HELLO, b'')),
                    ('console_denied', Result(0, HELLO)),
                    ('pure_core', Result(0, b'result=INT(41)'))]:
        try:
            validate(mode, r)
        except Blocked:
            pass
        else:
            raise AssertionError('unsafe observable contract accepted')
    with tempfile.TemporaryDirectory(prefix='NORDOI_R3R11_SYNTHETIC_') as dirname:
        p = Path(dirname) / 'receipt.json'
        write_private_exclusive(p, {'schema': SCHEMA, 'synthetic_only': True})
        assert json.loads(p.read_text(encoding='utf-8'))['synthetic_only']
        try:
            write_private_exclusive(p, {})
        except Blocked:
            pass
        else:
            raise AssertionError('overwritten private report')
    print('G03_R3_R11_SELFTEST=SYNTHETIC_PASS')
    print('REAL_BENCHMARK=NOT_RUN_IN_SELFTEST')
    print('HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED')
    print('PRODUCTION=NOT_AUTHORIZED')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('selftest')
    m = sub.add_parser('measure')
    m.add_argument('--out', required=True)
    m.add_argument('--samples', type=int, default=30)
    m.add_argument('--warmups', type=int, default=5)
    args = parser.parse_args()
    try:
        if args.command == 'selftest':
            selftest()
        else:
            if not 10 <= args.samples <= 200 or not 1 <= args.warmups <= 30:
                raise Blocked('invalid sampling limits')
            measure(args.out, samples=args.samples, warmups=args.warmups)
        return 0
    except (Blocked, OSError, AssertionError, ValueError):
        print('G03_R3_R11_GATE=BLOCKED', file=sys.stderr)
        print('PRODUCTION=NOT_AUTHORIZED', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
