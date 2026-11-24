"""Real Rust evidence, independent receiver subprocess and hostile packages."""
import base64
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app030_verified_reservation_archive"))
import test_app030 as legacy
import transfer_app031 as app

class Transfer(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Archive("test_01_real_complete_archive")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.base=self.t.f.base
        self.packet=self.base/"transfer.json"
        self.receiver=self.base/"receiver"
        self.archive=self.t.dest

    def prepare(self):
        self.ap=self.t.make()["archive_sha256"]
        self.tp=app.export(self.archive,self.ap,self.packet)["transfer_sha256"]

    def receive(self,**change):
        a=dict(packet=self.packet,transfer_pin=self.tp,archive_pin=self.ap,target=self.receiver)
        a.update(change)
        return app.receive(**a)

    def blocked(self,**change):
        with self.assertRaises(Exception): self.receive(**change)
        self.assertFalse(self.receiver.exists())

    def mutate(self,fn):
        obj=json.loads(self.packet.read_bytes())
        fn(obj)
        raw=app.portable.serialize(obj)
        self.packet.write_bytes(raw)
        return app.sha(raw)

    def test_01_real_receiver_separate_process(self):
        self.prepare()
        cmd=[sys.executable,"-B",str(HERE/"transfer_app031.py"),"receive",
             "--packet",str(self.packet),"--transfer-pin",self.tp,"--archive-pin",self.ap,
             "--target",str(self.receiver)]
        p=subprocess.run(cmd,capture_output=True,timeout=120)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        r=json.loads(p.stdout)
        self.assertEqual(r["status"],"TRANSFER_RECEIVED_VERIFIED")
        self.assertEqual(r["verified_native_jobs"],6)
        self.assertFalse(r["execution_authorized"])

    def test_02_receiver_without_original_archive(self):
        self.prepare()
        # Move the archive within the disposable test fixture only.
        self.archive.rename(self.base/"unavailable-to-receiver")
        r=self.receive()
        self.assertEqual(r["status"],"TRANSFER_RECEIVED_VERIFIED")

    def test_03_wrong_transfer_pin(self):
        self.prepare()
        self.blocked(transfer_pin="0"*64)

    def test_04_wrong_archive_pin(self):
        self.prepare()
        self.blocked(archive_pin="0"*64)

    def test_05_path_traversal_even_rehashed(self):
        self.prepare()
        p=self.mutate(lambda o:o["files"][0].update(path="../escape"))
        self.blocked(transfer_pin=p)
        self.assertFalse((self.base/"escape").exists())

    def test_06_duplicate_entry_even_rehashed(self):
        self.prepare()
        p=self.mutate(lambda o:o["files"].append(o["files"][0]))
        self.blocked(transfer_pin=p)

    def test_07_forged_native_bytes_even_rehashed(self):
        self.prepare()
        def alter(o):
            x=next(x for x in o["files"] if x["path"].startswith("native/"))
            data=b"fake"
            x.update(bytes=len(data),sha256=app.sha(data),base64=base64.b64encode(data).decode())
        self.blocked(transfer_pin=self.mutate(alter))

    def test_08_missing_required_entry(self):
        self.prepare()
        self.blocked(transfer_pin=self.mutate(lambda o:o.update(files=[x for x in o["files"] if x["path"]!="ARCHIVE.sha256"])))

    def test_09_existing_destination_untouched(self):
        self.prepare()
        self.receiver.mkdir()
        keep=self.receiver/"keep"
        keep.write_bytes(b"keep")
        with self.assertRaises(Exception): self.receive()
        self.assertEqual(keep.read_bytes(),b"keep")

    def test_10_export_existing_packet_untouched(self):
        self.prepare()
        before=self.packet.read_bytes()
        with self.assertRaises(Exception): app.export(self.archive,self.ap,self.packet)
        self.assertEqual(before,self.packet.read_bytes())

    def test_11_noncanonical_and_truncated(self):
        self.prepare()
        raw=self.packet.read_bytes()
        for b in (raw+b" ",raw[:-2]):
            self.packet.write_bytes(b)
            self.blocked(transfer_pin=app.sha(b))

    def test_12_original_preserved_no_transport_claim(self):
        self.prepare()
        before=app.collect(self.archive,self.ap)
        r=self.receive()
        self.assertEqual(before,app.collect(self.archive,self.ap))
        self.assertFalse(r["sender_identity_authenticated"])
        self.assertFalse(r["transport_encrypted"])
        self.assertEqual(r["distributed_anti_replay"],"NOT_PROVEN")
        self.assertFalse(r["reservation_release_authorized"])

if __name__=="__main__":
    unittest.main()
