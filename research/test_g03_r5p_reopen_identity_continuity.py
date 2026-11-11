"""24 R5P actual-file controls: 16 portable, eight POSIX native.

No simulated external authority. Native experiments use separate real OS
processes and actual on-disk SQLite / checkpoint identity. Inode reuse is a
bounded observational probe, not a promised event or a security guarantee.
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
import g03_r5m_file_integrity_quarantine as witness
import g03_r5p_reopen_identity_continuity as lab

class PortableCloseReopen(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5p-portable-")
        self.root=Path(self.temp.name)
        self.a,self.b,self.j=(self.root/"primary",self.root/"witness",self.root/"journal")
        for directory in (self.a,self.b,self.j):
            directory.mkdir(mode=0o700)
            if os.name!="nt":os.chmod(directory,0o700)
        if os.name!="nt":os.chmod(self.root,0o700)
        with denial.DenialJournal(self.a) as d:
            self.assertEqual(d.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(witness.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.j) as d:
            self.assertEqual(d.audit()["local_height"],0)
        self.ticket=lab.prepare(self.a,self.b,"c"*64)
    def tearDown(self):self.temp.cleanup()
    def check(self,ticket=None,**kwargs):
        result=lab.evaluate(self.ticket if ticket is None else ticket,
                            self.a,self.b,self.j,**kwargs)
        self.assertEqual(result["status"],"BLOCK")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertFalse(result["native_authority_granted"])
        return result
    def changed(self,**values):
        d=json.loads(self.ticket)
        d.update(values)
        return lab.canonical(d)
    def reject(self,ticket):
        with self.assertRaises(lab.Refused):lab.parse(ticket)
    def test_01_actual_closed_reopened_descriptor_metadata_matches(self):
        observed=lab.epoch(self.a,self.b)
        token=lab.parse(self.ticket)
        self.assertEqual(token["db_identity"],observed["db_identity"])
        self.assertEqual(token["witness_identity"],observed["witness_identity"])
    def test_02_exact_canonical_ticket_and_source_pin(self):
        token=lab.parse(self.ticket)
        self.assertEqual(token["parent"],lab.PARENT)
        self.assertEqual(lab.canonical(token),self.ticket)
    def test_03_verified_reopened_files_only_record_local_denial(self):
        r=self.check()
        self.assertEqual(r["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertEqual(r["local_denial_height"],1)
        self.assertFalse(r["independent_identity_proof"])
    def test_04_persisted_local_nonce_replay_is_blocked(self):
        self.check()
        self.assertIn("ALREADY_RECORDED_DENIAL_NONCE",self.check()["detail"])
        self.assertEqual(denial.execute(self.j,"audit")["local_height"],1)
    def test_05_fresh_nonce_records_only_a_second_denial(self):
        self.check()
        r=self.check(lab.prepare(self.a,self.b,"d"*64))
        self.assertEqual(r["local_denial_height"],2)
    def test_06_deleted_primary_does_not_self_heal(self):
        path=self.a/denial.NAME
        path.unlink()
        self.assertEqual(self.check()["reason"],"REOPEN_IDENTITY_QUARANTINED")
        self.assertFalse(path.exists())
    def test_07_deleted_witness_does_not_self_heal(self):
        path=self.b/witness.WITNESS
        path.unlink()
        self.assertEqual(self.check()["reason"],"REOPEN_IDENTITY_QUARANTINED")
        self.assertFalse(path.exists())
    def test_08_missing_denial_journal_is_not_reinitialized(self):
        path=self.j/denial.NAME;path.unlink()
        self.assertIn("DENIAL_JOURNAL_MISSING_NO_REINITIALIZATION",
                      self.check()["detail"])
        self.assertFalse(path.exists())
    def test_09_grant_revoke_or_recovery_operation_cannot_parse(self):
        for operation in ("GRANT","UNREVOKE","RECOVER","APPROVE","ALLOW"):
            self.reject(self.changed(operation=operation))
    def test_10_forged_source_protocol_project_refused(self):
        for field,word in (("parent","0"*40),("protocol","other"),("project","forged")):
            self.reject(self.changed(**{field:word}))
    def test_11_duplicate_json_and_noncanonical_json_refused(self):
        self.reject(self.ticket.replace(b'"project":',b'"project":"fake","project":'))
        self.reject(json.dumps(json.loads(self.ticket),indent=2).encode())
    def test_12_boolean_or_negative_inode_fields_refused(self):
        for field in ("db_identity","witness_identity"):
            old=lab.parse(self.ticket)[field]
            for value in (True,-1,"1"):
                forged=old.copy();forged[1]=value
                self.reject(self.changed(**{field:forged}))
    def test_13_zero_or_boolean_height_refused(self):
        self.reject(self.changed(height=0))
        self.reject(self.changed(height=True))
    def test_14_nonce_and_digest_substitution_refused(self):
        for field in ("nonce","head","db_sha256","witness_sha256"):
            self.reject(self.changed(**{field:"0"*64}))
    def test_15_malformed_and_oversized_reopen_ticket_refused(self):
        for ticket in ("not bytes",b"{}",b"x"*(lab.MAX_TICKET_BYTES+1)):
            self.reject(ticket)
    def test_16_actual_separate_os_process_cli_always_exits_deny_two(self):
        ticket=self.root/"intent.json"
        ticket.write_bytes(self.ticket)
        cmd=[sys.executable,"-B",str(Path(lab.__file__)),"--assess",
             "--ticket-file",str(ticket),"--primary-dir",str(self.a),
             "--witness-dir",str(self.b),"--journal-dir",str(self.j)]
        p=subprocess.run(cmd,text=True,capture_output=True,timeout=30)
        self.assertEqual(p.returncode,2,p.stderr)
        receipt=json.loads(p.stdout)
        self.assertEqual(receipt["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertEqual(receipt["admission"],"NOT_AUTHORIZED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "genuine POSIX real process atomic inode replacement qualification")
class NativeCloseReopen(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5p-native-")
        self.root=Path(self.temp.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=(self.root/"primary",self.root/"witness",self.root/"journal")
        for directory in (self.a,self.b,self.j):
            directory.mkdir(mode=0o700);os.chmod(directory,0o700)
        with denial.DenialJournal(self.a) as d:
            self.assertEqual(d.record("a"*64,"b"*64,"candidate_a")["local_height"],1)
        self.assertEqual(witness.capture(self.a,self.b)["local_height"],1)
        with denial.DenialJournal(self.j) as d:self.assertEqual(d.audit()["local_height"],0)
        self.db=self.a/denial.NAME;self.wp=self.b/witness.WITNESS
        self.back_db=self.root/"copy.sqlite3";self.back_wp=self.root/"copy.json"
        shutil.copy2(self.db,self.back_db)
        shutil.copy2(self.wp,self.back_wp)
        self.ticket=lab.prepare(self.a,self.b,"c"*64)
    def tearDown(self):self.temp.cleanup()
    def check(self,ticket=None,**kwargs):
        r=lab.evaluate(self.ticket if ticket is None else ticket,
                       self.a,self.b,self.j,**kwargs)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        return r
    def test_17_real_close_reopen_without_swap_is_denial_only(self):
        r=self.check()
        self.assertEqual(r["reason"],"REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
        self.assertTrue(r["close_reopen_checked"])
    def test_18_real_second_process_swaps_primary_while_descriptors_closed(self):
        r=self.check(after_close=lambda:lab.replace_second_process(self.back_db,self.db))
        self.assertEqual(r["reason"],"REOPEN_IDENTITY_QUARANTINED")
        self.assertIn("PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN",r["detail"])
    def test_19_real_second_process_swaps_witness_while_descriptors_closed(self):
        r=self.check(after_close=lambda:lab.replace_second_process(self.back_wp,self.wp))
        self.assertIn("PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN",r["detail"])
    def test_20_real_second_process_swaps_primary_before_denial(self):
        r=self.check(before_denial=lambda:lab.replace_second_process(self.back_db,self.db))
        self.assertIn("PATH_IDENTITY_CHANGED_BEFORE_LOCAL_DENIAL",r["detail"])
        self.assertEqual(denial._run("audit",self.j)["local_height"],0)
    def test_21_successive_two_byte_identical_swaps_while_closed(self):
        def two_swaps():
            lab.replace_second_process(self.back_db,self.db)
            lab.replace_second_process(self.back_db,self.db)
        r=self.check(after_close=two_swaps)
        self.assertIn("PATH_IDENTITY_CHANGED_ACROSS_CLOSE_REOPEN",r["detail"])
    def test_22_stale_epoch_fails_after_reopen_fresh_ticket_only_denies(self):
        old=self.ticket
        lab.replace_second_process(self.back_db,self.db)
        self.assertIn("CLOSED_DESCRIPTOR_TICKET_STALE_DB_IDENTITY",
                      self.check(old)["detail"])
        fresh=lab.prepare(self.a,self.b,"d"*64)
        self.assertEqual(self.check(fresh)["reason"],
                         "REOPENED_IDENTITY_MATCH_ONLY_LOCAL_DENIAL")
    def test_23_bounded_real_inode_reuse_churn_not_equated_with_authority(self):
        result=lab.actual_reuse_probe(self.back_db,self.db,12)
        self.assertIn(result,("OBSERVED_NO_AUTHORITY",
                              "NOT_OBSERVED_BOUNDED_NO_AUTHORITY"))
        self.assertEqual(self.check(lab.prepare(self.a,self.b,"e"*64))["admission"],
                         "NOT_AUTHORIZED")
    def test_24_actual_native_cli_end_to_end_exit_two(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                         text=True,capture_output=True,timeout=180)
        self.assertEqual(p.returncode,2,p.stderr)
        x=json.loads(p.stdout)
        self.assertEqual(x["reason"],"REAL_CLOSE_REOPEN_IDENTITY_CONTINUITY_DENIED")
        self.assertEqual(x["closed_descriptor_primary_swap"],"QUARANTINED")
        self.assertEqual(x["witness_swap_before_denial"],"QUARANTINED")
        self.assertEqual(x["two_successive_atomic_swaps"],"QUARANTINED")
        self.assertIn(x["bounded_real_inode_reuse_probe"],(
            "OBSERVED_NO_AUTHORITY","NOT_OBSERVED_BOUNDED_NO_AUTHORITY"))
        self.assertFalse(x["native_authority_granted"])

if __name__=="__main__":unittest.main()
