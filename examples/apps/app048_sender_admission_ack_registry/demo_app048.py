"""Actual signed admission acknowledgment and exclusive sender-side receipt."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app048 import Sender

def main(proof):
    t=Sender("test_01_real_acceptance_and_audit")
    t.setUp()
    try:
        t.register()
        pending=t.audit()
        r=t.accept()
        before=t.snapshot()
        duplicate=t.accept()
        conflict=t.accept(**t.alternate())
        preserved=t.snapshot()==before
        audited=t.audit()
        checks=[
          ("EXPECTATION_REGISTERED",pending["status"]=="ACK_PENDING"),
          ("ACK_ACCEPTED_ONCE",r["status"]=="ACK_ACCEPTED_ONCE"),
          ("DUPLICATE_BLOCKED",duplicate["status"]=="DUPLICATE_ACK_BLOCKED"),
          ("AUTHENTIC_CONTRADICTION_BLOCKED",conflict["status"]=="CONTRADICTORY_SIGNED_ACK_BLOCKED"),
          ("FIRST_DECISION_PRESERVED",preserved and audited["record"]==r["record"]),
          ("READONLY_AUDIT_VERIFIED",audited["status"]=="ACK_ACCEPTED_VERIFIED"),
          ("NO_DISTRIBUTED_OVERCLAIM",audited["distributed_anti_replay"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not audited["execution_authorized"] and not audited["reservation_release_authorized"] and not audited["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP048_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP048_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":audited},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
