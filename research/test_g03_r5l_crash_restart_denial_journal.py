"""R5L: 24 adversarial tests, real SQLite durability, SIGKILL and OS restart.

16 cross-platform real SQLite/canonical/replay denials. 8 POSIX native
process-kill/concurrency tests; Unix-only tests skipped, never faked, on Windows.
All reports BLOCK/NOT_AUTHORIZED, no independent custodian or production grant.
"""
import json
from contextlib import closing
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5h_signed_revocation_lab as rev
import g03_r5l_crash_restart_denial_journal as lab

CLAIM="a"*64
FIRST="b"*64
SECOND="c"*64
THIRD="d"*64

class PortableActualJournal(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-portable-")
        self.root=Path(self.temp.name)
        self.path=self.root/"journal"
        self.path.mkdir(mode=0o700)
        if os.name!="nt":
            os.chmod(self.root,0o700)
            os.chmod(self.path,0o700)
        with lab.DenialJournal(self.path) as j:
            self.assertEqual(j.audit()["local_height"],0)
    def tearDown(self):self.temp.cleanup()
    def accepted_deny(self,nonce=FIRST,role="candidate_a",claim=CLAIM):
        output=lab.execute(self.path,"record",nonce,claim,role)
        self.assertEqual(output["status"],"BLOCK")
        self.assertEqual(output["admission"],"NOT_AUTHORIZED")
        return output
    def rejected(self,reason="LOCAL_DENIAL_JOURNAL_REFUSED",**kwargs):
        output=lab.execute(self.path,"record",**kwargs)
        self.assertEqual(output["status"],"BLOCK")
        self.assertEqual(output["admission"],"NOT_AUTHORIZED")
        self.assertEqual(output["reason"],reason)
        return output
    def test_01_real_empty_journal_is_denied_not_admitted(self):
        r=lab.execute(self.path,"audit")
        self.assertEqual(r["reason"],"LOCAL_DENIAL_HISTORY_NOT_EXTERNAL")
        self.assertEqual(r["local_height"],0)
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
    def test_02_actual_sqlite_file_and_separate_directory_exist(self):
        self.assertEqual((self.path/lab.NAME).read_bytes()[:16],lab.SQLITE_HEADER)
        self.assertTrue((self.path/lab.NAME).is_file())
    def test_03_actual_durable_deny_commit_not_authority(self):
        r=self.accepted_deny()
        self.assertEqual(r["local_height"],1)
        self.assertEqual(r["reason"],"LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION")
    def test_04_repeated_same_real_nonce_refused(self):
        self.accepted_deny()
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",
                      self.rejected(nonce=FIRST,claim_sha=CLAIM,role="candidate_a")["detail"])
    def test_05_reopen_preserves_committed_denial_height(self):
        self.accepted_deny()
        with lab.DenialJournal(self.path) as j:
            self.assertEqual(j.audit()["local_height"],1)
    def test_06_wrong_nonce_and_bool_denied(self):
        self.rejected(nonce="0"*64,claim_sha=CLAIM,role="candidate_a")
        self.rejected(nonce=True,claim_sha=CLAIM,role="candidate_a")
    def test_07_wrong_claim_hash_and_bool_denied(self):
        self.rejected(nonce=FIRST,claim_sha="bad",role="candidate_a")
        self.rejected(nonce=FIRST,claim_sha=True,role="candidate_a")
    def test_08_unrecognized_role_never_allowed(self):
        for role in ("guardian","admin","APPROVE",None):
            self.rejected(nonce=FIRST,claim_sha=CLAIM,role=role)
    def test_09_existing_deleted_state_must_not_recreate_genesis(self):
        with closing(sqlite3.connect(self.path/lab.NAME)) as c, c:
            c.execute("DELETE FROM state")
        r=lab.execute(self.path,"audit")
        self.assertEqual(r["reason"],"LOCAL_DENIAL_JOURNAL_REFUSED")
        self.assertIn("LOCAL_DENIAL_STATE_MISSING",r["detail"])
    def test_10_tampered_state_height_detected(self):
        self.accepted_deny()
        with closing(sqlite3.connect(self.path/lab.NAME)) as c, c:
            c.execute("UPDATE state SET height=99 WHERE id=1")
        self.assertIn("LOCAL_DENIAL_METADATA_MISMATCH",
                      lab.execute(self.path,"audit")["detail"])
    def test_11_tampered_real_chain_head_detected(self):
        self.accepted_deny()
        with closing(sqlite3.connect(self.path/lab.NAME)) as c, c:
            c.execute("UPDATE denials SET head=? WHERE seq=1",("e"*64,))
        self.assertIn("LOCAL_DENIAL_CHAIN_INVALID",
                      lab.execute(self.path,"audit")["detail"])
    def test_12_corrupted_sqlite_header_refused(self):
        (self.path/lab.NAME).write_bytes(b"not an SQLite file")
        r=lab.execute(self.path,"audit")
        self.assertEqual(r["reason"],"LOCAL_DENIAL_JOURNAL_REFUSED")
    def test_13_symlink_database_refused_when_supported(self):
        current=self.path/lab.NAME
        current.unlink()
        outside=self.root/"outside"
        outside.write_bytes(b"placeholder")
        try:current.symlink_to(outside)
        except (OSError,NotImplementedError):
            self.assertEqual(lab.execute(self.path,"audit")["status"],"BLOCK")
        else:
            self.assertIn("JOURNAL_FILE_SYMLINK",
                          lab.execute(self.path,"audit")["detail"])
    def test_14_nonprivate_directory_refused_on_posix(self):
        if os.name!="nt":
            os.chmod(self.path,0o755)
            self.assertIn("JOURNAL_DIRECTORY_PERMISSIONS",
                          lab.execute(self.path,"audit")["detail"])
        else:
            self.assertEqual(lab.execute(self.path,"audit")["admission"],"NOT_AUTHORIZED")
    def test_15_repeated_audit_does_not_mutate_actual_database(self):
        before=(self.path/lab.NAME).read_bytes()
        for _ in range(2):
            self.assertEqual(lab.execute(self.path,"audit")["local_height"],0)
        self.assertEqual(before,(self.path/lab.NAME).read_bytes())
    def test_16_real_portable_subprocess_audit_never_admits(self):
        proc=subprocess.run(lab._process("audit",self.path),capture_output=True,
                            text=True,timeout=30,check=False)
        self.assertEqual(proc.returncode,2)
        result=json.loads(proc.stdout)
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertEqual(result["local_height"],0)

