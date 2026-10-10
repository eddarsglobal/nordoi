//! R0.14 TEST-ONLY: synthetic authentication admission and revocation governance.
//! Fixture SHA-256 digests are PUBLIC and NOT digital signatures or authenticated identity.

#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;
#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod frozen_reference;

use frozen_reference::{Command, Model, Receipt, ScopeBudget, ScopeOutcome};
use std::collections::BTreeSet;

const BASE_SHA: &str = "35e048dd83f2c072a6c454494ac099906156dc38";
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
const R11_GATES: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const R12_LEDGER: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const R13_LEDGER: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const R14_REGISTRY: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const R14_HEADER: &str = "id\tgate_id\tevidence_id\tintegrity_id\tarea\tbaseline_sha\tstatus\tscope\tproducer_key\treviewer_key\ttrust_root\tauthentication\trevocation\tlimitation";
const FIXTURE_ROOT: &str = "research://fixture/root-01";
const FIXTURE_REVOKER: &str = "research://fixture/revocation-steward";

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(64);
    for value in certified_sha256::sha256(bytes) {
        write!(&mut result, "{value:02x}").expect("String formatting cannot fail");
    }
    result
}

fn append_field(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn parse_rows<'a>(
    source: &'a str,
    expected_header: &str,
    columns: usize,
) -> Option<Vec<Vec<&'a str>>> {
    let mut lines = source.lines();
    if lines.next()? != expected_header {
        return None;
    }
    let mut result = Vec::new();
    for line in lines {
        if line.is_empty() || line.contains('\r') {
            return None;
        }
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != columns || row.iter().any(|v| v.trim().is_empty()) {
            return None;
        }
        result.push(row);
    }
    Some(result)
}

fn historical_boundary() -> bool {
    let Some(gates) = parse_rows(
        R11_GATES,
        "id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation",
        6,
    ) else {
        return false;
    };
    let Some(evidence) = parse_rows(R12_LEDGER,
        "id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation", 9) else { return false; };
    // Require the exact frozen R0.13 schema, not merely a header prefix.
    let r13_header = "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation";
    let Some(integrity) = parse_rows(R13_LEDGER, r13_header, 16) else {
        return false;
    };
    if gates.len() != COUNT || evidence.len() != COUNT || integrity.len() != COUNT {
        return false;
    }
    (0..COUNT).all(|i| {
        let g = &gates[i];
        let e = &evidence[i];
        let c = &integrity[i];
        g[0] == format!("R11-G{:02}", i + 1)
            && g[1] == AREAS[i]
            && g[2] == "UNMET"
            && g[3] == "RESEARCH_ONLY"
            && e[0] == format!("R12-E{:02}", i + 1)
            && e[1] == g[0]
            && e[2] == AREAS[i]
            && e[3] == "d61487bd2392890b3aaaa6ef33fce4371cfb6850"
            && e[4] == "ABSENT"
            && e[5] == "RESEARCH_ONLY"
            && e[6] == "NONE"
            && e[7] == "NONE"
            && c[0] == format!("R13-L{:02}", i + 1)
            && c[1] == g[0]
            && c[2] == e[0]
            && c[3] == AREAS[i]
            && c[4] == "851d978c36f4c12fc16c115e89a7774e33ce361b"
            && c[5] == "ABSENT"
            && c[6] == "RESEARCH_ONLY"
            && c[7..15].iter().all(|x| *x == "NONE")
    })
}

fn canonical_registry() -> bool {
    if !historical_boundary() {
        return false;
    }
    let Some(rows) = parse_rows(R14_REGISTRY, R14_HEADER, 14) else {
        return false;
    };
    rows.len() == COUNT
        && rows.iter().enumerate().all(|(i, row)| {
            row[0] == format!("R14-A{:02}", i + 1)
                && row[1] == format!("R11-G{:02}", i + 1)
                && row[2] == format!("R12-E{:02}", i + 1)
                && row[3] == format!("R13-L{:02}", i + 1)
                && row[4] == AREAS[i]
                && row[5] == BASE_SHA
                && row[6] == "ABSENT"
                && row[7] == "RESEARCH_ONLY"
                && row[8..13].iter().all(|v| *v == "NONE")
                && row[13].len() >= 65
        })
}

#[derive(Clone, Debug)]
struct FixtureReceipt {
    id: String,
    gate: String,
    evidence: String,
    integrity: String,
    area: String,
    baseline: String,
    scope: String,
    producer: String,
    reviewer: String,
    key: String,
    root: String,
    epoch: u64,
    sequence: u64,
    artifact: Vec<u8>,
    artifact_sha: String,
    fixture_digest: String,
    signature_claim: String,
}

