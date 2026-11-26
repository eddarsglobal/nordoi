"""Real recipient-signed acceptance assertion, independently replayed evidence."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app047 import Ack

def main(proof):
    t=Ack("test_01_real_recipient_ack")
    t.setUp()
    try:
        t.complete()
        v=t.verify()
        t.t.relocate()
        moved=t.verify()
        checks=[
          ("RECIPIENT_SIGNATURE_VERIFIED",v["status"]=="RECIPIENT_ACCEPTANCE_SIGNATURE_VERIFIED"),
          ("ED25519_VERIFIED",v["crypto_signature"]=="SSHSIG_ED25519_VERIFIED"),
          ("RECEIPT_PRESERVED",v["record"]==t.t.result["record"]),
          ("RECIPIENT_KEY_BOUND",v["receiver_key_sha256"]==t.receiver_pin),
          ("ORIGINAL_LOCATIONS_NOT_REQUIRED",moved==v),
          ("NO_REMOTE_STATE_OVERCLAIM",v["independent_historical_acceptance"]=="NOT_PROVEN" and v["live_nonce_state"]=="NOT_VERIFIED"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        t.sign(key=t.t.t.t.key)
        try:
            t.verify()
        except Exception:
            checks.append(("SENDER_KEY_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP047_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP047_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
