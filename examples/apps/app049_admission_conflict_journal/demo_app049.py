"""Actual distinct authenticated declarations, retained and independently replayable."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app049 import Conflict

def main(proof):
    t=Conflict("test_01_real_conflict_preserved")
    t.setUp()
    try:
        before=t.t.snapshot()
        r=t.preserve()
        original_preserved=t.t.snapshot()==before
        journal=t.snapshot()
        duplicate=t.preserve()
        unchanged=t.snapshot()==journal
        a=t.audit()
        t.t.root.rename(t.t.base/"detached-registry")
        detached=t.audit()
        checks=[
          ("CONFLICT_PRESERVED",r["status"]=="CONFLICT_PRESERVED"),
          ("DISTINCT_AUTHENTICATED_ACKS",r["record"]["original_ack_sha256"]!=r["record"]["conflicting_ack_sha256"]),
          ("ORIGINAL_REGISTRY_PRESERVED",original_preserved),
          ("DUPLICATE_BLOCKED",duplicate["status"]=="DUPLICATE_CONFLICT_BLOCKED" and unchanged),
          ("READONLY_AUDIT_VERIFIED",a["status"]=="CONFLICT_VERIFIED" and a["record"]==r["record"]),
          ("DETACHED_AUDIT_VERIFIED",detached==a and t.snapshot()==journal),
          ("NO_FRAUD_OR_COMPLETENESS_OVERCLAIM",not a["fraud_proven"] and a["journal_completeness"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not a["execution_authorized"] and not a["reservation_release_authorized"] and not a["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP049_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP049_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":a},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
