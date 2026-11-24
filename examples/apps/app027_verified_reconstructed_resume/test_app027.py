"""Real P2.7 integration: reconstruct, then continue on existing partial root."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app026_verified_checkpoint_reconstruction"))
import test_app026 as previous
import resume_app027 as app

class Continuation(unittest.TestCase):
    def setUp(self):
        self.t=previous.Reconstruction("test_01_one_extension_real_portable")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.f=self.t.f
        self.attempt=self.f.base/"app027-attempt"

    def prepare(self,n=1):
        self.f.fresh_partial(n)
        self.new_pin=self.t.run_app()["new_checkpoint_sha256"]
        self.before=self.f.fixture.native_bytes(self.f.root)

    def execute(self,**overrides):
        f=self.f
        args=dict(old=f.bundle,old_pin=f.pin,root=f.root,suffix=f.suffix,suffix_pin=f.suffix_pin,
            new=self.t.dest,new_pin=self.new_pin,engine=f.fixture.engine,engine_pin=f.fixture.engine_pin,
            source=f.fixture.source,source_pin=f.fixture.source_pin,workers=2,attempt=self.attempt,consent=app.CONSENT)
        args.update(overrides)
        return app.execute(**args)

    def denied(self,**overrides):
        with self.assertRaises(Exception): self.execute(**overrides)
        self.assertEqual(self.before,self.f.fixture.native_bytes(self.f.root))
        self.assertFalse(self.attempt.exists())

    def test_01_three_missing_native_jobs(self):
        self.prepare()
        r=self.execute()
        self.assertEqual((r["historical_events"],r["new_verified_events"],r["verified_native_jobs"]),(3,3,6))
        rows=[json.loads(x) for x in (self.attempt/"native-suffix.jsonl").read_bytes().splitlines()]
        self.assertEqual([x["step"] for x in rows],[4,5,6])
        self.assertTrue(all(x["status"]=="COMMITTED" for x in rows))
        self.assertEqual(r["status"],"COMPLETE_VERIFIED")

    def test_02_two_workers_or_one_with_four_history(self):
        self.prepare(2)
        r=self.execute(workers=1)
        self.assertEqual((r["historical_events"],r["new_verified_events"]),(4,2))
        self.assertEqual(r["verified_events_total"],6)

    def test_03_preserve_history_and_original_packages(self):
        self.prepare()
        old=(self.f.bundle/"portable.json").read_bytes()
        new=(self.t.dest/"portable.json").read_bytes()
        suffix=self.f.suffix.read_bytes()
        self.execute()
        after=self.f.fixture.native_bytes(self.f.root)
        self.assertTrue(all(after[k]==v for k,v in self.before.items()))
        self.assertEqual(old,(self.f.bundle/"portable.json").read_bytes())
        self.assertEqual(new,(self.t.dest/"portable.json").read_bytes())
        self.assertEqual(suffix,self.f.suffix.read_bytes())

    def test_04_consent_required(self):
        self.prepare()
        self.denied(consent="NO")

    def test_05_old_pin_wrong(self):
        self.prepare()
        self.denied(old_pin="0"*64)

    def test_06_reconstructed_pin_wrong(self):
        self.prepare()
        self.denied(new_pin="0"*64)

    def test_07_suffix_pin_wrong(self):
        self.prepare()
        self.denied(suffix_pin="0"*64)

    def test_08_original_checkpoint_not_reconstruction(self):
        self.prepare()
        self.denied(new=self.f.bundle,new_pin=self.f.pin)

    def test_09_engine_or_source_wrong(self):
        self.prepare()
        self.denied(engine_pin="0"*64)
        self.denied(source_pin="0"*64)

    def test_10_existing_attempt_no_overwrite(self):
        self.prepare()
        self.attempt.mkdir()
        sentinel=self.attempt/"sentinel"
        sentinel.write_bytes(b"keep")
        with self.assertRaises(Exception): self.execute()
        self.assertEqual(sentinel.read_bytes(),b"keep")
        self.assertEqual(self.before,self.f.fixture.native_bytes(self.f.root))

    def test_11_completed_workspace_cannot_replay(self):
        self.prepare()
        self.execute()
        after=self.f.fixture.native_bytes(self.f.root)
        second=self.f.base/"second-attempt"
        with self.assertRaises(Exception): self.execute(attempt=second)
        self.assertFalse(second.exists())
        self.assertEqual(after,self.f.fixture.native_bytes(self.f.root))

    def test_12_local_receipts_and_no_global_claim(self):
        self.prepare()
        r=self.execute()
        start=json.loads((self.attempt/"START.json").read_bytes())
        final=json.loads((self.attempt/"COMPLETE.json").read_bytes())
        self.assertEqual(start["reconstructed_pin"],self.new_pin)
        self.assertEqual(final,r)
        self.assertEqual(r["new_suffix_sha256"],hashlib.sha256((self.attempt/"native-suffix.jsonl").read_bytes()).hexdigest())
        self.assertFalse(r["automatic_retry"])
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["production"],"NOT_AUTHORIZED")

if __name__=="__main__":
    unittest.main()
