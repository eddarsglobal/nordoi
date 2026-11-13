"""NORDOI G0.3 R5Y: 24 real and adversarial local freshness cases.

16 portable canonical SQLite/nonce/floor controls, eight POSIX native actual
OpenSSH signatures and second real OS process restore. No independent
external monotonicity or native authority is represented by these tests.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r5y_signed_nonce_rollback as lab
import g03_r5x_signed_external_provenance as signed_receipt
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as osgate
import g03_r5t_signed_receipt_provenance as ssh
import g03_r5u_key_rotation_custody as keys

class PortableFreshness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.PREFIX)
        self.root = Path(self.t.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        a = osgate.observe("owner_proxy", "lab-owner-domain", "a" * 64, 0,
                           "b" * 64, "c" * 64)
        b = dict(a, role="custodian_proxy", pid=a["pid"]+1,
                 claimed_domain="lab-guardian-domain", key_sha256="d" * 64)
        self.ar, self.br = osgate.canonical(a), osgate.canonical(b)
        self.d = intake.canonical(intake.candidate(self.ar, self.br))
        self.raw = signed_receipt.canonical(
            signed_receipt.make_candidate(self.d, "d" * 64, 1000))
        self.doc = signed_receipt.parse(self.raw)
        lab.bootstrap(self.root)

    def tearDown(self):
        self.t.cleanup()

    def commit(self, raw=None, when=1100, floor=0):
        out = lab.commit_local_denial(
            self.root, self.raw if raw is None else raw, when, floor)
        self.assertEqual(out["status"], "BLOCK")
        self.assertEqual(out["admission"], "NOT_AUTHORIZED")
        self.assertFalse(out["native_authority_granted"])
        self.assertEqual(out["native_admission_gates_unmet"], 12)
        return out

    def test_01_canonical_local_receipt_proof_protocol(self):
        r = self.commit()
        self.assertEqual(r["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.assertEqual(r["local_height"], 1)
        self.assertEqual(len(r["canonical_local_proof_sha256"]), 64)

    def test_02_sqlite_actual_file_created_with_one_height(self):
        self.commit()
        state = lab.audit(self.root)
        self.assertTrue(lab.db_path(self.root).is_file())
        self.assertEqual(state["height"], 1)
        self.assertEqual(state["challenges"], ("c" * 64,))
        self.assertEqual(state["global_antirollback"], "NOT_IMPLEMENTED")

    def test_03_duplicate_spent_challenge_denied(self):
        self.commit()
        self.assertEqual(self.commit()["reason"], "LOCAL_SIGNED_NONCE_ALREADY_SPENT")
        self.assertEqual(lab.audit(self.root)["height"], 1)

    def test_04_different_signed_receipt_same_nonce_denied(self):
        self.commit()
        x = dict(self.doc, claimed_admin_domain="lab-alt-guardian")
        changed = signed_receipt.canonical(x)
        self.assertEqual(self.commit(changed)["reason"],
                         "LOCAL_SIGNED_NONCE_ALREADY_SPENT")

    def test_05_local_future_receipt_denied(self):
        self.assertEqual(self.commit(when=999)["reason"],
                         "LOCAL_UNTRUSTED_TIMEBOX_INVALID")
        self.assertEqual(lab.audit(self.root)["height"], 0)

    def test_06_local_expired_receipt_denied(self):
        self.assertEqual(self.commit(when=1301)["reason"],
                         "LOCAL_UNTRUSTED_TIMEBOX_INVALID")

    def test_07_local_counter_rejects_bool_negative_and_float(self):
        for when, floor in ((True, 0), (-1, 0), (0.2, 0),
                            (1100, True), (1100, -1), (1100, 0.2)):
            with self.assertRaises(lab.Refused):
                self.commit(when=when, floor=floor)

    def test_08_local_floor_progresses_but_not_global(self):
        self.commit(floor=3)
        self.assertEqual(lab.audit(self.root)["revocation_floor"], 3)
        self.assertEqual(lab.audit(self.root)["global_antirollback"],
                         "NOT_IMPLEMENTED")

    def test_09_new_challenge_with_floor_regression_refused(self):
        self.commit(floor=5)
        x = dict(self.doc, challenge="f" * 64)
        self.assertEqual(self.commit(signed_receipt.canonical(x), floor=4)["reason"],
                         "LOCAL_REVOCATION_FLOOR_REGRESSION")

    def test_10_old_snapshot_restore_loses_local_nonce(self):
        first = lab.db_path(self.root)
        snapshot = self.root / "old.sqlite3"
        shutil.copy2(first, snapshot)
        self.commit()
        self.assertEqual(lab.audit(self.root)["height"], 1)
        shutil.copy2(snapshot, first)
        self.assertEqual(lab.audit(self.root)["height"], 0)
        self.assertEqual(self.commit()["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")

    def test_11_local_json_proof_duplicate_and_noncanonical_rejected(self):
        self.commit()
        x = dict(protocol=lab.PROTOCOL, project=lab.PROJECT, parent=lab.PARENT,
                 status=lab.STATUS, receipt_sha256=lab.sha(self.raw),
                 challenge="c" * 64, history_head="b" * 64, key_epoch=0,
                 local_height=1, observed_at=1100, local_revocation_floor=0)
        valid = lab.canonical(x)
        self.assertEqual(lab.parse_proof(valid), x)
        for wrong in (valid.replace(b'"project":', b'"project":"fake","project":'),
                      json.dumps(x, indent=2).encode(),
                      lab.canonical(dict(x, grant=True))):
            with self.assertRaises(lab.Refused):
                lab.parse_proof(wrong)

    def test_12_local_proof_spoofed_source_and_grant_rejected(self):
        x = dict(protocol=lab.PROTOCOL, project=lab.PROJECT, parent=lab.PARENT,
                 status=lab.STATUS, receipt_sha256=lab.sha(self.raw),
                 challenge="c" * 64, history_head="b" * 64, key_epoch=0,
                 local_height=1, observed_at=1100, local_revocation_floor=0)
        for change in ({"parent": "0" * 40}, {"status": "GRANT"},
                       {"project": "imposter/repo"}):
            with self.assertRaises(lab.Refused):
                lab.parse_proof(lab.canonical(dict(x, **change)))

    def test_13_local_proof_malformed_digest_height_or_epoch(self):
        x = dict(protocol=lab.PROTOCOL, project=lab.PROJECT, parent=lab.PARENT,
                 status=lab.STATUS, receipt_sha256=lab.sha(self.raw),
                 challenge="c" * 64, history_head="b" * 64, key_epoch=0,
                 local_height=1, observed_at=1100, local_revocation_floor=0)
        for change in ({"receipt_sha256": "bad"}, {"challenge": "Z" * 64},
                       {"local_height": 0}, {"key_epoch": True},
                       {"observed_at": -1}):
            with self.assertRaises(lab.Refused):
                lab.parse_proof(lab.canonical(dict(x, **change)))

    def test_14_lab_root_invalid_outside_or_symlink(self):
        with self.assertRaises(lab.Refused):
            lab.private_root(self.root.parent)
        alias = self.root.parent / (self.root.name + "-symlink")
        try:
            alias.symlink_to(self.root, target_is_directory=True)
        except (OSError, NotImplementedError):
            pass
        else:
            try:
                with self.assertRaises(lab.Refused):
                    lab.private_root(alias)
            finally:
                alias.unlink()

    def test_15_same_file_bootstrap_refused_without_overwrite(self):
        with self.assertRaises(FileExistsError):
            lab.bootstrap(self.root)
        self.assertEqual(lab.audit(self.root)["height"], 0)

    def test_16_unsigned_structural_local_entry_is_not_external_proof(self):
        self.assertEqual(self.commit()["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.assertEqual(lab.audit(self.root)["global_antirollback"],
                         "NOT_IMPLEMENTED")
        self.assertEqual(lab.deny("RESEARCH_ONLY")["external_monotonicity"],
                         "NOT_VERIFIED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "Genuine POSIX local SSHSIG and actual second process required")
class NativeSignedFreshness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.PREFIX)
        self.root = Path(self.t.name)
        os.chmod(self.root, 0o700)
        self.ok, _ = ssh.create_same_admin_key(self.root, "r5y_owner")
        self.ck, self.allow = ssh.create_same_admin_key(self.root, "r5y_guardian")
        self.op = keys.public_sha(self.root / "r5y_owner.pub")
        self.cp = keys.public_sha(self.root / "r5y_guardian.pub")
        self.pub = self.root / "r5y_guardian.pub"
        self.signatures = []
        lab.bootstrap(self.root)

    def tearDown(self):
        self.t.cleanup()

    def signed(self, nonce="c" * 64):
        owner = osgate.child_observe("owner_proxy", "lab-owner-domain",
                                     self.op, 0, "b" * 64, nonce)
        candidate = osgate.child_observe("custodian_proxy", "lab-guardian-domain",
                                         self.cp, 0, "b" * 64, nonce)
        ar, br = osgate.canonical(owner), osgate.canonical(candidate)
        dossier = intake.canonical(intake.candidate(ar, br))
        receipt = signed_receipt.canonical(
            signed_receipt.make_candidate(dossier, self.cp, 1000))
        path = self.root / ("receipt_" + nonce[:8] + ".json")
        path.write_bytes(receipt)
        os.chmod(path, 0o600)
        sig = signed_receipt.sign_local(path, self.ck)
        self.signatures.append((receipt, dossier, ar, br, sig))
        return self.signatures[-1]

    def verified(self, fixture, raw=None, allowed=None, now=1100):
        receipt, dossier, ar, br, sig = fixture
        return signed_receipt.verify(
            receipt if raw is None else raw, dossier, ar, br, now,
            sig, self.allow if allowed is None else allowed, self.pub)

    def test_17_genuine_local_openssh_signature_with_same_uid_denies(self):
        fixture = self.signed()
        out = self.verified(fixture)
        self.assertTrue(out["cryptographic_signature_valid"])
        self.assertEqual(out["reason"],
                         "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN")
        self.assertFalse(out["signed_receipt_is_external_authority"])

    def test_18_real_signed_nonce_first_committed_then_replay_blocked(self):
        fixture = self.signed()
        self.assertTrue(self.verified(fixture)["cryptographic_signature_valid"])
        raw = fixture[0]
        self.assertEqual(lab.commit_local_denial(self.root, raw, 1100)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.assertEqual(lab.commit_local_denial(self.root, raw, 1100)["reason"],
                         "LOCAL_SIGNED_NONCE_ALREADY_SPENT")

    def test_19_actual_tampered_signature_rejected_and_not_authority(self):
        fixture = self.signed()
        bad = signed_receipt.canonical(dict(signed_receipt.parse(fixture[0]),
                                             claimed_admin_domain="lab-forged-domain"))
        r = self.verified(fixture, raw=bad)
        self.assertEqual(r["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")
        self.assertEqual(r["admission"], "NOT_AUTHORIZED")

    def test_20_wrong_valid_local_signer_rejected(self):
        fixture = self.signed()
        wrong = self.root / "r5y_wrong.allowed_signers"
        wrong.write_text("same-admin-research-fixture " +
                         (self.root / "r5y_owner.pub").read_text("ascii"),
                         encoding="ascii")
        os.chmod(wrong, 0o600)
        out = self.verified(fixture, allowed=wrong)
        self.assertEqual(out["reason"], "SIGNED_PROVENANCE_EVIDENCE_QUARANTINED")

    def test_21_real_signed_stale_timebox_rejected(self):
        fixture = self.signed()
        out = self.verified(fixture, now=1400)
        self.assertEqual(out["reason"], "UNTRUSTED_CLOCK_TIMEBOX_STALE_OR_FUTURE")

    def test_22_local_signed_floor_regression_still_no_global_floor(self):
        fixture = self.signed()
        self.assertTrue(self.verified(fixture)["cryptographic_signature_valid"])
        lab.commit_local_denial(self.root, fixture[0], 1100, 7)
        other = self.signed("d" * 64)
        self.assertEqual(lab.commit_local_denial(self.root, other[0], 1100, 6)["reason"],
                         "LOCAL_REVOCATION_FLOOR_REGRESSION")
        self.assertEqual(lab.audit(self.root)["global_antirollback"],
                         "NOT_IMPLEMENTED")

    def test_23_real_second_os_process_coherent_rollback_reissues_spent_nonce(self):
        first = self.signed()
        second = self.signed("d" * 64)
        lab.commit_local_denial(self.root, first[0], 1100, 1)
        old = self.root / "before_second.sqlite3"
        shutil.copy2(lab.db_path(self.root), old)
        os.chmod(old, 0o600)
        lab.commit_local_denial(self.root, second[0], 1100, 2)
        self.assertEqual(lab.audit(self.root)["height"], 2)
        helper = lab.child_restore(self.root, old)
        self.assertNotEqual(helper["pid"], os.getpid())
        self.assertEqual(helper["uid"], os.getuid())
        self.assertEqual(lab.audit(self.root)["height"], 1)
        self.assertTrue(self.verified(second)["cryptographic_signature_valid"])
        self.assertEqual(lab.commit_local_denial(self.root, second[0], 1100, 2)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")

    def test_24_genuine_full_native_exercise_denied_exit_two(self):
        proc = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                               "--exercise"], capture_output=True, text=True,
                              timeout=180, check=False)
        self.assertEqual(proc.returncode, 2, proc.stderr)
        result = json.loads(proc.stdout)
        self.assertEqual(result["reason"],
                         "REAL_SSHSIG_NONCE_ROLLBACK_COUNTEREXAMPLE_QUALIFIED")
        self.assertEqual(result["genuine_second_process_restore"], "PASS")
        self.assertEqual(result["old_spent_nonce_after_coherent_restore"],
                         "REISSUED_LOCALLY")
        self.assertEqual(result["native_admission"], "BLOCK")
        self.assertEqual(result["native_admission_gates_unmet"], 12)

if __name__ == "__main__":
    unittest.main()
