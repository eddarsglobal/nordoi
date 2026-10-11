#!/usr/bin/env python3
"""Adversarial mutation tests for P0.5B's narrow facade source policy."""
from pathlib import Path
import importlib.util
import sys
sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
source = ROOT / 'scripts/production_p05b_surface_check.py'
spec = importlib.util.spec_from_file_location('p05b_check', source)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
facade = (ROOT / 'production/native-effect-p03/src/enforced.rs').read_text()
allowed = {'new', 'pending_effect_count', 'acquire', 'recover', 'cycle_to', 'dispatch_next', 'release'}
module.facade_check(facade, allowed)

mutants = {
    'public raw backend escape': facade.replace('backend: ProductionEffectBackend<Backend, Source>', 'pub backend: ProductionEffectBackend<Backend, Source>', 1),
    'unprotected backend field': facade.replace('backend: ProductionEffectBackend<Backend, Source>', 'backend: Backend', 1),
    'wrongly raw dispatch method': facade.replace('dispatch_next_effect_with_fenced_journal(', 'dispatch_next_effect(', 1),
    'unfenced cycle': facade.replace('cycle_to_with_fenced_effect_journal(', 'cycle_to(', 1),
    'new public getter': facade + '\nimpl<Store,Backend,Source> FencedProductionEffects<Store,Backend,Source> { pub fn backend_mut(&mut self) {} }\n',
    'direct raw execution': facade + '\nfn sneaky(&mut self) { self.backend.execute_with_context(request); }\n',
    'new exported unsafe API': facade + '\npub struct RawProductionDispatch;\n',
    'filesystem side channel': facade + '\nfn sneaky_file() { std::fs::write("x", "y"); }\n',
    'raw import': facade + '\nuse crate::observable_io_p26;\n',
    'missing fenced backend argument': facade.replace('&mut self.backend,', '&mut other.backend,', 1),
}

for label, mutant in mutants.items():
    assert mutant != facade, f'mutation not applied: {label}'
    try:
        module.facade_check(mutant, allowed)
    except module.PolicyError:
        print('DENIED:', label)
    else:
        raise AssertionError(f'P0.5B failed to detect dangerous mutation: {label}')

print('P0.5B NEGATIVE MUTATION TESTS: 10/10 PASS')
print('NO GLOBAL KERNEL AUTHORIZATION CLAIMED')
