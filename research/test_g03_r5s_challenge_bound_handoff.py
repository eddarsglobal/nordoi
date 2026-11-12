"""24 actual R5S challenge-bound three-file transfer controls.

16 portable: real SQLite/JSON archive, timestamps, challenge and local nonce.
8 POSIX native: actual separate receiver OS process and os.replace/rollback.
All verified handoff results are BLOCK: no external guardian has enrolled.
"""
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5q_coherent_local_rollback as history
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5s_challenge_bound_handoff as lab

class PortableTransfer(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-portable-")
        self.root=Path(self.tmp.name)
        if os.name!="nt":os.chmod(self.root,0o700)
        self.a,self.b,self.j=history.fixture(self.root)
        self.ledger=self.root/"receipt_nonces"
        self.ledger.mkdir(mode=0o700)
        if os.name!="nt":os.chmod(self.ledger,0o700)
        with denial.DenialJournal(self.ledger) as db:
            self.assertEqual(db.audit()["local_height"],0)
        self.archive=self.root/"transfer.zip"
        self.nonce="a"*64
        self.now=1100
        self.sent=lab.prepare(self.a,self.b,self.j,self.nonce,1000,1200,self.archive)
    def tearDown(self):self.tmp.cleanup()
    def check(self,path=None,nonce=None,now=None,ledger=None):
        result=lab.inspect(self.archive if path is None else path,
                           self.nonce if nonce is None else nonce,
                           self.now if now is None else now,
                           ledger)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["independent_custodian"],"NOT_ENROLLED")
        return result
    def manifest(self):
        with zipfile.ZipFile(self.archive) as z:
            return lab.manifest_parse(z.read("manifest.json"))
    def rewrite(self,manifest=None,blobs=None):
        with zipfile.ZipFile(self.archive) as z:
            m=self.manifest() if manifest is None else manifest
            data={key:z.read(key) for key in lab.ASSETS}
        if blobs is not None:data.update(blobs)
        self.archive.write_bytes(lab.bytes_zip(m,data))
        if os.name!="nt":os.chmod(self.archive,0o600)
    def test_01_actual_three_real_files_exported_as_canonical_zip(self):
        self.assertEqual(self.sent["reason"],"LOCAL_HANDOFF_ENVELOPE_EXPORTED_NOT_CUSTODY")
        with zipfile.ZipFile(self.archive) as z:
            self.assertEqual(set(z.namelist()),set(lab.ENTRIES))
            self.assertTrue(z.read(lab.ASSETS[0]).startswith(b"SQLite format 3\x00"))
            self.assertTrue(z.read(lab.ASSETS[2]).startswith(b"SQLite format 3\x00"))
        self.assertEqual(self.check()["reason"],"HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY")
    def test_02_manifest_exact_parent_challenge_and_all_three_hashes(self):
        m=self.manifest()
        self.assertEqual(m["parent"],lab.PARENT)
        self.assertEqual(m["challenge"],self.nonce)
        self.assertEqual(set(m["asset_sha256"]),set(lab.ASSETS))
        self.assertEqual(m["projection_sha256"],
                         history.digest_projection(history.projection(self.a,self.b,self.j)))
    def test_03_first_real_spent_nonce_local_commit_only(self):
        x=self.check(ledger=self.ledger)
        self.assertEqual(x["receiver_local_denial_height"],1)
        self.assertFalse(x["independent_custody_verified"])
    def test_04_repeated_local_challenge_is_quarantined(self):
        self.check(ledger=self.ledger)
        x=self.check(ledger=self.ledger)
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",x["detail"])
    def test_05_wrong_receiver_challenge_is_refused(self):
        self.assertIn("CHALLENGE_MISMATCH",self.check(nonce="b"*64)["detail"])
    def test_06_expired_local_claimed_timebox_is_quarantined(self):
        self.assertIn("TIMEBOX_EXPIRED",self.check(now=1300)["detail"])
    def test_07_claim_created_in_untrusted_local_future_is_quarantined(self):
        self.assertIn("TIMEBOX_EXPIRED",self.check(now=900)["detail"])
    def test_08_forged_invalid_ttl_and_boolean_timestamp_refused(self):
        m=self.manifest();m["expires_at"]=m["created_at"]+301
        with self.assertRaises(lab.Refused):lab.manifest_parse(lab.canonical(m))
        m=self.manifest();m["created_at"]=True
        with self.assertRaises(lab.Refused):lab.manifest_parse(lab.canonical(m))
    def test_09_duplicate_or_noncanonical_json_manifest_refused(self):
        with zipfile.ZipFile(self.archive) as z:
            raw=z.read("manifest.json")
        with self.assertRaises(lab.Refused):
            lab.manifest_parse(raw.replace(b'"project":',b'"project":"fake","project":'))
        with self.assertRaises(lab.Refused):
            lab.manifest_parse(json.dumps(json.loads(raw),indent=2).encode())
    def test_10_corrupted_real_archive_zip_is_quarantined(self):
        self.archive.write_bytes(b"Not a ZIP!")
        self.assertEqual(self.check()["reason"],"HANDOFF_TRANSFER_QUARANTINED")
    def test_11_modified_payload_against_original_hash_is_quarantined(self):
        with zipfile.ZipFile(self.archive) as z:
            primary=z.read(lab.ASSETS[0])
        self.rewrite(blobs={lab.ASSETS[0]:primary+b"tampered"})
        self.assertIn("ENCLOSED_FILE_SHA256_MISMATCH",self.check()["detail"])
    def test_12_forged_manifest_projection_hash_is_quarantined(self):
        m=self.manifest();m["projection_sha256"]="0"*64
        self.rewrite(manifest=m)
        self.assertIn("PROJECTION_MISMATCH",self.check()["detail"])
    def test_13_missing_an_asset_in_archive_is_quarantined(self):
        with zipfile.ZipFile(self.archive) as z:
            entries={k:z.read(k) for k in lab.ENTRIES if k!=lab.ASSETS[1]}
        with zipfile.ZipFile(self.archive,"w") as z:
            for k,v in entries.items():z.writestr(k,v)
        self.assertIn("EXACT_FOUR_FIXED_NAMES",self.check()["detail"])
    def test_14_archive_with_traversal_path_never_extracts(self):
        with zipfile.ZipFile(self.archive,"w") as z:
            z.writestr("../escape",b"escape")
            z.writestr("manifest.json",lab.canonical(self.manifest()))
        self.assertEqual(self.check()["reason"],"HANDOFF_TRANSFER_QUARANTINED")
        self.assertFalse((self.root.parent/"escape").exists())
    def test_15_valid_hash_manifest_cannot_make_corrupt_sqlite_consistent(self):
        with zipfile.ZipFile(self.archive) as z:
            assets={k:z.read(k) for k in lab.ASSETS}
        assets[lab.ASSETS[0]]=b"Invalid SQLite!"
        m=self.manifest();m["asset_sha256"][lab.ASSETS[0]]=lab.sha(assets[lab.ASSETS[0]])
        self.archive.write_bytes(lab.bytes_zip(m,assets))
        self.assertIn("RECEIVED_SQLITE_JSON_HISTORY_INVALID",self.check()["detail"])
    def test_16_actual_new_python_receiver_process_always_blocks(self):
        x=lab.second_process_inspection(self.archive,self.nonce,1100)
        self.assertEqual(x["reason"],"HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY")
        self.assertFalse(x["independent_custody_verified"])
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "POSIX actual second OS process, os.replace, rollback and denied nonce")
class NativeTransfer(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-native-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=history.fixture(self.root)
        self.rcv=self.root/"receipt"
        self.rcv.mkdir(mode=0o700);os.chmod(self.rcv,0o700)
        with denial.DenialJournal(self.rcv) as db:
            self.assertEqual(db.audit()["local_height"],0)
        self.old=self.root/"old-nonce.sqlite3"
        shutil.copy2(self.rcv/denial.NAME,self.old)
        self.path=self.root/"transfer.zip"
        lab.prepare(self.a,self.b,self.j,"a"*64,1000,1200,self.path)
    def tearDown(self):self.tmp.cleanup()
    def test_17_genuine_cross_os_process_received_three_sqlite_json_assets(self):
        r=lab.second_process_inspection(self.path,"a"*64,1100,self.rcv)
        self.assertEqual(r["reason"],"HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY")
        self.assertEqual(r["receiver_local_denial_height"],1)
        self.assertFalse(r["independent_custody_verified"])
    def test_18_second_actual_receiver_process_refuses_same_nonce(self):
        lab.second_process_inspection(self.path,"a"*64,1100,self.rcv)
        again=lab.second_process_inspection(self.path,"a"*64,1100,self.rcv)
        self.assertEqual(again["reason"],"HANDOFF_TRANSFER_QUARANTINED")
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",again["detail"])
    def test_19_real_world_b_does_not_invalidate_old_offline_bundle_without_highwater(self):
        history.advance_world(self.a,self.b,self.j)
        old=lab.second_process_inspection(self.path,"a"*64,1100)
        self.assertEqual(old["received_height"],1)
        self.assertEqual(history.projection(self.a,self.b,self.j)["primary_height"],2)
        self.assertEqual(old["reason"],"HANDOFF_BYTES_AND_CHALLENGE_VERIFIED_LOCAL_ONLY")
    def test_20_new_history_b_package_differs_but_still_only_local_receipt(self):
        history.advance_world(self.a,self.b,self.j)
        package=self.root/"world_b.zip"
        lab.prepare(self.a,self.b,self.j,"b"*64,1000,1200,package)
        x=lab.second_process_inspection(package,"b"*64,1100)
        y=lab.second_process_inspection(self.path,"a"*64,1100)
        self.assertNotEqual(x["received_history_sha256"],y["received_history_sha256"])
        self.assertFalse(x["independent_custody_verified"])
    def test_21_real_separate_process_restores_old_local_nonce_register_and_replays(self):
        first=lab.second_process_inspection(self.path,"a"*64,1100,self.rcv)
        self.assertEqual(first["receiver_local_denial_height"],1)
        fdgate.real_atomic_replace(self.old,self.rcv/denial.NAME)
        second=lab.second_process_inspection(self.path,"a"*64,1100,self.rcv)
        self.assertEqual(second["receiver_local_denial_height"],1)
        self.assertEqual(second["admission"],"NOT_AUTHORIZED")
    def test_22_real_identical_new_inode_atomic_bundle_swap_while_open_quarantined(self):
        dup=self.root/"duplicate.zip"
        shutil.copy2(self.path,dup);os.chmod(dup,0o600)
        r=lab.inspect(self.path,"a"*64,1100,
                      after_open=lambda:fdgate.real_atomic_replace(dup,self.path))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",r["detail"])
    def test_23_same_admin_receiver_output_proves_no_real_custodian_signature(self):
        x=lab.second_process_inspection(self.path,"a"*64,1100)
        self.assertEqual(x["independent_custodian"],"NOT_ENROLLED")
        self.assertEqual(x["external_monotonic_history"],"NOT_PROVEN")
        self.assertFalse(x["independent_custody_verified"])
    def test_24_actual_native_end_to_end_handoff_exercise_exit_two(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--exercise"]
        p=subprocess.run(cmd,text=True,capture_output=True,timeout=180)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"REAL_TRANSFER_WITHOUT_INDEPENDENT_CUSTODY_QUALIFIED")
        self.assertTrue(x["real_second_process_verified_integrity"])
        self.assertEqual(x["challenge_replay_same_local_ledger"],"QUARANTINED")
        self.assertEqual(x["challenge_replay_after_local_ledger_rollback"],"LOCAL_DENIAL_AGAIN")
        self.assertEqual(x["older_self_consistent_bundle_after_real_world_advance"],
                         "STILL_LOCALLY_VERIFIABLE")
        self.assertEqual(x["actual_external_signed_receipt"],"NONE")
        self.assertFalse(x["native_authority_granted"])

if __name__=="__main__":unittest.main()
