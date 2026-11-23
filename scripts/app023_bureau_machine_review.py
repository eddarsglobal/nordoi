#!/usr/bin/env python3
"""APP-023: nine executed machine Bureau review FUNCTIONS, not nine AI agents.

They audit actual source and immutable governance. PASS is code-level
research evidence, never independent external certification or permission.
"""
from __future__ import annotations
import argparse
import ast
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from scripts import roadmap_guardian as historic
from scripts import app_research_guardian as research

PARENT="1fd98f87a776ef41a2367df0857c1a075e4a5d1a"
APP="examples/apps/app023_bounded_concurrent_dag/"
GATE="governance/NORDOI_APP023_CONCURRENT_DAG_RESEARCH_V1.json"
WORKFLOW=".github/workflows/nordoi_app023_concurrent_dag.yml"
ROLES=historic.ROLE_IDS


def check(v,reason):
    if not v:
        raise ValueError(reason)


def load(path,*clauses):
    p=ROOT/path
    check(p.is_file() and not p.is_symlink(),"MISSING_OR_SYMLINKED:"+path)
    s=p.read_text("utf-8")
    check(0<len(s)<150000,"FILE_SIZE_INVALID:"+path)
    for clause in clauses:
        check(clause in s,"CRITICAL_CLAUSE_MISSING:"+path+":"+clause)
    return "SOURCE_CLAUSES_"+str(len(clauses))


def frozen(*paths):
    r=subprocess.run(["git","diff","--exit-code",PARENT,"--",*paths],
        cwd=ROOT,capture_output=True,timeout=30)
    check(r.returncode==0,"HISTORICAL_SOURCE_MUTATED")
    return "EXACT_PARENT_UNCHANGED"