fn fixture_commit(r: &FixtureReceipt) -> String {
    let mut bytes = b"NORDOI-R014-PUBLIC-FIXTURE-NOT-A-SIGNATURE-v1".to_vec();
    for field in [
        r.id.as_bytes(),
        r.gate.as_bytes(),
        r.evidence.as_bytes(),
        r.integrity.as_bytes(),
        r.area.as_bytes(),
        r.baseline.as_bytes(),
        r.scope.as_bytes(),
        r.producer.as_bytes(),
        r.reviewer.as_bytes(),
        r.key.as_bytes(),
        r.root.as_bytes(),
        r.artifact_sha.as_bytes(),
    ] {
        append_field(&mut bytes, field);
    }
    append_field(&mut bytes, &r.epoch.to_be_bytes());
    append_field(&mut bytes, &r.sequence.to_be_bytes());
    digest(&bytes)
}

fn make_receipts() -> Vec<FixtureReceipt> {
    AREAS
        .iter()
        .enumerate()
        .map(|(i, &area)| {
            let artifact = format!("R014-PUBLIC-RESEARCH-FIXTURE-{}-{area}", i + 1).into_bytes();
            let mut r = FixtureReceipt {
                id: format!("R14-A{:02}", i + 1),
                gate: format!("R11-G{:02}", i + 1),
                evidence: format!("R12-E{:02}", i + 1),
                integrity: format!("R13-L{:02}", i + 1),
                area: area.to_string(),
                baseline: BASE_SHA.to_string(),
                scope: "RESEARCH_ONLY".to_string(),
                producer: format!("research://fixture/producer/{:02}", i + 1),
                reviewer: format!("research://fixture/reviewer/{:02}", i + 1),
                key: format!("research://fixture/key/{:02}", i + 1),
                root: FIXTURE_ROOT.to_string(),
                epoch: 1,
                sequence: (i + 1) as u64,
                artifact_sha: digest(&artifact),
                artifact,
                fixture_digest: String::new(),
                signature_claim: "NONE".to_string(),
            };
            r.fixture_digest = fixture_commit(&r);
            r
        })
        .collect()
}

fn reseal(r: &mut FixtureReceipt) {
    r.fixture_digest = fixture_commit(r);
}

#[derive(Clone, Debug)]
struct Revocation {
    target: String,
    authority: String,
    epoch: u64,
    reason: String,
    fixture_digest: String,
}

fn revoke(target: &str, epoch: u64) -> Revocation {
    let mut r = Revocation {
        target: target.to_string(),
        authority: FIXTURE_REVOKER.to_string(),
        epoch,
        reason: "TEST_ONLY_KEY_COMPROMISE".to_string(),
        fixture_digest: String::new(),
    };
    r.fixture_digest = revocation_commit(&r);
    r
}

