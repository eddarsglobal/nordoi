"""APP-024: actual NORDOI Rust checkpoint/portable/concurrent resume tests.

Every positive checkpoint is produced by NORDOI native Rust via APP-012, sealed
by APP-013, packaged/restored by APP-014. No simulated decisions/signatures.
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

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
sys.path.insert(0,str(HERE.parent/"app014_portable_checkpoint"))
import test_app014 as legacy
sys.path.insert(0,str(HERE))
import run_app024 as app
ENGINE=legacy.ENGINE
SOURCE=legacy.SOURCE


def node(code,deps=None,gate="always"):
    return legacy.node(code,deps,gate)


DEFAULT=[
    node(73),node(80),
    node(73),node(59),
    node(60,[3,4],"all_committed"),
    node(80,[5],"all_approved")
]


class RealPortableConcurrentResume(unittest.TestCase):
    # Reuse only real cross-verified checkpoint fixture METHODS, not inherited
    # test methods, so the suite contains exactly 24 APP-024 cases.
    create=legacy.PortableNativeEvidence.create
    call=legacy.PortableNativeEvidence.call
    packed=legacy.PortableNativeEvidence.packed
    restored_ok=legacy.PortableNativeEvidence.restored_ok
    native_bytes=legacy.PortableNativeEvidence.native_bytes
    rewrite_outer=legacy.PortableNativeEvidence.rewrite_outer

    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi_app024_native_")
        self.base=Path(self.temp.name)
        self.native=self.base/"actual-original-rust"
        self.native.mkdir()
        self.sealed=self.base/"verified-app013"
        self.bundle=self.base/"portable-app014"
        self.restored=self.base/"restored-real-rust"
        self.restored.mkdir()
        self.plan=self.base/"original-plan.json"
        self.events=self.base/"original-prefix.jsonl"
        self.engine=ENGINE
        self.source=SOURCE
        self.engine_pin=app.pinned.engine_sha256(self.engine)
        self.source_pin=app.pinned.source_tree_sha256(self.source)
        self.create(DEFAULT,stop=2)
        self.packed()
        self.restored_ok()
        self.historic=self.native_bytes(self.restored)

    def tearDown(self):
        self.temp.cleanup()

    def launch(self,*,workers=2,pin=None,root=None,source=None,
               source_pin=None,engine_pin=None,checkpoint=None):
        cmd=[sys.executable,"-B",str(HERE/"run_app024.py"),
            "--portable-dir",str(self.bundle if checkpoint is None else checkpoint),
            "--expected-portable-sha256",
              self.outer_pin if pin is None else pin,
            "--restore-root",str(self.restored if root is None else root),
            "--engine",str(self.engine),
            "--source-root",str(self.source if source is None else source),
            "--expected-engine-sha256",
              self.engine_pin if engine_pin is None else engine_pin,
            "--expected-source-sha256",
              self.source_pin if source_pin is None else source_pin,
            "--max-in-flight",str(workers)]
        proc=subprocess.run(cmd,cwd=REPO,capture_output=True,timeout=220)
        try:
            rows=[json.loads(x) for x in proc.stdout.splitlines()]
        except (ValueError,UnicodeError) as exc:
            self.fail(f"NON_JSON:{exc}:{proc.stdout!r}:{proc.stderr!r}")
        return proc,rows

    def good(self,expected=None,**kw):
        p,rows=self.launch(**kw)
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        if expected is None:
            expected=["COMMITTED"]*4
        self.assertEqual([x["status"] for x in rows],expected)
        self.assertEqual([x["step"] for x in rows],
                         list(range(3,3+len(rows))))
        self.assertTrue(all(x["schema"]==app.EVENT for x in rows))
        return rows

    def denied(self,code=None,**kw):
        p,rows=self.launch(**kw)
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(len(rows),1,rows)
        self.assertEqual(rows[0]["status"],"REJECTED")
        if code is not None:
            self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")
        self.assertEqual(self.native_bytes(self.restored),self.historic)

    def full_audit(self,suffix):
        checkpoint=self.bundle/"handoff"
        original=(checkpoint/"native-prefix.jsonl").read_bytes()
        path=self.base/"independent-full.jsonl"
        path.write_bytes(original+b"".join(app.json_line(x) for x in suffix))
        report,code=app.auditor.audit(
            self.restored,checkpoint/"plan.json",path,
            json.loads((self.bundle/"portable.json").read_bytes())["plan_sha256"])
        self.assertEqual(code,0,report)
        self.assertEqual(report["status"],"PASS")
        return report

    def test_01_real_parallel_native_checkpoint_resume(self):
        rows=self.good()
        self.assertEqual([x["decision"] for x in rows],[1,0,1,1])

    def test_02_only_missing_suffix_not_historical_replay(self):
        rows=self.good()
        self.assertEqual([x["step"] for x in rows],[3,4,5,6])
        self.assertEqual(len(self.historic),4)
        self.assertEqual(len(self.native_bytes(self.restored)),12)

    def test_03_historical_native_bytes_remain_unchanged(self):
        self.good()
        now=self.native_bytes(self.restored)
        self.assertTrue(all(now[k]==v for k,v in self.historic.items()))

    def test_04_single_and_two_worker_native_results_identical(self):
        first=self.good(workers=1)
        second_root=self.base/"two-workers"
        second_root.mkdir()
        app.portable.restore(self.bundle,self.outer_pin,second_root)
        p,second=self.launch(workers=2,root=second_root)
        self.assertEqual(p.returncode,0,(second,p.stderr))
        self.assertEqual(first,second)

    def test_05_full_independent_app010_audit_passes(self):
        self.assertEqual(self.full_audit(self.good())["events_checked"],6)

    def test_06_real_native_join_uses_verified_parent_decisions(self):
        rows=self.good()
        self.assertEqual(rows[2]["depends_on"],[3,4])
        self.assertEqual(rows[2]["decision"],1)

    def test_07_all_approved_join_skips_after_rejection(self):
        self.create([node(73),node(80),node(73),node(59),
                     node(60,[3,4],"all_approved")],stop=2)
        # The fixture's original portable package must be replaced with a
        # NEW authentic package, never a fabricated changed manifest.
        shutil.rmtree(self.bundle)
        shutil.rmtree(self.restored)
        self.restored.mkdir()
        self.packed()
        self.restored_ok()
        rows=self.good(["COMMITTED","COMMITTED","SKIPPED"])
        self.assertFalse((self.restored/"job-0005").exists())
        self.full_audit(rows)

    def test_08_any_rejected_join_executes(self):
        self.create([node(73),node(80),node(73),node(59),
                     node(60,[3,4],"any_rejected")],stop=2)
        shutil.rmtree(self.bundle)
        shutil.rmtree(self.restored)
        self.restored.mkdir()
        self.packed()
        self.restored_ok()
        rows=self.good(["COMMITTED"]*3)
        self.assertEqual(rows[-1]["decision"],1)

    def test_09_eight_node_bound_runs_only_missing_nodes(self):
        self.create([node(73),node(80),node(73),node(59),
                     node(80),node(73),node(60,[3,4],"all_committed"),
                     node(80,[5,6,7],"all_committed")],stop=2)
        shutil.rmtree(self.bundle)
        shutil.rmtree(self.restored)
        self.restored.mkdir()
        self.packed()
        self.restored_ok()
        rows=self.good(["COMMITTED"]*6)
        self.assertEqual(len(rows),6)

    def test_10_repeated_key_code_distinct_verified_jobs(self):
        self.create([node(73),node(80),node(73),node(73),
                     node(73,[3,4],"all_approved")],stop=2)
        shutil.rmtree(self.bundle)
        shutil.rmtree(self.restored)
        self.restored.mkdir()
        self.packed()
        self.restored_ok()
        self.good(["COMMITTED"]*3)
        self.assertTrue((self.restored/"job-0005").is_dir())

    def test_11_real_p27_files_independently_verified(self):
        for row in self.good():
            verified=app.native_audit.verify_native_record(
                row["native_receipt"],row["step"],self.restored)
            self.assertEqual(verified["decision"],row["decision"])
            self.assertEqual(len(verified["files"]),2)

    def test_12_historical_portable_prefix_untouched(self):
        old=(self.bundle/"handoff/native-prefix.jsonl").read_bytes()
        self.good()
        self.assertEqual((self.bundle/"handoff/native-prefix.jsonl").read_bytes(),old)

    def test_13_wrong_external_pin_denied_before_effect(self):
        self.denied("PORTABLE_EXTERNAL_PIN_MISMATCH",pin="0"*64)

    def test_14_modified_restored_native_file_denied_before_effect(self):
        f=self.restored/"job-0001/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"TAMPER")
        self.historic=self.native_bytes(self.restored)
        self.denied()

    def test_15_extra_unverified_job_blocks_resume(self):
        (self.restored/"job-0003").mkdir()
        self.historic=self.native_bytes(self.restored)
        self.denied()

    def test_16_engine_pin_mismatch_denied(self):
        self.denied("ENGINE_PIN_MISMATCH",engine_pin="0"*64)

    def test_17_source_pin_mismatch_denied(self):
        self.denied("SOURCE_PIN_MISMATCH",source_pin="f"*64)

    def test_18_missing_portable_marker_denied(self):
        (self.bundle/"PORTABLE.sha256").unlink()
        self.denied()

    def test_19_replaced_checkpoint_plan_denied(self):
        f=self.bundle/"handoff/plan.json"
        f.write_bytes(f.read_bytes()+b" ")
        self.denied()

    def test_20_unrestored_directory_denied(self):
        empty=self.base/"no-restoration"
        empty.mkdir()
        self.denied(root=empty)

    def test_21_invalid_worker_count_denied_by_api(self):
        with self.assertRaises(app.portable.Refused) as caught:
            app.run(self.bundle,self.outer_pin,self.restored,
                    self.engine,self.source,self.engine_pin,self.source_pin,3)
        self.assertEqual(caught.exception.code,"APP024_CONCURRENCY_LIMIT_INVALID")
        self.assertEqual(self.native_bytes(self.restored),self.historic)

    def test_22_real_native_error_after_checkpoint_no_child_admitted(self):
        changed=self.base/"actual-modified-noi-source"
        shutil.copytree(self.source,changed)
        rule=changed/"app/rules.noi"
        before=rule.read_text(encoding="utf-8")
        self.assertIn("else { 0 }",before)
        rule.write_text(before.replace("else { 0 }","else { 2 }"),
                        encoding="utf-8")
        pin=app.pinned.source_tree_sha256(changed)
        p,rows=self.launch(source=changed,source_pin=pin)
        self.assertEqual(p.returncode,3,(rows,p.stderr))
        self.assertIn("ERROR",[x["status"] for x in rows])
        self.assertTrue((self.restored/"job-0003/quality-decision/report.txt").is_file())
        self.assertFalse((self.restored/"job-0005").exists())
        self.assertEqual(self.historic,{k:v for k,v in
          self.native_bytes(self.restored).items() if k in self.historic})

    def test_23_replaying_after_clean_success_is_denied(self):
        self.good()
        saved=self.native_bytes(self.restored)
        p,rows=self.launch()
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(self.native_bytes(self.restored),saved)

    def test_24_checkpoint_copied_to_new_portable_location(self):
        copied=self.base/"copied-checkpoint"
        shutil.copytree(self.bundle,copied)
        rows=self.good(checkpoint=copied)
        self.assertEqual(len(rows),4)


if __name__=="__main__":
    unittest.main()
