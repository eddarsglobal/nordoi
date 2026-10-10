//! NORDOI R0.18 — adversarial differential replay of immutable public research fixtures.
//! TEST-ONLY / RESEARCH_ONLY: no authenticated evidence, external review or native authority.
#[path = "../src/effect_audit/hash.rs"]
mod certified_sha256;

use std::fmt::Write;

const BASE: &str = "c5f76bc2e393f95f38552778f870fac4be8206c3";
const PACK_SHA: &str = "f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393";
const TRANS_SHA: &str = "1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab";
const MAN_SHA: &str = "f4dfbb95a61e1a9184ca459f7279a0c686166de10411554c8364445cb75dcb24";
const REG_SHA: &str = "c0d649990dbeb318408a8e82af34478fc7cf1f50834e8f69a05d0464d1e2e242";
const CAMPAIGN_SHA: &str = "281ea03f893b54e2c03d12e7df1236da249ff1d8a61cbe0a8f5b4fb18d26b783";
const RESULT_SHA: &str = "9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293";
const PACK: &[u8] = include_bytes!("../research/fixtures/r16_synthetic_bundle_v1.bin");
const TRANSCRIPT: &[u8] =
    include_bytes!("../research/fixtures/r17_independent_replay_transcript_v1.bin");
const MANIFEST: &str = include_str!("../governance/r16_offline_bundle_manifest_v1.tsv");
const REGISTRY: &str = include_str!("../governance/r17_replay_registry_v1.tsv");
const CAMPAIGN: &str = include_str!("../governance/r18_fault_campaign_v1.tsv");
const GOLDEN: &[u8] = include_bytes!("../research/fixtures/r18_differential_fault_results_v1.bin");
const GATES: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");
const EVIDENCE: &str = include_str!("../governance/r12_evidence_ledger_v1.tsv");
const INTEGRITY: &str = include_str!("../governance/r13_integrity_chain_v1.tsv");
const AUTH: &str = include_str!("../governance/r14_authentication_registry_v1.tsv");
const VERIFY: &str = include_str!("../governance/r15_trust_policy_registry_v1.tsv");
const M16: &str = "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tverification_id\tarea\tbaseline_sha\tadmitted_status\tscope\toffset\tlength\tfixture_sha256\tmock_producer\tsignature_ref\ttrust_root\tlimitation";
const M17: &str = "id\tbundle_id\tgate_id\tarea\tbaseline_sha\tstatus\tscope\treplay_mode\tpayload_offset\tpayload_length\tpayload_sha256\ttranscript_sha256\texternal_verifier\ttrust_root\tlimitation";
const C18: &str = "id\tdomain\toperation\tposition\targument\treader_a\treader_b\tmutated_sha256\texpected\tscope\tlimitation";
const EXPECTED_AREAS: [&str; 12] = [
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
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

struct CampaignReport {
    bytes: Vec<u8>,
    total: usize,
    structurally_readable: usize,
    rejected_by_reader_a: usize,
    disagreements: usize,
    every_fault_invalid: bool,
    checked_reader_a: usize,
    checked_reader_b: usize,
    checked_digests: usize,
}

fn shahex(b: &[u8]) -> String {
    let mut s = String::with_capacity(64);
    for byte in certified_sha256::sha256(b) {
        write!(&mut s, "{byte:02x}").expect("formatting into a String cannot fail");
    }
    s
}

fn raw_hash(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, dst) in out.iter_mut().enumerate() {
        *dst = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn normalized_lf(input: &str) -> Option<String> {
    let result = input.replace("\r\n", "\n");
    if result.contains('\r') {
        return None;
    }
    Some(result)
}

fn table<'a>(source: &'a str, header: &str, columns: usize) -> Option<Vec<Vec<&'a str>>> {
    // `lines()` strips ordinary CRLF terminators without changing each field's content.
    // Internal CR bytes remain malformed, including when the checkout uses CRLF.
    normalized_lf(source)?;
    let mut lines = source.lines();
    if lines.next()? != header {
        return None;
    }
    let mut rows = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.len() != columns || cells.iter().any(|cell| cell.trim().is_empty()) {
            return None;
        }
        rows.push(cells);
    }
    Some(rows)
}

fn u16at(bytes: &[u8], pos: usize) -> Option<usize> {
    let value = bytes.get(pos..pos.checked_add(2)?)?;
    Some(usize::from(u16::from_le_bytes([value[0], value[1]])))
}

