//! R0.13 TEST-ONLY: SHA-256 byte integrity of synthetic proposals, NOT trust or production authority.
//! Includes the certified SHA-256 implementation (and its two FIPS test vectors) unchanged.

#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;
#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod frozen_reference;

use frozen_reference::{Command, Model, Receipt, ScopeBudget, ScopeOutcome};
use std::collections::BTreeSet;

const BASE_SHA: &str = "851d978c36f4c12fc16c115e89a7774e33ce361b";
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
const CANONICAL_HEADER: &str = "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation";

fn hex_digest(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for byte in certified_sha256::sha256(bytes) {
        use std::fmt::Write;
        write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
    }
    result
}

fn length_bound_write(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u64).to_be_bytes());
    buf.extend_from_slice(data);
}

fn chain_commit(link: &Link) -> String {
    let mut data = b"NORDOI-R013-RESEARCH-CHAIN-v1".to_vec();
    for field in [
        link.id.as_bytes(),
        link.gate.as_bytes(),
        link.evidence.as_bytes(),
        link.area.as_bytes(),
        link.baseline.as_bytes(),
        link.status.as_bytes(),
        link.scope.as_bytes(),
        link.payload_sha.as_bytes(),
        link.previous.as_bytes(),
        link.producer.as_bytes(),
        link.reviewer.as_bytes(),
        link.signature.as_bytes(),
        link.trust_root.as_bytes(),
    ] {
        length_bound_write(&mut data, field);
    }
    hex_digest(&data)
}

#[derive(Clone, Debug)]
struct Link {
    id: String,
    gate: String,
    evidence: String,
    area: String,
    baseline: String,
    status: String,
    scope: String,
    payload: Vec<u8>,
    payload_sha: String,
    previous: String,
    chain_sha: String,
    producer: String,
    reviewer: String,
    signature: String,
    trust_root: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Assessment {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn parse_registry(src: &str) -> Result<Vec<Vec<&str>>, &'static str> {
    let mut lines = src.lines();
    if lines.next() != Some(CANONICAL_HEADER) {
        return Err("unrecognized header");
    }
    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() || line.contains('\r') {
            return Err("empty or CRLF row");
        }
        let cols: Vec<_> = line.split('\t').collect();
        if cols.len() != 16 || cols.iter().any(|value| value.trim().is_empty()) {
            return Err("malformed registry row");
        }
        rows.push(cols);
    }
    Ok(rows)
}

fn frozen_input_boundary() -> bool {
    let mut gates = R11_GATES.lines();
    if gates.next() != Some("id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation") {
        return false;
    }
    let gate_rows: Vec<_> = gates.collect();
    let mut r12 = R12_LEDGER.lines();
    if r12.next() != Some("id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation") {
        return false;
    }
    let evidence_rows: Vec<_> = r12.collect();
    if gate_rows.len() != COUNT || evidence_rows.len() != COUNT {
        return false;
    }
    (0..COUNT).all(|i| {
        let g: Vec<_> = gate_rows[i].split('\t').collect();
        let e: Vec<_> = evidence_rows[i].split('\t').collect();
        g.len() == 6
            && e.len() == 9
            && g[0] == format!("R11-G{:02}", i + 1)
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
    })
}

fn canonical_absent_registry() -> bool {
    if !frozen_input_boundary() {
        return false;
    }
    let Ok(rows) = parse_registry(R13_LEDGER) else {
        return false;
    };
    rows.len() == COUNT
        && rows.iter().enumerate().all(|(i, row)| {
            row[0] == format!("R13-L{:02}", i + 1)
                && row[1] == format!("R11-G{:02}", i + 1)
                && row[2] == format!("R12-E{:02}", i + 1)
                && row[3] == AREAS[i]
                && row[4] == BASE_SHA
                && row[5] == "ABSENT"
                && row[6] == "RESEARCH_ONLY"
                && row[7..15].iter().all(|&item| item == "NONE")
                && row[15].len() > 65
        })
}

fn synthetic_chain() -> Vec<Link> {
    let mut chain = Vec::new();
    let mut previous = "GENESIS".to_owned();
    for (index, &area) in AREAS.iter().enumerate() {
        let payload = format!("R013-SYNTHETIC-NOT-NATIVE:{:02}:{area}", index + 1).into_bytes();
        let mut link = Link {
            id: format!("R13-L{:02}", index + 1),
            gate: format!("R11-G{:02}", index + 1),
            evidence: format!("R12-E{:02}", index + 1),
            area: area.to_owned(),
            baseline: BASE_SHA.to_owned(),
            status: "PROPOSED".to_owned(),
            scope: "RESEARCH_ONLY".to_owned(),
            payload_sha: hex_digest(&payload),
            payload,
            previous: previous.clone(),
            chain_sha: String::new(),
            producer: format!("research://synthetic/producer/{:02}", index + 1),
            reviewer: format!("research://synthetic/reviewer/{:02}", index + 1),
            signature: "NONE".to_owned(),
            trust_root: "NONE".to_owned(),
        };
        link.chain_sha = chain_commit(&link);
        previous = link.chain_sha.clone();
        chain.push(link);
    }
    chain
}

