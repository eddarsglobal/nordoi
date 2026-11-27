"""Actual verified successor, competing valid branch and retained read-only audit."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app051 import Successor

def main(proof):
    t=Successor("test_01_real_admission_and_live_audit")
    t.setUp()
    try:
        r=t.admit()
        before=t.snapshot()
        duplicate=t.admit()
        competing=t.admit(**t.alternate())
        live=t.audit(journal=t.t.journal)
        offline=t.audit()
        checks=[
          ("SUCCESSOR_ADMITTED",r["status"]=="SUCCESSOR_ADMITTED_ONCE"),
          ("LIVE_INVENTORY_VERIFIED",r["live_inventory_verified"] and live["live_inventory_verified"]),
          ("DUPLICATE_BLOCKED",duplicate["status"]=="DUPLICATE_SUCCESSOR_BLOCKED"),
          ("COMPETING_BRANCH_BLOCKED",competing["status"]=="COMPETING_SUCCESSOR_BLOCKED"),
          ("FIRST_DECISION_PRESERVED",t.snapshot()==before and live["record"]==r["record"]),
          ("OFFLINE_AUDIT_VERIFIED",offline["status"]=="SUCCESSOR_VERIFIED" and not offline["live_inventory_verified"] and offline["record"]==r["record"]),
          ("NO_GLOBAL_OVERCLAIM",offline["distributed_exclusivity"]=="NOT_PROVEN" and offline["global_antirollback"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not offline["execution_authorized"] and not offline["reservation_release_authorized"] and not offline["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP051_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP051_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":offline,"competing":competing},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
