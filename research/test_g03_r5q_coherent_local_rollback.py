"""24 actual security research tests R5Q: no local proof of global history.

16 portable tests use real R5L SQLite, real R5M witness and R5P local refusal.
8 POSIX-native tests invoke separate real OS processes to restore actual files.
All outcomes deny. These tests NEVER claim a real external root or admission.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5m_file_integrity_quarantine as witness
import g03_r5p_reopen_identity_continuity as reopen
import g03_r5q_coherent_local_rollback as lab

class PortableThreeStoreCounterexample(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-portable-")
        self.root=Path(self.tmp.name)
        if os.name!="nt":os.chmod(self.root,0o700)
        self.a,self.b,self.j=lab.fixture(self.root)
    def tearDown(self):self.tmp.cleanup()
    def check(self):
        r=lab.report_audit(self.a,self.b,self.j)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        return r
    def test_01_real_three_evidence_files_are_consistent_and_deny(self):
        self.assertEqual(self.check()["reason"],"COHERENT_LOCAL_EVIDENCE_NOT_GLOBAL_HISTORY")
        self.assertEqual(lab.projection(self.a,self.b,self.j)["primary_height"],1)
    def test_02_exact_real_db_and_local_witness_sha_are_recorded(self):
        p=lab.projection(self.a,self.b,self.j)
        self.assertEqual(p["primary_db_sha256"],
            hashlib.sha256((self.a/denial.NAME).read_bytes()).hexdigest())
        self.assertEqual(p["witness_bytes_sha256"],
            hashlib.sha256((self.b/witness.WITNESS).read_bytes()).hexdigest())
    def test_03_committed_real_local_spent_nonce_witness(self):
        p=lab.projection(self.a,self.b,self.j)
        self.assertEqual(p["local_spent_nonce_height"],1)
        self.assertEqual([x[1] for x in p["spent_nonce_records"]],["c"*64])
    def test_04_three_real_evidence_files_not_synthetic_json(self):
        db,wp,jd=lab.paths(self.a,self.b,self.j)
        self.assertTrue(db.read_bytes().startswith(denial.SQLITE_HEADER))
        self.assertEqual(jd.read_bytes()[:16],denial.SQLITE_HEADER)
        self.assertEqual(json.loads(wp.read_bytes())["height"],1)
    def test_05_primary_missing_does_not_reinitialize_sqlite(self):
        db=self.a/denial.NAME;db.unlink()
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
        self.assertFalse(db.exists())
    def test_06_witness_missing_does_not_reinitialize_json(self):
        wp=self.b/witness.WITNESS;wp.unlink()
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
        self.assertFalse(wp.exists())
    def test_07_spent_nonce_journal_missing_no_auto_genesis(self):
        jd=self.j/denial.NAME;jd.unlink()
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
        self.assertFalse(jd.exists())
    def test_08_real_primary_db_header_corrupted_denied(self):
        (self.a/denial.NAME).write_bytes(b"invalid header")
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_09_actual_local_witness_json_corrupted_denied(self):
        (self.b/witness.WITNESS).write_bytes(b"{}")
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_10_actual_local_nonce_sqlite_corrupted_denied(self):
        (self.j/denial.NAME).write_bytes(b"invalid")
        self.assertEqual(self.check()["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_11_two_real_local_histories_have_different_projections(self):
        original=lab.digest_projection(lab.projection(self.a,self.b,self.j))
        lab.advance_world(self.a,self.b,self.j)
        changed=lab.projection(self.a,self.b,self.j)
        self.assertNotEqual(original,lab.digest_projection(changed))
        self.assertEqual(changed["primary_height"],2)
        self.assertEqual(changed["local_spent_nonce_height"],2)
    def test_12_actual_locally_spent_nonce_replay_refused_before_rollback(self):
        first=lab.use_nonce(self.a,self.b,self.j,"c"*64)
        self.assertEqual(first["reason"],"REOPEN_IDENTITY_QUARANTINED")
        self.assertEqual(lab.projection(self.a,self.b,self.j)["local_spent_nonce_height"],1)
    def test_13_legacy_invariants_remain_only_block(self):
        x=reopen.prepare(self.a,self.b,"e"*64)
        r=reopen.evaluate(x,self.a,self.b,self.j)
        self.assertEqual(r["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
    def test_14_overlap_primary_witness_journal_refused(self):
        self.assertEqual(lab.report_audit(self.a,self.b,self.a)["reason"],"LOCAL_EVIDENCE_QUARANTINED")
        self.assertEqual(lab.report_audit(self.a,self.a,self.j)["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_15_real_disposable_copies_are_separate_but_same_admin(self):
        snapshot=lab.file_snapshots(self.root,self.a,self.b,self.j,"history_a")
        for source,name in zip(lab.paths(self.a,self.b,self.j),lab.PARTS):
            self.assertEqual((snapshot/name).read_bytes(),source.read_bytes())
        self.assertNotEqual(snapshot.parent,self.a)
        self.assertFalse(lab.projection(self.a,self.b,self.j)["all_local_authority_only"] is False)
    def test_16_genuine_new_os_process_audits_consistent_local_only(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--audit",
             "--primary-dir",str(self.a),"--witness-dir",str(self.b),
             "--journal-dir",str(self.j)]
        result=subprocess.run(cmd,capture_output=True,text=True,timeout=35)
        self.assertEqual(result.returncode,2,result.stderr)
        x=json.loads(result.stdout)
        self.assertEqual(x["reason"],"COHERENT_LOCAL_EVIDENCE_NOT_GLOBAL_HISTORY")
        self.assertFalse(x["externally_authenticated_history"])
        self.assertEqual(x["local_projection_sha256"],lab.digest_projection(lab.projection(self.a,self.b,self.j)))

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "real Unix/OpenSSH subprocess restore and local nonce replay witness required")
class NativeTwoHistoryRollback(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-native-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=lab.fixture(self.root)
        self.baseline=lab.projection(self.a,self.b,self.j)
        self.snap_a=lab.file_snapshots(self.root,self.a,self.b,self.j,"history_a")
        lab.advance_world(self.a,self.b,self.j)
        self.advanced=lab.projection(self.a,self.b,self.j)
        self.snap_b=lab.file_snapshots(self.root,self.a,self.b,self.j,"history_b")
    def tearDown(self):self.tmp.cleanup()
    def rollback(self,names=lab.PARTS):
        return lab.actual_second_process_restore(
            self.root,self.snap_a,self.a,self.b,self.j,names)
    def test_17_real_advanced_world_has_spent_second_nonce(self):
        self.assertEqual(self.baseline["primary_height"],1)
        self.assertEqual(self.advanced["primary_height"],2)
        self.assertEqual(self.advanced["local_spent_nonce_height"],2)
        self.assertEqual(self.advanced["spent_nonce_records"][-1][1],"d"*64)
    def test_18_primary_only_old_restore_rejected_with_newer_witness(self):
        self.rollback(("primary.sqlite3",))
        x=lab.report_audit(self.a,self.b,self.j)
        self.assertEqual(x["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_19_witness_only_restore_rejected_with_newer_primary(self):
        self.rollback(("local_witness.json",))
        x=lab.report_audit(self.a,self.b,self.j)
        self.assertEqual(x["reason"],"LOCAL_EVIDENCE_QUARANTINED")
    def test_20_restored_only_spent_nonce_register_is_locally_undetectable(self):
        self.rollback(("spent_nonces.sqlite3",))
        # The primary+checkpoint still attest height two. The distinct local
        # refusal ledger has reverted, but no independent protected highwater.
        self.assertEqual(lab.report_audit(self.a,self.b,self.j)["reason"],
                         "COHERENT_LOCAL_EVIDENCE_NOT_GLOBAL_HISTORY")
        self.assertEqual(lab.projection(self.a,self.b,self.j)["local_spent_nonce_height"],1)
    def test_21_real_three_file_second_process_restore_matches_old_projection(self):
        self.assertTrue(self.rollback()["no_independent_external_custody"])
        self.assertEqual(lab.projection(self.a,self.b,self.j),self.baseline)
        self.assertNotEqual(self.baseline,self.advanced)
    def test_22_old_advanced_ticket_rejected_but_fresh_old_ticket_only_denies(self):
        advanced=reopen.prepare(self.a,self.b,"1"*64)
        self.rollback()
        r=reopen.evaluate(advanced,self.a,self.b,self.j)
        self.assertEqual(r["reason"],"REOPEN_IDENTITY_QUARANTINED")
        new=lab.use_nonce(self.a,self.b,self.j,"2"*64)
        self.assertEqual(new["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertEqual(new["admission"],"NOT_AUTHORIZED")
    def test_23_globally_spent_nonce_lost_by_three_file_rollback(self):
        self.assertEqual(self.advanced["spent_nonce_records"][-1][1],"d"*64)
        self.rollback()
        self.assertEqual(lab.projection(self.a,self.b,self.j)["local_spent_nonce_height"],1)
        reissued=lab.use_nonce(self.a,self.b,self.j,"d"*64)
        self.assertEqual(reissued["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertEqual(reissued["local_denial_height"],2)
        self.assertEqual(reissued["admission"],"NOT_AUTHORIZED")
    def test_24_full_native_real_subprocess_two_world_exercise_always_denied(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--exercise"]
        proc=subprocess.run(cmd,text=True,capture_output=True,timeout=180)
        self.assertEqual(proc.returncode,2,proc.stderr)
        x=json.loads(proc.stdout)
        self.assertEqual(x["reason"],"REAL_TWO_HISTORIES_SAME_LOCAL_EVIDENCE_DENIED")
        self.assertTrue(x["same_application_evidence_before_after"])
        self.assertTrue(x["fresh_os_process_observed_old_local_history"])
        self.assertTrue(x["globally_spent_nonce_reissued_after_full_rollback"])
        self.assertEqual(x["primary_only_rollback_with_newer_witness"],"QUARANTINED")
        self.assertEqual(x["globally_spent_nonce_reissued_result"],"LOCAL_DENIAL_ONLY")
        self.assertFalse(x["protected_independent_high_water"])
        self.assertFalse(x["native_authority_granted"])
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
