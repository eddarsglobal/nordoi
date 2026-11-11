"""R5I: 24 actual SQLite revocation floor negative controls; no guardian enrollment."""
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
import g03_r5h_signed_revocation_lab as signed
import g03_r5i_revocation_floor as floor

class PortableRevocationFloor(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5i-portable-")
        self.root=Path(self.temp.name)
        self.primary=self.root/"primary"
        self.secondary=self.root/"floor"
        self.primary.mkdir(mode=0o700)
        self.secondary.mkdir(mode=0o700)
        os.chmod(self.primary,0o700)
        os.chmod(self.secondary,0o700)
        with signed.RevocationLedger(self.primary) as x:
            self.assertFalse(x.audit()["local_revoked"])
        self.assertEqual(floor.execute(self.primary,self.secondary,"audit")["reason"],
                         "LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT")
    def tearDown(self):self.temp.cleanup()
    def audit(self):
        r=floor.execute(self.primary,self.secondary,"audit")
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def refused(self,a=None,b=None,action="audit"):
        r=floor.execute(self.primary if a is None else a,
                        self.secondary if b is None else b,action)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["reason"],"LOCAL_FLOOR_OPERATION_REFUSED")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def test_01_real_two_sqlite_genesis_no_admission(self):
        x=self.audit()
        self.assertEqual(x["floor_height"],0)
        self.assertEqual(x["primary_height"],0)
        self.assertEqual(x["independent_guardian"],"NOT_ENROLLED")
    def test_02_two_distinct_actual_on_disk_sqlite_files(self):
        a=self.primary/signed.DB_NAME
        b=self.secondary/floor.NAME
        self.assertEqual(a.read_bytes()[:16],b"SQLite format 3\x00")
        self.assertEqual(b.read_bytes()[:16],b"SQLite format 3\x00")
        self.assertNotEqual(a.resolve(),b.resolve())
    def test_03_genesis_has_no_revocation_to_checkpoint(self):
        x=self.refused(action="checkpoint")
        self.assertIn("NO_NEXT_SIGNED_REVOCATION",x["detail"])
    def test_04_actual_cli_audit_exits_two_even_at_genesis(self):
        r=subprocess.run([sys.executable,"-B",str(Path(floor.__file__)),
             "--primary-directory",str(self.primary),"--floor-directory",str(self.secondary),
             "--action","audit"],capture_output=True,text=True,timeout=20)
        self.assertEqual(r.returncode,2)
        self.assertEqual(json.loads(r.stdout)["admission"],"NOT_AUTHORIZED")
    def test_05_same_directory_refused(self):
        self.assertIn("STORE_ROOT_OVERLAP",self.refused(b=self.primary)["detail"])
    def test_06_nested_directory_refused(self):
        sub=self.primary/"nested"
        sub.mkdir(mode=0o700)
        os.chmod(sub,0o700)
        self.assertIn("STORE_ROOT_OVERLAP",self.refused(b=sub)["detail"])
    def test_07_nonexistent_primary_directory_refused(self):
        self.refused(a=self.root/"missing-primary")
    def test_08_nonexistent_floor_directory_refused(self):
        self.refused(b=self.root/"missing-floor")
    def test_09_primary_database_missing_refused_not_silently_created(self):
        (self.primary/signed.DB_NAME).unlink()
        self.assertIn("PRIMARY_SIGNED_REVOCATION_DB_MISSING",self.refused()["detail"])
        self.assertFalse((self.primary/signed.DB_NAME).exists())
    def test_10_corrupted_floor_database_refused(self):
        (self.secondary/floor.NAME).write_bytes(b"corrupted-SQLite")
        self.refused()
    def test_11_changed_floor_state_height_is_detected(self):
        c=sqlite3.connect(self.secondary/floor.NAME)
        with c:c.execute("UPDATE state SET height=999 WHERE id=1")
        c.close()
        self.assertIn("FLOOR_METADATA_MISMATCH",self.refused()["detail"])
    def test_12_deleted_floor_state_refused(self):
        c=sqlite3.connect(self.secondary/floor.NAME)
        with c:c.execute("DELETE FROM state WHERE id=1")
        c.close()
        self.assertIn("FLOOR_METADATA_MISSING",self.refused()["detail"])
    def test_13_untrusted_fake_floor_row_never_admitted(self):
        c=sqlite3.connect(self.secondary/floor.NAME)
        with c:c.execute("INSERT INTO checkpoints VALUES(1,4,?,?,?,?,?,?)",
                         ("a"*64,"b"*64,signed.GENESIS,"c"*64,floor.GENESIS,"d"*64))
        c.close()
        self.assertIn("FLOOR_CHAIN_INVALID",self.refused()["detail"])
    def test_14_read_only_audit_does_not_modify_existing_db_bytes(self):
        p=self.primary/signed.DB_NAME
        w=self.secondary/floor.NAME
        before=(p.read_bytes(),w.read_bytes())
        self.audit()
        self.assertEqual(before,(p.read_bytes(),w.read_bytes()))
    def test_15_unknown_operation_never_accepted(self):
        self.assertIn("UNKNOWN_ACTION",self.refused(action="release")["detail"])
    def test_16_relative_paths_refused(self):
        self.refused(a="relative/publisher")
        self.refused(b="relative/witness")

@unittest.skipUnless(os.name!="nt" and Path(signed.SSH).is_file(),
                      "native Unix/OpenSSH Ed25519 fixture required")
