//! NORDOI R0.17 — offline dual-decoder / Python-Rust replay reproducibility.
//! Public TEST-ONLY bytes; local consistency is NOT third-party evidence verification.
//! RESEARCH_ONLY; no real signatures, trust root, native execution, or deployment.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "a5d7e9669a9b6d4be6a47bed17756946df24c9bb";
const PACK_SHA: &str = "f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393";
const TRANSCRIPT_SHA: &str = "1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab";
const PACK: &[u8] = include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin");
const TRANSCRIPT: &[u8] =
    include_bytes!("../research/fixtures/r17_independent_replay_transcript_v1.bin");
const R11: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const R12: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const R13: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const R14: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const R15: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const R16: &str = include_str!("../governance/r16_offline_bundle_manifest_v1.tsv");
const R17: &str = include_str!("../governance/r17_replay_registry_v1.tsv");
const M16: &str = "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tverification_id\tarea\tbaseline_sha\tadmitted_status\tscope\toffset\tlength\tfixture_sha256\tmock_producer\tsignature_ref\ttrust_root\tlimitation";
const M17: &str = "id\tbundle_id\tgate_id\tarea\tbaseline_sha\tstatus\tscope\treplay_mode\tpayload_offset\tpayload_length\tpayload_sha256\ttranscript_sha256\texternal_verifier\ttrust_root\tlimitation";
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplayRecord {
    index: usize,
    offset: usize,
    length: usize,
    digest: [u8; 32],
}

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Invalid,
    Blocked,
    ResearchReviewOnly,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for b in certified_sha256::sha256(bytes) {
        write!(&mut result, "{b:02x}").expect("writing to String cannot fail");
    }
    result
}

fn digest_hex(digest: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for &b in digest {
        write!(&mut result, "{b:02x}").expect("String write cannot fail");
    }
    result
}

fn tabular<'a>(text: &'a str, header: &str, columns: usize) -> Option<Vec<Vec<&'a str>>> {
    // str::lines() strips ordinary CRLF record terminators on Windows.
    // Embedded carriage returns remain malformed and must fail closed.
    let mut lines = text.lines();
    if lines.next()? != header {
        return None;
    }
    let mut result = Vec::new();
    for line in lines {
        if line.contains('\r') {
            return None;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != columns || fields.iter().any(|v| v.trim().is_empty()) {
            return None;
        }
        result.push(fields);
    }
    Some(result)
}

fn u16_at(bytes: &[u8], pos: usize) -> Option<usize> {
    let b = bytes.get(pos..pos.checked_add(2)?)?;
    Some(usize::from(u16::from_le_bytes([b[0], b[1]])))
}

// Implementation A: sequential wire traversal using a moving cursor and checked arithmetic.
fn cursor_decode(bytes: &[u8]) -> Option<Vec<ReplayRecord>> {
    if bytes.get(..8)? != &b"NDR16PK1"[..] || u16_at(bytes, 8)? != 12 {
        return None;
    }
    let mut pos = 10usize;
    let mut result = Vec::new();
    for index in 1..=12 {
        let size = u16_at(bytes, pos)?;
        if !(1..=256).contains(&size) {
            return None;
        }
        pos = pos.checked_add(2)?;
        let data = bytes.get(pos..pos.checked_add(size)?)?;
        result.push(ReplayRecord {
            index,
            offset: pos,
            length: size,
            digest: certified_sha256::sha256(data),
        });
        pos = pos.checked_add(size)?;
    }
    if pos != bytes.len() {
        return None;
    }
    Some(result)
}

