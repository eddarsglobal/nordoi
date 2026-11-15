"""G0.3 R6F BUREAU-001 R4 — 24 real file/custody/race negative tests.

16 portable cases examine metadata/keys/denial. Eight POSIX OpenSSH native
cases use real Ed25519 keys and real second OS process os.replace. The
synchronization hook creates a deterministic interleaving; no synthetic
cryptographic signature or simulated file operation.
"""
import base64
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import g03_r6f_r4_fd_race as lab

def ed25519(raw=b"\x12"*32, comment=b"test"):
    wire=b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20"+raw
    return b"ssh-ed25519 "+base64.b64encode(wire)+b" "+comment+b"\n"

class PortableFDSourceAndDeny(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi-r6f-r4-portable-")
        self.root=Path(self.temp.name)
        self.pub=self.root/"owner.pub"
        self.allowed=self.root/"owner.allowed"
        self.pub.write_bytes(ed25519())
        self.allowed.write_bytes(lab.r3.expected_allowed_bytes(self.pub.read_bytes()))
        if os.name!="nt":
            os.chmod(self.root,0o700)
            os.chmod(self.allowed,0o600)
        self.pin=lab.r3.keys.public_sha(self.pub)

    def tearDown(self):
        self.temp.cleanup()

    def test_01_exact_protocol_and_frozen_r3_parent(self):
        self.assertEqual(lab.PARENT,"2dd46dc07568bf382c49b1c8bca0cba8900b7b1f")
        self.assertIn("FD-PATH-RACE-DENY",lab.PROTOCOL)

    def test_02_no_native_authority_can_be_issued(self):
        x=lab.deny("R4_TEST")
        self.assertEqual(x["status"],"BLOCK")
        self.assertEqual(x["admission"],"NOT_AUTHORIZED")
        self.assertEqual(x["native_admission_gates_unmet"],12)
        self.assertEqual(x["production"],"NOT_AUTHORIZED")

    def test_03_exact_original_allowlist_matches_ssh_key(self):
        self.assertEqual(self.allowed.read_bytes(),
                         lab.r3.expected_allowed_bytes(self.pub.read_bytes()))
        self.assertTrue(lab.r3.bind(self.pub,self.allowed,self.pin))

    def test_04_changed_signature_identity_denied_before_privilege(self):
        badpub=self.root/"rogue.pub"
        badpub.write_bytes(ed25519(raw=b"\x13"*32))
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(badpub,self.allowed,lab.r3.keys.public_sha(badpub))

    def test_05_allowed_signer_principal_substitution_rejected(self):
        self.allowed.write_bytes(b"other-id "+self.pub.read_bytes())
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_06_multiple_accepted_keys_rejected(self):
        with self.allowed.open("ab") as f:
            f.write(lab.ssh.IDENTITY.encode()+b" "+ed25519(raw=b"\x14"*32))
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_07_missing_pinned_public_file_denied(self):
        self.pub.unlink()
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_08_missing_allowed_signers_denied(self):
        self.allowed.unlink()
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_09_wrong_pinned_public_key_digest_denied(self):
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,"0"*64)

    def test_10_extra_allowlist_options_refused(self):
        self.allowed.write_bytes(lab.ssh.IDENTITY.encode()
                                 +b' namespaces="r6e" '+self.pub.read_bytes())
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_11_invalid_ed25519_wire_encoding_refused(self):
        bad=b"ssh-ed25519 "+base64.b64encode(b"not a wire key")+b"\n"
        with self.assertRaises(lab.r3.Refused):
            lab.r3.parse_ed25519_public(bad)

    def test_12_extra_public_key_line_rejected(self):
        with self.assertRaises(lab.r3.Refused):
            lab.r3.parse_ed25519_public(self.pub.read_bytes()*2)

    def test_13_empty_allowlist_refused(self):
        self.allowed.write_bytes(b"")
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    def test_14_oversized_allowlist_refused(self):
        self.allowed.write_bytes(b"x"*(lab.MAX_ALLOW+1))
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,self.allowed,self.pin)

    @unittest.skipIf(os.name=="nt","symlink creation not assumed on Windows")
    def test_15_symlink_is_not_a_frozen_allowed_fd(self):
        link=self.root/"linked.allowed"
        link.symlink_to(self.allowed.name)
        with self.assertRaises(lab.r3.Refused):
            lab.r3.bind(self.pub,link,self.pin)

    def test_16_untrusted_record_never_proves_implementation_or_external_quorum(self):
        x=lab.assess_lab(None)
        self.assertEqual(x["status"],"BLOCK")
        self.assertFalse(x["native_authority_granted"])

@unittest.skipUnless(lab.is_posix_ssh(),
                     "genuine local POSIX OpenSSH FD and signed OS process required")
