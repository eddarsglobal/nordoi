//! R0.22 synthetic witness fork isolation with monotonic replay.
//! TEST-ONLY / RESEARCH_ONLY. No real signing, external verifier or native authority.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "a34bc552b31661570f270c6c75700849b261084c";
const R21_SHA: &str = "cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba";
const POLICY_SHA: &str = "31a2f2ef5fbadf9d3a026e570db45e9da0c0d990ffafb52a3577bc267b14155f";
const TRANSCRIPT_SHA: &str = "bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da";
const LIMITATION: &str =
    "SYNTHETIC_ONLY_NO_REAL_SIGNING_VERIFICATION_PRODUCTION_OR_NATIVE_AUTHORITY";
const HEADER: &str = "id\tgate_id\tarea\tscenario\tbaseline_sha\tevidence_sha256\tactor\ttarget\troot_claim\tepoch_claim\theight_claim\tparent_claim\twitness_claim\tproducer\tscope\tsignature\tverifier\tci\treview\tnative\tbefore_state\tafter_state\texpected\tlimitation";
const ROWS: &str = include_str!("../governance/r22_witness_forks_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r22_witness_forks_v1.bin");
const EVIDENCE: &[u8] = include_bytes!("../research/fixtures/r21_cross_root_transcript_v1.bin");
const NATIVE: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EXTERNAL: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const SCENARIOS: [&str; 15] = [
    "REGISTER_A",
    "REGISTER_B",
    "ADVANCE_A",
    "ADVANCE_B",
    "FORK_A_EQUAL_HEIGHT",
    "REPLAY_A",
    "STALE_PARENT_A",
    "CROSS_ROOT_PARENT",
    "ADVANCE_B_3",
    "REVOKE_A",
    "RESURRECT_A",
    "ADVANCE_B_AFTER_REVOKE",
    "FORGED_EXTERNAL",
    "TAMPER_R21_LINEAGE",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Lane {
    epoch: u8,
    root: String,
    height: u8,
    head: String,
    revoked: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
    a: Lane,
    b: Lane,
}

fn init_lane() -> Lane {
    Lane {
        epoch: 0,
        root: "NONE".to_owned(),
        height: 0,
        head: "0".repeat(64),
        revoked: false,
    }
}
fn initial() -> State {
    State {
        a: init_lane(),
        b: init_lane(),
    }
}
fn hex_hash(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in certified_sha256::sha256(bytes) {
        write!(&mut out, "{byte:02x}").expect("String formatting is infallible");
    }
    out
}
fn root_name(gate: usize, lane: &str, epoch: u8) -> String {
    format!("MOCK-G{gate:02}-{lane}-E{epoch}")
}
fn witness(
    gate: usize,
    lane: &str,
    epoch: u8,
    height: u8,
    parent: &str,
    root: &str,
    action: &str,
) -> String {
    hex_hash(
        format!("R22-WITNESS|{gate:02}|{lane}|{epoch}|{height}|{parent}|{root}|{action}")
            .as_bytes(),
    )
}
fn state_text(state: &State) -> String {
    format!(
        "A={}:{}:{}:{}:{}|B={}:{}:{}:{}:{}",
        state.a.epoch,
        state.a.root,
        state.a.height,
        state.a.head,
        u8::from(state.a.revoked),
        state.b.epoch,
        state.b.root,
        state.b.height,
        state.b.head,
        u8::from(state.b.revoked)
    )
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
    let mut rows = Vec::new();
    for line in lines {
        let row: Vec<String> = line.split('\t').map(ToOwned::to_owned).collect();
        if row.len() != 24 || row.iter().any(String::is_empty) {
            return None;
        }
        rows.push(row);
    }
    Some(rows)
}
fn label(d: Decision) -> &'static str {
    match d {
        Decision::Invalid => "INVALID",
        Decision::Blocked => "BLOCKED",
        Decision::ResearchReviewOnly => "RESEARCH_REVIEW_ONLY",
    }
}
fn code(d: Decision) -> u8 {
    match d {
        Decision::Invalid => 0,
        Decision::Blocked => 1,
        Decision::ResearchReviewOnly => 2,
    }
}
fn apply(row: &[String], state: &mut State, gate: usize) -> Decision {
    if row.len() != 24 || row[4] != BASE || row[5] != R21_SHA {
        return Decision::Invalid;
    }
    let (Ok(epoch), Ok(height)) = (row[9].parse::<u8>(), row[10].parse::<u8>()) else {
        return Decision::Invalid;
    };
    if row[13] != "MOCK_R22"
        || row[14] != "RESEARCH_ONLY"
        || row[15] != "NONE"
        || row[16] != "NONE"
        || row[17] != "YES"
        || row[18] != "YES"
        || row[19] != "NO"
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
    let b_height = state.b.height;
    let lane = if target == "A" {
        &mut state.a
    } else {
        &mut state.b
    };
    if lane.revoked {
        return Decision::Blocked;
    }
    let action = row[3].as_str();
    if row[12] != witness(gate, target, epoch, height, &row[11], &row[8], action) {
        return Decision::Blocked;
    }
    if row[11] != lane.head || lane.height.checked_add(1) != Some(height) {
        return Decision::Blocked;
    }
    let mut revoke = false;
    let acceptable = match action {
        "REGISTER_A" | "REGISTER_B" if action.ends_with(target) => {
            lane.height == 0 && epoch == 1 && height == 1 && row[8] == root_name(gate, target, 1)
        }
        "ADVANCE_A" | "ADVANCE_B" if action.ends_with(target) => {
            lane.height == 1 && epoch == 2 && height == 2 && row[8] == root_name(gate, target, 2)
        }
        "ADVANCE_B_3" if target == "B" => {
            lane.height == 2 && epoch == 3 && height == 3 && row[8] == root_name(gate, target, 3)
        }
        "REVOKE_A" if target == "A" => {
            let valid = lane.height == 2
                && epoch == 2
                && height == 3
                && row[8] == lane.root
                && b_height == 3;
            revoke = valid;
            valid
        }
        "ADVANCE_B_AFTER_REVOKE" if target == "B" => {
            lane.height == 3
                && a_revoked
                && epoch == 4
                && height == 4
                && row[8] == root_name(gate, target, 4)
        }
        _ => false,
    };
    if !acceptable {
        return Decision::Blocked;
    }
    lane.epoch = epoch;
    lane.root.clone_from(&row[8]);
    lane.height = height;
    lane.head.clone_from(&row[12]);
    lane.revoked = revoke;
    Decision::ResearchReviewOnly
}
fn hex20(source: &str) -> Option<[u8; 20]> {
    if source.len() != 40 {
        return None;
    }
    let mut out = [0u8; 20];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(source.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}
fn rebuild(source: &str) -> Option<Vec<u8>> {
    let rows = parse_rows(source)?;
    if rows.len() != 180 {
        return None;
    }
    let mut output = Vec::from(&b"NDR22FK1"[..]);
    output.extend_from_slice(&180u16.to_le_bytes());
    output.extend_from_slice(&certified_sha256::sha256(EVIDENCE));
    output.extend_from_slice(&hex20(BASE)?);
    let mut state = initial();
    for (idx, row) in rows.iter().enumerate() {
        let gate = idx / 15 + 1;
        let scenario = idx % 15;
        if scenario == 0 {
            state = initial();
        }
        if row[0] != format!("R22-X{:03}", idx + 1)
            || row[1] != format!("R11-G{gate:02}")
            || row[2] != AREAS[gate - 1]
            || row[3] != SCENARIOS[scenario]
            || row[4] != BASE
            || row[23] != LIMITATION
        {
            return None;
        }
        let before = state.clone();
        if row[20] != state_text(&before) {
            return None;
        }
        let d = apply(row, &mut state, gate);
        let expected = if scenario == 13 {
            Decision::Invalid
        } else if matches!(scenario, 0 | 1 | 2 | 3 | 8 | 9 | 11) {
            Decision::ResearchReviewOnly
        } else {
            Decision::Blocked
        };
        if d != expected || row[22] != label(d) || row[21] != state_text(&state) {
            return None;
        }
        output.extend_from_slice(&u16::try_from(idx + 1).ok()?.to_le_bytes());
        output.extend_from_slice(&[
            u8::try_from(scenario).ok()?,
            code(d),
            before.a.height,
            state.a.height,
            before.b.height,
            state.b.height,
            u8::from(state.a.revoked),
            u8::from(state.b.revoked),
        ]);
        output.extend_from_slice(&certified_sha256::sha256(
            format!("{}\n", row.join("\t")).as_bytes(),
        ));
    }
    let digest = certified_sha256::sha256(&output);
    output.extend_from_slice(&digest);
    Some(output)
}
fn historic_status(source: &str, expected: &str) -> bool {
    let Some(lf) = normalized_lf(source) else {
        return false;
    };
    let mut lines = lf.lines();
    let Some(header) = lines.next() else {
        return false;
    };
    let cols: Vec<&str> = header.split('\t').collect();
    let Some(area) = cols.iter().position(|s| *s == "area") else {
        return false;
    };
    let Some(status) = cols.iter().position(|s| *s == "status") else {
        return false;
    };
    let records: Vec<&str> = lines.collect();
    records.len() == 12
        && records.iter().enumerate().all(|(i, line)| {
            let parts: Vec<&str> = line.split('\t').collect();
            parts.len() == cols.len() && parts[area] == AREAS[i] && parts[status] == expected
        })
}
fn sample(idx: usize) -> Vec<String> {
    parse_rows(ROWS).expect("frozen registry")[idx].clone()
}
fn edit(idx: usize, column: usize, value: &str) -> String {
    let lf = normalized_lf(ROWS).expect("canonical fixture");
    let mut lines: Vec<String> = lf.lines().map(ToOwned::to_owned).collect();
    let mut row: Vec<String> = lines[idx + 1].split('\t').map(ToOwned::to_owned).collect();
    row[column] = value.to_owned();
    lines[idx + 1] = row.join("\t");
    lines.join("\n") + "\n"
}
fn replay_to(gate: usize, last: usize) -> State {
    let rows = parse_rows(ROWS).expect("frozen fixture");
    let mut state = initial();
    for i in 0..=last {
        let _ = apply(&rows[(gate - 1) * 15 + i], &mut state, gate);
    }
    state
}

#[test]
fn certified_r021_baseline_is_exact() {
    assert_eq!(BASE, "a34bc552b31661570f270c6c75700849b261084c");
}

#[test]
fn certified_r021_binary_sha_is_exact() {
    assert_eq!(hex_hash(EVIDENCE), R21_SHA);
}

#[test]
fn historical_twelve_native_gates_stay_unmet() {
    assert!(historic_status(NATIVE, "UNMET"));
}

#[test]
fn historical_twelve_external_verifiers_stay_absent() {
    assert!(historic_status(EXTERNAL, "ABSENT"));
}

#[test]
fn registry_sha_is_frozen() {
    assert_eq!(
        hex_hash(normalized_lf(ROWS).unwrap().as_bytes()),
        POLICY_SHA
    );
}

#[test]
fn registry_has_one_hundred_eighty_rows() {
    assert_eq!(parse_rows(ROWS).unwrap().len(), 180);
}

#[test]
fn twelve_gate_groups_are_exact() {
    for (idx, r) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(r[2], AREAS[idx / 15]);
    }
}

