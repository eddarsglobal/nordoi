"""NORDOI R6F BUREAU-001 R2 — 24 read-only specialist-evidence refusal tests.

All actual file tampering happens in disposable fixture copies. The only
accepted optional submission outcome is STAGED_UNVERIFIED_LOCAL_REVIEW_LOG_ONLY.
"""
import copy
import hashlib
import json
from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts import bureau_001_r2_review_intake as lab

class ReviewIntake(unittest.TestCase):
    def setUp(self):
        self.t = tempfile.TemporaryDirectory(prefix="nordoi-r6f-r2-")
        self.root = Path(self.t.name)
        manifest = json.loads((ROOT/lab.previous.MANIFEST).read_text())
        files = ([r["path"] for r in manifest["pinned_files"]]
                 + ["CONSTITUTION.md",
                    "governance/future_native_gate_v1.tsv",
                    lab.previous.MANIFEST, lab.previous.CHRONOLOGY, lab.REGISTRY])
        for path in set(files):
            dest = self.root/path
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT/path, dest)
        self.record = {
            "schema": lab.CANDIDATE_SCHEMA,
            "project": "eddarsglobal/nordoi",
            "parent_head": lab.PARENT,
            "role": "CIPHER",
            "source_git_blob_sha1": self.registry()["roles"][3]["git_blob_sha1"],
            "reviewer_claim": "local reviewer",
            "provenance": "LOCAL_SELF_REPORTED_ONLY",
            "execution_command_record": "python -B -m unittest -v",
            "review_result_claim": "PASS",
            "evidence_log_relpath": "governance/review_logs/r2_cipher.log",
            "evidence_log_sha256": hashlib.sha256(b"example log\n").hexdigest(),
            "requested_decision": "STAGE_ONLY",
            "independent_operator_attestation": None,
            "independent_signer_signature": None,
        }
        self.log = self.root/self.record["evidence_log_relpath"]
        self.log.parent.mkdir(parents=True, exist_ok=True)
        self.log.write_bytes(b"example log\n")
        self.dossier_path = "governance/review_logs/r2_cipher.json"
        self.write_dossier()

    def tearDown(self):
        self.t.cleanup()

    def registry(self):
        return json.loads((self.root/lab.REGISTRY).read_text())

    def change(self, function):
        p=self.root/lab.REGISTRY
        d=json.loads(p.read_text())
        function(d)
        p.write_text(json.dumps(d,indent=2)+"\n")

    def write_dossier(self):
        p=self.root/self.dossier_path
        p.parent.mkdir(parents=True,exist_ok=True)
        p.write_bytes(lab.canonical(self.record))

    def blocked(self):
        with self.assertRaises((lab.Blocked,lab.previous.Blocked,
                                lab.previous.guardian.Blocked)):
            lab.audit(self.root)

    def stage_blocked(self):
        with self.assertRaises((lab.Blocked,lab.previous.Blocked,
                                lab.previous.guardian.Blocked)):
            lab.stage_submission(self.root,self.dossier_path)

    def test_01_nine_source_perspectives_are_not_nine_executed_agents(self):
        x=lab.audit(self.root)
        self.assertEqual(x["SOURCE_PERSPECTIVES_INSPECTED"],"9/9")
        self.assertEqual(x["INDEPENDENT_REVIEWER_EXECUTIONS"],0)
        self.assertEqual(x["INDEPENDENTLY_CERTIFIED_SPECIALIST_REPORTS"],0)

    def test_02_r1_exact_99_blob_history_rechecked(self):
        self.assertEqual(lab.audit(self.root)["R1_PINNED_SOURCES"],"99/99_PASS")

    def test_03_r6g_and_native_admissions_remain_locked(self):
        x=lab.audit(self.root)
        self.assertEqual(x["R6G"],"LOCKED_BY_SEQUENCE")
        self.assertEqual(x["NATIVE_ADMISSION_GATES"],"12_UNMET")
        self.assertEqual(x["PRODUCTION"],"NOT_AUTHORIZED")

    def test_04_every_role_evidence_is_actual_source_anchor(self):
        _,r,rows=lab.gate(self.root)
        self.assertEqual(len(rows),9)
        self.assertEqual(tuple(rows),lab.ROLES)
        for k,row in rows.items():
            src=lab.previous.git_checkout_bytes(
                (self.root/row["source_path"]).read_bytes())
            self.assertIn(row["literal_source_anchor"].encode(),src)

    def test_05_valid_self_reported_log_only_staged_not_certified(self):
        x=lab.stage_submission(self.root,self.dossier_path)
        self.assertEqual(x["BUREAU001_R2"],
                         "STAGED_UNVERIFIED_LOCAL_REVIEW_LOG_ONLY")
        self.assertFalse(x["REVIEWER_IDENTITY_AUTHENTICATED"])
        self.assertFalse(x["INDEPENDENT_EXECUTION_PROVEN"])
        self.assertEqual(x["SPECIALIST_CERTIFICATION"],"PENDING")

    def test_06_missing_role_evidence_blocks(self):
        self.change(lambda r:r["roles"].pop())
        self.blocked()

    def test_07_duplicate_or_reordered_role_blocks(self):
        self.change(lambda r:r["roles"].__setitem__(
            1,copy.deepcopy(r["roles"][0])))
        self.blocked()

    def test_08_forged_source_git_pin_blocks(self):
        self.change(lambda r:r["roles"][3].__setitem__(
            "git_blob_sha1","a"*40))
        self.blocked()

    def test_09_fake_source_anchor_not_in_git_bytes_blocks(self):
        self.change(lambda r:r["roles"][1].__setitem__(
            "literal_source_anchor","NO_REAL_SOURCE_ANCHOR_EXISTS"))
        self.blocked()

    def test_10_reused_identical_source_across_roles_blocks(self):
        def change(r):
            r["roles"][2]["source_path"]=r["roles"][1]["source_path"]
            r["roles"][2]["git_blob_sha1"]=r["roles"][1]["git_blob_sha1"]
            r["roles"][2]["literal_source_anchor"]=r["roles"][1]["literal_source_anchor"]
        self.change(change)
        self.blocked()

    def test_11_fake_independent_agent_execution_blocks(self):
        self.change(lambda r:r.__setitem__(
            "independent_specialist_agents_executed",9))
        self.blocked()

    def test_12_fake_specialist_certification_blocks(self):
        self.change(lambda r:r["roles"][0].__setitem__(
            "certification","CERTIFIED"))
        self.blocked()

    def test_13_forged_logged_independent_agent_blocks(self):
        self.change(lambda r:r["roles"][4].__setitem__(
            "independent_execution_log","forged.log"))
        self.blocked()

    def test_14_inserted_self_certifying_review_blocks(self):
        self.change(lambda r:r["submissions"].append(
            {"role":"CIPHER","verdict":"CERTIFIED"}))
        self.blocked()

    def test_15_register_cannot_unlock_r6g(self):
        self.change(lambda r:r.__setitem__("r6g","IN_PROGRESS"))
        self.blocked()

    def test_16_register_cannot_modify_native_gate_counts(self):
        self.change(lambda r:r.__setitem__(
            "native_admission_gates_unmet",0))
        self.blocked()

    def test_17_review_submission_rejects_forged_external_attestation(self):
        self.record["independent_operator_attestation"]="signed:fake"
        self.write_dossier()
        self.stage_blocked()

    def test_18_review_submission_rejects_privileged_decision(self):
        self.record["requested_decision"]="CERTIFY_AND_DEPLOY"
        self.write_dossier()
        self.stage_blocked()

    def test_19_review_submission_rejects_wrong_parent_or_role_sha(self):
        self.record["parent_head"]="f"*40
        self.write_dossier()
        self.stage_blocked()
        self.record["parent_head"]=lab.PARENT
        self.record["source_git_blob_sha1"]="0"*40
        self.write_dossier()
        self.stage_blocked()

    def test_20_review_submission_rejects_log_tampering(self):
        self.log.write_bytes(b"altered log\n")
        self.stage_blocked()

    def test_21_review_submission_rejects_path_traversal(self):
        self.record["evidence_log_relpath"]="../private.key"
        self.write_dossier()
        self.stage_blocked()

    @unittest.skipIf(os.name=="nt","Creating symlinks may require Windows administrator")
    def test_22_review_submission_rejects_symlinked_log(self):
        self.log.unlink()
        external=self.root/"governance"/"review_logs"/"fake.log"
        external.write_bytes(b"example log\n")
        self.log.symlink_to(external.name)
        self.stage_blocked()

    def test_23_duplicate_fields_and_noncanonical_dossier_rejected(self):
        p=self.root/self.dossier_path
        raw=p.read_bytes()
        p.write_bytes(raw.replace(b'"role":',b'"role":"FERRUM","role":'))
        self.stage_blocked()
        p.write_text(json.dumps(self.record,indent=2))
        self.stage_blocked()

    def test_24_no_writes_even_when_staging_and_attempting_certification(self):
        before=sorted((str(x.relative_to(self.root)),x.read_bytes())
                      for x in self.root.rglob("*") if x.is_file())
        stage=lab.stage_submission(self.root,self.dossier_path)
        self.assertEqual(stage["WRITES"],0)
        p=subprocess.run([
            sys.executable,"-B",str(ROOT/"scripts/bureau_001_r2_review_intake.py"),
            "--root",str(self.root),"--attempt-certify","BUREAU"
        ],capture_output=True,text=True,timeout=45,check=False)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["TRANSITION"],"BLOCK")
        after=sorted((str(x.relative_to(self.root)),x.read_bytes())
                     for x in self.root.rglob("*") if x.is_file())
        self.assertEqual(before,after)

if __name__=="__main__":
    unittest.main()
