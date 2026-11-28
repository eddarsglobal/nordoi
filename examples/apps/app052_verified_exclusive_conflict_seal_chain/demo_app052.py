"""Genuine two-step exclusive chain, pinned endpoints and live-head verification."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app052 import Chain

def main(proof):
    t=Chain("test_01_real_two_step_chain")
    t.setUp()
    try:
        t.extend()
        before=t.t.snapshot()
        offline=t.audit()
        live=t.audit(journal=t.t.t.journal)
        unchanged=t.t.snapshot()==before
        moved=t.base/"removed-chain-decision"
        t.t.folder.rename(moved)
        blocked=False
        try:
            t.audit()
        except Exception:
            blocked=True
        finally:
            moved.rename(t.t.folder)
        checks=[
          ("GENESIS_PIN_VERIFIED",offline["anchor_sha256"]==t.anchor),
          ("TWO_TRANSITIONS_VERIFIED",offline["status"]=="EXCLUSIVE_CHAIN_VERIFIED" and offline["transition_count"]==2),
          ("EXPECTED_HEAD_VERIFIED",offline["head_sha256"]==t.head),
          ("LIVE_HEAD_INVENTORY_VERIFIED",live["head_inventory_verified"] and not offline["head_inventory_verified"]),
          ("MISSING_DECISION_BLOCKED",blocked),
          ("READONLY_CHAIN_AUDIT",unchanged and t.t.snapshot()==before),
          ("NO_GLOBAL_OVERCLAIM",offline["distributed_exclusivity"]=="NOT_PROVEN" and offline["historical_live_replay"]=="NOT_ATTESTED"),
          ("NO_EXECUTION_AUTHORITY",not offline["execution_authorized"] and not offline["reservation_release_authorized"] and not offline["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP052_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP052_DEMO.json").write_text(json.dumps({"assertions":claims,"offline":offline,"live":live},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
