"""APP-005 native bounded JSONL batch real engine tests — no mock/simulation.

Every behavioral scenario invokes the genuine Rust NORDOI binary and checks
real files, byte-level JSON, capability denial and partial publication.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE=REPO/"examples/apps/app003_integrated_quality/src"
sys.path.insert(0,str(REPO/"examples/apps/app004_agent_json"))
import run_app004

EXE=Path(os.environ.get("NORDOI_APP005_EXE",
    str(REPO/"target/debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))
SCHEMA="NORDOI_AGENT_REQUEST_V1"

def request(score):
    return json.dumps({"schema":SCHEMA,"key_code":score},
                      separators=(",",":")).encode("utf-8")

def batch(*scores):
    return b"\n".join(request(s) for s in scores)+b"\n"

def actual(payload,root=None,grant=None,module="app.main",other=()):
    cmd=[str(EXE.resolve(strict=True)),"agent-batch-jsonl",
         str(root if root is not None else SOURCE),module]
    if grant is not None:
        cmd+=["--grant-output-dir",str(grant)]
    cmd+=list(other)
    p=subprocess.run(cmd,input=payload,cwd=REPO,capture_output=True,timeout=120)
    out=p.stdout.decode("utf-8",errors="strict")
    records=[json.loads(x) for x in out.splitlines()]
    return p,records,p.stderr.decode("utf-8",errors="replace")

class NORDOIRealAgentJsonlBatch(unittest.TestCase):
    def test_01_real_compiled_nordoi_binary(self):
        self.assertTrue(EXE.is_file(),EXE)

    def test_02_native_help_declares_batch(self):
        p=subprocess.run([str(EXE),"--help"],capture_output=True,timeout=20)
        self.assertEqual(p.returncode,0)
        self.assertIn(b"agent-batch-jsonl",p.stdout)

    def real_batch(self,scores):
        with tempfile.TemporaryDirectory(prefix="noi_app005_") as d:
            p,records,stderr=actual(batch(*scores),grant=d)
            self.assertEqual(p.returncode,0,p.stdout.decode()+stderr)
            self.assertEqual(len(records),len(scores))
            self.assertEqual({x.name for x in Path(d).iterdir()},
                             {f"job-{i:04}" for i in range(1,len(scores)+1)})
            for i,(score,record) in enumerate(zip(scores,records),1):
                self.assertEqual(record["status"],"COMMITTED")
                self.assertEqual(record["key_code"],score)
                self.assertEqual(record["decision"],int(score>=60))
                self.assertEqual(record["schema"],run_app004.SCHEMA)
                run_app004.verify_actual_files(Path(d)/f"job-{i:04}",
                                                score,record,stderr)
            return p.stdout

    def test_03_two_opposite_decisions(self):self.real_batch([73,59])
    def test_04_one_request_supported(self):self.real_batch([73])
    def test_05_three_requests_with_threshold(self):self.real_batch([0,60,100])
    def test_06_eight_requests_at_limit(self):self.real_batch([0,20,59,60,61,73,90,100])
    def test_07_repeated_inputs_use_distinct_dirs(self):self.real_batch([73,73,73])
    def test_08_reverse_order(self):self.real_batch([59,73])
    def test_09_per_request_result_bytes_deterministic(self):
        a=self.real_batch([73,59])
        b=self.real_batch([73,59])
        self.assertEqual(a,b)

    def test_10_identical_receipts_to_two_individual_R2_requests(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_") as d:
            p,records,stderr=actual(batch(73,59),grant=d)
            self.assertEqual(p.returncode,0,stderr)
            for i,score in enumerate([73,59],1):
                with tempfile.TemporaryDirectory(prefix="noi_app005_single_") as t:
                    original=subprocess.run(
                        [str(EXE),"agent-request-json",str(SOURCE),"app.main",
                         "--grant-output-dir",t],
                        cwd=REPO,input=request(score),capture_output=True,timeout=120,
                    )
                    self.assertEqual(original.returncode,0,original.stderr)
                    self.assertEqual(
                        p.stdout.splitlines()[i-1],original.stdout.strip()
                    )

    def test_11_missing_grant_denied_before_input_use(self):
        p,records,_=actual(batch(73),grant=None)
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(records[0]["error_code"],"MISSING_EXPLICIT_GRANT")

    def test_12_bad_grant_switch_denied(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_") as d:
            p,records,_=actual(batch(73),other=("--wrong-flag",d))
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"INVALID_GRANT_SYNTAX")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_13_invalid_explicit_parent_denied(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_") as d:
            missing=Path(d)/"not-there"
            p,records,_=actual(batch(73),grant=missing)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"INVALID_EXPLICIT_BATCH_GRANT")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_14_empty_batch_denied(self):self.no_effect_denial(b"","EMPTY_AGENT_BATCH")
    def test_15_empty_middle_line_denied(self):
        self.no_effect_denial(request(73)+b"\n\n"+request(59), "EMPTY_AGENT_BATCH_LINE")
    def test_16_ninth_request_denied(self):
        self.no_effect_denial(batch(*([73]*9)),"TOO_MANY_AGENT_REQUESTS")
    def test_17_oversized_total_input_denied(self):
        self.no_effect_denial(b" "*(8*4097+1),"AGENT_BATCH_TOO_LARGE")
    def test_18_one_giant_line_denied(self):
        self.no_effect_denial(request(73)+b" "*4097,"AGENT_BATCH_LINE_TOO_LARGE")
    def test_19_invalid_second_line_denies_first_without_any_output(self):
        self.no_effect_denial(request(73)+b"\n"+b"{invalid}\n","INVALID_REQUEST_JSON")
    def test_20_invalid_third_line_denies_first_two(self):
        self.no_effect_denial(batch(73,59)+b"not-json\n","INVALID_REQUEST_JSON")
    def test_21_duplicate_score_in_later_line_denied(self):
        b2=b'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":59,"key_code":1}'
        self.no_effect_denial(request(73)+b"\n"+b2,b"".decode() or "DUPLICATE_REQUEST_FIELD")
    def test_22_grant_injected_in_second_json_denied(self):
        evil=b'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":59,"output_dir":"/tmp"}'
        self.no_effect_denial(request(73)+b"\n"+evil,"UNKNOWN_REQUEST_FIELD")
    def test_23_wrong_schema_in_later_line_denied(self):
        self.no_effect_denial(request(73)+b"\n"+b'{"schema":"V2","key_code":59}',
                              "INVALID_REQUEST_SCHEMA")
    def test_24_non_utf8_second_line_denied(self):
        self.no_effect_denial(request(73)+b"\n\xff","INVALID_REQUEST_UTF8")
    def test_25_number_overflow_second_line_denied(self):
        self.no_effect_denial(
            request(73)+b'\n{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":4294967296}',
            "INVALID_REQUEST_KEY_CODE")
    def test_26_missing_request_schema_denied(self):
        self.no_effect_denial(b'{"key_code":73}',"MISSING_REQUEST_FIELD")
    def test_27_trailing_extra_newline_denied(self):
        self.no_effect_denial(batch(73)+b"\n","EMPTY_AGENT_BATCH_LINE")

    def no_effect_denial(self,payload,reason):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            p,records,err=actual(payload,grant=d)
            self.assertNotEqual(p.returncode,0,err)
            self.assertEqual(len(records),1)
            self.assertEqual(records[0]["status"],"ERROR")
            self.assertEqual(records[0]["error_code"],reason)
            self.assertEqual(records[0]["publication"],"UNKNOWN")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_28_refuse_overwrite_of_existing_job_0001_before_any_effect(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            pre=Path(d)/"job-0001"
            pre.mkdir()
            (pre/"sentinel.txt").write_bytes(b"DO_NOT_OVERWRITE")
            p,records,_=actual(batch(73,59),grant=d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"BATCH_TARGET_NOT_FRESH")
            self.assertEqual((pre/"sentinel.txt").read_bytes(),b"DO_NOT_OVERWRITE")
            self.assertFalse((Path(d)/"job-0002").exists())

    def test_29_refuse_existing_job_0002_before_any_effect(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            (Path(d)/"job-0002").mkdir()
            p,records,_=actual(batch(73,59),grant=d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"BATCH_TARGET_NOT_FRESH")
            self.assertFalse((Path(d)/"job-0001").exists())

    def test_30_refuse_job_symlink_if_supported(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            real=Path(d)/"external"
            real.mkdir()
            link=Path(d)/"job-0001"
            try:link.symlink_to(real,target_is_directory=True)
            except (OSError,NotImplementedError):
                self.skipTest("symlink creation unavailable on this runner")
            p,records,_=actual(batch(73),grant=d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"BATCH_TARGET_NOT_FRESH")
            self.assertEqual(list(real.iterdir()),[])

    def test_31_refuse_granted_parent_symlink_if_supported(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            real=Path(d)/"real"
            real.mkdir()
            link=Path(d)/"link"
            try:link.symlink_to(real,target_is_directory=True)
            except (OSError,NotImplementedError):
                self.skipTest("symlink creation unavailable on this runner")
            p,records,_=actual(batch(73),grant=link)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[0]["error_code"],"INVALID_EXPLICIT_BATCH_GRANT")
            self.assertEqual(list(real.iterdir()),[])

    def test_32_invalid_module_does_not_claim_publication(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_denied_") as d:
            p,records,_=actual(batch(73),grant=d,module="app.nope")
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(records[-1]["status"],"ERROR")
            self.assertEqual(records[-1]["publication"],"UNKNOWN")
            self.assertFalse((Path(d)/"job-0001"/"quality-decision").exists())

    def test_33_partial_batch_failure_never_claims_rollback(self):
        # First job commits; second rule gives INT(2). No global rollback.
        with tempfile.TemporaryDirectory(prefix="noi_app005_partial_") as d:
            root=Path(d)/"source"
            shutil.copytree(SOURCE,root)
            original=(root/"app"/"rules.noi").read_text(encoding="utf-8")
            self.assertIn("else { 0 }",original)
            (root/"app"/"rules.noi").write_text(
                original.replace("else { 0 }","else { 2 }"),encoding="utf-8",
            )
            grant=Path(d)/"out"
            grant.mkdir()
            p,records,err=actual(batch(73,59),grant=grant,root=root)
            self.assertNotEqual(p.returncode,0,err)
            self.assertEqual(len(records),2)
            self.assertEqual(records[0]["status"],"COMMITTED")
            self.assertEqual(records[-1]["status"],"ERROR")
            self.assertEqual(records[-1]["publication"],"UNKNOWN")
            self.assertTrue((grant/"job-0001"/"quality-decision"/"decision.txt").is_file())
            self.assertFalse((grant/"job-0002"/"quality-decision").exists())

    def test_34_real_source_threshold_mutation_changes_each_decision(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_mutated_") as d:
            root=Path(d)/"source"
            shutil.copytree(SOURCE,root)
            rule=root/"app"/"rules.noi"
            s=rule.read_text(encoding="utf-8")
            self.assertIn("score >= 60",s)
            rule.write_text(s.replace("score >= 60","score >= 90"),
                            encoding="utf-8")
            out=Path(d)/"out"
            out.mkdir()
            p,records,err=actual(batch(73,95),grant=out,root=root)
            self.assertEqual(p.returncode,0,err)
            self.assertEqual([x["decision"] for x in records],[0,1])

    def test_35_user_agent_client_three_jobs(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_client_") as d:
            p=subprocess.run(
                [sys.executable,str(HERE/"run_app005.py"),"--scores","73","59","60",
                 "--engine",str(EXE),"--output-parent",d],
                cwd=REPO,capture_output=True,timeout=120,
            )
            self.assertEqual(p.returncode,0,p.stderr.decode("utf-8",errors="replace"))
            results=[json.loads(s) for s in p.stdout.decode("utf-8").splitlines()]
            self.assertEqual([x["decision"] for x in results],[1,0,1])
            dirs=list(Path(d).glob("NORDOI_APP005_*/job-*/quality-decision/decision.txt"))
            self.assertEqual(len(dirs),3)

    def test_36_client_invalid_scores_fail_without_directory(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_client_") as d:
            for value in ("101","-1"):
                p=subprocess.run([sys.executable,str(HERE/"run_app005.py"),
                    "--scores","73",value,"--engine",str(EXE),"--output-parent",d],
                    cwd=REPO,capture_output=True,timeout=120)
                self.assertNotEqual(p.returncode,0)
                self.assertEqual(json.loads(p.stdout)["status"],"ERROR")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_37_reject_missing_explicit_grant_no_worktree_changes(self):
        p,records,_=actual(batch(73,59))
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(len(records),1)

    def test_38_actual_jsonl_has_no_host_absolute_paths(self):
        with tempfile.TemporaryDirectory(prefix="noi_app005_") as d:
            p,records,_=actual(batch(73,59),grant=d)
            self.assertEqual(p.returncode,0)
            self.assertNotIn(d,p.stdout.decode("utf-8"))
            self.assertEqual(len(records),2)

    def test_39_source_contract_real_native_only(self):
        src=(REPO/"src/bin/nordoi/agent_batch.rs").read_text(encoding="utf-8")
        self.assertIn("agent_request::parse_agent_request(line)",src)
        self.assertIn("app003::run_app004_agent_json_cli(",src)
        self.assertIn("fs::create_dir(&output)",src)
        self.assertNotIn("std::process::Command",src)
        self.assertNotIn("serde_json",src)

    def test_40_no_human_or_network_required(self):
        text=(HERE/"run_app005.py").read_text(encoding="utf-8")
        self.assertNotIn("score>=60",text.replace(" ",""))
        self.assertNotIn("requests.post",text)
        self.assertIn("sys.dont_write_bytecode=True",text)
        self.assertIn("verify_actual_files",text)

if __name__=="__main__":
    unittest.main()