// Reader A: sequential length-prefix traversal, independent of manifest row offsets.
fn decode_a(pack: &[u8]) -> Option<Vec<Record>> {
    if pack.get(..8)? != b"NDR16PK1" || u16at(pack, 8)? != 12 {
        return None;
    }
    let mut cursor = 10usize;
    let mut records = Vec::new();
    for index in 1..=12 {
        let size = u16at(pack, cursor)?;
        cursor = cursor.checked_add(2)?;
        if !(1..=256).contains(&size) {
            return None;
        }
        let payload = pack.get(cursor..cursor.checked_add(size)?)?;
        records.push(Record {
            index,
            offset: cursor,
            length: size,
            digest: certified_sha256::sha256(payload),
        });
        cursor = cursor.checked_add(size)?;
    }
    if cursor != pack.len() {
        return None;
    }
    Some(records)
}

// Reader B: independent manifest-directed random access with exact coverage.
fn decode_b(pack: &[u8], manifest: &str) -> Option<Vec<Record>> {
    if shahex(normalized_lf(manifest)?.as_bytes()) != MAN_SHA {
        return None;
    }
    if pack.get(..8)? != b"NDR16PK1" || pack.get(8..10)? != &[12u8, 0u8][..] {
        return None;
    }
    let rows = table(manifest, M16, 17)?;
    if rows.len() != 12 {
        return None;
    }
    let mut cursor = 10usize;
    let mut records = Vec::new();
    for (n, row) in rows.iter().enumerate() {
        let offset: usize = row[10].parse().ok()?;
        let size: usize = row[11].parse().ok()?;
        if !(1..=256).contains(&size) || offset != cursor.checked_add(2)? {
            return None;
        }
        if u16at(pack, cursor)? != size {
            return None;
        }
        let payload = pack.get(offset..offset.checked_add(size)?)?;
        records.push(Record {
            index: n + 1,
            offset,
            length: size,
            digest: certified_sha256::sha256(payload),
        });
        cursor = offset.checked_add(size)?;
    }
    if cursor != pack.len() {
        return None;
    }
    Some(records)
}

fn mutation(input: &[u8], operation: &str, pos: usize, argument: usize) -> Option<Vec<u8>> {
    if input.len() > 16_384 || argument > 65_535 || pos > input.len() {
        return None;
    }
    let mut changed = input.to_vec();
    match operation {
        "XOR" => {
            *changed.get_mut(pos)? ^= u8::try_from(argument).ok()?;
        }
        "ZERO" => {
            *changed.get_mut(pos)? = 0;
        }
        "CUT" => changed.truncate(pos),
        "ADD" => changed.push(u8::try_from(argument).ok()?),
        "SWAP" => {
            if pos >= changed.len() || argument >= changed.len() {
                return None;
            }
            changed.swap(pos, argument);
        }
        _ => return None,
    }
    if changed == input {
        return None;
    } // No vacuous or silently repaired corruption.
    Some(changed)
}

fn source_kind(name: &str) -> Option<u8> {
    match name {
        "PACK" => Some(0),
        "TRANSCRIPT" => Some(1),
        "MANIFEST" => Some(2),
        "REGISTRY" => Some(3),
        _ => None,
    }
}

fn frozen_status() -> bool {
    let Some(g) = table(
        GATES,
        "id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation",
        6,
    ) else {
        return false;
    };
    let Some(e) = table(EVIDENCE, "id\tgate_id\tarea\tbaseline_sha\tstatus\tevidence_scope\tartifact_ref\treviewer_ref\tlimitation", 9) else { return false; };
    let Some(i) = table(INTEGRITY, "id\tgate_id\tevidence_id\tarea\tbaseline_sha\tstatus\tscope\tartifact_sha256\tprevious_chain_sha256\tchain_sha256\tproducer_ref\treviewer_ref\tsignature_ref\ttrust_root\trevocation\tlimitation", 16) else { return false; };
    let Some(a) = table(AUTH, "id\tgate_id\tevidence_id\tintegrity_id\tarea\tbaseline_sha\tstatus\tscope\tproducer_key\treviewer_key\ttrust_root\tauthentication\trevocation\tlimitation", 14) else { return false; };
    let Some(v) = table(VERIFY, "id\tgate_id\tevidence_id\tintegrity_id\tauthentication_id\tarea\tbaseline_sha\tstatus\tscope\tpolicy\tverifier_ref\ttrust_root\tlimitation", 13) else { return false; };
    let Some(m) = table(MANIFEST, M16, 17) else {
        return false;
    };
    let Some(r) = table(REGISTRY, M17, 15) else {
        return false;
    };
    if [
        g.len(),
        e.len(),
        i.len(),
        a.len(),
        v.len(),
        m.len(),
        r.len(),
    ] != [12; 7]
    {
        return false;
    }
    for n in 0..12 {
        let id = n + 1;
        if g[n][0] != format!("R11-G{id:02}")
            || g[n][1] != EXPECTED_AREAS[n]
            || g[n][2] != "UNMET"
            || g[n][3] != "RESEARCH_ONLY"
            || e[n][0] != format!("R12-E{id:02}")
            || e[n][4] != "ABSENT"
            || e[n][5] != "RESEARCH_ONLY"
            || i[n][0] != format!("R13-L{id:02}")
            || i[n][5] != "ABSENT"
            || i[n][6] != "RESEARCH_ONLY"
            || a[n][0] != format!("R14-A{id:02}")
            || a[n][6] != "ABSENT"
            || a[n][7] != "RESEARCH_ONLY"
            || v[n][0] != format!("R15-V{id:02}")
            || v[n][7] != "ABSENT"
            || v[n][8] != "RESEARCH_ONLY"
            || m[n][0] != format!("R16-B{id:02}")
            || m[n][8] != "ABSENT"
            || m[n][9] != "RESEARCH_ONLY"
            || r[n][0] != format!("R17-R{id:02}")
            || r[n][5] != "ABSENT"
            || r[n][6] != "RESEARCH_ONLY"
        {
            return false;
        }
    }
    true
}

