"""Real two-transition chain reconstructed from historical signed evidence."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app042 import Chain

def main(proof):
    t=Chain("test_01_real_two_transition_chain")
    t.setUp()
    try:
        v=t.audit()
        checks=[
          ("CHAIN_VERIFIED",v["status"]=="LOCAL_CHAIN_VERIFIED"),
          ("ANCHOR_BOUND",v["anchor_sha256"]==t.anchor),
          ("HEAD_IDENTIFIED",v["head_sha256"]==t.head),
          ("TWO_TRANSITIONS_VERIFIED",v["transition_count"]==2),
          ("DETERMINISTIC_AUDIT",t.audit()==v),
          ("NO_GLOBAL_OVERCLAIM",v["distributed_consensus"]=="NOT_PROVEN" and v["global_antirollback"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        (t.t.successors/("f"*64)).mkdir()
        try:
            t.audit()
        except Exception:
            checks.append(("ORPHAN_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP042_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP042_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
