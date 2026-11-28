"""Portable two-transition archive and detached structural replay."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app053 import Archive

def main(proof):
    t=Archive("test_01_real_two_transition_archive")
    t.setUp()
    try:
        t.t.extend()
        before=t.t.t.snapshot()
        packed=t.pack()
        second=t.t.base/"second.json"
        duplicate=t.pack(output=second)
        source_unchanged=t.t.t.snapshot()==before
        for i,p in enumerate((t.t.root,t.t.file,t.t.t.t.journal)):
            p.rename(t.t.base/("detached-demo-"+str(i)))
        verified=t.verify(packed["archive_sha256"])
        raw=t.output.read_bytes()
        pin=t.mutate(lambda d:d["members"][t.t.anchor].pop("CHILD.json"))
        blocked=False
        try:
            t.verify(pin)
        except Exception:
            blocked=True
        finally:
            t.output.write_bytes(raw)
        checks=[
          ("CHAIN_ARCHIVED",packed["status"]=="EXCLUSIVE_CHAIN_ARCHIVED" and packed["transition_count"]==2),
          ("DETERMINISTIC_BYTES",duplicate["archive_sha256"]==packed["archive_sha256"] and second.read_bytes()==raw),
          ("EXACT_MEMBERS",packed["member_count"]==10),
          ("DETACHED_REPLAY",verified["status"]=="EXCLUSIVE_CHAIN_ARCHIVE_VERIFIED" and not verified["original_locations_required"]),
          ("EXPECTED_HEAD",verified["head_sha256"]==t.t.head and verified["decisions_sha256"]==packed["decisions_sha256"]),
          ("MISSING_MEMBER_BLOCKED",blocked),
          ("SOURCE_PRESERVED_NO_OVERCLAIM",source_unchanged and verified["historical_live_replay"]=="NOT_ATTESTED" and not verified["head_inventory_verified"]),
          ("NO_EXECUTION_AUTHORITY",not verified["execution_authorized"] and not verified["reservation_release_authorized"] and not verified["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP053_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP053_DEMO.json").write_text(json.dumps({"assertions":claims,"archive":packed,"verified":verified},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
