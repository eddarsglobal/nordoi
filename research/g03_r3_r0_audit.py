#!/usr/bin/env python3
"""G0.3 R3: pinned, read-only source-surface audit; NOT a usability study."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

REMOTE = 'https://github.com/eddarsglobal/nordoi.git'
BASE = '7442b06238dfa9351ae932c0cc636c0c99c3dfea'
PRODUCTION = 'e2898bf7c511e53636ff28c9fd3efc93165a0215'
PINS = {
 'docs/NORDOI_G02_FIRST_USE_QUICKSTART.md': '0aa6b2af0d67c55b9d70d89d9b8b0fb5d970646e',
 'examples/p2_1_console/hello.noi': 'e1b74cdabb63716744832c518e9e81e41aa3f14b',
 'examples/p2_2_dynamic/value.noi': 'd7bbbc3a73c158bddac347bc0a46db359d07ed30',
 'examples/p2_3_structured/value.noi': '12fa00f5f9f17b22fcebd77aa5f957fadf4ca397',
 'examples/p2_4_multi/value.noi': 'aff1422903d4be3214b12391990d00d101cc1798',
 'examples/p2_5_file/report.noi': '857dcfc7fb2eb247d9c8a9479e19430d8b34c94b',
 'examples/p2_6_dynamic_file/report.noi': '36a0251675ff7870e6f4f152d7bcf2f4b41bebab',
 'examples/p2_7_bundle/report.noi': '8cf6f2dae53340cb44b212806103860810f83719',
 'governance/future_native_gate_v1.tsv': '1ec1125f614c28cd8a51660975544cb6aa93577a',
 'laws/LAW_0001_NORDOI_MASTER_LAW.md': 'e599795d88e964042a02cfa52d5bf72f1a0750cd',
 'research/LANGUAGE_INTELLIGENCE_CHARTER.md': '1f76027a9853e9fa77417f1a0b6363bcea3ecc41',
}
KEYWORDS = ('module','effect','entry','emits','input','const','writes','returns','fn','import')
PRIMARY = 'cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console'

def git_blob(data):
 return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\x00' + data).hexdigest()

def metrics(data):
 text = data.decode('utf-8', errors='strict')
 tokens = re.findall(r'[A-Za-z_][A-Za-z_0-9]*', text)
 return {'bytes_utf8': len(data), 'nonblank_lines': sum(bool(line.strip()) for line in text.splitlines()),
         'distinct_keyword_types': sorted(set(tokens).intersection(KEYWORDS)),
         'semicolons': text.count(';')}

def verify_local(root):
 out = {}
 for rel,pin in PINS.items():
  file = root / rel
  if file.is_symlink() or not file.is_file():
   raise RuntimeError('MISSING_OR_SYMLINK: ' + rel)
  data = file.read_bytes()
  if git_blob(data) != pin:
   raise RuntimeError('SOURCE_BLOB_MISMATCH: ' + rel)
  out[rel] = data
 guide = out['docs/NORDOI_G02_FIRST_USE_QUICKSTART.md'].decode('utf-8')
 if PRIMARY not in guide or 'declaration alone is not authorization' not in guide or 'V0.1 HALT-only' not in guide:
  raise RuntimeError('R2_FIRST_USE_AUTHORITY_DRIFT')
 if b'effect ConsoleWrite;' not in out['examples/p2_1_console/hello.noi']:
  raise RuntimeError('SOURCE_EFFECT_DRIFT')
 return out

def git(*args):
 env = {**os.environ, 'GIT_TERMINAL_PROMPT': '0'}
 r = subprocess.run(['git',*args], capture_output=True, text=True, env=env, timeout=100, check=True)
 return r.stdout.strip()

def verify_remote():
 for branch, expected in [('main',BASE),('production/p0.1',PRODUCTION),('candidate/g03-r2-beginner-comprehension-20261106',BASE)]:
  result = git('ls-remote','--exit-code',REMOTE,'refs/heads/'+branch)
  sha = result.split()[0] if result else ''
  if sha != expected:
   raise RuntimeError(f'REMOTE_PIN_MISMATCH_{branch}: {sha}')
 print('G03_R3_REMOTE_MAIN_STUDY_PRODUCTION=PASS')

def selftest():
 hello = b'module app.main;\n\neffect ConsoleWrite;\n\nentry main emits "Hello from NORDOI P2.1!\\n";\n'
 if len(hello) != 86 or git_blob(hello) != PINS['examples/p2_1_console/hello.noi']:
  raise RuntimeError('HELLO_PIN_FAILED')
 m = metrics(hello)
 if m['nonblank_lines'] != 3 or m['semicolons'] != 3 or m['distinct_keyword_types'] != ['effect','emits','entry','module']:
  raise RuntimeError('METRIC_SELFTEST_FAILED')
 print('G03_R3_METRICS_SELFTEST=PASS')
 print('G03_R3_SOURCE_GIT_BLOB_SELFTEST=PASS')

def main():
 p = argparse.ArgumentParser(description=__doc__)
 p.add_argument('mode', choices=['selftest','check','run'])
 p.add_argument('--output', type=Path, help='Exclusive private JSON report path (run only)')
 a = p.parse_args()
 selftest()
 if a.mode == 'selftest': return
 verify_remote()
 if a.mode == 'check': return
 if not a.output: p.error('run requires --output')
 if a.output.is_symlink() or a.output.exists(): raise RuntimeError('OUTPUT_EXISTS_OR_SYMLINK')
 if not a.output.parent.is_dir() or a.output.parent.is_symlink(): raise RuntimeError('INVALID_OUTPUT_DIR')
 with tempfile.TemporaryDirectory(prefix='nordoi-g03-r3-isolated-') as d:
  repo = Path(d)/'repo'
  git('clone','--quiet','--depth','1','--single-branch','--branch','main',REMOTE,str(repo))
  if git('-C',str(repo),'rev-parse','HEAD') != BASE: raise RuntimeError('CLONE_HEAD_PIN_MISMATCH')
  sources=verify_local(repo)
  entries={p:metrics(data) for p,data in sorted(sources.items()) if p.startswith('examples/')}
  result={'schema':'NORDOI-G03-R3-SURFACE-AUDIT-v1','base_commit':BASE,
    'verified_blobs':len(sources),'examples':entries,
    'baseline':'CURRENT_EXECUTABLE_P2_1_SOURCE_UNCHANGED',
    'alternatives':{'A':'EXISTING_GUIDE','B':'PREBUILT_BINARY_DOCUMENTATION_ONLY',
                    'C':'GUIDED_FIRST_USE_TOOL_CONCEPT_ONLY','D':'NEW_GRAMMAR_HOLD_NOT_EXECUTABLE'},
    'cognitive_validity':'PROXY_METRICS_NOT_HUMAN_EVIDENCE',
    'language_intelligence':'EXTERNAL_COMPARISONS_DESCRIPTIVE_NOT_PERFORMANCE_BENCHMARKS',
    'r2_study':'INCOMPLETE_A1_B0_HUMAN_PROVENANCE_UNVERIFIED',
    'recommendation':'B_DOCS_FIRST_PROPOSAL_NOT_VALIDATED',
    'new_syntax':'NONE','source_modifications':'NONE','github_push':'NONE','production':'NOT_AUTHORIZED'}
  blob = (json.dumps(result,ensure_ascii=False,indent=2,sort_keys=True)+'\n').encode()
  fd = os.open(str(a.output),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
  with os.fdopen(fd,'wb') as f: f.write(blob)
  print('G03_R3_EXAMPLES=7_VERIFIED')
  print('G03_R3_SOURCE_BLOBS=11/11_PASS')
  print('G03_R3_FIRST_PROGRAM_BYTES='+str(entries['examples/p2_1_console/hello.noi']['bytes_utf8']))
  print('G03_R3_HUMAN_USABILITY=UNMEASURED')
  print('G03_R3_DECISION=B_DOCS_FIRST_DESIGN_ONLY')
  print('G03_R3_REPORT='+str(a.output))
  print('PRODUCTION=NOT_AUTHORIZED')

if __name__=='__main__':
 try: main()
 except Exception as exc:
  print('G03_R3_ERROR='+str(exc))
  raise SystemExit(1)