fn assess(chain: &[Link], revoked: &[&str], ci: bool, approval: bool) -> Assessment {
    if !canonical_absent_registry() {
        return Assessment::Invalid;
    }
    if chain.is_empty() && revoked.is_empty() {
        return Assessment::Blocked;
    }
    if chain.len() != COUNT {
        return Assessment::Invalid;
    }
    let mut seen_ids = BTreeSet::new();
    let mut seen_payloads = BTreeSet::new();
    let mut previous = "GENESIS".to_owned();
    for (index, link) in chain.iter().enumerate() {
        if link.id != format!("R13-L{:02}", index + 1)
            || link.gate != format!("R11-G{:02}", index + 1)
            || link.evidence != format!("R12-E{:02}", index + 1)
            || link.area != AREAS[index]
            || link.baseline != BASE_SHA
            || link.status != "PROPOSED"
            || link.scope != "RESEARCH_ONLY"
            || link.payload.is_empty()
            || link.payload.len() > 2048
            || link.payload_sha != hex_digest(&link.payload)
            || link.previous != previous
            || link.chain_sha != chain_commit(link)
            || !link.producer.starts_with("research://synthetic/producer/")
            || !link.reviewer.starts_with("research://synthetic/reviewer/")
            || link.producer == link.reviewer
            || link.signature != "NONE"
            || link.trust_root != "NONE"
            || !seen_ids.insert(link.id.as_str())
            || !seen_payloads.insert(link.payload_sha.as_str())
        {
            return Assessment::Invalid;
        }
        previous = link.chain_sha.clone();
    }
    let mut revoked_set = BTreeSet::new();
    for &item in revoked {
        if !revoked_set.insert(item) || !seen_ids.contains(item) {
            return Assessment::Invalid;
        }
    }
    if !revoked_set.is_empty() || !ci || !approval {
        return Assessment::Blocked;
    }
    Assessment::ResearchReviewOnly
}

fn reseal_from(chain: &mut [Link], start: usize) {
    let mut previous = if start == 0 {
        "GENESIS".to_owned()
    } else {
        chain[start - 1].chain_sha.clone()
    };
    for link in chain.iter_mut().skip(start) {
        link.previous = previous;
        link.chain_sha = chain_commit(link);
        previous = link.chain_sha.clone();
    }
}

