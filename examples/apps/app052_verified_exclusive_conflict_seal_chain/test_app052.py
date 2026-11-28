"""Exact APP-051 chain replay, genuine two-step extension and negative cases."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app051_exclusive_conflict_seal_successor"))
import test_app051 as legacy
import chain_app052 as app

class Chain(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Successor("test_01_real_admission_and_live_audit")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.root=self.t.root
        self.base=self.t.base
        self.jid=self.t.t.jid
        self.anchor=self.t.old["seal_sha256"]
        self.head=self.t.new["seal_sha256"]
        self.file=self.base/"chain-plan.json"
        rec=self.t.admit()["record"]
        self.steps=[self.step(rec)]
        self.pin=self.write_plan()

    def step(self,rec):
        return dict(parent_sha256=rec["context"]["parent_sha256"],child_sha256=rec["child_sha256"],
                    plan_sha256=rec["plan_sha256"],decision_sha256=app.transfer.sha(app.portable.serialize(rec)))

    def write_plan(self,**change):
        doc=dict(schema=app.PLAN,journal_id=self.jid,anchor_sha256=self.anchor,head_sha256=self.head,steps=self.steps)
        doc.update(change)
        raw=app.portable.serialize(doc)
        self.file.write_bytes(raw)
        return app.transfer.sha(raw)

    def args(self,**change):
        a=dict(successor_registry=self.root,plan_file=self.file,plan_pin=self.pin,journal_id=self.jid,
               anchor_pin=self.anchor,head_pin=self.head)
        a.update(change)
        return a

    def audit(self,**change): return app.audit(**self.args(**change))

    def extend(self):
        # Genuine second independently signed conflict; copy only its preserved proof into the same journal.
        other=legacy.legacy.Seal("test_01_real_seal_and_audit")
        other.setUp()
        self.addCleanup(other.doCleanups)
        other.t.preserve()
        self.assertNotEqual(other.ident,self.t.t.ident)
        shutil.copytree(other.t.folder(),self.t.t.journal/other.ident)
        entries=json.loads(self.t.t.plan1.read_bytes())["entries"]
        entries.update(json.loads(other.plan1.read_bytes())["entries"])
        plan2=self.base/"plan-two.json"
        p2=self.t.t.write_plan(plan2,entries)
        seal2=self.base/"seal-two.json"
        new=app.prior.prior.seal(journal=self.t.t.journal,plan=plan2,plan_pin=p2,journal_id=self.jid,
                                 output=seal2,parent=self.t.t.one,parent_pin=self.head)
        rec=app.prior.admit(successor_registry=self.root,journal=self.t.t.journal,parent=self.t.t.one,
                           parent_pin=self.head,child=seal2,child_pin=new["seal_sha256"],
                           plan=plan2,plan_pin=p2,journal_id=self.jid)["record"]
        self.steps.append(self.step(rec))
        self.head=new["seal_sha256"]
        self.pin=self.write_plan()

    def test_01_real_two_step_chain(self):
        self.extend()
        out=self.audit(journal=self.t.t.journal)
        self.assertEqual(out["status"],"EXCLUSIVE_CHAIN_VERIFIED")
        self.assertEqual(out["transition_count"],2)
        self.assertTrue(out["head_inventory_verified"])
        self.assertEqual(out["head_sha256"],self.head)

    def test_02_wrong_external_pins(self):
        for name in ("plan_pin","anchor_pin","head_pin","journal_id"):
            with self.assertRaises(Exception): self.audit(**{name:"a"*64})

    def test_03_missing_decision(self):
        self.t.folder.rename(self.base/"removed-decision")
        with self.assertRaises(Exception): self.audit()

    def test_04_orphan_decision(self):
        (self.root/("b"*64)).mkdir()
        with self.assertRaises(Exception): self.audit()

    def test_05_cycle_rejected(self):
        steps=[dict(self.steps[0],child_sha256=self.anchor)]
        pin=self.write_plan(steps=steps,head_sha256=self.anchor)
        with self.assertRaises(Exception): self.audit(plan_pin=pin,head_pin=self.anchor)

    def test_06_broken_link_rejected(self):
        pin=self.write_plan(steps=[dict(self.steps[0],parent_sha256="b"*64)])
        with self.assertRaises(Exception): self.audit(plan_pin=pin)

    def test_07_stored_member_tamper(self):
        for n in ("PARENT.json","CHILD.json","PLAN.json","START.json","COMPLETE.json"):
            p=self.t.folder/n
            raw=p.read_bytes()
            p.write_bytes(b"{}")
            with self.assertRaises(Exception): self.audit()
            p.write_bytes(raw)
        self.assertEqual(self.audit()["status"],"EXCLUSIVE_CHAIN_VERIFIED")

    def test_08_external_decision_pin_rejected(self):
        pin=self.write_plan(steps=[dict(self.steps[0],decision_sha256="c"*64)])
        with self.assertRaises(Exception): self.audit(plan_pin=pin)

    def test_09_truncated_chain_not_expected_head(self):
        self.extend()
        pin=self.write_plan(steps=self.steps[:1])
        with self.assertRaises(Exception): self.audit(plan_pin=pin)

    def test_10_live_head_tamper_distinction(self):
        self.t.t.t.folder().rename(self.base/"missing-live-proof")
        self.assertFalse(self.audit()["head_inventory_verified"])
        with self.assertRaises(Exception): self.audit(journal=self.t.t.journal)

    def test_11_restart_readonly(self):
        before=self.t.snapshot()
        for i,p in enumerate((self.t.t.zero,self.t.t.one,self.t.t.plan1,self.t.t.journal)):
            p.rename(self.base/("detached-"+str(i)))
        cmd=[sys.executable,"-B",str(HERE/"chain_app052.py")]+[
             s for k,v in self.args().items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"EXCLUSIVE_CHAIN_VERIFIED")
        self.assertEqual(self.t.snapshot(),before)

    def test_12_scope_and_overlap(self):
        with self.assertRaises(Exception): self.audit(journal=self.root)
        r=self.audit()
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["distributed_exclusivity"],"NOT_PROVEN")
        self.assertEqual(r["historical_live_replay"],"NOT_ATTESTED")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
