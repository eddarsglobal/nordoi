"""NORDOI R4Y negative assurance: 18 adversarial evidence intake tests."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).parent))
import g03_r4y_external_custody_intake as audit
def inventory(items=()):
    return json.dumps({"version": audit.VERSION, "project": audit.PROJECT,
                       "evidence": list(items)}, separators=(",", ":")).encode()
def item(gate, source="CLAIMED_INDEPENDENT_CONTROLLER"):
    return {"gate": gate, "source": source, "reference": "reference-" + gate,
            "sha256": "a" * 64}
class R4YCustodyAdmissionTests(unittest.TestCase):
    def assert_blocked(self, data):
        out = audit.audit_bytes(data)
        self.assertEqual(out["status"], "BLOCK")
        self.assertEqual(out["admission"], "NOT_AUTHORIZED")
        self.assertNotIn("PASS", str(out))
        return out
    def test_01_real_unenrolled_baseline_is_blocked(self):
        self.assertEqual(self.assert_blocked(inventory())["missing_gates"], list(audit.GATES))
    def test_02_all_six_self_claims_are_not_independent_proof(self):
        out = self.assert_blocked(inventory(item(g) for g in audit.GATES))
        self.assertEqual(out["reason"], "CLAIMS_NOT_EXTERNALLY_AUTHENTICATED")
        self.assertEqual(out["missing_gates"], [])
    def test_03_repository_witness_same_admin_not_qualified(self):
        out = self.assert_blocked(inventory(item(g, "REPOSITORY_ONLY") for g in audit.GATES))
        self.assertEqual(out["external_operator"], "NOT_VERIFIED")
    def test_04_owner_local_only_cannot_substitute_out_of_band(self):
        out = self.assert_blocked(inventory(item(g, "OWNER_LOCAL_RECORD") for g in audit.GATES))
        self.assertEqual(out["owner_out_of_band_root"], "NOT_VERIFIED")
    def test_05_out_of_band_string_claim_is_untrusted(self):
        self.assert_blocked(inventory(item(g, "CLAIMED_OUT_OF_BAND") for g in audit.GATES))
    def test_06_witness_name_not_protected_monotonic_proof(self):
        out = self.assert_blocked(inventory(item(g, "CLAIMED_PROTECTED_WITNESS") for g in audit.GATES))
        self.assertEqual(out["independent_monotonic_witness"], "NOT_VERIFIED")
        self.assertEqual(out["durable_nonce_ledger"], "NOT_VERIFIED")
    def test_07_duplicate_top_level_field_deny(self):
        out = self.assert_blocked(b'{"version":"x","version":"x","project":"x","evidence":[]}')
        self.assertIn("DUPLICATE_JSON_KEY", out["detail"])
    def test_08_duplicate_nested_field_deny(self):
        bad = inventory([item("G1")]).decode().replace('"gate":"G1"', '"gate":"G1","gate":"G2"')
        self.assertIn("DUPLICATE_JSON_KEY", self.assert_blocked(bad.encode())["detail"])
    def test_09_duplicate_gate_entries_deny(self):
        self.assert_blocked(inventory([item("G1"), item("G1")]))
    def test_10_unknown_gate_or_field_deny(self):
        self.assert_blocked(inventory([item("G7")]))
        record = item("G1")
        record["authorized"] = True
        self.assert_blocked(inventory([record]))
    def test_11_wrong_project_and_version_deny(self):
        self.assert_blocked(inventory().replace(audit.PROJECT.encode(), b"other/repo"))
        self.assert_blocked(inventory().replace(audit.VERSION.encode(), b"R4Yv0"))
    def test_12_bad_digest_and_reference_deny(self):
        record = item("G1")
        record["sha256"] = "a" * 63
        self.assert_blocked(inventory([record]))
        record = item("G1")
        record["reference"] = "../../private"
        self.assert_blocked(inventory([record]))
    def test_13_invalid_utf8_json_and_oversized_deny(self):
        for data in (b"\xff", b"{", b"x" * (audit.MAX_BYTES + 1), b""):
            self.assert_blocked(data)
    def test_14_booleans_or_nulls_cannot_replace_provenance(self):
        record = item("G1")
        record["source"] = True
        self.assert_blocked(inventory([record]))
        record["source"] = None
        self.assert_blocked(inventory([record]))
    def test_15_regular_file_audit_is_read_only_and_blocking(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "manifest.json"
            content = inventory(item(g) for g in audit.GATES)
            path.write_bytes(content)
            before = sorted(p.name for p in Path(folder).iterdir())
            self.assertEqual(audit.audit_file(path)["status"], "BLOCK")
            self.assertEqual(path.read_bytes(), content)
            self.assertEqual(before, sorted(p.name for p in Path(folder).iterdir()))
    @unittest.skipUnless(hasattr(os, "O_NOFOLLOW"), "O_NOFOLLOW absent")
    def test_16_symlink_alias_is_rejected_by_file_intake(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "real.json"
            link = Path(folder) / "alias.json"
            target.write_bytes(inventory())
            try:
                link.symlink_to(target)
            except OSError:
                self.skipTest("Symlink creation unavailable")
            with self.assertRaises((OSError, audit.InvalidIntake)):
                audit.audit_file(link)
    def test_17_cli_cannot_return_success_for_complete_self_claims(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "intake.json"
            path.write_bytes(inventory(item(g) for g in audit.GATES))
            out = subprocess.run([sys.executable, "-B", str(Path(audit.__file__)),
                                  "--manifest", str(path)], capture_output=True, text=True, check=False)
            self.assertEqual(out.returncode, 2)
            self.assertEqual(json.loads(out.stdout)["status"], "BLOCK")
            self.assertEqual(out.stderr, "")
    def test_18_no_manifest_never_implicitly_admits(self):
        with tempfile.TemporaryDirectory() as folder:
            out = subprocess.run([sys.executable, "-B", str(Path(audit.__file__)),
                                  "--manifest", str(Path(folder) / "absent")],
                                 capture_output=True, text=True, check=False)
            self.assertEqual(out.returncode, 2)
            self.assertEqual(json.loads(out.stdout)["reason"], "EVIDENCE_FILE_UNAVAILABLE_OR_UNSAFE")
if __name__ == "__main__":
    unittest.main()
