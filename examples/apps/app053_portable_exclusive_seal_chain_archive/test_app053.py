"""Real exclusive chain archive and detached structural replay."""
import base64
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app052_verified_exclusive_conflict_seal_chain"))
import test_app052 as legacy
import archive_app053 as app

class Archive(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Chain("test_01_real_two_step_chain")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.output=self.t.base/"portable-exclusive-chain.json"

    def args(self,**change):
        a=self.t.args()
        a["output"]=self.output
        a.update(change)
        return a

    def pack(self,**change): return app.archive(**self.args(**change))

    def verify_args(self,pin,**change):
        a={k:v for k,v in self.args().items() if k not in ("successor_registry","plan_file")}
        a["archive_pin"]=pin
        a.update(change)
        return a

    def verify(self,pin,**change): return app.verify(**self.verify_args(pin,**change))

    def mutate(self,change):
        doc=json.loads(self.output.read_bytes())
        change(doc)
        raw=app.portable.serialize(doc)
        self.output.write_bytes(raw)
        return app.transfer.sha(raw)

    def test_01_real_two_transition_archive(self):
        self.t.extend()
        r=self.pack()
        v=self.verify(r["archive_sha256"])
        self.assertEqual(v["status"],"EXCLUSIVE_CHAIN_ARCHIVE_VERIFIED")
        self.assertEqual(v["transition_count"],2)
        self.assertEqual(v["member_count"],10)
        self.assertEqual(v["head_sha256"],self.t.head)

    def test_02_detached_restart(self):
        r=self.pack()
        for i,p in enumerate((self.t.root,self.t.file,self.t.t.t.journal,self.t.t.t.one,self.t.t.t.zero,self.t.t.t.plan1)):
            p.rename(self.t.base/("detached-"+str(i)))
        before=self.output.read_bytes()
        cmd=[sys.executable,"-B",str(HERE/"archive_app053.py"),"verify"]+[
             s for k,v in self.verify_args(r["archive_sha256"]).items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"EXCLUSIVE_CHAIN_ARCHIVE_VERIFIED")
        self.assertEqual(self.output.read_bytes(),before)

    def test_03_deterministic_archive(self):
        r=self.pack()
        second=self.t.base/"second.json"
        other=self.pack(output=second)
        self.assertEqual(r["archive_sha256"],other["archive_sha256"])
        self.assertEqual(self.output.read_bytes(),second.read_bytes())

    def test_04_wrong_external_pins(self):
        r=self.pack()
        with self.assertRaises(Exception): self.verify("a"*64)
        for n in ("plan_pin","journal_id","anchor_pin","head_pin"):
            with self.assertRaises(Exception): self.verify(r["archive_sha256"],**{n:"a"*64})

    def test_05_missing_decision_or_member(self):
        self.pack()
        original=self.output.read_bytes()
        for fn in (lambda d:d["members"].pop(self.t.anchor),
                   lambda d:d["members"][self.t.anchor].pop("CHILD.json")):
            self.output.write_bytes(original)
            with self.assertRaises(Exception): self.verify(self.mutate(fn))

    def test_06_tampered_decision_rejected_even_repinned(self):
        self.pack()
        pin=self.mutate(lambda d:d["members"][self.t.anchor].update({"COMPLETE.json":base64.b64encode(b"{}").decode("ascii")}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_07_path_traversal_and_extra_members(self):
        self.pack()
        original=self.output.read_bytes()
        for fn in (lambda d:d["members"].update({"../escape":{}}),
                   lambda d:d["members"][self.t.anchor].update({"../../escape":"YQ=="}),
                   lambda d:d["members"][self.t.anchor].update({"PRIVATE.json":"YQ=="})):
            self.output.write_bytes(original)
            with self.assertRaises(Exception): self.verify(self.mutate(fn))
        self.assertFalse((self.t.base/"escape").exists())

    def test_08_invalid_base64_and_noncanonical_json(self):
        self.pack()
        original=self.output.read_bytes()
        pin=self.mutate(lambda d:d["members"][self.t.anchor].update({"START.json":"!!!!"}))
        with self.assertRaises(Exception): self.verify(pin)
        self.output.write_bytes(original+b" ")
        with self.assertRaises(Exception): self.verify(app.transfer.sha(original+b" "))

    def test_09_private_unrelated_data_excluded(self):
        (self.t.base/"unrelated-private.txt").write_text("PRIVATE_UNRELATED_SENTINEL",encoding="utf-8")
        r=self.pack()
        doc=json.loads(self.output.read_bytes())
        self.assertEqual(set(doc["members"]),{self.t.anchor})
        self.assertEqual(set(doc["members"][self.t.anchor]),set(app.LIMITS))
        self.assertNotIn(b"PRIVATE_UNRELATED_SENTINEL",self.output.read_bytes())
        self.assertEqual(r["member_count"],5)

    def test_10_invalid_source_no_publication(self):
        (self.t.t.folder/"CHILD.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.pack()
        self.assertFalse(self.output.exists())

    def test_11_no_overwrite_or_registry_overlap(self):
        r=self.pack()
        before=self.output.read_bytes()
        with self.assertRaises(Exception): self.pack()
        with self.assertRaises(Exception): self.pack(output=self.t.root/"new.json")
        self.assertEqual(self.output.read_bytes(),before)
        self.assertEqual(self.verify(r["archive_sha256"])["status"],"EXCLUSIVE_CHAIN_ARCHIVE_VERIFIED")

    def test_12_scope_and_authority(self):
        r=self.pack()
        self.assertFalse(r["head_inventory_verified"])
        self.assertEqual(r["archive_encryption"],"NONE")
        self.assertEqual(r["historical_live_replay"],"NOT_ATTESTED")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