// Implementation B: manifest-directed random access, separately checking prefix and exact coverage.
fn indexed_decode(bytes: &[u8], manifest: &str) -> Option<Vec<ReplayRecord>> {
    if bytes.get(..8)? != &b"NDR16PK1"[..] || u16_at(bytes, 8)? != 12 {
        return None;
    }
    let rows = tabular(manifest, M16, 17)?;
    if rows.len() != 12 {
        return None;
    }
    let mut previous_end = 10usize;
    let mut result = Vec::new();
    for (n, row) in rows.iter().enumerate() {
        let index = n + 1;
        let offset: usize = row[10].parse().ok()?;
        let length: usize = row[11].parse().ok()?;
        if !(1..=256).contains(&length) || offset != previous_end.checked_add(2)? {
            return None;
        }
        if u16_at(bytes, previous_end)? != length {
            return None;
        }
        let data = bytes.get(offset..offset.checked_add(length)?)?;
        let digest = certified_sha256::sha256(data);
        if row[0] != format!("R16-B{index:02}")
            || row[1] != format!("R11-G{index:02}")
            || row[2] != format!("R12-E{index:02}")
            || row[3] != format!("R13-L{index:02}")
            || row[4] != format!("R14-A{index:02}")
            || row[5] != format!("R15-V{index:02}")
            || row[6] != AREAS[n]
            || row[7] != "3158ee20a894461df5402335b7db5509db9abd02"
            || row[8] != "ABSENT"
            || row[9] != "RESEARCH_ONLY"
            || row[12] != sha256_hex(data)
            || row[13] != format!("MOCK-PRODUCER-{index:02}")
            || row[14] != "NONE"
            || row[15] != "NONE"
            || row[16].len() < 65
        {
            return None;
        }
        result.push(ReplayRecord {
            index,
            offset,
            length,
            digest,
        });
        previous_end = offset.checked_add(length)?;
    }
    if previous_end != bytes.len() {
        return None;
    }
    Some(result)
}

fn historic_boundaries() -> bool {
    let Some(gates) = tabular(
        R11,
        "id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation",
        6,
    ) else {
        return false;
    };
    let Some(evidence) = tabular(R12, "id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation", 9) else { return false; };
    let Some(integrity) = tabular(R13, "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation", 16) else { return false; };
    let Some(authentication) = tabular(R14, "id\tgate_id\tevidence_id\tintegrity_id\tarea\tbaseline_sha\tstatus\tscope\tproducer_key\treviewer_key\ttrust_root\tauthentication\trevocation\tlimitation", 14) else { return false; };
    let Some(verification) = tabular(R15, "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tarea\tbaseline_sha\tstatus\tscope\tpolicy\tverifier_ref\ttrust_root\tlimitation", 13) else { return false; };
    if [
        gates.len(),
        evidence.len(),
        integrity.len(),
        authentication.len(),
        verification.len(),
    ] != [12; 5]
    {
        return false;
    }
    for n in 0..12 {
        if gates[n][0] != format!("R11-G{:02}", n + 1)
            || gates[n][1] != AREAS[n]
            || gates[n][2] != "UNMET"
            || gates[n][3] != "RESEARCH_ONLY"
            || evidence[n][0] != format!("R12-E{:02}", n + 1)
            || evidence[n][1] != gates[n][0]
            || evidence[n][4] != "ABSENT"
            || evidence[n][5] != "RESEARCH_ONLY"
            || evidence[n][6] != "NONE"
            || evidence[n][7] != "NONE"
            || integrity[n][0] != format!("R13-L{:02}", n + 1)
            || integrity[n][1] != gates[n][0]
            || integrity[n][5] != "ABSENT"
            || integrity[n][6] != "RESEARCH_ONLY"
            || authentication[n][0] != format!("R14-A{:02}", n + 1)
            || authentication[n][1] != gates[n][0]
            || authentication[n][6] != "ABSENT"
            || authentication[n][7] != "RESEARCH_ONLY"
            || verification[n][0] != format!("R15-V{:02}", n + 1)
            || verification[n][1] != gates[n][0]
            || verification[n][7] != "ABSENT"
            || verification[n][8] != "RESEARCH_ONLY"
        {
            return false;
        }
    }
    true
}

fn validate_registry(registry: &str, records: &[ReplayRecord]) -> bool {
    let Some(rows) = tabular(registry, M17, 15) else {
        return false;
    };
    if rows.len() != 12 || records.len() != 12 {
        return false;
    }
    for (n, row) in rows.iter().enumerate() {
        let r = &records[n];
        if row[0] != format!("R17-R{:02}", n + 1)
            || row[1] != format!("R16-B{:02}", n + 1)
            || row[2] != format!("R11-G{:02}", n + 1)
            || row[3] != AREAS[n]
            || row[4] != BASE
            || row[5] != "ABSENT"
            || row[6] != "RESEARCH_ONLY"
            || row[7] != "DUAL_LOCAL_DECODE"
            || row[8] != r.offset.to_string()
            || row[9] != r.length.to_string()
            || row[10] != digest_hex(&r.digest)
            || row[11] != TRANSCRIPT_SHA
            || row[12] != "NONE"
            || row[13] != "NONE"
            || row[14].len() < 65
        {
            return false;
        }
    }
    true
}

