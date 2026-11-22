"""APP-013 real native checkpoint handoff and untrusted-bundle tests.

Fixtures use APP-012 to run the genuine compiled NORDOI Rust executable to
publish real P2.7 files and native JSONL. Only adversarial negative tests
modify those original bytes. No mocked or simulated native decision engine.
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
import handoff_app013 as handoff
sys.path.insert(0, str(HERE.parent / "app012_clean_checkpoint"))
import run_app012 as paused


def node(key_code: int, deps=None, gate="always") -> dict:
    return {"key_code": key_code, "depends_on": [] if deps is None else deps,
            "gate": gate}


class RealVerifiedHandoff(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nordoi_app013_real_")
        self.base = Path(self.temp.name)
        self.root = self.base / "real-native-output"
        self.root.mkdir()
        self.bundle = self.base / "untrusted-handoff-bundle"
        self.plan = self.base / "original-native-plan.json"
        self.prefix = self.base / "original-native-events.jsonl"
        self.source = SOURCE
        self.engine = ENGINE
        self.engine_sha = paused.pinned.engine_sha256(self.engine)
        self.source_sha = paused.pinned.source_tree_sha256(self.source)
        self.stop = 2
        self.build([node(73),node(59),node(60,[1,2],"all_committed"),
                    node(80,[3],"all_approved")])

    def tearDown(self):
        self.temp.cleanup()

    def build(self, nodes, stop=None):
        if stop is not None:
            self.stop = stop
        if self.root.exists():
            shutil.rmtree(self.root)
        self.root.mkdir()
        self.plan.write_text(json.dumps({"schema":"NORDOI_AGENT_DAG_V1",
            "nodes": nodes},separators=(",",":"),ensure_ascii=True),
            encoding="utf-8")
        self.plan_sha = hashlib.sha256(self.plan.read_bytes()).hexdigest()
        proc = subprocess.run([sys.executable,"-B",
            str(HERE.parent/"app012_clean_checkpoint/run_app012.py"),
            "--plan",str(self.plan),"--engine",str(self.engine),
            "--source-root",str(self.source),"--grant-output-dir",str(self.root),
            "--expected-plan-sha256",self.plan_sha,
            "--expected-engine-sha256",self.engine_sha,
            "--expected-source-sha256",self.source_sha,
            "--stop-after",str(self.stop)],capture_output=True,cwd=REPO,timeout=160)
        self.assertEqual(proc.returncode,0,(proc.stdout,proc.stderr))
        self.prefix.write_bytes(proc.stdout)
        self.assertEqual(len(proc.stdout.splitlines()),self.stop)

    def call(self,action,*,anchor=None,bundle=None,root=None,plan=None,prefix=None,
             plan_pin=None):
        command=[sys.executable,"-B",str(HERE/"handoff_app013.py"),action,
                 "--bundle-dir",str(self.bundle if bundle is None else bundle),
                 "--native-root",str(self.root if root is None else root)]
        if action=="seal":
            command += ["--plan",str(self.plan if plan is None else plan),
                        "--prefix-events",str(self.prefix if prefix is None else prefix),
                        "--expected-plan-sha256",self.plan_sha if plan_pin is None else plan_pin]
        else:
            command += ["--expected-seal-sha256",self.anchor if anchor is None else anchor]
        if action=="resume":
            command += ["--engine",str(self.engine),
                        "--source-root",str(self.source),
                        "--expected-engine-sha256",self.engine_sha,
                        "--expected-source-sha256",self.source_sha]
        proc=subprocess.run(command,capture_output=True,cwd=REPO,timeout=165)
        try:
            rows=[json.loads(x) for x in proc.stdout.splitlines()]
        except (UnicodeError,ValueError):
            self.fail(f"invalid handoff json: {proc.stdout!r}; {proc.stderr!r}")
        return proc,rows

    def seal_ok(self):
        proc,rows=self.call("seal")
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"SEALED")
        self.anchor=rows[0]["manifest_sha256"]
        self.assertEqual(len(self.anchor),64)
        self.assertEqual({x.name for x in self.bundle.iterdir()},
                         set(handoff.FILES))
        return rows[0]

    def verify_ok(self):
        proc,rows=self.call("verify")
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"VERIFIED")
        return rows[0]

    def denied(self,action,code,*,anchor=None,bundle=None,root=None,plan=None,
               prefix=None,plan_pin=None):
        proc,rows=self.call(action,anchor=anchor,bundle=bundle,root=root,
                            plan=plan,prefix=prefix,plan_pin=plan_pin)
        self.assertEqual(proc.returncode,2,(rows,proc.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"REJECTED")
        self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")

    def inventory(self):
        return {x.relative_to(self.root).as_posix():x.read_bytes()
                for x in self.root.rglob("*") if x.is_file()}

    def rewrite_manifest(self,modify):
        path=self.bundle/"manifest.json"
        data=json.loads(path.read_text(encoding="utf-8"))
        modify(data)
        raw=handoff.encode(data)
        path.write_bytes(raw)
        self.anchor=hashlib.sha256(raw).hexdigest()
        (self.bundle/"SEAL.sha256").write_text(self.anchor+"\n",encoding="ascii")

    def mutate_prefix(self,modify):
        events=[json.loads(x) for x in self.prefix.read_bytes().splitlines()]
        modify(events)
        self.prefix.write_bytes(b"".join(handoff.encode(x) for x in events))

    def test_01_real_two_job_checkpoint_seals(self):
        row=self.seal_ok()
        self.assertEqual(row["native_jobs_verified"],2)
        self.assertEqual(row["native_files_verified"],4)

    def test_02_native_handoff_can_be_verified_readonly(self):
        self.seal_ok()
        self.assertEqual(self.verify_ok()["native_jobs_verified"],2)

    def test_03_manifests_are_deterministic_not_host_path_dependent(self):
        self.seal_ok()
        expected=self.anchor
        data=(self.bundle/"manifest.json").read_bytes()
        self.bundle=self.base/"second-bundle"
        self.assertEqual(self.seal_ok()["manifest_sha256"],expected)
        self.assertEqual((self.bundle/"manifest.json").read_bytes(),data)

    def test_04_exact_real_native_file_inventory(self):
        self.seal_ok()
        record=json.loads((self.bundle/"manifest.json").read_bytes())
        self.assertEqual(len(record["native_files"]),4)
        for item in record["native_files"]:
            path=self.root.joinpath(*item["path"].split("/"))
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),
                             item["sha256"])
            self.assertEqual(path.stat().st_size,item["bytes"])

    def test_05_sealed_plan_is_exact_original_bytes(self):
        self.seal_ok()
        self.assertEqual((self.bundle/"plan.json").read_bytes(),self.plan.read_bytes())

    def test_06_sealed_prefix_is_exact_original_native_stdout(self):
        self.seal_ok()
        self.assertEqual((self.bundle/"native-prefix.jsonl").read_bytes(),
                         self.prefix.read_bytes())

    def test_07_direct_handoff_resume_runs_only_remaining_real_native_jobs(self):
        self.seal_ok()
        before=self.inventory()
        proc,rows=self.call("resume")
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.assertEqual([r["status"] for r in rows],["COMMITTED","COMMITTED"])
        self.assertEqual([r["step"] for r in rows],[3,4])
        self.assertEqual({k:(self.root/k).read_bytes() for k in before},before)
        merged=self.base/"joined-native-events.jsonl"
        merged.write_bytes(self.prefix.read_bytes()+proc.stdout)
        report,code=handoff.independent.audit(self.root,self.plan,merged,self.plan_sha)
        self.assertEqual(code,0,report)
        self.assertEqual(report["committed_jobs_verified"],4)

    def test_08_resume_from_one_native_job_handoff(self):
        self.build([node(73),node(59),node(60)],stop=1)
        self.seal_ok()
        proc,rows=self.call("resume")
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.assertEqual([r["step"] for r in rows],[2,3])

    def test_09_seal_checkpoint_with_historical_skipped_node(self):
        self.build([node(59),node(73,[1],"all_approved"),node(73)],stop=2)
        self.assertFalse((self.root/"job-0002").exists())
        row=self.seal_ok()
        self.assertEqual(row["native_files_verified"],2)
        self.verify_ok()

    def test_10_resume_historical_skipped_node_correctly(self):
        self.build([node(59),node(73,[1],"all_approved"),node(73)],stop=2)
        self.seal_ok()
        proc,rows=self.call("resume")
        self.assertEqual(proc.returncode,0,(rows,proc.stderr))
        self.assertEqual(rows[0]["step"],3)
        self.assertEqual(rows[0]["status"],"COMMITTED")

    def test_11_seal_eight_node_graph_at_seven(self):
        self.build([node(i) for i in (0,20,59,60,61,73,90,100)],stop=7)
        row=self.seal_ok()
        self.assertEqual(row["native_jobs_verified"],7)
        self.assertEqual(row["native_files_verified"],14)
        self.verify_ok()

    def test_12_verifier_does_not_modify_seal_or_native_files(self):
        self.seal_ok()
        files=self.inventory()
        bundle_files={x.name:x.read_bytes() for x in self.bundle.iterdir()}
        self.verify_ok()
        self.assertEqual(self.inventory(),files)
        self.assertEqual({x.name:x.read_bytes() for x in self.bundle.iterdir()},
                         bundle_files)

    def test_13_manifest_contains_no_absolute_host_paths(self):
        self.seal_ok()
        manifest=(self.bundle/"manifest.json").read_text(encoding="utf-8")
        self.assertNotIn(str(self.base),manifest)
        self.assertNotIn(str(self.root),manifest)
        self.assertEqual(json.loads(manifest)["pin_provenance"],
                         "EXTERNAL_TRUST_NOT_ATTESTED")

    def test_14_wrong_external_seal_pin_rejected(self):
        self.seal_ok()
        self.denied("verify","HANDOFF_SEAL_PIN_MISMATCH",anchor="0"*64)

    def test_15_unpinned_resume_rejected_before_native_execution(self):
        self.seal_ok()
        self.denied("resume","HANDOFF_SEAL_PIN_MISMATCH",anchor="0"*64)
        self.assertEqual({x.name for x in self.root.iterdir()},
                         {"job-0001","job-0002"})

    def test_16_truncated_seal_marker_rejected(self):
        self.seal_ok()
        (self.bundle/"SEAL.sha256").write_text("a",encoding="ascii")
        self.denied("verify","HANDOFF_MARKER_MISMATCH")

    def test_17_missing_final_seal_marker_rejected(self):
        self.seal_ok()
        (self.bundle/"SEAL.sha256").unlink()
        self.denied("verify","HANDOFF_FILE_SET_CHANGED")

    def test_18_unexpected_handoff_file_rejected(self):
        self.seal_ok()
        (self.bundle/"agent-secret.txt").write_text("do not read")
        self.denied("verify","HANDOFF_FILE_SET_CHANGED")

    def test_19_missing_handoff_plan_file_rejected(self):
        self.seal_ok()
        (self.bundle/"plan.json").unlink()
        self.denied("verify","HANDOFF_FILE_SET_CHANGED")

    def test_20_missing_handoff_events_file_rejected(self):
        self.seal_ok()
        (self.bundle/"native-prefix.jsonl").unlink()
        self.denied("verify","HANDOFF_FILE_SET_CHANGED")

    def test_21_modified_manifest_cannot_pass_old_external_pin(self):
        self.seal_ok()
        f=self.bundle/"manifest.json"
        f.write_bytes(f.read_bytes()+b"tamper")
        self.denied("verify","HANDOFF_SEAL_PIN_MISMATCH")

    def test_22_modified_bundle_plan_denied_despite_matching_manifest(self):
        self.seal_ok()
        f=self.bundle/"plan.json"
        f.write_bytes(f.read_bytes()+b" ")
        self.denied("verify","PLAN_SHA256_PIN_MISMATCH")

    def test_23_modified_bundle_prefix_denied_with_original_manifest(self):
        self.seal_ok()
        f=self.bundle/"native-prefix.jsonl"
        f.write_bytes(f.read_bytes().replace(b"\n",b"\r\n"))
        self.denied("verify","HANDOFF_MANIFEST_CONTENT_CONTRADICTION")

    def test_24_modified_original_native_file_denied(self):
        self.seal_ok()
        f=self.root/"job-0002/quality-decision/report.txt"
        f.write_bytes(f.read_bytes()+b"forged")
        self.denied("verify","REAL_FILE_CONTENT_DIFFERS_FROM_RECEIPT")

    def test_25_deleted_original_native_file_denied(self):
        self.seal_ok()
        (self.root/"job-0001/quality-decision/decision.txt").unlink()
        self.denied("verify","BUNDLE_FILE_SET_CHANGED")

    def test_26_extra_original_native_file_denied(self):
        self.seal_ok()
        (self.root/"job-0001/quality-decision/extra.bin").write_bytes(b"x")
        self.denied("verify","BUNDLE_FILE_SET_CHANGED")

    def test_27_extra_original_native_job_denied(self):
        self.seal_ok()
        (self.root/"job-0003").mkdir()
        self.denied("verify","ROOT_UNEXPECTED_JOB_OR_FILE")

    def test_28_symlink_handoff_directory_denied_where_supported(self):
        self.seal_ok()
        alias=self.base/"alias-bundle"
        try:
            alias.symlink_to(self.bundle,target_is_directory=True)
        except (OSError,NotImplementedError):
            self.skipTest("OS does not allow symlink creation")
        self.denied("verify","HANDOFF_NOT_REAL_DIRECTORY",bundle=alias)

    def test_29_symlink_manifest_file_denied_where_supported(self):
        self.seal_ok()
        original=self.bundle/"manifest.json"
        alternative=self.base/"outside-manifest.json"
        alternative.write_bytes(original.read_bytes())
        original.unlink()
        try:
            original.symlink_to(alternative)
        except (OSError,NotImplementedError):
            self.skipTest("OS does not allow symlink creation")
        self.denied("verify","HANDOFF_MANIFEST_NOT_REGULAR")

    def test_30_manifest_duplicate_json_fields_rejected_even_if_reanchored(self):
        self.seal_ok()
        path=self.bundle/"manifest.json"
        raw=path.read_bytes().replace(
            b'"schema":"NORDOI_AGENT_CHECKPOINT_HANDOFF_V1"',
            b'"schema":"NORDOI_AGENT_CHECKPOINT_HANDOFF_V1","schema":"NORDOI_AGENT_CHECKPOINT_HANDOFF_V1"',1)
        path.write_bytes(raw)
        self.anchor=hashlib.sha256(raw).hexdigest()
        (self.bundle/"SEAL.sha256").write_text(self.anchor+"\n")
        self.denied("verify","DUPLICATE_RECEIPT_FIELD")

    def test_31_forged_inventory_with_new_self_trusted_hash_rejected(self):
        self.seal_ok()
        self.rewrite_manifest(lambda m: m["native_files"][0].__setitem__("sha256","0"*64))
        self.denied("verify","HANDOFF_MANIFEST_CONTENT_CONTRADICTION")

    def test_32_forged_committed_count_with_new_hash_rejected(self):
        self.seal_ok()
        self.rewrite_manifest(lambda m: m.__setitem__("committed",0))
        self.denied("verify","HANDOFF_MANIFEST_CONTENT_CONTRADICTION")

    def test_33_false_external_authority_claim_with_new_hash_rejected(self):
        self.seal_ok()
        self.rewrite_manifest(lambda m: m.__setitem__("pin_provenance","SIGNATURE_PROVEN"))
        self.denied("verify","HANDOFF_TRUST_CLAIM_INVALID")

    def test_34_existing_seal_directory_refuses_overwrite(self):
        self.seal_ok()
        original={x.name:x.read_bytes() for x in self.bundle.iterdir()}
        self.denied("seal","HANDOFF_DESTINATION_ALREADY_EXISTS")
        self.assertEqual({x.name:x.read_bytes() for x in self.bundle.iterdir()},original)

    def test_35_missing_bundle_parent_denies_before_effect(self):
        self.bundle=self.base/"missing"/"new-handoff"
        self.denied("seal","SEAL_PARENT_NOT_REAL_DIRECTORY")
        self.assertFalse(self.bundle.exists())

    def test_36_handoff_inside_real_native_root_is_refused(self):
        self.bundle=self.root/"not-allowed"
        self.denied("seal","HANDOFF_OVERLAPS_NATIVE_ROOT")
        self.assertFalse(self.bundle.exists())

    def test_37_wrong_trusted_plan_pin_prevents_seal(self):
        self.denied("seal","PLAN_SHA256_PIN_MISMATCH",plan_pin="0"*64)
        self.assertFalse(self.bundle.exists())

    def test_38_terminal_error_prefix_cannot_be_sealed(self):
        rows=[json.loads(x) for x in self.prefix.read_bytes().splitlines()]
        rows.append({"schema":"NORDOI_AGENT_DAG_EVENT_V1","step":3,
                     "status":"ERROR","gate":"all_committed",
                     "depends_on":[1,2],"plan_sha256":self.plan_sha,
                     "error_code":"NATIVE_OPERATION_FAILED",
                     "native_exit_code":7,"publication":"UNKNOWN"})
        self.prefix.write_bytes(b"".join(handoff.encode(x) for x in rows))
        self.denied("seal","NOT_CLEAN_PROPER_PREFIX")
        self.assertFalse(self.bundle.exists())

    def test_39_complete_graph_cannot_be_sealed_as_resume_checkpoint(self):
        rows=[json.loads(x) for x in self.prefix.read_bytes().splitlines()]
        for i,n in enumerate((60,80),3):
            # Do not invent fake receipts; only the clean-prefix length is
            # needed to reject a dishonest claim that a complete DAG can resume.
            rows.append({"schema":"NORDOI_AGENT_DAG_EVENT_V1","step":i,
                         "status":"SKIPPED","gate":"always",
                         "depends_on":[],"plan_sha256":self.plan_sha,
                         "reason":"DEPENDENCY_GATE_FALSE"})
        self.prefix.write_bytes(b"".join(handoff.encode(x) for x in rows))
        self.denied("seal","NOT_CLEAN_PROPER_PREFIX")

    def test_40_replaying_trusted_handoff_after_native_resume_is_denied(self):
        self.seal_ok()
        p,rows=self.call("resume")
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        self.denied("resume","ROOT_UNEXPECTED_JOB_OR_FILE")
        # No historical native P2.7 bundle may be overwritten by the replay.
        self.assertEqual({x.name for x in self.root.iterdir()},
                         {"job-0001","job-0002","job-0003","job-0004"})


if __name__=="__main__":
    unittest.main()
