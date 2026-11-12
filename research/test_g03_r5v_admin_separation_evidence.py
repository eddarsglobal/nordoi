"""G0.3 R5V: 24 adversarial OS identity and same-admin key custody controls.

16 portable strict observation/policy cases; eight genuine POSIX OpenSSH,
different real OS processes, local signed revocation and SQLite rollback.
Synthesized labels/UIDs are TEST INPUTS, not real independent guardians.
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
import g03_r5o_fd_identity_quarantine as fdgate
import g03_r5q_coherent_local_rollback as history
import g03_r5s_challenge_bound_handoff as handoff
import g03_r5t_signed_receipt_provenance as signed
import g03_r5u_key_rotation_custody as rotate
import g03_r5v_admin_separation_evidence as lab

class PortableAdminSeparation(unittest.TestCase):
    def setUp(self):
        self.root=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5v-portable-")
        self.home=Path(self.root.name)
        if os.name!="nt":os.chmod(self.home,0o700)
        self.a=lab.observe("owner_proxy","lab-owner-space","a"*64,0,"b"*64,"c"*64)
        self.b=dict(self.a)
        self.b.update(role="custodian_proxy",claimed_domain="lab-elsewhere-space",
                      key_sha256="d"*64,pid=self.a["pid"]+1)
    def tearDown(self):self.root.cleanup()
    def claim(self,**kwargs):
        x=dict(self.a);x.update(kwargs)
        return lab.canonical(x)
    def rejected(self,payload):
        with self.assertRaises(lab.Refused):lab.parse(payload)
    def check(self,owner=None,candidate=None):
        r=lab.assess(owner or self.a,candidate or self.b)
        self.assertEqual(r["status"],"BLOCK")
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(r["independent_admin_custody"],"NOT_PROVEN")
        return r
    def test_01_true_os_pid_and_identity_observation_canonical(self):
        self.assertEqual(self.a["pid"],os.getpid())
        self.assertEqual(self.a["euid"],os.geteuid() if hasattr(os,"geteuid") else None)
        self.assertEqual(self.a["uid"],os.getuid() if hasattr(os,"getuid") else None)
        self.assertEqual(lab.parse(lab.canonical(self.a)),self.a)
    def test_02_exact_parent_project_and_no_grant_fields(self):
        self.assertEqual(self.a["parent"],lab.PARENT)
        self.assertEqual(self.a["project"],"eddarsglobal/nordoi")
        self.assertNotIn("native_grant",self.a)
        self.assertEqual(set(self.a),set(lab.FIELDS))
    def test_03_different_claimed_domains_under_same_real_uid_not_independent(self):
        r=self.check()
        if os.name!="nt":
            self.assertEqual(r["reason"],"ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED")
            self.assertTrue(r["claimed_different_domains_not_proof"])
        else:self.assertEqual(r["reason"],"OS_USER_ID_SEPARATION_NOT_OBSERVABLE_ON_PLATFORM")
    def test_04_distinct_pid_and_key_pin_only_do_not_admit(self):
        self.assertNotEqual(self.a["pid"],self.b["pid"])
        self.assertNotEqual(self.a["key_sha256"],self.b["key_sha256"])
        self.check()
    def test_05_fake_different_uid_even_if_claimed_is_not_custody(self):
        if os.name=="nt":
            self.assertEqual(self.check()["reason"],"OS_USER_ID_SEPARATION_NOT_OBSERVABLE_ON_PLATFORM")
        else:
            forged=dict(self.b)
            forged["uid"]=self.a["uid"]+1
            forged["euid"]=self.a["euid"]+1
            self.assertEqual(self.check(candidate=forged)["reason"],
                             "DIFFERENT_OS_UID_STILL_NO_INDEPENDENT_CUSTODY_PROOF")
    def test_06_same_process_id_claim_refused(self):
        b=dict(self.b);b["pid"]=self.a["pid"]
        self.assertIn("SAME_OS_PROCESS",self.check(candidate=b)["detail"])
    def test_07_same_public_key_pin_refused(self):
        b=dict(self.b);b["key_sha256"]=self.a["key_sha256"]
        self.assertIn("SAME_KEY",self.check(candidate=b)["detail"])
    def test_08_wrong_challenge_refused(self):
        b=dict(self.b);b["nonce"]="f"*64
        self.assertIn("OBSERVED_CHALLENGE",self.check(candidate=b)["detail"])
    def test_09_wrong_history_head_or_key_epoch_refused(self):
        for change in ({"history_head":"f"*64},{"key_epoch":1}):
            b=dict(self.b);b.update(change)
            self.assertIn("OBSERVED_CHALLENGE_OR_HISTORY_MISMATCH",
                          self.check(candidate=b)["detail"])
    def test_10_malformed_role_or_domain_refused(self):
        for attr,value in (("role","guardian_owner"),
                           ("claimed_domain","../../admin"),
                           ("claimed_domain","external genuine")):
            self.rejected(self.claim(**{attr:value}))
    def test_11_malformed_uid_euid_gid_egid_claim_refused(self):
        for attr in ("uid","euid","gid","egid"):
            for value in (-1,True,"100"):
                self.rejected(self.claim(**{attr:value}))
    def test_12_boolean_bad_pid_or_epoch_refused(self):
        for attr,value in (("pid",True),("pid",0),("ppid",-1),
                           ("key_epoch",True),("key_epoch",-1)):
            self.rejected(self.claim(**{attr:value}))
    def test_13_invalid_key_hash_history_or_nonce_refused(self):
        for attr in ("key_sha256","history_head","nonce"):
            self.rejected(self.claim(**{attr:"bad"}))
    def test_14_duplicate_noncanonical_extra_or_missing_field_refused(self):
        self.rejected(self.claim(extra_grant=True))
        d=dict(self.a);del d["uid"];self.rejected(lab.canonical(d))
        raw=lab.canonical(self.a)
        self.rejected(raw.replace(b'"project":',b'"project":"faked","project":'))
        self.rejected(json.dumps(self.a,indent=2).encode())
    def test_15_oversized_invalid_utf8_and_unpinned_protocol_refused(self):
        for blob in (b"\xff",b"x"*(lab.MAX_REPORT+1),
                     self.claim(protocol="forged"),
                     self.claim(parent="0"*40)):
            self.rejected(blob)
    def test_16_real_fresh_subprocess_reports_same_uid_or_unavailable(self):
        b=lab.child_observe("custodian_proxy","lab-candidate-space",
                            "d"*64,0,"b"*64,"c"*64)
        self.assertNotEqual(self.a["pid"],b["pid"])
        self.assertEqual(self.a["uid"],b["uid"])
        result=self.check(candidate=b)
        if os.name!="nt":
            self.assertEqual(result["reason"],"ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED")

@unittest.skipUnless(os.name!="nt" and Path("/usr/bin/ssh-keygen").is_file(),
                      "genuine POSIX observed UID/eUID, two actual processes and two real SSHSIG keys")
class NativeAdminBoundary(unittest.TestCase):
    def setUp(self):
        self.t=tempfile.TemporaryDirectory(prefix="nordoi-r5l-r5q-r5s-r5t-r5u-r5v-native-")
        self.root=Path(self.t.name);os.chmod(self.root,0o700)
        self.a,self.b,self.j=history.fixture(self.root)
        self.zip=self.root/"archive.zip"
        handoff.prepare(self.a,self.b,self.j,"a"*64,1000,1200,self.zip)
        self.receipt=self.root/"receipt.json"
        self.receipt.write_bytes(signed.prepare_receipt(self.zip,"a"*64,1100))
        os.chmod(self.receipt,0o600)
        self.okey,self.oallow=signed.create_same_admin_key(self.root,"owner_signer")
        self.ckey,self.callow=signed.create_same_admin_key(self.root,"candidate_signer")
        self.opub=self.root/"owner_signer.pub"
        self.cpub=self.root/"candidate_signer.pub"
        self.opin=rotate.public_sha(self.opub)
        self.cpin=rotate.public_sha(self.cpub)
        self.genesis=rotate.bootstrap(self.root,self.opin)
        self.owner=lab.child_observe("owner_proxy","lab-owner-space",
                                     self.opin,0,self.genesis["head"],"a"*64)
        self.candidate=lab.child_observe("custodian_proxy","lab-candidate-space",
                                         self.cpin,0,self.genesis["head"],"a"*64)
        self.ofile=self.root/"owner.json"
        self.cfile=self.root/"candidate.json"
        self.ofile.write_bytes(lab.canonical(self.owner))
        self.cfile.write_bytes(lab.canonical(self.candidate))
        os.chmod(self.ofile,0o600);os.chmod(self.cfile,0o600)
        self.osig=lab.signature(self.ofile,self.okey)
        self.csig=lab.signature(self.cfile,self.ckey)
    def tearDown(self):self.t.cleanup()
    def compare(self):
        r=lab.assess(self.owner,self.candidate)
        self.assertEqual(r["admission"],"NOT_AUTHORIZED")
        self.assertFalse(r["native_authority_granted"])
        self.assertEqual(r["reason"],"ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED")
        return r
    def rotate_revocation(self):
        p=self.root/"rotation.json"
        body=rotate.intent(rotate.ROTATE,1,self.opin,self.cpin,
                           rotate.sha(self.receipt.read_bytes()),"1"*64)
        p.write_bytes(body)
        old=rotate.sign_intent(p,self.okey)
        other=self.root/"next_rot.json";other.write_bytes(body)
        nxt=rotate.sign_intent(other,self.ckey)
        r=rotate.append(self.root,body,old,self.oallow,nxt,self.callow)
        self.assertEqual(r["reason"],"REAL_LOCAL_DUAL_SSHSIG_ROTATION_DENIED",r)
        return body,old,nxt
    def test_17_actual_distinct_os_processes_same_uid_and_euid(self):
        self.assertNotEqual(self.owner["pid"],self.candidate["pid"])
        self.assertEqual(self.owner["uid"],self.candidate["uid"])
        self.assertEqual(self.owner["euid"],self.candidate["euid"])
        self.assertTrue(self.compare()["actual_distinct_processes"])
    def test_18_valid_genuine_two_distinct_ed25519_identity_signatures(self):
        for p,s,a,pub,pin in ((self.ofile,self.osig,self.oallow,self.opub,self.opin),
                              (self.cfile,self.csig,self.callow,self.cpub,self.cpin)):
            r=lab.verify(p,s,a,pub,pin)
            self.assertEqual(r["reason"],"REAL_SIGNED_LOCAL_OS_OBSERVATION_NOT_EXTERNAL")
            self.assertTrue(r["signature_valid_for_local_key"])
            self.assertFalse(r["independently_authenticated_signer"])
        self.assertNotEqual(self.opin,self.cpin)
        self.compare()
    def test_19_modified_canonical_os_report_rejected_by_real_ed25519(self):
        x=lab.parse(self.cfile.read_bytes());x["ppid"]=x["ppid"]+1
        self.cfile.write_bytes(lab.canonical(x))
        r=lab.verify(self.cfile,self.csig,self.callow,self.cpub,self.cpin)
        self.assertIn("REAL_OS_REPORT_ED25519_SIGNATURE_INVALID",r["detail"])
        self.assertEqual(r["reason"],"OS_REPORT_SIGNATURE_QUARANTINED")
    def test_20_wrong_signer_and_public_pin_disqualified(self):
        wrong=lab.verify(self.cfile,self.csig,self.oallow,self.cpub,self.cpin)
        self.assertIn("REAL_OS_REPORT_ED25519_SIGNATURE_INVALID",wrong["detail"])
        altered=lab.verify(self.cfile,self.csig,self.callow,self.cpub,self.opin)
        self.assertIn("OS_REPORT_KEY_NOT_BOUND",altered["detail"])
    def test_21_distinct_actual_keys_rotate_locally_not_admin_domain(self):
        self.rotate_revocation()
        self.assertEqual(rotate.audit(self.root)["active"],self.cpin)
        self.compare()
    def test_22_real_signed_successor_revocation_still_denies_identity(self):
        self.rotate_revocation()
        r=self.root/"revoke.json"
        b=rotate.intent(rotate.REVOKE,2,self.cpin,self.cpin,
                        rotate.sha(self.receipt.read_bytes()),"2"*64)
        r.write_bytes(b)
        sig=rotate.sign_intent(r,self.ckey)
        out=rotate.append(self.root,b,sig,self.callow)
        self.assertEqual(out["reason"],"REAL_LOCAL_SIGNED_REVOCATION_DENIED")
        self.assertTrue(rotate.audit(self.root)["revoked"])
        self.compare()
    def test_23_second_actual_os_process_restores_revocation_snapshot(self):
        self.rotate_revocation()
        old=self.root/"pre_revocation.sqlite3"
        shutil.copy2(rotate.ledger_path(self.root),old)
        r=self.root/"revoke.json"
        b=rotate.intent(rotate.REVOKE,2,self.cpin,self.cpin,
                        rotate.sha(self.receipt.read_bytes()),"2"*64)
        r.write_bytes(b);s=rotate.sign_intent(r,self.ckey)
        rotate.append(self.root,b,s,self.callow)
        self.assertTrue(rotate.audit(self.root)["revoked"])
        fdgate.real_atomic_replace(old,rotate.ledger_path(self.root))
        self.assertFalse(rotate.audit(self.root)["revoked"])
        self.assertEqual(self.compare()["independent_admin_custody"],"NOT_PROVEN")
    def test_24_real_full_os_identity_and_revocation_cli_exit_two(self):
        result=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),"--exercise"],
                              text=True,capture_output=True,timeout=180)
        self.assertEqual(result.returncode,2,result.stderr)
        x=json.loads(result.stdout)
        self.assertEqual(x["reason"],"REAL_OS_ADMIN_CUSTODY_DISQUALIFICATION_QUALIFIED")
        self.assertEqual(x["real_child_processes"],"DIFFERENT_PIDS_SAME_UID_EUID")
        self.assertEqual(x["real_os_uid_euid_separation"],"NOT_OBSERVED")
        self.assertEqual(x["real_signed_successor_revocation"],"LOCAL_BLOCK")
        self.assertEqual(x["local_revocation_after_second_process_rollback"],"LOST_LOCALLY")
        self.assertEqual(x["real_second_process_signature_inode_swap"],"QUARANTINED")
        self.assertEqual(x["native_admission"],"BLOCK")
        self.assertFalse(x["native_authority_granted"])

if __name__=="__main__":unittest.main()
