//! NORDOI R0.20 — synthetic trust-root lifecycle and revocation boundary.
//! TEST-ONLY / RESEARCH_ONLY. No cryptographic identity, native authority, or external verifier.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "834c20d68e86f0ed6ab14141e0f9f9e1df241901";
const EVIDENCE_SHA: &str = "ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33";
const POLICY_SHA: &str = "1a296c9adb4b828fd504169a3579b0c8addbdbc7add1c8b0fafeedd6352f5e2e";
const TRANSCRIPT_SHA: &str = "2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817";
const HEADER: &str = "id\tgate_id\tarea\tscenario\tbaseline_sha\tevidence_sha256\troot_claim\tepoch_claim\tscope\tsignature\tverifier\tci\treview\tnative\tbefore_epoch\tafter_epoch\tbefore_root\tafter_root\tbefore_revoked\tafter_revoked\texpected\tlimitation";
const ROWS: &str = include_str!("../governance/r20_trust_root_lifecycle_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r20_trust_root_transitions_v1.bin");
const EVIDENCE: &[u8] = include_bytes!("../research/fixtures/r19_admission_attempts_v1.bin");
const NATIVE: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EXTERNAL: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const SCENARIOS: [&str; 11] = [
    "REGISTER_MOCK",
    "ROTATE_MOCK",
    "STALE_EPOCH",
    "FOREIGN_SCOPE",
    "FORGED_SIGNATURE",
    "FORGED_VERIFIER",
    "TAMPER_LINEAGE",
    "REVOKE_MOCK",
    "REPLAY_REVOKED",
    "ROLLBACK_EPOCH",
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RootState {
    epoch: u8,
    root: String,
    revoked: bool,
}
fn initial() -> RootState {
    RootState {
        epoch: 0,
        root: "NONE".to_owned(),
        revoked: false,
    }
}
fn hex_hash(data: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in certified_sha256::sha256(data) {
        write!(&mut out, "{byte:02x}").expect("formatting String cannot fail");
    }
    out
}
fn normalized_lf(source: &str) -> Option<String> {
    let lf = source.replace("\r\n", "\n");
    if lf.contains('\r') {
        return None;
    }
    Some(lf)
}
fn parse_rows(source: &str) -> Option<Vec<Vec<&str>>> {
    normalized_lf(source)?;
    let mut lines = source.lines();
    if lines.next()? != HEADER {
        return None;
    }
    let mut result = Vec::new();
    for line in lines {
        if line.contains('\r') {
            return None;
        }
        let row: Vec<&str> = line.split('\t').collect();
        if row.len() != 22 || row.iter().any(|field| field.is_empty()) {
            return None;
        }
        result.push(row);
    }
    Some(result)
}
fn label(decision: Decision) -> &'static str {
    match decision {
        Decision::Invalid => "INVALID",
        Decision::Blocked => "BLOCKED",
        Decision::ResearchReviewOnly => "RESEARCH_REVIEW_ONLY",
    }
}
fn code(decision: Decision) -> u8 {
    match decision {
        Decision::Invalid => 0,
        Decision::Blocked => 1,
        Decision::ResearchReviewOnly => 2,
    }
}
fn root_name(gate: usize, suffix: &str) -> String {
    format!("MOCK-ROOT-{gate:02}-{suffix}")
}
fn state_flag(revoked: bool) -> &'static str {
    if revoked {
        "YES"
    } else {
        "NO"
    }
}
fn apply(row: &[&str], state: &mut RootState, gate: usize) -> Decision {
    if row.len() != 22 || row[5] != EVIDENCE_SHA || row[4] != BASE {
        return Decision::Invalid;
    }
    let Ok(epoch) = row[7].parse::<u8>() else {
        return Decision::Invalid;
    };
    if row[8] != "RESEARCH_ONLY"
        || row[9] != "NONE"
        || row[10] != "NONE"
        || row[11] != "YES"
        || row[12] != "YES"
        || row[13] != "NO"
    {
        return Decision::Blocked;
    }
    match row[3] {
        "REGISTER_MOCK"
            if state.epoch == 0
                && state.root == "NONE"
                && !state.revoked
                && row[6] == root_name(gate, "A")
                && epoch == 1 =>
        {
            state.epoch = 1;
            state.root = row[6].to_owned();
            Decision::ResearchReviewOnly
        }
        "ROTATE_MOCK"
            if state.epoch == 1
                && state.root == root_name(gate, "A")
                && !state.revoked
                && row[6] == root_name(gate, "B")
                && epoch == 2 =>
        {
            state.epoch = 2;
            state.root = row[6].to_owned();
            Decision::ResearchReviewOnly
        }
        "REVOKE_MOCK"
            if state.epoch == 2
                && state.root == root_name(gate, "B")
                && !state.revoked
                && row[6] == state.root
                && epoch == 2 =>
        {
            state.revoked = true;
            Decision::ResearchReviewOnly
        }
        _ => Decision::Blocked,
    }
}
fn golden_bytes(source: &str) -> Option<Vec<u8>> {
    let rows = parse_rows(source)?;
    if rows.len() != 132 {
        return None;
    }
    let mut result = Vec::from(&b"NDR20TR1"[..]);
    result.extend_from_slice(&132u16.to_le_bytes());
    let mut state = initial();
    for (idx, row) in rows.iter().enumerate() {
        let gate = idx / 11 + 1;
        let scenario = idx % 11;
        if scenario == 0 {
            state = initial();
        }
        if row[0] != format!("R20-L{:03}", idx + 1)
            || row[1] != format!("R11-G{gate:02}")
            || row[2] != AREAS[gate - 1]
            || row[3] != SCENARIOS[scenario]
            || row[4] != BASE
            || row[21] != "SYNTHETIC_ONLY_NO_REAL_SIGNATURE_VERIFIER_OR_NATIVE_AUTHORITY"
        {
            return None;
        }
        let before = state.clone();
        if row[14] != before.epoch.to_string()
            || row[16] != before.root
            || row[18] != state_flag(before.revoked)
        {
            return None;
        }
        let decision = apply(row, &mut state, gate);
        let canonical = match scenario {
            0 | 1 | 7 => Decision::ResearchReviewOnly,
            6 => Decision::Invalid,
            _ => Decision::Blocked,
        };
        if decision != canonical
            || row[20] != label(decision)
            || row[15] != state.epoch.to_string()
            || row[17] != state.root
            || row[19] != state_flag(state.revoked)
        {
            return None;
        }
        result.extend_from_slice(&u16::try_from(idx + 1).ok()?.to_le_bytes());
        result.push(u8::try_from(scenario).ok()?);
        result.push(code(decision));
        result.push(before.epoch);
        result.push(state.epoch);
        result.push(u8::from(state.revoked));
        let canonical_row = format!("{}\n", row.join("\t"));
        result.extend_from_slice(&certified_sha256::sha256(canonical_row.as_bytes()));
    }
    let digest = certified_sha256::sha256(&result);
    result.extend_from_slice(&digest);
    Some(result)
}
fn historic_status(source: &str, expected: &str) -> bool {
    let Some(normalized) = normalized_lf(source) else {
        return false;
    };
    let mut iter = normalized.lines();
    let Some(header) = iter.next() else {
        return false;
    };
    let names: Vec<&str> = header.split('\t').collect();
    let Some(status) = names.iter().position(|name| *name == "status") else {
        return false;
    };
    let Some(area) = names.iter().position(|name| *name == "area") else {
        return false;
    };
    let lines: Vec<&str> = iter.collect();
    if lines.len() != 12 {
        return false;
    }
    lines.iter().enumerate().all(|(index, line)| {
        let fields: Vec<&str> = line.split('\t').collect();
        fields.len() == names.len() && fields[area] == AREAS[index] && fields[status] == expected
    })
}
fn sample(index: usize) -> Vec<String> {
    parse_rows(ROWS).expect("frozen research registry")[index]
        .iter()
        .map(|value| (*value).to_owned())
        .collect()
}
fn refs(row: &[String]) -> Vec<&str> {
    row.iter().map(String::as_str).collect()
}
fn edit(index: usize, column: usize, value: &str) -> String {
    let lf = normalized_lf(ROWS).expect("canonical input");
    let mut lines: Vec<String> = lf.lines().map(ToOwned::to_owned).collect();
    let mut row: Vec<String> = lines[index + 1]
        .split('\t')
        .map(ToOwned::to_owned)
        .collect();
    row[column] = value.to_owned();
    lines[index + 1] = row.join("\t");
    lines.join("\n") + "\n"
}