fn revocation_commit(r: &Revocation) -> String {
    let mut bytes = b"NORDOI-R014-PUBLIC-REVOCATION-FIXTURE-v1".to_vec();
    for field in [
        r.target.as_bytes(),
        r.authority.as_bytes(),
        r.reason.as_bytes(),
    ] {
        append_field(&mut bytes, field);
    }
    append_field(&mut bytes, &r.epoch.to_be_bytes());
    digest(&bytes)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn assess(
    receipts: &[FixtureReceipt],
    revocations: &[Revocation],
    ci: bool,
    approval: bool,
) -> Decision {
    if !canonical_registry() {
        return Decision::Invalid;
    }
    if receipts.is_empty() && revocations.is_empty() {
        return Decision::Blocked;
    }
    if receipts.len() != COUNT {
        return Decision::Invalid;
    }
    let mut ids = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for (i, r) in receipts.iter().enumerate() {
        if r.id != format!("R14-A{:02}", i + 1)
            || r.gate != format!("R11-G{:02}", i + 1)
            || r.evidence != format!("R12-E{:02}", i + 1)
            || r.integrity != format!("R13-L{:02}", i + 1)
            || r.area != AREAS[i]
            || r.baseline != BASE_SHA
            || r.scope != "RESEARCH_ONLY"
            || r.producer != format!("research://fixture/producer/{:02}", i + 1)
            || r.reviewer != format!("research://fixture/reviewer/{:02}", i + 1)
            || r.key != format!("research://fixture/key/{:02}", i + 1)
            || r.producer == r.reviewer
            || r.root != FIXTURE_ROOT
            || r.epoch != 1
            || r.sequence != (i + 1) as u64
            || r.artifact.is_empty()
            || r.artifact.len() > 2048
            || r.artifact_sha != digest(&r.artifact)
            || r.fixture_digest != fixture_commit(r)
            || r.signature_claim != "NONE"
            || !ids.insert(r.id.as_str())
            || !keys.insert(r.key.as_str())
            || !digests.insert(r.artifact_sha.as_str())
        {
            return Decision::Invalid;
        }
    }
    let mut targets = BTreeSet::new();
    for revoke in revocations {
        let known = revoke.target == FIXTURE_ROOT || keys.contains(revoke.target.as_str());
        if !known
            || !targets.insert(revoke.target.as_str())
            || revoke.authority != FIXTURE_REVOKER
            || revoke.epoch != 1
            || revoke.reason != "TEST_ONLY_KEY_COMPROMISE"
            || revoke.fixture_digest != revocation_commit(revoke)
        {
            return Decision::Invalid;
        }
    }
    if !revocations.is_empty() || !ci || !approval {
        return Decision::Blocked;
    }
    // Public fixture checks never authenticate a human or authorize the native runtime.
    Decision::ResearchReviewOnly
}

#[test]
fn certified_r013_sha_is_exact() {
    assert_eq!(BASE_SHA, "35e048dd83f2c072a6c454494ac099906156dc38");
}
#[test]
fn frozen_r011_gates_are_unmet() {
    assert!(historical_boundary());
}
#[test]
fn frozen_r012_evidence_is_absent() {
    assert!(R12_LEDGER
        .lines()
        .skip(1)
        .all(|l| l.contains("\tABSENT\tRESEARCH_ONLY\tNONE\tNONE\t")));
}
#[test]
fn frozen_r013_integrity_is_absent() {
    assert!(R13_LEDGER
        .lines()
        .skip(1)
        .all(|l| l.contains("\tABSENT\tRESEARCH_ONLY\tNONE\t")));
}
#[test]
fn r014_registry_has_twelve_rows() {
    assert_eq!(
        parse_rows(R14_REGISTRY, R14_HEADER, 14).unwrap().len(),
        COUNT
    );
}
#[test]
fn canonical_r014_registry_rejects_identity_claims() {
    assert!(canonical_registry());
}
#[test]
fn malformed_r014_registry_fails_closed() {
    assert!(parse_rows("wrong\n", R14_HEADER, 14).is_none());
}
#[test]
fn malformed_r014_row_fails_closed() {
    let s = R14_REGISTRY.replacen("\tNONE\tNONE\t", "\tNONE\t", 1);
    assert!(parse_rows(&s, R14_HEADER, 14).is_none());
}
#[test]
fn fixtures_have_unique_key_identifiers() {
    let r = make_receipts();
    let keys: BTreeSet<_> = r.iter().map(|x| &x.key).collect();
    assert_eq!(keys.len(), COUNT);
}
#[test]
fn public_fixture_digests_are_deterministic() {
    assert_eq!(
        make_receipts()[0].fixture_digest,
        make_receipts()[0].fixture_digest
    );
}
#[test]
fn full_fixture_can_only_enter_research_review() {
    assert_eq!(
        assess(&make_receipts(), &[], true, true),
        Decision::ResearchReviewOnly
    );
    println!("R14_WITNESS version=1 kind=public_authentication_fixture records=12 trust_roots_admitted=0 real_signatures_verified=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
#[test]
fn empty_fixture_stays_blocked() {
    assert_eq!(assess(&[], &[], true, true), Decision::Blocked);
}
#[test]
fn missing_receipt_fails_closed() {
    let mut r = make_receipts();
    r.pop();
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn reordered_receipts_fail_closed() {
    let mut r = make_receipts();
    r.swap(0, 1);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn forged_gate_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].gate = "R11-G12".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn forged_evidence_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].evidence = "R12-E12".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn forged_integrity_lineage_fails_closed() {
    let mut r = make_receipts();
    r[0].integrity = "R13-L12".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn changed_area_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].area = "FAKE".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn forged_baseline_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].baseline = "0".repeat(40);
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn producer_identity_swap_fails_closed() {
    let mut r = make_receipts();
    r[0].producer = "research://fixture/producer/02".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn reviewer_identity_swap_fails_closed() {
    let mut r = make_receipts();
    r[0].reviewer = "research://fixture/reviewer/02".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn producer_reviewer_role_collision_fails_closed() {
    let mut r = make_receipts();
    r[0].reviewer = r[0].producer.clone();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn unknown_key_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].key = "research://fixture/key/99".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn duplicate_key_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].key = r[1].key.clone();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn foreign_trust_root_even_with_reseal_fails_closed() {
    let mut r = make_receipts();
    r[0].root = "research://fixture/root-unknown".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn zero_epoch_fails_closed() {
    let mut r = make_receipts();
    r[0].epoch = 0;
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn key_rotation_requires_separate_future_review() {
    let mut r = make_receipts();
    r[0].epoch = 2;
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn replayed_sequence_fails_closed() {
    let mut r = make_receipts();
    r[1].sequence = 1;
    reseal(&mut r[1]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn forged_fixture_digest_fails_closed() {
    let mut r = make_receipts();
    r[0].fixture_digest = "0".repeat(64);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn artifact_tamper_fails_closed() {
    let mut r = make_receipts();
    r[0].artifact[0] ^= 1;
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn real_signature_claim_is_never_accepted() {
    let mut r = make_receipts();
    r[0].signature_claim = "SIGNED".into();
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn native_scope_claim_fails_closed() {
    let mut r = make_receipts();
    r[0].scope = "NATIVE".into();
    reseal(&mut r[0]);
    assert_eq!(assess(&r, &[], true, true), Decision::Invalid);
}
#[test]
fn ci_without_approval_stays_blocked() {
    assert_eq!(
        assess(&make_receipts(), &[], true, false),
        Decision::Blocked
    );
}
#[test]
fn approval_without_ci_stays_blocked() {
    assert_eq!(
        assess(&make_receipts(), &[], false, true),
        Decision::Blocked
    );
}
#[test]
fn key_revocation_blocks_even_green_ci() {
    let r = make_receipts();
    let v = revoke(&r[0].key, 1);
    assert_eq!(assess(&r, &[v], true, true), Decision::Blocked);
}
#[test]
fn root_revocation_blocks_all_proposals() {
    let r = make_receipts();
    let v = revoke(FIXTURE_ROOT, 1);
    assert_eq!(assess(&r, &[v], true, true), Decision::Blocked);
}
#[test]
fn unknown_revocation_target_fails_closed() {
    let v = revoke("research://fixture/key/99", 1);
    assert_eq!(
        assess(&make_receipts(), &[v], true, true),
        Decision::Invalid
    );
}
#[test]
fn duplicate_revocations_fail_closed() {
    let r = make_receipts();
    let v = revoke(&r[0].key, 1);
    assert_eq!(assess(&r, &[v.clone(), v], true, true), Decision::Invalid);
}
#[test]
fn forged_revocation_authority_fails_closed() {
    let r = make_receipts();
    let mut v = revoke(&r[0].key, 1);
    v.authority = "unknown".into();
    v.fixture_digest = revocation_commit(&v);
    assert_eq!(assess(&r, &[v], true, true), Decision::Invalid);
}
#[test]
fn stale_revocation_epoch_fails_closed() {
    let r = make_receipts();
    let v = revoke(&r[0].key, 0);
    assert_eq!(assess(&r, &[v], true, true), Decision::Invalid);
}
#[test]
fn forged_revocation_digest_fails_closed() {
    let r = make_receipts();
    let mut v = revoke(&r[0].key, 1);
    v.fixture_digest = "0".repeat(64);
    assert_eq!(assess(&r, &[v], true, true), Decision::Invalid);
}
#[test]
fn empty_revocation_reason_fails_closed() {
    let r = make_receipts();
    let mut v = revoke(&r[0].key, 1);
    v.reason.clear();
    v.fixture_digest = revocation_commit(&v);
    assert_eq!(assess(&r, &[v], true, true), Decision::Invalid);
}
#[test]
fn frozen_r02_smoke_replay_still_passes() {
    let (mut model, _) = Model::bootstrap(
        0x5231_0014,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        2,
    );
    let root = model.root();
    assert!(
        matches!(model.apply(Command::Close { scope: root }), Ok(Receipt::Closed(ref r)) if r.outcome == ScopeOutcome::Succeeded)
    );
    assert_eq!(model.replay_checked(), Ok(()));
}
#[test]
fn no_native_interface_or_manifest_export() {
    for src in [include_str!("../src/lib.rs"), include_str!("../Cargo.toml")] {
        assert!(!src.contains("resource_task_r14"));
        assert!(!src.contains("r14_authentication_registry"));
    }
    assert!(canonical_registry());
}