fn canonical_roots() -> bool {
    shahex(PACK) == PACK_SHA
        && shahex(TRANSCRIPT) == TRANS_SHA
        && normalized_lf(MANIFEST).is_some_and(|v| shahex(v.as_bytes()) == MAN_SHA)
        && normalized_lf(REGISTRY).is_some_and(|v| shahex(v.as_bytes()) == REG_SHA)
        && normalized_lf(CAMPAIGN).is_some_and(|v| shahex(v.as_bytes()) == CAMPAIGN_SHA)
}

fn campaign() -> Option<CampaignReport> {
    if !canonical_roots() || !frozen_status() {
        return None;
    }
    let rows = table(CAMPAIGN, C18, 11)?;
    if rows.len() != 60 {
        return None;
    }
    let man = normalized_lf(MANIFEST)?;
    let reg = normalized_lf(REGISTRY)?;
    let frozen_a = decode_a(PACK)?;
    let frozen_b = decode_b(PACK, &man)?;
    if frozen_a != frozen_b || frozen_a.len() != 12 {
        return None;
    }
    for (n, record) in frozen_a.iter().enumerate() {
        if record.index != n + 1
            || !(1..=256).contains(&record.length)
            || record.digest
                != certified_sha256::sha256(
                    PACK.get(record.offset..record.offset.checked_add(record.length)?)?,
                )
        {
            return None;
        }
    }
    let mut result = Vec::with_capacity(2307);
    result.extend_from_slice(b"NDR18FLT1");
    result.extend_from_slice(BASE.as_bytes());
    result.extend_from_slice(&60u16.to_le_bytes());
    result.extend_from_slice(&raw_hash(PACK_SHA)?);
    result.extend_from_slice(&raw_hash(TRANS_SHA)?);
    let mut summary = CampaignReport {
        bytes: Vec::new(),
        total: 0,
        structurally_readable: 0,
        rejected_by_reader_a: 0,
        disagreements: 0,
        every_fault_invalid: true,
        checked_reader_a: 0,
        checked_reader_b: 0,
        checked_digests: 0,
    };
    for (n, row) in rows.iter().enumerate() {
        let index = n + 1;
        let kind = source_kind(row[1])?;
        let position: usize = row[3].parse().ok()?;
        let arg: usize = row[4].parse().ok()?;
        if row[0] != format!("R18-F{index:02}")
            || row[8] != "INVALID"
            || row[9] != "RESEARCH_ONLY"
            || row[10].len() < 65
        {
            return None;
        }
        let original: &[u8] = match kind {
            0 => PACK,
            1 => TRANSCRIPT,
            2 => man.as_bytes(),
            3 => reg.as_bytes(),
            _ => return None,
        };
        let changed = mutation(original, row[2], position, arg)?;
        if row[7] != shahex(&changed) {
            return None;
        }
        summary.checked_digests += 1;
        let pack: &[u8] = if kind == 0 { &changed } else { PACK };
        let manifest = if kind == 2 {
            std::str::from_utf8(&changed).ok()
        } else {
            Some(man.as_str())
        };
        let a = decode_a(pack);
        let b = manifest.and_then(|v| decode_b(pack, v));
        let readable_a = a.is_some();
        let readable_b = b.is_some();
        if row[5] != if readable_a { "PASS" } else { "REJECT" }
            || row[6] != if readable_b { "PASS" } else { "REJECT" }
        {
            return None;
        }
        summary.checked_reader_a += 1;
        summary.checked_reader_b += 1;
        if readable_a && readable_b {
            summary.structurally_readable += 1;
        }
        if !readable_a {
            summary.rejected_by_reader_a += 1;
        }
        if readable_a != readable_b {
            summary.disagreements += 1;
        }
        let trans: &[u8] = if kind == 1 { &changed } else { TRANSCRIPT };
        let registry: &[u8] = if kind == 3 { &changed } else { reg.as_bytes() };
        let invalid = shahex(pack) != PACK_SHA
            || shahex(trans) != TRANS_SHA
            || manifest
                .is_none_or(|v| normalized_lf(v).is_none_or(|s| shahex(s.as_bytes()) != MAN_SHA))
            || shahex(registry) != REG_SHA
            || a != b;
        summary.every_fault_invalid &= invalid;
        if !invalid {
            return None;
        }
        let mask = u8::from(readable_a) | (u8::from(readable_b) << 1);
        result.extend_from_slice(&u16::try_from(index).ok()?.to_le_bytes());
        result.push(kind);
        result.push(mask);
        result.extend_from_slice(&certified_sha256::sha256(&changed));
        summary.total += 1;
    }
    result.extend_from_slice(&certified_sha256::sha256(&result));
    summary.bytes = result;
    Some(summary)
}

