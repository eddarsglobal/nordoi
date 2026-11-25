"""Real recipient signature over a locally verified reception receipt."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app036 import Ack

def main(proof):
    t=Ack("test_01_real_signed_ack")
    t.setUp()
    try:
        r,p=t.complete()
        v=t.verify()
        claims=[]
        for name,ok in (
          ("RECIPIENT_SIGNATURE_VERIFIED",v["status"]=="RECIPIENT_ACK_SIGNATURE_VERIFIED"),
          ("RECEIPT_DIGEST_BOUND",v["receipt_sha256"]==p["receipt_sha256"]),
          ("CHALLENGE_BOUND",v["challenge_sha256"]==t.challenge_pin),
          ("SIX_NATIVE_JOBS",v["recipient_asserted_native_jobs"]==6),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"]),
          ("NO_REMOTE_STATE_OVERCLAIM",not v["remote_filesystem_independently_verified"] and not v["current_remote_state_proven"]),
          ("NO_NONCE_RESET",not v["nonce_reset_authorized"])):
            assert ok,name
            claims.append("APP036_"+name+"=PASS")
        t.sign(key=t.t.t.private)
        try:
            t.verify()
        except Exception:
            claims.append("APP036_WRONG_SIGNER_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP036_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
