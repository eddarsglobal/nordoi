"""Actual native completion, archive verification, tampering rejection."""
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from test_app030 import Archive
import archive_app030 as app

def main(proof):
    t=Archive("test_01_real_complete_archive")
    t.setUp()
    try:
        t.t.finish()
        before=t.f.fixture.native_bytes(t.f.root)
        r=app.archive(t.dest,**t.args())
        v=app.verify(t.dest,r["archive_sha256"])
        claims=[]
        for name,ok in (
          ("ARCHIVE_CREATED",r["status"]=="ARCHIVE_CREATED_VERIFIED"),
          ("OFFLINE_VERIFY",v["status"]=="ARCHIVE_VERIFIED"),
          ("SIX_NATIVE_JOBS",v["verified_native_jobs"]==6),
          ("ORIGINAL_BYTES_PRESERVED",before==t.f.fixture.native_bytes(t.f.root)),
          ("RESERVATION_RETAINED",t.t.claim.is_dir() and r["reservation_retained"]),
          ("NO_RELEASE_AUTHORITY",v["reservation_release_authorized"] is False),
          ("NO_REPLAY_AUTHORITY",v["execution_authorized"] is False and v["automatic_retry"] is False)):
            assert ok,name
            claims.append("APP030_"+name+"=PASS")
        p=t.dest/"reservation"/"COMPLETE.json"
        p.write_bytes(p.read_bytes()+b" ")
        try:
            app.verify(t.dest,r["archive_sha256"])
        except Exception:
            claims.append("APP030_TAMPERED_ARCHIVE_BLOCKED=PASS")
        assert len(claims)==8
        proof.mkdir(parents=True,exist_ok=True)
        (proof/"APP030_DEMO.json").write_text(json.dumps({"assertions":claims,"created":r,"verified":v},indent=2)+"\n",encoding="utf-8")
        for claim in claims: print(claim)
    finally:
        t.doCleanups()

if __name__=="__main__":
    main(Path(sys.argv[1]))
