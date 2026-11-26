"""Real signed conflict journal snapshot with externally pinned digest."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app039 import Seal

def main(proof):
    t=Seal("test_01_real_signed_journal_seal")
    t.setUp()
    try:
        t.t.make_conflict("1"*64)
        t.t.preserve()
        r=t.seal()
        v=t.verify(r["seal_sha256"])
        checks=[
          ("TWO_SIGNED_CONTRADICTIONS",r["entry_count"]==2),
          ("EXTERNAL_PIN_VERIFIED",v["status"]=="JOURNAL_SEAL_VERIFIED"),
          ("INVENTORY_DIGEST_MATCH",r["inventory_sha256"]==v["inventory_sha256"]),
          ("DETERMINISTIC_SEAL",t.seal(output=t.base/"second.json")["seal_sha256"]==r["seal_sha256"]),
          ("ORIGINAL_REGISTRY_PRESERVED",t.t.t.audit()["record"]==t.t.original),
          ("NO_COMPLETENESS_OVERCLAIM",v["journal_completeness"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        t.t.directory.rename(t.base/"removed-conflict")
        try:
            t.verify(r["seal_sha256"])
        except Exception:
            checks.append(("REMOVAL_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP039_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP039_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
