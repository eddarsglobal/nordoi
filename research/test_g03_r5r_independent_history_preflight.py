"""G0.3 R5R real external-historical-authority qualification — deny only.

16 portable actual SQLite/JSON input and strict canonical falsification tests;
8 POSIX tests with actual OpenSSH Ed25519, second OS process replacement and
the genuine earlier R5Q coherent three-file rollback counterexample.

A local self-signed claim is NOT an independent external guardian. NO ADMISSION.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5l_crash_restart_denial_journal as denial
import g03_r5q_coherent_local_rollback as histories
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5r_independent_history_preflight as lab

class PortableAuthorityPreflight(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5r-portable-")
        self.root=Path(self.temp.name)
        if os.name!="nt":os.chmod(self.root,0o700)
        self.a,self.b,self.j=histories.fixture(self.root)
        self.projection=histories.projection(self.a,self.b,self.j)
        self.bundle=self.root/"candidate.json"
        self.bundle.write_bytes(lab.create_claim(self.projection))
        if os.name!="nt":os.chmod(self.bundle,0o600)
    def tearDown(self):self.temp.cleanup()
    def check(self,path=None):
        result=lab.examine(self.a,self.b,self.j,
                           self.bundle if path is None else path)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["independent_history_authority"],"NOT_ENROLLED")
        self.assertEqual(result["external_monotonic_highwater"],"NOT_VERIFIED")
        return result
    def claim(self,**kwargs):
        d=lab.parse(self.bundle.read_bytes())
        d.update(kwargs)
        return lab.canonical(d)
    def write(self,payload):
        self.bundle.write_bytes(payload)
        if os.name!="nt":os.chmod(self.bundle,0o600)
    def refused(self,payload):
        with self.assertRaises(lab.Refused):lab.parse(payload)
    def test_01_real_three_local_stores_and_no_remote_custodian(self):
        self.assertEqual(self.projection["primary_height"],1)
        self.assertEqual(self.projection["local_spent_nonce_height"],1)
        self.assertTrue(self.projection["all_local_authority_only"])
        self.assertEqual(self.check()["reason"],"SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
    def test_02_twelve_exact_independent_external_requirements(self):
        self.assertEqual(len(lab.GATES),12)
        d=lab.parse(self.bundle.read_bytes())
        self.assertEqual(set(d["self_declared_controls"]),set(lab.GATES))
        self.assertEqual(d["parent"],"8d15f9661d97a8cc30a1eca66322be28ca500abc")
    def test_03_absent_external_bundle_is_not_implicit_enrollment(self):
        r=lab.examine(self.a,self.b,self.j,None)
        self.assertEqual(r["reason"],"REAL_EXTERNAL_CUSTODY_EVIDENCE_NOT_PROVIDED")
        self.assertFalse(r["external_provenance_verified"])
    def test_04_all_twelve_true_locally_still_never_authorize(self):
        r=self.check()
        self.assertEqual(r["reason"],"SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
        self.assertTrue(r["all_twelve_self_claimed"])
        self.assertFalse(r["independently_enrolled_owner_root"])
    def test_05_one_false_external_claim_is_not_a_satisfied_gate(self):
        doc=lab.parse(self.bundle.read_bytes())
        doc["self_declared_controls"][lab.GATES[3]]=False
        self.write(lab.canonical(doc))
        self.assertEqual(self.check()["reason"],"REQUIRED_EXTERNAL_CONTROL_CLAIM_INCOMPLETE")
    def test_06_candidate_monotonic_height_lower_than_real_local(self):
        self.write(self.claim(claimed_external_height=0))
        self.assertIn("CLAIMED_EXTERNAL_HEIGHT_BEHIND_LOCAL",self.check()["detail"])
    def test_07_candidate_global_nonce_counter_behind_real_local(self):
        self.write(self.claim(claimed_external_spent_nonces=0))
        self.assertIn("CLAIMED_GLOBAL_NONCE_FLOOR_BEHIND_LOCAL",self.check()["detail"])
    def test_08_local_history_digest_must_bind_real_sqlite_and_witness(self):
        self.write(self.claim(local_projection_sha256="0"*64))
        self.assertIn("LOCAL_ASSERTION_WRONG_HISTORY_DIGEST",self.check()["detail"])
    def test_09_protocol_parent_project_or_grant_operation_rejected(self):
        d=lab.parse(self.bundle.read_bytes())
        for field,value in (("protocol","bad"),("parent","0"*40),
                            ("project","other"),("operation","GRANT"),
                            ("operation","ENROLL")):
            changed=dict(d);changed[field]=value
            self.refused(lab.canonical(changed))
    def test_10_external_control_keys_cannot_be_missing_or_added(self):
        d=lab.parse(self.bundle.read_bytes())
        del d["self_declared_controls"][lab.GATES[0]]
        self.refused(lab.canonical(d))
        d=lab.parse(lab.create_claim(self.projection))
        d["self_declared_controls"]["invented_external_guardian"]=True
        self.refused(lab.canonical(d))
    def test_11_noncanonical_duplicate_or_oversize_input_refused(self):
        self.refused(json.dumps(lab.parse(self.bundle.read_bytes()),indent=2).encode())
        self.refused(self.bundle.read_bytes().replace(b'"project":',
                                                    b'"project":"fake","project":'))
        self.refused(b"X"*(lab.MAX_BYTES+1))
    def test_12_claimed_guardian_name_is_not_untrusted_freeform_code(self):
        for name in ("", "../guardian", "Guardian With Spaces", "x", "root@other"):
            self.refused(self.claim(claimed_guardian_id=name))
    def test_13_false_types_for_counters_nonce_root_and_control_rejected(self):
        for key,value in (("claimed_external_height",True),
                          ("claimed_external_spent_nonces",-1),
                          ("nonce","x"*64),("claimed_owner_root_sha256","short")):
            self.refused(self.claim(**{key:value}))
        d=lab.parse(self.bundle.read_bytes())
        d["self_declared_controls"][lab.GATES[0]]="true"
        self.refused(lab.canonical(d))
    def test_14_all_twelve_false_still_no_external_enrollment(self):
        self.write(lab.create_claim(self.projection,"a"*64,all_claims=False))
        self.assertEqual(self.check()["reason"],"REQUIRED_EXTERNAL_CONTROL_CLAIM_INCOMPLETE")
    def test_15_real_local_json_witness_corruption_disqualifies_claim(self):
        witness=self.b/"g03_r5m_local_witness.json"
        # Use the actual known witness path, not an invented document file.
        from g03_r5m_file_integrity_quarantine import WITNESS
        witness=self.b/WITNESS
        witness.write_bytes(b"{}")
        self.assertEqual(self.check()["reason"],"UNTRUSTED_HISTORY_CUSTODY_EVIDENCE_QUARANTINED")
    def test_16_real_fresh_python_process_cli_can_only_report_block(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--assess",
             "--primary-dir",str(self.a),"--witness-dir",str(self.b),
             "--journal-dir",str(self.j),"--candidate-file",str(self.bundle)]
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=45)
        self.assertEqual(p.returncode,2,p.stderr)
        data=json.loads(p.stdout)
        self.assertEqual(data["reason"],"SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
        self.assertFalse(data["native_authority_granted"])

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "real POSIX OpenSSH signature and second OS process evidence test")
class NativeExternalCustodyNegativeControls(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5r-native-")
        self.root=Path(self.temp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=histories.fixture(self.root)
        self.projection=histories.projection(self.a,self.b,self.j)
        self.bundle=self.root/"candidate.json"
        self.bytes=lab.create_claim(self.projection)
        self.bundle.write_bytes(self.bytes);os.chmod(self.bundle,0o600)
    def tearDown(self):self.temp.cleanup()
    def check(self,**kw):
        result=lab.examine(self.a,self.b,self.j,self.bundle,**kw)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        return result
    def test_17_real_local_ed25519_signature_is_valid_key_possession_only(self):
        valid,forgery=lab.actual_same_user_sign_and_verify(self.root,self.bytes)
        self.assertTrue(valid)
        self.assertTrue(forgery)
    def test_18_signed_locally_claimed_all_twelve_true_must_still_refuse(self):
        valid,_=lab.actual_same_user_sign_and_verify(self.root,self.bytes)
        self.assertTrue(valid)
        self.assertEqual(self.check()["reason"],
                         "SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
    def test_19_real_separate_os_process_swaps_identical_candidate_bytes(self):
        duplicate=self.root/"same_claim_bytes.json"
        shutil.copy2(self.bundle,duplicate);os.chmod(duplicate,0o600)
        x=self.check(after_first_observation=lambda:
                     fdgate.real_atomic_replace(duplicate,self.bundle))
        self.assertIn("OPEN_FD_AND_CURRENT_PATH_DIVERGED",x["detail"])
        self.assertEqual(x["reason"],"UNTRUSTED_HISTORY_CUSTODY_EVIDENCE_QUARANTINED")
    def test_20_real_world_b_history_rejects_old_a_credential_digest(self):
        histories.advance_world(self.a,self.b,self.j)
        self.assertIn("LOCAL_ASSERTION_WRONG_HISTORY_DIGEST",self.check()["detail"])
    def test_21_real_three_file_joint_rollback_reenables_old_claim_not_authority(self):
        snapshot=histories.file_snapshots(self.root,self.a,self.b,self.j,"history_a")
        histories.advance_world(self.a,self.b,self.j)
        self.assertEqual(self.check()["reason"],"UNTRUSTED_HISTORY_CUSTODY_EVIDENCE_QUARANTINED")
        histories.actual_second_process_restore(self.root,snapshot,self.a,self.b,self.j)
        self.assertEqual(self.check()["reason"],
                         "SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
    def test_22_deleted_assertion_file_never_silently_enrolls_root(self):
        self.bundle.unlink()
        self.assertEqual(self.check()["reason"],"UNTRUSTED_HISTORY_CUSTODY_EVIDENCE_QUARANTINED")
        self.assertFalse(self.bundle.exists())
    def test_23_fresh_os_process_after_full_rollback_still_has_no_external_root(self):
        snapshot=histories.file_snapshots(self.root,self.a,self.b,self.j,"history_a")
        histories.advance_world(self.a,self.b,self.j)
        histories.actual_second_process_restore(self.root,snapshot,self.a,self.b,self.j)
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--assess",
             "--primary-dir",str(self.a),"--witness-dir",str(self.b),
             "--journal-dir",str(self.j),"--candidate-file",str(self.bundle)]
        p=subprocess.run(cmd,text=True,capture_output=True,timeout=40)
        self.assertEqual(p.returncode,2,p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["reason"],"SELF_ASSERTED_EXTERNAL_CUSTODY_NOT_VERIFIABLE")
        self.assertEqual(r["independent_history_authority"],"NOT_ENROLLED")
    def test_24_actual_native_disqualification_exercise_ends_with_block(self):
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--exercise"]
        p=subprocess.run(cmd,text=True,capture_output=True,timeout=180)
        self.assertEqual(p.returncode,2,p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["reason"],"REAL_EXTERNAL_HISTORY_AUTHORITY_ABSENT_CONFIRMED")
        self.assertEqual(r["real_same_uid_ed25519_signature"],"VALID_KEY_POSSESSION_ONLY")
        self.assertEqual(r["altered_local_signed_claim"],"SIGNATURE_REJECTED")
        self.assertEqual(r["all_twelve_claims_without_real_oob_root"],"BLOCK")
        self.assertEqual(r["stale_claim_after_real_history_advance"],"QUARANTINED")
        self.assertEqual(r["old_local_history_after_joint_rollback"],"SELF_ASSERTED_ONLY")
        self.assertEqual(r["real_second_process_identical_bundle_swap"],"QUARANTINED")
        self.assertFalse(r["verified_external_legal_custodian"])
        self.assertFalse(r["verified_external_monotonic_height"])
        self.assertFalse(r["native_authority_granted"])

if __name__=="__main__":unittest.main()
