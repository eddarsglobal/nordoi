"""APP-017 — 40 actual native receiver-roster rotation/freeze tests.

The fixture independently generates authentic Rust P2.7 packages and two
coherent .noi histories using APP-016's verified historical generator. Tests
are NEW assertions for receiver roster gating; all APP-016 tests are also
run unchanged in the separate regression suite.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
ENGINE = REPO / "target/debug" / ("nordoi.exe" if sys.platform == "win32" else "nordoi")
SOURCE = REPO / "examples/apps/app003_integrated_quality/src"
sys.path.insert(0, str(HERE))
import roster_app017 as roster


class ReceiverRevocationRoster(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(HERE.parent / "app016_receiver_trust_policy"))
        import test_app016
        cls._fixture_class = test_app016.ReceiverOwnedPolicy
        cls._fixture_class.setUpClass()

    @classmethod
    def tearDownClass(cls):
        cls._fixture_class.tearDownClass()

    def setUp(self):
        self.fx = self._fixture_class("test_01_inspect_only_uses_receiver_file")
        self.fx.setUp()
        self.original_policy = self.fx.work / "ORIGINAL_receiver_policy_snapshot.json"
        self.original_policy.write_bytes(self.fx.policy_file.read_bytes())
        self.original_policy_sha = self.fx.anchor
        self.original_pkg = self.fx.work / "ORIGINAL_checkpoint_snapshot"
        shutil.copytree(self.fx.portable, self.original_pkg)
        self.roster_file = self.fx.work / "RECEIVER_REVOCATION_ROSTER.json"
        self.roster = {
            "schema": roster.SCHEMA,
            "generation": 1,
            "state": "ACTIVE",
            "active_policy_sha256": self.fx.anchor,
            "revoked_policy_sha256": [],
            "pin_origin": "RECEIVER_SUPPLIED_NOT_EXTERNALLY_ATTESTED",
            "signature": "NOT_PRESENT",
            "release": "RESEARCH_ONLY",
            "production": "NOT_AUTHORIZED",
            "durable_anti_rollback": "NOT_PROVIDED",
        }
        self.anchor = self.write_roster()

    def tearDown(self):
        self.fx.tearDown()

    def write_roster(self):
        raw = roster.canonical(self.roster)
        self.roster_file.write_bytes(raw)
        self.anchor = roster.digest(raw)
        return self.anchor

    def call(self, action="status", *, anchor=None, min_generation=1,
             policy_file=None, package=None, source=None, engine=None,
             dest=None, roster_file=None):
        cmd = [
            sys.executable, "-B", str(HERE / "roster_app017.py"), action,
            "--receiver-roster",
            str(self.roster_file if roster_file is None else roster_file),
            "--expected-roster-sha256",
            self.anchor if anchor is None else anchor,
            "--receiver-minimum-generation", str(min_generation),
        ]
        if action != "status":
            cmd.extend([
                "--receiver-policy", str(self.fx.policy_file
                                         if policy_file is None else policy_file),
                "--portable-dir", str(self.fx.portable
                                      if package is None else package),
            ])
        if action in ("verify","restore","resume"):
            cmd.extend([
                "--engine", str(ENGINE if engine is None else engine),
                "--source-root", str(SOURCE if source is None else source),
            ])
        if action in ("restore","resume"):
            cmd.extend(["--restore-root",str(self.fx.dest if dest is None else dest)])
        result = subprocess.run(cmd,capture_output=True,cwd=REPO,timeout=180)
        try:
            rows = [json.loads(x) for x in result.stdout.splitlines()]
        except (ValueError,UnicodeError):
            self.fail(f"non-JSON: {result.stdout!r} stderr={result.stderr!r}")
        return result, rows

    def good(self, action="status", **kwargs):
        p,rows=self.call(action,**kwargs)
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        if action != "resume":
            self.assertEqual(len(rows),1)
        return p,rows

    def rejected(self, code, action="inspect", **kwargs):
        p,rows=self.call(action,**kwargs)
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"REJECTED")
        self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")

    def rotated(self):
        self.fx.take_alternate(alter_policy=True,alter_source=True)
        new_policy_sha = self.fx.anchor
        self.roster.update({
            "generation":2,
            "active_policy_sha256":new_policy_sha,
            "revoked_policy_sha256":[self.original_policy_sha],
        })
        self.write_roster()
        return new_policy_sha

    def frozen(self):
        self.roster.update({
            "generation":2, "state":"FROZEN",
            "active_policy_sha256":None,
            "revoked_policy_sha256":[self.original_policy_sha],
        })
        self.write_roster()

    def test_01_v1_roster_status_active_without_native_engine(self):
        r=self.good()[1][0]
        self.assertEqual(r["status"],"ACTIVE")
        self.assertEqual(r["native_engine"],"NOT_EXECUTED")

    def test_02_v1_roster_sha_is_external_pin(self):
        r=self.good()[1][0]
        self.assertEqual(r["roster_sha256"],self.anchor)

    def test_03_v1_generation_is_one(self):
        self.assertEqual(self.good()[1][0]["generation"],1)

    def test_04_original_policy_is_admitted_through_roster(self):
        self.assertEqual(self.good("inspect")[1][0]["status"],"POLICY_MATCH")

    def test_05_inspection_does_not_call_rust(self):
        self.assertEqual(self.good("inspect")[1][0]["native_engine"],
                         "NOT_EXECUTED")

    def test_06_true_pinned_native_replay_is_allowed_by_v1(self):
        r=self.good("verify")[1][0]
        self.assertEqual(r["status"],"PASS")
        self.assertEqual(r["native_jobs_recomputed"],2)
        self.assertEqual(r["native_files_bytewise_equal"],4)

    def test_07_exclusive_restore_authorized_by_v1(self):
        self.assertEqual(self.good("restore")[1][0]["status"],"RESTORED")

    def test_08_true_native_resume_only_executes_future_jobs(self):
        self.good("restore")
        _,rows=self.good("resume")
        self.assertEqual([e["step"] for e in rows],[3,4])

    def test_09_v1_resume_dag_has_independent_app010_pass(self):
        self.good("restore")
        p,rows=self.good("resume")
        combined=self.fx.work/"v1_whole_dag.jsonl"
        combined.write_bytes((self.fx.portable/"handoff/native-prefix.jsonl").read_bytes()
                             +p.stdout)
        report,code=roster.portable.independent.audit(
            self.fx.dest,self.fx.portable/"handoff/plan.json",
            combined,self.fx.policy["plan_sha256"])
        self.assertEqual(code,0,report)
        self.assertEqual(report["status"],"PASS")

    def test_10_v1_second_native_resume_refused(self):
        self.good("restore")
        self.good("resume")
        self.rejected("ROOT_UNEXPECTED_JOB_OR_FILE","resume")

    def test_11_replacement_roster_rotates_to_generation_two(self):
        self.rotated()
        self.assertEqual(self.good()[1][0]["generation"],2)

    def test_12_rotation_records_exact_previous_policy_revocation(self):
        self.rotated()
        self.assertEqual(self.good()[1][0]["revoked_count"],1)

    def test_13_original_old_policy_accepted_under_original_roster(self):
        self.good("inspect",policy_file=self.original_policy,
                  package=self.original_pkg)

    def test_14_original_policy_rejected_after_receiver_roster_rotation(self):
        self.rotated()
        self.rejected("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH",
            policy_file=self.original_policy,package=self.original_pkg)

    def test_15_old_roster_external_pin_rejected_after_rotation(self):
        prior=self.anchor
        self.rotated()
        self.rejected("ROSTER_EXTERNAL_PIN_MISMATCH",anchor=prior)

    def test_16_alternate_checkpoint_rejected_by_old_active_policy(self):
        self.fx.take_alternate(alter_policy=True,alter_source=True)
        self.rejected("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH")

    def test_17_rotated_roster_can_admit_new_receiver_policy(self):
        self.rotated()
        self.assertEqual(self.good("inspect")[1][0]["status"],"POLICY_MATCH")

    def test_18_rotated_policy_and_real_alternative_source_replay(self):
        self.rotated()
        self.assertEqual(self.good("verify",source=self.fx.alt_source)[1][0]["status"],
                         "PASS")

    def test_19_rotated_policy_does_not_accept_original_source(self):
        self.rotated()
        self.rejected("SOURCE_PIN_MISMATCH","verify")

    def test_20_rotated_policy_and_true_source_restore(self):
        self.rotated()
        self.assertEqual(self.good("restore",source=self.fx.alt_source)[1][0]["status"],
                         "RESTORED")

    def test_21_rotated_policy_and_true_source_resume(self):
        self.rotated()
        self.good("restore",source=self.fx.alt_source)
        _,rows=self.good("resume",source=self.fx.alt_source)
        self.assertEqual([r["step"] for r in rows],[3,4])

    def test_22_receiver_minimum_generation_blocks_stale_v1(self):
        self.rejected("ROSTER_BELOW_RECEIVER_MINIMUM_GENERATION",
                      min_generation=2)

    def test_23_receiver_minimum_generation_accepts_current_v2(self):
        self.rotated()
        self.good("inspect",min_generation=2)

    def test_24_receiver_minimum_generation_three_blocks_v2(self):
        self.rotated()
        self.rejected("ROSTER_BELOW_RECEIVER_MINIMUM_GENERATION",
                      min_generation=3)

    def test_25_receiver_frozen_roster_has_no_active_policy(self):
        self.frozen()
        r=self.good()[1][0]
        self.assertEqual(r["status"],"FROZEN")
        self.assertIsNone(r["active_policy_sha256"])

    def test_26_frozen_roster_blocks_inspection_admission(self):
        self.frozen()
        self.rejected("ROSTER_FROZEN_DENIES_ADMISSION")

    def test_27_frozen_roster_blocks_native_replay(self):
        self.frozen()
        self.rejected("ROSTER_FROZEN_DENIES_ADMISSION","verify")

    def test_28_frozen_roster_blocks_restore_before_effect(self):
        self.frozen()
        self.rejected("ROSTER_FROZEN_DENIES_ADMISSION","restore")
        self.assertFalse(list(self.fx.dest.iterdir()))

    def test_29_frozen_roster_blocks_resume_before_effect(self):
        self.frozen()
        self.rejected("ROSTER_FROZEN_DENIES_ADMISSION","resume")
        self.assertFalse(list(self.fx.dest.iterdir()))

    def test_30_frozen_roster_cannot_claim_active_policy_sha(self):
        self.frozen()
        self.roster["active_policy_sha256"]=self.original_policy_sha
        self.write_roster()
        self.rejected("ROSTER_FROZEN_MUST_HAVE_NO_ACTIVE_POLICY")

    def test_31_active_roster_cannot_revoke_its_own_policy(self):
        self.roster["revoked_policy_sha256"]=[self.original_policy_sha]
        self.write_roster()
        self.rejected("ROSTER_ACTIVE_POLICY_IS_REVOKED")

    def test_32_duplicate_revoked_policy_sha_is_rejected(self):
        self.roster["revoked_policy_sha256"]=["0"*64,"0"*64]
        self.write_roster()
        self.rejected("ROSTER_REVOKED_LIST_INVALID")

    def test_33_unsorted_revoked_policy_sha_is_rejected(self):
        self.roster["revoked_policy_sha256"]=["f"*64,"0"*64]
        self.write_roster()
        self.rejected("ROSTER_REVOKED_LIST_INVALID")

    def test_34_revocation_roster_has_bounded_policy_count(self):
        self.roster["revoked_policy_sha256"]=[format(i,"064x") for i in range(33)]
        self.write_roster()
        self.rejected("ROSTER_REVOKED_LIST_INVALID")

    def test_35_tampered_roster_with_old_pin_denied(self):
        self.roster_file.write_bytes(self.roster_file.read_bytes()+b"tamper")
        self.rejected("ROSTER_EXTERNAL_PIN_MISMATCH")

    def test_36_roster_with_forged_signature_denied(self):
        self.roster["signature"]="SIGNED_BY_BUREAU"
        self.write_roster()
        self.rejected("ROSTER_AUTHORITY_CLAIM_INVALID")

    def test_37_roster_with_forged_durable_guarantee_denied(self):
        self.roster["durable_anti_rollback"]="GUARANTEED"
        self.write_roster()
        self.rejected("ROSTER_AUTHORITY_CLAIM_INVALID")

    def test_38_roster_must_be_canonical_even_if_new_sha_trusted(self):
        raw=json.dumps(self.roster,indent=2,sort_keys=True).encode()+b"\n"
        self.roster_file.write_bytes(raw)
        self.anchor=hashlib.sha256(raw).hexdigest()
        self.rejected("ROSTER_NOT_CANONICAL")

    def test_39_roster_inside_untrusted_portable_refused(self):
        embedded=self.fx.portable/"embedded-roster.json"
        embedded.write_bytes(self.roster_file.read_bytes())
        self.rejected("ROSTER_OR_POLICY_INSIDE_UNTRUSTED_PACKAGE",
                      roster_file=embedded)

    def test_40_status_candidly_declares_no_durable_anti_rollback(self):
        row=self.good()[1][0]
        self.assertEqual(row["durable_anti_rollback"],"NOT_PROVIDED")
        self.assertEqual(row["external_pin_authentication"],"NOT_PERFORMED")
        self.assertEqual(row["production"],"NOT_AUTHORIZED")


if __name__ == "__main__":
    unittest.main()
