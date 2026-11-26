"""Actual recipient signature and sender-side exclusive acknowledgment registry."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app037 import Registry
import registry_app037 as app

def main(proof):
    t=Registry("test_01_real_ack_accept_and_audit")
    t.setUp()
    try:
        pending=t.audit()
        r=t.accept()
        a=t.audit()
        duplicate=t.accept()
        o=json.loads(t.t.ack.read_bytes())
        o["receipt_sha256"]="0"*64
        t.t.ack.write_bytes(app.portable.serialize(o))
        t.t.sign()
        contradiction=t.accept()
        checks=(
            ("EXPECTED_CONTEXT_REGISTERED",pending["status"]=="ACK_PENDING"),
            ("SIGNED_ACK_ACCEPTED",r["status"]=="ACK_ACCEPTED_ONCE"),
            ("AUDIT_VERIFIED",a["status"]=="ACK_ACCEPTED_VERIFIED" and a["record"]==r["record"]),
            ("SIX_NATIVE_JOBS",r["record"]["recipient_asserted_native_jobs"]==6),
            ("DUPLICATE_BLOCKED",duplicate["status"]=="DUPLICATE_ACK_BLOCKED"),
            ("SIGNED_CONTRADICTION_BLOCKED",contradiction["status"]=="CONTRADICTORY_SIGNED_ACK_BLOCKED"),
            ("ORIGINAL_RECORD_PRESERVED",t.audit()["record"]==r["record"]),
            ("NO_EXECUTION_AUTHORITY",not a["execution_authorized"] and not a["reservation_release_authorized"] and not a["nonce_reset_authorized"]))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP037_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP037_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":a,"contradiction":contradiction},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
