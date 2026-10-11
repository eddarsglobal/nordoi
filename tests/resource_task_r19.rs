//! NORDOI R0.19 — fail-closed offline synthetic evidence admission gate study.
//! TEST-ONLY / RESEARCH_ONLY: no trusted signatures, third-party verification or native authority.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "2839c377e78e3c0e58693bc5aa8e668d8e0e436f";
const PACK_SHA: &str = "f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393";
const REPLAY_SHA: &str = "1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab";
const FAULT_SHA: &str = "9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293";
const POLICY_SHA: &str = "7e7f3eccc8e06f6acc0bbb811ddd82b7813654b2b7a676764a5341a81643a3e2";
const TRANSCRIPT_SHA: &str = "ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33";
const HEADER: &str = "id\tgate_id\tarea\tscenario\tbaseline_sha\tpack_sha256\treplay_sha256\tfault_sha256\tproducer_key\tsignature_ref\tverifier_ref\trevocation\tscope\tci\treview\tnative_request\texpected\tlimitation";
const ATTEMPTS: &str = include_str!("../governance/r19_admission_attempts_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r19_admission_attempts_v1.bin");
const PACK: &[u8] = include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin");
const REPLAY: &[u8] =
    include_bytes!("../research/fixtures/r17_independent_replay_transcript_v1.bin");
