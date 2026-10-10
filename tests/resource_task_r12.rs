//! NORDOI R0.12 — test-only evidence admission boundary, not an evidence authority.
//! The immutable R0.11 native registry remains 12/12 UNMET, irrespective of mock approvals.

#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Command, Model, Receipt, ScopeBudget, ScopeOutcome};
use std::collections::BTreeSet;

const BASELINE_SHA: &str = "d61487bd2392890b3aaaa6ef33fce4371cfb6850";
const GATE_COUNT: usize = 12;
const AREAS: [&str; GATE_COUNT] = [
    "LIFECYCLE_SEMANTICS",
    "EXECUTOR_SCHEDULING",
    "MEMORY_OWNERSHIP",
    "CAPABILITY_AUTHORITY",
    "CANCELLATION_CLEANUP",
    "EFFECT_IO_BOUNDARY",
    "CRASH_RECOVERY",
    "ADVERSARIAL_TESTING",
    "CROSS_PLATFORM",
    "COMPILER_NAIR_BINDING",
    "RESOURCE_LIMITS",
    "INDEPENDENT_REVIEW",
];
const EVIDENCE_LEDGER: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const PREVIOUS_GATES: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Evidence<'a> {
    id: &'a str,
    gate_id: &'a str,
    area: &'a str,
    baseline_sha: &'a str,
    status: &'a str,
    evidence_scope: &'a str,
    artifact_ref: &'a str,
    reviewer_ref: &'a str,
    limitation: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Advisory {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn parse_ledger(src: &str) -> Result<Vec<Evidence<'_>>, &'static str> {
    let mut lines = src.lines();
    if lines.next() != Some("id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation") {
        return Err("unrecognized ledger schema");
    }
    let mut entries = Vec::new();
    for line in lines {
        if line.trim().is_empty() || line.contains('\r') {
            return Err("empty or malformed evidence row");
        }
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() != 9 {
            return Err("incorrect evidence row column count");
        }
        if !matches!(parts[4], "ABSENT" | "PROPOSED") || parts[5] != "RESEARCH_ONLY" {
            return Err("unsupported status or evidence scope");
        }
        entries.push(Evidence {
            id: parts[0],
            gate_id: parts[1],
            area: parts[2],
            baseline_sha: parts[3],
            status: parts[4],
            evidence_scope: parts[5],
            artifact_ref: parts[6],
            reviewer_ref: parts[7],
            limitation: parts[8],
        });
    }
    Ok(entries)
}

fn ledger() -> Vec<Evidence<'static>> {
    parse_ledger(EVIDENCE_LEDGER).expect("canonical evidence fixture must parse")
}

fn frozen_native_gates_unmet() -> bool {
    let mut lines = PREVIOUS_GATES.lines();
    if lines.next() != Some("id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation") {
        return false;
    }
    let rows: Vec<_> = lines.collect();
    if rows.len() != GATE_COUNT {
        return false;
    }
    rows.iter().enumerate().all(|(index, row)| {
        let cols: Vec<_> = row.split('\t').collect();
        cols.len() == 6
            && cols[0] == format!("R11-G{:02}", index + 1)
            && cols[1] == AREAS[index]
            && cols[2] == "UNMET"
            && cols[3] == "RESEARCH_ONLY"
    })
}

