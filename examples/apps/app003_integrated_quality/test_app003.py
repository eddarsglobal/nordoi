"""APP-003 actual source-to-native-engine verification; no mock/simulator/human tester.

The new compiler reads one integrated NOI entry source plus imported pure
modules. All positive and negative tests run the same compiled Rust engine.
Mutations happen only in isolated temporary COPIES of the sources.
"""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SRC=HERE/"src"
MAIN=SRC/"app"/"main.noi"
RULES=SRC/"app"/"rules.noi"
sys.path.insert(0,str(HERE))
import run_app003

ENGINE=Path(os.getenv("NORDOI_APP003_EXE",
  str(REPO/"target"/"debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))

def actual(score, output=None, source_root=SRC, module="app.main"):
    args=[str(ENGINE.resolve(strict=True)),"integrated-bundle-run",
          str(source_root),module,str(score)]
    if output is not None:
        args+=["--grant-output-dir",str(output)]
    return subprocess.run(args,cwd=REPO,capture_output=True,text=True,timeout=120)

def isolated(temp):
    root=Path(temp)/"sources"
    shutil.copytree(SRC,root)
    out=Path(temp)/"out"
    out.mkdir()
    return root, out

def edit(path, before, after):
    current=path.read_text()
    if before not in current:
        raise AssertionError("fixture anchor missing "+before)
    path.write_text(current.replace(before,after,1))

class RealAPP003IntegratedNoi(unittest.TestCase):
    def test_01_actual_nordoi_executable_exists(self):
        self.assertTrue(ENGINE.is_file(),ENGINE)

    def test_02_new_native_cli_usage_exposed(self):
        p=subprocess.run([str(ENGINE),"--help"],capture_output=True,text=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertIn("nordoi integrated-bundle-run",p.stdout)

    def check_real_score(self,score,approve):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=actual(score,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("source-profile=APP003_SINGLE_NOI_IMPORTS_AND_EFFECTS",p.stdout)
            self.assertIn("modules=2",p.stdout)
            self.assertIn(f"original-key-code={score}",p.stdout)
            self.assertIn(f"rule-result=INT({1 if approve else 0})",p.stdout)
            self.assertIn(f'selected="{"approved" if approve else "rejected"}"',p.stdout)
            for name,want in run_app003.expected(score).items():
                self.assertEqual((Path(t)/"quality-decision"/name).read_bytes(),
                                 want.encode("utf-8"))
            self.assertEqual(
                {x.name for x in (Path(t)/"quality-decision").iterdir()},
                {"decision.txt","report.txt"}
            )

    def test_03_score73_approved_from_imported_rule(self):
        self.check_real_score(73,True)
    def test_04_score59_rejected_from_imported_rule(self):
        self.check_real_score(59,False)
    def test_05_score60_exact_threshold_approved(self):
        self.check_real_score(60,True)
    def test_06_score0_rejected(self):
        self.check_real_score(0,False)
    def test_07_score100_approved(self):
        self.check_real_score(100,True)

    def test_08_missing_grant_denied_before_program_execution(self):
        p=actual(73)
        self.assertNotEqual(p.returncode,0)
        self.assertIn("explicit --grant-output-dir required",p.stderr)

    def test_09_same_bundle_second_publication_denied_no_overwrite(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            a=actual(73,t)
            self.assertEqual(a.returncode,0,a.stderr)
            bundle=Path(t)/"quality-decision"
            originals={x.name:x.read_bytes() for x in bundle.iterdir()}
            b=actual(59,t)
            self.assertNotEqual(b.returncode,0)
            self.assertEqual(originals,
                             {x.name:x.read_bytes() for x in bundle.iterdir()})

    def test_10_rule_change_threshold90_changes_actual_branch_without_python(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            rules=root/"app"/"rules.noi"
            edit(rules,"score >= 60","score >= 90")
            p=actual(73,out,root)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("rule-result=INT(0)",p.stdout)
            self.assertIn('selected="rejected"',p.stdout)
            self.assertIn("approved=false",
                          (out/"quality-decision"/"decision.txt").read_text())

    def test_11_non_boolean_integer_rule_two_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            (root/"app"/"rules.noi").write_text(
                "module app.rules;\nfn approved(score) returns 2;\n")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("exactly INT(0) or INT(1)",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_12_boolean_rule_value_denied_not_truthy(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            (root/"app"/"rules.noi").write_text(
                "module app.rules;\nfn approved(score) returns score >= 60;\n")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_13_missing_import_module_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            (root/"app"/"rules.noi").unlink()
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_14_rule_module_identity_mismatch_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"rules.noi","module app.rules;",
                 "module app.fake;")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_15_missing_rule_function_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","rules.approved(key_code)",
                 "rules.missing(key_code)")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_16_unknown_import_alias_denied_before_runtime(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","rules.approved(key_code)",
                 "phantom.approved(key_code)")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("unknown import alias",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_17_different_output_targets_in_inactive_arm_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            main=root/"app"/"main.noi"
            txt=main.read_text()
            old='    rejected {\n        "decision.txt"'
            # exact first target only in inactive branch
            self.assertIn(old,txt)
            main.write_text(txt.replace(old,
                 '    rejected {\n        "other.txt"',1))
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("identical bundle name and exact FileWrite targets",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_18_malformed_inactive_arm_denied_before_publication(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi",
                 '"module-decision=rejected, input="',
                 '"module-decision=rejected, input=" + +')
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_19_duplicate_target_rejected_before_publication(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi",
                 '"report.txt" emits "module-decision=approved',
                 '"decision.txt" emits "module-decision=approved')
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_20_network_effect_instead_of_filewrite_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","effect FileWrite;",
                 "effect Network;")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_21_ambiguous_import_alias_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","import app.rules;",
                 "import app.rules;\nimport another.rules;")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("ambiguous import aliases",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_22_duplicate_import_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","import app.rules;",
                 "import app.rules;\nimport app.rules;")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("duplicate import",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_23_missing_import_declaration_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","import app.rules;\n","")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("requires at least one imported rule",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_24_unclosed_effectful_branch_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            main=root/"app"/"main.noi"
            txt=main.read_text()
            self.assertTrue(txt.endswith("};\n"))
            main.write_text(txt[:-3])
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_25_extra_top_level_source_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            (root/"app"/"main.noi").open("a").write(
                "entry hidden returns 1;\n")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertIn("extra source",p.stderr)
            self.assertFalse((out/"quality-decision").exists())

    def test_26_non_keycode_rule_argument_denied(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","rules.approved(key_code)",
                 "rules.approved(73)")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_27_unexpected_prelude_const_denied_not_silently_ignored(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi","input key_code;",
                 "input key_code;\nconst shadow = 99;")
            p=actual(73,out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse((out/"quality-decision").exists())

    def test_28_no_globally_effectful_imports_or_general_api_claim(self):
        source=MAIN.read_text()
        self.assertEqual(source.count("entry main"),1)
        self.assertIn("import app.rules;",source)
        self.assertIn("effect FileWrite;",source)
        self.assertIn("approved {",source)
        self.assertIn("rejected {",source)
        self.assertEqual(set(SRC.rglob("*.noi")),{MAIN,RULES})

    def test_29_genuine_composed_V13_and_P27_receipts(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=actual(73,t)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertRegex(p.stdout,r"rule-receipt-hex=[0-9a-f]{40,}")
            self.assertRegex(p.stdout,r"bundle-receipt-sha256=[0-9a-f]{64}")
            self.assertRegex(p.stderr,r"receipt-sha256=[0-9a-f]{64}")

    def test_30_real_exact_permission_only_and_no_crash_claim(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=actual(73,t)
            self.assertEqual(p.returncode,0,p.stderr)
            for x in ("ambient-authority=NONE","grant-count=2",
                      "authority=EXPLICIT","crash-durability=UNCLAIMED"):
                self.assertIn(x,p.stdout)
            for x in (
                'FileWrite("quality-decision/decision.txt")',
                'FileWrite("quality-decision/report.txt")',
                "publication=DIRECTORY-RENAME",
                "overwrite=DENIED",
                "partial-final-state=FORBIDDEN"):
                self.assertIn(x,p.stderr)

    def test_31_real_user_launcher_score73(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=subprocess.run([sys.executable,str(HERE/"run_app003.py"),
                "--score","73","--engine",str(ENGINE),"--output-parent",t],
                cwd=REPO,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("NORDOI_APP003=PASS",p.stdout)
            self.assertIn("SELECTED_BRANCH=approved",p.stdout)
            self.assertEqual(len(list(Path(t).glob(
                "NORDOI_APP003_*/quality-decision/decision.txt"))),1)

    def test_32_real_user_launcher_score59(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=subprocess.run([sys.executable,str(HERE/"run_app003.py"),
                "--score","59","--engine",str(ENGINE),"--output-parent",t],
                cwd=REPO,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("NORDOI_APP003=PASS",p.stdout)
            self.assertIn("SELECTED_BRANCH=rejected",p.stdout)

    def test_33_user_invalid_scores_no_files_created(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            for score in ["101","-1"]:
                p=subprocess.run([sys.executable,str(HERE/"run_app003.py"),
                    "--score",score,"--engine",str(ENGINE),"--output-parent",t],
                    cwd=REPO,capture_output=True,text=True,timeout=120)
                self.assertNotEqual(p.returncode,0)
                self.assertIn("INVALID_SCORE",p.stderr)
            self.assertEqual(list(Path(t).iterdir()),[])

    def test_34_invalid_key_code_denied_by_native_cli(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=actual("not-a-number",t)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(list(Path(t).iterdir()),[])

    def test_35_untrusted_relative_module_path_rejected(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            p=actual(73,t,SRC,"..")
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(list(Path(t).iterdir()),[])

    def test_36_quoted_braces_do_not_escape_lexer_branch(self):
        with tempfile.TemporaryDirectory(prefix="nordoi_app003_") as t:
            root,out=isolated(t)
            edit(root/"app"/"main.noi",
                '"module-decision=approved, input="',
                '"module-decision=approved } branch, input="')
            p=actual(73,out,root)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn("approved } branch",
                (out/"quality-decision"/"report.txt").read_text())

    def test_37_historical_APP001_and_APP002_stay_unchanged(self):
        old1=(REPO/"examples/apps/app001_quality_gate/quality_gate.noi").read_text()
        old2=(REPO/"examples/apps/app002_rule_bundle/rules/src/app/main.noi").read_text()
        self.assertIn("entry main writes",old1)
        self.assertIn("entry main returns rules.approved(key_code);",old2)

    def test_38_new_compiler_has_real_V13_and_P27_calls(self):
        src=(REPO/"src/bin/nordoi/app003.rs").read_text()
        self.assertIn("execute_module_graph_v13(",src)
        self.assertIn("execute_atomic_bundle_p27(",src)
        self.assertIn("compile_atomic_bundle_plan_p27(",src)
        self.assertIn("lex(source)",src)
        self.assertNotIn("std::process::Command",src)

if __name__=="__main__":
    unittest.main()
