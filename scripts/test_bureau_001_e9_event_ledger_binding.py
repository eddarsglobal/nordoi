"""E9 real event↔disk ledger mismatch tests, machine-only research."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e9_event_ledger_binding as e9

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

N="a"*64
RECEIPT="b"*64
def events(h,nonce=N,receipt=RECEIPT):
    return [e9.e8.signed_event(f,nonce,receipt,h) for f in e9.e8.FORKS]

class E9PortableTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix=e9.e8.LAB_PREFIX)
        self.root=Path(self.tmp.name)
        if os.name=="posix":os.chmod(self.root,0o700)
    def tearDown(self):self.tmp.cleanup()
    def seed(self,nonce=N,receipt=RECEIPT):
        orig=e9.e8.e7.initialize(self.root)
        for fork in e9.e8.FORKS:
            db=e9.e8.fork_path(self.root,fork)
            shutil.copy2(orig,db)
            self.assertIn("COMMITTED",e9.e8.e7.consume_atomic(
                db,nonce,receipt,head()))
        return orig
    def test_01_e9_source_audit_denies(self):
        self.assertEqual(e9.audit(ROOT,head())["result"],"BLOCK")
    def test_02_e8_parent_head_pinned(self):
        self.assertEqual(e9.PARENT,"bf4c0d66256ad3a871e44cae39808af5ea1418d4")
    def test_03_two_green_e8_ci_runs_pinned(self):
        c=e9.e3.strictly_load(e9.e3.real_file_bytes(ROOT/e9.CONTRACT))
        self.assertEqual(c["prior_e8"]["specialized_run_id"],44036977230)
        self.assertEqual(c["prior_e8"]["rust_run_id"],44036977539)
    def test_04_permanent_zero_human_policy(self):
        c=e9.e3.strictly_load(e9.e3.real_file_bytes(ROOT/e9.CONTRACT))
        self.assertFalse(c["human_testers_allowed"])
        self.assertEqual(c["human_testers_executed"],0)
        self.assertEqual(c["external_witness_receipts"],0)
    def test_05_bad_source_head_rejected(self):
        with self.assertRaises(Exception):e9.audit(ROOT,"0"*40)
    def test_06_real_two_fork_records_created(self):
        self.seed()
        self.assertEqual(e9.record(self.root,"A"),e9.record(self.root,"B"))
    def test_07_true_record_exact_tuple(self):
        self.seed()
        self.assertEqual(e9.record(self.root,"A"),
            (e9.e8.e7.NAMESPACE,N,RECEIPT,head()))
    def test_08_honest_event_passes_record_guard(self):
        self.seed()
        self.assertTrue(e9.guard_record(self.root,events(head())[0],
                                        N,RECEIPT,head()))
    def test_09_honest_events_still_conflict_block(self):
        self.seed()
        d=e9.guarded_reconcile(self.root,events(head()),N,RECEIPT,head())
        self.assertTrue(d["local_split_brain_detected"])
        self.assertEqual(d["result"],"BLOCK")
    def test_10_forged_b_event_has_valid_local_sha(self):
        fake=e9.e8.signed_event("B","f"*64,RECEIPT,head())
        self.assertEqual(fake,e9.e8.signed_event(
            fake["fork"],fake["nonce"],fake["receipt"],fake["head"]))
    def test_11_old_e8_reconcile_misses_existing_conflict_flag(self):
        self.seed()
        forged=[events(head())[0],e9.e8.signed_event("B","f"*64,RECEIPT,head())]
        d=e9.e8.reconcile(forged)
        self.assertIs(d["split_brain_detected"],False)
        self.assertEqual(d["result"],"BLOCK")
    def test_12_new_guard_rejects_forged_record_claim(self):
        self.seed()
        forged=[events(head())[0],e9.e8.signed_event("B","f"*64,RECEIPT,head())]
        with self.assertRaisesRegex(e9.Blocked,"EVENT_PERSISTED_ROW_MISMATCH"):
            e9.guarded_reconcile(self.root,forged,N,RECEIPT,head())
    def test_13_changed_receipt_rejected_against_db(self):
        self.seed()
        with self.assertRaisesRegex(e9.Blocked,"EVENT_PERSISTED_ROW_MISMATCH"):
            e9.guard_record(self.root,
                e9.e8.signed_event("A",N,"e"*64,head()),N,RECEIPT,head())
    def test_14_changed_head_rejected_against_db(self):
        self.seed()
        with self.assertRaisesRegex(e9.Blocked,"EVENT_PERSISTED_ROW_MISMATCH"):
            e9.guard_record(self.root,
                e9.e8.signed_event("A",N,RECEIPT,"f"*40),N,RECEIPT,head())
    def test_15_forged_event_sha_not_accepted(self):
        self.seed()
        a=e9.e8.signed_event("A",N,RECEIPT,head())
        a["event_sha256"]="0"*64
        with self.assertRaisesRegex(e9.Blocked,"EVENT_HASH"):
            e9.guard_record(self.root,a,N,RECEIPT,head())
    def test_16_same_fork_twice_rejected(self):
        self.seed()
        x=events(head())[0]
        with self.assertRaisesRegex(e9.Blocked,"FORKS_NOT_BOTH"):
            e9.guarded_reconcile(self.root,[x,x],N,RECEIPT,head())
    def test_17_known_branch_missing_rejected(self):
        self.seed()
        with self.assertRaisesRegex(e9.Blocked,"INCOMPLETE"):
            e9.guarded_reconcile(self.root,[events(head())[0]],N,RECEIPT,head())
    def test_18_disposable_missing_db_rejected_and_native_locked(self):
        with self.assertRaisesRegex(e9.Blocked,"DATABASE_MISSING"):
            e9.record(self.root,"A")
        d=e9.outcome("LAB",r6g="UNLOCK",production="YES",
                     native_admission_gates_unmet=0)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")

@unittest.skipUnless(e9.e8.e7.e6.r4.is_posix_ssh(),
                     "requires true POSIX OpenSSH Ed25519 and OS processes")
class E9TrueSignedAndOS(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix=e9.e8.LAB_PREFIX)
        self.root=Path(self.tmp.name)
        os.chmod(self.root,0o700)
        self.fixture=e9.e8.e7.e6.local_crypto_fixture(self.root,head())
        orig=e9.e8.e7.initialize(self.root)
        for fork in e9.e8.FORKS:
            shutil.copy2(orig,e9.e8.fork_path(self.root,fork))
    def tearDown(self):self.tmp.cleanup()
    def test_19_genuine_ssh_signed_receipt_verified(self):
        nonce,sha=e9.e8.e7.local_signed_context(self.root,head())
        self.assertEqual(nonce,self.fixture["nonce"])
        self.assertEqual(len(sha),64)
    def test_20_true_two_e8_workers_spend_distinct_files(self):
        rs=e9.launch_real_e8_workers(self.root,head())
        self.assertEqual(len({x["process_pid"] for x in rs}),2)
        self.assertEqual({x["fork"] for x in rs},set(e9.e8.FORKS))
    def test_21_valid_signed_context_binds_both_real_rows(self):
        rs=e9.launch_real_e8_workers(self.root,head())
        nonce,sha=e9.e8.e7.local_signed_context(self.root,head())
        d=e9.guarded_reconcile(self.root,[r["event"] for r in rs],
                               nonce,sha,head())
        self.assertTrue(d["local_split_brain_detected"])
    def test_22_recomputed_b_event_does_not_rewrite_sqlite(self):
        rs=e9.launch_real_e8_workers(self.root,head())
        nonce,sha=e9.e8.e7.local_signed_context(self.root,head())
        forged=[r["event"] for r in rs]
        fake_nonce="f"*64 if nonce!="f"*64 else "0"*64
        forged=[e9.e8.signed_event("B",fake_nonce,sha,head())
                if x["fork"]=="B" else x for x in forged]
        self.assertFalse(e9.e8.reconcile(forged)["split_brain_detected"])
        with self.assertRaisesRegex(e9.Blocked,"EVENT_PERSISTED_ROW_MISMATCH"):
            e9.guarded_reconcile(self.root,forged,nonce,sha,head())
    def test_23_full_exact_head_os_experiment(self):
        d=e9.exercise(ROOT,head())
        self.assertEqual(d["exact_source_head"],head())
        self.assertTrue(d["new_event_to_sqlite_guard_rejected"])
    def test_24_negative_counterexample_never_grants_native_rights(self):
        d=e9.exercise(ROOT,head())
        self.assertTrue(d["old_e8_split_brain_conflict_flag_false_negative"])
        self.assertTrue(d["old_e8_result_remained_block"])
        self.assertFalse(d["external_complete_history_proven"])
        self.assertEqual(d["result"],"BLOCK")
        self.assertEqual(d["native_admission_gates_unmet"],12)

if __name__=="__main__":unittest.main()
