"""Real authenticated journal proof, sealed append-only transition and tamper refusal."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app050 import Seal

def main(proof):
    t=Seal("test_01_real_seal_and_audit")
    t.setUp()
    try:
        old,new=t.pair()
        before=t.t.snapshot()
        audit=t.audit(new["seal_sha256"])
        link=t.transition(old,new)
        unchanged=t.t.snapshot()==before
        p=t.t.folder()
        moved=t.base/"removed-for-demonstration"
        p.rename(moved)
        blocked=False
        try:
            t.audit(new["seal_sha256"])
        except Exception:
            blocked=True
        finally:
            moved.rename(p)
        checks=[
          ("GENESIS_SEALED",old["status"]=="JOURNAL_SEALED" and old["record"]["sequence"]==0),
          ("AUTHENTIC_INVENTORY_SEALED",new["status"]=="JOURNAL_SEALED" and set(new["record"]["entries"])=={t.ident}),
          ("EXACT_PARENT_LINK",new["record"]["parent_sha256"]==old["seal_sha256"]),
          ("APPEND_ONLY_TRANSITION",link["status"]=="SEAL_TRANSITION_VERIFIED" and link["added"]==1),
          ("READONLY_LIVE_AUDIT",audit["status"]=="JOURNAL_SEAL_VERIFIED" and unchanged),
          ("DELETION_BLOCKED",blocked),
          ("NO_GLOBAL_OVERCLAIM",audit["global_antirollback"]=="NOT_PROVEN" and audit["independent_completeness"]=="NOT_PROVEN"),
          ("NO_EXECUTION_AUTHORITY",not audit["execution_authorized"] and not audit["reservation_release_authorized"] and not audit["nonce_reset_authorized"])]
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP050_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP050_DEMO.json").write_text(json.dumps({"assertions":claims,"genesis":old,"verified":audit,"transition":link},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
