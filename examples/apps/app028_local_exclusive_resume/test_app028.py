"""Real multiprocess reservation races and genuine Rust continuation."""
from pathlib import Path
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app027_verified_reconstructed_resume"))
import test_app027 as legacy
import exclusive_app028 as app

# Actual child processes, no mock lock or simulated native outputs.
HELPER="""import sys,time,json
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,sys.argv[1])
import exclusive_app028 as app
try:
    _,key,claim=app.reserve(Path(sys.argv[2]),{"child":sys.argv[3]})
    print("RESERVED",flush=True)
    if sys.argv[3]=="hold": time.sleep(120)
except Exception as exc:
    print(getattr(exc,"code","BLOCK"),flush=True)
    sys.exit(3)
"""

class Reservation(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="app028_lock_")
        self.addCleanup(self.tmp.cleanup)
        self.base=Path(self.tmp.name)
        self.root=self.base/"workspace"
        self.root.mkdir()

    def child(self,root=None,mode="once"):
        p=subprocess.Popen([sys.executable,"-B","-c",HELPER,str(HERE),str(root or self.root),mode],
                           stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        self.addCleanup(self.close_child,p)
        return p

    @staticmethod
    def close_child(p):
        if p.poll() is None: p.kill()
        p.communicate(timeout=15)

    def test_01_exclusive_process_race(self):
        ps=[self.child() for _ in range(6)]
        results=[p.communicate(timeout=45) for p in ps]
        self.assertEqual(sorted(p.returncode for p in ps),[0,3,3,3,3,3])
        self.assertEqual(sum(out.strip()==b"RESERVED" for out,_ in results),1)

    def test_02_killed_owner_never_reclaimed(self):
        p=self.child(mode="hold")
        deadline=time.monotonic()+20
        claim=app.location(self.root)[2]
        while not (claim/"START.json").exists() and time.monotonic()<deadline:
            time.sleep(.02)
        self.assertTrue((claim/"START.json").is_file())
        p.kill()
        p.communicate(timeout=15)
        q=self.child()
        out,_=q.communicate(timeout=20)
        self.assertEqual(q.returncode,3,out)
        self.assertEqual(app.inspect(self.root)["status"],"RESERVATION_PRESENT_ACTIVE_OR_INTERRUPTED_BLOCKED")

    def test_03_different_context_same_workspace(self):
        app.reserve(self.root,{"pin":"first"})
        with self.assertRaises(Exception): app.reserve(self.root,{"pin":"second"})

    def test_04_alias_same_identity(self):
        app.reserve(self.root,{"pin":"first"})
        alias=self.base/"."/"workspace"
        self.assertEqual(app.location(self.root)[2],app.location(alias)[2])
        with self.assertRaises(Exception): app.reserve(alias,{})

    def test_05_distinct_workspaces(self):
        other=self.base/"other"
        other.mkdir()
        a=app.reserve(self.root,{})[2]
        b=app.reserve(other,{})[2]
        self.assertNotEqual(a,b)

    def test_06_empty_abandoned_marker_blocks(self):
        app.location(self.root)[2].mkdir()
        with self.assertRaises(Exception): app.reserve(self.root,{})
        self.assertFalse(app.inspect(self.root)["execution_authorized"])

    def test_07_file_obstacle_untouched(self):
        claim=app.location(self.root)[2]
        claim.write_bytes(b"preserve")
        with self.assertRaises(Exception): app.reserve(self.root,{})
        self.assertEqual(claim.read_bytes(),b"preserve")
        self.assertEqual(app.inspect(self.root)["status"],"RESERVATION_OBSTACLE_BLOCKED")

    def test_08_status_is_not_authorization(self):
        self.assertEqual(app.inspect(self.root)["status"],"NO_LOCAL_RESERVATION_OBSERVED")
        self.assertFalse(app.inspect(self.root)["execution_authorized"])
        self.assertFalse(app.location(self.root)[2].exists())
        claim=app.reserve(self.root,{})[2]
        (claim/"COMPLETE.json").write_bytes(b"unverified")
        self.assertEqual(app.inspect(self.root)["status"],"COMPLETION_RECEIPT_PRESENT_UNVERIFIED_BLOCKED")
        with self.assertRaises(Exception): app.reserve(self.root,{})

class Native(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Continuation("test_01_three_missing_native_jobs")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.prepare()
        self.f=self.t.f
        self.claim=app.location(self.f.root)[2]

    def args(self):
        f=self.f
        return dict(old=f.bundle,old_pin=f.pin,root=f.root,suffix=f.suffix,suffix_pin=f.suffix_pin,
            new=self.t.t.dest,new_pin=self.t.new_pin,engine=f.fixture.engine,engine_pin=f.fixture.engine_pin,
            source=f.fixture.source,source_pin=f.fixture.source_pin,workers=2,consent=app.previous.CONSENT)

    def launch(self):
        cmd=[sys.executable,"-B",str(HERE/"exclusive_app028.py"),"run"]
        for k,v in self.args().items(): cmd.extend(["--"+k.replace("_","-"),str(v)])
        p=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        self.addCleanup(Reservation.close_child,p)
        return p

    def test_09_real_native_competing_continuations(self):
        ps=[self.launch(),self.launch()]
        data=[json.loads(p.communicate(timeout=300)[0]) for p in ps]
        self.assertEqual(sorted(p.returncode for p in ps),[0,3],data)
        good=next(x for x in data if x["status"]=="COMPLETE_VERIFIED")
        self.assertEqual(good["continuation"]["new_verified_events"],3)
        self.assertEqual(good["continuation"]["verified_native_jobs"],6)
        rows=[json.loads(x) for x in (self.claim/"native-attempt"/"native-suffix.jsonl").read_bytes().splitlines()]
        self.assertEqual([x["step"] for x in rows],[4,5,6])
        after=self.f.fixture.native_bytes(self.f.root)
        self.assertTrue(all(after[k]==v for k,v in self.t.before.items()))

    def test_10_abandoned_reservation_prevents_native_effects(self):
        app.reserve(self.f.root,{})
        before=self.f.fixture.native_bytes(self.f.root)
        with self.assertRaises(Exception): app.execute(**self.args())
        self.assertEqual(before,self.f.fixture.native_bytes(self.f.root))

    def test_11_invalid_pin_and_consent_no_reservation(self):
        for change in ({"new_pin":"0"*64},{"consent":"NO"}):
            a=self.args()
            a.update(change)
            with self.assertRaises(Exception): app.execute(**a)
            self.assertFalse(self.claim.exists())

    def test_12_success_receipts_retained(self):
        result=app.execute(**self.args())
        self.assertTrue(result["reservation_retained"])
        self.assertEqual(json.loads((self.claim/"COMPLETE.json").read_bytes()),result)
        self.assertTrue((self.claim/"START.json").is_file())
        with self.assertRaises(Exception): app.reserve(self.f.root,{})
        self.assertEqual(result["global_antirollback"],"NOT_PROVEN")

if __name__=="__main__":
    unittest.main()
