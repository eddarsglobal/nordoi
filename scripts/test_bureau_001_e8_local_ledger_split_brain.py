"""NORDOI R6F E8: exclusively machine-controlled split-brain proof."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e8_local_ledger_split_brain as e8

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()

N="a"*64
R="b"*64

class PortableE8(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix=e8.LAB_PREFIX)
        self.root=Path(self.tmp.name)
        if os.name=="posix":os.chmod(self.root,0o700)
    def tearDown(self):self.tmp.cleanup()
    def test_01_source_audit_is_block(self):
        self.assertEqual(e8.audit(ROOT,head())["result"],"BLOCK")
    def test_02_parent_exact(self):
        self.assertEqual(e8.PARENT,"81c84f116b3ae6003b84c335e47af4d1b84637ef")
    def test_03_wrong_head_denied(self):
        with self.assertRaises(Exception):e8.audit(ROOT,"f"*40)
    def test_04_two_real_independent_sqlite_files(self):
        original=e8.e7.initialize(self.root)
        for fork in e8.FORKS:shutil.copy2(original,e8.fork_path(self.root,fork))
        self.assertNotEqual(e8.fork_path(self.root,"A"),e8.fork_path(self.root,"B"))
    def test_05_each_fork_can_locally_commit_same_nonce(self):
        original=e8.e7.initialize(self.root)
        for fork in e8.FORKS:
            db=e8.fork_path(self.root,fork)
            shutil.copy2(original,db)
            self.assertTrue(e8.e7.consume_atomic(db,N,R,head()).startswith("LOCAL_SPENT"))
            self.assertEqual(e8.e7.count_nonce(db,N),1)
    def test_06_each_fork_rejects_local_duplicate(self):
        original=e8.e7.initialize(self.root)
        for fork in e8.FORKS:
            db=e8.fork_path(self.root,fork)
            shutil.copy2(original,db)
            e8.e7.consume_atomic(db,N,R,head())
            self.assertIn("DUPLICATE_BLOCKED",e8.e7.consume_atomic(db,N,R,head()))
    def test_07_original_remains_unspent(self):
        original=e8.e7.initialize(self.root)
        for fork in e8.FORKS:
            db=e8.fork_path(self.root,fork)
            shutil.copy2(original,db)
            e8.e7.consume_atomic(db,N,R,head())
        self.assertEqual(e8.e7.count_nonce(original,N),0)
    def test_08_local_events_have_distinct_fork_hash(self):
        self.assertNotEqual(e8.signed_event("A",N,R,head())["event_sha256"],
                            e8.signed_event("B",N,R,head())["event_sha256"])
    def test_09_two_events_detect_split_brain(self):
        x=e8.reconcile([e8.signed_event(f,N,R,head()) for f in e8.FORKS])
        self.assertTrue(x["split_brain_detected"])
        self.assertEqual(x["result"],"BLOCK")
    def test_10_missing_fork_denied(self):
        x=e8.reconcile([e8.signed_event("A",N,R,head())])
        self.assertIn("INCOMPLETE",x["reason"])
    def test_11_same_fork_double_submission_denied(self):
        with self.assertRaises(e8.Blocked):
            e8.reconcile([e8.signed_event("A",N,R,head())]*2)
    def test_12_tampered_event_hash_denied(self):
        events=[e8.signed_event(f,N,R,head()) for f in e8.FORKS]
        events[0]["event_sha256"]="f"*64
        with self.assertRaises(e8.Blocked):e8.reconcile(events)
    def test_13_different_nonce_does_not_falsely_claim_overlap(self):
        x=e8.reconcile([e8.signed_event("A",N,R,head()),
                        e8.signed_event("B","c"*64,R,head())])
        self.assertFalse(x["split_brain_detected"])
        self.assertEqual(x["result"],"BLOCK")
    def test_14_invalid_fork_rejected(self):
        with self.assertRaises(e8.Blocked):e8.fork_path(self.root,"C")
    def test_15_native_and_production_never_unlocked(self):
        x=e8.result("LAB",r6g="OPEN",production="YES",native_admission_gates_unmet=0)
        self.assertEqual(x["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(x["production"],"NOT_AUTHORIZED")
        self.assertEqual(x["native_admission_gates_unmet"],12)
    def test_16_invalid_nonce_denied(self):
        with self.assertRaises(e8.Blocked):e8.signed_event("A","bad",R,head())
    def test_17_empty_fork_evidence_denied(self):
        self.assertIn("INCOMPLETE",e8.reconcile([])["reason"])
    def test_18_zero_humans_and_independent_receipts(self):
        x=e8.audit(ROOT,head())
        self.assertEqual(x["real_human_testers"],0)
        self.assertEqual(x["external_witness_receipts"],0)

@unittest.skipUnless(e8.e7.e6.r4.is_posix_ssh(),
    "requires genuine POSIX OpenSSH Ed25519, no simulated cryptography")
class RealE8(unittest.TestCase):
    def test_19_genuine_full_fork_exercise(self):
        x=e8.exercise(ROOT,head())
        self.assertTrue(x["real_signed_receipt_verified_by_both"])
    def test_20_two_distinct_os_processes(self):
        self.assertEqual(e8.exercise(ROOT,head())["distinct_os_processes"],2)
    def test_21_both_local_spends_and_reconciliation(self):
        x=e8.exercise(ROOT,head())
        self.assertTrue(x["same_nonce_locally_committed_on_each_fork"])
        self.assertTrue(x["local_split_brain_detected_on_reconciliation"])
    def test_22_withheld_evidence_fails_closed(self):
        self.assertTrue(e8.exercise(ROOT,head())["missing_branch_evidence_blocked"])
    def test_23_no_externally_authorized_history(self):
        x=e8.exercise(ROOT,head())
        self.assertFalse(x["globally_monotonic_antirollback_verified"])
    def test_24_full_lab_returns_only_block(self):
        x=e8.exercise(ROOT,head())
        self.assertEqual(x["result"],"BLOCK")
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertEqual(x["production"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
