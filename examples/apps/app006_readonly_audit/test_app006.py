"""APP-006 genuine, read-only integrity tests of real APP-005 NORDOI batches.

All fixtures come from actually running the compiled NORDOI Rust binary.
Negative tests subsequently mutate the real files or captured receipts:
the native engine is NEVER mocked and no scientific simulation occurs.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
SOURCE = REPO / "examples/apps/app003_integrated_quality/src"
EXE = Path(os.environ.get("NORDOI_APP006_EXE", str(
    REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
)))
sys.path.insert(0, str(HERE))
import audit_app006 as audit

def request(code: int) -> bytes:
    return json.dumps({"schema": "NORDOI_AGENT_REQUEST_V1", "key_code": code},
                      separators=(",", ":")).encode("utf-8")

def run_native(root: Path, codes=(73, 59), source=SOURCE, raw_input=None):
    lines = raw_input if raw_input is not None else b"".join(request(x)+b"\n" for x in codes)
    proc = subprocess.run(
        [str(EXE.resolve(strict=True)), "agent-batch-jsonl", str(source), "app.main",
         "--grant-output-dir", str(root)],
        input=lines, cwd=REPO, capture_output=True, timeout=120,
    )
    return proc

class ReadOnlyAgentBatchAudit(unittest.TestCase):
    def setUp(self):
        self.working = tempfile.TemporaryDirectory(prefix="nordoi_app006_real_")
        self.base = Path(self.working.name)
        self.root = self.base / "published"
        self.root.mkdir()
        self.receipts = self.base / "original-native.jsonl"
        self.original = run_native(self.root)
        self.assertEqual(self.original.returncode, 0,
                         self.original.stderr.decode("utf-8", "replace"))
        self.receipts.write_bytes(self.original.stdout)

    def tearDown(self):
        self.working.cleanup()

    def check(self):
        process = subprocess.run(
            [sys.executable, "-B", str(HERE/"audit_app006.py"),
             "--batch-root", str(self.root), "--receipts", str(self.receipts)],
            cwd=REPO, capture_output=True, timeout=30,
        )
        result = json.loads(process.stdout.decode("utf-8", errors="strict"))
        return process, result

    def tamper(self, transform):
        data = self.receipts.read_text(encoding="utf-8").splitlines()
        records = [json.loads(x) for x in data]
        transform(records)
        self.receipts.write_text(
            "".join(json.dumps(x, ensure_ascii=True, separators=(",",":")) + "\n"
                    for x in records), encoding="utf-8",
        )

    def rejected(self, reason):
        p,report = self.check()
        self.assertEqual(p.returncode,2,p.stderr)
        self.assertEqual(report["schema"],audit.AUDIT_SCHEMA)
        self.assertEqual(report["status"],"REJECTED")
        self.assertEqual(report["error_code"],reason)
        self.assertFalse(report["verified"])

    def test_01_real_native_NORDOI_binary_exists(self):
        self.assertTrue(EXE.is_file())

    def test_02_unmodified_real_two_job_batch_passes(self):
        p,r = self.check()
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertEqual(r["status"],"PASS")
        self.assertEqual(r["committed_jobs_verified"],2)
        self.assertEqual([x["key_code"] for x in r["jobs"]],[73,59])
        self.assertEqual([x["decision"] for x in r["jobs"]],[1,0])
        self.assertEqual(len(r["jobs"][0]["files"]),2)

    def test_03_audit_stdout_is_exactly_one_utf8_json_line(self):
        p,_ = self.check()
        self.assertEqual(p.returncode,0)
        self.assertEqual(p.stdout.count(b"\n"),1)
        self.assertEqual(p.stderr,b"")

    def test_04_audit_is_read_only_for_actual_native_bytes(self):
        before = {x.relative_to(self.base).as_posix():x.read_bytes()
                  for x in self.base.rglob("*") if x.is_file()}
        p,_ = self.check()
        self.assertEqual(p.returncode,0)
        after = {x.relative_to(self.base).as_posix():x.read_bytes()
                 for x in self.base.rglob("*") if x.is_file()}
        self.assertEqual(before,after)

    def test_05_audit_report_is_deterministic(self):
        a,x = self.check()
        b,y = self.check()
        self.assertEqual(a.returncode,0)
        self.assertEqual(b.returncode,0)
        self.assertEqual(a.stdout,b.stdout)
        self.assertEqual(x,y)

    def test_06_actual_file_sha256_is_computed_not_native_proof_mislabel(self):
        import hashlib
        p,report=self.check()
        self.assertEqual(p.returncode,0)
        path=self.root/"job-0001/quality-decision/decision.txt"
        digest=hashlib.sha256(path.read_bytes()).hexdigest()
        obs=next(x for x in report["jobs"][0]["files"] if x["name"]=="decision.txt")
        self.assertEqual(obs["observed_sha256"],digest)
        self.assertNotIn("verified_signature",report)

    def test_07_native_receipts_sha256_is_of_real_original_jsonl(self):
        import hashlib
        p,report=self.check()
        self.assertEqual(p.returncode,0)
        self.assertEqual(report["input_receipts_sha256"],
                         hashlib.sha256(self.original.stdout).hexdigest())

    def test_08_file_byte_change_fails(self):
        path=self.root/"job-0001/quality-decision/decision.txt"
        path.write_bytes(path.read_bytes()+b"TAMPER")
        self.rejected("REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_09_second_job_byte_change_fails(self):
        path=self.root/"job-0002/quality-decision/report.txt"
        path.write_bytes(b"altered")
        self.rejected("REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_10_missing_file_fails(self):
        (self.root/"job-0001/quality-decision/report.txt").unlink()
        self.rejected("BUNDLE_FILE_SET_CHANGED")

    def test_11_extra_file_fails(self):
        (self.root/"job-0002/quality-decision/other.txt").write_bytes(b"stray")
        self.rejected("BUNDLE_FILE_SET_CHANGED")

    def test_12_missing_whole_job_fails(self):
        shutil.rmtree(self.root/"job-0001")
        self.rejected("MISSING_OR_SYMLINKED_JOB_DIRECTORY")

    def test_13_extra_job_fails(self):
        (self.root/"job-0003").mkdir()
        self.rejected("BATCH_ROOT_UNEXPECTED_FILES_OR_JOBS")

    def test_14_extra_root_file_fails(self):
        (self.root/"unrelated.txt").write_bytes(b"stale")
        self.rejected("BATCH_ROOT_UNEXPECTED_FILES_OR_JOBS")

    def test_15_changed_native_decision_with_original_files_fails(self):
        self.tamper(lambda r:r[0].__setitem__("decision",0))
        self.rejected("COMMIT_BRANCH_CONTRADICTION")

    def test_16_wrong_native_bundle_label_fails(self):
        self.tamper(lambda r:r[0].__setitem__("bundle","elsewhere"))
        self.rejected("COMMIT_BUNDLE_INVALID")

    def test_17_native_authority_ambient_escalation_fails(self):
        self.tamper(lambda r:r[0]["authority"].__setitem__("ambient","ALL"))
        self.rejected("COMMIT_AUTHORITY_INVALID")

    def test_18_fake_crash_durability_claim_fails(self):
        self.tamper(lambda r:r[0].__setitem__("crash_durability","GUARANTEED"))
        self.rejected("COMMIT_CRASH_CLAIM_INVALID")

    def test_19_wrong_native_receipt_size_field_fails(self):
        self.tamper(lambda r:r[0]["files"][0].__setitem__("bytes",1))
        self.rejected("FILE_RECEIPT_SIZE_CONTRADICTION")

    def test_20_bad_receipt_proof_shape_fails(self):
        self.tamper(lambda r:r[0]["files"][0].__setitem__("proof_sha256","pretend"))
        self.rejected("FILE_P27_PROOF_SHAPE_INVALID")

    def test_21_bad_bundle_receipt_sha_shape_fails(self):
        self.tamper(lambda r:r[1].__setitem__("bundle_receipt_sha256","abc"))
        self.rejected("COMMIT_P27_RECEIPT_INVALID")

    def test_22_forged_file_name_fails(self):
        self.tamper(lambda r:r[0]["files"][0].__setitem__("name","../escape"))
        self.rejected("FILE_RECEIPT_NAME_INVALID")

    def test_23_duplicate_file_receipt_name_fails(self):
        self.tamper(lambda r:r[0]["files"][1].__setitem__(
            "name",r[0]["files"][0]["name"]))
        self.rejected("FILE_RECEIPT_NAME_INVALID")

    def test_24_unknown_receipt_field_fails(self):
        self.tamper(lambda r:r[0].__setitem__("admin_override",True))
        self.rejected("COMMIT_FIELDS_INVALID")

    def test_25_invalid_receipt_json_fails(self):
        self.receipts.write_bytes(b"not json\n")
        self.rejected("INVALID_RECEIPT_JSON")

    def test_26_duplicate_raw_json_key_fails(self):
        raw=self.receipts.read_text(encoding="utf-8")
        raw=raw.replace('"status":"COMMITTED"',
                        '"status":"COMMITTED","status":"COMMITTED"',1)
        self.receipts.write_text(raw,encoding="utf-8")
        self.rejected("DUPLICATE_RECEIPT_FIELD")

    def test_27_receipt_missing_final_newline_fails(self):
        self.receipts.write_bytes(self.receipts.read_bytes().rstrip(b"\n"))
        self.rejected("RECEIPTS_NOT_NEWLINE_TERMINATED")

    def test_28_receipt_invalid_utf8_fails(self):
        self.receipts.write_bytes(b"\xff\n")
        self.rejected("RECEIPTS_NOT_UTF8")

    def test_29_receipts_symlink_fails_if_os_allows(self):
        real=self.base/"moved.jsonl"
        self.receipts.rename(real)
        try:self.receipts.symlink_to(real)
        except (OSError,NotImplementedError):
            self.skipTest("Creating symlink denied by OS")
        self.rejected("RECEIPTS_NOT_BOUNDED_REGULAR_FILE")

    def test_30_file_symlink_fails_if_os_allows(self):
        target=self.root/"job-0001/quality-decision/decision.txt"
        elsewhere=self.base/"outside-file"
        elsewhere.write_bytes(target.read_bytes())
        target.unlink()
        try:target.symlink_to(elsewhere)
        except (OSError,NotImplementedError):
            self.skipTest("Creating symlink denied by OS")
        self.rejected("FILE_MISSING_SYMLINK_OR_OVERSIZED")

    def test_31_root_symlink_fails_if_os_allows(self):
        sibling=self.base/"symlink-to-root"
        try:sibling.symlink_to(self.root,target_is_directory=True)
        except (OSError,NotImplementedError):
            self.skipTest("Creating symlink denied by OS")
        try:audit.audit(sibling,self.receipts)
        except audit.AuditRejected as error:
            self.assertEqual(error.code,"BATCH_ROOT_NOT_REAL_DIRECTORY")
        else:
            self.fail("symlinked root should not be trusted")

    def test_32_partial_real_native_failure_reports_partial_not_pass(self):
        root=self.base/"sources"
        shutil.copytree(SOURCE,root)
        rules=root/"app/rules.noi"
        content=rules.read_text(encoding="utf-8")
        self.assertIn("else { 0 }",content)
        rules.write_text(content.replace("else { 0 }","else { 2 }"),
                         encoding="utf-8")
        shutil.rmtree(self.root)
        self.root.mkdir()
        result=run_native(self.root,source=root)
        self.assertNotEqual(result.returncode,0)
        self.receipts.write_bytes(result.stdout)
        p,report=self.check()
        self.assertEqual(p.returncode,3,p.stderr)
        self.assertEqual(report["status"],"PARTIAL")
        self.assertEqual(report["committed_jobs_verified"],1)
        self.assertEqual(report["native_error_code"],"NATIVE_OPERATION_FAILED")
        self.assertTrue((self.root/"job-0001/quality-decision/decision.txt").is_file())

    def test_33_native_prevalidation_failure_reports_zero_commits(self):
        shutil.rmtree(self.root)
        self.root.mkdir()
        raw=request(73)+b"\n{not-json}\n"
        result=run_native(self.root,raw_input=raw)
        self.assertNotEqual(result.returncode,0)
        self.receipts.write_bytes(result.stdout)
        p,report=self.check()
        self.assertEqual(p.returncode,3)
        self.assertEqual(report["status"],"ERROR")
        self.assertEqual(report["committed_jobs_verified"],0)
        self.assertEqual(list(self.root.iterdir()),[])

    def test_34_partial_native_error_cannot_be_forged_as_pass(self):
        self.tamper(lambda r:r.append({
            "schema":audit.NATIVE_SCHEMA,"status":"ERROR",
            "error_code":"NATIVE_OPERATION_FAILED","exit_code":5,
            "publication":"UNKNOWN"
        }))
        p,report=self.check()
        self.assertEqual(p.returncode,3)
        self.assertEqual(report["status"],"PARTIAL")
        self.assertEqual(report["committed_jobs_verified"],2)

    def test_35_error_record_before_success_fails(self):
        self.tamper(lambda r:r.insert(0,{
            "schema":audit.NATIVE_SCHEMA,"status":"ERROR",
            "error_code":"INVALID_REQUEST_JSON","exit_code":4,
            "publication":"UNKNOWN"
        }))
        self.rejected("ERROR_MUST_TERMINATE_BATCH")

    def test_36_bad_error_false_no_publication_claim_fails(self):
        self.tamper(lambda r:r.append({
            "schema":audit.NATIVE_SCHEMA,"status":"ERROR",
            "error_code":"NATIVE_OPERATION_FAILED","exit_code":5,
            "publication":"NONE"
        }))
        self.rejected("ERROR_CLAIMS_FALSE_PUBLICATION")

    def test_37_wrong_extra_partial_job_content_fails(self):
        self.tamper(lambda r:r.append({
            "schema":audit.NATIVE_SCHEMA,"status":"ERROR",
            "error_code":"NATIVE_OPERATION_FAILED","exit_code":5,
            "publication":"UNKNOWN"
        }))
        p=self.root/"job-0003"
        p.mkdir()
        (p/"unwanted.txt").write_bytes(b"tamper")
        self.rejected("FAILED_JOB_LEFT_UNEXPECTED_FILES")

    def test_38_missing_receipts_fails(self):
        self.receipts.unlink()
        self.rejected("RECEIPTS_NOT_BOUNDED_REGULAR_FILE")

    def test_39_real_one_job_native_passes(self):
        shutil.rmtree(self.root)
        self.root.mkdir()
        r=run_native(self.root,codes=(73,))
        self.assertEqual(r.returncode,0)
        self.receipts.write_bytes(r.stdout)
        p,report=self.check()
        self.assertEqual(p.returncode,0)
        self.assertEqual(report["committed_jobs_verified"],1)

    def test_40_no_network_no_engine_reexecution_no_filewrite(self):
        source=(HERE/"audit_app006.py").read_text(encoding="utf-8")
        self.assertNotIn("subprocess",source)
        self.assertNotIn("requests.",source)
        self.assertNotIn("create_dir(",source)
        self.assertIn("READ_ONLY_CONTENT_CONSISTENCY_NOT_AUTHENTICATED_SIGNATURE",
                      source)

if __name__ == "__main__":
    unittest.main()
