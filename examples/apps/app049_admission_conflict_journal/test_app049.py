"""Genuine alternate signed declarations and local journal adversarial cases."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app048_sender_admission_ack_registry"))
import test_app048 as legacy
import conflict_app049 as app

class Conflict(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Sender("test_01_real_acceptance_and_audit")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.register()
        self.record=self.t.accept()["record"]
        self.pin=app.transfer.sha(app.portable.serialize(self.record))
        self.alt=self.t.alternate()
        self.journal=self.t.base/"conflict-journal"
        self.journal.mkdir()

    def args(self,**change):
        a=self.t.args(**self.alt)
        a.update(journal=self.journal,original_record_pin=self.pin,
                 conflicting_ack_pin=app.transfer.sha(a["ack"].read_bytes()))
        a.update(change)
        return a

    def audit_args(self,**change):
        a={k:v for k,v in self.args().items() if k not in
           ("ack_registry","package","ack","ack_signature","receiver_public_key")}
        a.update(change)
        return a

    def preserve(self,**change): return app.preserve(**self.args(**change))
    def audit(self,**change): return app.audit(**self.audit_args(**change))
    def folder(self):
        a=self.audit_args()
        ctx=app.prior.context(a["receiver_key_pin"],**self.t.t.trust())
        start=app.statement(ctx,self.pin,a["package_pin"],a["receipt_pin"],a["conflicting_ack_pin"])
        return self.journal/app.identity(start)

    def snapshot(self):
        return {str(p):p.read_bytes() for p in self.journal.rglob("*") if p.is_file()}

    def cmd(self,action):
        a=self.args() if action=="preserve" else self.audit_args()
        return [sys.executable,"-B",str(HERE/"conflict_app049.py"),action]+[
            s for k,v in a.items() for s in ("--"+k.replace("_","-"),str(v))]

    def test_01_real_conflict_preserved(self):
        before=self.t.snapshot()
        r=self.preserve()
        self.assertEqual(r["status"],"CONFLICT_PRESERVED")
        self.assertEqual(self.audit()["record"],r["record"])
        self.assertEqual(self.t.snapshot(),before)
        self.assertNotEqual(r["record"]["original_ack_sha256"],r["record"]["conflicting_ack_sha256"])

    def test_02_duplicate_no_overwrite(self):
        self.preserve()
        before=self.snapshot()
        self.assertEqual(self.preserve()["status"],"DUPLICATE_CONFLICT_BLOCKED")
        self.assertEqual(self.snapshot(),before)

    def test_03_same_declaration_not_conflict(self):
        a=self.t.args()
        a["conflicting_ack_pin"]=app.transfer.sha(a["ack"].read_bytes())
        with self.assertRaises(Exception): self.preserve(**a)
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_04_invalid_signature_no_entry(self):
        self.alt["ack_signature"].write_bytes(b"invalid")
        with self.assertRaises(Exception): self.preserve()
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_05_wrong_original_pin_no_entry(self):
        with self.assertRaises(Exception): self.preserve(original_record_pin="0"*64)
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_06_wrong_context_or_candidate_pin(self):
        for change in (dict(receiver="other"),dict(conflicting_ack_pin="0"*64),dict(package_pin="0"*64)):
            with self.assertRaises(Exception): self.preserve(**change)
        self.assertEqual(list(self.journal.iterdir()),[])

    def test_07_interrupted_marker_blocks(self):
        self.folder().mkdir()
        with self.assertRaises(Exception): self.preserve()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.folder().iterdir()),[])

    def test_08_interrupted_complete_blocks(self):
        real=app.portable.safe_write
        target=self.folder()/"COMPLETE.json"
        def fail(path,raw):
            if Path(path)==target: raise OSError("injected conflict completion failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"injected conflict completion failure"): self.preserve()
        with self.assertRaises(Exception): self.preserve()
        with self.assertRaises(Exception): self.audit()

    def test_09_competing_processes_one_winner(self):
        ps=[subprocess.Popen(self.cmd("preserve"),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for _ in range(2)]
        try:
            out=[p.communicate(timeout=600) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            self.assertEqual(self.audit()["status"],"CONFLICT_VERIFIED")
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_10_tampering_either_proof_blocks(self):
        self.preserve()
        for n in ("FIRST-ACK.json","OTHER-ACK.sig","ORIGINAL.json","COMPLETE.json"):
            p=self.folder()/n
            raw=p.read_bytes()
            p.write_bytes(b"{}")
            with self.assertRaises(Exception): self.audit()
            p.write_bytes(raw)
        self.assertEqual(self.audit()["status"],"CONFLICT_VERIFIED")

    def test_11_detached_readonly_restart(self):
        self.preserve()
        cmd=self.cmd("audit")
        self.t.root.rename(self.t.base/"relocated-sender-registry")
        for i,n in enumerate(("package","ack","ack_signature")):
            self.alt[n].rename(self.t.base/("relocated-conflict-"+str(i)))
        before=self.snapshot()
        p=subprocess.run(cmd,capture_output=True,timeout=600)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"CONFLICT_VERIFIED")
        self.assertEqual(self.snapshot(),before)

    def test_12_scope_and_overlap(self):
        with self.assertRaises(Exception): self.preserve(journal=self.t.root)
        r=self.preserve()
        self.assertFalse(r["fraud_proven"])
        self.assertFalse(r["remote_state_proven"])
        self.assertEqual(r["journal_completeness"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
