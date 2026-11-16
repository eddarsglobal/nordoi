"""NORDOI E3 — 24 actual file/provenance adversarial tests, zero human testers.

Fixture log strings are labelled parser fixtures, never represented as executed
machine evidence. Genuine role suites run separately on pinned macOS Intel CI.
"""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
import bureau_001_e3_exact_head_provenance as e3
import bureau_001_e2_machine_only as e2

def current_head():
    return subprocess.check_output(
        ["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()

def parser_fixture():
    """REAL temp file byte fixtures: no test execution or attestation."""
    logs={}
    records=[]
    for role,cmd,expected in e2.specs():
        if expected.startswith("PYTHON_PASS_"):
            raw=b"Ran 24 tests in 0.123s\n\nOK\n"
        elif expected.startswith("RUST_PASS_"):
            n=int(expected.rsplit("_",1)[1])
            raw=("test result: ok. "+str(n)+
                 " passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n").encode()
        else:raw=b"cargo fmt --check completed\n"
        logs[role]=raw
        records.append(dict(role=role,
          execution_kind="ACTUAL_DETERMINISTIC_MACHINE_TEST",
          command=cmd,expected_control=expected,exit_code=0,status="PASS",
          claimed_independent_specialist=False,human_tester=False,
          elapsed_monotonic_seconds=0.01,
          log_sha256=hashlib.sha256(raw).hexdigest()))
    manifest={
        "schema":e2.SCHEMA,
        "source_head_before_after":current_head(),
        "source_worktree_clean":True,"runner_os":"Darwin",
        "runner_architecture":"x86_64","role_suite_count_expected":9,
        "role_suite_count_executed":9,"all_nine_machine_suites_passed":True,
        "real_human_testers":0,"independent_specialist_reviews_received":0,
        "outside_administrative_authority_authenticated":False,
        "native_admission_gates_unmet":12,"r6g":"LOCKED_BY_SEQUENCE",
        "production":"NOT_AUTHORIZED","machine_results":records
    }
    return manifest,logs

class ExactHeadSourceTests(unittest.TestCase):
    def test_01_actual_repo_head_matches_itself(self):
        self.assertEqual(e3.check_head(ROOT,current_head()),current_head())
    def test_02_old_synthetic_merge_head_not_accepted_as_new_head(self):
        with self.assertRaisesRegex(e3.Rejected,"CHECKOUT_HEAD_DIFFERS"):
            e3.check_head(ROOT,e3.OBSERVED_E2_MERGE)
    def test_03_original_e2_pr_head_differs_from_merge(self):
        self.assertNotEqual(e3.OBSERVED_E2_MERGE,e3.BASE_E2)
    def test_04_wrong_short_expected_head_rejected(self):
        with self.assertRaisesRegex(e3.Rejected,"EXPECTED_PR_HEAD"):
            e3.check_head(ROOT,"33a677")
    def test_05_wrong_uppercase_head_rejected(self):
        with self.assertRaisesRegex(e3.Rejected,"EXPECTED_PR_HEAD"):
            e3.check_head(ROOT,current_head().upper())
    def test_06_actual_e2_policy_and_r1_git_pins_audited(self):
        d=e3.audit(ROOT,current_head())
        self.assertEqual(d["e2_source_preflight"],"99/99_PASS")
        self.assertEqual(d["prior_e2_was_synthetic_merge"],True)
    def test_07_policy_exact_archived_sha_and_commit_boundary(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e3.POLICY))
        self.assertEqual(p["parent_e2_exact_head"],e3.BASE_E2)
        self.assertEqual(p["observed_e2_run"]["actual_checkout_head_from_e2_json"],
                         e3.OBSERVED_E2_MERGE)
    def test_08_nine_real_prior_log_digests_all_pinned_in_observation(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e3.POLICY))
        h=p["observed_e2_run"]["per_role_log_digests_reported_in_e2_job_log"]
        self.assertEqual(tuple(h),e3.ROLES)
        self.assertTrue(all(e3.HEX64.fullmatch(x) for x in h.values()))
    def test_09_no_human_no_independent_external_authority(self):
        p=e3.strictly_load(e3.real_file_bytes(ROOT/e3.POLICY))
        self.assertFalse(p["human_testers_allowed"])
        self.assertEqual(p["independent_specialist_reviews"],0)
        self.assertEqual(p["real_external_custodian"],"NOT_ENROLLED")
    def test_10_all_twelve_native_gates_unmet(self):
        d=e3.audit(ROOT,current_head())
        self.assertEqual(d["native_admission_gates_unmet"],12)
        self.assertEqual(d["r6g"],"LOCKED_BY_SEQUENCE")
    def test_11_read_actual_regular_file_bytes(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"evidence.log"
            p.write_bytes(b"real filesystem test\n")
            self.assertEqual(e3.real_file_bytes(p),b"real filesystem test\n")
    def test_12_actual_symlink_log_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            a=Path(d)/"a";b=Path(d)/"b"
            a.write_bytes(b"source")
            b.symlink_to(a)
            with self.assertRaisesRegex(e3.Rejected,"NOT_REGULAR_FILE"):
                e3.real_file_bytes(b)
    def test_13_missing_real_log_not_treated_as_pass(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(FileNotFoundError):
                e3.real_file_bytes(Path(d)/"never_created")
    def test_14_duplicate_json_key_is_rejected(self):
        with self.assertRaisesRegex(e3.Rejected,"DUPLICATE_EVIDENCE_KEY"):
            e3.strictly_load(b'{"status":"PASS","status":"BLOCK"}')
    def test_15_nonfinite_json_is_rejected(self):
        with self.assertRaisesRegex(e3.Rejected,"NONFINITE"):
            e3.strictly_load(b'{"value":NaN}')
    def test_16_oversized_actual_evidence_file_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"large.log";p.write_bytes(b"x"*1025)
            with self.assertRaisesRegex(e3.Rejected,"OVERSIZE"):
                e3.real_file_bytes(p,limit=1024)

class MachineEvidenceAdversarialParserTests(unittest.TestCase):
    def test_17_only_as_parser_fixture_nine_records_consistent(self):
        m,logs=parser_fixture()
        d=e3.validate_manifest(m,current_head(),logs)
        self.assertEqual(d["role_logs_sha256_verified"],9)
        self.assertEqual(d["evidence_class"],
             "SELF_CONSISTENT_CI_MACHINE_PROOF_NOT_INDEPENDENT_AUTHORITY")
    def test_18_actual_atomic_log_bytes_replacement_caught(self):
        m,logs=parser_fixture()
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"ATLAS.log";swap=Path(d)/"swap"
            p.write_bytes(logs["ATLAS"]);swap.write_bytes(b"false log bytes")
            os.replace(swap,p)
            logs["ATLAS"]=e3.real_file_bytes(p)
            with self.assertRaisesRegex(e3.Rejected,"LOG_BYTES_DIFFER"):
                e3.validate_manifest(m,current_head(),logs)
    def test_19_stale_e2_merge_sha_rejected_even_if_logs_match(self):
        m,logs=parser_fixture()
        m["source_head_before_after"]=e3.OBSERVED_E2_MERGE
        with self.assertRaisesRegex(e3.Rejected,"UNVERIFIED_HEAD"):
            e3.validate_manifest(m,current_head(),logs)
    def test_20_false_tenth_machine_role_rejected(self):
        m,logs=parser_fixture()
        m["machine_results"].append(deepcopy(m["machine_results"][0]))
        with self.assertRaisesRegex(e3.Rejected,"ROLE"):
            e3.validate_manifest(m,current_head(),logs)
    def test_21_replaced_role_command_rejected(self):
        m,logs=parser_fixture()
        m["machine_results"][0]["command"]=["sh","-c","echo PASS"]
        with self.assertRaisesRegex(e3.Rejected,"ROLE_COMMAND"):
            e3.validate_manifest(m,current_head(),logs)
    def test_22_false_independent_specialist_claim_rejected(self):
        m,logs=parser_fixture()
        m["machine_results"][0]["claimed_independent_specialist"]=True
        with self.assertRaisesRegex(e3.Rejected,"ROLE_COMMAND"):
            e3.validate_manifest(m,current_head(),logs)
    def test_23_actual_python_pass_count_mismatch_rejected(self):
        m,logs=parser_fixture()
        role="BOUNDARY"
        raw=b"Ran 23 tests in 0.123s\n\nOK\n"
        logs[role]=raw
        m["machine_results"][2]["log_sha256"]=hashlib.sha256(raw).hexdigest()
        with self.assertRaisesRegex(e3.Rejected,"ROLE_COUNTS"):
            e3.validate_manifest(m,current_head(),logs)
    def test_24_no_external_root_may_be_claimed_by_ci_self_hash(self):
        m,logs=parser_fixture()
        m["outside_administrative_authority_authenticated"]=True
        with self.assertRaisesRegex(e3.Rejected,"UNVERIFIED_HEAD"):
            e3.validate_manifest(m,current_head(),logs)

if __name__=="__main__":unittest.main()