class ActualSecondProcessRace(unittest.TestCase):
    def setUp(self):
        self.home,self.data=lab.r6e.create_lab()

    def tearDown(self):
        self.home.cleanup()

    def victim(self,suffix):
        dst=self.data["root"]/("nordoi-r4-victim-"+suffix+".allowed")
        lab.copy_private(self.data["guardian_allowed"],dst)
        return dst

    def rogue(self):
        return lab.r3.create_rogue_signed_receipt(self.data)

    def test_17_uncontended_six_real_local_signatures_held_fd(self):
        result=lab.assess_lab(self.data)
        self.assertEqual(result["reason"],
                         "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY")
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["native_admission_gates_unmet"],12)

    def test_18_r3_path_to_ssh_process_real_atomic_replacement_misattribution(self):
        signature,allowed,_=self.rogue()
        victim=self.victim("legacy")
        old=lab.legacy_r3_race(self.data,signature,allowed,victim)
        self.assertEqual(old["reason"],
                         "R3_SIX_LOCAL_SSHSIGS_BOUND_TO_EXACT_DECLARED_KEYS_NO_EXTERNAL_CUSTODY")
        self.assertEqual(old["admission"],"NOT_AUTHORIZED")
        self.assertEqual(victim.read_bytes(),allowed.read_bytes())

    def test_19_r4_keeps_original_fd_refuses_same_race(self):
        signature,allowed,_=self.rogue()
        victim=self.victim("new")
        called=[False]
        def swap(command):
            if not called[0] and "-s" in command and command[command.index("-s")+1]==str(signature):
                lab.atomic_child_replace(allowed,victim)
                called[0]=True
        result=lab.assess_lab(self.data,local_signature=signature,
                              local_allowed=victim,before_verify=swap)
        self.assertTrue(called[0])
        self.assertEqual(result["reason"],"R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED")
        self.assertEqual(result["admission"],"NOT_AUTHORIZED")
        self.assertEqual(result["native_admission_gates_unmet"],12)

    def test_20_original_fd_snapshot_detects_child_atomic_path_inode_change(self):
        rogue_sig,rogue_allow,_=self.rogue()
        victim=self.victim("inode")
        with lab.fdgate.OpenSnapshot(victim) as frozen:
            old_bytes=os.pread(frozen.fd,lab.MAX_ALLOW,0)
            lab.atomic_child_replace(rogue_allow,victim)
            self.assertEqual(os.pread(frozen.fd,lab.MAX_ALLOW,0),old_bytes)
            with self.assertRaises(lab.fdgate.Refused):
                frozen.check()

    def test_21_real_distinct_key_and_same_admin_owned_path(self):
        _,allowed,pub=self.rogue()
        self.assertNotEqual(lab.r3.keys.public_sha(pub),
                            lab.r3.keys.public_sha(self.data["guardian_public"]))
        self.assertEqual(allowed.parent,self.data["root"])
        self.assertEqual(os.stat(allowed).st_uid,os.getuid())

    def test_22_rogue_substitution_before_open_is_rejected(self):
        signature,allowed,_=self.rogue()
        result=lab.assess_lab(self.data,local_signature=signature,
                              local_allowed=allowed)
        self.assertEqual(result["reason"],"R4_SIGNER_FILE_RACE_OR_PROVENANCE_QUARANTINED")
        self.assertEqual(result["status"],"BLOCK")

    def test_23_damaged_sshsig_cannot_pass_fd_pinned_verify(self):
        path=self.data["root"]/"nordoi-r4-corrupt.sig"
        path.write_bytes(b"not-an-sshsig")
        os.chmod(path,0o600)
        result=lab.assess_lab(self.data,local_signature=path)
        self.assertNotEqual(result["reason"],
                            "R4_SIX_LOCAL_SSHSIGS_VERIFIED_WITH_HELD_ALLOWED_FDS_NO_EXTERNAL_CUSTODY")
        self.assertEqual(result["status"],"BLOCK")

    def test_24_real_cli_exercise_proves_interleaving_no_native_grant(self):
        p=subprocess.run([sys.executable,"-B",str(Path(lab.__file__)),
                          "--exercise"],capture_output=True,text=True,timeout=240)
        self.assertEqual(p.returncode,2,p.stderr)
        result=json.loads(p.stdout)
        self.assertEqual(result["reason"],
                         "R4_REAL_SECOND_PROCESS_SSHSIG_PATH_RACE_REPRODUCED_AND_QUARANTINED")
        self.assertTrue(result["second_real_os_process_atomically_replaced_allowlist"])
        self.assertTrue(result["r4_uses_already_open_allowed_signers_fds"])
        self.assertFalse(result["native_authority_granted"])
        self.assertEqual(result["native_admission_gates_unmet"],12)

if __name__=="__main__":
    unittest.main()
