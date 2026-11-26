"""Two real valid successor seals, one permanent local decision."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app041 import Successor

def main(proof):
    t=Successor("test_01_real_successor_decision")
    t.setUp()
    try:
        a,b=t.alternatives()
        r=t.accept(**a)
        blocked=t.accept(**b)
        duplicate=t.accept(**a)
        v=t.audit(**a)
        checks=[
          ("VALID_SUCCESSOR_ACCEPTED",r["status"]=="SUCCESSOR_ACCEPTED_ONCE"),
          ("VALID_ALTERNATIVE_BLOCKED",blocked["status"]=="ALTERNATIVE_SUCCESSOR_BLOCKED"),
          ("DUPLICATE_BLOCKED",duplicate["status"]=="DUPLICATE_SUCCESSOR_BLOCKED"),
          ("FIRST_DECISION_PRESERVED",blocked["record"]==r["record"]==v["record"]),
          ("READONLY_AUDIT_VERIFIED",v["status"]=="SUCCESSOR_DECISION_VERIFIED"),
          ("DISTINCT_SUCCESSORS",blocked["rejected_successor_sha256"]!=r["record"]["successor_seal_sha256"]),
          ("NO_DISTRIBUTED_OVERCLAIM",v["distributed_fork_prevention"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP041_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP041_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v,"blocked":blocked},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
