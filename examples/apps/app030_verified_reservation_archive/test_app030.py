"""Actual native evidence archiving and read-only verification."""
import json
from pathlib import Path
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app029_reservation_forensics"))
import test_app029 as legacy
import archive_app030 as app

class Archive(unittest.TestCase):
    def setUp(self):
        self.t=legacy.ReservationAudit("test_01_complete_real_native_evidence")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.f=self.t.f
        self.dest=self.f.base/"app030-archive"

    def args(self):
        a=self.t.t.args()
        for k in ("engine","source","workers","consent"): a.pop(k)
        a["final_suffix_pin"]=getattr(self.t,"pin","0"*64)
        return a

    def make(self):
        self.t.finish()
        return app.archive(self.dest,**self.args())

    def test_01_real_complete_archive(self):
        r=self.make()
        self.assertEqual(r["status"],"ARCHIVE_CREATED_VERIFIED")
        self.assertEqual(r["verified_native_jobs"],6)
        self.assertTrue(r["reservation_retained"])
        self.assertFalse(r["reservation_release_authorized"])

    def test_02_offline_verify(self):
        r=self.make()
        v=app.verify(self.dest,r["archive_sha256"])
        self.assertEqual(v["status"],"ARCHIVE_VERIFIED")
        self.assertFalse(v["execution_authorized"])

    def test_03_original_evidence_preserved(self):
        self.t.finish()
        def snapshot():
            return {str(p):p.read_bytes() for p in self.f.base.rglob("*") if p.is_file() and self.dest not in p.parents}
        before=snapshot()
        app.archive(self.dest,**self.args())
        self.assertEqual(before,snapshot())
        self.assertTrue(self.t.claim.is_dir())

    def test_04_incomplete_reservation_refused(self):
        app.prior.exclusive.reserve(self.f.root,{})
        with self.assertRaises(Exception): app.archive(self.dest,**self.args())
        self.assertFalse(self.dest.exists())

    def test_05_forged_completion_refused(self):
        self.t.finish()
        self.t.mutate(self.t.claim/"COMPLETE.json",lambda x:x.update(reservation_retained=False))
        with self.assertRaises(Exception): app.archive(self.dest,**self.args())
        self.assertFalse(self.dest.exists())

    def test_06_existing_destination_preserved(self):
        self.t.finish()
        self.dest.mkdir()
        p=self.dest/"keep"
        p.write_bytes(b"keep")
        with self.assertRaises(Exception): app.archive(self.dest,**self.args())
        self.assertEqual(p.read_bytes(),b"keep")

    def test_07_overlap_refused(self):
        self.t.finish()
        target=self.t.claim/"forbidden"
        with self.assertRaises(Exception): app.archive(target,**self.args())
        self.assertFalse(target.exists())

    def test_08_wrong_archive_pin(self):
        self.make()
        with self.assertRaises(Exception): app.verify(self.dest,"0"*64)

    def test_09_missing_final_marker(self):
        r=self.make()
        (self.dest/"ARCHIVE.sha256").unlink()
        with self.assertRaises(Exception): app.verify(self.dest,r["archive_sha256"])

    def test_10_tampered_native_evidence(self):
        r=self.make()
        p=next((self.dest/"native"/"job-0006"/"quality-decision").iterdir())
        p.write_bytes(p.read_bytes()+b"tamper")
        with self.assertRaises(Exception): app.verify(self.dest,r["archive_sha256"])

    def test_11_unexpected_file(self):
        r=self.make()
        (self.dest/"extra").write_bytes(b"x")
        with self.assertRaises(Exception): app.verify(self.dest,r["archive_sha256"])

    def test_12_receipt_tamper_and_reservation_still_blocks(self):
        r=self.make()
        p=self.dest/"reservation"/"COMPLETE.json"
        p.write_bytes(p.read_bytes()+b" ")
        with self.assertRaises(Exception): app.verify(self.dest,r["archive_sha256"])
        with self.assertRaises(Exception): app.prior.exclusive.reserve(self.f.root,{})
        self.assertTrue(self.t.claim.is_dir())

if __name__=="__main__":
    unittest.main()
