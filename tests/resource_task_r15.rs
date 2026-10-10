//! NORDOI R0.15 TEST-ONLY independent research-review policy.
//! Two synthetic reviews and matching SHA-256 checks do not authenticate people,
//! establish production trust, or grant native execution authority.

#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::collections::BTreeSet;
use std::fmt::Write;

const BASE_SHA: &str = "86fc90a79f5d0cef83c730bcd7c02175307fffd3";
const COUNT: usize = 12;
const AREAS: [&str; COUNT] = [
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
const R11: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const R12: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const R13: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const R14: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const R15: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const HEADER: &str = "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tarea\tbaseline_sha\tstatus\tscope\tpolicy\tverifier_ref\ttrust_root\tlimitation";
const POLICY: &str = "DUAL_SYNTHETIC_REVIEW";

fn parse_rows<'a>(data: &'a str, header: &str, columns: usize) -> Option<Vec<Vec<&'a str>>> {
    let mut lines = data.lines();
    if lines.next()? != header {
        return None;
    }
    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() || line.contains('\r') {
            return None;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.len() != columns || cells.iter().any(|cell| cell.trim().is_empty()) {
            return None;
        }
        rows.push(cells);
    }
    Some(rows)
}

fn historical_absence() -> bool {
    let Some(g) = parse_rows(
        R11,
        "id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation",
        6,
    ) else {
        return false;
    };
    let Some(e) = parse_rows(R12, "id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation", 9) else { return false; };
    let Some(i) = parse_rows(R13, "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation", 16) else { return false; };
    let Some(a) = parse_rows(R14, "id\tgate_id\tevidence_id\tintegrity_id\tarea\tbaseline_sha\tstatus\tscope\tproducer_key\treviewer_key\ttrust_root\tauthentication\trevocation\tlimitation", 14) else { return false; };
    if g.len() != COUNT || e.len() != COUNT || i.len() != COUNT || a.len() != COUNT {
        return false;
    }
    (0..COUNT).all(|n| {
        g[n][0] == format!("R11-G{:02}", n + 1)
            && g[n][1] == AREAS[n]
            && g[n][2] == "UNMET"
            && g[n][3] == "RESEARCH_ONLY"
            && e[n][0] == format!("R12-E{:02}", n + 1)
            && e[n][1] == g[n][0]
            && e[n][2] == AREAS[n]
            && e[n][3] == "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
            && e[n][4] == "ABSENT"
            && e[n][5] == "RESEARCH_ONLY"
            && e[n][6] == "NONE"
            && e[n][7] == "NONE"
            && i[n][0] == format!("R13-L{:02}", n + 1)
            && i[n][1] == g[n][0]
            && i[n][2] == e[n][0]
            && i[n][3] == AREAS[n]
            && i[n][4] == "851d978c36f4c12fc16c115e89a7774e33ce361b"
            && i[n][5] == "ABSENT"
            && i[n][6] == "RESEARCH_ONLY"
            && i[n][7..15].iter().all(|x| *x == "NONE")
            && a[n][0] == format!("R14-A{:02}", n + 1)
            && a[n][1] == g[n][0]
            && a[n][2] == e[n][0]
            && a[n][3] == i[n][0]
            && a[n][4] == AREAS[n]
            && a[n][5] == "35e048dd83f2c072a6c454494ac099906156dc38"
            && a[n][6] == "ABSENT"
            && a[n][7] == "RESEARCH_ONLY"
            && a[n][8..13].iter().all(|x| *x == "NONE")
    })
}

fn canonical_policy() -> bool {
    if !historical_absence() {
        return false;
    }
    let Some(rows) = parse_rows(R15, HEADER, 13) else {
        return false;
    };
    rows.len() == COUNT
        && rows.iter().enumerate().all(|(n, p)| {
            p[0] == format!("R15-V{:02}", n + 1)
                && p[1] == format!("R11-G{:02}", n + 1)
                && p[2] == format!("R12-E{:02}", n + 1)
                && p[3] == format!("R13-L{:02}", n + 1)
                && p[4] == format!("R14-A{:02}", n + 1)
                && p[5] == AREAS[n]
                && p[6] == BASE_SHA
                && p[7] == "ABSENT"
                && p[8] == "RESEARCH_ONLY"
                && p[9] == POLICY
                && p[10] == "NONE"
                && p[11] == "NONE"
                && p[12].len() >= 65
        })
}

fn sha256_hex(data: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for b in certified_sha256::sha256(data) {
        write!(&mut result, "{b:02x}").expect("writing into a String cannot fail");
    }
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Review {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Debug)]
struct Proposal {
    id: String,
    gate: String,
    area: String,
    baseline: String,
    policy: String,
    scope: String,
    producer: String,
    reviewer_one: String,
    reviewer_two: String,
    bytes: Vec<u8>,
    sha256: String,
    epoch: u64,
    review_one: Review,
    review_two: Review,
    revoked: bool,
    external_signature_claim: bool,
    external_trust_root: String,
}