fn anchored_report() -> Option<CampaignReport> {
    let report = campaign()?;
    if report.bytes.len() != 2307 || report.bytes != GOLDEN || shahex(GOLDEN) != RESULT_SHA {
        return None;
    }
    Some(report)
}

fn assess(ci: bool, review: bool) -> Decision {
    if anchored_report().is_none() {
        return Decision::Invalid;
    }
    if !ci || !review {
        return Decision::Blocked;
    }
    Decision::ResearchReviewOnly // Public synthetic research output; not a native authority grant.
}

#[test]
fn certified_r017_sha_is_exact() {
    assert_eq!(BASE, "c5f76bc2e393f95f38552778f870fac4be8206c3");
}

#[test]
fn frozen_pack_sha_is_exact() {
    assert_eq!(shahex(PACK), PACK_SHA);
}

#[test]
fn frozen_replay_sha_is_exact() {
    assert_eq!(shahex(TRANSCRIPT), TRANS_SHA);
}

#[test]
fn fault_campaign_has_sixty_entries() {
    assert_eq!(table(CAMPAIGN, C18, 11).unwrap().len(), 60);
}

#[test]
fn fault_campaign_checksum_is_frozen() {
    assert_eq!(
        shahex(normalized_lf(CAMPAIGN).unwrap().as_bytes()),
        CAMPAIGN_SHA
    );
}

#[test]
fn fault_results_have_fixed_length() {
    assert_eq!(GOLDEN.len(), 2307);
}

#[test]
fn fault_results_have_correct_magic() {
    assert_eq!(&GOLDEN[..9], b"NDR18FLT1");
}

#[test]
fn fault_results_are_anchored() {
    assert_eq!(shahex(GOLDEN), RESULT_SHA);
}

#[test]
fn fault_results_bind_certified_commit() {
    assert_eq!(&GOLDEN[9..49], BASE.as_bytes());
}

#[test]
fn fault_results_bind_both_input_roots() {
    assert_eq!(&GOLDEN[51..83], &raw_hash(PACK_SHA).unwrap()[..]);
    assert_eq!(&GOLDEN[83..115], &raw_hash(TRANS_SHA).unwrap()[..]);
}

#[test]
fn fault_results_bind_case_count() {
    assert_eq!(&GOLDEN[49..51], &60u16.to_le_bytes());
}

#[test]
fn canonical_packet_reader_a_has_twelve_records() {
    assert_eq!(decode_a(PACK).unwrap().len(), 12);
}

#[test]
fn canonical_packet_reader_b_has_twelve_records() {
    assert_eq!(decode_b(PACK, MANIFEST).unwrap().len(), 12);
}

#[test]
fn canonical_readers_agree_record_for_record() {
    assert_eq!(decode_a(PACK), decode_b(PACK, MANIFEST));
}

#[test]
fn canonical_transcript_has_valid_checksum() {
    assert_eq!(
        &TRANSCRIPT[563..],
        &certified_sha256::sha256(&TRANSCRIPT[..563])[..]
    );
}

