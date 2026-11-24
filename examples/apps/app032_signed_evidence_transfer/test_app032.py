"""Genuine Ed25519 OpenSSH signatures and native Rust transfer evidence."""
import hashlib
from pathlib import Path
import secrets
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app031_pinned_evidence_transfer"))
import test_app031 as legacy
import signed_app032 as app

class SignedTransfer(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Transfer("test_01_real_receiver_separate_process")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.prepare()
        self.base=self.t.base
        self.private=self.base/"sender-key"
        self.public=self.base/"trusted.pub"
        self.document=self.base/"statement.json"
        self.signature=self.base/"statement.sig"
        self.nonce=secrets.token_hex(32)
        self.generate(self.private)
        fields=Path(str(self.private)+".pub").read_text("ascii").split()
        self.public.write_bytes((" ".join(fields[:2])+"\n").encode("ascii"))
        self.key_pin=hashlib.sha256(self.public.read_bytes()).hexdigest()
        a=self.args()
        for k in ("public_key","signed_statement","signature"): a.pop(k)
        a["target"]=self.document
        app.prepare(**a)
        self.sign()

    def generate(self,path):
        p=subprocess.run(["ssh-keygen","-q","-t","ed25519","-N","","-f",str(path)],
                         capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)

    def sign(self,key=None,namespace=None):
        p=subprocess.run(["ssh-keygen","-Y","sign","-f",str(key or self.private),
                         "-n",namespace or app.NAMESPACE],input=self.document.read_bytes(),
                         capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.signature.write_bytes(p.stdout)

    def args(self):
        return dict(packet=self.t.packet,transfer_pin=self.t.tp,archive_pin=self.t.ap,
            sender="agent-a",receiver="agent-b",nonce=self.nonce,key_pin=self.key_pin,
            public_key=self.public,signed_statement=self.document,signature=self.signature,
            target=self.t.receiver)

    def receive(self,**change):
        a=self.args()
        a.update(change)
        return app.receive(**a)

    def blocked(self,**change):
        with self.assertRaises(Exception): self.receive(**change)
        self.assertFalse(self.t.receiver.exists())

    def test_01_actual_signature_and_native_evidence(self):
        r=self.receive()
        self.assertEqual(r["status"],"SIGNED_TRANSFER_VERIFIED")
        self.assertEqual(r["crypto_signature"],"SSHSIG_ED25519_VERIFIED")
        self.assertEqual(r["verified_native_jobs"],6)

    def test_02_wrong_key_pin(self):
        self.blocked(key_pin="0"*64)

    def test_03_wrong_receiver(self):
        self.blocked(receiver="agent-c")

    def test_04_wrong_sender(self):
        self.blocked(sender="agent-c")

    def test_05_wrong_nonce(self):
        self.blocked(nonce="0"*64)

    def test_06_wrong_namespace(self):
        self.sign(namespace="nordoi-other")
        self.blocked()

    def test_07_other_key_signature(self):
        other=self.base/"other-key"
        self.generate(other)
        self.sign(key=other)
        self.blocked()

    def test_08_altered_statement(self):
        self.document.write_bytes(self.document.read_bytes()+b" ")
        self.blocked()

    def test_09_altered_packet(self):
        self.t.packet.write_bytes(self.t.packet.read_bytes()+b" ")
        self.blocked()

    def test_10_signature_truncated(self):
        self.signature.write_bytes(self.signature.read_bytes()[:50])
        self.blocked()

    def test_11_wrong_archive_or_transfer_pin(self):
        self.blocked(archive_pin="0"*64)
        self.blocked(transfer_pin="0"*64)

    def test_12_no_replay_or_identity_overclaim(self):
        r=self.receive()
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["transport_encrypted"])
        self.assertEqual(r["nonce_consumption"],"NOT_IMPLEMENTED")
        self.assertEqual(r["independent_key_custody"],"NOT_PROVEN")
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")

if __name__=="__main__":
    unittest.main()