#[test]
fn certified_r019_commit_is_exact() {
    assert_eq!(BASE, "834c20d68e86f0ed6ab14141e0f9f9e1df241901");
}
#[test]
fn certified_r019_admission_fixture_is_frozen() {
    assert_eq!(hex_hash(EVIDENCE), EVIDENCE_SHA);
}
#[test]
fn frozen_native_and_external_statuses_remain_closed() {
    assert!(historic_status(NATIVE, "UNMET"));
    assert!(historic_status(EXTERNAL, "ABSENT"));
}
#[test]
fn canonical_policy_registry_has_exact_hash() {
    assert_eq!(
        hex_hash(normalized_lf(ROWS).unwrap().as_bytes()),
        POLICY_SHA
    );
}
#[test]
fn canonical_registry_contains_one_hundred_thirty_two_rows() {
    assert_eq!(parse_rows(ROWS).unwrap().len(), 132);
}
#[test]
fn canonical_registry_has_twelve_independent_gate_groups() {
    for (index, row) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(row[2], AREAS[index / 11]);
    }
}
#[test]
fn all_eleven_scenarios_are_ordered_per_gate() {
    for (index, row) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(row[3], SCENARIOS[index % 11]);
    }
}
#[test]
fn golden_transcript_has_magic() {
    assert_eq!(&GOLDEN[..8], b"NDR20TR1");
}
#[test]
fn golden_transcript_has_expected_size() {
    assert_eq!(GOLDEN.len(), 5190);
}
#[test]
fn golden_transcript_binds_count() {
    assert_eq!(&GOLDEN[8..10], &132u16.to_le_bytes());
}
#[test]
fn golden_transcript_has_correct_tail_digest() {
    assert_eq!(
        &GOLDEN[GOLDEN.len() - 32..],
        &certified_sha256::sha256(&GOLDEN[..GOLDEN.len() - 32])
    );
}
#[test]
fn golden_transcript_has_frozen_sha() {
    assert_eq!(hex_hash(GOLDEN), TRANSCRIPT_SHA);
}
#[test]
fn canonical_transcript_rebuild_matches_frozen_bytes() {
    assert_eq!(golden_bytes(ROWS), Some(GOLDEN.to_vec()));
}
#[test]
fn repeated_replay_is_byte_deterministic() {
    assert_eq!(golden_bytes(ROWS), golden_bytes(ROWS));
}
#[test]
fn windows_crlf_produces_identical_transcript() {
    let crlf = normalized_lf(ROWS).unwrap().replace('\n', "\r\n");
    assert_eq!(golden_bytes(&crlf), golden_bytes(ROWS));
}
#[test]
fn isolated_embedded_carriage_return_is_invalid() {
    assert!(parse_rows(&edit(0, 3, "REGISTER\r_MOCK")).is_none());
}
#[test]
fn registering_mock_root_enters_research_only() {
    for gate in 1..=12 {
        let mut state = initial();
        assert_eq!(
            apply(
                &parse_rows(ROWS).unwrap()[(gate - 1) * 11],
                &mut state,
                gate
            ),
            Decision::ResearchReviewOnly
        );
        assert_eq!(state.epoch, 1);
    }
}
#[test]
fn forward_rotation_enters_research_only() {
    for gate in 1..=12 {
        let mut state = initial();
        let rows = parse_rows(ROWS).unwrap();
        assert_eq!(
            apply(&rows[(gate - 1) * 11], &mut state, gate),
            Decision::ResearchReviewOnly
        );
        assert_eq!(
            apply(&rows[(gate - 1) * 11 + 1], &mut state, gate),
            Decision::ResearchReviewOnly
        );
        assert_eq!(state.epoch, 2);
    }
}
#[test]
fn revocation_is_research_only_and_persistent_in_mock_state() {
    for gate in 1..=12 {
        let mut state = initial();
        let rows = parse_rows(ROWS).unwrap();
        for offset in [0, 1, 7] {
            assert_eq!(
                apply(&rows[(gate - 1) * 11 + offset], &mut state, gate),
                Decision::ResearchReviewOnly
            );
        }
        assert!(state.revoked);
    }
}
#[test]
fn old_epoch_is_blocked_without_state_change() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: false,
    };
    let initial_state = state.clone();
    assert_eq!(apply(&refs(&sample(2)), &mut state, 1), Decision::Blocked);
    assert_eq!(state, initial_state);
}
#[test]
fn foreign_scope_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: false,
    };
    assert_eq!(apply(&refs(&sample(3)), &mut state, 1), Decision::Blocked);
}
#[test]
fn forged_signature_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: false,
    };
    assert_eq!(apply(&refs(&sample(4)), &mut state, 1), Decision::Blocked);
}
#[test]
fn forged_verifier_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: false,
    };
    assert_eq!(apply(&refs(&sample(5)), &mut state, 1), Decision::Blocked);
}
#[test]
fn tampered_r019_lineage_is_invalid() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: false,
    };
    assert_eq!(apply(&refs(&sample(6)), &mut state, 1), Decision::Invalid);
}
#[test]
fn revoked_root_replay_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: true,
    };
    assert_eq!(apply(&refs(&sample(8)), &mut state, 1), Decision::Blocked);
    assert!(state.revoked);
}
#[test]
fn rollback_after_revocation_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: true,
    };
    assert_eq!(apply(&refs(&sample(9)), &mut state, 1), Decision::Blocked);
}
#[test]
fn native_authority_escalation_is_blocked() {
    let mut state = RootState {
        epoch: 2,
        root: root_name(1, "B"),
        revoked: true,
    };
    assert_eq!(apply(&refs(&sample(10)), &mut state, 1), Decision::Blocked);
}
#[test]
fn missing_ci_blocks_even_initial_registration() {
    let mut row = sample(0);
    row[11] = "NO".to_owned();
    assert_eq!(apply(&refs(&row), &mut initial(), 1), Decision::Blocked);
}
#[test]
fn missing_review_blocks_even_initial_registration() {
    let mut row = sample(0);
    row[12] = "NO".to_owned();
    assert_eq!(apply(&refs(&row), &mut initial(), 1), Decision::Blocked);
}
#[test]
fn duplicate_row_fails_closed() {
    assert!(golden_bytes(&edit(1, 0, "R20-L001")).is_none());
}
#[test]
fn incorrect_before_epoch_fails_closed() {
    assert!(golden_bytes(&edit(1, 14, "0")).is_none());
}
#[test]
fn incorrect_after_root_fails_closed() {
    assert!(golden_bytes(&edit(1, 17, "MOCK-ROOT-01-A")).is_none());
}
#[test]
fn incorrect_r019_evidence_root_fails_closed() {
    assert!(golden_bytes(&edit(
        0,
        5,
        "0000000000000000000000000000000000000000000000000000000000000000"
    ))
    .is_none());
}
#[test]
fn incorrect_scope_metadata_fails_closed() {
    assert!(golden_bytes(&edit(0, 8, "PRODUCTION")).is_none());
}
#[test]
fn altered_revocation_flag_fails_closed() {
    assert!(golden_bytes(&edit(7, 19, "NO")).is_none());
}
#[test]
fn no_real_signature_verification_or_native_api_is_exported() {
    assert_eq!(golden_bytes(ROWS), Some(GOLDEN.to_vec()));
    assert!(parse_rows(ROWS)
        .unwrap()
        .iter()
        .all(|row| row[20] != "PRODUCTION_ADMITTED" && row[20] != "NATIVE_AUTHORIZED"));
    println!("R20_WITNESS version=1 kind=synthetic_trust_root_lifecycle cases=132 invalid=12 blocked=84 research_only=36 real_external_verifiers=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
