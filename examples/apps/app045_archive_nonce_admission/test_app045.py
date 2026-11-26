"""Actual authenticated archives, local challenges, process races and interruption."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app044_signed_chain_archive"))
import test_app044 as legacy
import admission_app045 as app

class Admission(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Signed("test_01_real_signed_archive")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.base
        self.challenges=self.base/"archive-challenges"
        self.challenges.mkdir()
        self.issued=self.issue()
        self.nonce=self.issued["nonce"]
        self.folder=self.challenges/self.nonce
        self.marker=self.folder/"consumed"
        self.t.prepare(nonce=self.nonce)
        self.t.sign()

    def issue(self,**change):
        a={k:v for k,v in self.t.args().items() if k not in ("archive","statement","nonce")}
        a.update(challenges=self.challenges)
        a.update(change)
        return app.issue(**a)

    def args(self,**change):
        a=self.t.args(nonce=self.nonce,signature=self.t.sig,public_key=self.t.public)
        a.update(challenges=self.challenges)
        a.update(change)
        return a

    def accept(self,**change): return app.accept(**self.args(**change))
    def audit(self,**change): return app.audit(**self.args(**change))

    def cmd(self,action="accept",**change):
        return [sys.executable,"-B",str(HERE/"admission_app045.py"),action]+[
            s for k,v in self.args(**change).items() for s in ("--"+k.replace("_","-"),str(v))]

    def test_01_real_acceptance_and_audit(self):
        r=self.accept()
        self.assertEqual(r["status"],"SIGNED_ARCHIVE_ACCEPTED_ONCE")
        self.assertEqual(self.audit()["record"],r["record"])
        self.assertEqual(r["record"]["challenge_sha256"],self.issued["challenge_sha256"])

    def test_02_replay_after_restart(self):
        self.accept()
        p=subprocess.run(self.cmd(),capture_output=True,timeout=300)
        self.assertEqual(p.returncode,3,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"UNCERTAIN_BLOCKED")

    def test_03_two_real_competing_receivers(self):
        ps=[subprocess.Popen(self.cmd(),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=300) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"ARCHIVE_ACCEPTANCE_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_04_wrong_signature_does_not_consume(self):
        self.t.sign(namespace="wrong-domain")
        with self.assertRaises(Exception): self.accept()
        self.assertFalse(self.marker.exists())
        self.t.sign()
        self.assertEqual(self.accept()["status"],"SIGNED_ARCHIVE_ACCEPTED_ONCE")

    def test_05_unissued_nonce_refused(self):
        with self.assertRaises(Exception): self.accept(nonce="0"*64)
        self.assertFalse(self.marker.exists())

    def test_06_wrong_context_refused(self):
        with self.assertRaises(Exception): self.accept(receiver="other")
        self.assertFalse(self.marker.exists())

    def test_07_incomplete_marker_permanent(self):
        self.marker.mkdir()
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.marker.iterdir()),[])

    def test_08_interrupted_publication_permanent(self):
        real=app.portable.safe_write
        target=self.marker.resolve(strict=False)/"COMPLETE.json"
        injected=[]
        def fail(path,raw):
            if Path(path).resolve(strict=False)==target:
                injected.append(target)
                raise OSError("injected admission failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"^injected admission failure$"): self.accept()
        self.assertEqual(injected,[target])
        self.assertTrue((self.marker/"START.json").is_file())
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()

    def test_09_tampered_receipt_refused(self):
        self.accept()
        (self.marker/"COMPLETE.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.audit()
        with self.assertRaises(Exception): self.accept()

    def test_10_fresh_challenge_distinct_and_old_signature_refused(self):
        second=self.issue()["nonce"]
        self.assertNotEqual(second,self.nonce)
        with self.assertRaises(Exception): self.accept(nonce=second)
        self.assertFalse((self.challenges/second/"consumed").exists())

    def test_11_readonly_restart_audit(self):
        self.accept()
        def snap():
            return {str(p):p.read_bytes() for p in self.challenges.rglob("*") if p.is_file()}
        before=snap()
        p=subprocess.run(self.cmd("audit"),capture_output=True,timeout=300)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"ARCHIVE_ACCEPTANCE_VERIFIED")
        self.assertEqual(snap(),before)

    def test_12_scope_and_authority(self):
        r=self.accept()
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
