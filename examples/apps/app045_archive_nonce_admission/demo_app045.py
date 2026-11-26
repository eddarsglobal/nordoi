"""Actual signed archive accepted once under a recipient-issued challenge."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app045 import Admission

def main(proof):
    t=Admission("test_01_real_acceptance_and_audit")
    t.setUp()
    try:
        r=t.accept()
        v=t.audit()
        checks=[
          ("CHALLENGE_BOUND",r["record"]["challenge_sha256"]==t.issued["challenge_sha256"]),
          ("ARCHIVE_ACCEPTED_ONCE",r["status"]=="SIGNED_ARCHIVE_ACCEPTED_ONCE"),
          ("RECEIPT_VERIFIED",v["status"]=="ARCHIVE_ACCEPTANCE_VERIFIED"),
          ("DECISION_PRESERVED",v["record"]==r["record"]),
          ("HEAD_PRESERVED",v["record"]["head_sha256"]==t.t.t.t.head),
          ("NO_DISTRIBUTED_OVERCLAIM",v["distributed_anti_replay"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        try:
            t.accept()
        except Exception:
            checks.append(("REPLAY_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP045_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP045_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