#[test]
fn fifteen_scenarios_are_ordered() {
    for (idx, r) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(r[3], SCENARIOS[idx % 15]);
    }
}

#[test]
fn header_schema_has_twenty_four_columns() {
    assert_eq!(HEADER.split('\t').count(), 24);
}

#[test]
fn transcript_magic_is_exact() {
    assert_eq!(&GOLDEN[..8], b"NDR22FK1");
}

#[test]
fn transcript_size_is_exact() {
    assert_eq!(GOLDEN.len(), 7654);
}

#[test]
fn transcript_case_count_is_exact() {
    assert_eq!(&GOLDEN[8..10], &180u16.to_le_bytes());
}

#[test]
fn transcript_binds_r021_binary() {
    assert_eq!(&GOLDEN[10..42], &certified_sha256::sha256(EVIDENCE));
}

#[test]
fn transcript_binds_r021_commit() {
    assert_eq!(&GOLDEN[42..62], &hex20(BASE).unwrap());
}

#[test]
fn transcript_tail_digest_is_exact() {
    let end = GOLDEN.len() - 32;
    assert_eq!(&GOLDEN[end..], &certified_sha256::sha256(&GOLDEN[..end]));
}

#[test]
fn transcript_sha256_is_frozen() {
    assert_eq!(hex_hash(GOLDEN), TRANSCRIPT_SHA);
}

