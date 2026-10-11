//! R0.23 Evidence Chain Finality & Equivocation Detection
//! TEST-ONLY / RESEARCH_ONLY. No real signatures, external authentication or native authority.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;
const BASE: &str = "9d0bbdbb5153e5d38ff0f387ee381ad806c790d9";
const R22_SHA: &str = "bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da";
const POLICY_SHA: &str = "6da4eb212e7a8e8ed1c297808ad6c7beaae14e6f2cb665b12c0d5d9b6c56dba8";
const TRANSCRIPT_SHA: &str = "26fd97b314f458458b4f89cbdde5b55d41403c5dc5ea89c061a057b5ecf494b7";
const LIMITATION: &str =
    "SYNTHETIC_ONLY_NO_REAL_SIGNING_VERIFICATION_PRODUCTION_OR_NATIVE_AUTHORITY";
const HEADER: &str = "id\tgate_id\tarea\tscenario\tbaseline_sha\tevidence_sha256\tactor\ttarget\troot_claim\tepoch_claim\theight_claim\tparent_claim\twitness_claim\tproducer\tscope\tsignature\tverifier\tci\treview\tnative\tbefore_state\tafter_state\texpected\tlimitation";
const SCENARIOS: [&str; 18] = [
    "REGISTER_A",
    "REGISTER_B",
    "ADVANCE_A",
    "ADVANCE_B",
    "FINALIZE_A",
    "EQUIVOCATE_A_HEIGHT3",
    "REWRITE_FINAL_A",
    "REPLAY_A_PRE_FINAL",
    "CROSS_ROOT_FINALITY",
    "ADVANCE_B_3",
    "FINALIZE_B",
    "EQUIVOCATE_B_HEIGHT4",
    "REVOKE_A",
    "ADVANCE_B_POST_REVOKE",
    "RESURRECT_A",
    "FORGED_EXTERNAL",
    "TAMPER_R22_LINEAGE",
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
const ROWS: &str = include_str!("../governance/r23_finality_equivocation_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r23_finality_equivocation_v1.bin");
const EVIDENCE: &[u8] = include_bytes!("../research/fixtures/r22_witness_forks_v1.bin");
const NATIVE: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EXTERNAL: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");

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
    final_height: u8,
    final_head: String,
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
        final_height: 0,
        final_head: "0".repeat(64),
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
        write!(&mut out, "{byte:02x}").expect("String formatter");
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
        format!("R23-WITNESS|{gate:02}|{lane}|{epoch}|{height}|{parent}|{root}|{action}")
            .as_bytes(),
    )
}
fn state_text(s: &State) -> String {
    format!(
        "A={}:{}:{}:{}:{}:{}:{}|B={}:{}:{}:{}:{}:{}:{}",
        s.a.epoch,
        s.a.root,
        s.a.height,
        s.a.head,
        u8::from(s.a.revoked),
        s.a.final_height,
        s.a.final_head,
        s.b.epoch,
        s.b.root,
        s.b.height,
        s.b.head,
        u8::from(s.b.revoked),
        s.b.final_height,
        s.b.final_head
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
    if row.len() != 24 || row[4] != BASE || row[5] != R22_SHA {
        return Decision::Invalid;
    }
    let (Ok(epoch), Ok(height)) = (row[9].parse::<u8>(), row[10].parse::<u8>()) else {
        return Decision::Invalid;
    };
    if row[13] != "MOCK_R23"
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
    let b_final_height = state.b.final_height;
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
    if row[11] != lane.head {
        return Decision::Blocked;
    }
    if action == "REVOKE_A" {
        if target != "A"
            || lane.height != 3
            || lane.final_height != 3
            || epoch != lane.epoch
            || height != lane.height
            || row[8] != lane.root
            || b_final_height != 4
        {
            return Decision::Blocked;
        }
        lane.revoked = true; // Do not mutate frozen head or final checkpoint.
        return Decision::ResearchReviewOnly;
    }
    if lane.height.checked_add(1) != Some(height) || epoch < lane.epoch {
        return Decision::Blocked;
    }
    let (valid, seal) = match action {
        "REGISTER_A" | "REGISTER_B" if action.ends_with(target) => (
            lane.height == 0 && height == 1 && epoch == 1 && row[8] == root_name(gate, target, 1),
            false,
        ),
        "ADVANCE_A" | "ADVANCE_B" if action.ends_with(target) => (
            lane.height == 1 && height == 2 && epoch == 2 && row[8] == root_name(gate, target, 2),
            false,
        ),
        "FINALIZE_A" if target == "A" => {
            let yes = lane.height == 2
                && lane.final_height == 0
                && epoch == 2
                && height == 3
                && row[8] == lane.root;
            (yes, yes)
        }
        "ADVANCE_B_3" if target == "B" => (
            lane.height == 2 && height == 3 && epoch == 3 && row[8] == root_name(gate, target, 3),
            false,
        ),
        "FINALIZE_B" if target == "B" => {
            let yes = lane.height == 3
                && lane.final_height == 0
                && epoch == 3
                && height == 4
                && row[8] == lane.root;
            (yes, yes)
        }
        "ADVANCE_B_POST_REVOKE" if target == "B" => (
            a_revoked
                && lane.height == 4
                && lane.final_height == 4
                && height == 5
                && epoch == 4
                && row[8] == root_name(gate, target, 4),
            false,
        ),
        _ => (false, false),
    };
    if !valid {
        return Decision::Blocked;
    }
    lane.epoch = epoch;
    lane.root.clone_from(&row[8]);
    lane.height = height;
    lane.head.clone_from(&row[12]);
    if seal {
        lane.final_height = height;
        lane.final_head.clone_from(&row[12]);
    }
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
    if rows.len() != 216 {
        return None;
    }
    let mut output = Vec::from(&b"NDR23FN1"[..]);
    output.extend_from_slice(&216u16.to_le_bytes());
    output.extend_from_slice(&certified_sha256::sha256(EVIDENCE));
    output.extend_from_slice(&hex20(BASE)?);
    let mut state = initial();
    for (idx, row) in rows.iter().enumerate() {
        let gate = idx / 18 + 1;
        let scenario = idx % 18;
        if scenario == 0 {
            state = initial();
        }
        if row[0] != format!("R23-X{:03}", idx + 1)
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
        let expected = if scenario == 16 {
            Decision::Invalid
        } else if matches!(scenario, 0 | 1 | 2 | 3 | 4 | 9 | 10 | 12 | 13) {
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
            state.a.final_height,
            state.b.final_height,
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
fn edit(idx: usize, col: usize, value: &str) -> String {
    let lf = normalized_lf(ROWS).expect("canonical fixture");
    let mut lines: Vec<String> = lf.lines().map(ToOwned::to_owned).collect();
    let mut row: Vec<String> = lines[idx + 1].split('\t').map(ToOwned::to_owned).collect();
    row[col] = value.to_owned();
    lines[idx + 1] = row.join("\t");
    lines.join("\n") + "\n"
}
fn replay_to(gate: usize, last: usize) -> State {
    let rows = parse_rows(ROWS).expect("frozen fixture");
    let mut state = initial();
    for i in 0..=last {
        let _ = apply(&rows[(gate - 1) * 18 + i], &mut state, gate);
    }
    state
}

#[test]
fn certified_r022_baseline_is_exact() {
    assert_eq!(BASE, "9d0bbdbb5153e5d38ff0f387ee381ad806c790d9");
}

#[test]
fn certified_r022_binary_sha_is_exact() {
    assert_eq!(hex_hash(EVIDENCE), R22_SHA);
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
fn canonical_registry_sha_is_exact() {
    assert_eq!(
        hex_hash(normalized_lf(ROWS).unwrap().as_bytes()),
        POLICY_SHA
    );
}

#[test]
fn canonical_registry_has_216_rows() {
    assert_eq!(parse_rows(ROWS).unwrap().len(), 216);
}

#[test]
fn canonical_registry_has_twelve_groups() {
    for (i, r) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(r[2], AREAS[i / 18]);
    }
}

#[test]
fn eighteen_scenarios_per_domain_are_ordered() {
    for (i, r) in parse_rows(ROWS).unwrap().iter().enumerate() {
        assert_eq!(r[3], SCENARIOS[i % 18]);
    }
}

#[test]
fn registry_header_has_24_columns() {
    assert_eq!(HEADER.split('\t').count(), 24);
}

#[test]
fn golden_transcript_has_expected_magic() {
    assert_eq!(&GOLDEN[..8], b"NDR23FN1");
}

#[test]
fn golden_transcript_has_expected_length() {
    assert_eq!(GOLDEN.len(), 9598);
}

#[test]
fn golden_transcript_has_expected_count() {
    assert_eq!(&GOLDEN[8..10], &216u16.to_le_bytes());
}

#[test]
fn golden_transcript_binds_r022_binary() {
    assert_eq!(&GOLDEN[10..42], &certified_sha256::sha256(EVIDENCE));
}

#[test]
fn golden_transcript_binds_r022_commit() {
    assert_eq!(&GOLDEN[42..62], &hex20(BASE).unwrap());
}

#[test]
fn golden_transcript_has_correct_tail_digest() {
    let n = GOLDEN.len() - 32;
    assert_eq!(&GOLDEN[n..], &certified_sha256::sha256(&GOLDEN[..n]));
}

#[test]
fn golden_transcript_sha256_is_frozen() {
    assert_eq!(hex_hash(GOLDEN), TRANSCRIPT_SHA);
}

#[test]
fn canonical_rebuild_matches_golden_bytes() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
}

#[test]
fn repeated_replay_is_deterministic() {
    assert_eq!(rebuild(ROWS), rebuild(ROWS));
}

#[test]
fn windows_crlf_produces_identical_transcript() {
    let lf = normalized_lf(ROWS).unwrap();
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(rebuild(&crlf), rebuild(ROWS));
}

#[test]
fn bare_carriage_return_is_rejected() {
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
fn advance_a_keeps_b_unchanged() {
    for g in 1..=12 {
        let s = replay_to(g, 2);
        assert_eq!((s.a.height, s.b.height), (2, 1));
    }
}

#[test]
fn advance_b_keeps_a_unchanged() {
    for g in 1..=12 {
        let s = replay_to(g, 3);
        assert_eq!((s.a.height, s.b.height), (2, 2));
    }
}

#[test]
fn finalize_a_commits_height_three() {
    for g in 1..=12 {
        let s = replay_to(g, 4);
        assert_eq!(s.a.final_height, 3);
        assert_eq!(s.a.final_head, s.a.head);
    }
}

#[test]
fn finalize_a_does_not_finalize_b() {
    for g in 1..=12 {
        let s = replay_to(g, 4);
        assert_eq!(s.b.final_height, 0);
    }
}

#[test]
fn equivocation_a_same_height_is_denied() {
    let mut s = replay_to(1, 4);
    let before = s.clone();
    assert_eq!(apply(&sample(5), &mut s, 1), Decision::Blocked);
    assert_eq!(s, before);
}

#[test]
fn rewrite_final_a_is_denied() {
    let mut s = replay_to(1, 5);
    let before = s.clone();
    assert_eq!(apply(&sample(6), &mut s, 1), Decision::Blocked);
    assert_eq!(s, before);
}

#[test]
fn ancestor_replay_a_is_denied() {
    let mut s = replay_to(1, 6);
    let before = s.clone();
    assert_eq!(apply(&sample(7), &mut s, 1), Decision::Blocked);
    assert_eq!(s, before);
}

#[test]
fn cross_root_finality_is_denied() {
    let mut s = replay_to(1, 7);
    let before = s.clone();
    assert_eq!(apply(&sample(8), &mut s, 1), Decision::Blocked);
    assert_eq!(s, before);
}

#[test]
fn b_advances_after_a_finality() {
    for g in 1..=12 {
        let s = replay_to(g, 9);
        assert_eq!((s.a.height, s.b.height), (3, 3));
    }
}

#[test]
fn finalize_b_commits_height_four() {
    for g in 1..=12 {
        let s = replay_to(g, 10);
        assert_eq!(s.b.final_height, 4);
        assert_eq!(s.b.final_head, s.b.head);
    }
}

#[test]
fn equivocation_b_same_height_is_denied() {
    let mut s = replay_to(1, 10);
    let before = s.clone();
    assert_eq!(apply(&sample(11), &mut s, 1), Decision::Blocked);
    assert_eq!(s, before);
}

#[test]
fn revocation_a_preserves_finalized_head() {
    for g in 1..=12 {
        let old = replay_to(g, 11);
        let s = replay_to(g, 12);
        assert!(s.a.revoked);
        assert_eq!(s.a.final_head, old.a.final_head);
        assert_eq!(s.a.head, old.a.head);
    }
}

#[test]
fn revocation_a_does_not_revoke_b() {
    for g in 1..=12 {
        let s = replay_to(g, 12);
        assert!(!s.b.revoked);
    }
}

#[test]
fn b_advances_after_a_revocation() {
    for g in 1..=12 {
        let s = replay_to(g, 13);
        assert!(s.a.revoked);
        assert_eq!(s.b.height, 5);
    }
}

#[test]
fn b_finality_survives_descendant() {
    for g in 1..=12 {
        let old = replay_to(g, 10);
        let s = replay_to(g, 13);
        assert_eq!(s.b.final_head, old.b.final_head);
        assert_ne!(s.b.head, s.b.final_head);
    }
}

#[test]
fn a_resurrection_is_denied() {
    let mut s = replay_to(1, 13);
    let old = s.clone();
    assert_eq!(apply(&sample(14), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn forged_external_verifier_is_denied() {
    let mut s = replay_to(1, 14);
    let old = s.clone();
    assert_eq!(apply(&sample(15), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn forged_parent_lineage_is_invalid() {
    let mut s = replay_to(1, 15);
    let old = s.clone();
    assert_eq!(apply(&sample(16), &mut s, 1), Decision::Invalid);
    assert_eq!(s, old);
}

#[test]
fn native_escalation_is_denied() {
    let mut s = replay_to(1, 16);
    let old = s.clone();
    assert_eq!(apply(&sample(17), &mut s, 1), Decision::Blocked);
    assert_eq!(s, old);
}

#[test]
fn duplicate_case_id_breaks_rebuild() {
    assert!(rebuild(&edit(1, 0, "R23-X001")).is_none());
}

#[test]
fn reordered_scenario_breaks_rebuild() {
    assert!(rebuild(&edit(1, 3, "REGISTER_A")).is_none());
}

#[test]
fn forged_before_state_breaks_rebuild() {
    assert!(rebuild(&edit(4, 20, "FAKE_STATE")).is_none());
}

#[test]
fn forged_after_state_breaks_rebuild() {
    assert!(rebuild(&edit(4, 21, "FAKE_STATE")).is_none());
}

#[test]
fn forged_witness_blocks_research() {
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
fn production_scope_is_never_admitted() {
    let mut r = sample(0);
    r[14] = "PRODUCTION".to_owned();
    assert_eq!(apply(&r, &mut initial(), 1), Decision::Blocked);
}

#[test]
fn synthetic_witness_never_grants_native_authority() {
    assert_eq!(rebuild(ROWS), Some(GOLDEN.to_vec()));
    assert!(parse_rows(ROWS)
        .unwrap()
        .iter()
        .all(|r| r[22] != "NATIVE_AUTHORIZED" && r[22] != "PRODUCTION_ADMITTED"));
    println!("R23_WITNESS version=1 cases=216 invalid=12 blocked=96 research_only=108 external_verifiers=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}
