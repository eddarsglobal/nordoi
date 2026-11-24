"""APP-025 real Rust native post-interruption forensic recovery tests.

Sources are APP-012 genuine NORDOI native P2.7, APP-013 seal, APP-014
portable package, APP-024 genuine Rust process executions. APP-025 itself
never invokes native execution or mutates the user's original evidence.
"""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import unittest

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
sys.path.insert(0,str(HERE.parent/"app024_verified_concurrent_resume"))
import test_app024 as integration
import run_app024 as resumer
sys.path.insert(0,str(HERE))
import audit_app025 as forensic


class NativeInterruptedForensicTests(unittest.TestCase):
    def setUp(self):
        self.fixture=integration.RealPortableConcurrentResume(
            "test_01_real_parallel_native_checkpoint_resume")
        self.fixture.setUp()  # actual Rust checkpoint, seal and restoration
        self.base=self.fixture.base
        self.root=self.fixture.restored
        self.bundle=self.fixture.bundle
        self.pin=self.fixture.outer_pin
        self.suffix=self.base/"captured-genuine-native-suffix.jsonl"
        self.write_suffix(b"")

    def tearDown(self):
        self.fixture.tearDown()

    def write_suffix(self,raw:bytes):
        self.suffix.write_bytes(raw)
        self.suffix_pin=hashlib.sha256(raw).hexdigest()

    def capture_success(self):
        rows=self.fixture.good(workers=2)
        self.write_suffix(b"".join(resumer.json_line(x) for x in rows))
        return rows

    def fresh_partial(self,number:int=1):
        # Build a real successful full Rust DAG, then COPIES only its
        # verified native jobs into another restored original checkpoint.
        # This is an offline recovery state, not a mock of Rust results.
        rows=self.capture_success()
        partial=self.base/("new_partial_"+str(number))
        partial.mkdir()
        resumer.portable.restore(self.bundle,self.pin,partial)
        for i in range(3,3+number):
            shutil.copytree(self.root/f"job-{i:04d}",
                            partial/f"job-{i:04d}")
        self.root=partial
        self.write_suffix(b"".join(resumer.json_line(x)
                                   for x in rows[:number]))
        return rows

    def call(self,*,root=None,bundle=None,pin=None,suffix=None,
             suffix_pin=None):
        p=subprocess.run(
           [sys.executable,"-B",str(HERE/"audit_app025.py"),
            "--portable-dir",str(self.bundle if bundle is None else bundle),
            "--expected-portable-sha256",self.pin if pin is None else pin,
            "--restore-root",str(self.root if root is None else root),
            "--suffix-events",str(self.suffix if suffix is None else suffix),
            "--expected-suffix-sha256",
                 self.suffix_pin if suffix_pin is None else suffix_pin],
           cwd=REPO,capture_output=True,timeout=75)
        try:
            data=[json.loads(x) for x in p.stdout.splitlines()]
        except (ValueError,UnicodeError) as e:
            self.fail(f"not JSON:{e}:{p.stdout!r}:{p.stderr!r}")
        self.assertEqual(len(data),1,(p.stdout,p.stderr))
        return p,data[0]

    def good(self,status):
        p,d=self.call()
        self.assertEqual(p.returncode,0,(d,p.stderr))
        self.assertEqual(d["status"],status)
        self.assertEqual(d["schema"],forensic.SCHEMA)
        self.assertEqual(d["actual_user_data_files_written"],0)
        self.assertEqual(d["native_rust_processes_started_by_auditor"],0)
        return d

    def blocked(self,reason=None):
        p,d=self.call()
        self.assertEqual(p.returncode,3,(d,p.stderr))
        self.assertEqual(d["status"],"UNCERTAIN_BLOCKED")
        self.assertEqual(d["next_action"],"STOP_NO_AUTOMATIC_NATIVE_REPLAY")
        if reason is not None:
            self.assertEqual(d["reason"],reason)
        return d

    def test_01_original_checkpoint_reusable_if_zero_new_effects(self):
        d=self.good("ORIGINAL_CHECKPOINT_REUSABLE")
        self.assertEqual(d["new_verified_events"],0)
        self.assertEqual(d["verified_native_jobs"],2)

    def test_02_real_complete_concurrent_dag_is_verified(self):
        self.capture_success()
        d=self.good("COMPLETE_VERIFIED")
        self.assertEqual(d["verified_events_total"],6)
        self.assertEqual(d["verified_native_jobs"],6)

    def test_03_single_verified_new_job_requires_new_checkpoint(self):
        self.fresh_partial(1)
        d=self.good("CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT")
        self.assertEqual(d["new_verified_events"],1)
        self.assertEqual(d["verified_native_jobs"],3)

    def test_04_two_native_siblings_can_form_clean_partial_prefix(self):
        self.fresh_partial(2)
        d=self.good("CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT")
        self.assertEqual(d["new_verified_events"],2)
        self.assertEqual(d["verified_native_jobs"],4)

    def test_05_no_old_job_is_replayed_by_readonly_auditor(self):
        original=self.fixture.native_bytes(self.root)
        self.good("ORIGINAL_CHECKPOINT_REUSABLE")
        self.assertEqual(original,self.fixture.native_bytes(self.root))

    def test_06_completed_history_preserves_real_native_file_bytes(self):
        self.capture_success()
        before=self.fixture.native_bytes(self.root)
        self.good("COMPLETE_VERIFIED")
        self.assertEqual(before,self.fixture.native_bytes(self.root))

    def test_07_clean_partial_audit_writes_no_user_data(self):
        self.fresh_partial()
        before=self.fixture.native_bytes(self.root)
        self.good("CLEAN_PREFIX_NEEDS_NEW_CHECKPOINT")
        self.assertEqual(before,self.fixture.native_bytes(self.root))

    def test_08_concurrent_orphan_without_suffix_is_blocked(self):
        self.capture_success()
        self.write_suffix(b"")
        self.blocked("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_09_only_first_suffix_event_but_other_job_exists_blocks(self):
        rows=self.capture_success()
        self.write_suffix(resumer.json_line(rows[0]))
        self.blocked("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_10_truncated_suffix_blocks_even_with_corresponding_real_files(self):
        self.capture_success()
        self.write_suffix(self.suffix.read_bytes()[:-1])
        self.blocked("SUFFIX_TRUNCATED_OR_UNTERMINATED")

    def test_11_wrong_external_suffix_pin_blocks(self):
        self.capture_success()
        p,d=self.call(suffix_pin="0"*64)
        self.assertEqual(p.returncode,3)
        self.assertEqual(d["reason"],"SUFFIX_EXTERNAL_PIN_MISMATCH")

    def test_12_wrong_portable_pin_blocks_original_restore(self):
        p,d=self.call(pin="f"*64)
        self.assertEqual(p.returncode,3)
        self.assertEqual(d["status"],"UNCERTAIN_BLOCKED")

    def test_13_corrupted_historical_native_bytes_block(self):
        f=self.root/"job-0001/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"tampering")
        self.blocked()

    def test_14_corrupted_new_native_bytes_block(self):
        self.capture_success()
        f=self.root/"job-0003/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"tampering")
        self.blocked()

    def test_15_additional_unlogged_native_job_blocks(self):
        self.fresh_partial(1)
        (self.root/"job-0004").mkdir()
        self.blocked("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_16_extra_native_file_blocks(self):
        self.capture_success()
        (self.root/"extra-native.txt").write_bytes(b"not trusted")
        self.blocked("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_17_native_error_cannot_be_promoted_to_checkpoint(self):
        # APP-024 real Rust failure: native source mutation causes an ERROR.
        changed=self.base/"real-bad-noi-source"
        shutil.copytree(self.fixture.source,changed)
        p=changed/"app/rules.noi"
        raw=p.read_text("utf-8")
        self.assertIn("else { 0 }",raw)
        p.write_text(raw.replace("else { 0 }","else { 2 }"),encoding="utf-8")
        pin=resumer.pinned.source_tree_sha256(changed)
        process,records=self.fixture.launch(source=changed,source_pin=pin)
        self.assertEqual(process.returncode,3,(records,process.stderr))
        self.assertTrue(any(x["status"]=="ERROR" for x in records))
        self.write_suffix(b"".join(resumer.json_line(x) for x in records))
        self.blocked("UNCERTAIN_OR_FAILED_NATIVE_SUFFIX")

    def test_18_duplicate_native_event_blocks_even_with_new_pin(self):
        rows=self.capture_success()
        self.write_suffix(resumer.json_line(rows[0])*2)
        self.blocked()

    def test_19_wrong_event_step_blocks_even_with_rehashed_suffix(self):
        rows=self.capture_success()
        rows[0]["step"]=7
        self.write_suffix(b"".join(resumer.json_line(x) for x in rows))
        self.blocked("EVENT_SCHEMA_OR_STEP_INVALID")

    def test_20_wrong_real_native_decision_blocks(self):
        rows=self.capture_success()
        rows[0]["decision"]=1-rows[0]["decision"]
        self.write_suffix(b"".join(resumer.json_line(x) for x in rows))
        self.blocked()

    def test_21_missing_original_checkpoint_marker_blocks(self):
        (self.bundle/"PORTABLE.sha256").unlink()
        self.blocked()

    def test_22_injected_json_duplicate_key_cannot_pass_even_with_pin(self):
        rows=self.capture_success()
        raw=resumer.json_line(rows[0])
        raw=raw.replace(b'"step":3',b'"step":3,"step":3')
        self.write_suffix(raw+b"".join(resumer.json_line(x) for x in rows[1:]))
        self.blocked("DUPLICATE_RECEIPT_FIELD")

    def test_23_mixed_line_endings_refused_in_same_verified_capture(self):
        rows=self.capture_success()
        self.write_suffix(resumer.json_line(rows[0]).replace(b"\n",b"\r\n")
                          +b"".join(resumer.json_line(x) for x in rows[1:]))
        self.blocked("SUFFIX_NEWLINE_STYLE_CHANGED")

    def test_24_external_hash_does_not_claim_antirollback_or_signature(self):
        self.capture_success()
        d=self.good("COMPLETE_VERIFIED")
        self.assertEqual(d["global_antirollback"],"NOT_PROVEN")
        self.assertEqual(d["external_pin_authentication"],"NOT_PROVEN")
        self.assertEqual(d["formal_bureau_certification"],"NOT_CERTIFIED")
        self.assertEqual(d["production"],"NOT_AUTHORIZED")


if __name__=="__main__":
    unittest.main()
