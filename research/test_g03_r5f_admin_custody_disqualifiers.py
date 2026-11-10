"""G0.3 R5F: 24 read-only actual OS custody disqualifier tests.

Uses genuine R5D and R5E SQLite on-disk stores in separate sibling dirs.
Does NOT simulate a separate human, independent OS admin, outside key or OOB root.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r5d_local_monotonic_ledger as primary
import g03_r5e_second_store_witness as witness
import g03_r5f_admin_custody_disqualifiers as audit

class CustodyDisqualifierTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory(prefix="nordoi-r5f-")
        self.base = Path(self.workspace.name)
        self.a = self.base / "publisher"
        self.b = self.base / "same-account-witness"
        self.a.mkdir(mode=0o700)
        self.b.mkdir(mode=0o700)
        for directory in (self.a, self.b):
            os.chmod(directory, 0o700)
        local = primary.execute(self.a, "append", nonce="a"*64, payload="b"*64,
                                epoch=4, expected_prev=primary.GENESIS)
        self.assertEqual(local["status"], "LAB_LOCAL_COMMIT")
        signed = witness.execute(self.a, self.b, "checkpoint")
        self.assertEqual(signed["status"], "LAB_WITNESS_COMMIT")

    def tearDown(self):
        self.workspace.cleanup()

    def check(self, a=None, b=None):
        result = audit.assess(self.a if a is None else a, self.b if b is None else b)
        self.assertEqual(result["status"], "BLOCK")
        self.assertEqual(result["admission"], "NOT_AUTHORIZED")
        self.assertEqual(result["production"], "NOT_AUTHORIZED")
        return result

    def denied(self, a=None, b=None):
        result = self.check(a, b)
        self.assertEqual(result["finding"], "UNSAFE_OR_MISSING_CUSTODY_EVIDENCE")
        return result

    def test_01_two_genuine_sqlite_files_inspected_but_never_admitted(self):
        x=self.check()
        self.assertTrue(x["distinct_file_inodes"])

    def test_02_same_local_uid_disqualifies_independence_on_posix(self):
        x=self.check()
        if os.name=="nt":
            self.assertEqual(x["finding"],"NT_OWNER_ACL_AND_ADMIN_DOMAIN_NOT_ATTESTED")
        else:
            self.assertEqual(x["finding"],"SAME_OS_UID_CONTROLS_BOTH_SQLITE_STORES")

    def test_03_two_separate_filenames_are_not_external_custody(self):
        x=self.check()
        self.assertEqual(x["administrator_independence"],"NOT_ATTESTED")

    def test_04_two_different_directories_are_not_external_custody(self):
        self.assertNotEqual(self.a.resolve(),self.b.resolve())
        self.assertEqual(self.check()["admission"],"NOT_AUTHORIZED")

    def test_05_same_storage_device_is_observed_not_a_trust_certificate(self):
        self.assertTrue(self.check()["same_storage_device"])

    def test_06_neither_actual_file_supplies_an_independent_identity(self):
        self.assertEqual(self.check()["external_identity"],"NOT_VERIFIED")

    def test_07_neither_actual_file_enrolls_owner_oob_root(self):
        self.assertEqual(self.check()["owner_oob_key_root"],"NOT_ENROLLED")

    def test_08_local_second_store_is_not_protected_remote_highwater(self):
        self.assertEqual(self.check()["external_protected_highwater"],"NOT_IMPLEMENTED")

    def test_09_local_unique_nonce_not_a_remote_spent_nonce_ledger(self):
        self.assertEqual(self.check()["remote_durable_nonce_ledger"],"NOT_IMPLEMENTED")

    def test_10_exact_cli_returns_exit_two_on_a_valid_two_store_pair(self):
        cmd=[sys.executable,"-B",str(Path(audit.__file__)),
             "--primary-directory",str(self.a),"--witness-directory",str(self.b)]
        proc=subprocess.run(cmd, capture_output=True,text=True,timeout=20)
        self.assertEqual(proc.returncode,2)
        self.assertEqual(json.loads(proc.stdout)["admission"],"NOT_AUTHORIZED")

    def test_11_read_only_audit_does_not_modify_primary_file_bytes(self):
        p=self.a/primary.DATABASE
        before=p.read_bytes()
        self.check()
        self.assertEqual(p.read_bytes(),before)

    def test_12_read_only_audit_does_not_modify_witness_file_bytes(self):
        p=self.b/witness.FILE
        before=p.read_bytes()
        self.check()
        self.assertEqual(p.read_bytes(),before)

    def test_13_absent_primary_file_is_denied(self):
        (self.a/primary.DATABASE).unlink()
        self.denied()

    def test_14_absent_witness_file_is_denied(self):
        (self.b/witness.FILE).unlink()
        self.denied()

    def test_15_primary_file_magic_corruption_is_denied(self):
        (self.a/primary.DATABASE).write_bytes(b"corrupted database")
        self.denied()

    def test_16_witness_file_magic_corruption_is_denied(self):
        (self.b/witness.FILE).write_bytes(b"corrupted database")
        self.denied()

    def test_17_primary_database_symlink_is_denied(self):
        original=self.a/primary.DATABASE
        other=self.base/"original-primary"
        original.rename(other)
        try:
            original.symlink_to(other)
        except OSError:
            self.skipTest("local symlink unavailable")
        self.denied()

    def test_18_witness_database_symlink_is_denied(self):
        original=self.b/witness.FILE
        other=self.base/"original-witness"
        original.rename(other)
        try:
            original.symlink_to(other)
        except OSError:
            self.skipTest("local symlink unavailable")
        self.denied()

    def test_19_symlink_primary_directory_is_denied(self):
        alias=self.base/"publisher-alias"
        try:
            alias.symlink_to(self.a, target_is_directory=True)
        except OSError:
            self.skipTest("local directory symlink unavailable")
        self.denied(a=alias)

    def test_20_symlink_witness_directory_is_denied(self):
        alias=self.base/"witness-alias"
        try:
            alias.symlink_to(self.b, target_is_directory=True)
        except OSError:
            self.skipTest("local directory symlink unavailable")
        self.denied(b=alias)

    def test_21_same_directory_is_denied(self):
        self.denied(b=self.a)

    def test_22_nested_directory_is_denied(self):
        sub=self.a/"nested"
        sub.mkdir(mode=0o700)
        os.chmod(sub,0o700)
        self.denied(b=sub)

    @unittest.skipIf(os.name=="nt","POSIX owner mode semantics required")
    def test_23_world_accessible_directory_is_denied(self):
        os.chmod(self.b,0o755)
        self.denied()

    def test_24_hardlinked_primary_and_witness_files_not_independent(self):
        if not hasattr(os,"link"):
            self.skipTest("hard links unavailable")
        local=self.a/primary.DATABASE
        target=self.b/witness.FILE
        target.unlink()
        try:
            os.link(local,target)
        except OSError:
            self.skipTest("hard links unavailable on volume")
        result=self.denied()
        self.assertIn("HARDLINK_OR_ALIAS",result["detail"])

if __name__=="__main__":
    unittest.main()