#[test]
fn certified_r012_baseline_is_exact() {
    assert_eq!(BASE_SHA, "851d978c36f4c12fc16c115e89a7774e33ce361b");
}
#[test]
fn r011_native_gates_remain_unmet() {
    assert!(frozen_input_boundary());
}
#[test]
fn r012_evidence_remains_absent() {
    assert!(R12_LEDGER
        .lines()
        .skip(1)
        .all(|l| l.contains("\tABSENT\tRESEARCH_ONLY\tNONE\tNONE\t")));
}
#[test]
fn canonical_r013_registry_has_twelve_records() {
    assert_eq!(parse_registry(R13_LEDGER).unwrap().len(), COUNT);
}
#[test]
fn canonical_r013_records_are_absent() {
    assert!(canonical_absent_registry());
}
#[test]
fn no_canonical_artifact_signature_or_trust_root_is_claimed() {
    assert!(parse_registry(R13_LEDGER)
        .unwrap()
        .iter()
        .all(|r| r[7..15].iter().all(|v| *v == "NONE")));
}
#[test]
fn sha256_abc_matches_reference_vector_without_new_dependency() {
    assert_eq!(
        hex_digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn synthetic_chain_has_twelve_sha256_digests() {
    let links = synthetic_chain();
    assert_eq!(links.len(), COUNT);
    assert!(links
        .iter()
        .all(|l| l.payload_sha.len() == 64 && l.chain_sha.len() == 64));
}
#[test]
fn synthetic_chain_matches_commit_transcript() {
    let links = synthetic_chain();
    assert!(links.iter().all(|l| l.chain_sha == chain_commit(l)));
}
#[test]
fn full_synthetic_chain_is_only_research_review() {
    assert_eq!(
        assess(&synthetic_chain(), &[], true, true),
        Assessment::ResearchReviewOnly
    );
    println!("R13_WITNESS version=1 kind=synthetic_sha256_chain links=12 native_gates_unmet=12 evidence_absent=12 decision=RESEARCH_REVIEW_ONLY outcome=PASS");
}
#[test]
fn synthetic_ci_without_approval_blocks() {
    assert_eq!(
        assess(&synthetic_chain(), &[], true, false),
        Assessment::Blocked
    );
}
#[test]
fn synthetic_approval_without_ci_blocks() {
    assert_eq!(
        assess(&synthetic_chain(), &[], false, true),
        Assessment::Blocked
    );
}
#[test]
fn empty_chain_remains_blocked() {
    assert_eq!(assess(&[], &[], true, true), Assessment::Blocked);
}
#[test]
fn missing_link_fails_closed() {
    let mut c = synthetic_chain();
    c.pop();
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn reordered_links_fail_closed() {
    let mut c = synthetic_chain();
    c.swap(0, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn duplicate_sequence_identity_fails_closed() {
    let mut c = synthetic_chain();
    let dupe = c[0].id.clone();
    c[1].id = dupe;
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn duplicate_payload_digest_fails_closed() {
    let mut c = synthetic_chain();
    let dupe = c[0].payload.clone();
    c[1].payload = dupe;
    c[1].payload_sha = hex_digest(&c[1].payload);
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn changed_gate_even_with_reseal_fails_closed() {
    let mut c = synthetic_chain();
    c[1].gate = "R11-G12".into();
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn changed_evidence_id_even_with_reseal_fails_closed() {
    let mut c = synthetic_chain();
    c[1].evidence = "R12-E12".into();
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn changed_area_even_with_reseal_fails_closed() {
    let mut c = synthetic_chain();
    c[1].area = "FAKE".into();
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn changed_baseline_sha_fails_closed() {
    let mut c = synthetic_chain();
    c[1].baseline = "0".repeat(40);
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn payload_byte_tamper_fails_digest_check() {
    let mut c = synthetic_chain();
    c[1].payload[0] ^= 1;
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn forged_payload_digest_fails_closed() {
    let mut c = synthetic_chain();
    c[1].payload_sha = "0".repeat(64);
    reseal_from(&mut c, 1);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn broken_previous_chain_digest_fails_closed() {
    let mut c = synthetic_chain();
    // Forge a wrong back-reference and recompute the altered link digest WITHOUT
    // repairing that reference. Reseal later links only, preserving the break.
    c[1].previous = "GENESIS".into();
    let forged_commit = chain_commit(&c[1]);
    c[1].chain_sha = forged_commit;
    reseal_from(&mut c, 2);
    assert_ne!(c[1].previous, c[0].chain_sha);
    // All per-link digests are now internally consistent: the sole fault is
    // the broken connection between the first and second links.
    assert!(c.iter().all(|link| link.chain_sha == chain_commit(link)));
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn forged_link_commit_digest_fails_closed() {
    let mut c = synthetic_chain();
    c[1].chain_sha = "0".repeat(64);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn synthetic_signature_claim_fails_closed() {
    let mut c = synthetic_chain();
    c[0].signature = "signed-by-untrusted-key".into();
    reseal_from(&mut c, 0);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn claimed_trust_root_fails_closed() {
    let mut c = synthetic_chain();
    c[0].trust_root = "root://unverified".into();
    reseal_from(&mut c, 0);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn unauthenticated_foreign_producer_fails_closed() {
    let mut c = synthetic_chain();
    c[0].producer = "https://unknown.example".into();
    reseal_from(&mut c, 0);
    assert_eq!(assess(&c, &[], true, true), Assessment::Invalid);
}
#[test]
fn unknown_revocation_fails_closed() {
    assert_eq!(
        assess(&synthetic_chain(), &["R13-L99"], true, true),
        Assessment::Invalid
    );
}
#[test]
fn revoked_proposal_blocks_even_with_green_ci() {
    assert_eq!(
        assess(&synthetic_chain(), &["R13-L03"], true, true),
        Assessment::Blocked
    );
}
#[test]
fn duplicate_revocation_fails_closed() {
    assert_eq!(
        assess(&synthetic_chain(), &["R13-L03", "R13-L03"], true, true),
        Assessment::Invalid
    );
}
#[test]
fn malformed_chain_registry_is_rejected() {
    assert!(parse_registry("invalid\n").is_err());
    assert!(parse_registry(&R13_LEDGER.replacen("\tNONE\tNONE\t", "\tNONE\t", 1)).is_err());
}
#[test]
fn frozen_r02_reference_replay_still_passes() {
    let (mut model, _) = Model::bootstrap(
        0x5231_0013,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        2,
    );
    let root = model.root();
    assert!(
        matches!(model.apply(Command::Close{scope:root}),Ok(Receipt::Closed(ref r)) if r.outcome == ScopeOutcome::Succeeded)
    );
    assert_eq!(model.replay_checked(), Ok(()));
}
#[test]
fn no_r013_native_or_manifest_export() {
    for src in [include_str!("../src/lib.rs"), include_str!("../Cargo.toml")] {
        assert!(!src.contains("resource_task_r13"));
        assert!(!src.contains("r13_integrity_chain"));
    }
    assert!(canonical_absent_registry());
}
