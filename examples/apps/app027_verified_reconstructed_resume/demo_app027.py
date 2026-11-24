"""Demonstrate actual native completion from a reconstructed checkpoint."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app027 import Continuation

def main(proof):
    t=Continuation("test_01_three_missing_native_jobs")
    t.setUp()
    try:
        t.prepare()
        r=t.execute()
        after=t.f.fixture.native_bytes(t.f.root)
        rows=[json.loads(x) for x in (t.attempt/"native-suffix.jsonl").read_bytes().splitlines()]
        claims=[]
        for name, ok in (
            ("HISTORICAL_THREE_VERIFIED",r["historical_events"]==3),
            ("THREE_NEW_NATIVE_EVENTS",r["new_verified_events"]==3),
            ("ONLY_STEPS_4_5_6",[x["step"] for x in rows]==[4,5,6]),
            ("HISTORICAL_BYTES_PRESERVED",all(after[k]==v for k,v in t.before.items())),
            ("FULL_DAG_VERIFIED",r["status"]=="COMPLETE_VERIFIED" and r["verified_native_jobs"]==6),
            ("LOCAL_ATTEMPT_RECEIPTS",(t.attempt/"START.json").is_file() and (t.attempt/"COMPLETE.json").is_file()),
            ("NO_AUTOMATIC_RETRY",r["automatic_retry"] is False),
            ("NO_GLOBAL_ANTIROLLBACK",r["global_antirollback"]=="NOT_PROVEN")):
            assert ok,name
            claims.append("APP027_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP027_DEMO.json").write_text(json.dumps({"result":r,"assertions":claims},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