class NativeSignedRevocationFloor(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.keys=tempfile.TemporaryDirectory(prefix="nordoi-r5i-keys-")
        cls.keydir=Path(cls.keys.name)
        os.chmod(cls.keydir,0o700)
        cls.private,cls.pub=signed.keygen(cls.keydir)
    @classmethod
    def tearDownClass(cls):cls.keys.cleanup()
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5i-native-")
        self.root=Path(self.temp.name)
        self.primary=self.root/"primary"
        self.second=self.root/"floor"
        self.primary.mkdir(mode=0o700)
        self.second.mkdir(mode=0o700)
        os.chmod(self.primary,0o700)
        os.chmod(self.second,0o700)
        with signed.RevocationLedger(self.primary) as db:
            self.assertEqual(db.audit()["local_height"],0)
        self.assertEqual(floor.execute(self.primary,self.second,"audit")["floor_height"],0)
        self.original=self.root/"signed-genesis.sqlite3"
        self.floor_genesis=self.root/"floor-genesis.sqlite3"
        shutil.copy2(self.primary/signed.DB_NAME,self.original)
        shutil.copy2(self.second/floor.NAME,self.floor_genesis)
    def tearDown(self):self.temp.cleanup()
    def revoke(self,nonce="a"*64):
        pkt=signed.make_packet(self.private,self.pub,self.keydir,nonce=nonce)
        with signed.RevocationLedger(self.primary) as db:
            out=db.revoke(pkt,self.pub,1800000060)
        self.assertEqual(out["status"],"LAB_LOCAL_REVOKE_COMMIT")
        self.assertEqual(out["admission"],"NOT_AUTHORIZED")
        return out
    def checkpoint(self):
        x=floor.execute(self.primary,self.second,"checkpoint")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
        return x
    def test_17_real_signed_revocation_checkpoint_commit_local_only(self):
        self.revoke()
        x=self.checkpoint()
        self.assertEqual(x["status"],"LAB_LOCAL_CHECKPOINT_COMMIT")
        self.assertEqual(x["floor_height"],1)
    def test_18_real_duplicate_checkpoint_refused(self):
        self.revoke()
        self.checkpoint()
        x=self.checkpoint()
        self.assertIn("CHECKPOINT_REPLAY",x["detail"])
    def test_19_real_revoked_height_one_verifies_on_reopen_and_separate_process(self):
        self.revoke()
        self.checkpoint()
        cmd=[sys.executable,"-B",str(Path(floor.__file__)),"--primary-directory",str(self.primary),
             "--floor-directory",str(self.second),"--action","audit"]
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=20)
        self.assertEqual(p.returncode,2)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT")
        self.assertEqual((x["primary_height"],x["floor_height"]),(1,1))
    def test_20_real_old_primary_restore_detected_if_floor_kept(self):
        self.revoke()
        self.checkpoint()
        shutil.copy2(self.original,self.primary/signed.DB_NAME)
        x=floor.execute(self.primary,self.second,"audit")
        self.assertEqual(x["reason"],"SIGNED_REVOCATION_LOSS_DETECTED_WITH_PRESERVED_LOCAL_FLOOR")
        self.assertEqual((x["primary_height"],x["floor_height"]),(0,1))
    def test_21_signed_revoke_same_height_fork_detected_by_floor(self):
        self.revoke()
        self.checkpoint()
        shutil.copy2(self.original,self.primary/signed.DB_NAME)
        self.revoke(nonce="c"*64)
        x=floor.execute(self.primary,self.second,"audit")
        self.assertEqual(x["reason"],"REVOCATION_FLOOR_PRIMARY_FORK_OR_EQUIVOCATION")
        self.assertEqual((x["primary_height"],x["floor_height"]),(1,1))
    def test_22_joint_old_sqlite_snapshot_restore_is_undetectable(self):
        self.revoke()
        self.checkpoint()
        shutil.copy2(self.original,self.primary/signed.DB_NAME)
        shutil.copy2(self.floor_genesis,self.second/floor.NAME)
        x=floor.execute(self.primary,self.second,"audit")
        self.assertEqual(x["reason"],"LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT")
        self.assertEqual((x["primary_height"],x["floor_height"]),(0,0))
        self.assertFalse(x["local_revoked"])
        self.assertEqual(x["global_antirollback"],"NOT_IMPLEMENTED")
    def test_23_floor_only_old_snapshot_restore_loses_local_checkpoint(self):
        self.revoke()
        self.checkpoint()
        shutil.copy2(self.floor_genesis,self.second/floor.NAME)
        x=floor.execute(self.primary,self.second,"audit")
        self.assertEqual(x["reason"],"PRIMARY_REVOCATION_AHEAD_OF_UNCHECKPOINTED_LOCAL_FLOOR")
        self.assertEqual((x["primary_height"],x["floor_height"]),(1,0))
    def test_24_native_cli_checkpoint_success_still_exits_two(self):
        self.revoke()
        cmd=[sys.executable,"-B",str(Path(floor.__file__)),"--primary-directory",str(self.primary),
             "--floor-directory",str(self.second),"--action","checkpoint"]
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=20)
        self.assertEqual(p.returncode,2)
        x=json.loads(p.stdout)
        self.assertEqual(x["status"],"LAB_LOCAL_CHECKPOINT_COMMIT")
        self.assertEqual(x["independent_guardian"],"NOT_ENROLLED")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
