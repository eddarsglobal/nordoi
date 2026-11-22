"""APP-018: 40 independent native-receipt freshness-journal tests on 4 OS.

Two coherent real Rust native histories are produced by APP-012→013→014
via the already qualified APP-017 fixture. New tests assert MONOTONIC local
receiver history survives Python process restart, old roster+old digest cannot
authorize against the intact new local anchor, and FROZEN stops native effects.

No power-cut simulation, independent certificate, TPM or malicious host
filesystem is claimed. Normal process restarts use fresh subprocesses.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
sys.path.insert(0,str(HERE))
import freshness_app018 as freshness
sys.path.insert(0,str(HERE.parent/"app017_receiver_revocation"))
from test_app017 import ReceiverRevocationRoster, ENGINE, SOURCE


class LocalReceiverFreshness(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        ReceiverRevocationRoster.setUpClass()

    @classmethod
    def tearDownClass(cls):
        ReceiverRevocationRoster.tearDownClass()

    def setUp(self):
        self.fx=ReceiverRevocationRoster(
            "test_01_v1_roster_status_active_without_native_engine")
        self.fx.setUp()
        self.root=self.fx.fx.work/"receiver-private-freshness-anchor"
        self.old_roster_file=self.fx.fx.work/"trusted-generation-one.json"
        self.old_roster_file.write_bytes(self.fx.roster_file.read_bytes())
        self.old_roster_pin=self.fx.anchor
        self.init()

    def tearDown(self):
        self.fx.tearDown()

    def call(self,action="status",*,root=None,roster_file=None,
             roster_sha=None,policy_file=None,package=None,
             engine=None,source=None,dest=None,min_generation=1):
        args=[
            sys.executable,"-B",str(HERE/"freshness_app018.py"),action,
            "--anchor-root",str(self.root if root is None else root),
        ]
        if action in ("bootstrap","advance"):
            args.extend([
                "--receiver-roster",
                str(self.fx.roster_file if roster_file is None else roster_file),
                "--expected-roster-sha256",
                self.fx.anchor if roster_sha is None else roster_sha,
                "--receiver-minimum-generation",str(min_generation),
            ])
        elif action!="status":
            args.extend([
                "--receiver-roster",
                str(self.fx.roster_file if roster_file is None else roster_file),
                "--receiver-policy",
                str(self.fx.fx.policy_file if policy_file is None else policy_file),
                "--portable-dir",
                str(self.fx.fx.portable if package is None else package),
            ])
            if action in ("verify","restore","resume"):
                args.extend([
                    "--engine",str(ENGINE if engine is None else engine),
                    "--source-root",str(SOURCE if source is None else source),
                ])
            if action in ("restore","resume"):
                args.extend([
                    "--restore-root",
                    str(self.fx.fx.dest if dest is None else dest),
                ])
        proc=subprocess.run(args,cwd=REPO,capture_output=True,timeout=190)
        try:
            lines=[json.loads(x) for x in proc.stdout.splitlines()]
        except (ValueError,UnicodeError):
            self.fail(f"NON-JSON output: {proc.stdout!r} {proc.stderr!r}")
        return proc,lines

    def good(self,action="status",**kwargs):
        p,rows=self.call(action,**kwargs)
        self.assertEqual(p.returncode,0,(rows,p.stderr))
        if action!="resume":
            self.assertEqual(len(rows),1)
        return p,rows

    def denied(self,code,action="inspect",**kwargs):
        p,rows=self.call(action,**kwargs)
        self.assertEqual(p.returncode,2,(rows,p.stderr))
        self.assertEqual(len(rows),1)
        self.assertEqual(rows[0]["status"],"REJECTED")
        self.assertEqual(rows[0]["error_code"],code)
        self.assertEqual(rows[0]["publication"],"NOT_STARTED")

    def init(self):
        result,rows=self.good("bootstrap")
        self.assertEqual(rows[0]["status"],"BOOTSTRAPPED")

    def rotate(self):
        self.fx.rotated()
        p,rows=self.good("advance")
        self.assertEqual(rows[0]["status"],"ADVANCED")
        return rows[0]

    def freeze(self):
        self.fx.frozen()
        p,rows=self.good("advance")
        self.assertEqual(rows[0]["roster_state"],"FROZEN")
        return rows[0]

    def native_snapshot(self,base):
        return {p.relative_to(base).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
                for p in base.rglob("*") if p.is_file()}

    def test_01_bootstrap_creates_exactly_one_local_head(self):
        self.assertTrue((self.root/"HEAD.json").is_file())

    def test_02_bootstrap_creates_immutable_first_journal_entry(self):
        self.assertTrue((self.root/"history/entry-00000001.json").is_file())

    def test_03_status_survives_real_python_process_restart(self):
        self.assertEqual(self.good("status")[1][0]["sequence"],1)

    def test_04_status_never_executes_native_engine(self):
        self.assertEqual(self.good("status")[1][0]["native_engine"],"NOT_EXECUTED")

    def test_05_journal_is_sha256_chain_valid(self):
        self.assertEqual(self.good("status")[1][0]["local_journal"],
                         "SHA256_CHAIN_VALID")

    def test_06_second_bootstrap_cannot_overwrite_existing_anchor(self):
        self.denied("ANCHOR_BOOTSTRAP_ROOT_ALREADY_EXISTS","bootstrap")

    def test_07_first_policy_and_real_portable_pass_inspect(self):
        self.assertEqual(self.good("inspect")[1][0]["status"],"POLICY_MATCH")

    def test_08_replay_real_rust_with_persisted_anchor_passes(self):
        r=self.good("verify")[1][0]
        self.assertEqual(r["native_jobs_recomputed"],2)
        self.assertEqual(r["native_files_bytewise_equal"],4)

    def test_09_local_anchor_gates_real_native_restore(self):
        r=self.good("restore")[1][0]
        self.assertEqual(r["status"],"RESTORED")
        self.assertEqual(r["local_freshness"],"VERIFIED_BEFORE_RESTORE")

    def test_10_real_native_resume_and_app010_full_audit(self):
        self.good("restore")
        p,rows=self.good("resume")
        self.assertEqual([x["step"] for x in rows],[3,4])
        whole=self.fx.fx.work/"full-freshness-dag.jsonl"
        whole.write_bytes((self.fx.fx.portable/"handoff/native-prefix.jsonl").read_bytes()
                          +p.stdout)
        report,code=freshness.portable.independent.audit(
            self.fx.fx.dest,self.fx.fx.portable/"handoff/plan.json",
            whole,self.fx.fx.policy["plan_sha256"])
        self.assertEqual(code,0,report)
        self.assertEqual(report["status"],"PASS")

    def test_11_second_native_resume_denied_again(self):
        self.good("restore")
        self.good("resume")
        self.denied("ROOT_UNEXPECTED_JOB_OR_FILE","resume")

    def test_12_monotonic_advance_to_generation_two(self):
        result=self.rotate()
        self.assertEqual(result["roster_generation"],2)

    def test_13_local_journal_has_second_immutable_entry(self):
        self.rotate()
        self.assertTrue((self.root/"history/entry-00000002.json").is_file())
        self.assertEqual(self.good("status")[1][0]["sequence"],2)

    def test_14_old_roster_plus_old_sha_is_refused_after_restart(self):
        self.rotate()
        self.denied("ROSTER_EXTERNAL_PIN_MISMATCH","inspect",
                    roster_file=self.old_roster_file,
                    policy_file=self.fx.original_policy,
                    package=self.fx.original_pkg)

    def test_15_old_original_policy_cannot_regain_active_rights(self):
        self.rotate()
        self.denied("RECEIVER_POLICY_EXTERNAL_PIN_MISMATCH",
                    "inspect",policy_file=self.fx.original_policy,
                    package=self.fx.original_pkg)

    def test_16_latest_policy_inspection_passes_v2(self):
        self.rotate()
        self.assertEqual(self.good("inspect")[1][0]["roster_generation"],2)

    def test_17_latest_receiver_policy_replays_real_alternative_source(self):
        self.rotate()
        r=self.good("verify",source=self.fx.fx.alt_source)[1][0]
        self.assertEqual(r["native_jobs_recomputed"],2)

    def test_18_latest_receiver_policy_refuses_old_noi_source(self):
        self.rotate()
        self.denied("SOURCE_PIN_MISMATCH","verify")

    def test_19_rotated_policy_allows_exclusive_native_restore(self):
        self.rotate()
        self.assertEqual(self.good("restore",source=self.fx.fx.alt_source)[1][0]["status"],
                         "RESTORED")

    def test_20_rotated_policy_native_resume_succeeds(self):
        self.rotate()
        self.good("restore",source=self.fx.fx.alt_source)
        p,rows=self.good("resume",source=self.fx.fx.alt_source)
        self.assertEqual([x["step"] for x in rows],[3,4])

    def test_21_stale_roster_cannot_advance_same_generation(self):
        self.denied("ROSTER_BELOW_RECEIVER_MINIMUM_GENERATION","advance")

    def test_22_wrong_new_roster_sha_denied_before_journal_write(self):
        self.fx.rotated()
        self.denied("ROSTER_EXTERNAL_PIN_MISMATCH","advance",
                    roster_sha="0"*64)
        self.assertEqual(self.good("status")[1][0]["sequence"],1)

    def test_23_stale_minimum_generation_higher_than_new_roster_denied(self):
        self.fx.rotated()
        self.denied("ROSTER_BELOW_RECEIVER_MINIMUM_GENERATION",
                    "advance",min_generation=3)

    def test_24_revoked_old_policy_cannot_be_readmitted_as_v3(self):
        self.rotate()
        old=self.fx.original_policy_sha
        current=self.fx.fx.anchor
        self.fx.roster.update({
            "generation":3,
            "state":"ACTIVE",
            "active_policy_sha256":old,
            "revoked_policy_sha256":[current],
        })
        self.fx.write_roster()
        self.denied("ANCHOR_PREVIOUSLY_REVOKED_POLICY_READMITTED",
                    "advance")

    def test_25_frozen_state_is_written_as_second_journal_event(self):
        self.freeze()
        r=self.good("status")[1][0]
        self.assertEqual(r["sequence"],2)
        self.assertEqual(r["status"],"FROZEN")

    def test_26_frozen_anchor_refuses_inspection(self):
        self.freeze()
        self.denied("ROSTER_FROZEN_DENIES_ADMISSION","inspect")

    def test_27_frozen_anchor_refuses_native_replay(self):
        self.freeze()
        self.denied("ROSTER_FROZEN_DENIES_ADMISSION","verify")

    def test_28_frozen_anchor_refuses_restore_without_effect(self):
        self.freeze()
        self.denied("ROSTER_FROZEN_DENIES_ADMISSION","restore")
        self.assertFalse(list(self.fx.fx.dest.iterdir()))

    def test_29_frozen_anchor_refuses_resume_without_effect(self):
        self.freeze()
        self.denied("ROSTER_FROZEN_DENIES_ADMISSION","resume")
        self.assertFalse(list(self.fx.fx.dest.iterdir()))

    def test_30_frozen_anchor_keeps_old_roster_blocked(self):
        self.freeze()
        self.denied("ROSTER_EXTERNAL_PIN_MISMATCH","inspect",
                    roster_file=self.old_roster_file,
                    policy_file=self.fx.original_policy,
                    package=self.fx.original_pkg)

    def test_31_existing_lock_denies_new_reader_fail_closed(self):
        lock=self.root/".lock"
        lock.mkdir()
        self.denied("ANCHOR_BUSY_OR_STALE_LOCK","status")

    def test_32_existing_lock_denies_rotation_fail_closed(self):
        self.fx.rotated()
        (self.root/".lock").mkdir()
        self.denied("ANCHOR_BUSY_OR_STALE_LOCK","advance")

    def test_33_manually_corrupted_head_is_denied(self):
        f=self.root/"HEAD.json"
        f.write_bytes(f.read_bytes()+b"x")
        self.denied("ANCHOR_HEAD_NOT_CANONICAL_JSON","status")

    def test_34_manually_corrupted_history_entry_is_denied(self):
        f=self.root/"history/entry-00000001.json"
        f.write_bytes(f.read_bytes()+b"xx")
        self.denied("ANCHOR_ENTRY_NOT_CANONICAL_JSON","status")

    def test_35_missing_history_entry_refuses_incomplete_chain(self):
        (self.root/"history/entry-00000001.json").unlink()
        self.denied("ANCHOR_HISTORY_COUNT_INVALID","status")

    def test_36_unexpected_extra_history_entry_refuses_chain(self):
        (self.root/"history/entry-99999999.json").write_bytes(b"{}\n")
        self.denied("ANCHOR_HISTORY_NONCONTIGUOUS_OR_EXTRA","status")

    def test_37_symlinked_head_is_refused_if_supported(self):
        real=self.fx.fx.work/"replacement-head.json"
        real.write_bytes((self.root/"HEAD.json").read_bytes())
        (self.root/"HEAD.json").unlink()
        try:
            (self.root/"HEAD.json").symlink_to(real)
        except (OSError,NotImplementedError):
            self.skipTest("native symlink not supported on runner")
        self.denied("ANCHOR_HEAD_NOT_REGULAR_BOUNDED","status")

    def test_38_missing_local_anchor_is_denied(self):
        self.denied("ANCHOR_ROOT_MISSING_OR_SYMLINKED","status",
                    root=self.fx.fx.work/"never-bootstrap")

    def test_39_head_rollback_against_new_append_only_history_denied(self):
        old=(self.root/"HEAD.json").read_bytes()
        self.rotate()
        (self.root/"HEAD.json").write_bytes(old)
        self.denied("ANCHOR_HEAD_HISTORY_MISMATCH","status")

    def test_40_report_disclaims_powercut_and_host_adversary_security(self):
        r=self.good("status")[1][0]
        self.assertEqual(r["power_loss_certification"],"NOT_PROVIDED")
        self.assertEqual(r["host_adversary_protection"],"NOT_PROVIDED")
        self.assertEqual(r["production"],"NOT_AUTHORIZED")


if __name__=="__main__":
    unittest.main()
