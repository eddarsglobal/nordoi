"""Real one-time acceptance, portable evidence, independent receipt pin."""
import base64
import json
from pathlib import Path
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app045_archive_nonce_admission"))
import test_app045 as legacy
import evidence_app046 as app

class Evidence(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Admission("test_01_real_acceptance_and_audit")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.base
        self.output=self.base/"admission-evidence.json"
        self.result=None

    def accept(self):
        self.result=self.t.accept()
        self.receipt_pin=app.transfer.sha(app.portable.serialize(self.result["record"]))

    def pack(self,**change):
        args=self.t.args(output=self.output,receipt_pin=self.receipt_pin)
        args.update(change)
        return app.pack(**args)

    def args(self,pin,**change):
        args={k:v for k,v in self.t.args().items() if k not in
              ("challenges","archive","statement","signature","public_key")}
        args.update(output=self.output,package_pin=pin,receipt_pin=self.receipt_pin)
        args.update(change)
        return args

    def verify(self,pin,**change):
        return app.verify(**self.args(pin,**change))

    def mutate(self,fn):
        doc=json.loads(self.output.read_bytes())
        fn(doc)
        raw=app.portable.serialize(doc)
        self.output.write_bytes(raw)
        return app.transfer.sha(raw)

    def relocate(self):
        for i,p in enumerate((self.t.challenges,self.t.t.t.output,self.t.t.message,self.t.t.sig,self.t.t.public)):
            p.rename(self.base/("detached-"+str(i)))

    def test_01_real_acceptance_evidence(self):
        self.accept()
        p=self.pack()
        v=self.verify(p["package_sha256"])
        self.assertEqual(v["status"],"ADMISSION_EVIDENCE_VERIFIED")
        self.assertEqual(v["record"],self.result["record"])

    def test_02_all_original_inputs_relocated(self):
        self.accept()
        p=self.pack()
        self.relocate()
        self.assertEqual(self.verify(p["package_sha256"])["record"],self.result["record"])

    def test_03_deterministic_exact_members(self):
        self.accept()
        p=self.pack()
        q=self.pack(output=self.base/"second.json")
        self.assertEqual(p["package_sha256"],q["package_sha256"])
        doc=json.loads(self.output.read_bytes())
        self.assertEqual(set(doc["members"]),set(app.LIMITS))

    def test_04_unconsumed_or_incomplete_refused(self):
        self.receipt_pin="0"*64
        with self.assertRaises(Exception): self.pack()
        self.t.marker.mkdir()
        with self.assertRaises(Exception): self.pack()
        self.assertFalse(self.output.exists())

    def test_05_wrong_external_pins(self):
        self.accept()
        p=self.pack()
        with self.assertRaises(Exception): self.verify("0"*64)
        for key in ("receipt_pin","head_pin","key_pin","anchor_pin","source_plan_pin","archive_pin"):
            with self.assertRaises(Exception): self.verify(p["package_sha256"],**{key:"0"*64})

    def test_06_receipt_tamper_even_with_new_package_pin(self):
        self.accept()
        self.pack()
        pin=self.mutate(lambda o:o["members"].update({"COMPLETE.json":"e30="}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_07_signature_tamper_even_with_new_package_pin(self):
        self.accept()
        self.pack()
        pin=self.mutate(lambda o:o["members"].update({"statement.sig":"eA=="}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_08_extra_member_path_refused(self):
        self.accept()
        self.pack()
        pin=self.mutate(lambda o:o["members"].update({"../escape":"eA=="}))
        with self.assertRaises(Exception): self.verify(pin)

    def test_09_missing_challenge_refused(self):
        self.accept()
        self.pack()
        pin=self.mutate(lambda o:o["members"].pop("CHALLENGE.json"))
        with self.assertRaises(Exception): self.verify(pin)

    def test_10_source_receipt_tamper_no_publication(self):
        self.accept()
        (self.t.marker/"COMPLETE.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.pack()
        self.assertFalse(self.output.exists())

    def test_11_readonly_restart_no_overwrite(self):
        self.accept()
        p=self.pack()
        before=self.output.read_bytes()
        cmd=[sys.executable,"-B",str(HERE/"evidence_app046.py"),"verify"]+[
            s for k,v in self.args(p["package_sha256"]).items()
            for s in ("--"+k.replace("_","-"),str(v))]
        proc=subprocess.run(cmd,capture_output=True,timeout=300)
        self.assertEqual(proc.returncode,0,proc.stdout+proc.stderr)
        with self.assertRaises(Exception): self.pack()
        self.assertEqual(self.output.read_bytes(),before)

    def test_12_scope_and_authority(self):
        self.accept()
        p=self.pack()
        self.assertEqual(p["independent_historical_acceptance"],"NOT_PROVEN")
        self.assertEqual(p["live_nonce_state"],"NOT_VERIFIED")
        self.assertFalse(p["execution_authorized"])
        self.assertFalse(p["reservation_release_authorized"])
        self.assertFalse(p["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
