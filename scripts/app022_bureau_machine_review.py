#!/usr/bin/env python3
"""APP-022: nine executed machine-review functions, not nine AI reviewers.

Every role has code-specific assertions with a pinned HEAD receipt. This
cannot certify administrative independence, actual autonomous reviewers,
remote custody, release approval, production, or R6F/R6G admission.
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
from scripts import roadmap_guardian as old
from scripts import app_research_guardian as newer

ROLES=old.ROLE_IDS
PARENT="9f18af45ff64695137e8e7c8d0874034803b521b"
DIR="examples/apps/app022_live_freshness_witness/"
GATE="governance/NORDOI_APP022_RESEARCH_GATE_V1.json"
WORKFLOW=".github/workflows/nordoi_app022_live_witness.yml"


def require(test,msg):
    if not test:
        raise ValueError(msg)


def source(name):
    path=ROOT/name
    require(path.is_file() and not path.is_symlink(),"SOURCE_MISSING_OR_LINKED:"+name)
    raw=path.read_text("utf-8")
    require(0<len(raw)<240000,"UNSAFE_SOURCE_SIZE")
    return raw


def tokens(name,*required):
    raw=source(name)
    for value in required:
        require(value in raw,"MISSING_CRITICAL_SOURCE:"+name+":"+value)
    return "REAL_SOURCE_CLAUSES_"+str(len(required))


def observed_head():
    p=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,
        capture_output=True,text=True,timeout=30,check=True)
    return p.stdout.strip()


def git_unchanged(*files):
    p=subprocess.run(["git","diff","--exit-code",PARENT,"--",*files],
                     cwd=ROOT,capture_output=True,timeout=30)
    require(p.returncode==0,"FROZEN_OR_PREVIOUS_APP_SOURCE_CHANGED")
    return "PRIOR_SOURCES_UNMODIFIED"


def run_role(role):
    legacy=old.audit(ROOT)
    addon=newer.audit(ROOT)
    require(legacy["R6G"]=="LOCKED_BY_SEQUENCE"
            and legacy["REVIEWS_SUBMITTED"]==0
            and addon["APP_BUREAU_FORMAL_CERTIFICATION"]=="NOT_CERTIFIED"
            and addon["APP_NEXT"]=="LOCKED_BY_GOVERNANCE_REVIEW",
            "HISTORICAL_GOVERNANCE_BYPASS")
    gate=json.loads(source(GATE))
    require(gate.get("schema")=="NORDOI_APP022_RESEARCH_GATE_V1"
            and gate.get("parent")==PARENT and
            gate.get("scope")=="AI_ONLY_NONPRIVILEGED_RESEARCH"
            and gate.get("status")=="CANDIDATE_PENDING_CI"
            and gate.get("formal_bureau_certification")=="NOT_CERTIFIED"
            and gate.get("native_admission_gates_remaining")==12
            and gate.get("r6g")=="LOCKED_BY_SEQUENCE"
            and gate.get("production")=="NOT_AUTHORIZED",
            "APP022_CANDIDATE_PROVENANCE_OR_DENIAL_BROKEN")
    checks=["HISTORICAL_GUARDIAN_AND_APP_EXTENSION=PASS",
            "APP022_DISTINCT_RESEARCH_GATE_SCOPE=PASS"]
    if role=="ATLAS":
        checks += [
          git_unchanged("src/","Cargo.toml","Cargo.lock","examples/apps/app021_authority_rotation/"),
          tokens(DIR+"receiver_app022.py",
                 "import receiver_app021 as previous",
                 "roster.previous.admitted_replay",
                 "portable.restore(", "portable.resume_native(",
                 "fresh=live_attestation("),
          tokens(DIR+"test_app022.py",
                 "import test_app021 as legacy","native_jobs_recomputed"),
        ]
    elif role=="FERRUM":
        checks += [
          tokens(DIR+"witness_app022.py",'("127.0.0.1",port)',
                 "ThreadingHTTPServer","def do_GET", "self.send_error(405)",
                 "self.state_file","private_key"),
          tokens(DIR+"receiver_app022.py",
                 "http.client.HTTPConnection(\"127.0.0.1\",port,timeout=4)",
                 "WITNESS_URL_LOCAL_LOOPBACK_REQUIRED"),
          git_unchanged("src/","Cargo.toml","Cargo.lock")
        ]
    elif role=="BOUNDARY":
        checks += [
          tokens(DIR+"receiver_app022.py",
                 "with anchor.locked(root):","os.urandom(32).hex()",
                 "roster.previous.admitted_replay(",
                 "fresh=live_attestation(args,root,head,again,event,key)",
                 "WITNESS_STATE_CHANGED_DURING_NATIVE_REPLAY"),
          tokens(DIR+"test_app022.py","test_11_witness_process_down_fail_closed_witness",
                 "test_23_frozen_rejects_native_effect_witness")
        ]
    elif role=="CIPHER":
        checks += [
          tokens(DIR+"witness_app022.py",
                 "nordoi-freshness@eddarsglobal",
                 "ssh-keygen","-Y","sign",
                 "nonce"),
          tokens(DIR+"receiver_app022.py",
                 "ssh-keygen","-Y","verify","signed.check_pubkey",
                 "expected_witness_key_sha256",
                 "WITNESS_SSHSIG_ED25519_VERIFICATION_FAILED"),
          tokens(DIR+"test_app022.py",
                 "test_25_wrong_witness_signing_key_denied_witness")
        ]
    elif role=="LEDGER":
        checks += [
          tokens(DIR+"witness_app022.py",
                 "state[\"counter\"]>=prior[\"counter\"]",
                 "WITNESS_IN_PROCESS_ROLLBACK_DENIED",
                 "WITNESS_SAME_COUNTER_STATE_MUTATION_DENIED"),
          tokens(DIR+"receiver_app022.py",
                 "statement[\"chain_sha256\"]==args.expected_chain_sha256",
                 "statement[\"head_sha256\"]==anchor.digest(anchor.canonical(head))",
                 "WITNESS_CURRENT_FRESHNESS_FLOOR_MISMATCH"),
          tokens(DIR+"test_app022.py",
                 "test_26_witness_cannot_prove_global_rollback_witness")
        ]
    elif role=="CAPSTONE":
        checks += [
          tokens(DIR+"receiver_app022.py",
                 '"native_admission":"RESEARCH_ONLY"',
                 '"production":"NOT_AUTHORIZED"',
                 "WITNESS_FROZEN_NATIVE_ADMISSION_DENIED"),
          tokens("RoadMap_APP022.md","NATIVE_ADMISSION_GATES=12_UNMET",
                 "PRODUCTION=NOT_AUTHORIZED","R6G=LOCKED_BY_SEQUENCE"),
          git_unchanged("RoadMap.md","Bureau.md",
                        "governance/NORDOI_BUREAU_PLAN_v1.json")
        ]
    elif role=="ADVERSARY":
        code=source(DIR+"test_app022.py")
        tree=ast.parse(code)
        methods=[n.name for n in ast.walk(tree)
                 if isinstance(n,(ast.FunctionDef,ast.AsyncFunctionDef))
                 and n.name.startswith("test_")]
        require(len(methods)==26 and len(set(methods))==26,
                "APP022_TWENTYSIX_REAL_TESTS_MISSING")
        checks += ["APP022_ADVERSARIAL_TEST_METHOD_COUNT=26",
          tokens(DIR+"test_app022.py",
                 "test_16_witness_same_counter_mutation_denied_witness",
                 "test_17_witness_process_counter_rollback_denied_witness",
                 "test_18_rotated_chain_requires_fresh_witness_witness",
                 "test_25_wrong_witness_signing_key_denied_witness",
                 "test_26_witness_cannot_prove_global_rollback_witness")
        ]
    elif role=="VERITY":
        checks += [
          tokens(WORKFLOW,
                 PARENT,"cargo build --offline --bin nordoi",
                 "macos-15-intel","macos-15","ubuntu-latest","windows-latest",
                 "app022_live_freshness_witness|test_app022.py|26",
                 "app021_authority_rotation|test_app021.py|40",
                 'test "$total" -eq 854', 'test "$index" -eq 24'),
          tokens(DIR+"demo_app022.py",
                 "APP022_LIVE_WITNESS_REAL_NATIVE_DEMO_RESEARCH_ONLY=PASS",
                 "assert len(claims)==14")
        ]
    elif role=="GUARDIAN":
        for target in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            require(old.deny_transition(target)["TRANSITION"]=="BLOCK",
                    "PRIVILEGED_TRANSITION_BYPASS")
        checks += ["THREE_PRIVILEGED_TRANSITIONS_DENIED",
          git_unchanged("scripts/roadmap_guardian.py",
                        "scripts/app_research_guardian.py",
                        "governance/NORDOI_APP_RESEARCH_ROADMAP_V1.json"),
          tokens("RoadMap_APP022.md","NOT_CERTIFIED",
                 "R6G=LOCKED_BY_SEQUENCE","antirollback")
        ]
    else:
        raise ValueError("UNSUPPORTED_BUREAU_ROLE")
    return checks


def main(argv=None):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--role",required=True,choices=ROLES)
    p.add_argument("--proof-dir",type=Path,required=True)
    args=p.parse_args(argv)
    sha=observed_head()
    receipt={
      "schema":"NORDOI_APP022_EXECUTED_MACHINE_FUNCTION_REVIEW_V1",
      "role":args.role,
      "source_sha":sha,
      "parent_sha":PARENT,
      "execution_type":"REAL_DETERMINISTIC_MACHINE_CODE_REVIEW",
      "independent_ai_agent":"NOT_PROVEN",
      "external_custodian":"NOT_PROVEN",
      "formal_bureau_certification":"NOT_CERTIFIED",
      "r6f_historical_reviews":"NINE_NOT_SUBMITTED",
      "r6g":"LOCKED_BY_SEQUENCE",
      "native_admission_gates":"12_UNMET",
      "production":"NOT_AUTHORIZED"
    }
    try:
        receipt["checks_performed"]=run_role(args.role)
        receipt["status"]="PASS_WITH_CONTROLS_RESEARCH_ONLY"
        code=0
    except (ValueError,old.Blocked,OSError,KeyError,TypeError) as e:
        receipt["checks_performed"]=[]
        receipt["status"]="BLOCK"
        receipt["error"]=str(e)
        code=2
    args.proof_dir.mkdir(parents=True,exist_ok=True)
    (args.proof_dir/("APP022_BUREAU_MACHINE_"+args.role+".json")).write_text(
        json.dumps(receipt,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(json.dumps(receipt,sort_keys=True))
    return code


if __name__=="__main__":
    raise SystemExit(main())
