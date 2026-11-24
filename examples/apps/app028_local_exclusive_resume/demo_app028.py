"""Real competing native continuations and killed reservation owner."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app028 import Native,Reservation
import exclusive_app028 as app

def main(proof):
    t=Native("test_09_real_native_competing_continuations")
    t.setUp()
    claims=[]
    try:
        ps=[t.launch(),t.launch()]
        results=[json.loads(p.communicate(timeout=300)[0]) for p in ps]
        good=next(x for x in results if x["status"]=="COMPLETE_VERIFIED")
        after=t.f.fixture.native_bytes(t.f.root)
        for name,ok in (
          ("ONE_PROCESS_WINS",sorted(p.returncode for p in ps)==[0,3]),
          ("THREE_NEW_NATIVE_EVENTS",good["continuation"]["new_verified_events"]==3),
          ("SIX_VERIFIED_NATIVE_JOBS",good["continuation"]["verified_native_jobs"]==6),
          ("HISTORICAL_BYTES_PRESERVED",all(after[k]==v for k,v in t.t.before.items())),
          ("RESERVATION_RETAINED",good["reservation_retained"] and t.claim.is_dir()),
          ("NO_AUTOMATIC_RETRY",good["automatic_retry"] is False),
          ("NO_GLOBAL_ANTIROLLBACK",good["global_antirollback"]=="NOT_PROVEN")):
            assert ok,name
            claims.append("APP028_"+name+"=PASS")
    finally:
        t.doCleanups()
    q=Reservation("test_02_killed_owner_never_reclaimed")
    q.setUp()
    try:
        q.test_02_killed_owner_never_reclaimed()
        claims.append("APP028_KILLED_OWNER_NOT_RECLAIMED=PASS")
    finally:
        q.doCleanups()
    assert len(claims)==8
    proof.mkdir(parents=True,exist_ok=True)
    (proof/"APP028_DEMO.json").write_text(json.dumps({"assertions":claims,"results":results},indent=2)+"\n",encoding="utf-8")
    for claim in claims: print(claim)

if __name__=="__main__":
    main(Path(sys.argv[1]))
