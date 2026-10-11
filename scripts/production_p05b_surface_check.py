#!/usr/bin/env python3
"""P0.5B narrow production-surface regression check. NOT a Rust proof or global closure.

Validates historical baselines through Git and fail-closed shape of the *selected*
P0.5A facade. Deliberately does not assert that other public kernel APIs vanish.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

POLICY_PATH = "production/p05b_surface_policy.json"


class PolicyError(Exception):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PolicyError(message)


def code_tokens(source: str) -> str:
    """Remove Rust comments/strings while preserving identifiers and punctuation.

    Refuses raw strings because a blind textual security check must not silently
    misparse newly introduced source forms. This is intentionally restrictive.
    """
    require(re.search(r'r#+\"', source) is None, "raw Rust string requires review")
    out = []
    i = 0
    n = len(source)
    depth = 0
    while i < n:
        if depth:
            if source.startswith('/*', i):
                depth += 1
                i += 2
            elif source.startswith('*/', i):
                depth -= 1
                i += 2
            else:
                i += 1
            continue
        if source.startswith('//', i):
            pos = source.find('\n', i)
            i = n if pos < 0 else pos
            continue
        if source.startswith('/*', i):
            depth = 1
            i += 2
            continue
        if source[i] == '"':
            # Ignore the contents of strings, which are not executable Rust.
            out.append('""')
            i += 1
            closed = False
            while i < n:
                if source[i] == '\\':
                    i += 2
                elif source[i] == '"':
                    i += 1
                    closed = True
                    break
                else:
                    i += 1
            require(closed, "unterminated Rust string")
            continue
        out.append(source[i])
        i += 1
    require(depth == 0, "unterminated Rust block comment")
    return ''.join(out)


def facade_check(source: str, allowed_methods: set[str]) -> None:
    code = code_tokens(source)
    flat = re.sub(r'\s+', ' ', code)
    struct = re.search(r'pub\s+struct\s+FencedProductionEffects\s*<\s*Store\s*,\s*Backend\s*,\s*Source\s*>\s*\{([^{}]+)\}', code, re.S)
    require(struct is not None, 'missing fenced facade struct')
    fields = re.findall(r'(?m)^\s*(\w+)\s*:\s*([^,\n]+(?:<[^>]+>)?)\s*,?\s*$', struct.group(1))
    expected_fields = {'event_loop', 'journal', 'dispatcher', 'backend'}
    field_names = {name for name, _ in fields}
    require(field_names == expected_fields, f'facade fields modified: {field_names}')
    require('pub ' not in struct.group(1), 'facade must not expose public fields')
    require(re.search(r'\bbackend\s*:\s*ProductionEffectBackend\s*<\s*Backend\s*,\s*Source\s*>', struct.group(1)) is not None, 'backend is not P0.4 durable guarded type')
    require(re.search(r'\bevent_loop\s*:\s*AtomicEventLoop\b', struct.group(1)) is not None, 'facade lost owned event loop')
    require(re.search(r'\bjournal\s*:\s*GovernedFencedEffectJournal\s*<\s*Store\s*>', struct.group(1)) is not None, 'facade lost fenced journal')
    require(re.search(r'\bdispatcher\s*:\s*GovernedEffectDispatcher\b', struct.group(1)) is not None, 'facade lost dispatcher')
    methods = re.findall(r'\bpub\s+(?:async\s+)?fn\s+(\w+)\s*(?:<|\()', code)
    require(set(methods) == allowed_methods and len(methods) == len(allowed_methods), f'public facade methods changed: {methods}')
    require(re.search(r'\bself\s*\.\s*event_loop\s*\.\s*cycle_to_with_fenced_effect_journal\s*\(', flat) is not None, 'cycle no longer journals with fencing')
    require(re.search(r'\bself\s*\.\s*event_loop\s*\.\s*dispatch_next_effect_with_fenced_journal\s*\(', flat) is not None, 'dispatch no longer uses fenced journal')
    require(re.search(r'&\s*mut\s+self\s*\.\s*backend\b', flat) is not None, 'dispatch does not use owned P0.4 backend')
    require('impl EffectBackend for FencedProductionEffects' not in flat, 'facade must not become raw backend')
    forbidden = [
        r'\bpub\s+fn\s+(?:into_inner|backend_mut|event_loop_mut|journal_mut|raw_dispatch)\s*\(',
        r'\.\s*(?:execute_with_context|execute|dispatch_next_effect|dispatch_next_with_delivery_context)\s*\(',
        r'\b(?:std\s*::\s*fs|std\s*::\s*process)\b|\bunsafe\s*\{',
        r'\b(?:observable_io_p2\d|Command\s*::\s*new)\b',
        r'\bpub\s+(?:struct|enum|trait)\s+(?!FencedProductionEffects\b)\w+',
    ]
    for pattern in forbidden:
        require(re.search(pattern, code) is None, f'new facade bypass candidate: {pattern}')
    require('backend: ProductionEffectBackend' in source, 'auditable explicit production backend required')


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True, stderr=subprocess.PIPE).strip()


def check_repo(repo: Path) -> None:
    policy = json.loads((repo / POLICY_PATH).read_text(encoding='utf-8'))
    require(policy['schema'] == 'nordoi-production-p05b-surface-policy-v1', 'unsupported policy schema')
    require(not policy['kernel_wide_mandatory_enforcement_certified'], 'cannot claim global enforcement')
    require(not policy['production_deployment_authorized'], 'cannot grant deployment')
    base = policy['certified_base_sha']
    branch = git(repo, 'branch', '--show-current')
    in_ci = os.environ.get('GITHUB_ACTIONS') == 'true'
    ci_branch = os.environ.get('GITHUB_REF_NAME') == policy['branch'] or os.environ.get('GITHUB_BASE_REF') == policy['branch']
    require(branch == policy['branch'] or (not branch and in_ci and ci_branch), 'wrong branch / CI target')
    require(git(repo, 'rev-parse', base + '^{commit}') == base, 'P0.5A baseline unavailable')
    require(subprocess.run(['git', '-C', str(repo), 'merge-base', '--is-ancestor', base, 'HEAD'], check=False).returncode == 0, 'P0.5A is not ancestor of HEAD')
    # In checkout with tags, pin r0.23. Shallow CI checkouts may omit tags;
    # the P0.5A hash and frozen historical paths are independently checked.
    if subprocess.run(['git', '-C', str(repo), 'rev-parse', '-q', '--verify', 'refs/tags/r0.23'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
        require(git(repo, 'rev-parse', 'r0.23^{commit}') == policy['research_tag_sha'], 'r0.23 tag changed')
    frozen = policy['frozen_historical_paths']
    require(len(frozen) == len(set(frozen)) and frozen, 'invalid frozen paths')
    for path in frozen:
        require(path.startswith('src/') and '..' not in Path(path).parts, 'invalid frozen historical path')
        require((repo / path).exists(), f'missing frozen path: {path}')
    # Worktree comparison includes staged and unstaged changes, not just HEAD.
    changed = subprocess.run(['git', '-C', str(repo), 'diff', '--quiet', base, '--', *frozen], check=False)
    require(changed.returncode == 0, f'historical effect or native I/O sources diverged from certified P0.5A: status {changed.returncode}')
    crate = repo / policy['protected_crate_path']
    lib = code_tokens((crate / 'src/lib.rs').read_text(encoding='utf-8'))
    modules = re.findall(r'(?m)^\s*pub\s+mod\s+(\w+)\s*;', lib)
    require(set(modules) == set(policy['allowed_submodules']) and len(modules) == len(policy['allowed_submodules']), f'new publicly exported production submodule: {modules}')
    declared = sorted(f.stem for f in (crate / 'src').glob('*.rs') if f.name != 'lib.rs')
    require(set(declared) == set(policy['allowed_submodules']), f'new module source introduced: {declared}')
    facade_check((repo / policy['facade_file']).read_text(encoding='utf-8'), set(policy['allowed_facade_methods']))
    production_src = (crate / 'src/production.rs').read_text(encoding='utf-8')
    require('pub struct ProductionEffectBackend' in production_src, 'P0.4 backend removed')
    require('GuardedEffectBackend' in production_src, 'P0.4 no longer wraps P0.3')
    require('fn execute_with_context(' in production_src, 'P0.4 context gate removed')
    legacy = (repo / 'src/effect_dispatch/backend.rs').read_text(encoding='utf-8')
    require('pub trait EffectBackend' in legacy and 'self.execute(&request.queued)' in legacy, 'historical fallback inventory changed: manual review required')
    print('P0.5B FENCED HOST SURFACE LOCK: PASS')
    print('HISTORICAL API BYPASSES: STILL PRESENT (NOT EXPORTED THROUGH THIS FACADE)')
    print('KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED')
    print('PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        check_repo(args.repo.resolve())
        return 0
    except (PolicyError, OSError, ValueError, KeyError, subprocess.CalledProcessError) as exc:
        print(f'P0.5B SURFACE LOCK: FAIL: {exc}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
