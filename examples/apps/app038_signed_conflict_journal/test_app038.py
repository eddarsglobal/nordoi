"""Actual signed Rust receipts, contradiction persistence, concurrent and read-only audit."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app037_sender_ack_registry"))
import test_app037 as legacy
import conflict_app038 as app

class Conflict(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Registry("test_01_real_ack_accept_and_audit")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.journal=self.t.t.base/"conflicts"
        self.journal.mkdir()
        self.original=self.t.accept()["record"]
        self.nonce=self.t.args()["nonce"]
        self.make_conflict("0"*64)

    def make_conflict(self,pin):
        obj=json.loads(self.t.t.ack.read_bytes())
        obj["receipt_sha256"]=pin
        self.t.t.ack.write_bytes(app.portable.serialize(obj))
        self.t.t.sign()
        self.pin=app.transfer.sha(self.t.t.ack.read_bytes())
        self.directory=self.journal/self.nonce/self.pin

    def args(self,**change):
        a=self.t.args()
        a.update(journal=self.journal)
        a.update(change)
        return a

    def preserve(self,**change):
        return app.preserve(**self.args(**change))

    def audit(self,**change):
        a=self.t.context_args()
        a.update(journal=self.journal,conflict_pin=self.pin)
        a.update(change)
        return app.audit(**a)

    def cmd(self):
        return [sys.executable,"-B",str(HERE/"conflict_app038.py"),"preserve"]+[
            s for k,v in self.args().items() for s in ("--"+k.replace("_","-"),str(v))]

    def test_01_real_signed_contradiction(self):
        r=self.preserve()
        self.assertEqual(r["status"],"SIGNED_CONTRADICTION_RECORDED")
        self.assertEqual(r["record"]["accepted_ack_sha256"],self.original["ack_sha256"])
        self.assertEqual(r["record"]["conflicting_ack_sha256"],self.pin)
        self.assertEqual(self.audit()["record"],r["record"])
        self.assertEqual(self.t.audit()["record"],self.original)

    def test_02_original_duplicate_not_journaled(self):
        self.t.t.ack.write_bytes((self.t.marker/"ACK.json").read_bytes())
        self.t.t.sign()
        self.assertEqual(self.preserve()["status"],"DUPLICATE_ACK_BLOCKED")
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_03_invalid_signature_no_entry(self):
        self.t.t.sign(namespace="wrong")
        with self.assertRaises(Exception): self.preserve()
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_04_repeated_conflict_after_restart(self):
        self.preserve()
        p=subprocess.run(self.cmd(),capture_output=True,timeout=120)
        self.assertEqual(p.returncode,3,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"CONTRADICTION_ALREADY_RECORDED")

    def test_05_actual_two_process_race(self):
        ps=[subprocess.Popen(self.cmd(),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=120) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"SIGNED_CONTRADICTION_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_06_multiple_distinct_conflicts(self):
        first=self.preserve()["record"]
        self.make_conflict("1"*64)
        second=self.preserve()["record"]
        self.assertNotEqual(first["conflicting_ack_sha256"],second["conflicting_ack_sha256"])
        self.assertEqual(self.audit(conflict_pin=first["conflicting_ack_sha256"])["record"],first)
        self.assertEqual(self.audit()["record"],second)

    def test_07_incomplete_entry_permanent(self):
        self.directory.mkdir(parents=True)
        with self.assertRaises(Exception): self.preserve()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.directory.iterdir()),[])

    def test_08_injected_write_failure_permanent(self):
        real=app.portable.safe_write
        destination=self.directory.resolve(strict=False)/"conflict-ACK.json"
        injected=[]
        def fail(path,raw):
            if Path(path).resolve(strict=False)==destination:
                injected.append(destination)
                raise OSError("injected conflict failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"^injected conflict failure$"): self.preserve()
        self.assertEqual(injected,[destination])
        self.assertTrue((self.directory/"START.json").is_file())
        self.assertFalse((self.directory/"COMPLETE.json").exists())
        with self.assertRaises(Exception): self.preserve()
        with self.assertRaises(Exception): self.audit()

    def test_09_tampered_journal_signature(self):
        self.preserve()
        (self.directory/"conflict-ACK.sig").write_bytes(b"invalid")
        with self.assertRaises(Exception): self.audit()

    def test_10_tampered_original_registry(self):
        self.preserve()
        (self.t.marker/"ACK.sig").write_bytes(b"invalid")
        with self.assertRaises(Exception): self.audit()

    def test_11_readonly_audit_and_overlap(self):
        with self.assertRaises(Exception): self.preserve(journal=self.t.registry)
        self.preserve()
        def snap():
            return {str(p):p.read_bytes() for root in (self.journal,self.t.registry)
                    for p in root.rglob("*") if p.is_file()}
        before=snap()
        self.assertEqual(self.audit()["status"],"SIGNED_CONTRADICTION_VERIFIED")
        self.assertEqual(snap(),before)

    def test_12_no_authority_or_completeness_claim(self):
        r=self.preserve()
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])
        self.assertEqual(r["journal_completeness"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")

if __name__=="__main__":
    unittest.main()
