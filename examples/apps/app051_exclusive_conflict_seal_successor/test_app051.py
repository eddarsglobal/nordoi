"""Real APP-050 successor admission, two valid branches, interruption and replay."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app050_conflict_journal_seal"))
import test_app050 as legacy
import successor_app051 as app

class Successor(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Seal("test_01_real_seal_and_audit")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.old,self.new=self.t.pair()
        self.base=self.t.base
        self.root=self.base/"successor-registry"
        self.root.mkdir()
        self.folder=self.root/self.old["seal_sha256"]

    def args(self,**change):
        a=dict(successor_registry=self.root,journal=self.t.journal,parent=self.t.zero,parent_pin=self.old["seal_sha256"],
               child=self.t.one,child_pin=self.new["seal_sha256"],plan=self.t.plan1,plan_pin=self.t.p1,journal_id=self.t.jid)
        a.update(change)
        return a

    def audit_args(self,**change):
        a={k:v for k,v in self.args().items() if k not in ("parent","child","plan","journal")}
        a.update(change)
        return a

    def admit(self,**change): return app.admit(**self.args(**change))
    def audit(self,**change): return app.audit(**self.audit_args(**change))
    def snapshot(self): return {str(p):p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
    def cmd(self,action,**change):
        a=self.args(**change) if action=="admit" else self.audit_args(**change)
        return [sys.executable,"-B",str(HERE/"successor_app051.py"),action]+[
            s for k,v in a.items() for s in ("--"+k.replace("_","-"),str(v))]

    def alternate(self):
        # Different valid journal snapshot under the same pinned empty genesis.
        other=legacy.Seal("test_01_real_seal_and_audit")
        other.setUp()
        self.addCleanup(other.doCleanups)
        child=other.full(parent=self.t.zero,parent_pin=self.old["seal_sha256"])
        self.assertNotEqual(child["seal_sha256"],self.new["seal_sha256"])
        return dict(journal=other.journal,child=other.one,child_pin=child["seal_sha256"],plan=other.plan1,plan_pin=other.p1)

    def test_01_real_admission_and_live_audit(self):
        r=self.admit()
        self.assertEqual(r["status"],"SUCCESSOR_ADMITTED_ONCE")
        self.assertTrue(r["live_inventory_verified"])
        self.assertEqual(self.audit(journal=self.t.journal)["record"],r["record"])

    def test_02_duplicate_blocked(self):
        self.admit()
        before=self.snapshot()
        self.assertEqual(self.admit()["status"],"DUPLICATE_SUCCESSOR_BLOCKED")
        self.assertEqual(self.snapshot(),before)

    def test_03_valid_competing_branch_blocked(self):
        r=self.admit()
        before=self.snapshot()
        out=self.admit(**self.alternate())
        self.assertEqual(out["status"],"COMPETING_SUCCESSOR_BLOCKED")
        self.assertEqual(out["accepted_child_sha256"],r["record"]["child_sha256"])
        self.assertEqual(self.snapshot(),before)

    def test_04_concurrent_distinct_branches_one_winner(self):
        alt=self.alternate()
        ps=[subprocess.Popen(self.cmd("admit",**a),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for a in ({},alt)]
        try:
            out=[p.communicate(timeout=600) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            won=json.loads(out[0 if ps[0].returncode==0 else 1][0])["record"]
            self.assertEqual(self.audit(child_pin=won["child_sha256"],plan_pin=won["plan_sha256"])["record"],won)
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_05_invalid_pins_no_reservation(self):
        for a in (dict(parent_pin="a"*64),dict(child_pin="a"*64),dict(plan_pin="a"*64),dict(journal_id="a"*64)):
            with self.assertRaises(Exception): self.admit(**a)
        self.assertEqual(list(self.root.iterdir()),[])

    def test_06_deleted_live_proof_no_reservation(self):
        self.t.t.folder().rename(self.base/"removed-proof")
        with self.assertRaises(Exception): self.admit()
        self.assertFalse(self.folder.exists())

    def test_07_incomplete_marker_blocks(self):
        self.folder.mkdir()
        with self.assertRaises(Exception): self.admit()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.folder.iterdir()),[])

    def test_08_interrupted_completion_blocks(self):
        real=app.portable.safe_write
        target=(self.folder/"COMPLETE.json").resolve(strict=False)
        def fail(path,raw):
            if Path(path).resolve(strict=False)==target: raise OSError("injected successor failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"injected successor failure"): self.admit()
        with self.assertRaises(Exception): self.admit()
        with self.assertRaises(Exception): self.audit()

    def test_09_stored_tamper_blocks(self):
        self.admit()
        for n in ("PARENT.json","CHILD.json","PLAN.json","START.json","COMPLETE.json"):
            p=self.folder/n
            raw=p.read_bytes()
            p.write_bytes(b"{}")
            with self.assertRaises(Exception): self.audit()
            p.write_bytes(raw)
        self.assertEqual(self.audit()["status"],"SUCCESSOR_VERIFIED")

    def test_10_expected_child_and_plan_required(self):
        self.admit()
        for a in (dict(child_pin="a"*64),dict(plan_pin="a"*64)):
            with self.assertRaises(Exception): self.audit(**a)

    def test_11_detached_restart_and_live_distinction(self):
        self.admit()
        before=self.snapshot()
        for i,p in enumerate((self.t.zero,self.t.one,self.t.plan1,self.t.journal)):
            p.rename(self.base/("detached-"+str(i)))
        p=subprocess.run(self.cmd("audit"),capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertFalse(json.loads(p.stdout)["live_inventory_verified"])
        self.assertEqual(self.snapshot(),before)
        with self.assertRaises(Exception): self.audit(journal=self.t.journal)

    def test_12_overlap_and_authority(self):
        with self.assertRaises(Exception): self.admit(successor_registry=self.t.journal)
        r=self.admit()
        self.assertEqual(r["distributed_exclusivity"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
