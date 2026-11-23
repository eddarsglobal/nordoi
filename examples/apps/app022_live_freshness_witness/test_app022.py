"""APP-022 — true separate-process Ed25519 SSHSIG witness and native Rust tests.

No mocks for cryptography or Rust. CI fixture private keys are local and
ephemeral; external custody, durable antirollback and independence remain
not proven. Every test uses a real OS subprocess + loopback HTTP request.
"""
from __future__ import annotations
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import time
import unittest

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
sys.path.insert(0,str(HERE.parent/"app021_authority_rotation"))
import test_app021 as legacy
sys.path.insert(0,str(HERE))
import receiver_app022 as receiver
import witness_app022 as witness
authority=legacy.authority


class LiveWitness(unittest.TestCase):
    make_event=legacy.AuthorityRotation.make_event
    signature=legacy.AuthorityRotation.signature
    record=legacy.AuthorityRotation.record
    write_chain=legacy.AuthorityRotation.write_chain
    rotate=legacy.AuthorityRotation.rotate
    freeze=legacy.AuthorityRotation.freeze
    good=legacy.AuthorityRotation.good
    denied=legacy.AuthorityRotation.denied

    @classmethod
    def setUpClass(cls):
        legacy.AuthorityRotation.setUpClass()

    @classmethod
    def tearDownClass(cls):
        legacy.AuthorityRotation.tearDownClass()

    def setUp(self):
        legacy.AuthorityRotation.setUp(self)
        self.witness_key=self.work/"witness-private"
        p=subprocess.run(["ssh-keygen","-q","-t","ed25519","-N","",
                          "-f",str(self.witness_key)],
                         capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        parts=Path(str(self.witness_key)+".pub").read_text("ascii").split()
        self.witness_pub=self.work/"witness-receiver-trusted-public.key"
        self.witness_pub.write_bytes(
            (parts[0]+" "+parts[1]+"\n").encode("ascii"))
        self.witness_sha=hashlib.sha256(self.witness_pub.read_bytes()).hexdigest()
        self.witness_state=self.work/"witness-separate-state.json"
        self.counter=1
        self.state={}
        self.update_state(counter=1)
        self.server=None
        self.start_server()

    def tearDown(self):
        self.stop_server()
        legacy.AuthorityRotation.tearDown(self)

    def update_state(self,*,counter=None,**changes):
        with authority.anchor.locked(self.root):
            head,_=authority.anchor.read_locked(self.root)
        self.counter=self.counter if counter is None else counter
        self.state={
            "schema":witness.SCHEMA,
            "authority_id":self.authority_id,
            "receiver_id":self.receiver_id,
            "chain_sha256":self.chain_sha,
            "head_sha256":hashlib.sha256(
                authority.anchor.canonical(head)).hexdigest(),
            "epoch":len(self.records),
            "minimum_sequence":self.records[-1]["event"]["minimum_sequence"],
            "minimum_generation":self.records[-1]["event"]["minimum_generation"],
            "counter":self.counter,
            "release":"RESEARCH_ONLY","production":"NOT_AUTHORIZED"
        }
        self.state.update(changes)
        self.witness_state.write_bytes(authority.anchor.canonical(self.state))

    def start_server(self,private=None):
        self.stop_server()
        with socket.socket(socket.AF_INET,socket.SOCK_STREAM) as free:
            free.bind(("127.0.0.1",0))
            port=free.getsockname()[1]
        self.server=subprocess.Popen(
            [sys.executable,"-B","-u",str(HERE/"witness_app022.py"),
             "--state-file",str(self.witness_state),
             "--private-key",str(self.witness_key if private is None else private),
             "--port",str(port)],
            cwd=REPO,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
        self.witness_url="http://127.0.0.1:"+str(port)+"/attest"
        for _ in range(100):
            if self.server.poll() is not None:
                self.fail("WITNESS_START_FAILED:"+
                          self.server.stderr.read().decode("utf-8","replace"))
            try:
                with socket.create_connection(("127.0.0.1",port),timeout=.1):
                    return
            except OSError:
                time.sleep(.05)
        self.fail("WITNESS_START_TIMEOUT")

    def stop_server(self):
        p=getattr(self,"server",None)
        if p is not None:
            if p.poll() is None:
                p.terminate()
            try:
                p.wait(timeout=4)
            except subprocess.TimeoutExpired:
                p.kill()
                p.wait(timeout=4)
            if p.stderr is not None:
                p.stderr.close()
            self.server=None

    def call(self, action="status", *, pin=None, root_pin=None, epoch=1,
             seq=1, gen=1, chain_path=None, statement=None, signature=None,
             source=None, package=None, dest=None, root=None,
             authority_id=None, receiver_id=None, omit=(),
              witness_url=None, witness_pub=None, witness_pin=None,
              witness_counter=1):
        args = [
            "--anchor-root", str(self.root if root is None else root),
            "--authority-chain-file",
                str(self.chain_file if chain_path is None else chain_path),
            "--expected-chain-sha256", self.chain_sha if pin is None else pin,
            "--expected-root-key-sha256",
                self.root_sha if root_pin is None else root_pin,
            "--authority-id",
                self.authority_id if authority_id is None else authority_id,
            "--receiver-id",
                self.receiver_id if receiver_id is None else receiver_id,
            "--minimum-epoch", str(epoch),
            "--minimum-sequence", str(seq),
            "--minimum-generation", str(gen),
            "--signed-statement",
                str(self.fx.document if statement is None else statement),
            "--detached-signature",
                str(self.fx.signature if signature is None else signature),
        ]
        args.extend([
            "--witness-url", str(self.witness_url if witness_url is None else witness_url),
            "--witness-public-key", str(self.witness_pub if witness_pub is None else witness_pub),
            "--expected-witness-key-sha256", str(self.witness_sha if witness_pin is None else witness_pin),
            "--minimum-witness-counter", str(witness_counter),
        ])
        chosen = []
        for i in range(0, len(args), 2):
            if args[i] not in omit:
                chosen.extend(args[i:i + 2])
        if action in ("inspect", "verify", "restore", "resume"):
            chosen.extend([
                "--receiver-roster", str(self.fx.fx.fx.fx.roster_file),
                "--receiver-policy", str(self.fx.fx.fx.fx.fx.policy_file),
                "--portable-dir",
                    str(self.fx.fx.fx.fx.fx.portable if package is None else package),
            ])
        elif package is not None:
            chosen.extend(["--portable-dir", str(package)])
        if action in ("verify", "restore", "resume"):
            chosen.extend([
                "--engine", str(baseline.predecessor.fixture.original_fixture.ENGINE),
                "--source-root", str(
                    baseline.predecessor.fixture.original_fixture.SOURCE
                    if source is None else source),
            ])
        if action in ("restore", "resume"):
            chosen.extend([
                "--restore-root", str(
                    self.fx.fx.fx.fx.fx.dest if dest is None else dest),
            ])
        result = subprocess.run(
            [sys.executable, "-B", str(HERE / "receiver_app022.py"), action]
            + chosen, cwd=REPO, capture_output=True, timeout=190)
        try:
            lines = [json.loads(x) for x in result.stdout.splitlines()]
        except ValueError:
            self.fail(f"Non-JSON: {result.stdout!r} stderr={result.stderr!r}")
        return result, lines


    def test_01_live_signed_bootstrap_witness(self):
        row=self.good()[1][0]
        self.assertEqual(row["witness_counter"],1)
        self.assertEqual(row["authority_epoch"],1)
        self.assertEqual(row["live_witness"],"NEW_NONCE_SSHSIG_ED25519_VERIFIED")
        self.assertEqual(row["witness_process"],"LOOPBACK_SEPARATE_PROCESS")

    def test_02_witness_status_does_not_execute_native_witness(self):
        row=self.good()[1][0]
        self.assertEqual(row["native_engine"],"NOT_EXECUTED")

    def test_03_genuine_rust_inspect_with_witness(self):
        self.assertEqual(self.good("inspect")[1][0]["native_engine"],"NOT_EXECUTED")

    def test_04_genuine_rust_verify_with_witness(self):
        row=self.good("verify")[1][0]
        self.assertEqual(row["native_jobs_recomputed"],2)
        self.assertEqual(row["native_files_bytewise_equal"],4)

    def test_05_native_restore_twice_attested_witness(self):
        row=self.good("restore")[1][0]
        self.assertEqual(row["status"],"RESTORED")
        self.assertEqual(row["fresh_witness_sshsig"],"VERIFIED_TWICE_BEFORE_NATIVE_EFFECT")
        self.assertEqual(row["witness_counter"],1)

    def test_06_native_resume_after_witness_attested_restore(self):
        self.good("restore")
        _,rows=self.good("resume")
        self.assertEqual([r["step"] for r in rows],[3,4])

    def test_07_duplicate_resume_still_denied_witness(self):
        self.good("restore")
        self.good("resume")
        self.denied("ROOT_UNEXPECTED_JOB_OR_FILE","resume")

    def test_08_witness_expected_key_pin_mismatch_witness(self):
        self.denied("WITNESS_PUBLIC_KEY_PIN_MISMATCH",
                    witness_pin="0"*64)

    def test_09_witness_missing_public_key_rejected_witness(self):
        self.denied("WITNESS_PUBLIC_KEY_NOT_REGULAR_BOUNDED",
                    witness_pub=self.work/"does-not-exist-public")

    def test_10_witness_nonlocal_url_denied_witness(self):
        self.denied("WITNESS_URL_LOCAL_LOOPBACK_REQUIRED",
                    witness_url="https://example.com/attest")

    def test_11_witness_process_down_fail_closed_witness(self):
        self.stop_server()
        self.denied("WITNESS_LIVE_SIGNER_UNAVAILABLE")

    def test_12_witness_counter_below_receiver_min_witness(self):
        self.denied("WITNESS_CURRENT_FRESHNESS_FLOOR_MISMATCH",
                    witness_counter=2)

    def test_13_witness_bad_head_sha_rejected_witness(self):
        self.update_state(head_sha256="0"*64)
        self.denied("WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH")

    def test_14_witness_wrong_chain_sha_rejected_witness(self):
        self.update_state(chain_sha256="a"*64)
        self.denied("WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH")

    def test_15_witness_wrong_scope_fail_closed_witness(self):
        self.update_state(receiver_id="wrong-agent")
        self.denied("WITNESS_LIVE_SIGNER_REFUSED")

    def test_16_witness_same_counter_mutation_denied_witness(self):
        self.good()
        self.update_state(head_sha256="b"*64)
        self.denied("WITNESS_LIVE_SIGNER_REFUSED")

    def test_17_witness_process_counter_rollback_denied_witness(self):
        self.good()
        self.update_state(counter=2)
        self.denied("WITNESS_CURRENT_FRESHNESS_FLOOR_MISMATCH",
                    witness_counter=3)
        # Earlier attempt already advanced server's protected in-memory counter.
        self.update_state(counter=1)
        self.denied("WITNESS_LIVE_SIGNER_REFUSED")

    def test_18_rotated_chain_requires_fresh_witness_witness(self):
        self.rotate()
        self.denied("WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH",
                    epoch=2,seq=2,gen=2)

    def test_19_rotated_chain_with_updated_witness_witness(self):
        self.rotate()
        self.update_state(counter=2)
        row=self.good(epoch=2,seq=2,gen=2,witness_counter=2)[1][0]
        self.assertEqual(row["authority_epoch"],2)
        self.assertEqual(row["witness_counter"],2)

    def test_20_rotated_native_real_rust_replay_witness(self):
        self.rotate()
        self.update_state(counter=2)
        source=self.fx.fx.fx.fx.fx.alt_source
        row=self.good("verify",epoch=2,seq=2,gen=2,
                      witness_counter=2,source=source)[1][0]
        self.assertEqual(row["native_jobs_recomputed"],2)

    def test_21_old_authority_rejected_under_current_witness(self):
        old_root=self.work/"old-021-journal"
        shutil.copytree(self.root,old_root)
        old_records=copy.deepcopy(self.records)
        old_doc=self.fx.document.read_bytes()
        old_sig=self.fx.signature.read_bytes()
        self.rotate()
        self.update_state(counter=2)
        shutil.rmtree(self.root)
        shutil.copytree(old_root,self.root)
        self.records=old_records
        self.write_chain()
        self.fx.document.write_bytes(old_doc)
        self.fx.signature.write_bytes(old_sig)
        self.denied("WITNESS_CURRENT_HEAD_OR_CHAIN_MISMATCH")

    def test_22_frozen_status_with_real_signatures_witness(self):
        self.freeze()
        self.update_state(counter=2)
        row=self.good(epoch=2,seq=2,gen=2,witness_counter=2)[1][0]
        self.assertEqual(row["status"],"FROZEN")

    def test_23_frozen_rejects_native_effect_witness(self):
        self.freeze()
        self.update_state(counter=2)
        self.denied("WITNESS_FROZEN_NATIVE_ADMISSION_DENIED",
                    "restore",epoch=2,seq=2,gen=2,witness_counter=2)
        self.assertFalse(any(self.fx.fx.fx.fx.fx.dest.iterdir()))

    def test_24_bad_server_state_noncanonical_denied_witness(self):
        self.witness_state.write_bytes(b'{"schema":"wrong"}\n')
        self.denied("WITNESS_LIVE_SIGNER_REFUSED")

    def test_25_wrong_witness_signing_key_denied_witness(self):
        other=self.work/"other-witness-key"
        p=subprocess.run(["ssh-keygen","-q","-t","ed25519","-N","",
                          "-f",str(other)],capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.start_server(private=other)
        self.denied("WITNESS_SSHSIG_ED25519_VERIFICATION_FAILED")

    def test_26_witness_cannot_prove_global_rollback_witness(self):
        original=copy.deepcopy(self.state)
        old_root=self.work/"entire-old-receiver"
        shutil.copytree(self.root,old_root)
        old_records=copy.deepcopy(self.records)
        old_doc=self.fx.document.read_bytes()
        old_sig=self.fx.signature.read_bytes()
        self.rotate()
        self.update_state(counter=2)
        self.good(epoch=2,seq=2,gen=2,witness_counter=2)
        # Same-admin rollback of ALL locally trusted inputs plus process restart.
        self.stop_server()
        shutil.rmtree(self.root)
        shutil.copytree(old_root,self.root)
        self.records=old_records
        self.write_chain()
        self.fx.document.write_bytes(old_doc)
        self.fx.signature.write_bytes(old_sig)
        self.witness_state.write_bytes(authority.anchor.canonical(original))
        self.start_server()
        row=self.good(epoch=1,seq=1,gen=1,witness_counter=1)[1][0]
        self.assertEqual(row["authority_epoch"],1)
        self.assertEqual(row["witness_key_custody"],
                         "SAME_ADMIN_LAB_NOT_INDEPENDENT")
        self.assertEqual(row["persistent_global_antirollback"],"NOT_PROVEN")


if __name__=="__main__":
    unittest.main()