@unittest.skipUnless(os.name!="nt" and Path(rev.SSH).is_file(),
                      "native POSIX process SIGKILL and real OpenSSH research control")
class NativeKillAndRestart(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-posix-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.dir=self.root/"journal";self.dir.mkdir(mode=0o700)
        os.chmod(self.dir,0o700)
        with lab.DenialJournal(self.dir) as j:self.assertEqual(j.audit()["local_height"],0)
    def tearDown(self):self.tmp.cleanup()
    def subprocess_deny(self,nonce,role="candidate_a"):
        return lab._run("record",self.dir,nonce,CLAIM,role)
    def test_17_real_two_ed25519_signers_and_native_crash_lab_always_block(self):
        report=lab.native_exercise()
        self.assertEqual(report["reason"],"REAL_PROCESS_KILL_AND_RESTART_RECOVERY_DENIED")
        self.assertEqual(report["real_temp_signers"],2)
        self.assertFalse(report["native_permission_granted"])
    def test_18_real_child_commits_deny_then_new_child_sees_durable_row(self):
        self.assertEqual(self.subprocess_deny(FIRST)["local_height"],1)
        self.assertEqual(lab._run("audit",self.dir)["local_height"],1)
    def test_19_actual_sigkill_uncommitted_txn_rolled_back(self):
        self.assertTrue(lab.actual_kill(self.dir,SECOND,CLAIM,"candidate_b"))
        self.assertEqual(lab._run("audit",self.dir)["local_height"],0)
    def test_20_real_durable_deny_preserved_after_second_process_sigkill(self):
        self.assertEqual(self.subprocess_deny(FIRST)["local_height"],1)
        self.assertTrue(lab.actual_kill(self.dir,SECOND,CLAIM,"candidate_b"))
        self.assertEqual(lab._run("audit",self.dir)["local_height"],1)
        self.assertEqual(self.subprocess_deny(SECOND,"candidate_b")["local_height"],2)
    def test_21_two_actual_parallel_children_same_nonce_at_most_one_local_commit(self):
        cmds=[lab._process("record",self.dir,FIRST,CLAIM,role)
              for role in lab.ROLES]
        children=[subprocess.Popen(cmd,stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE,text=True) for cmd in cmds]
        outputs=[]
        for p in children:
            stdout,stderr=p.communicate(timeout=30)
            self.assertEqual(p.returncode,2,stderr)
            outputs.append(json.loads(stdout))
        self.assertEqual(sorted(x["reason"] for x in outputs),[
            "LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION","LOCAL_DENIAL_JOURNAL_REFUSED"])
        self.assertEqual(lab._run("audit",self.dir)["local_height"],1)
    def test_22_two_parallel_children_distinct_nonces_both_write_only_denials(self):
        args=[("candidate_a",FIRST),("candidate_b",SECOND)]
        children=[subprocess.Popen(lab._process("record",self.dir,nonce,CLAIM,role),
                                   stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
                  for role,nonce in args]
        for p in children:
            stdout,stderr=p.communicate(timeout=30)
            self.assertEqual(p.returncode,2,stderr)
            result=json.loads(stdout)
            self.assertEqual(result["admission"],"NOT_AUTHORIZED")
            self.assertEqual(result["reason"],"LOCAL_DENIAL_DURABLY_COMMITTED_NOT_ADMISSION")
        self.assertEqual(lab._run("audit",self.dir)["local_height"],2)
    def test_23_real_same_admin_old_sqlite_snapshot_rolls_back_only_local_denial(self):
        old=self.root/"old-journal.sqlite3"
        shutil.copy2(self.dir/lab.NAME,old)
        self.assertEqual(self.subprocess_deny(FIRST)["local_height"],1)
        shutil.copy2(old,self.dir/lab.NAME)
        r=lab._run("audit",self.dir)
        self.assertEqual(r["local_height"],0)
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertEqual(r["global_monotonic_antirollback"],"NOT_IMPLEMENTED")
    def test_24_real_end_to_end_native_cli_sigkill_restart_concurrency_exit_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=120,check=False)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"REAL_PROCESS_KILL_AND_RESTART_RECOVERY_DENIED")
        self.assertEqual(x["durable_precrash_denial"],"PRESERVED")
        self.assertEqual(x["killed_uncommitted_denial"],"ROLLED_BACK")
        self.assertEqual(x["concurrent_same_nonce"],"ONE_LOCAL_DENY_COMMIT_ONE_REPLAY_BLOCK")
        self.assertEqual(x["final_local_denial_height"],3)
        self.assertFalse(x["native_permission_granted"])

if __name__=="__main__":unittest.main()
