"""Actual Ed25519 signed native evidence and read-only classification."""
from pathlib import Path
import json
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app033_local_nonce_consumption"))
import test_app033 as legacy
import audit_app034 as app

class Audit(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Nonce("test_01_signed_acceptance_and_receipt")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.target=self.t.target
        self.marker=self.t.marker()

    def check(self,**change):
        return app.classify(**self.t.args(**change))

    def accepted(self):
        self.t.receive()
        return self.check()

    def blocked(self,**change):
        r=self.check(**change)
        self.assertEqual(r["status"],"UNCERTAIN_BLOCKED")
        self.assertFalse(r["nonce_reset_authorized"])

    def test_01_issued_not_authorization(self):
        r=self.check()
        self.assertEqual(r["status"],"ISSUED")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(self.marker.exists())

    def test_02_complete_real_native_signature(self):
        r=self.accepted()
        self.assertEqual(r["status"],"CONSUMED_VERIFIED")
        self.assertEqual(r["verified_native_jobs"],6)
        self.assertTrue(r["signature_reverified"])

    def test_03_empty_marker_incomplete(self):
        self.marker.mkdir()
        self.assertEqual(self.check()["status"],"CONSUMED_INCOMPLETE")

    def test_04_started_without_completion(self):
        app.previous.consume(self.marker.parent,self.t.issued["challenge"])
        self.assertEqual(self.check()["status"],"CONSUMED_INCOMPLETE")

    def test_05_fake_completion_receipt(self):
        self.accepted()
        p=self.marker/"COMPLETE.json"
        obj=json.loads(p.read_bytes())
        obj["verified_native_jobs"]=99
        p.write_bytes(app.portable.serialize(obj))
        self.blocked()

    def test_06_bad_start_receipt(self):
        self.accepted()
        (self.marker/"START.json").write_bytes(b"{}")
        self.blocked()

    def test_07_missing_published_evidence(self):
        self.accepted()
        self.target.rename(self.t.t.base/"missing")
        self.blocked()

    def test_08_tampered_published_evidence(self):
        self.accepted()
        p=next((self.target/"native"/"job-0006"/"quality-decision").iterdir())
        p.write_bytes(p.read_bytes()+b"tamper")
        self.blocked()

    def test_09_wrong_signature_or_packet(self):
        self.accepted()
        self.t.t.sign(namespace="wrong")
        self.blocked()
        self.t.t.sign()
        p=self.t.t.t.packet
        p.write_bytes(p.read_bytes()+b" ")
        self.blocked()

    def test_10_wrong_external_context(self):
        self.accepted()
        self.blocked(receiver="agent-c")
        self.blocked(archive_pin="0"*64)
        self.blocked(nonce="0"*64)

    def test_11_unexpected_marker_entry(self):
        self.marker.mkdir()
        (self.marker/"unknown").write_bytes(b"x")
        self.blocked()

    def test_12_readonly_and_replay_still_blocked(self):
        self.accepted()
        base=self.t.t.base
        def snap():
            return {str(p):p.read_bytes() for p in base.rglob("*") if p.is_file()}
        before=snap()
        r=self.check()
        self.assertEqual(r["status"],"CONSUMED_VERIFIED")
        self.assertEqual(before,snap())
        self.assertFalse(r["nonce_reset_authorized"])
        with self.assertRaises(Exception): self.t.receive(target=base/"replay")
        self.assertFalse((base/"replay").exists())

if __name__=="__main__":
    unittest.main()
