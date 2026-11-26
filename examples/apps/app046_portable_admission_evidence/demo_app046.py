"""Portable evidence from a real authenticated one-time acceptance."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app046 import Evidence

def main(proof):
    t=Evidence("test_01_real_acceptance_evidence")
    t.setUp()
    try:
        t.accept()
        p=t.pack()
        v=t.verify(p["package_sha256"])
        t.relocate()
        moved=t.verify(p["package_sha256"])
        checks=[
          ("PACKAGE_CREATED",p["status"]=="ADMISSION_EVIDENCE_PACKED"),
          ("PACKAGE_VERIFIED",v["status"]=="ADMISSION_EVIDENCE_VERIFIED"),
          ("RECEIPT_PIN_BOUND",v["receipt_sha256"]==t.receipt_pin),
          ("RECEIPT_PRESERVED",v["record"]==t.result["record"]),
          ("ORIGINAL_LOCATIONS_NOT_REQUIRED",moved==v and not v["original_locations_required"]),
          ("NO_HISTORICAL_OVERCLAIM",v["independent_historical_acceptance"]=="NOT_PROVEN"),
          ("NO_LIVE_STATE_OVERCLAIM",v["live_nonce_state"]=="NOT_VERIFIED"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP046_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP046_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