fn transcript_from(records: &[ReplayRecord]) -> Option<Vec<u8>> {
    if records.len() != 12 {
        return None;
    }
    let mut result = Vec::with_capacity(595);
    result.extend_from_slice(b"NDR17RPL1");
    result.extend_from_slice(BASE.as_bytes());
    result.extend_from_slice(&12u16.to_le_bytes());
    let raw_root = certified_sha256::sha256(PACK);
    result.extend_from_slice(&raw_root);
    for (n, r) in records.iter().enumerate() {
        if r.index != n + 1 {
            return None;
        }
        result.extend_from_slice(&u16::try_from(r.index).ok()?.to_le_bytes());
        result.extend_from_slice(&u32::try_from(r.offset).ok()?.to_le_bytes());
        result.extend_from_slice(&u16::try_from(r.length).ok()?.to_le_bytes());
        result.extend_from_slice(&r.digest);
    }
    let checksum = certified_sha256::sha256(&result);
    result.extend_from_slice(&checksum);
    Some(result)
}

fn transcript_valid(bytes: &[u8], records: &[ReplayRecord]) -> bool {
    if bytes.len() != 595 || bytes.get(..9) != Some(&b"NDR17RPL1"[..]) {
        return false;
    }
    if bytes.get(9..49) != Some(BASE.as_bytes()) {
        return false;
    }
    if bytes.get(49..51) != Some(&12u16.to_le_bytes()[..]) {
        return false;
    }
    let root = certified_sha256::sha256(PACK);
    if bytes.get(51..83) != Some(&root[..]) {
        return false;
    }
    if bytes.get(563..595) != Some(&certified_sha256::sha256(&bytes[..563])[..]) {
        return false;
    }
    if sha256_hex(bytes) != TRANSCRIPT_SHA {
        return false;
    }
    transcript_from(records).as_deref() == Some(bytes)
}

fn assess(
    bytes: &[u8],
    manifest: &str,
    registry: &str,
    transcript: &[u8],
    ci: bool,
    review: bool,
) -> Decision {
    if !historic_boundaries() || sha256_hex(bytes) != PACK_SHA {
        return Decision::Invalid;
    }
    let Some(cursor) = cursor_decode(bytes) else {
        return Decision::Invalid;
    };
    let Some(indexed) = indexed_decode(bytes, manifest) else {
        return Decision::Invalid;
    };
    if cursor != indexed
        || !validate_registry(registry, &cursor)
        || !transcript_valid(transcript, &cursor)
    {
        return Decision::Invalid;
    }
    if !ci || !review {
        return Decision::Blocked;
    }
    Decision::ResearchReviewOnly // NEVER a native authority grant.
}

fn canonical() -> Decision {
    assess(PACK, R16, R17, TRANSCRIPT, true, true)
}
fn replace_cell(
    text: &str,
    header: &str,
    count: usize,
    row: usize,
    col: usize,
    value: &str,
) -> String {
    let mut table: Vec<Vec<String>> = tabular(text, header, count)
        .expect("frozen fixture")
        .iter()
        .map(|line| line.iter().map(|cell| (*cell).to_owned()).collect())
        .collect();
    table[row][col] = value.to_owned();
    let mut result = format!("{header}\n");
    for fields in table {
        result.push_str(&fields.join("\t"));
        result.push('\n');
    }
    result
}
fn altered_blob(pos: usize, byte: u8) -> Vec<u8> {
    let mut result = PACK.to_vec();
    result[pos] = byte;
    result
}

