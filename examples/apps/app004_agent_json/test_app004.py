"""APP-004 native machine protocol tests — real NORDOI, no mocks/simulations."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SOURCE=REPO/"examples/apps/app003_integrated_quality/src"
sys.path.insert(0,str(HERE))
import run_app004

EXE=Path(os.environ.get("NORDOI_APP004_EXE",
    str(REPO/"target/debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))

def native(score,dir=None,root=SOURCE,module="app.main",grant_arg="--grant-output-dir"):
    cmd=[str(EXE.resolve(strict=True)),"agent-bundle-json",str(root),
         module,str(score)]
    if dir is not None:cmd+=[grant_arg,str(dir)]
    p=subprocess.run(cmd,cwd=REPO,capture_output=True,text=True,timeout=120)
    obj=run_app004.exact_native_response(p.stdout)
    return p,obj

def copies(tmp):
    source=Path(tmp)/"src"
    shutil.copytree(SOURCE,source)
    out=Path(tmp)/"output"
    out.mkdir()
    return source,out

def change(path,old,new):
    # Explicit UTF-8 on every platform: Windows locale encoding is not NOI.
    text=path.read_text(encoding="utf-8")
    if old not in text:raise AssertionError("fixture anchor missing")
    path.write_text(text.replace(old,new,1),encoding="utf-8")

class APP004NativeAgentJson(unittest.TestCase):
    def test_01_real_nordoi_binary_exists(self):
        self.assertTrue(EXE.is_file(),EXE)
    def test_02_help_exposes_machine_protocol(self):
        p=subprocess.run([str(EXE),"--help"],capture_output=True,text=True,timeout=30)
        self.assertEqual(p.returncode,0,p.stderr)
        self.assertIn("agent-bundle-json",p.stdout)

    def score(self,num,result):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,data=native(num,d)
            self.assertEqual(p.returncode,0,p.stderr+p.stdout)
            self.assertEqual(data["schema"],run_app004.SCHEMA)
            self.assertEqual(data["status"],"COMMITTED")
            self.assertEqual(data["key_code"],num)
            self.assertEqual(data["decision"],result)
            self.assertEqual(data["selected"],"approved" if result else "rejected")
            run_app004.verify_actual_files(Path(d),num,data,p.stderr)
            return data
    def test_03_real_approved_score_73(self):
        self.score(73,1)
    def test_04_real_rejected_score_59(self):
        self.score(59,0)
    def test_05_real_threshold_score_60(self):
        self.score(60,1)
    def test_06_real_boundary_zero(self):
        self.score(0,0)
    def test_07_real_boundary_hundred(self):
        self.score(100,1)

    def test_08_success_native_stdout_one_json_line_not_logs(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native(73,d)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertEqual(p.stdout.count("\n"),1)
            self.assertNotIn("integrated-bundle-run entry-module",p.stdout)
            self.assertNotIn("bundle-write module",p.stdout)
            self.assertIn("bundle-write module",p.stderr)
            self.assertEqual(x["status"],"COMMITTED")

    def test_09_json_result_is_byte_identical_across_fresh_output_parents(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as a:
            with tempfile.TemporaryDirectory(prefix="app004_real_") as b:
                first,x=native(73,a)
                second,y=native(73,b)
                self.assertEqual(first.returncode,0,first.stderr)
                self.assertEqual(second.returncode,0,second.stderr)
                self.assertEqual(first.stdout,second.stdout)
                self.assertEqual(x,y)
                self.assertNotIn(a,first.stdout)
                self.assertNotIn(b,second.stdout)

    def test_10_missing_exact_grant_returns_JSON_and_denies(self):
        p,x=native(73,None)
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(x["status"],"ERROR")
        self.assertEqual(x["error_code"],"MISSING_EXPLICIT_GRANT")
        self.assertEqual(x["publication"],"UNKNOWN")

    def test_11_bad_keycode_is_JSON_usage_failure(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native("abc",d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["error_code"],"INVALID_KEY_CODE")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_12_bad_grant_switch_fails_as_JSON_no_files(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native(73,d,grant_arg="--dangerous")
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["error_code"],"INVALID_GRANT_SYNTAX")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_13_extra_argument_denied_as_JSON(self):
        p=subprocess.run([str(EXE),"agent-bundle-json",str(SOURCE),
            "app.main","73","--grant-output-dir","any","extra"],cwd=REPO,
            capture_output=True,text=True,timeout=120)
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(json.loads(p.stdout)["error_code"],"BAD_ARGUMENT_COUNT")

    def test_14_missing_module_fails_in_JSON_without_publication(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native(73,d,module="app.unknown")
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_15_tampered_import_fails_closed_and_JSON(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            root,out=copies(d)
            (root/"app"/"rules.noi").unlink()
            p,x=native(73,out,root=root)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(out.iterdir()),[])

    def test_16_invalid_rule_value_fails_before_publication(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            root,out=copies(d)
            (root/"app"/"rules.noi").write_text(
                "module app.rules;\nfn approved(score) returns 2;\n")
            p,x=native(73,out,root=root)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(out.iterdir()),[])

    def test_17_mutating_imported_threshold_changes_native_json_decision(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            root,out=copies(d)
            change(root/"app"/"rules.noi","score >= 60","score >= 90")
            p,x=native(73,out,root=root)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertEqual(x["decision"],0)
            self.assertEqual(x["selected"],"rejected")
            self.assertIn("approved=false",(out/"quality-decision"/"decision.txt").read_text())

    def test_18_divergent_inactive_arm_denied_without_files(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            root,out=copies(d)
            change(root/"app"/"main.noi",
               '    rejected {\n        "decision.txt"',
               '    rejected {\n        "unexpected.txt"')
            p,x=native(73,out,root=root)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(out.iterdir()),[])

    def test_19_second_publication_denies_overwrite_as_JSON(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,_=native(73,d)
            self.assertEqual(p.returncode,0,p.stderr)
            bundle=Path(d)/"quality-decision"
            before={x.name:x.read_bytes() for x in bundle.iterdir()}
            denied,x=native(59,d)
            self.assertNotEqual(denied.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(before,{x.name:x.read_bytes() for x in bundle.iterdir()})

    def test_20_unicode_and_curly_brace_content_escaped_by_native_json(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            root,out=copies(d)
            # Compose the Unicode test marker from code points so Windows
            # runner log/codepage handling cannot reinterpret a source literal.
            marker=chr(0x00e9)+"preuve } machine"
            change(root/"app"/"main.noi",
                '"module-decision=approved, input="',
                '"'+marker+', input="')
            p,x=native(73,out,root=root)
            self.assertEqual(p.returncode,0,p.stdout+p.stderr)
            self.assertIn(marker,x["files"][1]["content"])
            run_app004.verify_actual_files(out,73,x,p.stderr)

    def test_21_proof_hashes_are_exact_64_lowercase_hex(self):
        x=self.score(73,1)
        for f in x["files"]:
            self.assertRegex(f["proof_sha256"],r"^[0-9a-f]{64}$")
        self.assertRegex(x["bundle_receipt_sha256"],r"^[0-9a-f]{64}$")

    def test_22_no_invented_crash_durability_or_ambient_authority(self):
        x=self.score(73,1)
        self.assertEqual(x["crash_durability"],"UNCLAIMED")
        self.assertEqual(x["partial_final_state"],"FORBIDDEN")
        self.assertEqual(x["authority"]["ambient"],"NONE")
        self.assertEqual(x["authority"]["overwrite"],"DENIED")

    def test_23_native_file_contents_are_decoded_json_and_equal_REAL_files(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native(73,d)
            self.assertEqual(p.returncode,0,p.stderr)
            for f in x["files"]:
                self.assertEqual((Path(d)/"quality-decision"/f["name"]).read_bytes(),
                                 f["content"].encode("utf-8"))
                self.assertEqual(f["bytes"],len(f["content"].encode("utf-8")))

    def test_24_client_reemits_genuine_native_json_for_score73(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p=subprocess.run([sys.executable,str(HERE/"run_app004.py"),
                "--score","73","--engine",str(EXE),"--output-parent",d],
                cwd=REPO,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stderr)
            x=run_app004.exact_native_response(p.stdout)
            self.assertEqual(x["decision"],1)
            self.assertEqual(len(list(Path(d).glob(
                "NORDOI_APP004_*/quality-decision/report.txt"))),1)

    def test_25_client_reemits_genuine_native_json_for_score59(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p=subprocess.run([sys.executable,str(HERE/"run_app004.py"),
                "--score","59","--engine",str(EXE),"--output-parent",d],
                cwd=REPO,capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertEqual(run_app004.exact_native_response(p.stdout)["selected"],"rejected")

    def test_26_client_refuses_out_of_bounds_scores_before_files(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            for num in ("-1","101"):
                p=subprocess.run([sys.executable,str(HERE/"run_app004.py"),
                    "--score",num,"--engine",str(EXE),"--output-parent",d],
                    cwd=REPO,capture_output=True,text=True,timeout=120)
                self.assertNotEqual(p.returncode,0)
                self.assertEqual(run_app004.exact_native_response(p.stdout)["status"],"ERROR")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_27_client_uses_no_python_branch_or_receipt_forgery(self):
        s=(HERE/"run_app004.py").read_text()
        self.assertNotIn("score >= 60",s)
        self.assertNotIn("subprocess.Popen",s)
        self.assertIn("agent-bundle-json",s)
        self.assertIn("verify_actual_files",s)

    def test_28_app003_original_cli_keeps_legacy_plain_text(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p=subprocess.run([str(EXE),"integrated-bundle-run",str(SOURCE),
              "app.main","73","--grant-output-dir",d],cwd=REPO,
              capture_output=True,text=True,timeout=120)
            self.assertEqual(p.returncode,0,p.stderr)
            self.assertIn("integrated-bundle-run entry-module=",p.stdout)
            self.assertIn("status=COMMITTED",p.stdout)

    def test_29_client_rejects_fabricated_local_json_and_mismatched_bytes(self):
        with tempfile.TemporaryDirectory(prefix="app004_real_") as d:
            p,x=native(73,d)
            self.assertEqual(p.returncode,0,p.stderr)
            x["files"][0]["content"]="not what is really on disk"
            with self.assertRaisesRegex(ValueError,"MISMATCH"):
                run_app004.verify_actual_files(Path(d),73,x,p.stderr)

    def test_30_agent_protocol_is_versioned_and_does_not_require_ai_in_runtime(self):
        self.assertEqual(run_app004.SCHEMA,"NORDOI_AGENT_BUNDLE_RESULT_V1")
        text=(REPO/"src/bin/nordoi/app003.rs").read_text()
        self.assertIn("execute_module_graph_v13(",text)
        self.assertIn("execute_atomic_bundle_p27(",text)
        self.assertIn("json_escape_cli(",text)
        self.assertNotIn("std::process::Command",text)

if __name__=="__main__":
    unittest.main()
