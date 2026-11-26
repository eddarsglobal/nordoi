"""Actual detached Ed25519 signature and full portable chain replay."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app044 import Signed

def main(proof):
    t=Signed("test_01_real_signed_archive")
    t.setUp()
    try:
        t.complete()
        v=t.verify()
        checks=[
          ("SIGNED_ARCHIVE_VERIFIED",v["status"]=="SIGNED_CHAIN_ARCHIVE_VERIFIED"),
          ("ED25519_VERIFIED",v["crypto_signature"]=="SSHSIG_ED25519_VERIFIED"),
          ("SENDER_KEY_BOUND",v["sender_key_sha256"]==t.key_pin),
          ("EXPECTED_HEAD_VERIFIED",v["head_sha256"]==t.t.t.head),
          ("PORTABLE_CHAIN_REPLAYED",v["transition_count"]==2),
          ("NO_FRESHNESS_OVERCLAIM",v["freshness"]=="NOT_PROVEN" and v["nonce_consumption"]=="NOT_IMPLEMENTED"),
          ("NO_EXECUTION_AUTHORITY",not v["execution_authorized"] and not v["reservation_release_authorized"] and not v["nonce_reset_authorized"])]
        t.sign(namespace="wrong-domain")
        try:
            t.verify()
        except Exception:
            checks.append(("WRONG_NAMESPACE_BLOCKED",True))
        claims=[]
        for name,ok in checks:
            assert ok,name
            claims.append("APP044_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP044_DEMO.json").write_text(json.dumps({"assertions":claims,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
