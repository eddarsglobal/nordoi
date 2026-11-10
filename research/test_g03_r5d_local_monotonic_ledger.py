"""G0.3 R5D: 24 actual SQLite durability/replay and negative authority tests.

The database is REAL local storage, deliberately roll-backable by its controller.
All records remain UNTRUSTED for production, even when locally durable.
"""
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
import g03_r5d_local_monotonic_ledger as lab

A="a"*64
B="b"*64
C="c"*64
D="d"*64
E="e"*64

class SQLiteDurabilityControls(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5d-real-sqlite-")
        self.root=Path(self.temp.name)
        os.chmod(self.root,0o700)
    def tearDown(self):
        self.temp.cleanup()
    def blocked(self, data):
        self.assertEqual(data["admission"],"NOT_AUTHORIZED")
        self.assertEqual(data["global_antirollback"],"NOT_IMPLEMENTED")
        return data
    def append(self,nonce=A,payload=B,epoch=4,prev=lab.GENESIS):
        return self.blocked(lab.execute(self.root,"append",nonce=nonce,
                           payload=payload,epoch=epoch,expected_prev=prev))
    def inspect(self):
        return self.blocked(lab.execute(self.root,"inspect"))
    def sql_mutate(self, sql, params=()):
        """Connection context alone does not close SQLite on Windows."""
        conn = sqlite3.connect(self.root/lab.DATABASE)
        try:
            with conn:
                conn.execute(sql, params)
        finally:
            conn.close()
    def test_01_initial_real_sqlite_genesis_is_not_external_highwater(self):
        x=self.inspect()
        self.assertEqual(x["local_height"],0)
        self.assertEqual(x["local_head_sha256"],lab.GENESIS)
        self.assertEqual(x["status"],"BLOCK")
        self.assertEqual(x["external_highwater"],"NOT_ENROLLED")
    def test_02_real_sqlite_full_sync_append_no_admission(self):
        x=self.append()
        self.assertEqual(x["status"],"LAB_LOCAL_COMMIT")
        self.assertEqual(x["local_height"],1)
        self.assertNotEqual(x["local_head_sha256"],lab.GENESIS)
    def test_03_reopen_after_close_retains_chain_and_nonce(self):
        x=self.append()
        y=self.inspect()
        self.assertEqual(y["local_head_sha256"],x["local_head_sha256"])
        self.assertEqual(y["local_height"],1)
    def test_04_exact_replay_same_nonce_rejected_even_after_restart(self):
        x=self.append()
        y=self.append(prev=x["local_head_sha256"])
        self.assertEqual(y["status"],"BLOCK")
        self.assertIn("CHALLENGE_ALREADY_CONSUMED",y["detail"])
    def test_05_nonce_replay_with_changed_payload_still_rejected(self):
        x=self.append()
        y=self.append(payload=C,prev=x["local_head_sha256"])
        self.assertEqual(y["status"],"BLOCK")
        self.assertEqual(self.inspect()["local_height"],1)
    def test_06_stale_previous_head_is_a_fork_not_a_commit(self):
        self.append()
        y=self.append(nonce=C)
        self.assertIn("STALE_OR_FORKED",y["detail"])
    def test_07_contiguous_second_append_changes_head(self):
        first=self.append()
        second=self.append(nonce=C,prev=first["local_head_sha256"])
        self.assertEqual(second["status"],"LAB_LOCAL_COMMIT")
        self.assertEqual(second["local_height"],2)
        self.assertNotEqual(second["local_head_sha256"],first["local_head_sha256"])
    def test_08_epoch_downgrade_refused(self):
        first=self.append(epoch=6)
        y=self.append(nonce=C,epoch=5,prev=first["local_head_sha256"])
        self.assertIn("EPOCH_ROLLBACK",y["detail"])
    def test_09_real_epoch_advance_persisted_across_restart(self):
        first=self.append(epoch=5)
        self.append(nonce=C,epoch=8,prev=first["local_head_sha256"])
        self.assertEqual(self.inspect()["local_epoch"],8)
    def test_10_invalid_epoch_types_fail_closed(self):
        for epoch in (True,3,-1,4.0,None):
            self.assertEqual(self.append(epoch=epoch)["status"],"BLOCK")
    def test_11_invalid_nonce_types_and_values_rejected(self):
        for nonce in (None,True,0,"a"*63,"A"*64,"0"*64):
            self.assertEqual(self.append(nonce=nonce)["status"],"BLOCK")
    def test_12_invalid_payload_and_previous_head_rejected(self):
        self.assertEqual(self.append(payload="0"*64)["status"],"BLOCK")
        self.assertEqual(self.append(prev="bad")["status"],"BLOCK")
    def test_13_mutated_actual_sqlite_chain_detected_on_reopen(self):
        self.append()
        self.sql_mutate("UPDATE events SET payload_sha=? WHERE seq=1",(E,))
        y=self.inspect()
        self.assertIn("LOCAL_LEDGER_CHAIN_CORRUPTED",y["detail"])
    def test_14_mutated_actual_sqlite_metadata_detected(self):
        self.append()
        self.sql_mutate("UPDATE metadata SET height=88")
        y=self.inspect()
        self.assertIn("METADATA_DOES_NOT_MATCH_CHAIN",y["detail"])
    def test_15_deleted_local_history_refused_against_metadata(self):
        self.append()
        self.sql_mutate("DELETE FROM events WHERE seq=1")
        y=self.inspect()
        self.assertIn("METADATA_DOES_NOT_MATCH_CHAIN",y["detail"])
    def test_16_added_unlinked_sqlite_event_refused(self):
        self.append()
        self.sql_mutate("INSERT INTO events(seq,epoch,nonce,payload_sha,prev,head) VALUES(2,4,?,?,?,?)",(C,D,lab.GENESIS,E))
        y=self.inspect()
        self.assertIn("LOCAL_LEDGER_CHAIN_CORRUPTED",y["detail"])
    def test_17_real_db_snapshot_rollback_is_NOT_detectable_without_external_anchor(self):
        first=self.append()
        saved=self.root/"owner-can-rollback.sqlite3"
        shutil.copy2(self.root/lab.DATABASE,saved)
        second=self.append(nonce=C,prev=first["local_head_sha256"])
        self.assertEqual(second["local_height"],2)
        shutil.copy2(saved,self.root/lab.DATABASE)
        outcome=self.inspect()
        self.assertEqual(outcome["local_height"],1)
        self.assertEqual(outcome["local_head_sha256"],first["local_head_sha256"])
        self.assertEqual(outcome["global_antirollback"],"NOT_IMPLEMENTED")
        self.assertEqual(outcome["status"],"BLOCK")
    def test_18_two_actual_processes_competing_nonce_exactly_one_local_commit(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--lab-directory",str(self.root),
             "--operation","append","--nonce",A,"--payload-sha256",B,
             "--epoch","4","--expected-prev",lab.GENESIS]
        one=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        two=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        out1,err1=one.communicate(timeout=30)
        out2,err2=two.communicate(timeout=30)
        self.assertEqual((one.returncode,two.returncode),(2,2))
        self.assertFalse(err1 or err2)
        results=[json.loads(out1),json.loads(out2)]
        self.assertEqual(sum(x["status"]=="LAB_LOCAL_COMMIT" for x in results),1)
        self.assertEqual(self.inspect()["local_height"],1)
    def test_19_local_database_symlink_refused(self):
        target=self.root/"otherdb"
        target.write_bytes(b"untrusted")
        try:
            (self.root/lab.DATABASE).symlink_to(target)
        except OSError:
            self.skipTest("symlink unavailable")
        self.assertEqual(self.inspect()["status"],"BLOCK")
    def test_20_local_database_fifo_refused_without_hanging(self):
        if not hasattr(os,"mkfifo"):
            self.skipTest("fifo not available")
        try:
            os.mkfifo(self.root/lab.DATABASE)
        except OSError:
            self.skipTest("fifo permissions")
        self.assertEqual(self.inspect()["status"],"BLOCK")
    @unittest.skipIf(os.name == "nt", "Windows file mode is not POSIX")
    def test_21_permission_hygiene_real_sqlite_private(self):
        self.append()
        self.assertEqual((self.root/lab.DATABASE).stat().st_mode & 0o777,0o600)
        self.assertEqual(self.root.stat().st_mode & 0o777,0o700)
    @unittest.skipIf(os.name == "nt", "Windows file mode is not POSIX")
    def test_22_world_readable_directory_does_not_qualify_local_hygiene(self):
        os.chmod(self.root,0o755)
        self.assertEqual(self.inspect()["status"],"BLOCK")
    def test_23_cli_inspect_exists_and_always_returns_block_exit_two(self):
        command=[sys.executable,"-B",str(Path(lab.__file__)),
                 "--lab-directory",str(self.root),"--operation","inspect"]
        result=subprocess.run(command,capture_output=True,text=True,timeout=25)
        self.assertEqual(result.returncode,2)
        data=json.loads(result.stdout)
        self.assertEqual(data["status"],"BLOCK")
        self.assertEqual(data["admission"],"NOT_AUTHORIZED")
    def test_24_real_local_commit_still_returns_cli_exit_two_not_authorization(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--lab-directory",str(self.root),
             "--operation","append","--nonce",A,"--payload-sha256",B,
             "--epoch","4","--expected-prev",lab.GENESIS]
        result=subprocess.run(cmd,capture_output=True,text=True,timeout=25)
        self.assertEqual(result.returncode,2)
        x=json.loads(result.stdout)
        self.assertEqual(x["status"],"LAB_LOCAL_COMMIT")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
        self.assertEqual(self.inspect()["local_height"],1)

if __name__=="__main__":
    unittest.main()
