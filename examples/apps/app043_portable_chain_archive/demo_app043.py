"""Portable replay of an actual two-transition signed chain."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app043 import Archive

def main(proof):
    t=Archive("test_01_real_portable_replay")
    t.setUp()
    try:
        a=t.create()
        v=t.verify(a["archive_sha256"])
        same=t.create(output=t.base/"second-archive.json")
        t.t.t.successors.rename(t.base/"detached-successors")
        t.t.first_journal.rename(t.base/"detached-history")
        t.t.plan.rename(t.base/"detached-plan")
        moved=t.verify(a["archive_sha256"])
        checks=[
          ("ARCHIVE_CREATED",a["status"]=="CHAIN_ARCHIVE_CREATED"),
          ("REPLAY_VERIFIED",v["status"]=="CHAIN_ARCHIVE_VERIFIED"),
          ("TWO_TRANSITIONS_PRESERVED",v["transition_count"]==2),
          ("HEAD_PRESERVED",v["head_sha256"]==t.t.head),
          ("DETERMINISTIC_ARCHIVE",same["archive_sha256"]==a["archive_sha256"]),
          ("ORIGINAL_LOCATIONS_NOT_REQUIRED",moved==v and not v["original_locations_required"]),
          ("NO_GLOBAL_OVERCLAIM",v["global_antirollback"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP043_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP043_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
