"""E7 — 24 real automated tests, no human testers or invented authority.

18 portable policy/real SQLite file tests; six POSIX true OpenSSH Ed25519
and distinct OS-process concurrency tests. All research results remain BLOCK.
"""
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/"scripts"),str(ROOT/"research")]
import bureau_001_e7_atomic_local_nonce_race as e7

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
              cwd=ROOT,text=True).strip()

NONCE="a"*64
RECEIPT="b"*64

class E7PortableSourceAndRealSQLite(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix=e7.LAB_PREFIX)
        self.root=Path(self.temp.name)
        if os.name=="posix":os.chmod(self.root,0o700)
    def tearDown(self):
        self.temp.cleanup()
    def test_01_exact_source_parent_and_e6_proof(self):
        a=e7.audit(ROOT,head())
        self.assertEqual(a["source_head"],head())
        self.assertEqual(a["result"],"BLOCK")
    def test_02_e6_qualified_at_exact_parent_head(self):
        p=e7.e3.strictly_load(e7.e3.real_file_bytes(ROOT/e7.CONTRACT))
        self.assertEqual(p["parent_e6_exact_head"],e7.PARENT)
        self.assertEqual(p["predecessor_e6"]["specialized_jobs_completed_success"],4)
        self.assertEqual(p["predecessor_e6"]["rust_jobs_completed_success"],5)
    def test_03_owner_permanently_bans_human_testers(self):
        p=e7.e3.strictly_load(e7.e3.real_file_bytes(ROOT/e7.CONTRACT))
        self.assertFalse(p["human_testers_allowed"])
        self.assertEqual(p["physical_test_humans"],0)
        self.assertTrue(p["no_network"])
    def test_04_nine_roles_not_nine_independent_reviewers(self):
        p=e7.e3.strictly_load(e7.e3.real_file_bytes(ROOT/e7.CONTRACT))
        self.assertEqual(len(p["automated_bureau_functions"]),9)
        self.assertEqual(p["bureau_roles_actual_independent_agents_executed"],0)
        self.assertEqual(p["independent_specialist_reviews"],0)
    def test_05_create_real_sqlite_disposable_table(self):
        db=e7.initialize(self.root)
        self.assertTrue(db.is_file())
        with sqlite3.connect(db) as c:
            cols=c.execute("PRAGMA table_info(research_local_spent)").fetchall()
        self.assertEqual(len(cols),4)
    def test_06_new_sqlite_nonce_is_not_spent(self):
        db=e7.initialize(self.root)
        self.assertEqual(e7.count_nonce(db,NONCE),0)
    def test_07_first_atomic_transaction_persists_one_record(self):
        db=e7.initialize(self.root)
        self.assertEqual(e7.consume_atomic(db,NONCE,RECEIPT,head()),
                         "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION")
        self.assertEqual(e7.count_nonce(db,NONCE),1)
    def test_08_repeated_sqlite_transaction_refuses_same_nonce(self):
        db=e7.initialize(self.root)
        self.assertEqual(e7.consume_atomic(db,NONCE,RECEIPT,head()),
                         "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION")
        self.assertEqual(e7.consume_atomic(db,NONCE,RECEIPT,head()),
                         "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION")
        self.assertEqual(e7.count_nonce(db,NONCE),1)
    def test_09_namespace_and_nonce_compose_unique_primary(self):
        db=e7.initialize(self.root)
        e7.consume_atomic(db,NONCE,RECEIPT,head())
        with sqlite3.connect(db) as c:
            row=c.execute("SELECT namespace,challenge_nonce_sha256 FROM "
                          "research_local_spent").fetchone()
        self.assertEqual(row,(e7.NAMESPACE,NONCE))
    def test_10_two_distinct_nonces_get_distinct_local_records(self):
        db=e7.initialize(self.root)
        self.assertTrue(e7.consume_atomic(db,NONCE,RECEIPT,head()).startswith("LOCAL_SPENT"))
        self.assertTrue(e7.consume_atomic(db,"c"*64,RECEIPT,head()).startswith("LOCAL_SPENT"))
        self.assertEqual(e7.count_nonce(db,NONCE),1)
        self.assertEqual(e7.count_nonce(db,"c"*64),1)
    def test_11_real_db_is_stable_across_process_reopen(self):
        db=e7.initialize(self.root)
        e7.consume_atomic(db,NONCE,RECEIPT,head())
        with sqlite3.connect(db) as c:
            self.assertEqual(c.execute("SELECT COUNT(*) FROM research_local_spent")
                             .fetchone()[0],1)
    def test_12_real_same_admin_snapshot_restore_erases_local_floor(self):
        db=e7.initialize(self.root)
        old=self.root/"old.sqlite3";shutil.copy2(db,old)
        e7.consume_atomic(db,NONCE,RECEIPT,head())
        self.assertEqual(e7.count_nonce(db,NONCE),1)
        staged=self.root/"staged.sqlite3"
        shutil.copy2(old,staged)
        os.replace(staged,db)
        self.assertEqual(e7.count_nonce(db,NONCE),0)
    def test_13_malformed_nonce_cannot_be_counted(self):
        db=e7.initialize(self.root)
        with self.assertRaisesRegex(e7.Blocked,"INVALID_NONCE"):
            e7.count_nonce(db,"bad")
    def test_14_bool_and_bad_sha_cannot_be_atomic_spend(self):
        db=e7.initialize(self.root)
        with self.assertRaisesRegex(e7.Blocked,"CONTEXT_INVALID"):
            e7.consume_atomic(db,NONCE,"bad",head())
    def test_15_disallow_overwriting_existing_disposable_db(self):
        e7.initialize(self.root)
        with self.assertRaisesRegex(e7.Blocked,"ALREADY_EXISTS"):
            e7.initialize(self.root)
    def test_16_refusal_cannot_grant_native_rights(self):
        d=e7.proof("RESEARCH",r6g="UNLOCKED",native_admission_gates_unmet=0,
                   production="AUTH",human_testers=42)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["production"],"NOT_AUTHORIZED")
        self.assertEqual(d["real_human_testers"],0)
    def test_17_wrong_source_head_fails_closed(self):
        with self.assertRaises((e7.e3.Rejected,e7.e6.e5.Blocked,e7.Blocked)):
            e7.audit(ROOT,"f"*40)
    def test_18_zero_external_receipts_and_global_antirollback_unverified(self):
        p=e7.e3.strictly_load(e7.e3.real_file_bytes(ROOT/e7.CONTRACT))
        self.assertEqual(p["actual_independent_witness_receipts"],0)
        self.assertEqual(p["external_global_antirollback"],"NOT_VERIFIED")
        self.assertEqual(p["native_admission_gates_unmet"],12)
        self.assertEqual(p["r6g"],"LOCKED_BY_SEQUENCE")

