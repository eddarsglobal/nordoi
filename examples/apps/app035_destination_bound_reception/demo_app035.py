"""Actual signed native reception and identical-byte substitution refusal."""
import json
from pathlib import Path
import shutil
import sys
sys.dont_write_bytecode=True
from test_app035 import Bound
import bound_app035 as app

def main(proof):
    t=Bound("test_01_real_bound_receipt")
    t.setUp()
    try:
        r=t.receive()
        a=t.audit()
        claims=[]
        for name,ok in (
          ("DESTINATION_BOUND",r["destination_identity"]==app.identity(t.target)),
          ("AUDIT_VERIFIED",a["status"]=="DESTINATION_BOUND_VERIFIED"),
          ("SIX_NATIVE_JOBS",a["verified_native_jobs"]==6),
          ("PATH_BOUND",a["destination"]["path"]==t.issued["challenge"]["destination"]["path"]),
          ("PERSISTENT_RECEIPT",(t.marker/"COMPLETE.json").is_file()),
          ("NO_RESET_AUTHORITY",not a["nonce_reset_authorized"]),
          ("NO_EXECUTION_AUTHORITY",not a["execution_authorized"] and not a["reservation_release_authorized"])):
            assert ok,name
            claims.append("APP035_"+name+"=PASS")
        retained=t.parent/"retained"
        t.target.rename(retained)
        shutil.copytree(retained,t.target)
        try:
            t.audit()
        except Exception:
            claims.append("APP035_IDENTICAL_BYTES_SUBSTITUTION_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP035_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":a},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
