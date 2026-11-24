"""Genuine native evidence exported and verified by a separate receiver process."""
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from test_app031 import Transfer
import transfer_app031 as app

def main(proof):
    t=Transfer("test_01_real_receiver_separate_process")
    t.setUp()
    try:
        t.prepare()
        before=t.packet.read_bytes()
        t.archive.rename(t.base/"unavailable-original")
        cmd=[sys.executable,"-B",str(Path(__file__).parent/"transfer_app031.py"),"receive",
             "--packet",str(t.packet),"--transfer-pin",t.tp,"--archive-pin",t.ap,
             "--target",str(t.receiver)]
        p=subprocess.run(cmd,capture_output=True,timeout=120)
        assert p.returncode==0,p.stdout+p.stderr
        r=json.loads(p.stdout)
        claims=[]
        for name,ok in (
          ("RECEIVER_PROCESS_VERIFIED",r["status"]=="TRANSFER_RECEIVED_VERIFIED"),
          ("ORIGINAL_UNAVAILABLE",not t.archive.exists()),
          ("SIX_NATIVE_JOBS",r["verified_native_jobs"]==6),
          ("PACKET_UNCHANGED",before==t.packet.read_bytes()),
          ("NO_EXECUTION_AUTHORITY",r["execution_authorized"] is False),
          ("NO_RELEASE_AUTHORITY",r["reservation_release_authorized"] is False),
          ("NO_UNPROVEN_SECURITY_CLAIMS",not r["sender_identity_authenticated"] and not r["transport_encrypted"] and r["distributed_anti_replay"]=="NOT_PROVEN")):
            assert ok,name
            claims.append("APP031_"+name+"=PASS")
        t.receiver=t.base/"rejected"
        bad=t.mutate(lambda o:o["files"][0].update(path="../escape"))
        t.blocked(transfer_pin=bad)
        claims.append("APP031_TRAVERSAL_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP031_DEMO.json").write_text(json.dumps({"assertions":claims,"receiver":r},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
