"""Real APP-028 native evidence and adversarial receipt checks."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app028_local_exclusive_resume"))
import test_app028 as legacy
import audit_app029 as app

class ReservationAudit(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Native("test_12_success_receipts_retained")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.f=self.t.f
        self.claim=self.t.claim
        self.attempt=self.claim/"native-attempt"

    def finish(self):
        r=app.exclusive.execute(**self.t.args())
        self.pin=r["continuation"]["new_suffix_sha256"]
        return r

    def call(self,**change):
        a=self.t.args()
        for name in ("engine","source","workers","consent"):
            a.pop(name)
        a["final_suffix_pin"]=getattr(self,"pin","0"*64)
        a.update(change)
        return app.classify(**a)

    def blocked(self,**change):
        r=self.call(**change)
        self.assertEqual(r["status"],"UNCERTAIN_BLOCKED",r)
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["automatic_retry"])
        return r

    @staticmethod
    def mutate(path,fn):
        obj=json.loads(path.read_bytes())
        fn(obj)
        path.write_text(json.dumps(obj),encoding="utf-8")

    def test_01_complete_real_native_evidence(self):
        self.finish()
        r=self.call()
        self.assertEqual(r["status"],"COMPLETE_EVIDENCE_VERIFIED",r)
        self.assertEqual((r["historical_events"],r["new_verified_events"],r["verified_native_jobs"]),(3,3,6))
        self.assertFalse(r["execution_authorized"])

    def test_02_no_reservation(self):
        self.blocked()
        self.assertFalse(self.claim.exists())

    def test_03_abandoned_reservation(self):
        app.exclusive.reserve(self.f.root,{})
        self.blocked()
        self.assertTrue(self.claim.is_dir())

    def test_04_missing_outer_completion(self):
        self.finish()
        (self.claim/"COMPLETE.json").unlink()
        self.blocked()

    def test_05_forged_outer_completion(self):
        self.finish()
        self.mutate(self.claim/"COMPLETE.json",lambda x:x["continuation"].update(verified_native_jobs=999))
        self.blocked()

    def test_06_forged_native_completion(self):
        self.finish()
        self.mutate(self.attempt/"COMPLETE.json",lambda x:x.update(new_verified_events=99))
        self.blocked()

    def test_07_forged_start_context(self):
        self.finish()
        self.mutate(self.claim/"START.json",lambda x:x["context"].update(engine_pin="0"*64))
        self.blocked()

    def test_08_forged_native_start(self):
        self.finish()
        self.mutate(self.attempt/"START.json",lambda x:x.update(historical_events=2))
        self.blocked()

    def test_09_wrong_external_pins(self):
        self.finish()
        for k in ("old_pin","new_pin","suffix_pin","engine_pin","source_pin","final_suffix_pin"):
            self.blocked(**{k:"0"*64})

    def test_10_truncated_final_suffix_even_if_rehashed(self):
        self.finish()
        path=self.attempt/"native-suffix.jsonl"
        raw=path.read_bytes()[:-1]
        path.write_bytes(raw)
        self.blocked(final_suffix_pin=hashlib.sha256(raw).hexdigest())

    def test_11_native_bytes_tampered(self):
        self.finish()
        files=sorted((self.f.root/"job-0006"/"quality-decision").iterdir())
        path=files[0]
        path.write_bytes(path.read_bytes()+b"tampered")
        self.blocked()

    def test_12_duplicate_key_and_readonly_success(self):
        self.finish()
        def snapshot():
            return {str(p.relative_to(self.f.base)):p.read_bytes() for p in self.f.base.rglob("*") if p.is_file()}
        before=snapshot()
        r=self.call()
        self.assertEqual(r["status"],"COMPLETE_EVIDENCE_VERIFIED",r)
        self.assertEqual(before,snapshot())
        self.assertEqual(r["native_processes_started"],0)
        self.assertFalse(r["engine_execution_identity_attested"])
        path=self.claim/"START.json"
        raw=path.read_text("utf-8").rstrip()
        path.write_text(raw[:-1]+',"status":"RESERVED"}',encoding="utf-8")
        self.blocked()

if __name__=="__main__":
    unittest.main()