#[test]
fn certified_r016_commit_is_exact() {
    assert_eq!(BASE, "a5d7e9669a9b6d4be6a47bed17756946df24c9bb");
}
#[test]
fn frozen_native_and_evidence_statuses_remain_blocked() {
    assert!(historic_boundaries());
}
#[test]
fn canonical_pack_root_is_frozen() {
    assert_eq!(sha256_hex(PACK), PACK_SHA);
}
#[test]
fn canonical_manifest_has_twelve_rows() {
    assert_eq!(tabular(R16, M16, 17).unwrap().len(), 12);
    // Both LF and Git's Windows CRLF checkout must parse identically.
    let crlf = R16.replace("\r\n", "\n").replace('\n', "\r\n");
    assert_eq!(tabular(&crlf, M16, 17).unwrap().len(), 12);
    let registry_crlf = R17.replace("\r\n", "\n").replace('\n', "\r\n");
    assert_eq!(tabular(&registry_crlf, M17, 15).unwrap().len(), 12);
    // An embedded CR inside a field is still invalid.
    let malformed = R16
        .replace("\r\n", "\n")
        .replacen("R16-B01", "R16-\rB01", 1);
    assert!(tabular(&malformed, M16, 17).is_none());
}
#[test]
fn canonical_registry_has_twelve_absent_rows() {
    let r = tabular(R17, M17, 15).unwrap();
    assert_eq!(r.len(), 12);
    assert!(r
        .iter()
        .all(|row| row[5] == "ABSENT" && row[12] == "NONE" && row[13] == "NONE"));
}
#[test]
fn canonical_transcript_has_fixed_length() {
    assert_eq!(TRANSCRIPT.len(), 595);
}
#[test]
fn canonical_transcript_has_fixed_root() {
    assert_eq!(sha256_hex(TRANSCRIPT), TRANSCRIPT_SHA);
}
#[test]
fn canonical_transcript_has_correct_magic() {
    assert_eq!(&TRANSCRIPT[..9], b"NDR17RPL1");
}
#[test]
fn canonical_transcript_binds_commit() {
    assert_eq!(&TRANSCRIPT[9..49], BASE.as_bytes());
}
#[test]
fn canonical_transcript_binds_pack_digest() {
    assert_eq!(&TRANSCRIPT[51..83], &certified_sha256::sha256(PACK)[..]);
}
#[test]
fn canonical_transcript_binds_count() {
    assert_eq!(&TRANSCRIPT[49..51], &12u16.to_le_bytes());
}
#[test]
fn canonical_transcript_checksum_matches_prefix() {
    assert_eq!(
        &TRANSCRIPT[563..],
        &certified_sha256::sha256(&TRANSCRIPT[..563])[..]
    );
}
#[test]
fn cursor_decoder_reads_twelve_records() {
    assert_eq!(cursor_decode(PACK).unwrap().len(), 12);
}
#[test]
fn indexed_decoder_reads_twelve_records() {
    assert_eq!(indexed_decode(PACK, R16).unwrap().len(), 12);
}
#[test]
fn both_decoders_agree_on_records() {
    assert_eq!(cursor_decode(PACK), indexed_decode(PACK, R16));
}
#[test]
fn rust_replay_rebuilds_golden_bytes() {
    assert_eq!(
        transcript_from(&cursor_decode(PACK).unwrap())
            .unwrap()
            .as_slice(),
        TRANSCRIPT
    );
}
#[test]
fn rust_replay_is_deterministic() {
    let records = indexed_decode(PACK, R16).unwrap();
    assert_eq!(transcript_from(&records), transcript_from(&records));
}
#[test]
fn python_reference_digest_is_anchored() {
    assert!(transcript_valid(TRANSCRIPT, &cursor_decode(PACK).unwrap()));
}
#[test]
fn payloads_cover_non_utf8_nul_and_crlf() {
    assert!(PACK.windows(4).any(|p| p == b"\xff\x80\r\n"));
    assert!(PACK.contains(&0));
}
#[test]
fn full_replay_stays_research_only() {
    assert_eq!(canonical(), Decision::ResearchReviewOnly);
    println!("R17_WITNESS version=1 kind=dual_local_replay records=12 python_hashlib_reference=1 rust_frozen_sha256=1 external_verifiers=0 native_gates_unmet=12 decision=RESEARCH_REVIEW_ONLY outcome=PASS");
}
#[test]
fn missing_synthetic_ci_blocks() {
    assert_eq!(
        assess(PACK, R16, R17, TRANSCRIPT, false, true),
        Decision::Blocked
    );
}
#[test]
fn missing_synthetic_review_blocks() {
    assert_eq!(
        assess(PACK, R16, R17, TRANSCRIPT, true, false),
        Decision::Blocked
    );
}
#[test]
fn missing_both_flags_blocks() {
    assert_eq!(
        assess(PACK, R16, R17, TRANSCRIPT, false, false),
        Decision::Blocked
    );
}
#[test]
fn empty_pack_is_invalid() {
    assert!(cursor_decode(&[]).is_none());
}
#[test]
fn truncated_header_is_invalid() {
    assert!(cursor_decode(&PACK[..9]).is_none());
}
#[test]
fn invalid_pack_magic_is_rejected() {
    assert!(cursor_decode(&altered_blob(0, b'X')).is_none());
}
#[test]
fn invalid_pack_count_is_rejected() {
    assert!(cursor_decode(&altered_blob(8, 11)).is_none());
}
#[test]
fn zero_length_is_rejected() {
    assert!(cursor_decode(&altered_blob(10, 0)).is_none());
}
#[test]
fn oversize_length_is_rejected() {
    let mut modified = PACK.to_vec();
    modified[10] = 1;
    modified[11] = 1;
    assert!(cursor_decode(&modified).is_none());
}
#[test]
fn truncated_payload_is_rejected() {
    assert!(cursor_decode(&PACK[..PACK.len() - 1]).is_none());
}
#[test]
fn trailing_pack_byte_is_rejected() {
    let mut b = PACK.to_vec();
    b.push(0);
    assert!(cursor_decode(&b).is_none());
}
#[test]
fn malformed_manifest_header_is_invalid() {
    let m = R16.replacen("id\tgate_id", "wrong\tgate_id", 1);
    assert_eq!(
        assess(PACK, &m, R17, TRANSCRIPT, true, true),
        Decision::Invalid
    );
}
#[test]
fn forged_manifest_offset_is_invalid() {
    let m = replace_cell(R16, M16, 17, 0, 10, "13");
    assert!(indexed_decode(PACK, &m).is_none());
}
#[test]
fn forged_manifest_length_is_invalid() {
    let m = replace_cell(R16, M16, 17, 0, 11, "46");
    assert!(indexed_decode(PACK, &m).is_none());
}
#[test]
fn forged_manifest_digest_is_invalid() {
    let m = replace_cell(R16, M16, 17, 0, 12, &"0".repeat(64));
    assert!(indexed_decode(PACK, &m).is_none());
}
#[test]
fn reordered_manifest_is_invalid() {
    let lines: Vec<&str> = R16.lines().collect();
    let mut swap = lines.clone();
    swap.swap(1, 2);
    let m = format!("{}\n", swap.join("\n"));
    assert!(indexed_decode(PACK, &m).is_none());
}
#[test]
fn missing_manifest_row_is_invalid() {
    let lines: Vec<&str> = R16.lines().collect();
    let m = format!("{}\n", lines[..lines.len() - 1].join("\n"));
    assert!(indexed_decode(PACK, &m).is_none());
}
#[test]
fn forged_registry_baseline_is_invalid() {
    let m = replace_cell(R17, M17, 15, 0, 4, &"0".repeat(40));
    assert_eq!(
        assess(PACK, R16, &m, TRANSCRIPT, true, true),
        Decision::Invalid
    );
}
#[test]
fn forged_registry_trust_root_is_invalid() {
    let m = replace_cell(R17, M17, 15, 0, 13, "UNTRUSTED-ROOT");
    assert_eq!(
        assess(PACK, R16, &m, TRANSCRIPT, true, true),
        Decision::Invalid
    );
}
#[test]
fn forged_registry_admission_is_invalid() {
    let m = replace_cell(R17, M17, 15, 0, 5, "VERIFIED");
    assert_eq!(
        assess(PACK, R16, &m, TRANSCRIPT, true, true),
        Decision::Invalid
    );
}
#[test]
fn forged_registry_digest_is_invalid() {
    let m = replace_cell(R17, M17, 15, 0, 10, &"a".repeat(64));
    assert_eq!(
        assess(PACK, R16, &m, TRANSCRIPT, true, true),
        Decision::Invalid
    );
}
#[test]
fn transcript_tamper_is_invalid_even_with_updated_tail() {
    let mut forged = TRANSCRIPT.to_vec();
    forged[90] ^= 1;
    let checksum = certified_sha256::sha256(&forged[..563]);
    forged[563..].copy_from_slice(&checksum);
    assert_eq!(
        assess(PACK, R16, R17, &forged, true, true),
        Decision::Invalid
    );
}
#[test]
fn transcript_truncation_is_invalid() {
    assert_eq!(
        assess(
            PACK,
            R16,
            R17,
            &TRANSCRIPT[..TRANSCRIPT.len() - 1],
            true,
            true
        ),
        Decision::Invalid
    );
}
#[test]
fn no_native_execution_api_is_exported() {
    assert!(!include_str!("../src/lib.rs").contains("resource_task_r17"));
    assert!(!include_str!("../Cargo.toml").contains("resource_task_r17"));
    assert_eq!(canonical(), Decision::ResearchReviewOnly);
}