#[test]
fn independent_rebuild_matches_binary() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
}

#[test]
fn independent_rebuild_is_deterministic() {
    assert_eq!(rebuild(ROWS), rebuild(ROWS));
}

#[test]
fn windows_crlf_normalizes_before_replay() {
    let crlf = normalized_lf(ROWS).unwrap().replace('\n', "\r\n");
    assert_eq!(rebuild(&crlf), rebuild(ROWS));
}

#[test]
fn embedded_carriage_return_fails_closed() {
    assert!(rebuild(&edit(0, 3, "REGISTER\r_A")).is_none());
}

#[test]
fn a_and_b_register_independently() {
    for g in 1..=12 {
        let s = replay_to(g, 1);
        assert_eq!((s.a.height, s.b.height), (1, 1));
    }
}

#[test]
fn a_and_b_heads_are_distinct() {
    for g in 1..=12 {
        let s = replay_to(g, 1);
        assert_ne!(s.a.head, s.b.head);
    }
}

#[test]
fn advance_a_does_not_touch_b() {
    for g in 1..=12 {
        let s = replay_to(g, 2);
        assert_eq!((s.a.height, s.b.height), (2, 1));
    }
}

#[test]
fn advance_b_does_not_touch_a() {
    for g in 1..=12 {
        let s = replay_to(g, 3);
        assert_eq!((s.a.height, s.b.height), (2, 2));
    }
}

