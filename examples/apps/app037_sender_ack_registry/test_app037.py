"""Real signed native acknowledgments, local atomic acceptance and audit."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app036_signed_reception_ack"))
import test_app036 as legacy
import registry_app037 as app

class Registry(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Ack("test_01_real_signed_ack")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.registry=self.t.base/"sender-registry"
        self.registry.mkdir()
        self.register()
        self.t.complete()
        self.marker=self.registry/self.t.t.t.nonce/"accepted"

    def args(self,**change):
        a=self.t.args()
        a.update(registry=self.registry)
        a.update(change)
        return a

    def context_args(self,**change):
        a=self.args(**change)
        for k in ("ack","ack_signature","receiver_public_key"): a.pop(k)
        return a

    def register(self,**change):
        return app.register(**self.context_args(**change))

    def accept(self,**change):
        return app.accept(**self.args(**change))

    def audit(self,**change):
        return app.audit(**self.context_args(**change))

    def cmd(self,action="accept"):
        args=self.args() if action=="accept" else self.context_args()
        return [sys.executable,"-B",str(HERE/"registry_app037.py"),action]+[
            s for k,v in args.items() for s in ("--"+k.replace("_","-"),str(v))]

    def test_01_real_ack_accept_and_audit(self):
        self.assertEqual(self.audit()["status"],"ACK_PENDING")
        r=self.accept()
        self.assertEqual(r["status"],"ACK_ACCEPTED_ONCE")
        self.assertEqual(r["record"]["recipient_asserted_native_jobs"],6)
        self.assertEqual(self.audit()["record"],r["record"])

    def test_02_duplicate_refused_after_restart(self):
        self.accept()
        p=subprocess.run(self.cmd(),capture_output=True,timeout=120)
        self.assertEqual(p.returncode,3,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"DUPLICATE_ACK_BLOCKED")

    def test_03_valid_signed_contradiction(self):
        accepted=self.accept()["record"]
        o=json.loads(self.t.ack.read_bytes())
        o["receipt_sha256"]="0"*64
        self.t.ack.write_bytes(app.portable.serialize(o))
        self.t.sign()
        r=self.accept()
        self.assertEqual(r["status"],"CONTRADICTORY_SIGNED_ACK_BLOCKED")
        self.assertEqual(r["accepted_ack_sha256"],accepted["ack_sha256"])
        self.assertNotEqual(r["conflicting_ack_sha256"],accepted["ack_sha256"])
        self.assertEqual(self.audit()["record"],accepted)

    def test_04_actual_two_process_race(self):
        ps=[subprocess.Popen(self.cmd(),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=120) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"ACK_ACCEPTED_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_05_invalid_signature_does_not_consume(self):
        self.t.sign(namespace="wrong")
        with self.assertRaises(Exception): self.accept()
        self.assertFalse(self.marker.exists())
        self.t.sign()
        self.assertEqual(self.accept()["status"],"ACK_ACCEPTED_ONCE")

    def test_06_unregistered_context(self):
        with self.assertRaises(Exception): self.accept(nonce="0"*64)
        self.assertFalse(self.marker.exists())

    def test_07_context_mismatch(self):
        with self.assertRaises(Exception): self.accept(receiver="other")
        self.assertFalse(self.marker.exists())

    def test_08_empty_marker_blocks(self):
        self.marker.mkdir()
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()

    def test_09_injected_write_failure_permanent(self):
        real=app.portable.safe_write
        def fail(path,raw):
            if path==self.marker/"ACK.json": raise OSError("injected registry failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaises(Exception): self.accept()
        self.assertTrue(self.marker.exists())
        self.assertFalse((self.marker/"COMPLETE.json").exists())
        with self.assertRaises(Exception): self.accept()

    def test_10_stored_signature_tamper(self):
        self.accept()
        (self.marker/"ACK.sig").write_bytes(b"fake")
        with self.assertRaises(Exception): self.audit()

    def test_11_readonly_audit_and_no_overwrite(self):
        self.accept()
        def snapshot():
            return {str(p):p.read_bytes() for p in self.registry.rglob("*") if p.is_file()}
        before=snapshot()
        self.assertEqual(self.audit()["status"],"ACK_ACCEPTED_VERIFIED")
        with self.assertRaises(Exception): self.register()
        self.assertEqual(snapshot(),before)

    def test_12_no_execution_or_remote_claim(self):
        r=self.accept()
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["current_remote_state_proven"])
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")

if __name__=="__main__":
    unittest.main()