// Synthetic policy ONLY: never verifies a signature, an artifact digest, an operating system
// runtime, or the source of an approval. Its strongest outcome is a research review request.
fn assess(evidence: &[Evidence<'_>], mock_ci: bool, mock_approval: bool) -> Advisory {
    if !frozen_native_gates_unmet() || evidence.len() != GATE_COUNT {
        return Advisory::Invalid;
    }
    let mut ids = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for (index, entry) in evidence.iter().enumerate() {
        if entry.id != format!("R12-E{:02}", index + 1)
            || entry.gate_id != format!("R11-G{:02}", index + 1)
            || entry.area != AREAS[index]
            || entry.baseline_sha != BASELINE_SHA
            || entry.evidence_scope != "RESEARCH_ONLY"
            || entry.limitation.trim().len() < 65
            || !ids.insert(entry.id)
        {
            return Advisory::Invalid;
        }
        match entry.status {
            "ABSENT" if entry.artifact_ref == "NONE" && entry.reviewer_ref == "NONE" => {}
            "PROPOSED"
                if entry.artifact_ref.starts_with("research://synthetic/")
                    && entry
                        .reviewer_ref
                        .starts_with("research://synthetic/reviewer/")
                    && artifacts.insert(entry.artifact_ref) => {}
            _ => return Advisory::Invalid,
        }
    }
    if !evidence.iter().all(|entry| entry.status == "PROPOSED") || !mock_ci || !mock_approval {
        return Advisory::Blocked;
    }
    Advisory::ResearchReviewOnly
}

fn proposed() -> Vec<Evidence<'static>> {
    let mut entries = ledger();
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.status = "PROPOSED";
        // The synthetic locator is metadata, NOT a verified artifact or content hash.
        entry.artifact_ref = match index {
            0 => "research://synthetic/01",
            1 => "research://synthetic/02",
            2 => "research://synthetic/03",
            3 => "research://synthetic/04",
            4 => "research://synthetic/05",
            5 => "research://synthetic/06",
            6 => "research://synthetic/07",
            7 => "research://synthetic/08",
            8 => "research://synthetic/09",
            9 => "research://synthetic/10",
            10 => "research://synthetic/11",
            _ => "research://synthetic/12",
        };
        entry.reviewer_ref = "research://synthetic/reviewer/unverified";
    }
    entries
}

#[test]
fn certified_r011_sha_is_exact() {
    assert_eq!(BASELINE_SHA, "d61487bd2392890b3aaaa6ef33fce4371cfb6850");
}

#[test]
fn previous_twelve_gates_remain_unmet() {
    assert!(frozen_native_gates_unmet());
}

#[test]
fn ledger_has_exactly_twelve_entries() {
    assert_eq!(ledger().len(), GATE_COUNT);
}

#[test]
fn evidence_ids_and_gate_mappings_are_ordered() {
    for (index, entry) in ledger().iter().enumerate() {
        assert_eq!(entry.id, format!("R12-E{:02}", index + 1));
        assert_eq!(entry.gate_id, format!("R11-G{:02}", index + 1));
        assert_eq!(entry.area, AREAS[index]);
    }
}

#[test]
fn every_canonical_evidence_record_is_absent() {
    assert!(ledger().iter().all(|entry| entry.status == "ABSENT"));
}

#[test]
fn no_canonical_artifact_or_reviewer_is_claimed() {
    assert!(ledger()
        .iter()
        .all(|entry| entry.artifact_ref == "NONE" && entry.reviewer_ref == "NONE"));
}

#[test]
fn every_canonical_record_is_research_only() {
    assert!(ledger()
        .iter()
        .all(|entry| entry.evidence_scope == "RESEARCH_ONLY"));
}

#[test]
fn every_record_has_explicit_research_limitation() {
    assert!(ledger().iter().all(|entry| entry.limitation.len() >= 65));
}

#[test]
fn default_assessment_is_blocked() {
    assert_eq!(assess(&ledger(), false, false), Advisory::Blocked);
}

#[test]
fn green_ci_does_not_promote_absent_evidence() {
    assert_eq!(assess(&ledger(), true, false), Advisory::Blocked);
    assert_eq!(assess(&ledger(), true, true), Advisory::Blocked);
}

#[test]
fn mock_approval_does_not_promote_absent_evidence() {
    assert_eq!(assess(&ledger(), false, true), Advisory::Blocked);
}

