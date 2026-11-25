"""Real signed native evidence, receiver-process races and restart refusal."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app032_signed_evidence_transfer"))
import test_app032 as legacy
import nonce_app033 as app

class Nonce(unittest.TestCase):
    def setUp(self):
        self.t=legacy.SignedTransfer("test_01_actual_signature_and_native_evidence")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.registry=self.t.base/"registry"
        self.registry.mkdir()
        self.target=self.t.t.receiver
        self.issued=self.issue()
        self.t.nonce=self.issued["nonce"]
        self.t.document.write_bytes(app.portable.serialize(self.issued["challenge"]["statement"]))
        self.t.sign()

    def issue(self):
        a=self.t.args()
        return app.issue(self.registry,**{k:a[k] for k in ("sender","receiver","transfer_pin","archive_pin","key_pin")})

    def args(self,**change):
        a=self.t.args()
        a.update(registry=self.registry,target=self.target)
        a.update(change)
        return a

    def receive(self,**change):
        return app.receive(**self.args(**change))

    def cmd(self,**change):
        a=self.args(**change)
        return [sys.executable,"-B",str(HERE/"nonce_app033.py"),"receive"]+[s for k,v in a.items() for s in ("--"+k.replace("_","-"),str(v))]

    def process(self,**change):
        return subprocess.run(self.cmd(**change),capture_output=True,timeout=180)

    def marker(self):
        return self.registry/self.t.nonce/"consumed"

    def test_01_signed_acceptance_and_receipt(self):
        r=self.receive()
        self.assertEqual(r["status"],"SIGNED_TRANSFER_ACCEPTED_ONCE")
        self.assertEqual(r["verified_native_jobs"],6)
        self.assertTrue((self.marker()/"COMPLETE.json").is_file())
        self.assertFalse(r["execution_authorized"])

    def test_02_replay_new_destination(self):
        self.receive()
        new=self.t.base/"replay"
        with self.assertRaises(Exception): self.receive(target=new)
        self.assertFalse(new.exists())

    def test_03_distinct_receiver_process_restart(self):
        p=self.process()
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        q=self.process(target=self.t.base/"restart-replay")
        self.assertEqual(q.returncode,3,q.stdout+q.stderr)

    def test_04_actual_two_process_race(self):
        targets=[self.t.base/"race-a",self.t.base/"race-b"]
        processes=[subprocess.Popen(self.cmd(target=x),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for x in targets]
        try:
            output=[p.communicate(timeout=180) for p in processes]
            self.assertEqual(sorted(p.returncode for p in processes),[0,3],output)
            self.assertEqual(sum(x.exists() for x in targets),1)
            self.assertTrue((self.marker()/"COMPLETE.json").exists())
        finally:
            for p in processes:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_05_invalid_signature_does_not_consume(self):
        self.t.sign(namespace="wrong")
        with self.assertRaises(Exception): self.receive()
        self.assertFalse(self.marker().exists())
        self.t.sign()
        self.assertEqual(self.receive()["status"],"SIGNED_TRANSFER_ACCEPTED_ONCE")

    def test_06_unissued_nonce_refused(self):
        with self.assertRaises(Exception): self.receive(nonce="0"*64)
        self.assertFalse(self.marker().exists())

    def test_07_context_mismatch_refused(self):
        with self.assertRaises(Exception): self.receive(receiver="agent-c")
        self.assertFalse(self.marker().exists())

    def test_08_empty_consumption_marker_blocks(self):
        self.marker().mkdir()
        with self.assertRaises(Exception): self.receive()
        self.assertFalse(self.target.exists())

    def test_09_publication_failure_retains_marker(self):
        # Inject a publication failure AFTER actual native/signature verification.
        real=app.transfer.populate
        def fail_only_final(directory,files):
            if directory==self.target: raise OSError("test publication failure")
            return real(directory,files)
        with patch.object(app.transfer,"populate",side_effect=fail_only_final):
            with self.assertRaises(Exception): self.receive()
        self.assertTrue((self.marker()/"START.json").exists())
        self.assertFalse((self.marker()/"COMPLETE.json").exists())
        with self.assertRaises(Exception): self.receive(target=self.t.base/"after-failure")

    def test_10_registry_overlap_refused(self):
        with self.assertRaises(Exception): self.receive(target=self.registry/"forbidden")
        self.assertFalse(self.marker().exists())

    def test_11_unique_challenges(self):
        seen={self.t.nonce}
        for _ in range(16):
            n=self.issue()["nonce"]
            self.assertNotIn(n,seen)
            seen.add(n)
        self.assertEqual(len(seen),17)

    def test_12_no_distributed_or_execution_claim(self):
        r=self.receive()
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["automatic_retry"])

if __name__=="__main__":
    unittest.main()
