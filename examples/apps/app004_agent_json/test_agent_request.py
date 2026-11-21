"""APP-004-R2: real native Rust machine JSON stdin to native JSON stdout tests.

Zero mock engine. Every behavioral test invokes target/debug/nordoi and
checks authentic P2.7 effects in isolated OS directories, not a simulator.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
SRC=REPO/"examples/apps/app003_integrated_quality/src"
sys.path.insert(0,str(HERE))
import run_app004

EXE=Path(os.environ.get("NORDOI_APP004_EXE",
    str(REPO/"target/debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))
SCHEMA="NORDOI_AGENT_REQUEST_V1"

def request(score=73,**more):
    return json.dumps({"schema":SCHEMA,"key_code":score,**more},
                      separators=(",",":"),ensure_ascii=False).encode("utf-8")

def call(payload,grant=None,root=SRC,module="app.main",extra=()):
    cmd=[str(EXE.resolve(strict=True)),"agent-request-json",str(root),module]
    if grant is not None:
        cmd+=["--grant-output-dir",str(grant)]
    cmd+=list(extra)
    p=subprocess.run(cmd,input=payload,cwd=REPO,capture_output=True,timeout=120)
    out=p.stdout.decode("utf-8",errors="strict")
    err=p.stderr.decode("utf-8",errors="replace")
    assert out.endswith("\n") and out.count("\n")==1,(out,err)
    result=json.loads(out)
    return p,result,out,err

def temporary_sources(folder):
    target=Path(folder)/"src"
    shutil.copytree(SRC,target)
    out=Path(folder)/"out"
    out.mkdir()
    return target,out

class RealNativeAgentRequestJson(unittest.TestCase):
    def test_01_real_nordoi_binary(self):
        self.assertTrue(EXE.is_file(),EXE)
    def test_02_command_declared_in_native_help(self):
        p=subprocess.run([str(EXE),"--help"],capture_output=True,timeout=30)
        self.assertEqual(p.returncode,0)
        self.assertIn("agent-request-json",p.stdout.decode("utf-8"))

    def verify(self,score,approved,bytes_data=None):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p,x,raw,err=call(request(score) if bytes_data is None else bytes_data,d)
            self.assertEqual(p.returncode,0,raw+err)
            self.assertEqual(x["status"],"COMMITTED")
            self.assertEqual(x["schema"],run_app004.SCHEMA)
            self.assertEqual(x["decision"],1 if approved else 0)
            self.assertEqual(x["selected"],"approved" if approved else "rejected")
            self.assertEqual(x["key_code"],score)
            run_app004.verify_actual_files(Path(d),score,x,err)
            return raw

    def test_03_real_approved_73(self):self.verify(73,True)
    def test_04_real_rejected_59(self):self.verify(59,False)
    def test_05_real_threshold_60(self):self.verify(60,True)
    def test_06_real_boundary_zero(self):self.verify(0,False)
    def test_07_real_boundary_hundred(self):self.verify(100,True)
    def test_08_whitespace_and_reverse_field_order(self):
        b=b' \r\n{\n "key_code" : 73,\t"schema":"NORDOI_AGENT_REQUEST_V1" }\n'
        self.verify(73,True,b)
    def test_09_standard_escaped_schema_key(self):
        b=br'{"sche\u006da":"NORDOI_AGENT_REQUEST_V1","key_code":73}'
        self.verify(73,True,b)
    def test_10_json_identical_to_original_native_app004_response(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as a:
            with tempfile.TemporaryDirectory(prefix="noi_request_") as b:
                p,x,raw,err=call(request(73),a)
                self.assertEqual(p.returncode,0,err)
                legacy=subprocess.run(
                    [str(EXE),"agent-bundle-json",str(SRC),"app.main","73",
                     "--grant-output-dir",b],
                    cwd=REPO,capture_output=True,timeout=120,
                )
                self.assertEqual(legacy.returncode,0,legacy.stderr)
                self.assertEqual(p.stdout,legacy.stdout)
                self.assertEqual(x,json.loads(legacy.stdout))

    def test_11_missing_grant_denied_before_parsing_json(self):
        p,x,raw,err=call(request(73),None)
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(x["error_code"],"MISSING_EXPLICIT_GRANT")
        self.assertEqual(x["publication"],"UNKNOWN")

    def test_12_no_input_fails_without_effect(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p,x,_,_=call(b"",d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_13_non_utf8_input_fails_without_effect(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p,x,_,_=call(b"\xff\xfe",d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["error_code"],"INVALID_REQUEST_UTF8")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_14_request_larger_than_4096_denied_without_effect(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p,x,_,_=call(b" "*4097,d)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["error_code"],"REQUEST_TOO_LARGE")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_15_wrong_schema_version_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V2","key_code":73}',
                  "INVALID_REQUEST_SCHEMA")
    def test_16_missing_schema_denied(self):
        self.deny(b'{"key_code":73}',"MISSING_REQUEST_FIELD")
    def test_17_missing_keycode_denied(self):
        self.deny(b'{"schema":"NORDOI_AGENT_REQUEST_V1"}',"MISSING_REQUEST_FIELD")
    def test_18_empty_json_object_denied(self):
        self.deny(b'{}',"MISSING_REQUEST_FIELD")
    def test_19_duplicate_score_field_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73,"key_code":74}',
                  "DUPLICATE_REQUEST_FIELD")
    def test_20_duplicate_schema_field_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}',
                  "DUPLICATE_REQUEST_FIELD")
    def test_21_untrusted_json_authority_injection_denied(self):
        self.deny(request(73,output_dir="/some/other/dir"),"UNKNOWN_REQUEST_FIELD")
    def test_22_untrusted_json_module_change_denied(self):
        self.deny(request(73,entry_module="attacker.main"),"UNKNOWN_REQUEST_FIELD")
    def test_23_non_integer_score_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":"73"}',
                  "INVALID_REQUEST_KEY_CODE")
    def test_24_negative_score_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":-1}',
                  "INVALID_REQUEST_KEY_CODE")
    def test_25_fraction_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73.0}',
                  "INVALID_REQUEST_JSON")
    def test_26_large_u32_overflow_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":4294967296}',
                  "INVALID_REQUEST_KEY_CODE")
    def test_27_boolean_score_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":true}',
                  "INVALID_REQUEST_KEY_CODE")
    def test_28_leading_zero_score_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":073}',
                  "INVALID_REQUEST_JSON")
    def test_29_trailing_json_document_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}{}',
                  "INVALID_REQUEST_JSON")
    def test_30_trailing_comma_denied(self):
        self.deny(br'{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73,}',
                  "INVALID_REQUEST_JSON")
    def test_31_unpaired_unicode_surrogate_denied(self):
        self.deny(br'{"schema":"\uD800","key_code":73}',"INVALID_REQUEST_JSON")
    def test_32_wrong_json_outer_type_denied(self):
        self.deny(br'[{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}]',
                  "INVALID_REQUEST_JSON")

    def deny(self,b,reason):
        with tempfile.TemporaryDirectory(prefix="noi_request_denied_") as d:
            p,x,raw,err=call(b,d)
            self.assertNotEqual(p.returncode,0,raw+err)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(x["error_code"],reason)
            self.assertEqual(x["publication"],"UNKNOWN")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_33_no_overwrite_after_success(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            first,x,_,_=call(request(73),d)
            self.assertEqual(first.returncode,0)
            target=Path(d)/"quality-decision"
            before={z.name:z.read_bytes() for z in target.iterdir()}
            second,error,_,_=call(request(59),d)
            self.assertNotEqual(second.returncode,0)
            self.assertEqual(error["status"],"ERROR")
            self.assertEqual(before,{z.name:z.read_bytes() for z in target.iterdir()})

    def test_34_changed_imported_rule_changes_native_decision(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            root,out=temporary_sources(d)
            rule=root/"app"/"rules.noi"
            txt=rule.read_text(encoding="utf-8")
            self.assertIn("score >= 60",txt)
            rule.write_text(txt.replace("score >= 60","score >= 90"),
                            encoding="utf-8")
            p,x,raw,err=call(request(73),out,root)
            self.assertEqual(p.returncode,0,raw+err)
            self.assertEqual(x["decision"],0)
            self.assertIn(b"approved=false",(out/"quality-decision"/"decision.txt").read_bytes())

    def test_35_bad_rule_no_publication(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            root,out=temporary_sources(d)
            (root/"app"/"rules.noi").write_text(
                "module app.rules;\nfn approved(score) returns 2;\n",
                encoding="utf-8")
            p,x,_,_=call(request(73),out,root)
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(x["status"],"ERROR")
            self.assertEqual(list(out.iterdir()),[])

    def test_36_invalid_source_root_no_publication(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p,x,_,_=call(request(73),d,Path(d)/"nonexistent")
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_37_adapter_approved_genuine_json(self):
        self.adapter(73,1)
    def test_38_adapter_rejected_genuine_json(self):
        self.adapter(59,0)
    def adapter(self,score,decision):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p=subprocess.run(
                [sys.executable,str(HERE/"run_agent_request.py"),"--score",str(score),
                 "--engine",str(EXE),"--output-parent",d],
                cwd=REPO,capture_output=True,timeout=120,
            )
            self.assertEqual(p.returncode,0,p.stderr.decode("utf-8",errors="replace"))
            obj=json.loads(p.stdout.decode("utf-8"))
            self.assertEqual(obj["decision"],decision)
            dirs=list(Path(d).glob("NORDOI_AGENT_REQUEST_*/quality-decision/report.txt"))
            self.assertEqual(len(dirs),1)

    def test_39_adapter_negative_score_creates_no_directories(self):
        with tempfile.TemporaryDirectory(prefix="noi_request_") as d:
            p=subprocess.run(
                [sys.executable,str(HERE/"run_agent_request.py"),"--score","-1",
                 "--engine",str(EXE),"--output-parent",d],
                cwd=REPO,capture_output=True,timeout=120,
            )
            self.assertNotEqual(p.returncode,0)
            self.assertEqual(json.loads(p.stdout)["status"],"ERROR")
            self.assertEqual(list(Path(d).iterdir()),[])

    def test_40_request_data_cannot_grant_os_authority(self):
        src=(REPO/"src/bin/nordoi/agent_request.rs").read_text(encoding="utf-8")
        self.assertIn("MAX_APP004_REQUEST_BYTES",src)
        self.assertIn("UNKNOWN_REQUEST_FIELD",src)
        self.assertIn("app003::run_app004_agent_json_cli(",src)
        self.assertNotIn("std::process::Command",src)
        self.assertNotIn("serde_json",src)

if __name__=="__main__":
    unittest.main()