#[test]
fn missing_evidence_row_fails_closed() {
    let mut entries = proposed();
    entries.pop();
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn duplicate_evidence_id_fails_closed() {
    let mut entries = proposed();
    entries[1] = entries[0].clone();
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn reordered_evidence_rows_fail_closed() {
    let mut entries = proposed();
    entries.swap(0, 1);
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn mismatched_gate_id_fails_closed() {
    let mut entries = proposed();
    entries[0].gate_id = "R11-G12";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn changed_area_fails_closed() {
    let mut entries = proposed();
    entries[2].area = "FAKE_AREA";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn stale_or_forged_baseline_sha_fails_closed() {
    let mut entries = proposed();
    entries[3].baseline_sha = "0000000000000000000000000000000000000000";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn unknown_status_cannot_be_interpreted_as_verified() {
    let broken = EVIDENCE_LEDGER.replacen("\tABSENT\t", "\tVERIFIED\t", 1);
    assert!(parse_ledger(&broken).is_err());
    let mut entries = proposed();
    entries[0].status = "NATIVE_VERIFIED";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn native_evidence_scope_is_not_admitted_by_test_harness() {
    let broken = EVIDENCE_LEDGER.replacen("\tRESEARCH_ONLY\t", "\tNATIVE_VERIFIED\t", 1);
    assert!(parse_ledger(&broken).is_err());
    let mut entries = proposed();
    entries[0].evidence_scope = "NATIVE_VERIFIED";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn absent_status_with_forged_artifact_fails_closed() {
    let mut entries = ledger();
    entries[0].artifact_ref = "research://synthetic/forged";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn proposed_status_missing_artifact_fails_closed() {
    let mut entries = proposed();
    entries[0].artifact_ref = "NONE";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn untrusted_foreign_artifact_locator_fails_closed() {
    let mut entries = proposed();
    entries[0].artifact_ref = "https://invalid.example/evidence";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn duplicate_artifact_locator_fails_closed() {
    let mut entries = proposed();
    let artifact = entries[0].artifact_ref;
    entries[1].artifact_ref = artifact;
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn proposed_status_missing_reviewer_fails_closed() {
    let mut entries = proposed();
    entries[1].reviewer_ref = "NONE";
    assert_eq!(assess(&entries, true, true), Advisory::Invalid);
}

#[test]
fn all_mock_proposals_never_authorize_native_execution() {
    let entries = proposed();
    assert_eq!(assess(&entries, true, true), Advisory::ResearchReviewOnly);
    assert!(frozen_native_gates_unmet());
    println!("R12_WITNESS version=1 kind=synthetic_evidence_admission records=12 status=RESEARCH_REVIEW_ONLY native_gates_unmet=12 outcome=PASS");
}

#[test]
fn missing_mock_approval_or_ci_blocks_review() {
    let entries = proposed();
    assert_eq!(assess(&entries, false, true), Advisory::Blocked);
    assert_eq!(assess(&entries, true, false), Advisory::Blocked);
}

#[test]
fn malformed_tsv_header_and_row_are_rejected() {
    assert!(parse_ledger("invalid\n").is_err());
    let broken = EVIDENCE_LEDGER.replacen("\tNONE\tNONE\t", "\tNONE\t", 1);
    assert!(parse_ledger(&broken).is_err());
}

#[test]
fn frozen_reference_smoke_replay_still_succeeds() {
    let (mut model, _host) = Model::bootstrap(
        0x5231_0012,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        2,
    );
    let root = model.root();
    let receipt = model.apply(Command::Close { scope: root });
    assert!(
        matches!(receipt, Ok(Receipt::Closed(ref report)) if report.outcome == ScopeOutcome::Succeeded)
    );
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn no_production_boundary_is_modified_or_exported() {
    let library = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");
    for forbidden in [
        "resource_task_r12",
        "r12_evidence_ledger",
        "r12_evidence_witnesses",
    ] {
        assert!(!library.contains(forbidden));
        assert!(!manifest.contains(forbidden));
    }
    assert!(PREVIOUS_GATES.contains("R11-G12\tINDEPENDENT_REVIEW\tUNMET"));
}
