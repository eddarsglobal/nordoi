"""E4 — 24 portable source/real-file tests, machine-only and no outside actors.

These portable parser fixtures are NOT the actual Mac Intel nine-machine suites.
The dedicated CI Mac Intel job separately exercises real nine original logs,
with a second OS process accepting a coordinated copied log+manifest mutation.
"""
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e4_coherent_evidence_tampering as e4
import bureau_001_e3_exact_head_provenance as e3
import bureau_001_e2_machine_only as e2
from test_bureau_001_e3_exact_head_provenance import parser_fixture

def head():
    return subprocess.check_output(["git","rev-parse","HEAD"],
                                   cwd=ROOT,text=True).strip()

def save_fixture(folder):
    m,logs=parser_fixture()
    for role in e4.ROLES:
        (folder/(role+".log")).write_bytes(logs[role])
    (folder/"E2_MACHINE_EVIDENCE.json").write_text(
       json.dumps(m,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    return m,logs

class E4SourceAndLocalWitness(unittest.TestCase):
    def test_01_source_frozen_e3_head_audited(self):
        a=e4.audit(ROOT,head())
        self.assertEqual(a["result"],"E4_SOURCE_AUDIT_BOUNDARY_PASS")
        self.assertEqual(a["exact_pr_head"],head())
    def test_02_documented_e3_original_machine_results_correct(self):
        a=e4.audit(ROOT,head())
        self.assertEqual(a["prior_e3_exact_head"],e4.PARENT_E3)
        self.assertEqual(a["apple_silicon_original"],"CANCELLED_NOT_PASS")
    def test_03_original_e3_real_nine_suites_not_human(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e4.POLICY))
        self.assertEqual(p["observed_e3"]["actual_machine_nine_suites"],
                         "9/9_PASS_ON_EXACT_HEAD")
        self.assertTrue(p["no_human_testers"])
    def test_04_original_e3_artifact_sha_exact_recorded(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e4.POLICY))
        self.assertEqual(p["observed_e3"]["reported_artifact_zip_sha256"],
            "8e676db2868b7e9b888eff55c58b598af775a17f52a1b60ffa1e17bcfa644ac5")
    def test_05_no_automatic_independent_certificate(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e4.POLICY))
        self.assertEqual(p["independent_specialist_reviews"],0)
        self.assertFalse(p["actual_external_trust_verified"])
    def test_06_twelve_external_native_gates_still_unmet(self):
        d=e4.audit(ROOT,head())
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
    def test_07_no_retroactive_claim_that_arm_ci_passed(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e4.POLICY))
        self.assertEqual(p["observed_e3"]["four_platform_source_jobs"]["macos_arm"],
                         "CANCELLED_NOT_PASS")
    def test_08_old_e3_merge_attribution_still_source_checked(self):
        self.assertEqual(e3.audit(ROOT,head())["e2_source_preflight"],
                         "99/99_PASS")
    def test_09_local_anchor_has_no_signer(self):
        m,logs=parser_fixture()
        a=e4.local_anchor(b"fixture",logs,head())
        self.assertIsNone(a["signer"])
        self.assertFalse(a["external_custodian"])
    def test_10_local_anchor_binds_all_nine_roles(self):
        m,logs=parser_fixture()
        x=e4.local_anchor(b"sample",logs,head())
        self.assertEqual(tuple(x["nine_log_sha256"]),e4.ROLES)
        self.assertEqual(x["manifest_sha256"],e4.sha(b"sample"))
    def test_11_local_anchor_detects_changed_manifest(self):
        m,logs=parser_fixture()
        a=e4.local_anchor(b"original",logs,head())
        with self.assertRaisesRegex(e4.Blocked,"PRIOR_MANIFEST"):
            e4.against_anchor(a,b"forged",logs,head())
    def test_12_local_anchor_detects_changed_real_file_bytes(self):
        m,logs=parser_fixture()
        a=e4.local_anchor(b"same",logs,head())
        logs["ATLAS"]=b"actually replaced bytes"
        with self.assertRaisesRegex(e4.Blocked,"PRIOR_LOG"):
            e4.against_anchor(a,b"same",logs,head())
    def test_13_a_same_admin_can_replace_a_local_anchor(self):
        m,logs=parser_fixture()
        logs["ATLAS"]=b"fully reauthored bytes"
        new=e4.local_anchor(b"new manifest",logs,head())
        self.assertTrue(e4.against_anchor(new,b"new manifest",logs,head()))
    def test_14_incorrect_source_head_not_accepted(self):
        m,logs=parser_fixture()
        a=e4.local_anchor(b"m",logs,head())
        with self.assertRaisesRegex(e4.Blocked,"UNVERIFIED_OR_FICTITIOUS"):
            e4.against_anchor(a,b"m",logs,"0"*40)
    def test_15_real_disk_write_atomic_replaces_inode_contents(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"a.log";p.write_bytes(b"original")
            e4.write_atomic_file(p,b"new reality")
            self.assertEqual(e3.real_file_bytes(p),b"new reality")
    def test_16_atomic_file_refuses_staged_path_overwrite(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"a.log";p.write_bytes(b"x")
            (Path(d)/"a.log.E4_staged").write_bytes(b"y")
            with self.assertRaisesRegex(e4.Blocked,"STAGED_FILE_ALREADY_EXISTS"):
                e4.write_atomic_file(p,b"z")

class E4ActualFileParserBoundary(unittest.TestCase):
    def test_17_original_fixture_e3_validates_on_exact_head(self):
        m,logs=parser_fixture()
        self.assertEqual(e3.validate_manifest(m,head(),logs)["role_logs_sha256_verified"],9)
    def test_18_file_copies_preserve_real_bytes_and_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/"original";p.mkdir()
            _,logs=save_fixture(p)
            q=e4.copy_actual(p,Path(tmp)/"copy")
            m2,logs2=e4.get_raw_evidence(q)
            self.assertEqual(logs2,logs)
            self.assertEqual(e3.strictly_load(m2)["source_head_before_after"],head())
    def test_19_fixture_coherent_tamper_passes_original_e3_parser(self):
        # A separate real Mac CI exercise will use actual 9 native logs,
        # whereas this verifies the pure validator's blind spot.
        with tempfile.TemporaryDirectory() as d:
            f=Path(d);save_fixture(f)
            e4.coherent_edit(f)
            m,logs=e4.get_raw_evidence(f)
            self.assertEqual(e3.validate_manifest(e3.strictly_load(m),
                head(),logs)["role_logs_sha256_verified"],9)
    def test_20_actual_files_original_anchor_catches_coherent_tamper(self):
        with tempfile.TemporaryDirectory() as d:
            f=Path(d);save_fixture(f)
            b,logs=e4.get_raw_evidence(f)
            before=e4.local_anchor(b,logs,head())
            e4.coherent_edit(f)
            b2,logs2=e4.get_raw_evidence(f)
            with self.assertRaisesRegex(e4.Blocked,"PRIOR_MANIFEST"):
                e4.against_anchor(before,b2,logs2,head())
    def test_21_same_admin_reauthored_anchor_passes_after_real_mutation(self):
        with tempfile.TemporaryDirectory() as d:
            f=Path(d);save_fixture(f)
            e4.coherent_edit(f)
            b,logs=e4.get_raw_evidence(f)
            new=e4.local_anchor(b,logs,head())
            self.assertTrue(e4.against_anchor(new,b,logs,head()))
    def test_22_rewrite_of_only_log_no_manifest_fails_original_e3(self):
        with tempfile.TemporaryDirectory() as d:
            f=Path(d);save_fixture(f)
            original=e3.real_file_bytes(f/"ATLAS.log")
            e4.write_atomic_file(f/"ATLAS.log",e4.MARK+original)
            man,logs=e4.get_raw_evidence(f)
            with self.assertRaisesRegex(e3.Rejected,"LOG_BYTES_DIFFER"):
                e3.validate_manifest(e3.strictly_load(man),head(),logs)
    def test_23_coordinated_rewrite_keeps_actual_source_head_but_not_history(self):
        with tempfile.TemporaryDirectory() as d:
            f=Path(d);save_fixture(f)
            b,logs=e4.get_raw_evidence(f)
            origin=e3.strictly_load(b)["source_head_before_after"]
            e4.coherent_edit(f)
            b2,logs2=e4.get_raw_evidence(f)
            self.assertEqual(e3.strictly_load(b2)["source_head_before_after"],origin)
            self.assertNotEqual(e4.sha(b),e4.sha(b2))
    def test_24_fabricated_external_provenance_not_attributed_to_local_anchor(self):
        _,logs=parser_fixture()
        a=e4.local_anchor(b"m",logs,head())
        a["external_custodian"]=True
        with self.assertRaisesRegex(e4.Blocked,"FICTITIOUS_EXTERNAL"):
            e4.against_anchor(a,b"m",logs,head())

if __name__=="__main__":
    unittest.main()
