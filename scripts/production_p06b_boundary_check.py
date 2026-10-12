#!/usr/bin/env python3
from __future__ import annotations
import argparse, os, re, subprocess, sys, tomllib
from pathlib import Path

BASE="8ec9b694a6759ee22f162667d4af3f7ed5c3bc37"
BRANCH="production/p0.1"
R023="0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5"

class BoundaryError(RuntimeError): pass

def require(v: bool, msg: str):
    if not v: raise BoundaryError(msg)

def git(repo: Path,*args:str)->str:
    return subprocess.check_output(["git","-C",str(repo),*args],text=True,stderr=subprocess.PIPE).strip()

def main()->int:
    ap=argparse.ArgumentParser(); ap.add_argument("--repo",default="."); args=ap.parse_args()
    root=Path(args.repo).resolve()
    branch=git(root,"branch","--show-current")
    in_ci=os.environ.get("GITHUB_ACTIONS")=="true"
    ci_branch=os.environ.get("GITHUB_REF_NAME")==BRANCH or os.environ.get("GITHUB_BASE_REF")==BRANCH
    require(branch==BRANCH or (not branch and in_ci and ci_branch),"wrong branch / CI target")
    require(git(root,"rev-parse",BASE+"^{commit}")==BASE,"P0.6A base unavailable")
    require(subprocess.run(["git","-C",str(root),"merge-base","--is-ancestor",BASE,"HEAD"],check=False).returncode==0,"P0.6A is not ancestor")
    if subprocess.run(["git","-C",str(root),"rev-parse","-q","--verify","refs/tags/r0.23"],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0:
        require(git(root,"rev-parse","r0.23^{commit}")==R023,"r0.23 changed")

    p03_manifest=tomllib.loads((root/"production/native-effect-p03/Cargo.toml").read_text())
    p06_manifest=tomllib.loads((root/"production/native-rollback-p06/Cargo.toml").read_text())
    p03_deps=set(p03_manifest.get("dependencies",{}))
    require(p03_deps=={"nordoi_kernel","nordoi-native-trust-p01","nordoi-rollback-anchor-p06"},f"P0.3 dependency set unexpected: {sorted(p03_deps)}")
    require(set(p06_manifest.get("dependencies",{}))==set(),"P0.6A contract gained normal dependencies")

    lib=(root/"production/native-effect-p03/src/lib.rs").read_text()
    prod=(root/"production/native-effect-p03/src/production.rs").read_text()
    enforced=(root/"production/native-effect-p03/src/enforced.rs").read_text()
    p06=(root/"production/native-rollback-p06/src/lib.rs").read_text()
    tests=(root/"production/native-effect-p03/tests/p06b_anchor.rs").read_text()

    for token in ["pub(crate) fn admit_with_context(","pub(crate) fn execute_admitted_with_context(","self.admit_with_context(request)?;","self.execute_admitted_with_context(request)"]:
        require(token in lib,f"missing P0.3 split invariant: {token}")
    for token in ["anchor: Option<Box<dyn MonotonicAnchor>>","impl<B, S> ProductionWitnessSource for ProductionEffectBackend<B, S>","fn anchor_barrier(&mut self)","verify_production_recovery(&mut *anchor, &*self)"]:
        require(token in prod,f"missing P0.6B backend invariant: {token}")
    for token in ["AnchorOpenMode","MonotonicAnchor","backend.attach_anchor(anchor, anchor_mode)?;","Result<Self, RecoveryError>"]:
        require(token in enforced,f"missing P0.6B facade invariant: {token}")
    require("pub enum AnchorOpenMode" in p06,"anchor open mode missing")
    require("A: MonotonicAnchor + ?Sized" in p06,"trait-object recovery support missing")

    # Enforce the critical source order inside P0.4 execute_with_context.
    start=prod.index("impl<B: EffectBackend, S: SignedEnvelopeSource> EffectBackend for ProductionEffectBackend")
    body=prod[start:]
    tokens=[
        "self.ledger.reserve(delivery, binding_hash)",
        "self.inner.admit_with_context(request)?;",
        "self.anchor_barrier()?;",
        "self.inner.execute_admitted_with_context(request)?;",
        ".acknowledged(&delivery, &binding_hash)",
    ]
    positions=[]
    cursor=0
    for token in tokens:
        pos=body.find(token,cursor)
        require(pos>=0,f"dispatch ordering token missing: {token}")
        positions.append(pos); cursor=pos+len(token)
    final_barrier=body.find("self.anchor_barrier()?;",positions[-1]+1)
    require(final_barrier>positions[-1],"post-ACK anchor barrier missing")
    require(positions==sorted(positions),"dispatch ordering changed")

    require(tests.count("#[test]")==6,f"P0.6B test inventory changed: {tests.count('#[test]')}")
    for name in [
        "successful_dispatch_anchors_before_and_after_external_effect",
        "pre_effect_anchor_failure_blocks_destination",
        "post_effect_anchor_failure_never_reexecutes_same_delivery",
        "recovery_mode_rejects_missing_anchor_before_facade_exists",
        "initialize_mode_rejects_already_initialized_anchor",
        "exact_recovery_then_dispatch_uses_per_dispatch_barriers",
    ]:
        require(name in tests,f"missing P0.6B test: {name}")

    # P0.5A/P0.5B facade shape remains closed: same fields and same public method names.
    struct=re.search(r"pub struct FencedProductionEffects<Store, Backend, Source>\s*\{([^{}]+)\}",enforced,re.S)
    require(struct is not None,"fenced facade missing")
    fields=set(re.findall(r"(?m)^\s*(\w+)\s*:",struct.group(1)))
    require(fields=={"event_loop","journal","dispatcher","backend"},f"facade fields changed: {fields}")
    methods=re.findall(r"\bpub\s+fn\s+(\w+)\s*(?:<|\()",enforced)
    require(set(methods)=={"new","pending_effect_count","acquire","recover","cycle_to","dispatch_next","release"},f"facade public methods changed: {methods}")

    print("P0.6B PER-DISPATCH ANCHOR SOURCE BOUNDARY: PASS")
    print("P0.6B TEST INVENTORY: 6")
    print("P0.5A/P0.5B FACADE SHAPE: PRESERVED")
    print("P0.5C DIRECT PROFILE BOUNDARY: PRESERVED")
    print("PER-DISPATCH ANCHOR BARRIER: INTEGRATED")
    print("INDEPENDENT ANCHOR BACKEND QUALITY: NOT CERTIFIED")
    print("KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED")
    print("PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED")
    return 0

if __name__=="__main__":
    try: raise SystemExit(main())
    except (BoundaryError,OSError,ValueError,KeyError,subprocess.CalledProcessError,tomllib.TOMLDecodeError) as exc:
        print(f"P0.6B BOUNDARY: FAIL: {exc}",file=sys.stderr); raise SystemExit(1)
