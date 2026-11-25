"""Real signed receipt audit, no change to source evidence."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app034 import Audit
import audit_app034 as app

def main(proof):
    t=Audit("test_02_complete_real_native_signature")
    t.setUp()
    try:
        issued=t.check()
        r=t.accepted()
        def snapshot():
            return {str(p):p.read_bytes() for p in t.t.t.base.rglob("*") if p.is_file()}
        before=snapshot()
        verified=t.check()
        claims=[]
        for name,ok in (
          ("ISSUED_CLASSIFIED",issued["status"]=="ISSUED"),
          ("COMPLETE_VERIFIED",r["status"]=="CONSUMED_VERIFIED"),
          ("SIGNATURE_REVERIFIED",r["signature_reverified"]),
          ("SIX_NATIVE_JOBS",r["verified_native_jobs"]==6),
          ("READONLY_EVIDENCE",verified==r and snapshot()==before),
          ("NO_RESET_AUTHORITY",not r["nonce_reset_authorized"]),
          ("NO_EXECUTION_AUTHORITY",not r["execution_authorized"] and not r["reservation_release_authorized"])):
            assert ok,name
            claims.append("APP034_"+name+"=PASS")
        p=t.marker/"COMPLETE.json"
        p.write_bytes(p.read_bytes()+b" ")
        t.blocked()
        claims.append("APP034_FORGED_RECEIPT_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP034_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":r},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
