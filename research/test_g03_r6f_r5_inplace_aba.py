"""NORDOI G0.3 R6F BUREAU-001 R5: 24 real provenance/ABA cases.

16 portable source/denial/path boundary checks; eight genuine POSIX cases
use installed OpenSSH real keys, second OS processes mutating and restoring
files, kernel ctime/inode measurements, and R5Q real historical replay.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6f_r5_inplace_aba as lab

class PortableBoundary(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5-portable-")
        self.root=Path(self.temp.name)
        if os.name!="nt": os.chmod(self.root,0o700)
        self.target=self.root/"sample.allowed"
        self.target.write_bytes(b"owner key sample\n")
        self.other=self.root/"alternate.allowed"
        self.other.write_bytes(b"another key sample\n")

    def tearDown(self):
        self.temp.cleanup()

    def test_01_immutable_r4_parent_commit(self):
        self.assertEqual(lab.PARENT,"4dd43f37e7a7ac0ca0b0b39fcc88248cd2bad9ad")

    def test_02_protocol_scoped_to_no_authority(self):
        self.assertIn("MULTIFILE-DENY",lab.PROTOCOL)

    def test_03_no_native_admission(self):
        d=lab.deny("RESEARCH_ONLY")
        self.assertEqual(d["status"],"BLOCK")
        self.assertEqual(d["admission"],"NOT_AUTHORIZED")
        self.assertFalse(d["native_authority_granted"])

    def test_04_twelve_native_gates_unmet(self):
        self.assertEqual(lab.deny("RESEARCH_ONLY")["native_admission_gates_unmet"],12)

    def test_05_production_never_authorized(self):
        self.assertEqual(lab.deny("RESEARCH_ONLY")["production"],"NOT_AUTHORIZED")

    def test_06_disposable_private_root_accepted(self):
        self.assertEqual(lab.check_research_path(self.root,self.target),self.target)

    def test_07_outside_disposable_root_denied(self):
        with tempfile.TemporaryDirectory() as outside:
            p=Path(outside)/"different.allowed";p.write_bytes(b"other\n")
            with self.assertRaises(lab.Refused):
                lab.check_research_path(self.root,p)

    def test_08_noncanonical_root_prefix_denied(self):
        with tempfile.TemporaryDirectory(prefix="not-nordoi-r5-") as d:
            p=Path(d)/"sample.allowed";p.write_bytes(b"test\n")
            with self.assertRaises(lab.Refused):
                lab.check_research_path(d,p)

    def test_09_subdirectory_path_denied(self):
        sub=self.root/"sub";sub.mkdir()
        p=sub/"child.allowed";p.write_bytes(b"other\n")
        with self.assertRaises(lab.Refused):
            lab.check_research_path(self.root,p)

    def test_10_missing_disposable_proof_denied(self):
        with self.assertRaises(lab.Refused):
            lab.check_research_path(self.root,self.root/"missing.allowed")

    @unittest.skipIf(os.name=="nt","Windows user symlink privilege not presumed")
    def test_11_symlink_disposable_proof_denied(self):
        link=self.root/"linked.allowed";link.symlink_to(self.target.name)
        with self.assertRaises(lab.Refused):
            lab.check_research_path(self.root,link)

    def test_12_sha_uses_real_bytes(self):
        self.assertEqual(lab.sha(self.target.read_bytes()),
                         hashlib.sha256(self.target.read_bytes()).hexdigest())

    def test_13_original_and_alternate_are_different_files(self):
        self.assertNotEqual(self.target.read_bytes(),self.other.read_bytes())

    def test_14_exact_previous_native_quarantine_reason(self):
        self.assertEqual(lab.EXPECTED_QUARANTINE,
                         "R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED")

    def test_15_child_inplace_mutation_rejects_wrong_pair_count(self):
        with self.assertRaises(lab.Refused):
            lab.second_process_inplace_restore(self.root,[])

    def test_16_bad_interleaving_operation_refused(self):
        with self.assertRaises(lab.Refused):
            lab.run_guarded_interleaving({}, "pretend_attack")

@unittest.skipUnless(lab.r4.is_posix_ssh(),
                     "real installed POSIX OpenSSH + inherited fd required")
class RealNativeABAAndCrossEpoch(unittest.TestCase):
    def setUp(self):
        self.home,self.data=lab.r6e.create_lab()

    def tearDown(self):
        self.home.cleanup()

    def victim(self,name):
        target=self.data["root"]/("nordoi-r5-"+name+".allowed")
        lab.r4.copy_private(self.data["guardian_allowed"],target)
        return target

    def rogue(self):
        return lab.r3.create_rogue_signed_receipt(self.data)

    def test_17_real_open_ssh_baseline_no_file_mutations(self):
        x=lab.r4.assess_lab(self.data)
        self.assertEqual(x["reason"],
                         "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY")
        self.assertFalse(x["native_authority_granted"])

    def test_18_actual_single_file_inplace_write_restore_quarantined(self):
        x,history=lab.run_guarded_interleaving(self.data,"single_inplace")
        self.assertEqual(x["reason"],lab.EXPECTED_QUARANTINE)
        self.assertTrue(history["all_original_bytes_restored"])
        self.assertTrue(history["all_original_inodes_restored"])
        self.assertTrue(history["kernel_ctime_changed"])

    def test_19_actual_paired_public_and_allowed_inplace_restore_quarantined(self):
        x,h=lab.run_guarded_interleaving(self.data,"paired_inplace")
        self.assertEqual(x["reason"],lab.EXPECTED_QUARANTINE)
        self.assertTrue(h["all_original_bytes_restored"])
        self.assertTrue(h["all_original_inodes_restored"])
        self.assertTrue(h["kernel_ctime_changed"])

    def test_20_actual_path_a_to_b_to_a_original_inode_restored_quarantined(self):
        x,h=lab.run_guarded_interleaving(self.data,"path_aba")
        self.assertEqual(x["reason"],lab.EXPECTED_QUARANTINE)
        self.assertTrue(h["original_inode_returned"])
        self.assertTrue(h["original_bytes_returned"])
        self.assertTrue(h["kernel_ctime_changed"])

    def test_21_second_os_process_restores_original_file_bytes_inplace(self):
        victim=self.victim("independent-inplace")
        _,alt,_=self.rogue()
        original=victim.read_bytes()
        inode=victim.stat().st_ino
        h=lab.second_process_inplace_restore(self.data["root"],[(victim,alt)])
        self.assertTrue(h["actual_os_file_writes"])
        self.assertEqual(original,victim.read_bytes())
        self.assertEqual(inode,victim.stat().st_ino)

    def test_22_second_os_process_restores_original_inode_via_path_aba(self):
        victim=self.victim("independent-aba")
        _,alt,_=self.rogue()
        alternate=self.data["root"]/"nordoi-r5-aba-input"
        lab.r4.copy_private(alt,alternate)
        before=victim.stat()
        h=lab.second_process_path_aba(self.data["root"],victim,alternate)
        self.assertTrue(h["original_inode_returned"])
        self.assertEqual(before.st_ino,victim.stat().st_ino)

    def test_23_real_cross_epoch_three_file_rollback_is_not_global_custody(self):
        x=lab.inspect_cross_epoch()
        self.assertTrue(x["same_admin_all_three_restored"])
        self.assertTrue(x["fresh_os_process_observed_old_local_history"])
        self.assertTrue(x["globally_spent_nonce_reissued_after_full_rollback"])
        self.assertFalse(x["protected_independent_high_water"])

    def test_24_real_cli_replay_returns_intentional_denial_2(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),
                          "--exercise"],capture_output=True,text=True,timeout=300,
                         check=False)
        self.assertEqual(p.returncode,2,p.stderr+"\n"+p.stdout)
        x=json.loads(p.stdout)
        self.assertEqual(
            x["reason"],
            "R5_REAL_INPLACE_ABA_MULTIFILE_DETECTED_BUT_CROSS_EPOCH_CUSTODY_ABSENT")
        self.assertTrue(x["actual_child_paired_multifile_mutation_and_restore"])
        self.assertTrue(x["coherent_local_multi_file_rollback_invisible_to_new_examiner"])
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertEqual(x["production"],"NOT_AUTHORIZED")

if __name__=="__main__":
    unittest.main()