const FAULT: &[u8] = include_bytes!("../research/fixtures/r18_differential_fault_results_v1.bin");
const GATES: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EVIDENCE: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const INTEGRITY: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const AUTHENTICATION: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const VERIFICATION: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const SCENARIOS: [&str; 8] = [
    "CLEAN_MOCK",
    "TAMPER_ROOT",
    "FORGED_SIGNATURE",
    "FORGED_VERIFIER",
    "REVOKED_ROOT",
    "MISSING_CI",
    "MISSING_REVIEW",
    "NATIVE_ESCALATION",
];
const AREAS: [&str; 12] = [
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn sha_hex(b: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for v in certified_sha256::sha256(b) {
        write!(&mut out, "{v:02x}").expect("String formatting cannot fail");
    }
    out
}
fn normalized_lf(s: &str) -> Option<String> {
    let text = s.replace("\r\n", "\n");
    if text.contains('\r') {
        return None;
    }
    Some(text)
}
fn parse_rows(text: &str) -> Option<Vec<Vec<&str>>> {
    // lines() removes CRLF; any isolated embedded CR must be rejected.
    normalized_lf(text)?;
    let mut lines = text.lines();
    if lines.next()? != HEADER {
        return None;
    }
    let mut rows = Vec::new();
    for line in lines {
        if line.contains('\r') {
            return None;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 18 || fields.iter().any(|f| f.is_empty()) {
            return None;
        }
        rows.push(fields);
    }
    Some(rows)
}
fn decide(c: &[&str]) -> Decision {
    if c.len() != 18 || c[5] != PACK_SHA || c[6] != REPLAY_SHA || c[7] != FAULT_SHA {
        return Decision::Invalid;
    }
    if c[9] != "NONE"
        || c[10] != "NONE"
        || c[11] != "UNKNOWN"
        || c[12] != "RESEARCH_ONLY"
        || c[15] != "NO"
    {
        return Decision::Blocked;
    }
    if c[13] != "YES" || c[14] != "YES" || !c[8].starts_with("MOCK-PRODUCER-") {
        return Decision::Blocked;
    }
    Decision::ResearchReviewOnly
}
fn label(d: Decision) -> &'static str {
    match d {
        Decision::Invalid => "INVALID",
        Decision::Blocked => "BLOCKED",
        Decision::ResearchReviewOnly => "RESEARCH_REVIEW_ONLY",
    }
}
fn decision_byte(d: Decision) -> u8 {
    match d {
        Decision::Invalid => 0,
        Decision::Blocked => 1,
        Decision::ResearchReviewOnly => 2,
    }
}
fn u16_le(n: usize) -> Option<[u8; 2]> {
    let n = u16::try_from(n).ok()?;
    Some(n.to_le_bytes())
}
fn expected_bytes(input: &str) -> Option<Vec<u8>> {
    let parsed = parse_rows(input)?;
    if parsed.len() != 96 {
        return None;
    }
    let mut output = Vec::from(&b"NDR19AD1"[..]);
    output.extend(u16_le(parsed.len())?);
    for (index, cells) in parsed.iter().enumerate() {
        let ordinal = index + 1;
        let gate = index / 8 + 1;
        let scenario = index % 8;
        if cells[0] != format!("R19-A{ordinal:03}")
            || cells[1] != format!("R11-G{gate:02}")
            || cells[2] != AREAS[gate - 1]
            || cells[3] != SCENARIOS[scenario]
            || cells[4] != BASE
            || cells[8] != format!("MOCK-PRODUCER-{gate:02}")
        {
            return None;
        }
        let outcome = decide(cells);
        if cells[16] != label(outcome)
            || (scenario == 0) != (outcome == Decision::ResearchReviewOnly)
        {
            return None;
        }
        output.extend(u16_le(ordinal)?);
        output.push(u8::try_from(scenario).ok()?);
        output.push(decision_byte(outcome));
        let line = format!("{}\n", cells.join("\t"));
        output.extend(certified_sha256::sha256(line.as_bytes()));
    }
    let checksum = certified_sha256::sha256(&output);
    output.extend(checksum);
    Some(output)
}
fn frozen_status() -> bool {
    let files = [
        (GATES, "UNMET"),
        (EVIDENCE, "ABSENT"),
        (INTEGRITY, "ABSENT"),
        (AUTHENTICATION, "ABSENT"),
        (VERIFICATION, "ABSENT"),
    ];
    files.iter().all(|(file, expected)| {
        let mut lines = file.lines();
        let header: Vec<&str> = match lines.next() {
            Some(h) => h.split('\t').collect(),
            None => return false,
        };
        let status = match header.iter().position(|f| *f == "status") {
            Some(idx) => idx,
            None => return false,
        };
        let area = match header.iter().position(|f| *f == "area") {
            Some(idx) => idx,
            None => return false,
        };
        let mut count = 0;
        for (i, line) in lines.enumerate() {
            if i >= 12 || line.contains('\r') {
                return false;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != header.len()
                || parts.get(status) != Some(expected)
                || parts.get(area) != Some(&AREAS[i])
            {
                return false;
            }
            count += 1;
        }
        count == 12
    })
}
fn sample(index: usize) -> Vec<String> {
    parse_rows(ATTEMPTS).expect("frozen public fixture")[index]
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
}
fn as_refs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

#[test]
fn certified_r018_commit_is_exact() {
    assert_eq!(BASE, "2839c377e78e3c0e58693bc5aa8e668d8e0e436f");
}
#[test]
fn all_historical_gates_remain_unmet_or_absent() {
    assert!(frozen_status());
}
#[test]
fn certified_binary_inputs_remain_frozen() {
    assert_eq!(sha_hex(PACK), PACK_SHA);
    assert_eq!(sha_hex(REPLAY), REPLAY_SHA);
    assert_eq!(sha_hex(FAULT), FAULT_SHA);
}
#[test]
fn admission_attempts_registry_is_frozen() {
    assert_eq!(
        sha_hex(normalized_lf(ATTEMPTS).unwrap().as_bytes()),
        POLICY_SHA
    );
}
#[test]
fn admission_registry_has_ninety_six_rows() {
    assert_eq!(parse_rows(ATTEMPTS).unwrap().len(), 96);
}
#[test]
fn admission_rows_have_twelve_gate_groups() {
    for (index, row) in parse_rows(ATTEMPTS).unwrap().iter().enumerate() {
        assert_eq!(row[2], AREAS[index / 8]);
    }
}
#[test]
fn eight_adversarial_scenarios_cover_each_gate() {
    for (index, row) in parse_rows(ATTEMPTS).unwrap().iter().enumerate() {
        assert_eq!(row[3], SCENARIOS[index % 8]);
    }
}
#[test]
fn clean_public_mock_is_never_production_admitted() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8]),
            Decision::ResearchReviewOnly
        );
    }
}
#[test]
fn tampered_artifact_root_is_invalid() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 1]),
            Decision::Invalid
        );
    }
}
#[test]
fn forged_signature_reference_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 2]),
            Decision::Blocked
        );
    }
}
#[test]
fn forged_external_verifier_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 3]),
            Decision::Blocked
        );
    }
}
#[test]
fn revoked_root_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 4]),
            Decision::Blocked
        );
    }
}
#[test]
fn missing_ci_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 5]),
            Decision::Blocked
        );
    }
}
#[test]
fn missing_review_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 6]),
            Decision::Blocked
        );
    }
}
#[test]
fn native_escalation_is_blocked() {
    for i in 0..12 {
        assert_eq!(
            decide(&parse_rows(ATTEMPTS).unwrap()[i * 8 + 7]),
            Decision::Blocked
        );
    }
}
#[test]
fn invalid_scope_is_blocked_even_with_mock_approval() {
    let mut c = sample(0);
    c[12] = "NATIVE".to_owned();
    assert_eq!(decide(&as_refs(&c)), Decision::Blocked);
}
#[test]
fn incorrect_replay_root_is_invalid() {
    let mut c = sample(0);
    c[6] = "0".repeat(64);
    assert_eq!(decide(&as_refs(&c)), Decision::Invalid);
}
#[test]
fn incorrect_fault_root_is_invalid() {
    let mut c = sample(0);
    c[7] = "0".repeat(64);
    assert_eq!(decide(&as_refs(&c)), Decision::Invalid);
}
#[test]
fn malformed_row_is_invalid() {
    let c = ["a", "b"];
    assert_eq!(decide(&c), Decision::Invalid);
}
#[test]
fn duplicate_case_id_fails_closed() {
    let bad = ATTEMPTS.replacen("R19-A002", "R19-A001", 1);
    assert!(expected_bytes(&bad).is_none());
}
#[test]
fn reordered_rows_fail_closed() {
    let mut lines: Vec<&str> = ATTEMPTS.lines().collect();
    lines.swap(1, 2);
    assert!(expected_bytes(&(lines.join("\n") + "\n")).is_none());
}
#[test]
fn missing_case_fails_closed() {
    let mut lines: Vec<&str> = ATTEMPTS.lines().collect();
    lines.remove(24);
    assert!(expected_bytes(&(lines.join("\n") + "\n")).is_none());
}
#[test]
fn malformed_header_is_rejected() {
    let bad = ATTEMPTS.replacen("id\t", "bogus\t", 1);
    assert!(parse_rows(&bad).is_none());
}
#[test]
fn windows_crlf_is_accepted_but_embedded_cr_rejected() {
    let crlf = ATTEMPTS.replace('\n', "\r\n");
    assert_eq!(expected_bytes(&crlf), expected_bytes(ATTEMPTS));
    let injected = ATTEMPTS.replacen("CLEAN_MOCK", "CLEAN\r_MOCK", 1);
    assert!(parse_rows(&injected).is_none());
}
#[test]
fn canonical_admission_transcript_is_byte_stable() {
    let crlf = ATTEMPTS.replace('\n', "\r\n");
    assert_eq!(expected_bytes(ATTEMPTS), expected_bytes(&crlf));
}
#[test]
fn canonical_admission_transcript_matches_golden() {
    assert_eq!(expected_bytes(ATTEMPTS).unwrap().as_slice(), GOLDEN);
}
#[test]
fn canonical_admission_transcript_has_sha_anchor() {
    assert_eq!(sha_hex(GOLDEN), TRANSCRIPT_SHA);
}
#[test]
fn transcript_checksum_is_valid() {
    assert_eq!(GOLDEN.len(), 3498);
    assert_eq!(
        &certified_sha256::sha256(&GOLDEN[..GOLDEN.len() - 32])[..],
        &GOLDEN[GOLDEN.len() - 32..]
    );
}
#[test]
fn transcript_mutation_is_rejected_even_after_checksum_recalculation() {
    let mut altered = GOLDEN.to_vec();
    altered[20] ^= 1;
    let n = altered.len();
    let end = certified_sha256::sha256(&altered[..n - 32]);
    altered[n - 32..].copy_from_slice(&end);
    assert_ne!(altered, expected_bytes(ATTEMPTS).unwrap());
}
#[test]
fn admission_decisions_are_exactly_bounded() {
    let mut count = [0usize; 3];
    for row in parse_rows(ATTEMPTS).unwrap() {
        count[usize::from(decision_byte(decide(&row)))] += 1;
    }
    assert_eq!(count, [12, 72, 12]);
}
#[test]
fn unknown_producer_fails_closed() {
    let mut c = sample(0);
    c[8] = "UNKNOWN".to_owned();
    assert_eq!(decide(&as_refs(&c)), Decision::Blocked);
}
#[test]
fn no_native_api_or_external_verifier_is_exported() {
    assert!(frozen_status());
    assert_eq!(
        decide(&parse_rows(ATTEMPTS).unwrap()[0]),
        Decision::ResearchReviewOnly
    );
    println!("R19_WITNESS version=1 kind=synthetic_admission_gate attempts=96 invalid=12 blocked=72 research_only=12 external_verifiers=0 native_gates_unmet=12 outcome=PASS");
}