fn fixtures() -> Vec<Proposal> {
    AREAS
        .iter()
        .enumerate()
        .map(|(n, area)| {
            let bytes = format!("R015-RESEARCH-FIXTURE-{:02}-{area}", n + 1).into_bytes();
            let sha256 = sha256_hex(&bytes);
            Proposal {
                id: format!("R15-V{:02}", n + 1),
                gate: format!("R11-G{:02}", n + 1),
                area: (*area).to_owned(),
                baseline: BASE_SHA.to_owned(),
                policy: POLICY.to_owned(),
                scope: "RESEARCH_ONLY".to_owned(),
                producer: format!("research://producer/{:02}", n + 1),
                reviewer_one: format!("research://reviewer-A/{:02}", n + 1),
                reviewer_two: format!("research://reviewer-B/{:02}", n + 1),
                bytes,
                sha256,
                epoch: 1,
                review_one: Review::Approved,
                review_two: Review::Approved,
                revoked: false,
                external_signature_claim: false,
                external_trust_root: "NONE".to_owned(),
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn assess(proposals: &[Proposal], ci: bool, approval: bool) -> Decision {
    if !canonical_policy() {
        return Decision::Invalid;
    }
    if proposals.is_empty() {
        return Decision::Blocked;
    }
    if proposals.len() != COUNT {
        return Decision::Invalid;
    }
    let mut identities = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
    for (n, proposal) in proposals.iter().enumerate() {
        if proposal.id != format!("R15-V{:02}", n + 1)
            || proposal.gate != format!("R11-G{:02}", n + 1)
            || proposal.area != AREAS[n]
            || proposal.baseline != BASE_SHA
            || proposal.policy != POLICY
            || proposal.scope != "RESEARCH_ONLY"
            || proposal.producer != format!("research://producer/{:02}", n + 1)
            || proposal.reviewer_one != format!("research://reviewer-A/{:02}", n + 1)
            || proposal.reviewer_two != format!("research://reviewer-B/{:02}", n + 1)
            || proposal.reviewer_one == proposal.reviewer_two
            || proposal.reviewer_one == proposal.producer
            || proposal.reviewer_two == proposal.producer
            || proposal.bytes.is_empty()
            || proposal.bytes.len() > 4096
            || proposal.sha256 != sha256_hex(&proposal.bytes)
            || proposal.epoch != 1
            || proposal.external_signature_claim
            || proposal.external_trust_root != "NONE"
            || !identities.insert(proposal.id.as_str())
            || !fingerprints.insert(proposal.sha256.as_str())
        {
            return Decision::Invalid;
        }
    }
    if proposals
        .iter()
        .any(|p| p.revoked || p.review_one != Review::Approved || p.review_two != Review::Approved)
        || !ci
        || !approval
    {
        return Decision::Blocked;
    }
    // These are untrusted synthetic reviewers, not authenticated independent people.
    Decision::ResearchReviewOnly
}

#[test]
fn certified_r014_baseline_is_exact() {
    assert_eq!(BASE_SHA, "86fc90a79f5d0cef83c730bcd7c02175307fffd3");
}
#[test]
fn frozen_r011_native_gates_are_unmet() {
    assert!(historical_absence());
}
#[test]
fn frozen_r012_evidence_remains_absent() {
    assert!(R12
        .lines()
        .skip(1)
        .all(|r| r.contains("\tABSENT\tRESEARCH_ONLY\tNONE\tNONE\t")));
}
#[test]
fn frozen_r013_integrity_remains_absent() {
    assert!(R13
        .lines()
        .skip(1)
        .all(|r| r.contains("\tABSENT\tRESEARCH_ONLY\tNONE\t")));
}
#[test]
fn frozen_r014_authentication_remains_absent() {
    assert!(R14
        .lines()
        .skip(1)
        .all(|r| r.contains("\tABSENT\tRESEARCH_ONLY\tNONE\t")));
}
#[test]
fn policy_registry_contains_twelve_absent_rows() {
    assert!(canonical_policy());
}
#[test]
fn policy_header_is_exact() {
    assert_eq!(parse_rows(R15, HEADER, 13).unwrap().len(), COUNT);
}
#[test]
fn malformed_policy_header_is_rejected() {
    assert!(parse_rows(R15, "bad header", 13).is_none());
}
#[test]
fn malformed_policy_column_count_is_rejected() {
    assert!(parse_rows(R15, HEADER, 12).is_none());
}
#[test]
fn research_fixtures_are_deterministic() {
    assert_eq!(fixtures()[0].sha256, fixtures()[0].sha256);
}
#[test]
fn full_public_fixture_is_only_research_review() {
    assert_eq!(
        assess(&fixtures(), true, true),
        Decision::ResearchReviewOnly
    );
    println!("R15_WITNESS version=1 kind=dual_synthetic_verification proposals=12 real_external_verifiers=0 signatures_verified=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
#[test]
fn empty_fixture_cannot_enter_review() {
    assert_eq!(assess(&[], true, true), Decision::Blocked);
}
#[test]
fn missing_proposal_is_invalid() {
    let mut p = fixtures();
    p.pop();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn extra_proposal_is_invalid() {
    let mut p = fixtures();
    let duplicate = p[0].clone();
    p.push(duplicate);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn reordered_proposals_are_invalid() {
    let mut p = fixtures();
    p.swap(0, 1);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_gate_is_invalid() {
    let mut p = fixtures();
    p[0].gate = "R11-G12".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_area_is_invalid() {
    let mut p = fixtures();
    p[0].area = "OTHER".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_policy_is_invalid() {
    let mut p = fixtures();
    p[0].policy = "AUTO_TRUST".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn forged_baseline_is_invalid() {
    let mut p = fixtures();
    p[0].baseline = "0".repeat(40);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_producer_is_invalid() {
    let mut p = fixtures();
    p[0].producer = "research://producer/02".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_reviewer_one_is_invalid() {
    let mut p = fixtures();
    p[0].reviewer_one = "research://reviewer-A/02".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn changed_reviewer_two_is_invalid() {
    let mut p = fixtures();
    p[0].reviewer_two = "research://reviewer-B/02".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn reviewer_role_collision_is_invalid() {
    let mut p = fixtures();
    p[0].reviewer_one = p[0].producer.clone();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn duplicate_reviewers_are_invalid() {
    let mut p = fixtures();
    p[0].reviewer_two = p[0].reviewer_one.clone();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn payload_tampering_without_hash_change_is_invalid() {
    let mut p = fixtures();
    p[0].bytes[0] ^= 1;
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn forged_sha256_is_invalid() {
    let mut p = fixtures();
    p[0].sha256 = "f".repeat(64);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn duplicate_payload_is_invalid_even_when_rehashed() {
    let mut p = fixtures();
    p[1].bytes = p[0].bytes.clone();
    p[1].sha256 = sha256_hex(&p[1].bytes);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn empty_payload_is_invalid() {
    let mut p = fixtures();
    p[0].bytes.clear();
    p[0].sha256 = sha256_hex(&p[0].bytes);
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn forged_signature_claim_is_invalid() {
    let mut p = fixtures();
    p[0].external_signature_claim = true;
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn claimed_trust_root_is_invalid() {
    let mut p = fixtures();
    p[0].external_trust_root = "claimed-key".into();
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn reviewer_one_rejects_stays_blocked() {
    let mut p = fixtures();
    p[0].review_one = Review::Rejected;
    assert_eq!(assess(&p, true, true), Decision::Blocked);
}
#[test]
fn reviewer_two_rejects_stays_blocked() {
    let mut p = fixtures();
    p[0].review_two = Review::Rejected;
    assert_eq!(assess(&p, true, true), Decision::Blocked);
}
#[test]
fn reviewer_one_pending_stays_blocked() {
    let mut p = fixtures();
    p[0].review_one = Review::Pending;
    assert_eq!(assess(&p, true, true), Decision::Blocked);
}
#[test]
fn reviewer_two_pending_stays_blocked() {
    let mut p = fixtures();
    p[0].review_two = Review::Pending;
    assert_eq!(assess(&p, true, true), Decision::Blocked);
}
#[test]
fn revoked_proposal_stays_blocked() {
    let mut p = fixtures();
    p[0].revoked = true;
    assert_eq!(assess(&p, true, true), Decision::Blocked);
}
#[test]
fn ci_without_explicit_approval_stays_blocked() {
    assert_eq!(assess(&fixtures(), true, false), Decision::Blocked);
}
#[test]
fn approval_without_ci_stays_blocked() {
    assert_eq!(assess(&fixtures(), false, true), Decision::Blocked);
}
#[test]
fn stale_policy_epoch_is_invalid() {
    let mut p = fixtures();
    p[0].epoch = 2;
    assert_eq!(assess(&p, true, true), Decision::Invalid);
}
#[test]
fn no_native_execution_api_is_provided() {
    assert_ne!(assess(&fixtures(), true, true), Decision::Blocked);
    assert_eq!(
        assess(&fixtures(), true, true),
        Decision::ResearchReviewOnly
    );
}
