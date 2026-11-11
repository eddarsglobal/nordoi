"""R5N: 24 real SQLite / actual OS process interrupted checkpoint-use tests.

16 portable tests exercise actual file-backed R5L/R5M and replay; 8 POSIX
native controls include real SIGKILL, real second-process file replacement.
No production admission or independent custodian is simulated.
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
import g03_r5m_file_integrity_quarantine as evidence
import g03_r5n_checkpoint_use_quarantine as gate

class PortableUseQuarantine(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5n-portable-")
        self.root=Path(self.tmp.name)
        self.a=self.root/"primary";self.b=self.root/"witness";self.c=self.root/"journal"
        for d in (self.a,self.b,self.c):
            d.mkdir(mode=0o700)
            if os.name!="nt":os.chmod(d,0o700)
        if os.name!="nt":os.chmod(self.root,0o700)
        with denial.DenialJournal(self.a) as j:
            self.assertEqual(j.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(evidence.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.c) as j:self.assertEqual(j.audit()["local_height"],0)
        self.token=gate.prepare(self.a,self.b,"c"*64)
    def tearDown(self):self.tmp.cleanup()
    def check(self,raw=None,**kwargs):
        result=gate.evaluate(self.token if raw is None else raw,self.a,self.b,self.c,**kwargs)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        return result
    def reject(self,raw):
        with self.assertRaises(gate.Refused):gate.parse(raw)
    def change(self,**kwargs):
        obj=json.loads(self.token)
        obj.update(kwargs)
        return gate.canonical(obj)
    def test_01_real_fresh_local_witness_and_deny_journal(self):
        self.assertEqual(evidence.audit(self.a,self.b)["local_height"],1)
        self.assertEqual(denial.execute(self.c,"audit")["local_height"],0)
    def test_02_token_is_canonical_and_bound_to_real_db_bytes(self):
        ticket=gate.parse(self.token)
        self.assertEqual(ticket["height"],1)
        self.assertEqual(ticket["db_sha256"],evidence.primary_state(self.a/denial.NAME)["db_sha256"])
    def test_03_real_matching_checkpoint_use_records_only_denial(self):
        r=self.check()
        self.assertEqual(r["reason"],"VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED")
        self.assertEqual(r["local_denial_height"],1)
    def test_04_real_duplicate_packet_nonce_refused(self):
        self.check()
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",self.check()["detail"])
        self.assertEqual(denial.execute(self.c,"audit")["local_height"],1)
    def test_05_new_nonce_records_second_local_denial_only(self):
        self.check()
        new=gate.prepare(self.a,self.b,"d"*64)
        r=self.check(new)
        self.assertEqual(r["local_denial_height"],2)
        self.assertFalse(r["native_authority_granted"])
    def test_06_missing_use_journal_not_auto_created(self):
        (self.c/denial.NAME).unlink()
        self.assertIn("DENIAL_JOURNAL_MISSING_NO_AUTO_CREATE",self.check()["detail"])
        self.assertFalse((self.c/denial.NAME).exists())
    def test_07_missing_witness_never_reinitialized(self):
        (self.b/evidence.WITNESS).unlink()
        self.assertEqual(self.check()["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
        self.assertFalse((self.b/evidence.WITNESS).exists())
    def test_08_missing_primary_not_auto_initialized(self):
        (self.a/denial.NAME).unlink()
        self.assertEqual(self.check()["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
        self.assertFalse((self.a/denial.NAME).exists())
    def test_09_invalid_type_and_oversized_packet_refused(self):
        self.reject("not bytes")
        self.reject(b"x"*(gate.MAX_BYTES+1))
    def test_10_duplicate_and_noncanonical_json_refused(self):
        self.reject(self.token.replace(b'"project":',b'"project":"forged","project":'))
        self.reject(json.dumps(json.loads(self.token),indent=2).encode("ascii"))
    def test_11_wrong_parent_protocol_project_refused(self):
        for k,v in (("parent","0"*40),("protocol","FAKE"),("project","other")):
            self.reject(self.change(**{k:v}))
    def test_12_grant_and_unrevoke_operation_refused(self):
        for op in ("GRANT","UNREVOKE","RECOVER"):
            self.reject(self.change(operation=op))
    def test_13_invalid_nonce_and_wrong_digest_refused(self):
        self.reject(self.change(nonce="bad"))
        self.reject(self.change(db_sha256="0"*64))
        self.reject(self.change(witness_sha256="z"*64))
    def test_14_boolean_or_genesis_height_refused(self):
        self.reject(self.change(height=True))
        self.reject(self.change(height=0))
    def test_15_tampered_primary_chain_without_protected_witness_refused(self):
        (self.a/denial.NAME).write_bytes(b"SQLite format 3\x00")
        self.assertEqual(self.check()["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
    def test_16_real_separate_cli_audit_never_admits(self):
        cli=[sys.executable,"-B",str(Path(gate.__file__)),"--assess",
             "--intent-file",str(self.root/"intent.json"),
             "--primary-dir",str(self.a),"--witness-dir",str(self.b),
             "--journal-dir",str(self.c)]
        (self.root/"intent.json").write_bytes(self.token)
        p=subprocess.run(cli,text=True,capture_output=True,timeout=30)
        self.assertEqual(p.returncode,2,p.stderr)
        obj=json.loads(p.stdout)
        self.assertEqual(obj["admission"],"NOT_AUTHORIZED")
        self.assertEqual(obj["reason"],"VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "real POSIX Unix/OpenSSH process termination and replacement required")
class NativeInterruptedUse(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5n-native-")
        self.root=Path(self.tmp.name);os.chmod(self.root,0o700)
        self.a=self.root/"primary";self.b=self.root/"witness";self.c=self.root/"journal"
        for d in (self.a,self.b,self.c):
            d.mkdir(mode=0o700);os.chmod(d,0o700)
        with denial.DenialJournal(self.a) as j:
            self.assertEqual(j.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(evidence.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.c) as j:self.assertEqual(j.audit()["local_height"],0)
        self.old_a=self.root/"old.sqlite3";self.old_b=self.root/"old.json"
        shutil.copy2(self.a/denial.NAME,self.old_a)
        shutil.copy2(self.b/evidence.WITNESS,self.old_b)
        self.old_ticket=gate.prepare(self.a,self.b,"c"*64)
        with denial.DenialJournal(self.a) as j:
            self.assertEqual(j.record("d"*64,"e"*64,"candidate_b")["local_height"],2)
        self.assertEqual(evidence.advance(self.a,self.b)["local_height"],2)
        self.token=gate.prepare(self.a,self.b,"f"*64)
    def tearDown(self):self.tmp.cleanup()
    def check(self,token=None,**kw):
        result=gate.evaluate(self.token if token is None else token,self.a,self.b,self.c,**kw)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        return result
    def test_17_real_persistent_height_two_checkpoint_still_denied(self):
        self.assertEqual(self.check()["reason"],"VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED")
        self.assertEqual(denial._run("audit",self.c)["local_height"],1)
    def test_18_real_second_process_truncates_between_first_and_second_check(self):
        r=self.check(after_first_observation=lambda:evidence._real_child_restore([
            ("TRUNC","32",self.a/denial.NAME)]))
        self.assertEqual(r["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
    def test_19_real_second_process_deletes_witness_after_first_check(self):
        r=self.check(after_first_observation=lambda:evidence._real_child_restore([
            ("DELETE","0",self.b/evidence.WITNESS)]))
        self.assertEqual(r["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
    def test_20_real_second_process_swaps_old_witness_before_commit(self):
        r=self.check(before_local_deny_commit=lambda:evidence._real_child_restore([
            ("COPY",self.old_b,self.b/evidence.WITNESS)]))
        self.assertEqual(r["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
        self.assertEqual(denial._run("audit",self.c)["local_height"],0)
    def test_21_real_second_process_restores_old_sqlite_before_denial(self):
        r=self.check(before_local_deny_commit=lambda:evidence._real_child_restore([
            ("COPY",self.old_a,self.a/denial.NAME)]))
        self.assertEqual(r["reason"],"CHECKPOINT_USE_INTERRUPTED_OR_QUARANTINED")
    def test_22_real_sigkill_uncommitted_use_denial_then_restart(self):
        self.assertTrue(denial.actual_kill(self.c,"2"*64,"3"*64,"candidate_a"))
        self.assertEqual(denial._run("audit",self.c)["local_height"],0)
        self.assertEqual(self.check()["reason"],"VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED")
        self.assertEqual(denial._run("audit",self.c)["local_height"],1)
    def test_23_both_old_local_artifacts_restored_still_deny_no_global_antirollback(self):
        evidence._real_child_restore([("COPY",self.old_a,self.a/denial.NAME),
                                      ("COPY",self.old_b,self.b/evidence.WITNESS)])
        r=self.check(token=self.old_ticket)
        self.assertEqual(r["reason"],"VERIFIED_LOCAL_CHECKPOINT_USE_DENIED_NOT_ADMITTED")
        self.assertEqual(denial._run("audit",self.c)["global_monotonic_antirollback"],
                         "NOT_IMPLEMENTED")
    def test_24_real_native_cli_full_interrupted_checkpoint_use_exits_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(gate.__file__)),"--exercise"],
                         text=True,capture_output=True,timeout=150)
        self.assertEqual(p.returncode,2,p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["reason"],"REAL_CHECKPOINT_USE_INTERRUPTION_ALWAYS_DENIED")
        self.assertEqual(r["genuine_os_sigkill_uncommitted_record"],"ROLLED_BACK")
        self.assertEqual(r["second_os_process_truncation"],"QUARANTINED")
        self.assertEqual(r["witness_swapped_before_denial_commit"],"QUARANTINED")
        self.assertFalse(r["native_authority_granted"])

if __name__=="__main__":unittest.main()