def run_role(role):
    base=historic.audit(ROOT)
    addon=research.audit(ROOT)
    check(base["REVIEWS_SUBMITTED"]==0 and
          base["BUREAU_CERTIFICATION"]=="PENDING" and
          base["R6G"]=="LOCKED_BY_SEQUENCE" and
          addon["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED",
          "HISTORICAL_GOVERNANCE_IMMUTABILITY_FAILED")
    data=json.loads((ROOT/GATE).read_text("utf-8"))
    check(data["schema"]=="NORDOI_APP023_CONCURRENT_DAG_RESEARCH_V1"
          and data["base_c20_exact_sha"]==PARENT
          and data["max_native_in_flight"]==2
          and data["new_tests_per_os"]==24
          and data["old_app_regressions_per_os"]==854
          and data["total_tests_per_os"]==878
          and data["formal_bureau_certification"]=="NOT_CERTIFIED"
          and data["r6g"]=="LOCKED_BY_SEQUENCE"
          and data["external_native_admission_gates_unmet"]==12
          and data["production"]=="NOT_AUTHORIZED",
          "APP023_GOVERNANCE_CONTRACT_INVALID")
    checks=["LEGACY_R6F_AND_APP_GOVERNANCE_NOT_PROMOTED=PASS",
            "APP023_EXACT_PARENT_RESEARCH_GATE=PASS"]
    if role=="ATLAS":
        checks += [frozen("src/","Cargo.toml","Cargo.lock",
                          "examples/apps/app009_agent_dag/",
                          "examples/apps/app022_live_freshness_witness/"),
          load(APP+"run_app023.py",
               "import run_app009 as dag","dag.load_plan(plan)",
               "dag.should_execute(", "ThreadPoolExecutor(max_workers=limit",
               "all(dep in finished for dep in node[\"depends_on\"])")]
    elif role=="FERRUM":
        checks += [
          load(APP+"run_app023.py",
               "subprocess.run(","agent-request-json",
               "str(engine.resolve(strict=True))",
               "thread_name_prefix=\"nordoi-app023\"",
               "timeout=120","1<=limit<=2"),
          load(APP+"test_app023.py","ENGINE=Path(os.environ",
               "concurrent.pinned.engine_sha256"),
          frozen("src/","Cargo.toml","Cargo.lock")
        ]
    elif role=="BOUNDARY":
        checks += [
          load(APP+"run_app023.py",
               "dag.load_plan(plan)","old.validate_grant(grant,source)",
               "target=grant/f\"job-{index:04d}\"",
               "str(source.resolve(strict=True))",
               "ENGINE_PIN_MISMATCH","SOURCE_PIN_MISMATCH"),
          load(APP+"test_app023.py",
               "test_17_agent_json_cannot_select_output_grant",
               "test_18_agent_json_cannot_select_engine",
               "test_19_unauthorized_nonempty_output_root_denied")
        ]
    elif role=="CIPHER":
        checks += [
          load(APP+"run_app023.py",
               "pinned.engine_sha256(engine)==expected_engine",
               "pinned.source_tree_sha256(source)==expected_source",
               "ENGINE_CHANGED_DURING_CONCURRENT_WORKFLOW",
               "SOURCE_CHANGED_DURING_CONCURRENT_WORKFLOW"),
          load("RoadMap_APP023.md",
               "pins","NOT_CERTIFIED","PRODUCTION=NOT_AUTHORIZED")
        ]
    elif role=="LEDGER":
        checks += [
          load(APP+"run_app023.py",
               "audit.verify_native_record(native,index,grant)",
               "native_receipt=native","for i in sorted(events)",
               "NATIVE_RESULT_NOT_VERIFIED",
               'publication="UNKNOWN"'),
          load(APP+"test_app023.py","test_13_executed_native_receipts_verify_true_files",
               "test_14_native_event_order_and_content_host_independent")
        ]
    elif role=="CAPSTONE":
        checks += [
          load(APP+"run_app023.py",
               "No job is abandoned",
               "Drain every already admitted task",
               "if not drain():","ThreadPoolExecutor(max_workers=limit"),
          load("RoadMap_APP023.md",
               "transaction globale","ERROR/publication=UNKNOWN","NATIVE_ADMISSION_GATES=12_UNMET")
        ]
    elif role=="ADVERSARY":
        source=(ROOT/APP/"test_app023.py").read_text("utf-8")
        functions=[n.name for n in ast.walk(ast.parse(source))
                   if isinstance(n,ast.FunctionDef) and n.name.startswith("test_")]
        check(len(functions)==24 and len(set(functions))==24,
              "APP023_TWENTYFOUR_ADVERSARIAL_TESTS_ABSENT")
        checks += ["APP023_TWENTYFOUR_REAL_TEST_METHODS=PASS",
                   load(APP+"test_app023.py",
                        "test_15_forward_dependency_denied_pre_effect",
                        "test_16_duplicate_dependency_denied_pre_effect",
                        "test_20_engine_pin_conflict_denied_pre_effect",
                        "test_21_source_pin_conflict_denied_pre_effect",
                        "test_24_max_two_worker_limit_checked_before_launch")]
    elif role=="VERITY":
        checks += [
          load(WORKFLOW,PARENT,"cargo build --offline --bin nordoi",
               "macos-15-intel","macos-15","ubuntu-latest","windows-latest",
               "app023_bounded_concurrent_dag|test_app023.py|24",
               "app022_live_freshness_witness|test_app022.py|26",
               'test "$index" -eq 25','test "$total" -eq 878'),
          load(APP+"demo_app023.py",
               "assert len(claims)==14",
               "APP023_GENUINE_NORDOI_RUST_CONCURRENT_DAG_DEMO=PASS")
        ]
    elif role=="GUARDIAN":
        for action in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            check(historic.deny_transition(action)["TRANSITION"]=="BLOCK",
                  "FORBIDDEN_ELEVATION_ALLOWED")
        checks += ["THREE_PRIVILEGED_TRANSITIONS_BLOCKED=PASS",
                   frozen("RoadMap.md","Bureau.md",
                          "scripts/roadmap_guardian.py",
                          "scripts/app_research_guardian.py",
                          "governance/NORDOI_BUREAU_PLAN_v1.json"),
                   load("RoadMap_APP023.md",
                        "APP023_FORMAL_BUREAU_CERTIFICATION=NOT_CERTIFIED",
                        "R6G=LOCKED_BY_SEQUENCE",
                        "MERGE=NONE")]
    else:
        raise ValueError("UNKNOWN_BUREAU_ROLE")
    return checks


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--role",required=True,choices=ROLES)
    p.add_argument("--proof-dir",type=Path,required=True)
    args=p.parse_args(argv)
    head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,
          check=True,capture_output=True,text=True,timeout=30).stdout.strip()
    receipt={
        "schema":"NORDOI_APP023_MACHINE_BUREAU_SOURCE_REVIEW_V1",
        "role":args.role,"exact_head_sha":head,"c20_parent_sha":PARENT,
        "review_mode":"ACTUAL_DETERMINISTIC_MACHINE_FUNCTION",
        "independent_ai_reviewer":"NOT_PROVEN",
        "independent_formal_bureau_certification":"NOT_CERTIFIED",
        "native_admission_gates":"12_UNMET",
        "r6f":"IN_PROGRESS_UNCERTIFIED","r6g":"LOCKED_BY_SEQUENCE",
        "production":"NOT_AUTHORIZED"
    }
    try:
        receipt["checks"]=run_role(args.role)
        receipt["status"]="PASS_WITH_CONTROLS_RESEARCH_ONLY"
        result=0
    except (ValueError,historic.Blocked,OSError,KeyError,TypeError,SyntaxError) as exc:
        receipt["checks"]=[]
        receipt["status"]="BLOCK"
        receipt["error"]=str(exc)
        result=2
    args.proof_dir.mkdir(parents=True,exist_ok=True)
    (args.proof_dir/f"APP023_MACHINE_{args.role}.json").write_text(
        json.dumps(receipt,sort_keys=True,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))
    return result


if __name__=="__main__":
    raise SystemExit(main())