#[test]
fn canonical_registry_is_frozen() {
    assert_eq!(shahex(normalized_lf(REGISTRY).unwrap().as_bytes()), REG_SHA);
}

#[test]
fn canonical_manifest_is_frozen() {
    assert_eq!(shahex(normalized_lf(MANIFEST).unwrap().as_bytes()), MAN_SHA);
}

#[test]
fn windows_crlf_is_normalized_before_hashing() {
    let crlf = MANIFEST.replace("\r\n", "\n").replace('\n', "\r\n");
    assert_eq!(shahex(normalized_lf(&crlf).unwrap().as_bytes()), MAN_SHA);
    assert_eq!(table(&crlf, M16, 17).unwrap().len(), 12);
}

#[test]
fn embedded_carriage_return_stays_rejected() {
    let broken = MANIFEST
        .replace("\r\n", "\n")
        .replacen("R16-B01", "R16-\rB01", 1);
    assert!(table(&broken, M16, 17).is_none());
}

#[test]
fn corrupt_packet_magic_is_rejected() {
    let broken = mutation(PACK, "XOR", 0, 1).unwrap();
    assert!(decode_a(&broken).is_none());
    assert!(decode_b(&broken, MANIFEST).is_none());
}

#[test]
fn wrong_packet_count_is_rejected() {
    let broken = mutation(PACK, "XOR", 8, 1).unwrap();
    assert!(decode_a(&broken).is_none());
    assert!(decode_b(&broken, MANIFEST).is_none());
}

#[test]
fn truncated_packet_is_rejected() {
    let broken = mutation(PACK, "CUT", PACK.len() - 1, 0).unwrap();
    assert!(decode_a(&broken).is_none());
    assert!(decode_b(&broken, MANIFEST).is_none());
}

#[test]
fn fault_mutations_are_nontrivial() {
    let broken = mutation(PACK, "XOR", 12, 1).unwrap();
    assert_ne!(broken, PACK);
    assert_ne!(shahex(&broken), PACK_SHA);
}

#[test]
fn fault_mutations_are_deterministic() {
    assert_eq!(mutation(PACK, "XOR", 12, 1), mutation(PACK, "XOR", 12, 1));
}

#[test]
fn fault_campaign_has_structurally_valid_tamper() {
    assert!(anchored_report().unwrap().structurally_readable >= 12);
}

#[test]
fn fault_campaign_has_decoder_rejections() {
    assert!(anchored_report().unwrap().rejected_by_reader_a >= 15);
}

#[test]
fn fault_campaign_has_differential_disagreements() {
    assert!(anchored_report().unwrap().disagreements >= 3);
}

#[test]
fn all_sixty_cases_match_golden_digest() {
    let r = anchored_report().unwrap();
    assert_eq!(shahex(&r.bytes), RESULT_SHA);
}

#[test]
fn all_sixty_cases_match_reader_a() {
    let r = anchored_report().unwrap();
    assert_eq!(r.checked_reader_a, 60);
}

#[test]
fn all_sixty_cases_match_reader_b() {
    let r = anchored_report().unwrap();
    assert_eq!(r.checked_reader_b, 60);
}

#[test]
fn all_sixty_cases_are_invalid_despite_rehashing() {
    let r = anchored_report().unwrap();
    assert!(r.every_fault_invalid);
    assert_eq!(r.checked_digests, 60);
}

#[test]
fn missing_ci_stays_blocked() {
    assert_eq!(assess(false, true), Decision::Blocked);
}

#[test]
fn missing_review_stays_blocked() {
    assert_eq!(assess(true, false), Decision::Blocked);
}

#[test]
fn missing_both_stays_blocked() {
    assert_eq!(assess(false, false), Decision::Blocked);
}

#[test]
fn full_public_fixture_only_enters_research_review() {
    assert_eq!(assess(true, true), Decision::ResearchReviewOnly);
    println!("R18_WITNESS version=1 kind=deterministic_adversarial_campaign faults=60 real_external_verifiers=0 native_gates_unmet=12 outcome=RESEARCH_REVIEW_ONLY");
}

#[test]
fn historic_statuses_remain_absent() {
    assert!(frozen_status());
}

#[test]
fn no_native_authority_or_external_verification() {
    assert_eq!(assess(true, true), Decision::ResearchReviewOnly);
    assert_eq!(anchored_report().unwrap().total, 60);
}

#[test]
fn golden_result_replay_is_deterministic() {
    assert_eq!(campaign().unwrap().bytes, campaign().unwrap().bytes);
}
