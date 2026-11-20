"""APP-002 real-data native CLI composition and fail-closed security tests.

Every test invokes the compiled Rust NORDOI binary. Negative tests modify
disposable source COPIES, not the repository nor historical evidence.
No mock/simulator for execution and no human testers.
"""
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE))
import run_app002

ENGINE=Path(os.environ.get("NORDOI_APP002_EXE",
  str(ROOT/"target"/"debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))
MODULE=HERE/"rules"/"src"
APPROVED=HERE/"approved.noi"
REJECTED=HERE/"rejected.noi"

def cli(score,folder=None,root=MODULE,approved=APPROVED,rejected=REJECTED):
    args=[str(ENGINE.resolve(strict=True)),"module-bundle-write",
          str(root),"app.main",str(approved),str(rejected),str(score)]
    if folder is not None:args+=["--grant-output-dir",str(folder)]
    return subprocess.run(args,cwd=ROOT,capture_output=True,text=True,timeout=120)

def copy_sources(temp):
    dest=Path(temp)/"sources"
    shutil.copytree(HERE/"rules",dest/"rules")
    shutil.copy2(APPROVED,dest/"approved.noi")
    shutil.copy2(REJECTED,dest/"rejected.noi")
    return dest/"rules"/"src",dest/"approved.noi",dest/"rejected.noi"

class NORDOIAPP002NativeRealTests(unittest.TestCase):
    def test_01_actual_binary_exists(self):
        self.assertTrue(ENGINE.is_file(),ENGINE)
    def test_02_native_help_has_new_command(self):
        p=subprocess.run([str(ENGINE),"--help"],capture_output=True,text=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertIn("nordoi module-bundle-write",p.stdout)
    def test_03_genuine_imported_rule_score73_selects_approved(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn('selected="approved"',p.stdout)
            self.assertIn("rule-result=INT(1)",p.stdout)
            self.assertIn("modules=2",p.stdout)
            for file,want in run_app002.expected(73).items():
                self.assertEqual((Path(t)/"quality-decision"/file).read_bytes(),want.encode())
    def test_04_genuine_imported_rule_score59_selects_rejected(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(59,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn('selected="rejected"',p.stdout)
            self.assertIn("rule-result=INT(0)",p.stdout)
            for file,want in run_app002.expected(59).items():
                self.assertEqual((Path(t)/"quality-decision"/file).read_bytes(),want.encode())
    def test_05_exact_threshold_score60_approved(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(60,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn('selected="approved"',p.stdout)
    def test_06_actual_zero_score_rejected(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(0,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn('selected="rejected"',p.stdout)
    def test_07_actual_hundred_score_approved(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(100,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn('selected="approved"',p.stdout)
    def test_08_no_explicit_grant_is_denied_and_no_files(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,None)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("explicit --grant-output-dir required",p.stderr)
            self.assertEqual(list(Path(t).iterdir()),[])
    def test_09_reuse_bundle_refuses_overwrite_and_preserves_old_bytes(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            first=cli(73,t)
            self.assertEqual(first.returncode,0,first.stderr)
            dst=Path(t)/"quality-decision"
            original={p.name:p.read_bytes() for p in dst.iterdir()}
            again=cli(59,t)
            self.assertNotEqual(again.returncode,0)
            self.assertEqual(original,{p.name:p.read_bytes() for p in dst.iterdir()})
    def test_10_exact_target_capability_set_same_on_both_arms(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,t)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertIn("grant-count=2",p.stdout)
            self.assertIn('FileWrite("quality-decision/decision.txt")',p.stderr)
            self.assertIn('FileWrite("quality-decision/report.txt")',p.stderr)
    def test_11_real_no_ambient_authority_receipt(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,t)
            self.assertEqual(p.returncode,0,p.stderr)
            for part in ("ambient-authority=NONE","authority=EXPLICIT",
                         "bridge=V13_RULE_SELECTS_P27_BRANCH",
                         "status=COMMITTED","crash-durability=UNCLAIMED"):
                self.assertIn(part,p.stdout)
            for part in ("publication=DIRECTORY-RENAME",
                         "partial-final-state=FORBIDDEN",
                         "overwrite=DENIED"):
                self.assertIn(part,p.stderr)
    def test_12_real_rule_and_bundle_receipt_both_present(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,t)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertRegex(p.stdout,r"rule-receipt-hex=[0-9a-f]{40,}")
            self.assertRegex(p.stdout,r"bundle-receipt-sha256=[0-9a-f]{64}")
            self.assertRegex(p.stderr,r"receipt-sha256=[0-9a-f]{64}")
    def test_13_rule_change_to_threshold90_changes_actual_selected_branch(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            rules=root/"app"/"rules.noi"
            content=rules.read_text()
            self.assertIn("score >= 60",content)
            rules.write_text(content.replace("score >= 60","score >= 90"))
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("rule-result=INT(0)",p.stdout)
            self.assertIn('selected="rejected"',p.stdout)
            self.assertIn("approved=false",(output/"quality-decision"/"decision.txt").read_text())
    def test_14_rule_result_two_refused_not_treated_as_truthy(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            rules=root/"app"/"rules.noi"
            rules.write_text("module app.rules;\nfn approved(score) returns 2;\n")
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("must be INT(0) or INT(1)",p.stderr)
            self.assertFalse((output/"quality-decision").exists())
    def test_15_rule_boolean_result_denied_without_publication(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            rules=root/"app"/"rules.noi"
            rules.write_text("module app.rules;\nfn approved(score) returns score >= 60;\n")
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((output/"quality-decision").exists())
    def test_16_missing_imported_module_fails_closed(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            (root/"app"/"rules.noi").unlink()
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((output/"quality-decision").exists())
    def test_17_different_bundle_names_fail_before_execution(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            no.write_text(no.read_text().replace('writes "quality-decision"',
                                                'writes "different-bundle"'))
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("identical bundle name",p.stderr)
            self.assertEqual(list(output.iterdir()),[])
    def test_18_different_file_target_sets_rejected(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            no.write_text(no.read_text().replace('"report.txt" emits','"extra.txt" emits'))
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("exact FileWrite targets",p.stderr)
            self.assertEqual(list(output.iterdir()),[])
    def test_19_invalid_inactive_output_arm_rejected_before_rule_write(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            root,yes,no=copy_sources(t)
            no.write_text("invalid NORDOI source!!\n")
            output=Path(t)/"out"
            output.mkdir()
            p=cli(73,output,root,yes,no)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((output/"quality-decision").exists())
    def test_20_duplicate_output_arm_paths_not_accepted(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli(73,t,MODULE,APPROVED,APPROVED)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("must be different",p.stderr)
            self.assertEqual(list(Path(t).iterdir()),[])
    def test_21_invalid_score_cli_not_parsed_as_u32(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=cli("bad-number",t)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("unsigned 32-bit",p.stderr)
            self.assertEqual(list(Path(t).iterdir()),[])
    def test_22_actual_app_runner_73_writes_two_correct_files(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=subprocess.run([sys.executable,str(HERE/"run_app002.py"),
                 "--score","73","--engine",str(ENGINE),
                 "--output-parent",t],cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("NORDOI_APP002=PASS",p.stdout)
            self.assertIn("SELECTED_BRANCH=approved",p.stdout)
            children=list(Path(t).glob("NORDOI_APP002_*/quality-decision"))
            self.assertEqual(len(children),1)
            for name,value in run_app002.expected(73).items():
                self.assertEqual((children[0]/name).read_text(),value)
    def test_23_actual_app_runner_59_selects_denied_template(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            p=subprocess.run([sys.executable,str(HERE/"run_app002.py"),
                 "--score","59","--engine",str(ENGINE),
                 "--output-parent",t],cwd=ROOT,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("SELECTED_BRANCH=rejected",p.stdout)
    def test_24_user_score_outside_bounded_range_denied_before_output(self):
        with tempfile.TemporaryDirectory(prefix="na2_") as t:
            for val in ["101","-1"]:
                p=subprocess.run([sys.executable,str(HERE/"run_app002.py"),
                    "--score",val,"--engine",str(ENGINE),
                    "--output-parent",t],cwd=ROOT,
                    capture_output=True,text=True,timeout=120)
                self.assertNotEqual(p.returncode,0)
                self.assertIn("INVALID_SCORE",p.stderr)
            self.assertEqual(list(Path(t).iterdir()),[])
    def test_25_original_APP001_source_remains_unchanged_by_new_branch(self):
        f=ROOT/"examples/apps/app001_quality_gate/quality_gate.noi"
        self.assertIn("passing_score = 60",f.read_text())
        self.assertIn("entry main writes",f.read_text())
    def test_26_app002_requires_two_authentic_nordoi_profile_executions(self):
        p=(ROOT/"src/bin/nordoi/app002_bridge.rs").read_text()
        self.assertIn("execute_module_graph_v13(",p)
        self.assertIn("execute_atomic_bundle_p27(",p)
        self.assertNotIn("std::process::Command",p)

if __name__=="__main__":
    unittest.main()
