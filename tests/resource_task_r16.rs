//! NORDOI R0.16 TEST-ONLY binary bundle parser + offline SHA-256 check.
//! Real synthetic bytes (including NUL/non-UTF-8) are NOT authenticated external evidence.
//! Passing this model never grants native execution, deployment, or trust.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::collections::BTreeSet;
use std::fmt::Write;

const BASE_SHA: &str = "3158ee20a894461df5402335b7db5509db9abd02";
const PACK_SHA: &str = "f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393";
const R11: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const R12: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const R13: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const R14: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const R15: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const R16: &str = include_str!("../governance/r16_offline_bundle_manifest_v1.tsv");
const FIXTURE: &[u8] = include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin");
const HEADER: &str = "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tverification_id\tarea\tbaseline_sha\tadmitted_status\tscope\toffset\tlength\tfixture_sha256\tmock_producer\tsignature_ref\ttrust_root\tlimitation";
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

fn sha256_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for octet in certified_sha256::sha256(data) {
        write!(&mut out, "{octet:02x}").expect("String writes cannot fail");
    }
    out
}

fn rows(text: &str, header: &str, cols: usize) -> Option<Vec<Vec<String>>> {
    let mut lines = text.lines();
    if lines.next()? != header {
        return None;
    }
    let mut out = Vec::new();
    for line in lines {
        if line.is_empty() || line.contains('\r') {
            return None;
        }
        let parts: Vec<String> = line.split('\t').map(|cell| cell.to_owned()).collect();
        if parts.len() != cols || parts.iter().any(|v| v.trim().is_empty()) {
            return None;
        }
        out.push(parts);
    }
    Some(out)
}

fn frozen_contract() -> bool {
    let Some(g) = rows(
        R11,
        "id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation",
        6,
    ) else {
        return false;
    };
    let Some(e) = rows(R12, "id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation", 9) else { return false; };
    let Some(i) = rows(R13, "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation", 16) else { return false; };
    let Some(a) = rows(R14, "id\tgate_id\tevidence_id\tintegrity_id\tarea\tbaseline_sha\tstatus\tscope\tproducer_key\treviewer_key\ttrust_root\tauthentication\trevocation\tlimitation", 14) else { return false; };
    let Some(v) = rows(R15, "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tarea\tbaseline_sha\tstatus\tscope\tpolicy\tverifier_ref\ttrust_root\tlimitation", 13) else { return false; };
    if [g.len(), e.len(), i.len(), a.len(), v.len()] != [12; 5] {
        return false;
    }
    for n in 0..12 {
        if g[n][0] != format!("R11-G{:02}", n + 1)
            || g[n][1] != AREAS[n]
            || g[n][2] != "UNMET"
            || g[n][3] != "RESEARCH_ONLY"
            || e[n][0] != format!("R12-E{:02}", n + 1)
            || e[n][1] != g[n][0]
            || e[n][2] != g[n][1]
            || e[n][4] != "ABSENT"
            || e[n][5] != "RESEARCH_ONLY"
            || e[n][6] != "NONE"
            || e[n][7] != "NONE"
            || i[n][0] != format!("R13-L{:02}", n + 1)
            || i[n][1] != g[n][0]
            || i[n][2] != e[n][0]
            || i[n][3] != AREAS[n]
            || i[n][5] != "ABSENT"
            || i[n][6] != "RESEARCH_ONLY"
            || i[n][7..15].iter().any(|cell| cell != "NONE")
            || a[n][0] != format!("R14-A{:02}", n + 1)
            || a[n][1] != g[n][0]
            || a[n][2] != e[n][0]
            || a[n][3] != i[n][0]
            || a[n][4] != AREAS[n]
            || a[n][6] != "ABSENT"
            || a[n][7] != "RESEARCH_ONLY"
            || a[n][8..13].iter().any(|cell| cell != "NONE")
            || v[n][0] != format!("R15-V{:02}", n + 1)
            || v[n][1] != g[n][0]
            || v[n][2] != e[n][0]
            || v[n][3] != i[n][0]
            || v[n][4] != a[n][0]
            || v[n][5] != AREAS[n]
            || v[n][7] != "ABSENT"
            || v[n][8] != "RESEARCH_ONLY"
            || v[n][9] != "DUAL_SYNTHETIC_REVIEW"
            || v[n][10] != "NONE"
            || v[n][11] != "NONE"
        {
            return false;
        }
    }
    true
}

