"""APP-016 genuine pinned receiver-side policy admission, 40 tests on four OS.

A fresh original and coherently altered alternative are generated with REAL
NORDOI Rust by APP-012 -> APP-013 -> APP-014, without simulated native output.
Receiver policies are written outside sender artifacts, with a separately
provided policy-file SHA-256. The verifier cannot choose engine/source pins
from the untrusted package. All runs are research-only, not signed or sandboxed.
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
import receiver_app016 as admission


def invoke(script, *args, timeout=180):
    p = subprocess.run([sys.executable, "-B", str(script), *map(str, args)],
                       cwd=REPO, capture_output=True, timeout=timeout)
    try:
        rows = [json.loads(x) for x in p.stdout.splitlines()]
    except (UnicodeError, ValueError):
        raise AssertionError(f"non-json stdout {p.stdout!r}; stderr {p.stderr!r}")
    return p, rows


class ReceiverOwnedPolicy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp_all = tempfile.TemporaryDirectory(prefix="nordoi_app016_real_")
        cls.root = Path(cls.tmp_all.name)
        cls.engine_sha = admission.pinned.engine_sha256(ENGINE)
        cls.source_sha = admission.pinned.source_tree_sha256(SOURCE)
        cls.alt_source = cls.root / "alternative-authentic-noi-source"
        shutil.copytree(SOURCE, cls.alt_source)
        f = cls.alt_source / "app/rules.noi"
        original = f.read_text(encoding="utf-8")
        assert "score >= 60" in original
        f.write_text(original.replace("score >= 60", "score >= 1000"),
                     encoding="utf-8")
        cls.alt_sha = admission.pinned.source_tree_sha256(cls.alt_source)
        cls.datasets = {}
        for label, source, pin in (
            ("original", SOURCE, cls.source_sha),
            ("alternative", cls.alt_source, cls.alt_sha)
        ):
            folder = cls.root / ("authentic-" + label)
            folder.mkdir()
            plan = folder / "native-plan.json"
            plan.write_bytes(admission.canonical_json({
                "schema": "NORDOI_AGENT_DAG_V1",
                "nodes": [
                    {"key_code":73, "depends_on":[], "gate":"always"},
                    {"key_code":59, "depends_on":[], "gate":"always"},
                    {"key_code":80, "depends_on":[1,2], "gate":"all_committed"},
                    {"key_code":90, "depends_on":[3], "gate":"all_approved"},
                ],
            }))
            plan_sha = admission.digest(plan.read_bytes())
            native = folder / "real-p27"
            native.mkdir()
            p,rows = invoke(
                HERE.parent/"app012_clean_checkpoint/run_app012.py",
                "--plan",plan, "--engine",ENGINE, "--source-root",source,
                "--grant-output-dir",native, "--stop-after","2",
                "--expected-plan-sha256",plan_sha,
                "--expected-engine-sha256",cls.engine_sha,
                "--expected-source-sha256",pin)
            assert p.returncode==0 and [r["status"] for r in rows]==["COMMITTED"]*2,(rows,p.stderr)
            prefix=folder/"native-prefix.jsonl"
            prefix.write_bytes(p.stdout)
            sealed=folder/"sealed-app013"
            p,rows=invoke(HERE.parent/"app013_verified_handoff/handoff_app013.py",
                "seal", "--bundle-dir",sealed,
                "--native-root",native,"--plan",plan,
                "--prefix-events",prefix,"--expected-plan-sha256",plan_sha)
            assert p.returncode==0,(rows,p.stderr)
            inner_pin=rows[0]["manifest_sha256"]
            portable=folder/"portable-app014"
            p,rows=invoke(HERE.parent/"app014_portable_checkpoint/portable_app014.py",
                "pack","--portable-dir",portable,"--handoff-dir",sealed,
                "--native-root",native,"--expected-handoff-sha256",inner_pin)
            assert p.returncode==0,(rows,p.stderr)
            cls.datasets[label]={
                "portable":portable,"pin":rows[0]["portable_sha256"],
                "plan_pin":plan_sha, "native":native, "prefix":prefix
            }

    @classmethod
    def tearDownClass(cls):
        cls.tmp_all.cleanup()

    def setUp(self):
        self.case=tempfile.TemporaryDirectory(prefix="app016_case_",dir=self.root)
        self.work=Path(self.case.name)
        self.portable=self.work/"received-portable"
        shutil.copytree(self.datasets["original"]["portable"],self.portable)
        self.dest=self.work/"native-target"
        self.dest.mkdir()
        self.policy_file=self.work/"receiver-owned-policy.json"
        self.policy={
            "schema":admission.SCHEMA,
            "policy_id":"receiver-lab-016",
            "engine_sha256":self.engine_sha,
            "source_tree_sha256":self.source_sha,
            "portable_sha256":self.datasets["original"]["pin"],
            "plan_sha256":self.datasets["original"]["plan_pin"],
            "allowed_actions":["inspect","restore","resume","verify"],
            "release":"RESEARCH_ONLY",
            "pin_origin":"RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED",
            "signature":"NOT_PRESENT",
            "production":"NOT_AUTHORIZED",
        }
        self.anchor=self.save_policy()

    def tearDown(self):
        self.case.cleanup()

    def save_policy(self):
        raw=admission.canonical_json(self.policy)
        self.policy_file.write_bytes(raw)
        self.anchor=admission.digest(raw)
        return self.anchor

    def call(self,action="inspect",*,policy_pin=None,engine=None,
             source=None,policy_path=None,portable_dir=None,dest=None):
        args=[
            action, "--receiver-policy",
            self.policy_file if policy_path is None else policy_path,
            "--expected-policy-sha256",
            self.anchor if policy_pin is None else policy_pin,
            "--portable-dir",
            self.portable if portable_dir is None else portable_dir,
        ]
        if action in ("restore","resume"):
            args += ["--restore-root", self.dest if dest is None else dest]
        if action in ("verify","restore","resume"):
            args += ["--engine", ENGINE if engine is None else engine,
                     "--source-root", SOURCE if source is None else source]
        return invoke(HERE/"receiver_app016.py",*args)

    def ok(self,action="inspect",**opts):
        p,rows=self.call(action,**opts)
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        if action!="resume":
            self.assertEqual(len(rows),1)
        return p,rows

    def rejected(self,code,action="inspect",**opts):
        p,rows=self.call(action,**opts)
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"REJECTED")
        self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")

    def take_alternate(self,*,alter_policy=False,alter_source=False):
        shutil.rmtree(self.portable)
        shutil.copytree(self.datasets["alternative"]["portable"],self.portable)
        if alter_policy:
            self.policy["portable_sha256"]=self.datasets["alternative"]["pin"]
        if alter_source:
            self.policy["source_tree_sha256"]=self.alt_sha
        if alter_policy or alter_source:
            self.save_policy()

    def native_hashes(self,root):
        return {p.relative_to(root).as_posix():admission.digest(p.read_bytes())
                for p in root.rglob("*") if p.is_file()}

    def test_01_inspect_only_uses_receiver_file(self):
        r=self.ok()[1][0]
        self.assertEqual(r["status"],"POLICY_MATCH")

    def test_02_inspect_has_no_native_replay(self):
        self.assertEqual(self.ok()[1][0]["native_replay"],"NOT_PERFORMED")

    def test_03_inspect_matches_two_real_committed_jobs(self):
        self.assertEqual(self.ok()[1][0]["native_jobs_held"],2)

    def test_04_inspect_returns_trusted_external_policy_sha(self):
        self.assertEqual(self.ok()[1][0]["receiver_policy_sha256"],self.anchor)

    def test_05_policy_is_canonical_and_byte_pinned(self):
        r=self.ok()[1][0]
        self.assertEqual(r["portable_sha256"],self.policy["portable_sha256"])
        self.assertEqual(self.anchor,admission.digest(self.policy_file.read_bytes()))

    def test_06_full_replay_with_receiver_pins_passes(self):
        r=self.ok("verify")[1][0]
        self.assertEqual(r["status"],"PASS")
        self.assertEqual(r["native_jobs_recomputed"],2)
        self.assertEqual(r["native_files_bytewise_equal"],4)

    def test_07_verify_no_modifications_to_received_package(self):
        before=self.native_hashes(self.portable)
        self.ok("verify")
        self.assertEqual(before,self.native_hashes(self.portable))

    def test_08_verify_cannot_create_restoration_effects(self):
        self.ok("verify")
        self.assertFalse(list(self.dest.iterdir()))

    def test_09_restore_succeeds_only_after_valid_replay(self):
        r=self.ok("restore")[1][0]
        self.assertEqual(r["status"],"RESTORED")
        self.assertEqual(r["replay_qualification"],
                         "PASS_EXACT_PINNED_NATIVE_RUST_REPLAY")

    def test_10_restored_files_match_original_p27(self):
        self.ok("restore")
        self.assertEqual(self.native_hashes(self.dest),
                         self.native_hashes(self.datasets["original"]["native"]))

    def test_11_native_resume_executes_only_remaining_jobs(self):
        self.ok("restore")
        p,rows=self.ok("resume")
        self.assertEqual([e["step"] for e in rows],[3,4])
        self.assertEqual([e["status"] for e in rows],
                         ["COMMITTED","COMMITTED"])

    def test_12_full_dag_passes_app010_independent_audit(self):
        self.ok("restore")
        p,rows=self.ok("resume")
        complete=self.work/"completed-app016-native-dag.jsonl"
        complete.write_bytes((self.portable/"handoff/native-prefix.jsonl").read_bytes()
                             +p.stdout)
        report,code=admission.portable.independent.audit(
            self.dest,self.portable/"handoff/plan.json",complete,
            self.policy["plan_sha256"])
        self.assertEqual(code,0,report)
        self.assertEqual(report["committed_jobs_verified"],4)

    def test_13_original_files_unchanged_after_resume(self):
        before=self.native_hashes(self.datasets["original"]["native"])
        self.ok("restore")
        self.ok("resume")
        self.assertEqual(before,self.native_hashes(self.datasets["original"]["native"]))

    def test_14_second_restore_denied_without_overwrite(self):
        self.ok("restore")
        before=self.native_hashes(self.dest)
        self.rejected("RESTORE_ROOT_NOT_EMPTY","restore")
        self.assertEqual(before,self.native_hashes(self.dest))

    def test_15_second_resume_rejected(self):
        self.ok("restore")
        self.ok("resume")
        self.rejected("ROOT_UNEXPECTED_JOB_OR_FILE","resume")

    def test_16_policy_without_verify_permission_refuses_verify(self):
        self.policy["allowed_actions"]=["inspect","restore","resume"]
        self.save_policy()
        self.rejected("RECEIVER_POLICY_ACTION_DENIED","verify")

    def test_17_inspect_permission_independent_of_replay_permission(self):
        self.policy["allowed_actions"]=["inspect"]
        self.save_policy()
        self.ok()
        self.rejected("RECEIVER_POLICY_ACTION_DENIED","verify")

    def test_18_policy_without_restore_permission_refuses_any_write(self):
        self.policy["allowed_actions"]=["inspect","resume","verify"]
        self.save_policy()
        self.rejected("RECEIVER_POLICY_ACTION_DENIED","restore")
        self.assertFalse(list(self.dest.iterdir()))

    def test_19_policy_without_resume_permission_refuses_resume(self):
        self.policy["allowed_actions"]=["inspect","restore","verify"]
        self.save_policy()
        self.ok("restore")
        self.rejected("RECEIVER_POLICY_ACTION_DENIED","resume")
        self.assertFalse((self.dest/"job-0003").exists())

    def test_20_wrong_receiver_policy_sha_is_refused(self):
        self.rejected("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH",
                      policy_pin="0"*64)

    def test_21_invalid_external_policy_sha_is_refused(self):
        self.rejected("RECEIVER_POLICY_EXTERNAL_PIN_INVALID",policy_pin="bad")

    def test_22_mutated_policy_with_stale_anchor_is_refused(self):
        self.policy_file.write_bytes(self.policy_file.read_bytes()+b"forged")
        self.rejected("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH")

    def test_23_noncanonical_policy_cannot_be_reanchored(self):
        data=json.dumps(self.policy,indent=2,sort_keys=True).encode()+b"\n"
        self.policy_file.write_bytes(data)
        self.anchor=admission.digest(data)
        self.rejected("RECEIVER_POLICY_NOT_CANONICAL")

    def test_24_duplicate_policy_json_field_is_rejected(self):
        raw=self.policy_file.read_bytes()
        raw=raw.replace(b'"schema":',b'"schema":"FORGED","schema":',1)
        self.policy_file.write_bytes(raw)
        self.anchor=admission.digest(raw)
        self.rejected("DUPLICATE_RECEIPT_FIELD")

    def test_25_unrecognized_policy_field_is_rejected(self):
        self.policy["unknown_feature"]="implicit-production"
        self.save_policy()
        self.rejected("RECEIVER_POLICY_SCHEMA_INVALID")

    def test_26_receiver_policy_cannot_claim_production(self):
        self.policy["production"]="AUTHORIZED"
        self.save_policy()
        self.rejected("RECEIVER_POLICY_AUTHORITY_CLAIM_INVALID")

    def test_27_receiver_policy_cannot_claim_signature(self):
        self.policy["signature"]="SIGNED"
        self.save_policy()
        self.rejected("RECEIVER_POLICY_AUTHORITY_CLAIM_INVALID")

    def test_28_unsorted_duplicate_actions_rejected(self):
        self.policy["allowed_actions"]=["verify","verify"]
        self.save_policy()
        self.rejected("RECEIVER_POLICY_ACTIONS_INVALID")

    def test_29_wrong_engine_pin_in_receiver_policy_denied(self):
        self.policy["engine_sha256"]="0"*64
        self.save_policy()
        self.rejected("ENGINE_PIN_MISMATCH","verify")

    def test_30_wrong_source_pin_in_receiver_policy_denied(self):
        self.policy["source_tree_sha256"]="0"*64
        self.save_policy()
        self.rejected("SOURCE_PIN_MISMATCH","verify")

    def test_31_foreign_portable_rejected_with_original_receiver_policy(self):
        self.take_alternate()
        self.rejected("PORTABLE_EXTERNAL_PIN_MISMATCH")

    def test_32_receiver_accepts_alternate_package_digest_but_original_source_pin(self):
        self.take_alternate(alter_policy=True)
        self.ok("inspect")
        self.rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH","verify")

    def test_33_coherent_alternate_native_history_cannot_restore(self):
        self.take_alternate(alter_policy=True)
        self.rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH","restore")
        self.assertFalse(list(self.dest.iterdir()))

    def test_34_coherent_alternate_native_history_cannot_resume(self):
        self.take_alternate(alter_policy=True)
        self.rejected("INDEPENDENT_NATIVE_RECEIPT_MISMATCH","resume")
        self.assertFalse((self.dest/"job-0003").exists())

    def test_35_alternative_source_path_cannot_override_policy_pin(self):
        self.rejected("SOURCE_PIN_MISMATCH","verify",source=self.alt_source)

    def test_36_external_receiver_policy_can_consciously_change_all_pins(self):
        self.take_alternate(alter_policy=True,alter_source=True)
        r=self.ok("verify",source=self.alt_source)[1][0]
        self.assertEqual(r["status"],"PASS")
        self.assertEqual(r["pin_origin"],
                         "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED")

    def test_37_wrong_plan_sha_in_policy_rejects_even_valid_portable(self):
        self.policy["plan_sha256"]="0"*64
        self.save_policy()
        self.rejected("RECEIVER_POLICY_PLAN_NOT_AUTHORIZED")

    def test_38_policy_cannot_live_inside_untrusted_portable(self):
        embedded=self.portable/"receiver-policy.json"
        embedded.write_bytes(self.policy_file.read_bytes())
        self.rejected("RECEIVER_POLICY_INSIDE_UNTRUSTED_PACKAGE",
                      policy_path=embedded)

    def test_39_symlinked_receiver_policy_rejected_where_supported(self):
        alias=self.work/"alias-policy.json"
        try:
            alias.symlink_to(self.policy_file)
        except (NotImplementedError,OSError):
            self.skipTest("symlinks not available on this OS")
        self.rejected("RECEIVER_POLICY_NOT_REGULAR_BOUNDED",
                      policy_path=alias)

    def test_40_policy_disclaims_authenticity_sandbox_and_production(self):
        r=self.ok("verify")[1][0]
        self.assertEqual(r["pin_origin"],
                         "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED")
        self.assertEqual(r["engine_sandbox"],"NOT_PROVIDED")
        self.assertEqual(r["signature"],"NOT_PRESENT")
        self.assertEqual(r["production"],"NOT_AUTHORIZED")


if __name__=="__main__":
    unittest.main()
