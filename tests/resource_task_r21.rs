//! NORDOI R0.21 - deterministic, synthetic cross-root epoch/revocation isolation.
//! TEST-ONLY / RESEARCH_ONLY. No real key, external verifier, or native authority.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "bd54ef97523fc9f415ce0d6eaa41830b7105b7bf";
const R20_SHA: &str = "2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817";
const POLICY_SHA: &str = "179b9e332d9562d90df7bdaec8c46297b9a467cfd04b728c47750b0aec7a9a57";
const TRANSCRIPT_SHA: &str = "cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba";
const LIMITATION: &str = "SYNTHETIC_ONLY_NO_REAL_SIGNATURE_EXTERNAL_VERIFIER_OR_NATIVE_AUTHORITY";
const HEADER: &str = "id\tgate_id\tarea\tscenario\tbaseline_sha\tevidence_sha256\tactor\ttarget\troot_claim\tepoch_claim\tproducer\tscope\tsignature\tverifier\tci\treview\tnative\tbefore_state\tafter_state\texpected\tlimitation";
const ROWS: &str = include_str!("../governance/r21_cross_root_epochs_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r21_cross_root_transcript_v1.bin");
const EVIDENCE: &[u8] = include_bytes!("../research/fixtures/r20_trust_root_transitions_v1.bin");
const NATIVE: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EXTERNAL: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const SCENARIOS: [&str; 13] = [
    "REGISTER_A",
    "REGISTER_B",
    "ROTATE_A",
    "CROSS_ROOT_WRITE",
    "STALE_A",
    "ROTATE_B",
    "EPOCH_COLLISION",
    "REVOKE_A",
    "RESURRECT_A",
    "REPLAY_A",
    "B_ROTATES_AFTER_A_REVOKED",
    "TAMPER_LINEAGE",
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
#[derive(Clone, Debug, PartialEq, Eq)]
struct Lane {
    epoch: u8,
    root: String,
    revoked: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    a: Lane,
    b: Lane,
}
fn initial_lane() -> Lane {
    Lane {
        epoch: 0,
        root: "NONE".to_owned(),
        revoked: false,
    }
}
fn initial() -> State {
    State {
        a: initial_lane(),
        b: initial_lane(),
    }
}
fn root_name(gate: usize, lane: &str, epoch: u8) -> String {
    format!("MOCK-G{gate:02}-{lane}-E{epoch}")
}
fn state_text(s: &State) -> String {
    format!(
        "A={}:{}:{}|B={}:{}:{}",
        s.a.epoch,
        s.a.root,
        u8::from(s.a.revoked),
        s.b.epoch,
        s.b.root,
        u8::from(s.b.revoked),
    )
}
fn hex_hash(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in certified_sha256::sha256(bytes) {
        write!(&mut out, "{byte:02x}").expect("String formatting is infallible");
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
fn parse_rows(source: &str) -> Option<Vec<Vec<String>>> {
    let lf = normalized_lf(source)?;
    let mut lines = lf.lines();
    if lines.next()? != HEADER {
        return None;
    }
    let mut records = Vec::new();
    for line in lines {
        let cols: Vec<String> = line.split('\t').map(ToOwned::to_owned).collect();
        if cols.len() != 21 || cols.iter().any(String::is_empty) {
            return None;
        }
        records.push(cols);
    }
    Some(records)
}
fn decision_label(decision: Decision) -> &'static str {
    match decision {
        Decision::Invalid => "INVALID",
        Decision::Blocked => "BLOCKED",
        Decision::ResearchReviewOnly => "RESEARCH_REVIEW_ONLY",
    }
}
fn decision_code(decision: Decision) -> u8 {
    match decision {
        Decision::Invalid => 0,
        Decision::Blocked => 1,
        Decision::ResearchReviewOnly => 2,
    }
}
fn apply(row: &[String], state: &mut State, gate: usize) -> Decision {
    if row.len() != 21 || row[4] != BASE || row[5] != R20_SHA {
        return Decision::Invalid;
    }
    let Ok(epoch) = row[9].parse::<u8>() else {
        return Decision::Invalid;
    };
    if row[10] != "MOCK_R21"
        || row[11] != "RESEARCH_ONLY"
        || row[12] != "NONE"
        || row[13] != "NONE"
        || row[14] != "YES"
        || row[15] != "YES"
        || row[16] != "NO"
    {
        return Decision::Blocked;
    }
    if row[6] != row[7] {
        return Decision::Blocked;
    }
    let target = row[7].as_str();
    if target != "A" && target != "B" {
        return Decision::Blocked;
    }
    let a_revoked = state.a.revoked;
    let lane = if target == "A" {
        &mut state.a
    } else {
        &mut state.b
    };
    if lane.revoked {
        return Decision::Blocked;
    }
    let action = row[3].as_str();
    if ((action == "REGISTER_A" && target == "A") || (action == "REGISTER_B" && target == "B"))
        && lane.epoch == 0
        && lane.root == "NONE"
        && epoch == 1
        && row[8] == root_name(gate, target, 1)
    {
        lane.epoch = 1;
        lane.root.clone_from(&row[8]);
        return Decision::ResearchReviewOnly;
    }
    if ((action == "ROTATE_A" && target == "A") || (action == "ROTATE_B" && target == "B"))
        && lane.epoch == 1
        && lane.root == root_name(gate, target, 1)
        && epoch == 2
        && row[8] == root_name(gate, target, 2)
    {
        lane.epoch = 2;
        lane.root.clone_from(&row[8]);
        return Decision::ResearchReviewOnly;
    }
    if action == "REVOKE_A"
        && target == "A"
        && lane.epoch == 2
        && lane.root == root_name(gate, "A", 2)
        && epoch == 2
        && row[8] == lane.root
    {
        lane.revoked = true;
        return Decision::ResearchReviewOnly;
    }
    if action == "B_ROTATES_AFTER_A_REVOKED"
        && target == "B"
        && a_revoked
        && lane.epoch == 2
        && lane.root == root_name(gate, "B", 2)
        && epoch == 3
        && row[8] == root_name(gate, "B", 3)
    {
        lane.epoch = 3;
        lane.root.clone_from(&row[8]);
        return Decision::ResearchReviewOnly;
    }
    Decision::Blocked
}
fn rebuild(source: &str) -> Option<Vec<u8>> {
    let rows = parse_rows(source)?;
    if rows.len() != 156 {
        return None;
    }
    let mut result = Vec::from(&b"NDR21XC1"[..]);
    result.extend_from_slice(&156u16.to_le_bytes());
    result.extend_from_slice(&certified_sha256::sha256(EVIDENCE));
    result.extend_from_slice(&hex_to_bytes20(BASE)?);
    let mut state = initial();
    for (idx, row) in rows.iter().enumerate() {
        let gate = idx / 13 + 1;
        let scenario = idx % 13;
        if scenario == 0 {
            state = initial();
        }
        if row[0] != format!("R21-X{:03}", idx + 1)
            || row[1] != format!("R11-G{gate:02}")
            || row[2] != AREAS[gate - 1]
            || row[3] != SCENARIOS[scenario]
            || row[4] != BASE
            || row[20] != LIMITATION
        {
            return None;
        }
        let before = state.clone();
        if row[17] != state_text(&before) {
            return None;
        }
        let outcome = apply(row, &mut state, gate);
        let expected = if scenario == 11 {
            Decision::Invalid
        } else if matches!(scenario, 0 | 1 | 2 | 5 | 7 | 10) {
            Decision::ResearchReviewOnly
        } else {
            Decision::Blocked
        };
        if outcome != expected
            || row[19] != decision_label(outcome)
            || row[18] != state_text(&state)
        {
            return None;
        }
        result.extend_from_slice(&u16::try_from(idx + 1).ok()?.to_le_bytes());
        result.extend_from_slice(&[
            u8::try_from(scenario).ok()?,
            decision_code(outcome),
            before.a.epoch,
            state.a.epoch,
            before.b.epoch,
            state.b.epoch,
            u8::from(state.a.revoked),
            u8::from(state.b.revoked),
        ]);
        let row_text = format!("{}\n", row.join("\t"));
        result.extend_from_slice(&certified_sha256::sha256(row_text.as_bytes()));
    }
    let digest = certified_sha256::sha256(&result);
    result.extend_from_slice(&digest);
    Some(result)
}
fn hex_to_bytes20(source: &str) -> Option<[u8; 20]> {
    if source.len() != 40 {
        return None;
    }
    let mut out = [0u8; 20];
    for (index, item) in out.iter_mut().enumerate() {
        *item = u8::from_str_radix(source.get(index * 2..index * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}
fn historic_status(source: &str, expected: &str) -> bool {
    let Some(lf) = normalized_lf(source) else {
        return false;
    };
    let mut lines = lf.lines();
    let Some(header) = lines.next() else {
        return false;
    };
    let columns: Vec<&str> = header.split('\t').collect();
    let Some(area_idx) = columns.iter().position(|column| *column == "area") else {
        return false;
    };
    let Some(status_idx) = columns.iter().position(|column| *column == "status") else {
        return false;
    };
    let records: Vec<&str> = lines.collect();
    records.len() == 12
        && records.iter().enumerate().all(|(idx, line)| {
            let fields: Vec<&str> = line.split('\t').collect();
            fields.len() == columns.len()
                && fields[area_idx] == AREAS[idx]
                && fields[status_idx] == expected
        })
}
fn sample(index: usize) -> Vec<String> {
    parse_rows(ROWS).expect("frozen registry")[index].clone()
}
fn edit(index: usize, column: usize, value: &str) -> String {
    let lf = normalized_lf(ROWS).expect("canonical registry");
    let mut lines: Vec<String> = lf.lines().map(ToOwned::to_owned).collect();
    let mut row: Vec<String> = lines[index + 1]
        .split('\t')
        .map(ToOwned::to_owned)
        .collect();
    row[column] = value.to_owned();
    lines[index + 1] = row.join("\t");
    lines.join("\n") + "\n"
}
fn replay_to_scenario(gate: usize, last_scenario: usize) -> State {
    let rows = parse_rows(ROWS).expect("frozen registry");
    let mut state = initial();
    for index in 0..=last_scenario {
        apply(&rows[(gate - 1) * 13 + index], &mut state, gate);
    }
    state
}

#[test]
fn r020_certified_commit_pinned() {
    assert_eq!(BASE, "bd54ef97523fc9f415ce0d6eaa41830b7105b7bf");
}
#[test]
fn r020_binary_sha_is_exact() {
    assert_eq!(hex_hash(EVIDENCE), R20_SHA);
}
#[test]
fn historical_native_gates_remain_unmet() {
    assert!(historic_status(NATIVE, "UNMET"));
}
#[test]
fn historical_external_verifiers_remain_absent() {
    assert!(historic_status(EXTERNAL, "ABSENT"));
}
#[test]
fn registry_sha_is_pinned_after_lf_normalization() {
    assert_eq!(
        hex_hash(normalized_lf(ROWS).unwrap().as_bytes()),
        POLICY_SHA
    );
}
#[test]
fn registry_has_exactly_156_rows() {
    assert_eq!(parse_rows(ROWS).unwrap().len(), 156);
}
#[test]
fn registry_has_twelve_isolated_domains() {
    for (idx, row) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(row[2], AREAS[idx / 13]);
    }
}
#[test]
fn registry_has_thirteen_scenarios_per_domain() {
    for (idx, row) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(row[3], SCENARIOS[idx % 13]);
    }
}
#[test]
fn header_contains_exact_schema() {
    assert_eq!(HEADER.split('\t').count(), 21);
}
#[test]
fn transcript_magic_is_frozen() {
    assert_eq!(&GOLDEN[..8], b"NDR21XC1");
}
#[test]
fn transcript_has_expected_length() {
    assert_eq!(GOLDEN.len(), 6646);
}
#[test]
fn transcript_binds_156_rows() {
    assert_eq!(&GOLDEN[8..10], &156u16.to_le_bytes());
}
#[test]
fn transcript_binds_r020_sha() {
    assert_eq!(&GOLDEN[10..42], &certified_sha256::sha256(EVIDENCE));
}
#[test]
fn transcript_binds_r020_commit() {
    assert_eq!(&GOLDEN[42..62], &hex_to_bytes20(BASE).unwrap());
}
#[test]
fn transcript_tail_digest_is_exact() {
    assert_eq!(
        &GOLDEN[GOLDEN.len() - 32..],
        &certified_sha256::sha256(&GOLDEN[..GOLDEN.len() - 32])
    );
}
#[test]
fn transcript_sha256_is_frozen() {
    assert_eq!(hex_hash(GOLDEN), TRANSCRIPT_SHA);
}
#[test]
fn canonical_replay_matches_golden() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
}
#[test]
fn replay_is_byte_deterministic() {
    assert_eq!(rebuild(ROWS), rebuild(ROWS));
}
#[test]
fn windows_crlf_replay_is_identical() {
    let crlf = normalized_lf(ROWS).unwrap().replace('\n', "\r\n");
    assert_eq!(rebuild(&crlf), rebuild(ROWS));
}
#[test]
fn invalid_embedded_carriage_return_fails_closed() {
    assert!(rebuild(&edit(0, 3, "REGISTER\r_A")).is_none());
}
#[test]
fn each_gate_can_register_a_and_b_independently() {
    for gate in 1..=12 {
        let state = replay_to_scenario(gate, 1);
        assert_eq!(state.a.epoch, 1);
        assert_eq!(state.b.epoch, 1);
        assert_ne!(state.a.root, state.b.root);
    }
}
#[test]
fn rotating_a_does_not_rotate_b() {
    for gate in 1..=12 {
        let state = replay_to_scenario(gate, 2);
        assert_eq!(state.a.epoch, 2);
        assert_eq!(state.b.epoch, 1);
    }
}
#[test]
fn cross_root_write_is_denied_without_mutation() {
    let mut state = replay_to_scenario(1, 2);
    let before = state.clone();
    assert_eq!(apply(&sample(3), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn stale_a_epoch_is_denied_without_mutation() {
    let mut state = replay_to_scenario(1, 3);
    let before = state.clone();
    assert_eq!(apply(&sample(4), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn rotating_b_does_not_rotate_a() {
    let state = replay_to_scenario(1, 5);
    assert_eq!(state.a.epoch, 2);
    assert_eq!(state.b.epoch, 2);
}
#[test]
fn cross_root_epoch_collision_is_denied() {
    let mut state = replay_to_scenario(1, 5);
    let before = state.clone();
    assert_eq!(apply(&sample(6), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn revocation_a_is_permanent_for_same_simulated_chain() {
    let state = replay_to_scenario(1, 7);
    assert!(state.a.revoked);
    assert!(!state.b.revoked);
}
#[test]
fn resurrection_a_is_denied_after_revocation() {
    let mut state = replay_to_scenario(1, 7);
    let before = state.clone();
    assert_eq!(apply(&sample(8), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn stale_replay_a_is_denied_after_revocation() {
    let mut state = replay_to_scenario(1, 8);
    let before = state.clone();
    assert_eq!(apply(&sample(9), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn unrelated_b_can_rotate_after_a_revocation() {
    for gate in 1..=12 {
        let state = replay_to_scenario(gate, 10);
        assert!(state.a.revoked);
        assert_eq!(state.a.epoch, 2);
        assert_eq!(state.b.epoch, 3);
        assert!(!state.b.revoked);
    }
}
#[test]
fn forged_r020_lineage_is_invalid() {
    let mut state = replay_to_scenario(1, 10);
    let before = state.clone();
    assert_eq!(apply(&sample(11), &mut state, 1), Decision::Invalid);
    assert_eq!(state, before);
}
#[test]
fn native_escalation_of_b_is_denied() {
    let mut state = replay_to_scenario(1, 11);
    let before = state.clone();
    assert_eq!(apply(&sample(12), &mut state, 1), Decision::Blocked);
    assert_eq!(state, before);
}
#[test]
fn altered_parent_hash_fails_closed() {
    assert!(rebuild(&edit(0, 5, &"0".repeat(64))).is_none());
}
#[test]
fn altered_commit_identity_fails_closed() {
    assert!(rebuild(&edit(0, 4, &"0".repeat(40))).is_none());
}
#[test]
fn duplicate_case_identity_fails_closed() {
    assert!(rebuild(&edit(1, 0, "R21-X001")).is_none());
}
#[test]
fn reordering_domain_scenarios_fails_closed() {
    assert!(rebuild(&edit(1, 3, "REGISTER_A")).is_none());
}
#[test]
fn forged_before_state_fails_closed() {
    assert!(rebuild(&edit(5, 17, "A=0:NONE:0|B=0:NONE:0")).is_none());
}
#[test]
fn forged_after_state_fails_closed() {
    assert!(rebuild(&edit(7, 18, "A=2:MOCK-G01-A-E2:0|B=2:MOCK-G01-B-E2:0")).is_none());
}
#[test]
fn missing_ci_blocks_registration() {
    let mut row = sample(0);
    row[14] = "NO".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn missing_review_blocks_registration() {
    let mut row = sample(0);
    row[15] = "NO".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn fake_signer_blocks_registration() {
    let mut row = sample(0);
    row[12] = "FAKE_SIGNER".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn fake_external_verifier_blocks_registration() {
    let mut row = sample(0);
    row[13] = "MOCK_VERIFIER".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn production_scope_never_enters_research_or_native() {
    let mut row = sample(0);
    row[11] = "PRODUCTION".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn unknown_producer_never_enters_research() {
    let mut row = sample(0);
    row[10] = "EXTERNAL".to_owned();
    assert_eq!(apply(&row, &mut initial(), 1), Decision::Blocked);
}
#[test]
fn transcript_rehash_does_not_admit_tamper() {
    let mut changed = GOLDEN.to_vec();
    changed[105] ^= 0x01;
    let size = changed.len();
    let sha = certified_sha256::sha256(&changed[..size - 32]);
    changed[size - 32..].copy_from_slice(&sha);
    assert_ne!(hex_hash(&changed), TRANSCRIPT_SHA);
    assert_ne!(rebuild(ROWS), Some(changed));
}
#[test]
fn cross_gate_root_identity_cannot_be_reused() {
    let mut row = sample(13);
    row[8] = root_name(1, "A", 1);
    assert_eq!(apply(&row, &mut initial(), 2), Decision::Blocked);
}
#[test]
fn all_156_decisions_have_only_three_research_labels() {
    let rows = parse_rows(ROWS).unwrap();
    let mut invalid = 0;
    let mut blocked = 0;
    let mut research = 0;
    for row in rows {
        match row[19].as_str() {
            "INVALID" => invalid += 1,
            "BLOCKED" => blocked += 1,
            "RESEARCH_REVIEW_ONLY" => research += 1,
            _ => panic!("unknown admission status"),
        }
    }
    assert_eq!((invalid, blocked, research), (12, 72, 72));
}
#[test]
fn synthetic_replay_never_exports_native_authority() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
    assert!(parse_rows(ROWS)
        .unwrap()
        .iter()
        .all(|r| r[19] != "NATIVE_AUTHORIZED" && r[19] != "PRODUCTION_ADMITTED"));
    println!("R21_WITNESS version=1 kind=cross_root_epoch_isolation cases=156 invalid=12 blocked=72 research_only=72 real_external_verifiers=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
