"""Real native completion followed by read-only audit and receipt tampering."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app029 import ReservationAudit

def main(proof):
    t=ReservationAudit("test_01_complete_real_native_evidence")
    t.setUp()
    try:
        t.finish()
        before=t.f.fixture.native_bytes(t.f.root)
        r=t.call()
        claims=[]
        for name,ok in (
          ("COMPLETE_NATIVE_EVIDENCE",r["status"]=="COMPLETE_EVIDENCE_VERIFIED"),
          ("SIX_NATIVE_JOBS",r["verified_native_jobs"]==6),
          ("HISTORY_PRESERVED",r["historical_events"]==3 and r["new_verified_events"]==3),
          ("NO_NATIVE_EXECUTION",r["native_processes_started"]==0),
          ("NO_RELEASE_AUTHORITY",r["reservation_release_authorized"] is False),
          ("NO_RETRY_AUTHORITY",r["execution_authorized"] is False and r["automatic_retry"] is False),
          ("NATIVE_BYTES_UNCHANGED",before==t.f.fixture.native_bytes(t.f.root))):
            assert ok,name
            claims.append("APP029_"+name+"=PASS")
        t.mutate(t.claim/"COMPLETE.json",lambda x:x["continuation"].update(verified_native_jobs=99))
        bad=t.blocked()
        claims.append("APP029_FORGED_COMPLETION_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP029_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":r,"tampered":bad},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
