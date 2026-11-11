"""G0.3 R5M: 24 actual file integrity negative controls, no external authority.

16 portable tests use real SQLite files, 8 POSIX process tests perform real
external process truncation/corruption/delete and same-admin snapshot restore.
"""
from contextlib import closing
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import g03_r5h_signed_revocation_lab as rev
import g03_r5l_crash_restart_denial_journal as prior
import g03_r5m_file_integrity_quarantine as gate

class PortableIntegrity(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5m-portable-")
        self.root=Path(self.tmp.name)
        self.primary=self.root/"primary"
        self.witness=self.root/"witness"
        self.primary.mkdir(mode=0o700);self.witness.mkdir(mode=0o700)
        if os.name!="nt":
            for p in (self.root,self.primary,self.witness):os.chmod(p,0o700)
        with prior.DenialJournal(self.primary) as j:
            self.assertEqual(j.audit()["local_height"],0)
        self.empty=self.root/"initial.sqlite3"
        shutil.copy2(self.primary/prior.NAME,self.empty)
        with prior.DenialJournal(self.primary) as j:
            self.assertEqual(j.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(gate.capture(self.primary,self.witness)["local_height"],1)
    def tearDown(self):self.tmp.cleanup()
    def block(self):
        r=gate.audit(self.primary,self.witness)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        return r
    def test_01_missing_preserved_witness_never_auto_created(self):
        (self.witness/gate.WITNESS).unlink()
        r=self.block()
        self.assertIn("PRESERVED_LOCAL_WITNESS_MISSING",r["detail"])
        self.assertFalse((self.witness/gate.WITNESS).exists())
    def test_02_unrevoked_genesis_cannot_be_checkpointed(self):
        shutil.copy2(self.empty,self.primary/prior.NAME)
        fresh=self.root/"new_witness"
        fresh.mkdir(mode=0o700)
        if os.name!="nt":os.chmod(fresh,0o700)
        with self.assertRaises(gate.Refused) as c:
            gate.capture(self.primary,fresh)
        self.assertIn("CANNOT_ANCHOR_PRE_DENIAL_GENESIS",str(c.exception))
    def test_03_real_preserved_file_matches_but_never_grants(self):
        r=self.block()
        self.assertEqual(r["reason"],"PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT")
        self.assertEqual(r["local_height"],1)
    def test_04_existing_preserved_witness_cannot_reinitialize(self):
        with self.assertRaises(gate.Refused) as c:
            gate.capture(self.primary,self.witness)
        self.assertIn("WITNESS_ALREADY_EXISTS",str(c.exception))
    def test_05_real_checkpoint_file_parses_and_binds_sha(self):
        value,raw=gate.read_witness(self.witness/gate.WITNESS)
        self.assertEqual(value["db_sha256"],gate.primary_state(self.primary/prior.NAME)["db_sha256"])
        self.assertEqual(raw,gate.canonical(value))
    def test_06_duplicate_witness_json_key_denied(self):
        path=self.witness/gate.WITNESS
        raw=path.read_bytes()
        path.write_bytes(raw.replace(b'"project":',b'"project":"fake","project":'))
        self.assertIn("DUPLICATE_WITNESS_FIELD",self.block()["detail"])
    def test_07_noncanonical_json_whitespace_denied(self):
        path=self.witness/gate.WITNESS
        path.write_text(json.dumps(json.loads(path.read_bytes()),indent=2),encoding="utf-8")
        self.assertIn("WITNESS_FIELDS_OR_CANONICAL_FAILURE",self.block()["detail"])
    def test_08_unknown_witness_key_denied(self):
        path=self.witness/gate.WITNESS
        data=json.loads(path.read_bytes());data["unknown"]="ALLOW"
        path.write_bytes(gate.canonical(data))
        self.assertIn("WITNESS_FIELDS_OR_CANONICAL_FAILURE",self.block()["detail"])
    def test_09_forged_witness_digest_denied(self):
        path=self.witness/gate.WITNESS
        data=json.loads(path.read_bytes());data["db_sha256"]="z"*64
        path.write_bytes(gate.canonical(data))
        self.assertIn("WITNESS_DIGEST_INVALID",self.block()["detail"])
    def test_10_wrong_pinned_parent_source_denied(self):
        path=self.witness/gate.WITNESS
        data=json.loads(path.read_bytes());data["r5l_parent"]="0"*40
        path.write_bytes(gate.canonical(data))
        self.assertIn("WITNESS_IDENTITY_OR_SOURCE_MISMATCH",self.block()["detail"])
    def test_11_real_sqlite_chain_row_tamper_detected(self):
        with closing(sqlite3.connect(self.primary/prior.NAME)) as c, c:
            c.execute("UPDATE denials SET nonce=? WHERE seq=1",("c"*64,))
        self.assertIn("PRIMARY_DENIAL_CHAIN_INVALID",self.block()["detail"])
    def test_12_real_sqlite_file_truncated_to_header_fragment_denied(self):
        (self.primary/prior.NAME).write_bytes(b"SQLite format 3\x00")
        self.assertIn("PRIMARY_SQLITE_TRUNCATED_OR_BAD_HEADER",self.block()["detail"])
    def test_13_primary_sqlite_deleted_must_not_be_regenerated(self):
        path=self.primary/prior.NAME
        path.unlink()
        self.assertIn("PRIMARY_SQLITE_MISSING_NO_REINITIALIZATION",self.block()["detail"])
        self.assertFalse(path.exists())
    def test_14_witness_symlink_never_read_as_protected_evidence(self):
        target=self.witness/gate.WITNESS
        alt=self.root/"other-witness"
        shutil.copy2(target,alt)
        target.unlink()
        try:target.symlink_to(alt)
        except (NotImplementedError,OSError):
            self.assertEqual(self.block()["status"],"BLOCK")
        else:self.assertIn("SYMLINK_OR_NONREGULAR_EVIDENCE",self.block()["detail"])
    def test_15_nested_primary_and_witness_roots_denied(self):
        r=gate.audit(self.primary,self.primary)
        self.assertIn("PRIMARY_WITNESS_ROOTS_OVERLAP",r["detail"])
    def test_16_real_portable_subprocess_audit_returns_deny_exit_two(self):
        command=[sys.executable,"-B",str(Path(gate.__file__)),"--audit",
                 "--primary-dir",str(self.primary),"--witness-dir",str(self.witness)]
        p=subprocess.run(command,capture_output=True,text=True,timeout=30)
        self.assertEqual(p.returncode,2,p.stderr)
        self.assertEqual(json.loads(p.stdout)["admission"],"NOT_AUTHORIZED")

@unittest.skipUnless(os.name!="nt" and Path(rev.SSH).is_file(),
                      "real Unix/OpenSSH and subprocess file integrity experiment required")
class NativeFileInterruption(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5m-native-")
        self.root=Path(self.tmp.name)
        self.a=self.root/"primary";self.b=self.root/"witness"
        self.a.mkdir(mode=0o700);self.b.mkdir(mode=0o700)
        for p in (self.root,self.a,self.b):os.chmod(p,0o700)
        with prior.DenialJournal(self.a) as j:
            self.assertEqual(j.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(gate.capture(self.a,self.b)["local_height"],1)
        self.old_db=self.root/"old-primary.sqlite3"
        self.old_witness=self.root/"old-local-witness.json"
        shutil.copy2(self.a/prior.NAME,self.old_db)
        shutil.copy2(self.b/gate.WITNESS,self.old_witness)
        with prior.DenialJournal(self.a) as j:
            self.assertEqual(j.record("c"*64,"d"*64,"candidate_b")["local_height"],2)
        self.assertEqual(gate.advance(self.a,self.b)["local_height"],2)
        self.assertEqual(gate.audit(self.a,self.b)["local_height"],2)
    def tearDown(self):self.tmp.cleanup()
    def test_17_native_real_sqlite_checkpoint_at_height_two(self):
        result=gate.audit(self.a,self.b)
        self.assertEqual(result["reason"],"PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
    def test_18_real_second_process_truncate_between_observations(self):
        result=gate.audit(self.a,self.b,after_first_observation=lambda:gate._real_child_restore([
            ("TRUNC","32",self.a/prior.NAME)]))
        self.assertEqual(result["reason"],"INTEGRITY_OR_RECOVERY_EVIDENCE_QUARANTINED")
    def test_19_real_child_corrupts_sqlite_header(self):
        gate._real_child_restore([("TRUNC","8",self.a/prior.NAME)])
        self.assertIn("PRIMARY_SQLITE_TRUNCATED_OR_BAD_HEADER",gate.audit(self.a,self.b)["detail"])
    def test_20_real_child_unlinks_sqlite_without_auto_init(self):
        gate._real_child_restore([("DELETE","0",self.a/prior.NAME)])
        self.assertIn("PRIMARY_SQLITE_MISSING_NO_REINITIALIZATION",gate.audit(self.a,self.b)["detail"])
        self.assertFalse((self.a/prior.NAME).exists())
    def test_21_valid_old_sqlite_snapshot_rejected_with_preserved_newer_witness(self):
        gate._real_child_restore([("COPY",self.old_db,self.a/prior.NAME)])
        r=gate.audit(self.a,self.b)
        self.assertIn("PRIMARY_REPLACED_WITH_PRESERVED_WITNESS",r["detail"])
    def test_22_real_child_replaces_witness_between_observations(self):
        r=gate.audit(self.a,self.b,after_first_observation=lambda:gate._real_child_restore([
            ("COPY",self.old_witness,self.b/gate.WITNESS)]))
        self.assertIn("EVIDENCE_CHANGED_DURING_TWO_READS",r["detail"])
    def test_23_same_admin_restores_two_old_matching_files_undetectable_locally(self):
        gate._real_child_restore([("COPY",self.old_db,self.a/prior.NAME),
                                  ("COPY",self.old_witness,self.b/gate.WITNESS)])
        r=gate.audit(self.a,self.b)
        self.assertEqual(r["reason"],"PRESERVED_LOCAL_WITNESS_MATCH_NOT_INDEPENDENT")
        self.assertEqual(r["local_height"],1)
        self.assertEqual(r["global_monotonic_antirollback"],"NOT_IMPLEMENTED")
    def test_24_real_cli_file_truncation_and_joint_rollback_exits_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--exercise"],
                         text=True,capture_output=True,timeout=60)
        self.assertEqual(p.returncode,2,p.stderr)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],"REAL_CORRUPTION_RECOVERY_REFUSED_WITH_LOCAL_WITNESS_LIMITS")
        self.assertTrue(result["preserved_witness_detected_primary_rollback"])
        self.assertEqual(result["two_restored_old_same_owner_artifacts"],"LOCALLY_MATCH_UNDETECTABLE")
        self.assertFalse(result["native_authority_granted"])

if __name__=="__main__":unittest.main()
