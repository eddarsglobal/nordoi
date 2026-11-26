"""Real signed evidence; distinct valid branches compete for one exclusive marker."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
from unittest.mock import patch
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
sys.path.insert(0,str(HERE.parent/"app040_journal_seal_continuity"))
import test_app040 as legacy
import successor_app041 as app

class Successor(unittest.TestCase):
    def setUp(self):
        self.t=legacy.Continuity("test_01_real_append_only_transition")
        self.t.setUp()
        self.addCleanup(self.t.doCleanups)
        self.t.extend()
        self.transition=self.t.publish()
        self.base=self.t.t.base
        self.successors=self.base/"successors"
        self.successors.mkdir()
        self.marker=self.successors/self.t.old["seal_sha256"]

    def args(self,**change):
        a=self.t.args()
        a.pop("output")
        a.update(successors=self.successors,transition_pin=self.transition["transition_sha256"])
        a.update(change)
        return a

    def accept(self,**change):
        return app.accept(**self.args(**change))

    def audit(self,**change):
        return app.audit(**self.args(**change))

    def cmd(self,action="accept",**change):
        return [sys.executable,"-B",str(HERE/"successor_app041.py"),action]+[
            s for k,v in self.args(**change).items() for s in ("--"+k.replace("_","-"),str(v))]

    def alternatives(self):
        # Both journal snapshots extend the same prior seal and are actually audited.
        original=self.t.t.t.journal
        first=self.base/"first-journal"
        shutil.copytree(original,first)
        self.t.t.t.directory.rename(self.base/"detached-first")
        self.t.t.t.make_conflict("2"*64)
        self.t.t.t.preserve()
        second=self.base/"second-seal.json"
        seal=self.t.t.seal(output=second)
        receipt=self.t.publish(output=self.base/"second-transition.json",new_seal=second,
                               new_pin=seal["seal_sha256"])
        a=dict(journal=first)
        b=dict(new_seal=second,new_pin=seal["seal_sha256"],transition_pin=receipt["transition_sha256"])
        for change in (a,b):
            args=self.args(**change)
            args.pop("successors")
            pin=args.pop("transition_pin")
            app.candidate(pin,args)
        return a,b

    def test_01_real_successor_decision(self):
        r=self.accept()
        self.assertEqual(r["status"],"SUCCESSOR_ACCEPTED_ONCE")
        self.assertEqual(self.audit()["record"],r["record"])

    def test_02_duplicate_after_restart(self):
        self.accept()
        p=subprocess.run(self.cmd(),capture_output=True,timeout=240)
        self.assertEqual(p.returncode,3,p.stdout+p.stderr)
        self.assertEqual(json.loads(p.stdout)["status"],"DUPLICATE_SUCCESSOR_BLOCKED")

    def test_03_distinct_valid_alternative_blocked(self):
        a,b=self.alternatives()
        first=self.accept(**a)["record"]
        blocked=self.accept(**b)
        self.assertEqual(blocked["status"],"ALTERNATIVE_SUCCESSOR_BLOCKED")
        self.assertEqual(blocked["record"],first)
        self.assertNotEqual(blocked["rejected_successor_sha256"],first["successor_seal_sha256"])
        self.assertEqual(self.audit(**a)["record"],first)
        with self.assertRaises(Exception): self.audit(**b)

    def test_04_two_real_competing_processes(self):
        a,b=self.alternatives()
        ps=[subprocess.Popen(self.cmd(**c),stdout=subprocess.PIPE,stderr=subprocess.PIPE) for c in (a,b)]
        try:
            out=[p.communicate(timeout=240) for p in ps]
            self.assertEqual(sorted(p.returncode for p in ps),[0,3],out)
            winner=0 if ps[0].returncode==0 else 1
            self.assertEqual(self.audit(**(a,b)[winner])["status"],"SUCCESSOR_DECISION_VERIFIED")
            with self.assertRaises(Exception): self.audit(**(a,b)[1-winner])
        finally:
            for p in ps:
                if p.poll() is None:
                    p.kill()
                    p.communicate()

    def test_05_invalid_transition_pin_no_marker(self):
        with self.assertRaises(Exception): self.accept(transition_pin="0"*64)
        self.assertFalse(self.marker.exists())

    def test_06_wrong_predecessor_no_marker(self):
        with self.assertRaises(Exception): self.accept(old_pin="0"*64)
        self.assertEqual(list(self.successors.iterdir()),[])

    def test_07_incomplete_marker_permanent(self):
        self.marker.mkdir()
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()
        self.assertEqual(list(self.marker.iterdir()),[])

    def test_08_injected_failure_permanent(self):
        real=app.portable.safe_write
        destination=self.marker.resolve(strict=False)/"TRANSITION.json"
        injected=[]
        def fail(path,raw):
            if Path(path).resolve(strict=False)==destination:
                injected.append(destination)
                raise OSError("injected successor failure")
            return real(path,raw)
        with patch.object(app.portable,"safe_write",side_effect=fail):
            with self.assertRaisesRegex(OSError,"^injected successor failure$"): self.accept()
        self.assertEqual(injected,[destination])
        self.assertTrue((self.marker/"START.json").is_file())
        self.assertFalse((self.marker/"COMPLETE.json").exists())
        with self.assertRaises(Exception): self.accept()
        with self.assertRaises(Exception): self.audit()

    def test_09_tampered_transition_refused(self):
        self.accept()
        (self.marker/"TRANSITION.json").write_bytes(b"{}")
        with self.assertRaises(Exception): self.audit()
        with self.assertRaises(Exception): self.accept()

    def test_10_readonly_audit(self):
        self.accept()
        before={p.name:p.read_bytes() for p in self.marker.iterdir()}
        p=subprocess.run(self.cmd("audit"),capture_output=True,timeout=240)
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertEqual({p.name:p.read_bytes() for p in self.marker.iterdir()},before)

    def test_11_overlap_and_invalid_current_journal(self):
        with self.assertRaises(Exception): self.accept(successors=self.t.t.t.journal)
        (self.t.t.t.directory/"conflict-ACK.sig").write_bytes(b"invalid")
        with self.assertRaises(Exception): self.accept()
        self.assertFalse(self.marker.exists())

    def test_12_scope_and_authority(self):
        r=self.accept()
        self.assertEqual(r["distributed_fork_prevention"],"NOT_PROVEN")
        self.assertEqual(r["global_antirollback"],"NOT_PROVEN")
        self.assertFalse(r["execution_authorized"])
        self.assertFalse(r["reservation_release_authorized"])
        self.assertFalse(r["nonce_reset_authorized"])

if __name__=="__main__":
    unittest.main()
