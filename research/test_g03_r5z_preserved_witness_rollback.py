"""G0.3 R5Z — 24 deny-only preserved witness and coherent rollback cases.

16 portable structural/local SQLite cases, eight genuinely signed POSIX
OpenSSH and second-process rollback cases; no independent remote actor exists.
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
import g03_r5z_preserved_witness_rollback as lab
import g03_r5y_signed_nonce_rollback as nonce
import g03_r5x_signed_external_provenance as receipt
import g03_r5w_external_admin_intake as intake
import g03_r5v_admin_separation_evidence as osgate
import g03_r5u_key_rotation_custody as keys
import g03_r5t_signed_receipt_provenance as signed

class PortableLocalWitness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.PREFIX)
        self.root = Path(self.t.name)
        if os.name != "nt":
            os.chmod(self.root, 0o700)
        self.a = osgate.observe("owner_proxy", "lab-owner-domain",
                                "a" * 64, 0, "b" * 64, "c" * 64)
        self.b = dict(self.a, role="custodian_proxy",
                      claimed_domain="lab-guardian-domain",
                      pid=self.a["pid"] + 1, key_sha256="d" * 64)
        nonce.bootstrap(self.root)
        self.r1 = self.make_receipt("c" * 64)
        self.r2 = self.make_receipt("d" * 64)
        self.r3 = self.make_receipt("e" * 64)
        self.assertEqual(nonce.commit_local_denial(self.root, self.r1, 1100, 1)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.h1 = lab.canonical(lab.candidate(self.root, self.r1, 1100))

    def tearDown(self):
        self.t.cleanup()

    def make_receipt(self, value):
        a, b = dict(self.a, nonce=value), dict(self.b, nonce=value)
        dossier = intake.canonical(intake.candidate(
            osgate.canonical(a), osgate.canonical(b)))
        return receipt.canonical(receipt.make_candidate(dossier, "d" * 64, 1000))

    def second(self):
        self.assertEqual(nonce.commit_local_denial(self.root, self.r2, 1100, 2)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        return lab.canonical(lab.candidate(self.root, self.r2, 1100))

    def assess(self, raw):
        d = lab.compare(self.root, raw)
        self.assertEqual(d["status"], "BLOCK")
        self.assertEqual(d["admission"], "NOT_AUTHORIZED")
        self.assertFalse(d["native_authority_granted"])
        self.assertEqual(d["native_admission_gates_unmet"], 12)
        return d

    def test_01_canonical_current_local_witness(self):
        self.assertEqual(lab.parse(self.h1)["local_height"], 1)
        self.assertEqual(lab.parse(self.h1)["custody"], "SAME_ADMIN_LOCAL_ONLY")
        self.assertEqual(set(lab.parse(self.h1)), lab.FIELDS)

    def test_02_source_project_signature_free_witness_not_authority(self):
        x = lab.parse(self.h1)
        self.assertEqual(x["parent"], lab.PARENT)
        self.assertIsNone(x["owner_oob_root"])
        self.assertIsNone(x["external_guardian"])
        self.assertEqual(x["admission"], "NOT_AUTHORIZED")

    def test_03_duplicate_json_field_refused(self):
        with self.assertRaises(lab.Refused):
            lab.parse(self.h1.replace(b'"protocol":', b'"protocol":"fake","protocol":'))

    def test_04_missing_extra_and_pretty_json_refused(self):
        x = lab.parse(self.h1)
        for raw in (lab.canonical(dict(x, native_grant=True)),
                    lab.canonical({k: v for k, v in x.items() if k != "project"}),
                    json.dumps(x, indent=2).encode(), self.h1.rstrip(b"\n")):
            with self.assertRaises(lab.Refused):
                lab.parse(raw)

    def test_05_invalid_utf8_and_oversized_quarantined(self):
        for raw in (b"\xff", b"x" * 2049, b""):
            with self.assertRaises(lab.Refused):
                lab.parse(raw)

    def test_06_wrong_frozen_source_and_privileged_status_refused(self):
        x = lab.parse(self.h1)
        for d in ({"parent": "0" * 40}, {"protocol": "fake"},
                  {"project": "fake/repo"}, {"admission": "GRANT"},
                  {"custody": "EXTERNAL_CERTIFIED"}):
            with self.assertRaises(lab.Refused):
                lab.parse(lab.canonical(dict(x, **d)))

    def test_07_claimed_owner_root_or_guardian_forgery_rejected(self):
        x = lab.parse(self.h1)
        for field in ("owner_oob_root", "external_guardian"):
            with self.assertRaises(lab.Refused):
                lab.parse(lab.canonical(dict(x, **{field: "f" * 64})))

    def test_08_bogus_digest_counter_bool_refused(self):
        x = lab.parse(self.h1)
        for field, value in (
            ("nonce_set_sha256", "bad"), ("source_receipt_sha256", "f" * 40),
            ("local_height", 0), ("local_height", True),
            ("local_revocation_floor", -1), ("observed_at", 1.5)
        ):
            with self.assertRaises(lab.Refused):
                lab.parse(lab.canonical(dict(x, **{field: value})))

    def test_09_preserved_local_copy_matches_without_global_trust(self):
        r = self.assess(self.h1)
        self.assertEqual(r["reason"], "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL")
        self.assertFalse(r["globally_protected_witness"])

    def test_10_local_primary_advances_beyond_old_witness(self):
        self.second()
        self.assertEqual(self.assess(self.h1)["reason"],
                         "PRIMARY_AHEAD_OF_OLDER_LOCAL_WITNESS")

    def test_11_retained_witness_detects_primary_only_restore(self):
        snapshot = self.root / "old.sqlite3"
        shutil.copy2(nonce.db_path(self.root), snapshot)
        h2 = self.second()
        shutil.copy2(snapshot, nonce.db_path(self.root))
        self.assertEqual(self.assess(h2)["reason"],
                         "PRESERVED_LOCAL_WITNESS_DETECTS_PRIMARY_ROLLBACK")

    def test_12_equal_height_but_different_nonce_set_detected(self):
        snapshot = self.root / "old.sqlite3"
        shutil.copy2(nonce.db_path(self.root), snapshot)
        h2 = self.second()
        shutil.copy2(snapshot, nonce.db_path(self.root))
        self.assertEqual(nonce.commit_local_denial(self.root, self.r3, 1100, 2)["reason"],
                         "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.assertEqual(self.assess(h2)["reason"],
                         "SAME_HEIGHT_OR_FLOOR_HISTORY_MISMATCH")

    def test_13_witness_saving_requires_fixed_path(self):
        with self.assertRaises(lab.Refused):
            lab.save_witness(self.root, self.h1, "external.json")
        self.assertFalse((self.root / "external.json").exists())

    def test_14_witness_cannot_overwrite_existing_copy(self):
        path = lab.save_witness(self.root, self.h1, lab.WITNESS)
        with self.assertRaises(FileExistsError):
            lab.save_witness(self.root, self.h1, lab.WITNESS)
        self.assertEqual(path.read_bytes(), self.h1)

    def test_15_no_arbitrary_directory_or_symlink_root(self):
        with self.assertRaises((lab.Refused, nonce.Refused)):
            lab.lab_root(self.root.parent)
        alias = self.root.parent / (self.root.name + "-alias")
        try:
            alias.symlink_to(self.root, target_is_directory=True)
        except (OSError, NotImplementedError):
            pass
        else:
            try:
                with self.assertRaises((lab.Refused, nonce.Refused)):
                    lab.lab_root(alias)
            finally:
                alias.unlink()

    def test_16_local_coherent_restored_witness_looks_consistent(self):
        old = lab.parse(self.h1)
        snap = self.root / "old.sqlite3"
        shutil.copy2(nonce.db_path(self.root), snap)
        self.second()
        shutil.copy2(snap, nonce.db_path(self.root))
        self.assertEqual(self.assess(lab.canonical(old))["reason"],
                         "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL")
        self.assertEqual(lab.deny("LOCAL_ONLY")["globally_monotonic_antirollback"],
                         "NOT_VERIFIED")

@unittest.skipUnless(os.name != "nt" and Path("/usr/bin/ssh-keygen").is_file(),
                     "real native POSIX OS processes and OpenSSH SSHSIG required")
class NativePreservedWitness(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix=lab.PREFIX)
        self.root = Path(self.t.name)
        os.chmod(self.root, 0o700)
        self.owner, self.owner_allow = signed.create_same_admin_key(
            self.root, "r5z_owner")
        self.guardian, self.allowed = signed.create_same_admin_key(
            self.root, "r5z_guardian")
        self.op = keys.public_sha(self.root / "r5z_owner.pub")
        self.gp = keys.public_sha(self.root / "r5z_guardian.pub")
        self.assertNotEqual(self.op, self.gp)
        nonce.bootstrap(self.root)
        self.first = self.signed_receipt("c" * 64, "receipt_one")
        self.second = self.signed_receipt("d" * 64, "receipt_two")
        self.assertEqual(nonce.commit_local_denial(
            self.root, self.first, 1100, 1)["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.old_db = self.root / "before_second.sqlite3"
        shutil.copy2(nonce.db_path(self.root), self.old_db)
        os.chmod(self.old_db, 0o600)
        self.h1 = lab.canonical(lab.candidate(self.root, self.first, 1100))
        self.old_witness = lab.save_witness(self.root, self.h1, lab.OLD_WITNESS)
        self.old_sig = lab.sign_local(self.old_witness, self.guardian)
        self.assertEqual(nonce.commit_local_denial(
            self.root, self.second, 1100, 2)["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")
        self.h2 = lab.canonical(lab.candidate(self.root, self.second, 1100))
        self.retained = lab.save_witness(self.root, self.h2, lab.WITNESS)
        self.retained_sig = lab.sign_local(self.retained, self.guardian)

    def tearDown(self):
        self.t.cleanup()

    def signed_receipt(self, value, filename):
        a = osgate.child_observe("owner_proxy", "lab-owner-domain",
                                 self.op, 0, "b" * 64, value)
        b = osgate.child_observe("custodian_proxy", "lab-guardian-domain",
                                 self.gp, 0, "b" * 64, value)
        self.assertNotEqual(a["pid"], b["pid"])
        self.assertEqual(a["uid"], b["uid"])
        self.assertEqual(a["euid"], b["euid"])
        ar, br = osgate.canonical(a), osgate.canonical(b)
        dossier = intake.canonical(intake.candidate(ar, br))
        raw = receipt.canonical(receipt.make_candidate(dossier, self.gp, 1000))
        path = self.root / (filename + ".json")
        path.write_bytes(raw)
        os.chmod(path, 0o600)
        sig = receipt.sign_local(path, self.guardian)
        verification = receipt.verify(raw, dossier, ar, br, 1100, sig,
                                      self.allowed, self.root / "r5z_guardian.pub")
        self.assertEqual(verification["reason"],
                         "REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN")
        return raw

    def test_17_genuine_signed_local_witness_not_external(self):
        v = lab.verify_local(self.h2, self.retained_sig, self.allowed)
        self.assertTrue(v["cryptographic_signature_valid"])
        self.assertEqual(v["reason"], "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL")
        self.assertEqual(v["native_admission_gates_unmet"], 12)

    def test_18_tampered_same_admin_witness_signature_invalid(self):
        x = dict(lab.parse(self.h2), observed_at=1101)
        v = lab.verify_local(lab.canonical(x), self.retained_sig, self.allowed)
        self.assertEqual(v["reason"], "SIGNED_LOCAL_WITNESS_QUARANTINED")
        self.assertIn("REAL_LOCAL_WITNESS_SSHSIG_INVALID", v["detail"])

    def test_19_valid_other_key_wrong_allowed_signer_rejected(self):
        v = lab.verify_local(self.h2, self.retained_sig, self.owner_allow)
        self.assertEqual(v["reason"], "SIGNED_LOCAL_WITNESS_QUARANTINED")

    def test_20_preserved_signed_witness_detects_primary_only_rollback(self):
        helper = nonce.child_restore(self.root, self.old_db)
        self.assertNotEqual(helper["pid"], os.getpid())
        self.assertEqual(helper["uid"], os.getuid())
        self.assertEqual(lab.compare(self.root, self.h2)["reason"],
                         "PRESERVED_LOCAL_WITNESS_DETECTS_PRIMARY_ROLLBACK")
        self.assertEqual(lab.verify_local(self.h2, self.retained_sig, self.allowed)[
                         "reason"], "REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL")

    def test_21_actual_joint_restore_local_history_and_signed_witness(self):
        nonce.child_restore(self.root, self.old_db)
        helper = lab.child_restore_witness(self.root, self.old_witness)
        self.assertEqual(helper["uid"], os.getuid())
        self.assertNotEqual(helper["pid"], os.getpid())
        now = self.retained.read_bytes()
        self.assertEqual(now, self.h1)
        self.assertEqual(lab.compare(self.root, now)["reason"],
                         "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL")
        self.assertTrue(lab.verify_local(now, self.retained_sig, self.allowed)[
                        "cryptographic_signature_valid"])
        self.assertEqual(nonce.commit_local_denial(
            self.root, self.second, 1100, 2)["reason"], "LOCAL_ONLY_SPENT_NONCE_COMMITTED")

    def test_22_outside_witness_restore_path_refused(self):
        other = self.root / "fake_witness.json"
        other.write_bytes(self.h1)
        with self.assertRaises(lab.Refused):
            lab.restore_witness_lab(self.root, other)
        self.assertEqual(self.retained.read_bytes(), self.h2)

    def test_23_fake_independent_credential_in_signed_schema_denied(self):
        x = dict(lab.parse(self.h2), external_guardian="a" * 64)
        with self.assertRaises(lab.Refused):
            lab.parse(lab.canonical(x))
        self.assertEqual(lab.compare(self.root, self.h2)["reason"],
                         "COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL")

    def test_24_true_full_native_process_exercise_exit_two(self):
        p = subprocess.run([sys.executable, "-B", str(Path(lab.__file__)),
                            "--exercise"], capture_output=True, text=True,
                           timeout=240, check=False)
        self.assertEqual(p.returncode, 2, p.stderr)
        result = json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "REAL_SIGNED_PRESERVED_VS_COHERENT_ROLLBACK_QUALIFIED")
        self.assertEqual(result["true_second_process_signed_witness_restore"], "PASS")
        self.assertEqual(result["coherent_both_history_restore"],
                         "LOCALLY_INDISTINGUISHABLE")
        self.assertEqual(result["native_admission"], "BLOCK")
        self.assertEqual(result["native_admission_gates_unmet"], 12)

if __name__ == "__main__":
    unittest.main()
