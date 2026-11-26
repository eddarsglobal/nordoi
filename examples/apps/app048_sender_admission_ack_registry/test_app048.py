"""Actual signed evidence, exclusive sender registry and alternate valid declarations."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app047_signed_admission_evidence"))
import test_app047 as legacy
import sender_app048 as app

class Sender(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Ack("test_01_real_recipient_ack")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.complete()
        self.base=self.t.base
        self.root=self.base/"sender-ack-registry"
        self.root.mkdir()
        self.nonce=self.t.trust()["nonce"]
        self.entry=self.root/self.nonce
        self.marker=self.entry/"accepted"

    def context(self,**change):
        a=dict(self.t.trust(),ack_registry=self.root,receiver_key_pin=self.t.receiver_pin)
        a.update(change)
        return a

    def register(self,**change): return app.register(**self.context(**change))
    def audit(self,**change): return app.audit(**self.context(**change))

    def args(self,**change):
        a=self.t.args()
        a.update(ack_registry=self.root)
        a.update(change)
        return a

    def accept(self,**change): return app.accept(**self.args(**change))

    def cmd(self,action,**change):
        a=self.args(**change) if action=="accept" else self.context(**change)
        return [sys.executable,"-B",str(HERE/"sender_app048.py"),action]+[
            s for k,v in a.items() for s in ("--"+k.replace("_","-"),str(v))]

    def snapshot(self):
        return {str(p):p.read_bytes() for p in self.root.rglob("*") if p.is_file()}

    def alternate(self):
        # Same issued context replicated into a second laboratory registry:
        # a genuine second signature uses the other SSHSIG hash algorithm.
        a=self.t.t.t
        signed=a.t
        challenges=self.base/"split-lab-challenges"
        folder=challenges/self.nonce
        folder.mkdir(parents=True)
        (folder/"CHALLENGE.json").write_bytes((a.folder/"CHALLENGE.json").read_bytes())
        other_sig=self.base/"alternate-sender.sig"
        cmd=["ssh-keygen","-Y","sign","-f",str(signed.key),"-n",
             app.prior.prior.prior.prior.NAMESPACE,"-O","hashalg=sha256"]
        p=subprocess.run(cmd,input=signed.message.read_bytes(),capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        other_sig.write_bytes(p.stdout)
        self.assertNotEqual(other_sig.read_bytes(),signed.sig.read_bytes())
        args=a.args(challenges=challenges,signature=other_sig)
        accepted=app.prior.prior.prior.accept(**args)
        receipt_pin=app.transfer.sha(app.portable.serialize(accepted["record"]))
        package=self.base/"alternate-package.json"
        packed=app.prior.prior.pack(output=package,receipt_pin=receipt_pin,**args)
        ack=self.base/"alternate-ack.json"
        app.prior.prepare(package=package,ack=ack,package_pin=packed["package_sha256"],
                          receipt_pin=receipt_pin,receiver_key_pin=self.t.receiver_pin,**args)
        sig=self.base/"alternate-ack.sig"
        p=subprocess.run(["ssh-keygen","-Y","sign","-f",str(self.t.private),"-n",app.prior.NAMESPACE],
                         input=ack.read_bytes(),capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        sig.write_bytes(p.stdout)
        return dict(package=package,ack=ack,ack_signature=sig,
                    package_pin=packed["package_sha256"],receipt_pin=receipt_pin)

    def test_01_real_acceptance_and_audit(self):
        self.register()
        self.assertEqual(self.audit()["status"],"ACK_PENDING")
        r=self.accept()
        self.assertEqual(r["status"],"ACK_ACCEPTED_ONCE")
        self.assertEqual(self.audit()["record"],r["record"])

    def test_02_duplicate_preserves_first(self):
        self.register()
        self.accept()
        before=self.snapshot()
        self.assertEqual(self.accept()["status"],"DUPLICATE_ACK_BLOCKED")
        self.assertEqual(self.snapshot(),before)

    def test_03_two_real_competing_senders(self):
        self.register()
        ps=[subprocess.Popen(self.cmd("accept"),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=600) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"ACK_ACCEPTED_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_04_invalid_signature_no_consumption(self):
        self.register()
        self.t.sign(namespace="wrong-domain")
        with self.assertRaises(Exception): self.accept()
        self.assertFalse(self.marker.exists())

    def test_05_unregistered_nonce_blocked(self):
        with self.assertRaises(Exception): self.accept()
        self.assertFalse(self.entry.exists())

    def test_06_context_substitution_blocked(self):
        self.register()
        with self.assertRaises(Exception): self.accept(receiver="other")
        self.assertFalse(self.marker.exists())

    def test_07_incomplete_marker_permanent(self):
        self.register()
        self.marker.mkdir()
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.marker.iterdir()),[])

    def test_08_interrupted_publication_permanent(self):
        self.register()
        real=app.portable.safe_write
        target=self.marker.resolve(strict=False)/"COMPLETE.json"
        hits=[]
        def fail(path,raw):
            if Path(path).resolve(strict=False)==target:
                hits.append(target)
                raise OSError("injected receipt failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"^injected receipt failure$"): self.accept()
        self.assertEqual(hits,[target])
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()

    def test_09_authentic_contradiction_preserves_first(self):
        self.register()
        r=self.accept()
        alternate=self.alternate()
        before=self.snapshot()
        other=self.accept(**alternate)
        self.assertEqual(other["status"],"CONTRADICTORY_SIGNED_ACK_BLOCKED")
        self.assertNotEqual(other["accepted_ack_sha256"],other["conflicting_ack_sha256"])
        self.assertFalse(other["conflict_persisted"])
        self.assertEqual(self.snapshot(),before)
        self.assertEqual(self.audit()["record"],r["record"])

    def test_10_stored_package_tamper_blocked(self):
        self.register()
        self.accept()
        (self.marker/"PACKAGE.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.audit()
        with self.assertRaises(Exception): self.accept()

    def test_11_readonly_restart_without_originals(self):
        self.register()
        self.accept()
        for i,p in enumerate((self.t.t.output,self.t.ack,self.t.sig,self.t.public)):
            p.rename(self.base/("moved-"+str(i)))
        before=self.snapshot()
        p=subprocess.run(self.cmd("audit"),capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"ACK_ACCEPTED_VERIFIED")
        self.assertEqual(self.snapshot(),before)

    def test_12_scope_and_authority(self):
        self.register()
        r=self.accept()
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")
        self.assertEqual(r["independent_historical_acceptance"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
