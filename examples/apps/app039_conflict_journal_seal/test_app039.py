"""Real signed contradictions, pinned snapshot, deletion and substitution resistance."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app038_signed_conflict_journal"))
import test_app038 as legacy
import seal_app039 as app

class Seal(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Conflict("test_01_real_signed_contradiction")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.preserve()
        self.base=self.t.t.t.base
        self.contexts=self.base/"trusted-contexts.json"
        self.output=self.base/"seal.json"
        args=self.t.t.context_args()
        args.pop("registry")
        self.write_contexts([args])

    def write_contexts(self,contexts):
        raw=app.portable.serialize({"schema":app.CONTEXT_SCHEMA,"contexts":contexts})
        self.contexts.write_bytes(raw)
        self.context_pin=app.transfer.sha(raw)

    def args(self,**change):
        a=dict(registry_path=self.t.t.registry,journal=self.t.journal,contexts_file=self.contexts,
               contexts_pin=self.context_pin,output=self.output)
        a.update(change)
        return a

    def seal(self,**change):
        return app.seal(**self.args(**change))

    def verify(self,pin,**change):
        return app.verify(seal_pin=pin,**self.args(**change))

    def test_01_real_signed_journal_seal(self):
        r=self.seal()
        self.assertEqual(r["entry_count"],1)
        v=self.verify(r["seal_sha256"])
        self.assertEqual(v["status"],"JOURNAL_SEAL_VERIFIED")
        self.assertEqual(v["inventory_sha256"],r["inventory_sha256"])

    def test_02_deterministic_multiple_conflicts(self):
        self.t.make_conflict("1"*64)
        self.t.preserve()
        a=self.seal()
        b=self.seal(output=self.base/"second-seal.json")
        self.assertEqual(a["entry_count"],2)
        self.assertEqual(a["seal_sha256"],b["seal_sha256"])

    def test_03_deleted_entry_refused(self):
        r=self.seal()
        self.t.directory.rename(self.base/"removed-entry")
        with self.assertRaises(Exception): self.verify(r["seal_sha256"])

    def test_04_valid_added_entry_refused(self):
        r=self.seal()
        self.t.make_conflict("1"*64)
        self.t.preserve()
        with self.assertRaises(Exception): self.verify(r["seal_sha256"])

    def test_05_signature_substitution_refused(self):
        r=self.seal()
        (self.t.directory/"conflict-ACK.sig").write_bytes(b"invalid")
        with self.assertRaises(Exception): self.verify(r["seal_sha256"])

    def test_06_forged_seal_refused_even_with_new_digest(self):
        r=self.seal()
        obj=json.loads(self.output.read_bytes())
        obj["entry_count"]=0
        raw=app.portable.serialize(obj)
        self.output.write_bytes(raw)
        with self.assertRaises(Exception): self.verify(r["seal_sha256"])
        with self.assertRaises(Exception): self.verify(app.transfer.sha(raw))

    def test_07_wrong_context_pin_refused(self):
        with self.assertRaises(Exception): self.seal(contexts_pin="0"*64)
        self.assertFalse(self.output.exists())

    def test_08_wrong_expected_context_refused(self):
        obj=json.loads(self.contexts.read_bytes())
        obj["contexts"][0]["receiver"]="other"
        self.write_contexts(obj["contexts"])
        with self.assertRaises(Exception): self.seal()
        self.assertFalse(self.output.exists())

    def test_09_incomplete_entry_refused(self):
        (self.t.directory/"COMPLETE.json").unlink()
        with self.assertRaises(Exception): self.seal()
        self.assertFalse(self.output.exists())

    def test_10_unknown_journal_entry_refused(self):
        (self.t.directory/"unexpected").write_bytes(b"x")
        with self.assertRaises(Exception): self.seal()

    def test_11_readonly_verify_restart_and_no_overwrite(self):
        r=self.seal()
        def snap():
            return {str(p):p.read_bytes() for root in (self.t.journal,self.t.t.registry)
                    for p in root.rglob("*") if p.is_file()}
        before=snap()
        raw=self.output.read_bytes()
        cmd=[sys.executable,"-B",str(HERE/"seal_app039.py"),"verify"]+[
            s for k,v in dict(self.args(),seal_pin=r["seal_sha256"]).items()
            for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=180)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"JOURNAL_SEAL_VERIFIED")
        with self.assertRaises(Exception): self.seal()
        self.assertEqual(self.output.read_bytes(),raw)
        self.assertEqual(snap(),before)

    def test_12_empty_snapshot_and_scope(self):
        empty=self.base/"empty-journal"
        empty.mkdir()
        self.write_contexts([])
        r=self.seal(journal=empty)
        self.assertEqual(r["entry_count"],0)
        self.assertEqual(self.verify(r["seal_sha256"],journal=empty)["entry_count"],0)
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])
        self.assertEqual(r["journal_completeness"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")

if __name__=="__main__":
    unittest.main()
