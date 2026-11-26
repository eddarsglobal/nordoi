"""Real signed portable chain; recipient trust and domain separation."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app043_portable_chain_archive"))
import test_app043 as legacy
import signed_app044 as app

class Signed(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Archive("test_01_real_portable_replay")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.archive=self.t.create()
        self.base=self.t.base
        self.key=self.base/"archive-signing-key"
        self.public=self.base/"archive-trusted.pub"
        self.message=self.base/"archive-statement.json"
        self.sig=self.base/"archive-statement.sig"
        self.generate(self.key)
        fields=Path(str(self.key)+".pub").read_text("ascii").split()
        self.public.write_bytes((" ".join(fields[:2])+"\n").encode("ascii"))
        self.key_pin=app.transfer.sha(self.public.read_bytes())

    def generate(self,path):
        p=subprocess.run(["ssh-keygen","-q","-t","ed25519","-N","","-f",str(path)],
                         capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)

    def args(self,**change):
        a=dict(archive=self.t.output,statement=self.message,sender="agent-archive",
               receiver="agent-auditor",nonce="a"*64,key_pin=self.key_pin,
               archive_pin=self.archive["archive_sha256"],source_plan_pin=self.t.t.pin,
               anchor_pin=self.t.t.anchor,head_pin=self.t.t.head)
        a.update(change)
        return a

    def prepare(self,**change):
        return app.prepare(**self.args(**change))

    def sign(self,key=None,namespace=None):
        p=subprocess.run(["ssh-keygen","-Y","sign","-f",str(key or self.key),"-n",namespace or app.NAMESPACE],
                         input=self.message.read_bytes(),capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.sig.write_bytes(p.stdout)

    def complete(self):
        self.prepare()
        self.sign()

    def verify(self,**change):
        a=self.args(signature=self.sig,public_key=self.public)
        a.update(change)
        return app.verify(**a)

    def test_01_real_signed_archive(self):
        self.complete()
        r=self.verify()
        self.assertEqual(r["status"],"SIGNED_CHAIN_ARCHIVE_VERIFIED")
        self.assertEqual(r["crypto_signature"],"SSHSIG_ED25519_VERIFIED")
        self.assertEqual(r["transition_count"],2)

    def test_02_wrong_key_pin(self):
        self.complete()
        with self.assertRaises(Exception): self.verify(key_pin="0"*64)

    def test_03_wrong_signing_key(self):
        self.complete()
        other=self.base/"other-archive-key"
        self.generate(other)
        self.sign(key=other)
        with self.assertRaises(Exception): self.verify()

    def test_04_wrong_namespace(self):
        self.complete()
        self.sign(namespace="nordoi-transfer@eddarsglobal")
        with self.assertRaises(Exception): self.verify()

    def test_05_identity_and_nonce_mismatch(self):
        self.complete()
        for change in (dict(sender="other"),dict(receiver="other"),dict(nonce="b"*64)):
            with self.assertRaises(Exception): self.verify(**change)

    def test_06_expected_pins_mismatch(self):
        self.complete()
        for k in ("archive_pin","source_plan_pin","anchor_pin","head_pin"):
            with self.assertRaises(Exception): self.verify(**{k:"0"*64})

    def test_07_tampered_archive(self):
        self.complete()
        self.t.output.write_bytes(b"{}")
        with self.assertRaises(Exception): self.verify()

    def test_08_tampered_statement(self):
        self.complete()
        o=json.loads(self.message.read_bytes())
        o["head_sha256"]="0"*64
        self.message.write_bytes(app.portable.serialize(o))
        with self.assertRaises(Exception): self.verify()

    def test_09_wrong_head_preparation(self):
        with self.assertRaises(Exception): self.prepare(head_pin="0"*64)
        self.assertFalse(self.message.exists())

    def test_10_no_statement_overwrite(self):
        self.complete()
        before=self.message.read_bytes()
        with self.assertRaises(Exception): self.prepare()
        self.assertEqual(self.message.read_bytes(),before)

    def test_11_relocated_source_readonly_restart(self):
        self.complete()
        self.t.t.t.successors.rename(self.base/"detached-successors")
        self.t.t.first_journal.rename(self.base/"detached-history")
        self.t.t.plan.rename(self.base/"detached-plan")
        paths=[self.t.output,self.message,self.sig,self.public]
        before={p:p.read_bytes() for p in paths}
        cmd=[sys.executable,"-B",str(HERE/"signed_app044.py"),"verify"]+[
            s for k,v in self.args(signature=self.sig,public_key=self.public).items()
            for s in ("--"+k.replace("_","-"),str(v))]
        p=subprocess.run(cmd,capture_output=True,timeout=300)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["head_sha256"],self.t.t.head)
        self.assertEqual({p:p.read_bytes() for p in paths},before)

    def test_12_scope_and_authority(self):
        self.complete()
        r=self.verify()
        self.assertEqual(r["nonce_consumption"],"NOT_IMPLEMENTED")
        self.assertEqual(r["independent_key_custody"],"NOT_PROVEN")
        self.assertEqual(r["freshness"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