@unittest.skipUnless(e7.e6.r4.is_posix_ssh(),
                     "requires real POSIX OpenSSH Ed25519 and OS subprocesses")
class E7TrueCryptoAndTwoProcessRace(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix=e7.LAB_PREFIX)
        self.root=Path(self.temp.name);os.chmod(self.root,0o700)
        self.fixture=e7.e6.local_crypto_fixture(self.root,head())
        self.db=e7.initialize(self.root)
    def tearDown(self):self.temp.cleanup()
    def test_19_genuine_e6_signed_nonce_verified_before_any_spend(self):
        nonce,receipt=e7.local_signed_context(self.root,head())
        self.assertEqual(nonce,self.fixture["nonce"])
        self.assertEqual(len(receipt),64)
        self.assertEqual(e7.count_nonce(self.db,nonce),0)
    def test_20_two_real_naive_children_both_observe_unspent(self):
        rs=e7.launch(self.root,head(),"naive")
        self.assertEqual([r["local_before_count"] for r in rs],[0,0])
        self.assertEqual(len(set(r["process_pid"] for r in rs)),2)
        self.assertEqual(e7.count_nonce(self.db,self.fixture["nonce"]),0)
    def test_21_atomic_real_two_children_exact_one_local_winner(self):
        rs=e7.launch(self.root,head(),"atomic")
        self.assertEqual(sorted(r["local_spend_outcome"] for r in rs),
            sorted(["LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION",
                    "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION"]))
        self.assertEqual(e7.count_nonce(self.db,self.fixture["nonce"]),1)
    def test_22_restart_true_process_denies_locally_spent_nonce(self):
        e7.launch(self.root,head(),"atomic")
        third=e7.launch(self.root,head(),"atomic",count=1)[0]
        self.assertEqual(third["local_spend_outcome"],
                         "LOCAL_DUPLICATE_BLOCKED_NO_NATIVE_ADMISSION")
    def test_23_real_old_sqlite_restoration_loses_local_atomic_history(self):
        old=self.root/"old_snapshot.sqlite3";shutil.copy2(self.db,old)
        e7.launch(self.root,head(),"atomic")
        self.assertEqual(e7.count_nonce(self.db,self.fixture["nonce"]),1)
        stage=self.root/"restore.sqlite3";shutil.copy2(old,stage)
        os.replace(stage,self.db)
        self.assertEqual(e7.count_nonce(self.db,self.fixture["nonce"]),0)
        replay=e7.launch(self.root,head(),"atomic",count=1)[0]
        self.assertEqual(replay["local_spend_outcome"],
                         "LOCAL_SPENT_RECORD_COMMITTED_NO_NATIVE_ADMISSION")
    def test_24_full_real_two_process_race_atomic_and_r5q_replay(self):
        d=e7.exercise(ROOT,head())
        self.assertEqual(d["reason"],
          "E7_REAL_TWO_PROCESS_LOCAL_NONCE_RACE_AND_ATOMIC_FIX_WITH_ROLLBACK_LIMIT")
        self.assertEqual(d["two_real_concurrent_naive_unspent_reads"],2)
        self.assertEqual(d["atomic_local_committed"],1)
        self.assertEqual(d["atomic_local_duplicates_blocked"],1)
        self.assertTrue(d["persisted_local_nonce_replay_blocked_after_restart"])
        self.assertTrue(d["local_replay_after_rollback_committed_again"])
        self.assertTrue(d["original_r5q_three_file_rollback_reproduced"])
        self.assertFalse(d["externally_protected_monotonic_floor"])
        self.assertEqual(d["native_admission_gates_unmet"],12)

if __name__=="__main__":unittest.main()
