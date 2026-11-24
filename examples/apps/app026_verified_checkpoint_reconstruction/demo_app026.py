"""Demonstration of real native receipt reconstruction, no replay by APP-026."""
from pathlib import Path
import sys
import json
sys.dont_write_bytecode=True
from test_app026 import Reconstruction
import reconstruct_app026 as app

def main(proof):
    t=Reconstruction("test_01_one_extension_real_portable")
    t.setUp()
    try:
        t.f.fresh_partial(1)
        before=t.f.fixture.native_bytes(t.f.root)
        result=t.run_app()
        manifest,_,events=app.portable.verify(t.dest,result["new_checkpoint_sha256"])
        claims=[]
        for name, condition in (
            ("PROPER_EXTENSION", len(events)==3),
            ("THREE_REAL_NATIVE_JOBS", manifest["committed"]==3),
            ("THREE_NODES_REMAIN", result["remaining_nodes"]==3),
            ("ORIGINAL_NATIVE_BYTES", before==t.f.fixture.native_bytes(t.f.root)),
            ("NO_NATIVE_REPLAY", result["native_processes_started"]==0),
            ("EXTERNAL_AUTH_NOT_PROVEN", result["external_pin_authentication"]=="NOT_PROVEN"),
            ("PRODUCTION_DENIED", result["production"]=="NOT_AUTHORIZED"),
            ("NEW_PIN_DIFFERS", result["new_checkpoint_sha256"]!=t.f.pin)):
            assert condition,name
            claims.append("APP026_"+name+"=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP026_DEMO.json").write_text(json.dumps({"result":result,"assertions":claims},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
