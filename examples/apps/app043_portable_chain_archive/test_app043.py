"""Real portable historical evidence, replay without source paths, adversarial archives."""
import base64
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app042_verified_seal_chain"))
import test_app042 as legacy
import archive_app043 as app

class Archive(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Chain("test_01_real_two_transition_chain")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.base
        self.output=self.base/"portable-chain.json"

    def create(self,**change):
        args=self.t.args()
        args.update(output=self.output)
        args.update(change)
        return app.archive(**args)

    def verify(self,pin,**change):
        args=dict(output=self.output,archive_pin=pin,source_plan_pin=self.t.pin,anchor_pin=self.t.anchor)
        args.update(change)
        return app.verify(**args)

    def mutate(self,fn):
        obj=json.loads(self.output.read_bytes())
        fn(obj)
        raw=app.portable.serialize(obj)
        self.output.write_bytes(raw)
        return app.transfer.sha(raw)

    def test_01_real_portable_replay(self):
        a=self.create()
        self.assertEqual(a["status"],"CHAIN_ARCHIVE_CREATED")
        v=self.verify(a["archive_sha256"])
        self.assertEqual(v["head_sha256"],self.t.head)
        self.assertEqual(v["transition_count"],2)

    def test_02_replay_after_source_relocation(self):
        a=self.create()
        self.t.t.successors.rename(self.base/"detached-successors")
        self.t.first_journal.rename(self.base/"detached-history")
        self.t.plan.rename(self.base/"detached-plan")
        self.assertEqual(self.verify(a["archive_sha256"])["status"],"CHAIN_ARCHIVE_VERIFIED")

    def test_03_deterministic_archive(self):
        a=self.create()
        b=self.create(output=self.base/"second-archive.json")
        self.assertEqual(a["archive_sha256"],b["archive_sha256"])

    def test_04_wrong_external_pins(self):
        a=self.create()
        for change in (dict(archive_pin="0"*64),dict(source_plan_pin="0"*64),dict(anchor_pin="0"*64)):
            args=dict(output=self.output,archive_pin=a["archive_sha256"],source_plan_pin=self.t.pin,anchor_pin=self.t.anchor)
            args.update(change)
            with self.assertRaises(Exception): app.verify(**args)

    def test_05_missing_member_refused(self):
        self.create()
        pin=self.mutate(lambda o:o["files"].pop(next(iter(o["files"]))))
        with self.assertRaises(Exception): self.verify(pin)

    def test_06_tampered_signature_refused(self):
        self.create()
        def edit(o):
            k=next(k for k in o["files"] if k.endswith("conflict-ACK.sig"))
            o["files"][k]=base64.b64encode(b"invalid").decode("ascii")
        with self.assertRaises(Exception): self.verify(self.mutate(edit))

    def test_07_path_traversal_refused(self):
        self.create()
        pin=self.mutate(lambda o:o["files"].update({"../escape":"eA=="}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_08_extra_member_refused(self):
        self.create()
        pin=self.mutate(lambda o:o["files"].update({"steps/63/old-seal.json":"eA=="}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_09_unrelated_private_file_excluded(self):
        (self.base/"private-secret").write_bytes(b"not-evidence")
        a=self.create()
        obj=json.loads(self.output.read_bytes())
        self.assertFalse(any("private" in k for k in obj["files"]))
        self.assertFalse(any(base64.b64decode(v)==b"not-evidence" for v in obj["files"].values()))
        self.assertEqual(self.verify(a["archive_sha256"])["status"],"CHAIN_ARCHIVE_VERIFIED")

    def test_10_invalid_source_no_publication(self):
        next(self.t.first_journal.rglob("conflict-ACK.sig")).write_bytes(b"invalid")
        with self.assertRaises(Exception): self.create()
        self.assertFalse(self.output.exists())

    def test_11_restart_and_no_overwrite(self):
        a=self.create()
        before=self.output.read_bytes()
        cmd=[sys.executable,"-B",str(HERE/"archive_app043.py"),"verify","--output",str(self.output),
             "--archive-pin",a["archive_sha256"],"--source-plan-pin",self.t.pin,"--anchor-pin",self.t.anchor]
        p=subprocess.run(cmd,capture_output=True,timeout=300)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        with self.assertRaises(Exception): self.create()
        self.assertEqual(self.output.read_bytes(),before)

    def test_12_scope_and_authority(self):
        a=self.create()
        self.assertFalse(a["original_locations_required"])
        self.assertEqual(a["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(a["execution_authorized"])
        self.assertFalse(a["reservation_release_authorized"])
        self.assertFalse(a["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