#[test]
fn same_height_fork_is_denied() {
    let mut s = replay_to(1, 3);
    let old = s.clone();
    assert_eq!(apply(&sample(4), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn duplicate_a_witness_is_denied() {
    let mut s = replay_to(1, 4);
    let old = s.clone();
    assert_eq!(apply(&sample(5), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn stale_parent_is_denied() {
    let mut s = replay_to(1, 5);
    let old = s.clone();
    assert_eq!(apply(&sample(6), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn cross_root_parent_is_denied() {
    let mut s = replay_to(1, 6);
    let old = s.clone();
    assert_eq!(apply(&sample(7), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn b_advances_to_height_three() {
    for g in 1..=12 {
        let s = replay_to(g, 8);
        assert_eq!((s.a.height, s.b.height), (2, 3));
    }
}

#[test]
fn revoking_a_persists_head_and_flag() {
    for g in 1..=12 {
        let s = replay_to(g, 9);
        assert_eq!(s.a.height, 3);
        assert!(s.a.revoked);
        assert!(!s.b.revoked);
    }
}

#[test]
fn revoked_a_cannot_resurrect() {
    let mut s = replay_to(1, 9);
    let old = s.clone();
    assert_eq!(apply(&sample(10), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn b_can_advance_after_a_revocation() {
    for g in 1..=12 {
        let s = replay_to(g, 11);
        assert!(s.a.revoked);
        assert_eq!((s.a.height, s.b.height), (3, 4));
    }
}

#[test]
fn forged_external_signer_is_blocked() {
    let mut s = replay_to(1, 11);
    let old = s.clone();
    assert_eq!(apply(&sample(12), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn tampered_r021_lineage_is_invalid() {
    let mut s = replay_to(1, 12);
    let old = s.clone();
    assert_eq!(apply(&sample(13), &mut s, 1), Decision::Invalid);
    assert_eq!(s, old);
}

#[test]
fn native_escalation_is_blocked() {
    let mut s = replay_to(1, 13);
    let old = s.clone();
    assert_eq!(apply(&sample(14), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn duplicate_case_id_fails_closed() {
    assert!(rebuild(&edit(1, 0, "R22-X001")).is_none());
}

#[test]
fn reordered_scenario_fails_closed() {
    assert!(rebuild(&edit(1, 3, "REGISTER_A")).is_none());
}

#[test]
fn wrong_parent_for_success_fails_closed() {
    assert!(rebuild(&edit(2, 11, &"0".repeat(64))).is_none());
}

#[test]
fn forged_after_state_fails_closed() {
    assert!(rebuild(&edit(9, 21, "FAKE_STATE")).is_none());
}

#[test]
fn forged_before_state_fails_closed() {
    assert!(rebuild(&edit(9, 20, "FAKE_STATE")).is_none());
}

#[test]
fn forged_witness_digest_blocks_research() {
    let mut r = sample(0);
    r[12] = "0".repeat(64);
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn fake_signer_blocks_registration() {
    let mut r = sample(0);
    r[15] = "FAKE".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn fake_verifier_blocks_registration() {
    let mut r = sample(0);
    r[16] = "FAKE".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn missing_ci_blocks_registration() {
    let mut r = sample(0);
    r[17] = "NO".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn missing_review_blocks_registration() {
    let mut r = sample(0);
    r[18] = "NO".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn production_scope_never_admitted() {
    let mut r = sample(0);
    r[14] = "PRODUCTION".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn transcript_tamper_stays_invalid_after_rehash() {
    let mut bad = GOLDEN.to_vec();
    bad[99] ^= 1;
    let end = bad.len() - 32;
    let d = certified_sha256::sha256(&bad[..end]);
    bad[end..].copy_from_slice(&d);
    assert_ne!(hex_hash(&bad), TRANSCRIPT_SHA);
    assert_ne!(rebuild(ROWS), Some(bad));
}

#[test]
fn synthetic_witness_does_not_export_native_authority() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
    assert!(parse_rows(ROWS)
        .unwrap()
        .iter()
        .all(|r| r[22] != "NATIVE_AUTHORIZED" && r[22] != "PRODUCTION_ADMITTED"));
    println!("R22_WITNESS version=1 cases=180 invalid=12 blocked=84 research_only=84 external_verifiers=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
