#!/usr/bin/env python3
"""GOV-FIX1: one actually executed deterministic machine check per Bureau role.

A CI job labelled ATLAS/FERRUM/... is not a distinct autonomous AI agent, a
third-party approval or a formal Bureau research certificate. The role output
contains source pin, real checks performed, and unavoidable scope limits.
Historical R6F functions remain NOT_SUBMITTED; no fake reports are written.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts import roadmap_guardian as guard

APP021 = "6e9c300922348d0ebab4318c7f9c2b4dd4cd76b4"


def text(path):
    p = ROOT / path
    if not p.is_file() or p.is_symlink():
        raise ValueError("MISSING_OR_LINKED_SOURCE:" + path)
    return p.read_text("utf-8")


def req(value, code):
    if not value:
        raise ValueError(code)


def includes(path, snippets, ident):
    raw = text(path)
    for snippet in snippets:
        req(snippet in raw, "MISSING_" + ident + "_" + snippet)
    return ident + "_REAL_SOURCE_CONTRACT_PASS"


def checks(role, guard_verdict):
    plan = json.loads(text(guard.PLAN))
    register = json.loads(text(guard.TRACK))
    records = register["milestones"]
    results = ["GUARDIAN_OFFLINE_STATIC_AUDIT_PASS"]
    req(guard_verdict["BUREAU_CERTIFICATION"] == "PENDING" and
        guard_verdict["APP_BUREAU_FORMAL_CERTIFICATION"] == "NOT_CERTIFIED"
        and guard_verdict["R6G"] == "LOCKED_BY_SEQUENCE",
        "FORMAL_BUREAU_STATUS_MISREPRESENTED")
    if role == "ATLAS":
        results += [
            includes("Cargo.toml", ["[package]"], "NORDOI_COMPILED_RUST_PACKAGE"),
            includes("examples/apps/app016_receiver_trust_policy/receiver_app016.py",
                     ["import", "def main"], "APP016_RECEIVER_ARCHITECTURE"),
            includes("examples/apps/app021_authority_rotation/receiver_app021.py",
                     ["import receiver_app020 as signed",
                      "return portable.resume_native("],
                     "APP021_REUSES_RUST_NATIVE_ENGINE")]
        req(records[-1]["parent_exact_sha"] == records[-2]["head_exact_sha"],
            "APP021_PARENT_SEMANTIC_SEQUENCE")
        results.append("APP021_EXACT_RESEARCH_PARENT_VALIDATED")
    elif role == "FERRUM":
        workflow = ".github/workflows/nordoi_app021_authority_rotation.yml"
        results += [includes(workflow,
                            ["macos-15-intel", "macos-15", "ubuntu-latest",
                             "windows-latest", "cargo build --offline --bin nordoi"],
                            "FOUR_OS_REAL_RUST_QUALIFICATION_DECLARED")]
        req(plan["native_admission"]["remaining"] == 12 and
            plan["native_admission"]["production"] == "NOT_AUTHORIZED",
            "NATIVE_ADMISSION_IMPLICITLY_GRANTED")
        results.append("TWELVE_NATIVE_OS_ADMISSION_DENIALS_PRESERVED")
    elif role == "BOUNDARY":
        results += [
            includes("examples/apps/app021_authority_rotation/receiver_app021.py",
                     ["signed.nonoverlap(", "with anchor.locked(root):",
                      "AUTHORITY_FROZEN_DENIES_NATIVE_ADMISSION"],
                     "PROCESS_BOUNDARY_AND_EFFECT_FENCE"),
            includes("examples/apps/app020_signed_head_provenance/receiver_app020.py",
                     ["signed_statement", "nonoverlap(", "anchor.active_locked("],
                     "APP020_SSHSIG_PACKAGE_BOUNDARY")]
        results.append("EXTERNAL_ADMINISTRATIVE_ISOLATION_NOT_PROVEN")
    elif role == "CIPHER":
        results += [
            includes("examples/apps/app021_authority_rotation/receiver_app021.py",
                     ["-Y", "verify_sshsig(", "outgoing_signature_b64",
                      "incoming_signature_b64", "AUTHORITY_REVOKED_KEY_READMITTED"],
                     "DUAL_OPENSSH_SSHSIG_SOURCE_CONTRACT"),
            includes("examples/apps/app020_signed_head_provenance/receiver_app020.py",
                     ["SSHSIG_SIGNATURE_VERIFICATION_FAILED", "NAMESPACE"],
                     "APP020_DISTINCT_HEAD_NAMESPACE")]
        results.append("EXTERNAL_PRIVATE_KEY_CUSTODY_NOT_PROVEN")
    elif role == "LEDGER":
        for i,(milestone,expected,prev,pr) in enumerate(guard.APP_CHAIN):
            req(records[i]["head_exact_sha"] == expected and
                records[i]["parent_exact_sha"] == prev and
                records[i]["pr_number"] == pr,
                "C20_IMMUTABLE_RESEARCH_CHAIN_INVALID")
        results.append("SIX_RESEARCH_GIT_PARENT_PINS_EXACT")
        req(register["protected_baseline"]["global_monotonic_antirollback"] ==
            "NOT_VERIFIED", "FALSE_GLOBAL_ANTIROLLBACK")
        results.append("GLOBAL_MONOTONIC_FRESHNESS_NOT_PROVEN")
    elif role == "CAPSTONE":
        req(all(x["production"] == "NOT_AUTHORIZED" and
                x["formal_bureau_certification"] == "NOT_CERTIFIED"
                for x in records), "FALSE_PRIVILEGED_CAPABILITY_ADMISSION")
        req(plan["approval_policy"]["no_ai_self_certification"] is True,
            "BUREAU_SELF_CERTIFICATION_ENABLED")
        results += ["ALL_SIX_APP_RESEARCH_OUTPUTS_NONPRIVILEGED",
                    "NO_AI_SELF_CERTIFICATION_PRESERVED"]
    elif role == "ADVERSARY":
        results += [
            includes("examples/apps/app021_authority_rotation/test_app021.py",
                     ["test_26_revoked_root_key_cannot_be_readmitted",
                      "test_27_rotation_requires_incoming_signature",
                      "test_39_complete_old_journal_chain_and_floors_can_be_replayed_limit"],
                     "REAL_NEGATIVE_SSHSIG_TESTS_PRESENT"),
            includes("tests/test_app_governance_reconciliation.py",
                     ["test_05_delete_app018_from_manifest_is_rejected",
                      "test_13_fake_bureau_certification_is_rejected",
                      "test_17_silent_native_gate_removal_is_rejected"],
                     "GUARDIAN_NEGATIVE_FALSIFICATION_TESTS_PRESENT")]
        results.append("NINE_INDEPENDENT_REVIEWERS_NOT_PROVEN")
    elif role == "VERITY":
        req(len(records) == 6 and all(
            x["exact_head_github_success"] == 13 and
            x["exact_head_github_total"] == 13 for x in records),
            "CI_OBSERVATION_MANIFEST_WRONG")
        req(register["observed_checks_source"] ==
            "GITHUB_CHECK_RUNS_EXACT_HEAD_VERIFIED_ON_2026_11_23_NOT_AN_INDEPENDENT_CUSTODIAN",
            "CI_RESULT_WASHED_AS_INDEPENDENT_CERTIFICATION")
        results += ["SIX_HISTORIC_EXACT_HEAD_CHECK_METADATA_OBSERVED",
                    "LIVE_GITHUB_STATUS_NOT_PROVEN_BY_OFFLINE_JOB"]
    elif role == "GUARDIAN":
        req(guard_verdict["REVIEWS_SUBMITTED"] == 0
            and guard_verdict["NATIVE_ADMISSION_GATES"] == "12_UNMET"
            and guard_verdict["APP_NEXT"] == "LOCKED_BY_GOVERNANCE_REVIEW",
            "GUARDIAN_FALSE_PROMOTION")
        for target in ("CERTIFIED_RESEARCH","NATIVE_ADMISSION","PRODUCTION"):
            req(guard.deny_transition(target)["TRANSITION"] == "BLOCK",
                "GUARDIAN_PROMOTION_NOT_BLOCKED")
        results += ["R6F_NINE_REVIEWS_NOT_SUBMITTED",
                    "R6G_NATIVE_AND_PRODUCTION_PROMOTIONS_DENIED"]
    else:
        raise ValueError("UNSUPPORTED_BUREAU_ROLE")
    return results


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--role", choices=guard.ROLE_IDS, required=True)
    p.add_argument("--proof-dir", type=Path, required=True)
    args = p.parse_args(argv)
    source = subprocess.run(["git", "rev-parse", "HEAD"],
                            cwd=ROOT, capture_output=True,text=True,timeout=20)
    observed_sha = source.stdout.strip() if source.returncode == 0 else "UNKNOWN"
    receipt = {
        "schema": "NORDOI_GOV_FIX1_BUREAU_MACHINE_FUNCTION_RECEIPT_V1",
        "role": args.role,
        "source_sha": observed_sha,
        "parent_application_sha": APP021,
        "execution": "ACTUAL_DETERMINISTIC_MACHINE_ROLE_FUNCTION",
        "autonomous_ai_agent_reviewer": "NOT_PROVEN",
        "independent_administrative_signer": "NOT_PROVEN",
        "r6f_formal_report": "NOT_SUBMITTED",
        "app_formal_certification": "NOT_CERTIFIED",
        "native_admission": "DENIED",
        "production": "NOT_AUTHORIZED",
    }
    try:
        verdict = guard.audit(ROOT)
        receipt["checks_performed"] = checks(args.role, verdict)
        receipt["status"] = "PASS_WITH_CONTROLS_RESEARCH_ONLY"
        code = 0
    except (ValueError,OSError,TypeError,KeyError,guard.Blocked) as e:
        receipt["status"] = "BLOCK"
        receipt["error"] = str(e)
        receipt["checks_performed"] = []
        code = 2
    args.proof_dir.mkdir(parents=True,exist_ok=True)
    path = args.proof_dir / ("GOV_FIX1_BUREAU_MACHINE_" + args.role + ".json")
    path.write_text(json.dumps(receipt, indent=2,sort_keys=True)+"\n",encoding="utf-8")
    print(json.dumps(receipt, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
