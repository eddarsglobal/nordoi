"""APP-023: real Rust-backed concurrent agent DAG verification on 4 hosts.

Every successful test executes genuine native NORDOI Rust P2.7 outputs; no
Python replacement of native business computation and no fake threads.
Malformed plan/authority tests fail before any FileWrite effect.
"""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2]
sys.path.insert(0,str(HERE.parent/"app009_agent_dag"))
import run_app009 as original
sys.path.insert(0,str(HERE))
import run_app023 as concurrent
SOURCE=REPO/"examples/apps/app003_integrated_quality/src"
ENGINE=Path(os.environ.get("NORDOI_APP023_EXE",
    str(REPO/"target/debug"/("nordoi.exe" if os.name=="nt" else "nordoi"))))


def node(code:int, deps=None, gate="always"):
    return {"key_code":code,"depends_on":[] if deps is None else deps,
            "gate":gate}


class NativeConcurrentDAG(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="nordoi_app023_real_")
        self.root=Path(self.temp.name)
        self.grant=self.root/"grant"
        self.grant.mkdir()
        self.plan=self.root/"plan.json"
        self.engine=ENGINE
        self.source=SOURCE
        self.engine_pin=concurrent.pinned.engine_sha256(self.engine)
        self.source_pin=concurrent.pinned.source_tree_sha256(self.source)
        self.write([node(73),node(59),node(60,[1,2],"all_committed")])

    def tearDown(self):
        self.temp.cleanup()

    def write(self,nodes):
        self.plan.write_text(json.dumps(
            {"schema":original.SCHEMA,"nodes":nodes},
            separators=(",",":"),ensure_ascii=True),encoding="utf-8")

    def run_cli(self,*,limit=2,grant=None,engine_pin=None,source_pin=None):
        p=subprocess.run(
            [sys.executable,"-B",str(HERE/"run_app023.py"),
             "--plan",str(self.plan),"--engine",str(self.engine),
             "--source-root",str(self.source),
             "--grant-output-dir",str(self.grant if grant is None else grant),
             "--expected-engine-sha256",
               self.engine_pin if engine_pin is None else engine_pin,
             "--expected-source-sha256",
               self.source_pin if source_pin is None else source_pin,
             "--max-in-flight",str(limit)],cwd=REPO,capture_output=True,
             timeout=180)
        try:
            records=[json.loads(x) for x in p.stdout.decode("utf-8").splitlines()]
        except (UnicodeError,ValueError) as exc:
            self.fail(f"bad JSON {p.stdout!r}, {p.stderr!r}: {exc}")
        return p,records

    def good(self,statuses,**kwargs):
        p,events=self.run_cli(**kwargs)
        self.assertEqual(p.returncode,0,(events,p.stderr))
        self.assertEqual([x["status"] for x in events],statuses)
        self.assertEqual([x["step"] for x in events],
                         list(range(1,len(events)+1)))
        self.assertTrue(all(x["schema"]==concurrent.EVENT for x in events))
        return events

    def deny(self,code):
        p,events=self.run_cli()
        self.assertEqual(p.returncode,2,(events,p.stderr))
        self.assertEqual(len(events),1)
        self.assertEqual(events[0]["status"],"REJECTED")
        self.assertEqual(events[0]["error_code"],code)
        self.assertEqual(events[0]["publication"],"NOT_STARTED")
        self.assertEqual(list(self.grant.iterdir()),[])

    def test_01_three_node_parallel_roots_and_native_join(self):
        events=self.good(["COMMITTED"]*3)
        self.assertEqual([x["decision"] for x in events],[1,0,1])
        self.assertEqual([x["job"] for x in events],
                         ["job-0001","job-0002","job-0003"])

    def test_02_one_worker_and_two_workers_match_authentic_receipts(self):
        first=self.good(["COMMITTED"]*3,limit=1)
        other=self.root/"parallel"
        other.mkdir()
        process,second=self.run_cli(limit=2,grant=other)
        self.assertEqual(process.returncode,0,(second,process.stderr))
        self.assertEqual(first,second)
        self.assertEqual(sorted(x.name for x in other.iterdir()),
                         sorted(x.name for x in self.grant.iterdir()))

    def test_03_four_independent_roots_real_rust(self):
        self.write([node(73),node(59),node(80),node(60)])
        rows=self.good(["COMMITTED"]*4)
        self.assertEqual([x["decision"] for x in rows],[1,0,1,1])

    def test_04_two_join_siblings_ready_in_parallel(self):
        self.write([node(73),node(80),node(59,[1,2],"all_approved"),
                    node(60,[1,2],"all_committed")])
        rows=self.good(["COMMITTED"]*4)
        self.assertEqual([x["decision"] for x in rows],[1,1,0,1])

    def test_05_all_approved_gate_skips_if_parent_rejected(self):
        self.write([node(73),node(59),node(60,[1,2],"all_approved")])
        self.good(["COMMITTED","COMMITTED","SKIPPED"])
        self.assertFalse((self.grant/"job-0003").exists())

    def test_06_all_rejected_gate_executes(self):
        self.write([node(59),node(0),node(73,[1,2],"all_rejected")])
        self.good(["COMMITTED"]*3)

    def test_07_any_approved_gate_executes(self):
        self.write([node(59),node(73),node(60,[1,2],"any_approved")])
        self.good(["COMMITTED"]*3)

    def test_08_any_rejected_gate_executes(self):
        self.write([node(73),node(59),node(60,[1,2],"any_rejected")])
        self.good(["COMMITTED"]*3)

    def test_09_gate_skip_does_not_count_as_committed(self):
        self.write([node(73),node(59),node(60,[1,2],"all_approved"),
                    node(73,[3],"all_committed")])
        self.good(["COMMITTED","COMMITTED","SKIPPED","SKIPPED"])
        self.assertFalse((self.grant/"job-0004").exists())

    def test_10_all_approved_with_two_rejects_skips(self):
        self.write([node(59),node(0),node(73,[1,2],"all_approved")])
        self.good(["COMMITTED","COMMITTED","SKIPPED"])

    def test_11_eight_node_bound_parallel_wave(self):
        self.write([node(73),node(59),node(80),node(60),
                    node(73,[1,2],"all_committed"),
                    node(60,[3,4],"all_committed"),
                    node(59,[5,6],"any_approved"),
                    node(73,[5,6],"all_committed")])
        rows=self.good(["COMMITTED"]*8)
        self.assertEqual(len(rows),8)

    def test_12_same_key_code_distinct_dirs_no_overwrite(self):
        self.write([node(73),node(73),node(73,[1,2],"all_approved")])
        rows=self.good(["COMMITTED"]*3)
        self.assertEqual(len({x["job"] for x in rows}),3)

    def test_13_executed_native_receipts_verify_true_files(self):
        rows=self.good(["COMMITTED"]*3)
        for x in rows:
            self.assertEqual(
                concurrent.audit.verify_native_record(
                    x["native_receipt"],x["step"],self.grant),None)
            self.assertTrue((self.grant/x["job"]).is_dir())

    def test_14_native_event_order_and_content_host_independent(self):
        rows=self.good(["COMMITTED"]*3)
        self.assertNotIn(str(self.root),json.dumps(rows))
        self.assertNotIn(str(self.source),json.dumps(rows))
        self.assertEqual(len({x["plan_sha256"] for x in rows}),1)

    def test_15_forward_dependency_denied_pre_effect(self):
        self.write([node(73,[2],"all_committed"),node(59)])
        self.deny("DAG_REFERENCE_NOT_PRIOR")

    def test_16_duplicate_dependency_denied_pre_effect(self):
        self.write([node(73),node(59,[1,1],"all_committed")])
        self.deny("DAG_REFERENCES_NOT_STRICTLY_ASCENDING")

    def test_17_agent_json_cannot_select_output_grant(self):
        raw={"schema":original.SCHEMA,
             "nodes":[dict(node(73),grant_output_dir="/tmp/agent-owned")]}
        self.plan.write_text(json.dumps(raw),encoding="utf-8")
        self.deny("DAG_NODE_FIELDS_INVALID")

    def test_18_agent_json_cannot_select_engine(self):
        raw={"schema":original.SCHEMA,
             "nodes":[dict(node(73),engine="/tmp/evil")]}
        self.plan.write_text(json.dumps(raw),encoding="utf-8")
        self.deny("DAG_NODE_FIELDS_INVALID")

    def test_19_unauthorized_nonempty_output_root_denied(self):
        (self.grant/"existing.txt").write_text("protect")
        p,events=self.run_cli()
        self.assertEqual(p.returncode,2)
        self.assertEqual(events[0]["error_code"],"EXPLICIT_GRANT_NOT_EMPTY")
        self.assertEqual((self.grant/"existing.txt").read_text(),"protect")

    def test_20_engine_pin_conflict_denied_pre_effect(self):
        p,events=self.run_cli(engine_pin="0"*64)
        self.assertEqual(p.returncode,2)
        self.assertEqual(events[0]["error_code"],"ENGINE_PIN_MISMATCH")
        self.assertFalse(list(self.grant.iterdir()))

    def test_21_source_pin_conflict_denied_pre_effect(self):
        p,events=self.run_cli(source_pin="f"*64)
        self.assertEqual(p.returncode,2)
        self.assertEqual(events[0]["error_code"],"SOURCE_PIN_MISMATCH")
        self.assertFalse(list(self.grant.iterdir()))

    def test_22_invalid_json_duplicate_keys_refused(self):
        self.plan.write_text(
            '{"schema":"NORDOI_AGENT_DAG_V1","nodes":[],"nodes":[]}',
            encoding="utf-8")
        p,events=self.run_cli()
        self.assertEqual(p.returncode,2)
        self.assertEqual(len(events),1)
        self.assertFalse(list(self.grant.iterdir()))

    def test_23_no_source_or_engine_mutation(self):
        before_source=self.source_pin
        before_engine=self.engine_pin
        self.good(["COMMITTED"]*3)
        self.assertEqual(concurrent.pinned.engine_sha256(self.engine),before_engine)
        self.assertEqual(concurrent.pinned.source_tree_sha256(self.source),before_source)

    def test_24_max_two_worker_limit_checked_before_launch(self):
        # Direct Python public entry rejects bypass of CLI validation.
        with self.assertRaises(concurrent.pinned.Denied) as ctx:
            concurrent.run(self.plan,self.engine,self.source,self.grant,
                           self.engine_pin,self.source_pin,3)
        self.assertEqual(ctx.exception.code,"CONCURRENT_IN_FLIGHT_LIMIT_INVALID")
        self.assertFalse(list(self.grant.iterdir()))


if __name__=="__main__":
    unittest.main()