fn decode_pack(bytes: &[u8]) -> Option<Vec<(usize, Vec<u8>)>> {
    if bytes.get(..8)? != &b"NDR16PK1"[..] {
        return None;
    }
    let count = bytes.get(8..10)?;
    let total = usize::from(u16::from_le_bytes([count[0], count[1]]));
    if total != 12 {
        return None;
    }
    let mut pos = 10usize;
    let mut found = Vec::new();
    for _ in 0..total {
        let length_bytes = bytes.get(pos..pos.checked_add(2)?)?;
        let len = usize::from(u16::from_le_bytes([length_bytes[0], length_bytes[1]]));
        if len == 0 || len > 256 {
            return None;
        }
        pos = pos.checked_add(2)?;
        let payload = bytes.get(pos..pos.checked_add(len)?)?;
        found.push((pos, payload.to_vec()));
        pos = pos.checked_add(len)?;
    }
    if pos != bytes.len() {
        return None;
    }
    Some(found)
}

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn assess(bytes: &[u8], manifest: &str, ci: bool, review: bool) -> Decision {
    if !frozen_contract() {
        return Decision::Invalid;
    }
    let Some(parsed) = decode_pack(bytes) else {
        return Decision::Invalid;
    };
    let Some(items) = rows(manifest, HEADER, 17) else {
        return Decision::Invalid;
    };
    if items.len() != 12 || sha256_hex(bytes) != PACK_SHA {
        return Decision::Invalid;
    }
    let mut ids = BTreeSet::new();
    let mut digests = BTreeSet::new();
    let mut previous_end = 10usize;
    for (n, (entry, (offset, payload))) in items.iter().zip(parsed.iter()).enumerate() {
        let Ok(declared_offset) = entry[10].parse::<usize>() else {
            return Decision::Invalid;
        };
        let Ok(declared_length) = entry[11].parse::<usize>() else {
            return Decision::Invalid;
        };
        if entry[0] != format!("R16-B{:02}", n + 1)
            || entry[1] != format!("R11-G{:02}", n + 1)
            || entry[2] != format!("R12-E{:02}", n + 1)
            || entry[3] != format!("R13-L{:02}", n + 1)
            || entry[4] != format!("R14-A{:02}", n + 1)
            || entry[5] != format!("R15-V{:02}", n + 1)
            || entry[6] != AREAS[n]
            || entry[7] != BASE_SHA
            || entry[8] != "ABSENT"
            || entry[9] != "RESEARCH_ONLY"
            || declared_offset != *offset
            || declared_length != payload.len()
            || *offset != previous_end + 2
            || entry[12] != sha256_hex(payload)
            || entry[13] != format!("MOCK-PRODUCER-{:02}", n + 1)
            || entry[14] != "NONE"
            || entry[15] != "NONE"
            || entry[16].len() < 65
            || !ids.insert(entry[0].as_str())
            || !digests.insert(entry[12].as_str())
        {
            return Decision::Invalid;
        }
        previous_end = *offset + payload.len();
    }
    if !ci || !review {
        return Decision::Blocked;
    }
    // Even with matching actual bytes + in-tree public hashes, producer is NOT authenticated.
    Decision::ResearchReviewOnly
}

fn canonical() -> Decision {
    assess(FIXTURE, R16, true, true)
}
fn mutated_manifest(index: usize, field: usize, value: &str) -> String {
    let mut r = rows(R16, HEADER, 17).expect("controlled fixture");
    r[index][field] = value.to_owned();
    let mut out = format!("{HEADER}\n");
    for item in r {
        out.push_str(&item.join("\t"));
        out.push('\n');
    }
    out
}
fn mutated_pack(offset: usize, byte: u8) -> Vec<u8> {
    let mut buf = FIXTURE.to_vec();
    buf[offset] = byte;
    buf
}
#[test]
fn certified_r015_commit_is_exact() {
    assert_eq!(BASE_SHA, "3158ee20a894461df5402335b7db5509db9abd02");
}

#[test]
fn frozen_r011_to_r015_statuses_remain_absent() {
    assert!(frozen_contract());
}

#[test]
fn canonical_manifest_has_twelve_rows() {
    assert_eq!(rows(R16, HEADER, 17).unwrap().len(), 12);
}

#[test]
fn canonical_pack_has_twelve_payloads() {
    assert_eq!(decode_pack(FIXTURE).unwrap().len(), 12);
}

#[test]
fn canonical_bundle_has_correct_root_digest() {
    assert_eq!(sha256_hex(FIXTURE), PACK_SHA);
}

#[test]
fn full_bundle_only_qualifies_for_research_review() {
    assert_eq!(canonical(), Decision::ResearchReviewOnly);
    println!("R16_WITNESS version=1 kind=offline_mock_byte_bundle records=12 real_external_evidence=0 native_gates_unmet=12 decision=RESEARCH_REVIEW_ONLY outcome=PASS");
}

#[test]
fn empty_pack_is_invalid() {
    assert_eq!(assess(&[], R16, true, true), Decision::Invalid);
}

