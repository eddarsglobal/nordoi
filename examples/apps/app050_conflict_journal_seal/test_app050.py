"""Real APP-049 conflict inventory seals, external pins and append-only verification."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app049_admission_conflict_journal"))
import test_app049 as legacy
import seal_app050 as app

class Seal(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Conflict("test_01_real_conflict_preserved")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.t.base
        self.journal=self.t.journal
        self.ident=self.t.folder().name
        self.jid=app.transfer.sha(b"APP050-test-journal")
        self.plan0=self.base/"plan-empty.json"
        self.plan1=self.base/"plan-one.json"
        self.zero=self.base/"seal-zero.json"
        self.one=self.base/"seal-one.json"
        args={k:v for k,v in self.t.audit_args().items() if k!="journal"}
        self.p0=self.write_plan(self.plan0,{})
        self.p1=self.write_plan(self.plan1,{self.ident:args})

    def write_plan(self,path,entries):
        raw=app.portable.serialize({"schema":app.PLAN,"journal_id":self.jid,"entries":entries})
        path.write_bytes(raw)
        return app.transfer.sha(raw)

    def seal_args(self,**change):
        a=dict(journal=self.journal,plan=self.plan1,plan_pin=self.p1,journal_id=self.jid,output=self.one)
        a.update(change)
        return a

    def empty(self):
        return app.seal(**self.seal_args(plan=self.plan0,plan_pin=self.p0,output=self.zero))

    def full(self,**change):
        self.t.preserve()
        return app.seal(**self.seal_args(**change))

    def pair(self):
        old=self.empty()
        new=self.full(parent=self.zero,parent_pin=old["seal_sha256"])
        return old,new

    def audit_args(self,pin,**change):
        a=dict(journal=self.journal,plan=self.plan1,plan_pin=self.p1,journal_id=self.jid,seal_file=self.one,seal_pin=pin)
        a.update(change)
        return a

    def audit(self,pin,**change): return app.audit(**self.audit_args(pin,**change))

    def transition(self,old,new,**change):
        a=dict(parent=self.zero,parent_pin=old["seal_sha256"],child=self.one,child_pin=new["seal_sha256"],journal_id=self.jid)
        a.update(change)
        return app.transition(**a)

    def test_01_real_seal_and_audit(self):
        r=self.full()
        self.assertEqual(self.audit(r["seal_sha256"])["record"],r["record"])
        self.assertEqual(set(r["record"]["entries"]),{self.ident})

    def test_02_append_only_transition(self):
        old,new=self.pair()
        self.assertEqual(self.transition(old,new)["added"],1)
        self.assertEqual(self.audit(new["seal_sha256"])["status"],"JOURNAL_SEAL_VERIFIED")

    def test_03_deleted_conflict_blocked(self):
        r=self.full()
        self.t.folder().rename(self.base/"removed-conflict")
        with self.assertRaises(Exception): self.audit(r["seal_sha256"])

    def test_04_unexpected_addition_blocked(self):
        r=self.full()
        (self.journal/("a"*64)).mkdir()
        with self.assertRaises(Exception): self.audit(r["seal_sha256"])

    def test_05_tampered_proof_blocked(self):
        r=self.full()
        p=self.t.folder()/"OTHER-ACK.sig"
        p.write_bytes(b"invalid")
        with self.assertRaises(Exception): self.audit(r["seal_sha256"])

    def test_06_wrong_external_pins(self):
        r=self.full()
        with self.assertRaises(Exception): self.audit("a"*64)
        with self.assertRaises(Exception): self.audit(r["seal_sha256"],plan_pin="a"*64)
        with self.assertRaises(Exception): self.audit(r["seal_sha256"],journal_id="a"*64)

    def test_07_invalid_expectation_no_publication(self):
        self.t.preserve()
        args={k:v for k,v in self.t.audit_args().items() if k!="journal"}
        args["original_record_pin"]="a"*64
        bad=self.base/"bad-plan.json"
        pin=self.write_plan(bad,{self.ident:args})
        with self.assertRaises(Exception): app.seal(**self.seal_args(plan=bad,plan_pin=pin))
        self.assertFalse(self.one.exists())

    def test_08_exclusive_output_and_overlap(self):
        r=self.full()
        before=self.one.read_bytes()
        with self.assertRaises(Exception): app.seal(**self.seal_args())
        with self.assertRaises(Exception): app.seal(**self.seal_args(output=self.journal/"bad.json"))
        self.assertEqual(self.one.read_bytes(),before)
        self.assertEqual(self.audit(r["seal_sha256"])["status"],"JOURNAL_SEAL_VERIFIED")

    def test_09_removal_or_no_growth_cannot_extend(self):
        r=self.full()
        self.t.folder().rename(self.base/"removed")
        with self.assertRaises(Exception): app.seal(**self.seal_args(plan=self.plan0,plan_pin=self.p0,output=self.zero,parent=self.one,parent_pin=r["seal_sha256"]))
        self.assertFalse(self.zero.exists())
        (self.base/"removed").rename(self.t.folder())
        with self.assertRaises(Exception): app.seal(**self.seal_args(output=self.zero,parent=self.one,parent_pin=r["seal_sha256"]))

    def test_10_wrong_link_or_mutated_member(self):
        old,new=self.pair()
        with self.assertRaises(Exception): self.transition(old,new,parent_pin="a"*64)
        doc=dict(new["record"],parent_sha256="b"*64)
        raw=app.portable.serialize(doc)
        self.one.write_bytes(raw)
        with self.assertRaises(Exception): self.transition(old,new,child_pin=app.transfer.sha(raw))
        # Structurally pinned snapshots still cannot replace an existing entry.
        previous=dict(new["record"])
        parent_raw=app.portable.serialize(previous)
        self.one.write_bytes(parent_raw)
        pin=app.transfer.sha(parent_raw)
        child=dict(previous,sequence=previous["sequence"]+1,parent_sha256=pin,
                   entries={self.ident:"c"*64,"d"*64:"e"*64})
        raw=app.portable.serialize(child)
        self.zero.write_bytes(raw)
        with self.assertRaises(Exception): app.transition(self.one,pin,self.zero,app.transfer.sha(raw),self.jid)

    def test_11_readonly_restart(self):
        r=self.full()
        before=self.t.snapshot()
        a=self.audit_args(r["seal_sha256"])
        cmd=[sys.executable,"-B",str(HERE/"seal_app050.py"),"audit"]+[
             s for k,v in a.items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["record"],r["record"])
        self.assertEqual(self.t.snapshot(),before)

    def test_12_scope_and_authority(self):
        r=self.full()
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["independent_completeness"],"NOT_PROVEN")
        self.assertEqual(r["exclusive_successor"],"NOT_IMPLEMENTED")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
