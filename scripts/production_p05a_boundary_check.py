#!/usr/bin/env python3
"""Static source checks for the narrow P0.5A facade. Not semantic proof."""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
module = (root / "production/native-effect-p03/src/enforced.rs").read_text()
legacy = (root / "src/effect_dispatch/backend.rs").read_text()
legacy_io = sorted((root / "src").glob("observable_io_p2*.rs"))
assert "pub struct FencedProductionEffects<Store, Backend, Source>" in module
assert "backend: ProductionEffectBackend<Backend, Source>" in module
assert "self.event_loop.dispatch_next_effect_with_fenced_journal(" in module
assert re.search(r"\bself\s*\.\s*event_loop\s*\.\s*cycle_to_with_fenced_effect_journal\s*\(", module)
assert "event_loop: AtomicEventLoop" in module
assert "pub fn dispatch_next(" in module
assert "pub fn backend_mut" not in module
assert "pub fn into_inner" not in module
assert "impl<" in module
assert "pub trait EffectBackend" in legacy
assert "self.execute(&request.queued)" in legacy
assert len(legacy_io) >= 7
assert not re.search(r"impl\s*<[^>]*>\s*EffectBackend\s+for\s+FencedProductionEffects", module)
print("P0.5A FACADE SOURCE BOUNDARY: PASS")
print("HISTORICAL BYPASS SURFACES REMAIN: CONFIRMED")
print("KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED")
print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")
