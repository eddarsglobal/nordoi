"""APP-001: actual NORDOI engine tests, with no synthetic execution or humans."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE))
import run_app001

BIN=Path(os.getenv("NORDOI_APP001_EXE",
      str(ROOT/"target"/"debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))
SOURCE=HERE/"quality_gate.noi"
PKG=HERE/"pure_project"

def engine(*arguments):
    return subprocess.run([str(BIN.resolve(strict=True)),*map(str,arguments)],
        cwd=ROOT,capture_output=True,text=True,timeout=120)

class App001ActualEngineTests(unittest.TestCase):
    def test_01_actual_NORDOI_binary_exists(self):
        self.assertTrue(BIN.is_file(),BIN)
    def test_02_two_real_files_for_below_threshold(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,59,"--grant-output-dir",tmp)
            self.assertEqual(p.returncode,0,p.stderr)
            out=Path(tmp)/"quality-gate"
            self.assertEqual({f.name for f in out.iterdir()},
                {"decision.txt","report.txt"})
            for name,want in run_app001.expected(59).items():
                self.assertEqual((out/name).read_bytes(),want.encode())
            self.assertIn("status=COMMITTED",p.stderr)
            self.assertIn("grant-count=2",p.stderr)
    def test_03_threshold_exact_passes(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,60,"--grant-output-dir",tmp)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertEqual((Path(tmp)/"quality-gate"/"decision.txt").read_text(),
                             run_app001.expected(60)["decision.txt"])
    def test_04_boundary_zero_score(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,0,"--grant-output-dir",tmp)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertEqual((Path(tmp)/"quality-gate"/"report.txt").read_text(),
                             run_app001.expected(0)["report.txt"])
    def test_05_boundary_hundred_score(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,100,"--grant-output-dir",tmp)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertEqual((Path(tmp)/"quality-gate"/"report.txt").read_text(),
                             run_app001.expected(100)["report.txt"])
    def test_06_no_grant_is_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,73)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("capability denied",p.stderr.lower())
            self.assertFalse((Path(tmp)/"quality-gate").exists())
    def test_07_second_write_to_same_bundle_is_denied_no_overwrite(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            a=engine("bundle-write",SOURCE,73,"--grant-output-dir",tmp)
            self.assertEqual(a.returncode,0,a.stderr)
            bundle=Path(tmp)/"quality-gate"
            before={name:(bundle/name).read_bytes()
                    for name in ("decision.txt","report.txt")}
            b=engine("bundle-write",SOURCE,59,"--grant-output-dir",tmp)
            self.assertNotEqual(b.returncode,0)
            self.assertEqual(before,{name:(bundle/name).read_bytes()
                                    for name in before})
    def test_08_real_receipt_has_bounded_explicit_authority(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=engine("bundle-write",SOURCE,73,"--grant-output-dir",tmp)
            self.assertEqual(p.returncode,0,p.stderr)
            for claim in ("authority=EXPLICIT","grant-count=2",
                "ambient-authority=NONE","overwrite=DENIED",
                "partial-final-state=FORBIDDEN","publication=DIRECTORY-RENAME",
                "crash-durability=UNCLAIMED"):
                self.assertIn(claim,p.stderr)
    def test_09_user_facing_app_below_threshold_produces_real_files(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=subprocess.run([sys.executable,str(HERE/"run_app001.py"),
                  "--score","59","--engine",str(BIN),"--output-parent",tmp],
                  cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertIn("NORDOI_APP001=PASS",p.stdout)
            self.assertIn("APPROVED=false",p.stdout)
            roots=list(Path(tmp).glob("NORDOI_APP001_*/quality-gate"))
            self.assertEqual(len(roots),1)
            for name,v in run_app001.expected(59).items():
                self.assertEqual((roots[0]/name).read_text(),v)
    def test_10_user_facing_app_above_threshold(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=subprocess.run([sys.executable,str(HERE/"run_app001.py"),
                  "--score","73","--engine",str(BIN),"--output-parent",tmp],
                  cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertIn("APPROVED=true",p.stdout)
    def test_11_invalid_score_101_denied_before_creating_files(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=subprocess.run([sys.executable,str(HERE/"run_app001.py"),
                  "--score","101","--engine",str(BIN),"--output-parent",tmp],
                  cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("INVALID_SCORE",p.stderr)
            self.assertEqual(list(Path(tmp).iterdir()),[])
    def test_12_invalid_negative_score_denied_before_creating_files(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            p=subprocess.run([sys.executable,str(HERE/"run_app001.py"),
                  "--score","-1","--engine",str(BIN),"--output-parent",tmp],
                  cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(list(Path(tmp).iterdir()),[])
    def test_13_modular_pure_rule_real_score_59(self):
        p=engine("module-graph-run",PKG/"src","app.main",59)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertIn("modules=2",p.stdout)
        self.assertIn("result=INT(0)",p.stdout)
        self.assertIn("authority=NONE",p.stdout)
    def test_14_modular_pure_rule_real_score_60(self):
        p=engine("module-graph-run",PKG/"src","app.main",60)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertIn("modules=2",p.stdout)
        self.assertIn("result=INT(1)",p.stdout)
        self.assertIn("runtime-fs=NONE",p.stdout)
    def test_15_modular_project_real_deterministic_build(self):
        with tempfile.TemporaryDirectory(prefix="nordoi-app001-") as tmp:
            d=Path(tmp)/"pure_project"
            shutil.copytree(PKG,d)
            a=engine("build",d)
            self.assertEqual(a.returncode,0,a.stderr+a.stdout)
            out=d/"build"/"app001-quality-rules-0.1.0.npkg"
            self.assertTrue(out.is_file())
            self.assertTrue((d/"NORDOI.lock").is_file())
            original=out.read_bytes()
            b=engine("build",d,"--locked")
            self.assertEqual(b.returncode,0,b.stderr+b.stdout)
            self.assertEqual(out.read_bytes(),original)
            self.assertIn("reproducible=true",b.stdout)
            c=engine("package-info",out)
            self.assertEqual(c.returncode,0,c.stderr+c.stdout)
    def test_16_pure_package_and_P27_are_distinct_engines(self):
        # Verify actual source is honestly scoped: P2.7 is an entry with
        # explicit FileWrite, pure project has a 2-module imported program.
        self.assertIn("effect FileWrite;",SOURCE.read_text())
        self.assertNotIn("import app.rules;",SOURCE.read_text())
        self.assertIn("import app.rules;",(PKG/"src/app/main.noi").read_text())
        self.assertIn("fn approved(",(PKG/"src/app/rules.noi").read_text())

if __name__=="__main__":unittest.main()
