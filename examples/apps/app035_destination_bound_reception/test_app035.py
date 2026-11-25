"""Real signed native evidence, destination substitution and process races."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app032_signed_evidence_transfer"))
import test_app032 as legacy
import bound_app035 as app

class Bound(unittest.TestCase):
    def setUp(self):
        self.t=legacy.SignedTransfer("test_01_actual_signature_and_native_evidence")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.registry=self.t.base/"registry035"
        self.registry.mkdir()
        self.parent=self.t.base/"destination-parent"
        self.parent.mkdir()
        self.target=self.parent/"received"
        a=self.t.args()
        self.issued=app.issue(self.registry,self.target,**{k:a[k] for k in
            ("sender","receiver","transfer_pin","archive_pin","key_pin")})
        self.t.nonce=self.issued["nonce"]
        self.t.document.write_bytes(app.portable.serialize(self.issued["challenge"]["statement"]))
        self.t.sign()
        self.marker=self.registry/self.t.nonce/"consumed"

    def args(self,**change):
        a=self.t.args()
        a.update(registry=self.registry,target=self.target)
        a.update(change)
        return a

    def receive(self,**change):
        return app.receive(**self.args(**change))

    def audit(self,**change):
        return app.audit(**self.args(**change))

    def cmd(self,action="receive",**change):
        return [sys.executable,"-B",str(HERE/"bound_app035.py"),action]+[
            s for k,v in self.args(**change).items() for s in ("--"+k.replace("_","-"),str(v))]

    def test_01_real_bound_receipt(self):
        r=self.receive()
        self.assertEqual(r["status"],"DESTINATION_BOUND_ACCEPTED_ONCE")
        self.assertEqual(r["destination_identity"],app.identity(self.target))
        self.assertEqual(r["verified_native_jobs"],6)
        self.assertFalse(r["execution_authorized"])

    def test_02_readonly_audit(self):
        self.receive()
        def snap():
            return {str(p):p.read_bytes() for p in self.t.base.rglob("*") if p.is_file()}
        before=snap()
        r=self.audit()
        self.assertEqual(r["status"],"DESTINATION_BOUND_VERIFIED")
        self.assertEqual(before,snap())
        self.assertFalse(r["nonce_reset_authorized"])

    def test_03_unexpected_destination_refused(self):
        other=self.parent/"other"
        with self.assertRaises(Exception): self.receive(target=other)
        self.assertFalse(other.exists())
        self.assertFalse(self.marker.exists())

    def test_04_parent_substitution_refused(self):
        self.parent.rename(self.t.base/"original-parent")
        self.parent.mkdir()
        with self.assertRaises(Exception): self.receive()
        self.assertFalse(self.marker.exists())
        self.assertFalse(self.target.exists())

    def test_05_same_bytes_replacement_refused(self):
        self.receive()
        retained=self.parent/"retained-original"
        self.target.rename(retained)
        shutil.copytree(retained,self.target)
        self.assertNotEqual(app.identity(retained),app.identity(self.target))
        with self.assertRaises(Exception): self.audit()

    def test_06_moved_destination_refused(self):
        self.receive()
        other=self.parent/"moved"
        self.target.rename(other)
        with self.assertRaises(Exception): self.audit(target=other)

    def test_07_receipt_tampering_refused(self):
        self.receive()
        p=self.marker/"COMPLETE.json"
        v=json.loads(p.read_bytes())
        v["verified_native_jobs"]=99
        p.write_bytes(app.portable.serialize(v))
        with self.assertRaises(Exception): self.audit()

    def test_08_identity_record_tampering_refused(self):
        self.receive()
        p=self.marker/"DESTINATION.json"
        v=json.loads(p.read_bytes())
        v["inode"]+=1
        p.write_bytes(app.portable.serialize(v))
        with self.assertRaises(Exception): self.audit()

    def test_09_actual_two_process_race(self):
        ps=[subprocess.Popen(self.cmd(),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=180) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"DESTINATION_BOUND_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_10_failure_after_consumption_is_permanent(self):
        real=app.transfer.populate
        def fail(directory,files):
            if directory==self.target.resolve(): raise OSError("injected publication failure")
            return real(directory,files)
        with patch.object(app.transfer,"populate",side_effect=fail):
            with self.assertRaises(Exception): self.receive()
        self.assertTrue(self.marker.exists())
        self.assertFalse((self.marker/"COMPLETE.json").exists())
        with self.assertRaises(Exception): self.receive()
        with self.assertRaises(Exception): self.audit()

    def test_11_invalid_signature_does_not_consume(self):
        self.t.sign(namespace="wrong")
        with self.assertRaises(Exception): self.receive()
        self.assertFalse(self.marker.exists())
        self.t.sign()
        self.assertEqual(self.receive()["status"],"DESTINATION_BOUND_ACCEPTED_ONCE")

    def test_12_replay_and_scope(self):
        r=self.receive()
        p=subprocess.run(self.cmd(),capture_output=True,timeout=180)
        self.assertEqual(p.returncode,3,p.stdout+p.stderr)
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")
        self.assertFalse(r["destination_signed_by_sender"])
        self.assertFalse(r["reservation_release_authorized"])

if __name__=="__main__":
    unittest.main()
