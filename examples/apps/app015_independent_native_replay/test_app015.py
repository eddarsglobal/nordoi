"""APP-015: real Rust replay of portable, checkpointed agent DAGs on four OS.

All valid fixtures originate in genuine APP-012 native Rust executions,
sealed through APP-013 and packed via APP-014. Adversarial tests alter those
real bytes or pin separately supplied engine/source changes.
No fake native rule-engine output, human signatures or production admission.
"""
from __future__ import annotations

import hashlib
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
ENGINE = REPO / "target/debug" / ("nordoi.exe" if os.name == "nt" else "nordoi")
sys.path.insert(0, str(HERE))
import replay_app015 as qualifier
sys.path.insert(0, str(HERE.parent / "app012_clean_checkpoint"))
import run_app012 as pause


def node(code: int, deps=None, gate="always"):
    return {"key_code": code, "depends_on": [] if deps is None else deps,
            "gate": gate}


class IndependentPinnedNativeReplay(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="nordoi_app015_real_")
        self.base = Path(self.tmp.name)
        self.native = self.base / "original-native"
        self.handoff = self.base / "real-app013-handoff"
        self.portable = self.base / "genuine-app014-portable"
        self.restored = self.base / "fresh-native-root"
        self.restored.mkdir()
        self.plan = self.base / "original-plan.json"
        self.prefix = self.base / "original-prefix.jsonl"
        self.engine = ENGINE
        self.source = SOURCE
        self.engine_pin = pause.pinned.engine_sha256(self.engine)
        self.source_pin = pause.pinned.source_tree_sha256(self.source)
        self.build([node(73), node(59), node(60,[1,2],"all_committed"),
                    node(80,[3],"all_approved")], stop=2)

    def tearDown(self):
        self.tmp.cleanup()

    def execute(self, args, timeout=180):
        p = subprocess.run([sys.executable, "-B", *map(str, args)],
                           cwd=REPO, capture_output=True, timeout=timeout)
        try:
            rows = [json.loads(l) for l in p.stdout.splitlines()]
        except (UnicodeError, ValueError) as error:
            self.fail(f"non-json native output: {p.stdout!r} {p.stderr!r}: {error}")
        return p, rows

    def build(self, nodes, stop):
        if self.native.exists():
            shutil.rmtree(self.native)
        if self.handoff.exists():
            shutil.rmtree(self.handoff)
        if self.portable.exists():
            shutil.rmtree(self.portable)
        self.native.mkdir()
        self.plan.write_text(json.dumps(
            {"schema":"NORDOI_AGENT_DAG_V1", "nodes":nodes},
            separators=(",",":"),ensure_ascii=True),encoding="utf-8")
        self.plan_pin = hashlib.sha256(self.plan.read_bytes()).hexdigest()
        proc,rows = self.execute([
            HERE.parent/"app012_clean_checkpoint/run_app012.py",
            "--plan",self.plan,"--engine",self.engine,
            "--source-root",self.source,"--grant-output-dir",self.native,
            "--expected-plan-sha256",self.plan_pin,
            "--expected-engine-sha256",self.engine_pin,
            "--expected-source-sha256",self.source_pin,
            "--stop-after",str(stop)], 155)
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.prefix.write_bytes(proc.stdout)
        self.assertEqual(len(rows),stop)
        seal,rows = self.execute([
            HERE.parent/"app013_verified_handoff/handoff_app013.py","seal",
            "--bundle-dir",self.handoff,"--native-root",self.native,
            "--plan",self.plan,"--prefix-events",self.prefix,
            "--expected-plan-sha256",self.plan_pin])
        self.assertEqual(seal.returncode,0,(rows,seal.stderr))
        self.inner_pin=rows[0]["manifest_sha256"]
        pkg,rows = self.execute([
            HERE.parent/"app014_portable_checkpoint/portable_app014.py","pack",
            "--portable-dir",self.portable,"--handoff-dir",self.handoff,
            "--native-root",self.native,
            "--expected-handoff-sha256",self.inner_pin])
        self.assertEqual(pkg.returncode,0,(rows,pkg.stderr))
        self.portable_pin=rows[0]["portable_sha256"]

    def command(self,action,*,pin=None,engine_pin=None,source_pin=None,
                engine=None,source=None,root=None,portable_dir=None):
        return [
            HERE/"replay_app015.py",action,
            "--portable-dir",self.portable if portable_dir is None else portable_dir,
            "--expected-portable-sha256",self.portable_pin if pin is None else pin,
            "--engine",self.engine if engine is None else engine,
            "--source-root",self.source if source is None else source,
            "--expected-engine-sha256",self.engine_pin if engine_pin is None else engine_pin,
            "--expected-source-sha256",self.source_pin if source_pin is None else source_pin,
        ] + ([] if action=="verify-replay" else [
            "--restore-root",self.restored if root is None else root
        ])

    def run_replay(self,action="verify-replay",**kw):
        return self.execute(self.command(action,**kw),timeout=180)

    def passing(self,action="verify-replay",**kw):
        p,rows = self.run_replay(action,**kw)
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        if action!="resume-replay":
            self.assertEqual(len(rows),1)
        return p,rows

    def rejected(self,code,action="verify-replay",**kw):
        p,rows = self.run_replay(action,**kw)
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"REJECTED")
        self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")

    def native_files(self,root):
        return {p.relative_to(root).as_posix():p.read_bytes()
                for p in root.rglob("*") if p.is_file()}

    def mutate_original_rule(self,broken=False):
        copied=self.base/"alternative-source"
        shutil.copytree(SOURCE,copied)
        rules=copied/"app/rules.noi"
        current=rules.read_text(encoding="utf-8")
        self.assertIn("score >= 60",current)
        if broken:
            current="this is not valid Nordoi source syntax ???\n"
        else:
            current=current.replace("score >= 60","score >= 1000")
        rules.write_text(current,encoding="utf-8")
        return copied, pause.pinned.source_tree_sha256(copied)

    def test_01_two_authentic_native_jobs_are_recomputed(self):
        _,r=self.passing()
        self.assertEqual(r[0]["native_jobs_recomputed"],2)

    def test_02_real_native_files_compared_byte_for_byte(self):
        _,r=self.passing()
        self.assertEqual(r[0]["native_files_bytewise_equal"],4)

    def test_03_entire_native_receipts_checked_not_only_decision(self):
        _,r=self.passing()
        self.assertTrue(r[0]["native_receipts_exactly_equal"])

    def test_04_both_engine_and_source_pins_recorded(self):
        _,r=self.passing()
        self.assertEqual(r[0]["engine_sha256"],self.engine_pin)
        self.assertEqual(r[0]["source_tree_sha256"],self.source_pin)

    def test_05_replay_is_private_and_keeps_portable_bytes_unchanged(self):
        before=self.native_files(self.portable)
        self.passing()
        self.assertEqual(self.native_files(self.portable),before)

    def test_06_original_native_source_files_remain_unchanged(self):
        before=self.native_files(self.native)
        self.passing()
        self.assertEqual(self.native_files(self.native),before)

    def test_07_checkpoints_remain_readonly_app014_verifiable(self):
        self.passing()
        manifest,_,_=qualifier.portable.verify(self.portable,self.portable_pin)
        self.assertEqual(manifest["committed"],2)

    def test_08_repeated_replay_is_deterministic(self):
        a=self.passing()[1][0]
        b=self.passing()[1][0]
        self.assertEqual(a,b)

    def test_09_externally_pinned_portable_digest_in_report(self):
        _,r=self.passing()
        self.assertEqual(r[0]["portable_sha256"],self.portable_pin)

    def test_10_replay_makes_no_external_signature_claim(self):
        _,r=self.passing()
        self.assertEqual(r[0]["external_pin_authentication"],"NOT_PERFORMED")
        self.assertEqual(r[0]["native_engine_sandbox"],"NOT_PROVIDED")

    def test_11_replay_does_not_claim_independent_p27_cryptography(self):
        _,r=self.passing()
        self.assertEqual(r[0]["p27_assurance"],
            "EXACT_PINNED_NATIVE_REPRODUCTION_NOT_SEPARATE_CRYPTO")

    def test_12_qualified_restore_of_historical_native_files(self):
        _,r=self.passing("restore-replay")
        self.assertEqual(r[0]["status"],"RESTORED")
        self.assertEqual(r[0]["replay_qualification"],"PASS_EXACT_NATIVE_RUST_REPLAY")

    def test_13_qualified_restored_file_bytes_match_original(self):
        self.passing("restore-replay")
        self.assertEqual(self.native_files(self.restored),self.native_files(self.native))

    def test_14_qualified_resume_runs_real_missing_dag_jobs(self):
        self.passing("restore-replay")
        p,r=self.passing("resume-replay")
        self.assertEqual([e["step"] for e in r],[3,4])
        self.assertEqual([e["status"] for e in r],["COMMITTED","COMMITTED"])

    def test_15_independent_app010_full_dag_passes_after_qualified_resume(self):
        self.passing("restore-replay")
        p,_=self.passing("resume-replay")
        all_events=self.base/"qualified-full-dag.jsonl"
        all_events.write_bytes(self.prefix.read_bytes()+p.stdout)
        report,status=qualifier.independent.audit(
            self.restored,self.plan,all_events,self.plan_pin)
        self.assertEqual(status,0,report)
        self.assertEqual(report["committed_jobs_verified"],4)

    def test_16_previous_restored_jobs_not_overwritten_on_resume(self):
        self.passing("restore-replay")
        historical=self.native_files(self.restored)
        self.passing("resume-replay")
        self.assertEqual({k:(self.restored/k).read_bytes() for k in historical},
                         historical)

    def test_17_real_one_job_checkpoint_is_replayed(self):
        self.build([node(73),node(59),node(60)],1)
        _,r=self.passing()
        self.assertEqual(r[0]["native_jobs_recomputed"],1)

    def test_18_three_job_checkpoint_is_replayed(self):
        self.build([node(73),node(59),node(60),node(80)],3)
        _,r=self.passing()
        self.assertEqual(r[0]["native_jobs_recomputed"],3)

    def test_19_seven_job_checkpoint_is_replayed(self):
        self.build([node(i) for i in (0,20,59,60,61,73,90,100)],7)
        _,r=self.passing()
        self.assertEqual(r[0]["native_jobs_recomputed"],7)

    def test_20_skipped_dependency_is_never_replayed_as_committed(self):
        self.build([node(59),node(73,[1],"all_approved"),node(73)],2)
        _,r=self.passing()
        self.assertEqual(r[0]["native_jobs_recomputed"],1)
        self.assertEqual(r[0]["skipped_nodes_independently_gated"],1)

    def test_21_skipped_historical_branch_then_real_qualified_resume(self):
        self.build([node(59),node(73,[1],"all_approved"),node(73)],2)
        self.passing("restore-replay")
        p,r=self.passing("resume-replay")
        self.assertEqual([x["step"] for x in r],[3])
        self.assertEqual(r[0]["status"],"COMMITTED")

    def test_22_wrong_portable_sha_is_denied_before_engine_execution(self):
        self.rejected("PORTABLE_EXTERNAL_PIN_MISMATCH",pin="0"*64)

    def test_23_wrong_engine_sha_is_denied(self):
        self.rejected("ENGINE_PIN_MISMATCH",engine_pin="0"*64)

    def test_24_wrong_source_sha_is_denied(self):
        self.rejected("SOURCE_PIN_MISMATCH",source_pin="0"*64)

    def test_25_invalid_engine_pin_format_is_denied(self):
        self.rejected("ENGINE_SOURCE_PIN_INVALID",engine_pin="not-a-sha")

    def test_26_invalid_source_pin_format_is_denied(self):
        self.rejected("ENGINE_SOURCE_PIN_INVALID",source_pin="not-a-sha")

    def test_27_modified_portable_manifest_denied_before_engine(self):
        f=self.portable/"portable.json"
        f.write_bytes(f.read_bytes()+b"tamper")
        self.rejected("PORTABLE_EXTERNAL_PIN_MISMATCH")

    def test_28_modified_native_p27_report_denied_before_engine(self):
        f=self.portable/"native/job-0001/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"tamper")
        self.rejected("REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_29_missing_native_p27_decision_file_denied(self):
        (self.portable/"native/job-0002/quality-decision/decision.txt").unlink()
        self.rejected("BUNDLE_FILE_SET_CHANGED")

    def test_30_extra_native_p27_file_denied(self):
        (self.portable/"native/job-0001/quality-decision/extra").write_bytes(b"x")
        self.rejected("BUNDLE_FILE_SET_CHANGED")

    def test_31_missing_outer_seal_denied(self):
        (self.portable/"PORTABLE.sha256").unlink()
        self.rejected("PORTABLE_FILE_SET_CHANGED")

    def test_32_disallowed_native_job_denied(self):
        (self.portable/"native/job-0003").mkdir()
        self.rejected("ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_33_changed_pinned_but_valid_native_rule_refused_by_replay(self):
        source,sha=self.mutate_original_rule()
        self.rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH",
                      source=source,source_pin=sha)

    def test_34_invalid_native_rule_produces_inconclusive_not_false_pass(self):
        source,sha=self.mutate_original_rule(broken=True)
        p,r=self.run_replay(source=source,source_pin=sha)
        self.assertEqual(p.returncode,3,(r,p.stderr))
        self.assertEqual(r[0]["status"],"INCONCLUSIVE")
        self.assertEqual(r[0]["publication"],"UNKNOWN")

    def test_35_failed_replay_never_restores_files(self):
        source,sha=self.mutate_original_rule()
        self.rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH",
                      "restore-replay",source=source,source_pin=sha)
        self.assertFalse(list(self.restored.iterdir()))

    def test_36_restore_nonempty_root_refused_without_overwrite(self):
        (self.restored/"existing").write_bytes(b"preserve")
        self.rejected("RESTORE_ROOT_NOT_EMPTY","restore-replay")
        self.assertEqual((self.restored/"existing").read_bytes(),b"preserve")

    def test_37_second_restore_after_success_refused(self):
        self.passing("restore-replay")
        prior=self.native_files(self.restored)
        self.rejected("RESTORE_ROOT_NOT_EMPTY","restore-replay")
        self.assertEqual(self.native_files(self.restored),prior)

    def test_38_resume_missing_historical_restore_rejected(self):
        self.rejected("MISSING_OR_SYMLINKED_JOB_DIRECTORY","resume-replay")
        self.assertFalse((self.restored/"job-0003").exists())

    def test_39_changed_restored_historical_file_blocks_qualified_resume(self):
        self.passing("restore-replay")
        f=self.restored/"job-0001/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"wrong")
        self.rejected("REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT","resume-replay")
        self.assertFalse((self.restored/"job-0003").exists())

    def test_40_second_resume_after_success_is_fail_closed(self):
        self.passing("restore-replay")
        self.passing("resume-replay")
        self.rejected("ROOT_UNEXPECTED_JOB_OR_FILE","resume-replay")


if __name__ == "__main__":
    unittest.main()
