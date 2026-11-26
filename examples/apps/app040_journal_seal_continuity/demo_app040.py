"""Real signed evidence, append-only seal transition and removal refusal."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app040 import Continuity

def main(proof):
    t=Continuity("test_01_real_append_only_transition")
    t.setUp()
    try:
        t.extend()
        r=t.publish()
        v=t.verify(r["transition_sha256"])
        checks=[
          ("PREVIOUS_ENTRY_PRESERVED",v["preserved_entry_count"]==1),
          ("NEW_ENTRY_VERIFIED",v["new_entry_count"]==1),
          ("PREVIOUS_SEAL_BOUND",v["previous_seal_sha256"]==t.old["seal_sha256"]),
          ("SUCCESSOR_SEAL_BOUND",v["successor_seal_sha256"]==t.new["seal_sha256"]),
          ("TRANSITION_VERIFIED",v["status"]=="SEAL_TRANSITION_VERIFIED"),
          ("NO_FORK_OVERCLAIM",v["fork_prevention"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        t.old_entry.rename(t.t.base/"removed-old")
        try:
            t.verify(r["transition_sha256"])
        except Exception:
            checks.append(("HISTORICAL_REMOVAL_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP040_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP040_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
