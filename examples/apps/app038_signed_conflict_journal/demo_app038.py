"""Actual signed conflicting acknowledgments and permanent separate evidence."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app038 import Conflict

def main(proof):
    t=Conflict("test_01_real_signed_contradiction")
    t.setUp()
    try:
        r=t.preserve()
        a=t.audit()
        duplicate=t.preserve()
        record=r["record"]
        checks=(
            ("SIGNED_CONTRADICTION_RECORDED",r["status"]=="SIGNED_CONTRADICTION_RECORDED"),
            ("BOTH_SIGNATURES_STORED",all((t.directory/(p+"-ACK.sig")).is_file() for p in ("original","conflict"))),
            ("DISTINCT_ACK_PINS",record["accepted_ack_sha256"]!=record["conflicting_ack_sha256"]),
            ("READONLY_AUDIT_VERIFIED",a["status"]=="SIGNED_CONTRADICTION_VERIFIED" and a["record"]==record),
            ("DUPLICATE_CONFLICT_BLOCKED",duplicate["status"]=="CONTRADICTION_ALREADY_RECORDED"),
            ("ORIGINAL_RECORD_PRESERVED",t.t.audit()["record"]==t.original),
            ("NO_COMPLETENESS_OVERCLAIM",a["journal_completeness"]=="NOT_PROVEN"),
            ("NO_EXECUTION_AUTHORITY",not a["execution_authorized"] and not a["reservation_release_authorized"] and not a["nonce_reset_authorized"]))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP038_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP038_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":a},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
