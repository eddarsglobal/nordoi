"""Real signed two-transition chain, bounded reconstruction and adversarial audit."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app041_exclusive_seal_successor"))
import test_app041 as legacy
import chain_app042 as app

class Chain(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Successor("test_01_real_successor_decision")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.base
        self.first_journal=self.base/"historical-journal"
        shutil.copytree(self.t.t.t.t.journal,self.first_journal)
        first=self.t.args(journal=self.first_journal)
        self.t.accept(journal=self.first_journal)
        self.t.t.t.t.make_conflict("2"*64)
        self.t.t.t.t.preserve()
        final_seal=self.base/"final-seal.json"
        final=self.t.t.t.seal(output=final_seal)
        second=self.t.args(old_seal=self.t.t.new_seal,old_pin=self.t.t.new["seal_sha256"],
                           new_seal=final_seal,new_pin=final["seal_sha256"])
        args={k:v for k,v in second.items() if k not in ("successors","transition_pin")}
        _,raw=app.prior.prior.transition(**args)
        second["transition_pin"]=app.transfer.sha(raw)
        app.prior.accept(**second)
        self.steps=[self.encode(first),self.encode(second)]
        self.anchor=first["old_pin"]
        self.head=second["new_pin"]
        self.plan=self.base/"chain-plan.json"
        self.write_plan(self.steps)

    def encode(self,step):
        return {k:v.relative_to(self.base).as_posix() if k in app.PATHS else v
                for k,v in step.items() if k!="successors"}

    def write_plan(self,steps):
        raw=app.portable.serialize({"schema":app.PLAN_SCHEMA,"steps":steps})
        self.plan.write_bytes(raw)
        self.pin=app.transfer.sha(raw)

    def args(self,**change):
        a=dict(successors=self.t.successors,evidence_root=self.base,plan_file=self.plan,
               plan_pin=self.pin,anchor_pin=self.anchor)
        a.update(change)
        return a

    def audit(self,**change):
        return app.audit(**self.args(**change))

    def test_01_real_two_transition_chain(self):
        r=self.audit()
        self.assertEqual(r["status"],"LOCAL_CHAIN_VERIFIED")
        self.assertEqual(r["transition_count"],2)
        self.assertEqual(r["head_sha256"],self.head)

    def test_02_wrong_anchor(self):
        with self.assertRaises(Exception): self.audit(anchor_pin="0"*64)

    def test_03_missing_middle_decision(self):
        (self.t.successors/self.steps[1]["old_pin"]).rename(self.base/"missing")
        with self.assertRaises(Exception): self.audit()

    def test_04_orphan_decision(self):
        (self.t.successors/("f"*64)).mkdir()
        with self.assertRaises(Exception): self.audit()

    def test_05_cycle(self):
        self.steps[1]["new_pin"]=self.anchor
        self.write_plan(self.steps)
        with self.assertRaises(Exception): self.audit()

    def test_06_broken_link(self):
        self.steps[1]["old_pin"]="0"*64
        self.write_plan(self.steps)
        with self.assertRaises(Exception): self.audit()

    def test_07_context_discontinuity(self):
        self.steps[1]["old_contexts_pin"]="0"*64
        self.write_plan(self.steps)
        with self.assertRaises(Exception): self.audit()

    def test_08_historical_signature_tamper(self):
        p=next(self.first_journal.rglob("conflict-ACK.sig"))
        p.write_bytes(b"invalid")
        with self.assertRaises(Exception): self.audit()

    def test_09_decision_tamper(self):
        (self.t.marker/"COMPLETE.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.audit()

    def test_10_plan_pin_and_path_escape(self):
        with self.assertRaises(Exception): self.audit(plan_pin="0"*64)
        self.steps[0]["journal"]="../other"
        self.write_plan(self.steps)
        with self.assertRaises(Exception): self.audit()

    def test_11_restart_readonly(self):
        def snap():
            return {str(p):p.read_bytes() for p in self.t.successors.rglob("*") if p.is_file()}
        before=snap()
        cmd=[sys.executable,"-B",str(HERE/"chain_app042.py")]+[
            s for k,v in self.args().items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=240)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["head_sha256"],self.head)
        self.assertEqual(snap(),before)

    def test_12_scope_and_authority(self):
        r=self.audit()
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["distributed_consensus"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
