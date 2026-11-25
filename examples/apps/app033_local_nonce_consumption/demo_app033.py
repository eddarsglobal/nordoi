"""Actual signed transfer, two competing recipient processes."""
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from test_app033 import Nonce

def main(proof):
    t=Nonce("test_04_actual_two_process_race")
    t.setUp()
    try:
        targets=[t.t.base/"demo-a",t.t.base/"demo-b"]
        processes=[subprocess.Popen(t.cmd(target=x),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for x in targets]
        try:
            output=[p.communicate(timeout=180) for p in processes]
            assert sorted(p.returncode for p in processes)==[0,3],output
            accepted=json.loads(next(o[0] for p,o in zip(processes,output) if p.returncode==0))
        finally:
            for p in processes:
                if p.poll() is None:
                    p.kill()
                    p.communicate()
        replay=t.process(target=t.t.base/"demo-restart")
        claims=[]
        for name,ok in (
          ("ONE_PROCESS_ACCEPTED",accepted["status"]=="SIGNED_TRANSFER_ACCEPTED_ONCE"),
          ("ONE_DESTINATION",sum(x.exists() for x in targets)==1),
          ("SIX_NATIVE_JOBS",accepted["verified_native_jobs"]==6),
          ("PERSISTENT_RECEIPT",(t.marker()/"COMPLETE.json").is_file()),
          ("RESTART_REPLAY_BLOCKED",replay.returncode==3 and not (t.t.base/"demo-restart").exists()),
          ("NO_EXECUTION_AUTHORITY",accepted["execution_authorized"] is False),
          ("NO_RELEASE_AUTHORITY",accepted["reservation_release_authorized"] is False),
          ("NO_DISTRIBUTED_OVERCLAIM",accepted["distributed_anti_replay"]=="NOT_PROVEN" and accepted["global_antirollback"]=="NOT_PROVEN")):
            assert ok,name
            claims.append("APP033_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP033_DEMO.json").write_text(json.dumps({"assertions":claims,"accepted":accepted},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