#[test]
fn missing_magic_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(0, b'X'), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn incorrect_magic_suffix_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(7, b'X'), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn incorrect_declared_count_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(8, 11), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn oversize_declared_count_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(9, 3), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn truncated_header_is_invalid() {
    assert_eq!(assess(&FIXTURE[..9], R16, true, true), Decision::Invalid);
}

#[test]
fn truncated_payload_is_invalid() {
    assert_eq!(
        assess(&FIXTURE[..FIXTURE.len() - 1], R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn trailing_byte_is_invalid() {
    let mut b = FIXTURE.to_vec();
    b.push(0);
    assert_eq!(assess(&b, R16, true, true), Decision::Invalid);
}

#[test]
fn zero_length_payload_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(10, 0), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn oversize_payload_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(11, 2), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn first_payload_byte_tampering_is_invalid() {
    assert_eq!(
        assess(&mutated_pack(12, b'X'), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn last_payload_byte_tampering_is_invalid() {
    let n = FIXTURE.len() - 1;
    assert_eq!(
        assess(&mutated_pack(n, 0), R16, true, true),
        Decision::Invalid
    );
}

#[test]
fn payloads_include_non_utf8_octets() {
    assert!(FIXTURE.contains(&0xff));
    assert!(FIXTURE.contains(&0x80));
}

#[test]
fn payloads_include_nul_and_crlf() {
    assert!(FIXTURE.contains(&0));
    assert!(FIXTURE.windows(2).any(|w| w == &b"\r\n"[..]));
}

#[test]
fn manifest_header_mismatch_is_invalid() {
    let forged = R16.replacen("id\tgate_id", "id\tnot_gate", 1);
    assert_eq!(assess(FIXTURE, &forged, true, true), Decision::Invalid);
}

#[test]
fn manifest_missing_row_is_invalid() {
    let mut lines: Vec<&str> = R16.lines().collect();
    lines.pop();
    let forged = format!("{}\n", lines.join("\n"));
    assert_eq!(assess(FIXTURE, &forged, true, true), Decision::Invalid);
}

#[test]
fn manifest_extra_row_is_invalid() {
    let mut forged = R16.to_owned();
    forged.push_str(R16.lines().nth(1).unwrap());
    forged.push('\n');
    assert_eq!(assess(FIXTURE, &forged, true, true), Decision::Invalid);
}

#[test]
fn manifest_corrupt_tsv_row_is_invalid() {
    let forged = R16.replacen("\tR11-G01\t", "\tR11-G01\tBROKEN\t", 1);
    assert_eq!(assess(FIXTURE, &forged, true, true), Decision::Invalid);
}

#[test]
fn wrong_bundle_identity_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 0, "FORGED"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_gate_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 1, "R11-G99"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_evidence_link_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 2, "R12-E99"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_integrity_link_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 3, "R13-L99"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_authentication_link_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 4, "R14-A99"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_verifier_link_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 5, "R15-V99"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_area_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 6, "FORGED_AREA"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_baseline_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 7, "deadbeef"), true, true),
        Decision::Invalid
    );
}

#[test]
fn forged_admitted_status_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 8, "VERIFIED"), true, true),
        Decision::Invalid
    );
}

#[test]
fn forged_scope_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 9, "NATIVE"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_offset_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 10, "99999"), true, true),
        Decision::Invalid
    );
}

#[test]
fn wrong_length_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 11, "2"), true, true),
        Decision::Invalid
    );
}

#[test]
fn forged_payload_digest_is_invalid() {
    assert_eq!(
        assess(
            FIXTURE,
            &mutated_manifest(
                0,
                12,
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
            true,
            true
        ),
        Decision::Invalid
    );
}

#[test]
fn forged_producer_is_invalid() {
    assert_eq!(
        assess(
            FIXTURE,
            &mutated_manifest(0, 13, "trusted-operator"),
            true,
            true
        ),
        Decision::Invalid
    );
}

#[test]
fn forged_signature_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 14, "SIGNED"), true, true),
        Decision::Invalid
    );
}

#[test]
fn forged_trust_root_is_invalid() {
    assert_eq!(
        assess(FIXTURE, &mutated_manifest(0, 15, "REAL_ROOT"), true, true),
        Decision::Invalid
    );
}

#[test]
fn missing_ci_blocks_even_mock_review() {
    assert_eq!(assess(FIXTURE, R16, false, true), Decision::Blocked);
}

#[test]
fn missing_reviewer_approval_blocks_even_mock_review() {
    assert_eq!(assess(FIXTURE, R16, true, false), Decision::Blocked);
}

#[test]
fn missing_ci_and_approval_blocks_mock_review() {
    assert_eq!(assess(FIXTURE, R16, false, false), Decision::Blocked);
}

#[test]
fn no_native_api_is_exported() {
    assert!(!include_str!("../src/lib.rs").contains("resource_task_r16"));
    assert!(!include_str!("../Cargo.toml").contains("r16_offline_bundle_manifest"));
}
