"""R5O: 24 genuine open-FD/pathname identity denial tests.

16 portable actual R5L/R5M/R5N real-files and 8 POSIX tests, including
separate OS process real os.replace of byte-identical paths while FD is held.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5m_file_integrity_quarantine as witness
import g03_r5o_fd_identity_quarantine as lab

class PortableIdentity(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5o-portable-")
        self.root=Path(self.tmp.name)
        self.a,self.b,self.j=(self.root/"primary",self.root/"witness",self.root/"journal")
        for directory in (self.a,self.b,self.j):
            directory.mkdir(mode=0o700)
            if os.name!="nt":os.chmod(directory,0o700)
        if os.name!="nt":os.chmod(self.root,0o700)
        with denial.DenialJournal(self.a) as x:
            self.assertEqual(x.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(witness.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.j) as x:self.assertEqual(x.audit()["local_height"],0)
        self.token=lab.prepare(self.a,self.b,"c"*64)
    def tearDown(self):self.tmp.cleanup()
    def check(self,raw=None):
        out=lab.evaluate(self.token if raw is None else raw,self.a,self.b,self.j)
        self.assertEqual(out["status"],"BLOCK")
        self.assertEqual(out["admission"],"NOT_AUTHORIZED")
        self.assertFalse(out["native_authority_granted"])
        return out
    def changed(self,**kwargs):
        raw=json.loads(self.token);raw.update(kwargs)
        return lab.canonical(raw)
    def reject(self,raw):
        with self.assertRaises(lab.Refused):lab.parse(raw)
    def test_01_existing_fd_and_path_match_never_admit(self):
        self.assertEqual(self.check()["reason"],"OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED")
    def test_02_pinned_real_primary_and_witness_inode(self):
        x=lab.parse(self.token)
        self.assertEqual(x["db_ino"],os.lstat(self.a/denial.NAME).st_ino)
        self.assertEqual(x["witness_ino"],os.lstat(self.b/witness.WITNESS).st_ino)
    def test_03_replayed_nonce_records_no_second_denial(self):
        self.check()
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",self.check()["detail"])
        self.assertEqual(denial.execute(self.j,"audit")["local_height"],1)
    def test_04_distinct_valid_nonce_records_only_local_denial(self):
        self.check()
        other=lab.prepare(self.a,self.b,"d"*64)
        self.assertEqual(self.check(other)["local_denial_height"],2)
    def test_05_missing_primary_refuses_without_recreation(self):
        (self.a/denial.NAME).unlink()
        self.assertEqual(self.check()["reason"],"FILE_IDENTITY_CHANGED_OR_UNVERIFIED_QUARANTINE")
        self.assertFalse((self.a/denial.NAME).exists())
    def test_06_missing_witness_refuses_without_recreation(self):
        (self.b/witness.WITNESS).unlink()
        self.assertEqual(self.check()["reason"],"FILE_IDENTITY_CHANGED_OR_UNVERIFIED_QUARANTINE")
        self.assertFalse((self.b/witness.WITNESS).exists())
    def test_07_missing_denial_journal_never_initialized(self):
        (self.j/denial.NAME).unlink()
        self.assertIn("DENIAL_JOURNAL_MISSING_NO_CREATE",self.check()["detail"])
    def test_08_wrong_input_types_and_oversize_rejected(self):
        self.reject("not bytes")
        self.reject(b"x"*(lab.LIMIT+1))
    def test_09_duplicate_and_noncanonical_json_rejected(self):
        self.reject(self.token.replace(b'"project":',b'"project":"bad","project":'))
        self.reject(json.dumps(json.loads(self.token),indent=2).encode())
    def test_10_wrong_protocol_project_parent_rejected(self):
        for key,val in (("protocol","bad"),("project","other"),("parent","0"*40)):
            self.reject(self.changed(**{key:val}))
    def test_11_allow_unrevoke_or_recover_operation_rejected(self):
        for action in ("ALLOW","UNREVOKE","RECOVER","APPROVE"):
            self.reject(self.changed(operation=action))
    def test_12_digest_nonce_and_hash_substitution_rejected(self):
        for key in ("nonce","head","db_sha256","witness_sha256"):
            self.reject(self.changed(**{key:"0"*64}))
    def test_13_bool_and_zero_height_rejected(self):
        self.reject(self.changed(height=True))
        self.reject(self.changed(height=0))
    def test_14_invalid_bool_and_negative_fd_inode_rejected(self):
        for key in ("db_dev","db_ino","witness_dev","witness_ino"):
            self.reject(self.changed(**{key:True}))
            self.reject(self.changed(**{key:-1}))
    def test_15_changed_inum_claim_never_admits(self):
        old=lab.parse(self.token)
        forged=self.changed(db_ino=old["db_ino"]+1)
        self.assertIn("EVIDENCE_USE_CHANGED_DB_INO",self.check(forged)["detail"])
    def test_16_real_child_process_assessment_exit_two(self):
        path=self.root/"intent.json";path.write_bytes(self.token)
        command=[sys.executable,"-B",str(Path(lab.__file__)),"--assess",
                 "--intent-file",str(path),"--primary-dir",str(self.a),
                 "--witness-dir",str(self.b),"--journal-dir",str(self.j)]
        p=subprocess.run(command,capture_output=True,text=True,timeout=25)
        self.assertEqual(p.returncode,2,p.stderr)
        self.assertEqual(json.loads(p.stdout)["admission"],"NOT_AUTHORIZED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "POSIX real old-descriptor and atomic os.replace experiment")
class NativeIdentityReplacement(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5o-native-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=(self.root/"primary",self.root/"witness",self.root/"journal")
        for d in (self.a,self.b,self.j):d.mkdir(mode=0o700);os.chmod(d,0o700)
        with denial.DenialJournal(self.a) as x:
            self.assertEqual(x.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(witness.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.j) as x:self.assertEqual(x.audit()["local_height"],0)
        self.db,self.proof=self.a/denial.NAME,self.b/witness.WITNESS
        self.old_db,self.old_proof=self.root/"same.sqlite3",self.root/"same.json"
        shutil.copy2(self.db,self.old_db);shutil.copy2(self.proof,self.old_proof)
        self.token=lab.prepare(self.a,self.b,"c"*64)
    def tearDown(self):self.tmp.cleanup()
    def check(self,token=None,**kwargs):
        r=lab.evaluate(self.token if token is None else token,
                       self.a,self.b,self.j,**kwargs)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def test_17_original_open_descriptor_matches_but_denies(self):
        self.assertEqual(self.check()["reason"],"OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED")
    def test_18_second_process_atomically_replaces_identical_primary_after_open(self):
        r=self.check(after_open=lambda:lab.real_atomic_replace(self.old_db,self.db))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
    def test_19_second_process_atomically_replaces_identical_witness_after_open(self):
        r=self.check(after_open=lambda:lab.real_atomic_replace(self.old_proof,self.proof))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
    def test_20_replacement_same_bytes_witness_just_before_local_deny(self):
        r=self.check(before_deny=lambda:lab.real_atomic_replace(self.old_proof,self.proof))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
        self.assertEqual(denial._run("audit",self.j)["local_height"],0)
    def test_21_replacement_same_bytes_primary_just_before_local_deny(self):
        r=self.check(before_deny=lambda:lab.real_atomic_replace(self.old_db,self.db))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
    def test_22_real_old_open_fd_survives_new_path_inode(self):
        with lab.OpenSnapshot(self.db) as opened:
            oldino=opened.before[1];oldsha=opened.digest
            lab.real_atomic_replace(self.old_db,self.db)
            self.assertNotEqual(os.lstat(self.db).st_ino,oldino)
            self.assertEqual(opened.digest,oldsha)
            with self.assertRaises(lab.Refused) as ctx:opened.check()
            self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",str(ctx.exception))
    def test_23_both_valid_matching_same_admin_files_replaced_new_inode(self):
        lab.real_atomic_replace(self.old_db,self.db)
        lab.real_atomic_replace(self.old_proof,self.proof)
        refreshed=lab.prepare(self.a,self.b,"d"*64)
        r=self.check(refreshed)
        self.assertEqual(r["reason"],"OPEN_FD_IDENTITY_MATCHED_ONLY_DENIAL_COMMITTED")
        self.assertEqual(r["global_monotonic_antirollback"],"NOT_IMPLEMENTED")
    def test_24_real_native_cli_identity_exercise_returns_exit_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         capture_output=True,text=True,timeout=120)
        self.assertEqual(p.returncode,2,p.stderr)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],"REAL_ATOMIC_FILE_IDENTITY_SWAPS_DENIED")
        self.assertTrue(result["old_open_fd_survived_atomic_path_change"])
        self.assertEqual(result["same_bytes_new_inode_primary"],"QUARANTINED")
        self.assertFalse(result["native_authority_granted"])

if __name__=="__main__":unittest.main()
