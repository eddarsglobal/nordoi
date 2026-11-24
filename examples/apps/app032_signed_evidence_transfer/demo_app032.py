"""Real Ed25519 signature and authentic native evidence."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app032 import SignedTransfer

def main(proof):
    t=SignedTransfer("test_01_actual_signature_and_native_evidence")
    t.setUp()
    try:
        r=t.receive()
        claims=[]
        for name,ok in (
            ("SIGNATURE_VERIFIED",r["crypto_signature"]=="SSHSIG_ED25519_VERIFIED"),
            ("SIX_NATIVE_JOBS",r["verified_native_jobs"]==6),
            ("RECIPIENT_BOUND",r["receiver"]=="agent-b"),
            ("NONCE_MATCH",r["nonce_matches_receiver_expectation"]),
            ("NO_EXECUTION_AUTHORITY",r["execution_authorized"] is False),
            ("NO_RELEASE_AUTHORITY",r["reservation_release_authorized"] is False),
            ("NO_REPLAY_OVERCLAIM",r["nonce_consumption"]=="NOT_IMPLEMENTED" and r["distributed_anti_replay"]=="NOT_PROVEN")):
            assert ok,name
            claims.append("APP032_"+name+"=PASS")
        t.t.receiver=t.base/"rejected"
        t.sign(namespace="wrong-domain")
        t.blocked()
        claims.append("APP032_WRONG_NAMESPACE_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP032_DEMO.json").write_text(json.dumps({"assertions":claims,"result":r},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
