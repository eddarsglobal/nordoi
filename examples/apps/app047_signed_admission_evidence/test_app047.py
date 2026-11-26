"""Real recipient-signed acceptance evidence and portable sender-side replay."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app046_portable_admission_evidence"))
import test_app046 as legacy
import ack_app047 as app

class Ack(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Evidence("test_01_real_acceptance_evidence")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.accept()
        self.package=self.t.pack()
        self.base=self.t.base
        self.private=self.base/"ack-recipient-key"
        self.public=self.base/"ack-recipient.pub"
        self.ack=self.base/"recipient-ack.json"
        self.sig=self.base/"recipient-ack.sig"
        self.t.t.t.generate(self.private)
        fields=Path(str(self.private)+".pub").read_text("ascii").split()
        self.public.write_bytes((" ".join(fields[:2])+"\n").encode("ascii"))
        self.receiver_pin=app.transfer.sha(self.public.read_bytes())

    def trust(self):
        return {k:v for k,v in self.t.t.args().items() if k not in
                ("challenges","archive","statement","signature","public_key")}

    def common(self):
        return dict(self.trust(),package=self.t.output,ack=self.ack,
                    package_pin=self.package["package_sha256"],receipt_pin=self.t.receipt_pin,
                    receiver_key_pin=self.receiver_pin)

    def prepare(self,**change):
        args=self.t.t.args()
        args.update(self.common())
        args.update(change)
        return app.prepare(**args)

    def sign(self,key=None,namespace=None):
        p=subprocess.run(["ssh-keygen","-Y","sign","-f",str(key or self.private),"-n",namespace or app.NAMESPACE],
                         input=self.ack.read_bytes(),capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.sig.write_bytes(p.stdout)

    def complete(self):
        self.prepare()
        self.sign()

    def args(self,**change):
        args=self.common()
        args.update(ack_signature=self.sig,receiver_public_key=self.public)
        args.update(change)
        return args

    def verify(self,**change): return app.verify(**self.args(**change))

    def test_01_real_recipient_ack(self):
        self.complete()
        r=self.verify()
        self.assertEqual(r["status"],"RECIPIENT_ACCEPTANCE_SIGNATURE_VERIFIED")
        self.assertEqual(r["record"],self.t.result["record"])

    def test_02_original_sources_relocated(self):
        self.complete()
        self.t.relocate()
        self.assertEqual(self.verify()["status"],"RECIPIENT_ACCEPTANCE_SIGNATURE_VERIFIED")

    def test_03_sender_key_cannot_ack(self):
        self.complete()
        self.sign(key=self.t.t.t.key)
        with self.assertRaises(Exception): self.verify()

    def test_04_wrong_namespace(self):
        self.complete()
        self.sign(namespace="nordoi-chain-archive@eddarsglobal")
        with self.assertRaises(Exception): self.verify()

    def test_05_wrong_receiver_key_pin(self):
        self.complete()
        with self.assertRaises(Exception): self.verify(receiver_key_pin="0"*64)

    def test_06_identity_and_context_mismatch(self):
        self.complete()
        for change in (dict(sender="other"),dict(receiver="other"),dict(nonce="0"*64),
                       dict(head_pin="0"*64),dict(receipt_pin="0"*64),dict(package_pin="0"*64)):
            with self.assertRaises(Exception): self.verify(**change)

    def test_07_tampered_ack_refused(self):
        self.complete()
        doc=json.loads(self.ack.read_bytes())
        doc["receipt_sha256"]="0"*64
        self.ack.write_bytes(app.portable.serialize(doc))
        with self.assertRaises(Exception): self.verify()

    def test_08_tampered_package_refused(self):
        self.complete()
        self.t.output.write_bytes(b"{}")
        with self.assertRaises(Exception): self.verify()

    def test_09_corrupted_live_receipt_no_preparation(self):
        (self.t.t.marker/"COMPLETE.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.prepare()
        self.assertFalse(self.ack.exists())

    def test_10_existing_ack_preserved(self):
        self.complete()
        before=self.ack.read_bytes()
        with self.assertRaises(Exception): self.prepare()
        self.assertEqual(self.ack.read_bytes(),before)

    def test_11_readonly_restart(self):
        self.complete()
        paths=(self.t.output,self.ack,self.sig,self.public)
        before={p:p.read_bytes() for p in paths}
        cmd=[sys.executable,"-B",str(HERE/"ack_app047.py"),"verify"]+[
            s for k,v in self.args().items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=300)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["record"],self.t.result["record"])
        self.assertEqual({p:p.read_bytes() for p in paths},before)

    def test_12_scope_and_authority(self):
        self.complete()
        r=self.verify()
        self.assertTrue(r["recipient_assertion"])
        self.assertEqual(r["independent_historical_acceptance"],"NOT_PROVEN")
        self.assertEqual(r["live_nonce_state"],"NOT_VERIFIED")
        self.assertEqual(r["ack_replay_prevention"],"NOT_IMPLEMENTED")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
