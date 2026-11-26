"""Real signed journal extension and externally anchored historical continuity."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app039_conflict_journal_seal"))
import test_app039 as legacy
import continuity_app040 as app

class Continuity(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Seal("test_01_real_signed_journal_seal")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.old=self.t.seal()
        self.old_entry=self.t.t.directory
        self.new_seal=self.t.base/"successor.json"
        self.output=self.t.base/"transition.json"
        self.new=None

    def extend(self):
        self.t.t.make_conflict("1"*64)
        self.t.t.preserve()
        self.new=self.t.seal(output=self.new_seal)

    def args(self,**change):
        a=dict(registry_path=self.t.t.t.registry,journal=self.t.t.journal,
               old_seal=self.t.output,old_pin=self.old["seal_sha256"],
               old_contexts=self.t.contexts,old_contexts_pin=self.t.context_pin,
               new_seal=self.new_seal,new_pin=self.new["seal_sha256"],
               new_contexts=self.t.contexts,new_contexts_pin=self.t.context_pin,output=self.output)
        a.update(change)
        return a

    def publish(self,**change):
        return app.publish(**self.args(**change))

    def verify(self,pin,**change):
        return app.verify(transition_pin=pin,**self.args(**change))

    def test_01_real_append_only_transition(self):
        self.extend()
        r=self.publish()
        self.assertEqual(r["status"],"SEAL_TRANSITION_CREATED")
        self.assertEqual(r["preserved_entry_count"],1)
        self.assertEqual(r["new_entry_count"],1)
        self.assertEqual(self.verify(r["transition_sha256"])["status"],"SEAL_TRANSITION_VERIFIED")

    def test_02_no_growth_refused(self):
        self.new=self.t.seal(output=self.new_seal)
        with self.assertRaises(Exception): self.publish()
        self.assertFalse(self.output.exists())

    def test_03_valid_replacement_seal_refused(self):
        self.old_entry.rename(self.t.base/"removed-old")
        self.extend()
        self.assertEqual(self.new["entry_count"],1)
        with self.assertRaises(Exception): self.publish()
        self.assertFalse(self.output.exists())

    def test_04_old_pin_mismatch(self):
        self.extend()
        with self.assertRaises(Exception): self.publish(old_pin="0"*64)

    def test_05_new_pin_mismatch(self):
        self.extend()
        with self.assertRaises(Exception): self.publish(new_pin="0"*64)

    def test_06_old_manifest_tampering(self):
        self.extend()
        obj=json.loads(self.t.output.read_bytes())
        obj["entries"][0]["files"]["COMPLETE.json"]="0"*64
        self.t.output.write_bytes(app.portable.serialize(obj))
        with self.assertRaises(Exception): self.publish()

    def test_07_wrong_historical_context(self):
        self.extend()
        doc=json.loads(self.t.contexts.read_bytes())
        doc["contexts"][0]["receiver"]="other"
        path=self.t.base/"wrong-context.json"
        raw=app.portable.serialize(doc)
        path.write_bytes(raw)
        with self.assertRaises(Exception):
            self.publish(old_contexts=path,old_contexts_pin=app.transfer.sha(raw))

    def test_08_current_journal_tamper_refused(self):
        self.extend()
        (self.t.t.directory/"conflict-ACK.sig").write_bytes(b"invalid")
        with self.assertRaises(Exception): self.publish()

    def test_09_transition_tamper_refused(self):
        self.extend()
        r=self.publish()
        obj=json.loads(self.output.read_bytes())
        obj["preserved_entry_count"]=0
        raw=app.portable.serialize(obj)
        self.output.write_bytes(raw)
        with self.assertRaises(Exception): self.verify(r["transition_sha256"])
        with self.assertRaises(Exception): self.verify(app.transfer.sha(raw))

    def test_10_restart_readonly_no_overwrite(self):
        self.extend()
        r=self.publish()
        paths=[self.t.output,self.new_seal,self.output,self.t.contexts]
        before={p:p.read_bytes() for p in paths}
        cmd=[sys.executable,"-B",str(HERE/"continuity_app040.py"),"verify"]+[
            s for k,v in dict(self.args(),transition_pin=r["transition_sha256"]).items()
            for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=180)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        with self.assertRaises(Exception): self.publish()
        self.assertEqual({p:p.read_bytes() for p in paths},before)

    def test_11_growth_after_successor_refused(self):
        self.extend()
        r=self.publish()
        self.t.t.make_conflict("2"*64)
        self.t.t.preserve()
        with self.assertRaises(Exception): self.verify(r["transition_sha256"])

    def test_12_scope_and_authority(self):
        self.extend()
        r=self.publish()
        self.assertEqual(r["fork_prevention"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
