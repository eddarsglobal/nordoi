"""Authentic Rust evidence, separate receiver signing key and sender verification."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app035_destination_bound_reception"))
import test_app035 as legacy
import ack_app036 as app

class Ack(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Bound("test_01_real_bound_receipt")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.t.base
        self.private=self.base/"receiver-key"
        self.public=self.base/"receiver-trusted.pub"
        self.t.t.generate(self.private)
        fields=Path(str(self.private)+".pub").read_text("ascii").split()
        self.public.write_bytes((" ".join(fields[:2])+"\n").encode("ascii"))
        self.receiver_pin=app.transfer.sha(self.public.read_bytes())
        self.challenge_pin=app.transfer.sha(app.portable.serialize(self.t.issued["challenge"]))
        self.ack=self.base/"ack.json"
        self.sig=self.base/"ack.sig"

    def prepare(self,**change):
        a=self.t.args()
        a.update(receiver_key_pin=self.receiver_pin,challenge_pin=self.challenge_pin,ack=self.ack)
        a.update(change)
        return app.prepare(**a)

    def sign(self,key=None,namespace=None):
        p=subprocess.run(["ssh-keygen","-Y","sign","-f",str(key or self.private),
                          "-n",namespace or app.NAMESPACE],input=self.ack.read_bytes(),
                         capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.sig.write_bytes(p.stdout)

    def complete(self):
        r=self.t.receive()
        p=self.prepare()
        self.sign()
        self.assertEqual(p["receipt_sha256"],app.transfer.sha(app.portable.serialize(r)))
        return r,p

    def args(self,**change):
        a={k:self.t.args()[k] for k in ("sender","receiver","nonce","transfer_pin","archive_pin","key_pin")}
        a.update(receiver_key_pin=self.receiver_pin,challenge_pin=self.challenge_pin,ack=self.ack,
                 ack_signature=self.sig,receiver_public_key=self.public)
        a.update(change)
        return a

    def verify(self,**change):
        return app.verify(**self.args(**change))

    def test_01_real_signed_ack(self):
        r,p=self.complete()
        v=self.verify()
        self.assertEqual(v["status"],"RECIPIENT_ACK_SIGNATURE_VERIFIED")
        self.assertEqual(v["receipt_sha256"],p["receipt_sha256"])
        self.assertEqual(v["recipient_asserted_native_jobs"],6)

    def test_02_sender_process_without_destination(self):
        self.complete()
        self.t.target.rename(self.t.parent/"unavailable")
        cmd=[sys.executable,"-B",str(HERE/"ack_app036.py"),"verify"]+[
            s for k,v in self.args().items() for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=120)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertFalse(json.loads(p.stdout)["current_remote_state_proven"])

    def test_03_unreceived_transfer_refused(self):
        with self.assertRaises(Exception): self.prepare()
        self.assertFalse(self.ack.exists())

    def test_04_identical_copy_substitution_refused(self):
        self.t.receive()
        other=self.t.parent/"original"
        self.t.target.rename(other)
        shutil.copytree(other,self.t.target)
        with self.assertRaises(Exception): self.prepare()
        self.assertFalse(self.ack.exists())

    def test_05_wrong_receiver_key(self):
        self.complete()
        with self.assertRaises(Exception): self.verify(receiver_key_pin="0"*64)

    def test_06_sender_key_cannot_ack(self):
        self.complete()
        self.sign(key=self.t.t.private)
        with self.assertRaises(Exception): self.verify()

    def test_07_wrong_namespace(self):
        self.complete()
        self.sign(namespace="wrong-domain")
        with self.assertRaises(Exception): self.verify()

    def test_08_wrong_context(self):
        self.complete()
        for k,v in (("sender","other"),("receiver","other"),("nonce","0"*64),
                    ("transfer_pin","0"*64),("archive_pin","0"*64),("challenge_pin","0"*64)):
            with self.assertRaises(Exception): self.verify(**{k:v})

    def test_09_receipt_digest_tampered(self):
        self.complete()
        o=json.loads(self.ack.read_bytes())
        o["receipt_sha256"]="0"*64
        self.ack.write_bytes(app.portable.serialize(o))
        with self.assertRaises(Exception): self.verify()

    def test_10_existing_ack_preserved(self):
        self.complete()
        before=self.ack.read_bytes()
        with self.assertRaises(Exception): self.prepare()
        self.assertEqual(before,self.ack.read_bytes())

    def test_11_wrong_challenge_pin_preparation(self):
        self.t.receive()
        with self.assertRaises(Exception): self.prepare(challenge_pin="0"*64)
        self.assertFalse(self.ack.exists())

    def test_12_no_remote_or_execution_overclaim(self):
        self.complete()
        v=self.verify()
        self.assertFalse(v["remote_filesystem_independently_verified"])
        self.assertFalse(v["current_remote_state_proven"])
        self.assertFalse(v["execution_authorized"])
        self.assertFalse(v["nonce_reset_authorized"])
        self.assertEqual(v["ack_replay_prevention"],"NOT_IMPLEMENTED")

if __name__=="__main__":
    unittest.main()
