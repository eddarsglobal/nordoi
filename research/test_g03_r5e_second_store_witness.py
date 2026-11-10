"""R5E: real separately persisted second store; denial-only adversarial evidence."""
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
import g03_r5d_local_monotonic_ledger as primary
import g03_r5e_second_store_witness as witness
A,B,C,D="a"*64,"b"*64,"c"*64,"d"*64

class TwoStoreActualFiles(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5e-two-store-")
        self.root=Path(self.temp.name)
        self.source=self.root/"publisher"
        self.remote=self.root/"second-store"
        self.source.mkdir(mode=0o700)
        self.remote.mkdir(mode=0o700)
        for p in (self.source,self.remote):os.chmod(p,0o700)
    def tearDown(self):self.temp.cleanup()
    def local(self,nonce=A,prev=primary.GENESIS,epoch=4):
        r=primary.execute(self.source,"append",nonce=nonce,payload=B,epoch=epoch,expected_prev=prev)
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def audit(self):
        r=witness.execute(self.source,self.remote,"audit")
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertEqual(r["global_antirollback"],"NOT_IMPLEMENTED")
        return r
    def checkpoint(self):
        r=witness.execute(self.source,self.remote,"checkpoint")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def ready(self):
        x=self.local();self.assertEqual(x["status"],"LAB_LOCAL_COMMIT")
        w=self.checkpoint();self.assertEqual(w["status"],"LAB_WITNESS_COMMIT")
        return x,w
    def mutate_db(self,filename,sql,params=()):
        conn=sqlite3.connect(filename)
        try:
            with conn:conn.execute(sql,params)
        finally:conn.close()
    def test_01_initial_two_actual_files_genesis_only(self):
        r=self.audit()
        self.assertEqual((r["local_height"],r["witness_height"]),(0,0))
        self.assertTrue((self.source/primary.DATABASE).is_file())
        self.assertTrue((self.remote/witness.FILE).is_file())
    def test_02_real_checkpoint_in_separate_sqlite_file(self):
        _,w=self.ready();self.assertEqual(w["witness_height"],1)
        self.assertNotEqual(w["witness_head_sha256"],witness.GENESIS)
        self.assertNotEqual((self.source/primary.DATABASE).resolve(),(self.remote/witness.FILE).resolve())
    def test_03_second_store_reopen_after_process_boundary(self):
        self.ready()
        script=Path(witness.__file__)
        r=subprocess.run([sys.executable,"-B",str(script),"--primary-directory",str(self.source),
                          "--witness-directory",str(self.remote),"--operation","audit"],
                         capture_output=True,text=True,timeout=25)
        self.assertEqual(r.returncode,2)
        self.assertEqual(json.loads(r.stdout)["reason"],"LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT")
    def test_04_checkpoints_must_be_strictly_sequential(self):
        self.ready()
        r=self.checkpoint()
        self.assertIn("REPLAY_GAP",r["detail"])
    def test_05_local_unwitnessed_append_detected_as_lag(self):
        self.local()
        self.assertEqual(self.audit()["reason"],"PRIMARY_AHEAD_OF_SECOND_STORE_UNCHECKPOINTED")
    def test_06_second_checkpoint_for_sequential_primary_height(self):
        first,_=self.ready()
        second=self.local(nonce=C,prev=first["local_head_sha256"])
        self.assertEqual(self.checkpoint()["witness_height"],2)
        self.assertEqual(self.audit()["reason"],"LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT")
        self.assertEqual(second["local_height"],2)
    def test_07_same_nonce_local_replay_denied_across_stores(self):
        first,_=self.ready()
        result=self.local(nonce=A,prev=first["local_head_sha256"])
        self.assertIn("CHALLENGE_ALREADY_CONSUMED",result["detail"])
        self.assertEqual(self.audit()["witness_height"],1)
    def test_08_old_snapshot_primary_rollback_detected_when_second_preserved(self):
        one,_=self.ready()
        saved=self.root/"primary.old.sqlite3"
        shutil.copy2(self.source/primary.DATABASE,saved)
        self.local(nonce=C,prev=one["local_head_sha256"])
        self.assertEqual(self.checkpoint()["witness_height"],2)
        shutil.copy2(saved,self.source/primary.DATABASE)
        result=self.audit()
        self.assertEqual(result["reason"],"PRIMARY_ROLLBACK_DETECTED_BY_PRESERVED_SECOND_STORE")
        self.assertEqual((result["local_height"],result["witness_height"]),(1,2))
    def test_09_primary_fork_same_height_not_matching_witness(self):
        one,_=self.ready()
        saved=self.root/"old.sqlite3"
        shutil.copy2(self.source/primary.DATABASE,saved)
        self.local(nonce=C,prev=one["local_head_sha256"])
        self.checkpoint()
        shutil.copy2(saved,self.source/primary.DATABASE)
        other= self.local(nonce=D,prev=one["local_head_sha256"])
        self.assertEqual(other["status"],"LAB_LOCAL_COMMIT")
        self.assertEqual(self.audit()["reason"],"WITNESS_PRIMARY_EQUIVOCATION")
    def test_10_full_snapshot_rollback_of_both_stores_undetectable(self):
        one,_=self.ready()
        a=self.root/"saved-publisher.sqlite3"
        b=self.root/"saved-witness.sqlite3"
        shutil.copy2(self.source/primary.DATABASE,a)
        shutil.copy2(self.remote/witness.FILE,b)
        self.local(nonce=C,prev=one["local_head_sha256"])
        self.checkpoint()
        shutil.copy2(a,self.source/primary.DATABASE)
        shutil.copy2(b,self.remote/witness.FILE)
        x=self.audit()
        self.assertEqual((x["local_height"],x["witness_height"]),(1,1))
        self.assertEqual(x["reason"],"LOCAL_TWO_STORE_MATCH_NOT_INDEPENDENT")
    def test_11_second_store_snapshot_rollback_causes_lag_not_proven_external(self):
        one,_=self.ready()
        saved=self.root/"witness.old.sqlite3"
        shutil.copy2(self.remote/witness.FILE,saved)
        self.local(nonce=C,prev=one["local_head_sha256"])
        self.checkpoint()
        shutil.copy2(saved,self.remote/witness.FILE)
        self.assertEqual(self.audit()["reason"],"PRIMARY_AHEAD_OF_SECOND_STORE_UNCHECKPOINTED")
    def test_12_witness_chain_manual_hash_corruption_detected(self):
        self.ready()
        self.mutate_db(self.remote/witness.FILE,"UPDATE checkpoints SET witness_head=? WHERE height=1",(D,))
        self.assertIn("WITNESS_CHAIN_INVALID",self.audit()["detail"])
    def test_13_witness_metadata_tamper_detected(self):
        self.ready()
        self.mutate_db(self.remote/witness.FILE,"UPDATE state SET height=99")
        self.assertIn("WITNESS_METADATA_MISMATCH",self.audit()["detail"])
    def test_14_witness_sequence_delete_detected(self):
        self.ready()
        self.mutate_db(self.remote/witness.FILE,"DELETE FROM checkpoints WHERE height=1")
        self.assertIn("WITNESS_METADATA_MISMATCH",self.audit()["detail"])
    def test_15_witness_failed_append_keeps_old_head(self):
        self.ready()
        self.checkpoint()
        self.assertEqual(self.audit()["witness_height"],1)
    def test_16_primary_epoch_increment_reflected_in_second_store(self):
        p=self.local(epoch=5)
        self.checkpoint()
        q=self.local(nonce=C,prev=p["local_head_sha256"],epoch=7)
        self.checkpoint()
        self.assertEqual(q["local_epoch"],7)
        conn=sqlite3.connect(self.remote/witness.FILE)
        try:
            epoch=conn.execute("SELECT epoch FROM state WHERE id=1").fetchone()[0]
        finally:
            conn.close()
        self.assertEqual(epoch,7)
    def test_17_primary_epoch_downgrade_refused(self):
        p=self.local(epoch=7);self.checkpoint()
        r=self.local(nonce=C,prev=p["local_head_sha256"],epoch=6)
        self.assertIn("EPOCH_ROLLBACK",r["detail"])
    def test_18_two_actual_processes_race_for_one_witness_checkpoint(self):
        self.local()
        cmd=[sys.executable,"-B",str(Path(witness.__file__)),"--primary-directory",str(self.source),
             "--witness-directory",str(self.remote),"--operation","checkpoint"]
        a=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        b=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        x,err1=a.communicate(timeout=30);y,err2=b.communicate(timeout=30)
        self.assertEqual((a.returncode,b.returncode),(2,2))
        self.assertFalse(err1 or err2)
        results=[json.loads(x),json.loads(y)]
        self.assertEqual(sum(r["status"]=="LAB_WITNESS_COMMIT" for r in results),1)
        self.assertEqual(self.audit()["witness_height"],1)
    def test_19_same_directory_prohibited_for_both_stores(self):
        self.assertEqual(witness.execute(self.source,self.source,"audit")["status"],"BLOCK")
    def test_20_nested_directory_prohibited_for_both_stores(self):
        nested=self.source/"other";nested.mkdir(mode=0o700)
        self.assertIn("DISTINCT_DIRECTORY_TREES",witness.execute(self.source,nested,"audit")["detail"])
    def test_21_symlink_witness_database_denied(self):
        target=self.root/"inert";target.write_bytes(b"unused")
        try:(self.remote/witness.FILE).symlink_to(target)
        except OSError:self.skipTest("symlink not supported")
        self.assertEqual(self.audit()["status"],"BLOCK")
    def test_22_fifo_witness_database_refused_when_supported(self):
        if not hasattr(os,"mkfifo"):self.skipTest("POSIX-only FIFO")
        try:os.mkfifo(self.remote/witness.FILE)
        except OSError:self.skipTest("FIFO not available")
        self.assertEqual(self.audit()["status"],"BLOCK")
    @unittest.skipIf(os.name=="nt","POSIX permission check")
    def test_23_insecure_witness_directory_refused(self):
        os.chmod(self.remote,0o755)
        self.assertEqual(self.audit()["status"],"BLOCK")
    def test_24_real_cli_checkpoint_returns_exit_two_even_on_local_commit(self):
        self.local()
        cmd=[sys.executable,"-B",str(Path(witness.__file__)),"--primary-directory",str(self.source),
             "--witness-directory",str(self.remote),"--operation","checkpoint"]
        r=subprocess.run(cmd,capture_output=True,text=True,timeout=25)
        self.assertEqual(r.returncode,2)
        self.assertEqual(json.loads(r.stdout)["status"],"LAB_WITNESS_COMMIT")
        self.assertEqual(json.loads(r.stdout)["admission"],"NOT_AUTHORIZED")

if __name__=="__main__":unittest.main()
