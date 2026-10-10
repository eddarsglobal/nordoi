//! NORDOI R0.10 — TEST-ONLY verification consolidation.
//! Historical source, witness and profile audit, not a runtime or formal proof.
//! R0.2 remains frozen; no implementation of a scheduler or new authority.

#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Command, Model, ModelError, Receipt, ScopeBudget, ScopeOutcome};
use std::collections::BTreeSet;

struct HistoricalSuite {
    id: &'static str,
    prefix: &'static str,
    count: usize,
    matrix: &'static str,
    source: &'static str,
}

fn suites() -> [HistoricalSuite; 7] {
    [
        HistoricalSuite {
            id: "R03",
            prefix: "R03-V",
            count: 9,
            matrix: include_str!("../governance/r03_state_space_obligations_v1.tsv"),
            source: include_str!("resource_task_r03.rs"),
        },
        HistoricalSuite {
            id: "R04",
            prefix: "R04-A",
            count: 12,
            matrix: include_str!("../governance/r04_adversarial_witnesses_v1.tsv"),
            source: include_str!("resource_task_r04.rs"),
        },
        HistoricalSuite {
            id: "R05",
            prefix: "R05-H",
            count: 14,
            matrix: include_str!("../governance/r05_hardening_witnesses_v1.tsv"),
            source: include_str!("resource_task_r05.rs"),
        },
        HistoricalSuite {
            id: "R06",
            prefix: "R06-C",
            count: 15,
            matrix: include_str!("../governance/r06_conformance_witnesses_v1.tsv"),
            source: include_str!("resource_task_r06.rs"),
        },
        HistoricalSuite {
            id: "R07",
            prefix: "R07-C",
            count: 18,
            matrix: include_str!("../governance/r07_multiscope_witnesses_v1.tsv"),
            source: include_str!("resource_task_r07.rs"),
        },
        HistoricalSuite {
            id: "R08",
            prefix: "R08-H",
            count: 26,
            matrix: include_str!("../governance/r08_hierarchy_witnesses_v1.tsv"),
            source: include_str!("resource_task_r08.rs"),
        },
        HistoricalSuite {
            id: "R09",
            prefix: "R09-B",
            count: 23,
            matrix: include_str!("../governance/r09_branch_boundary_witnesses_v1.tsv"),
            source: include_str!("resource_task_r09.rs"),
        },
    ]
}

fn parsed(source: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "#[test]" {
            let signature = lines.next().expect("missing function after #[test]").trim();
            let name = signature
                .strip_prefix("fn ")
                .expect("missing fn after #[test]")
                .split('(')
                .next()
                .expect("test name");
            assert!(!name.is_empty());
            names.push(name);
        }
    }
    names
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MatrixRow<'a> {
    id: &'a str,
    witness: &'a str,
    category: &'a str,
    limitation: &'a str,
}

fn parse_matrix(src: &str) -> Vec<MatrixRow<'_>> {
    let mut lines = src.lines();
    let header = lines.next().expect("header");
    assert!(
        header.starts_with("id\twitness\t"),
        "unexpected witness matrix header"
    );
    assert!(header.ends_with("\tlimitation"), "no declared limitations");
    lines
        .filter(|row| !row.trim().is_empty())
        .map(|line| {
            let parts: Vec<_> = line.split('\t').collect();
            assert_eq!(parts.len(), 4, "incorrect witness row shape");
            MatrixRow {
                id: parts[0],
                witness: parts[1],
                category: parts[2],
                limitation: parts[3],
            }
        })
        .collect()
}

fn material() -> Vec<u8> {
    let mut bytes = Vec::new();
    for suite in suites() {
        bytes.extend_from_slice(suite.id.as_bytes());
        bytes.extend_from_slice(&(suite.count as u64).to_be_bytes());
        for value in [suite.matrix, suite.source] {
            bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
            bytes.extend_from_slice(value.as_bytes());
        }
    }
    bytes
}

fn digest(bytes: &[u8]) -> u64 {
    // Noncryptographic deterministic fingerprint of this test's audit material;
    // NOT an attestation, authentication or substitute for git tree hashes.
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100_0000_01b3);
    }
    state
}

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

#[test]
fn historic_witness_count_is_exactly_117() {
    assert_eq!(suites().iter().map(|suite| suite.count).sum::<usize>(), 117);
}

#[test]
fn all_seven_historical_matrices_have_exact_row_counts() {
    for suite in suites() {
        assert_eq!(
            parse_matrix(suite.matrix).len(),
            suite.count,
            "{}",
            suite.id
        );
    }
}

#[test]
fn historical_governance_witnesses_biject_with_rust_tests_in_order() {
    for suite in suites() {
        let names = parsed(suite.source);
        let rows = parse_matrix(suite.matrix);
        assert_eq!(names.len(), suite.count, "{}", suite.id);
        let witnesses: Vec<_> = rows.iter().map(|row| row.witness).collect();
        assert_eq!(names, witnesses, "{}", suite.id);
    }
}

#[test]
fn cross_version_witness_identifiers_are_unique_and_ordered() {
    let mut seen = BTreeSet::new();
    for suite in suites() {
        for (i, row) in parse_matrix(suite.matrix).iter().enumerate() {
            assert_eq!(row.id, format!("{}{:02}", suite.prefix, i + 1));
            assert!(seen.insert(row.id.to_owned()), "duplicate {}", row.id);
        }
    }
    assert_eq!(seen.len(), 117);
}

#[test]
fn evidence_categories_and_limitations_are_not_blank() {
    for suite in suites() {
        for row in parse_matrix(suite.matrix) {
            assert!(!row.category.trim().is_empty(), "{}", row.id);
            assert!(
                row.limitation.trim().len() >= 15,
                "missing limitation: {}",
                row.id
            );
        }
    }
}

#[test]
fn no_certified_suite_disables_tests_using_ignore() {
    for suite in suites() {
        assert!(!suite.source.contains("#[ignore]"), "{}", suite.id);
        assert!(!suite.source.contains("#[should_panic]"), "{}", suite.id);
    }
}

#[test]
fn historical_tests_all_reference_frozen_r02_without_export() {
    for suite in suites() {
        assert!(
            suite
                .source
                .contains("#[path = \"../src/resource_task_r02.rs\"]"),
            "{}",
            suite.id
        );
    }
}

#[test]
fn independent_oracle_lineage_is_present_r06_through_r09() {
    for suite in suites().iter().skip(3) {
        assert!(suite.source.contains("struct Oracle"), "{}", suite.id);
        assert!(suite.source.contains("impl Oracle"), "{}", suite.id);
    }
}

#[test]
fn independent_oracles_do_not_use_observed_sut_outcomes_as_predictions() {
    for suite in suites().iter().skip(3) {
        for forbidden in [
            "self.model.phase(",
            "self.model.report(",
            "self.model.outcome(",
        ] {
            assert!(
                !suite.source.contains(forbidden),
                "{} uses {}",
                suite.id,
                forbidden
            );
        }
    }
}

#[test]
fn no_historical_suite_introduces_os_threads_or_unsafe_blocks() {
    for suite in suites() {
        for forbidden in ["std::thread::spawn", "tokio::spawn", "unsafe {"] {
            assert!(
                !suite.source.contains(forbidden),
                "{} uses {}",
                suite.id,
                forbidden
            );
        }
    }
}

#[test]
fn certified_library_and_manifest_do_not_export_reference_models() {
    let lib = include_str!("../src/lib.rs");
    let cargo = include_str!("../Cargo.toml");
    for name in [
        "resource_task_r02",
        "resource_task_r03",
        "resource_task_r04",
        "resource_task_r05",
        "resource_task_r06",
        "resource_task_r07",
        "resource_task_r08",
        "resource_task_r09",
        "resource_task_r10",
    ] {
        assert!(!lib.contains(name), "lib.rs exports {}", name);
        assert!(!cargo.contains(name), "Cargo.toml binds {}", name);
    }
}

#[test]
fn finite_profile_arithmetic_has_explicit_finite_bounds() {
    assert_eq!(5_u64.pow(5), 3_125);
    assert_eq!(6_u64.pow(5), 7_776);
    assert_eq!(7_u64.pow(5), 16_807);
    assert_eq!(factorial(6) / 2_u64.pow(3), 90);
    assert_eq!(factorial(8) / 2_u64.pow(4), 2_520);
}

#[test]
fn generated_corpus_arithmetic_is_consistent_across_versions() {
    let profiles: [(usize, usize, usize); 5] = [
        (16, 96, 1_536),
        (32, 96, 3_072),
        (32, 128, 4_096),
        (32, 160, 5_120),
        (40, 160, 6_400),
    ];
    for (seeds, steps, attempts) in profiles {
        assert_eq!(seeds * steps, attempts);
    }
}

#[test]
fn canonical_material_fingerprint_is_reproducible_in_process() {
    let first = material();
    let second = material();
    assert_eq!(first, second);
    assert_eq!(digest(&first), digest(&second));
    println!("R10_WITNESS version=1 kind=historic_matrix_check suites=7 witnesses=117 fingerprint={:016x} outcome=PASS", digest(&first));
}

#[test]
fn synthetic_witness_swap_is_detected_by_bijection_check() {
    let suite = &suites()[6];
    let tests = parsed(suite.source);
    let mut rows: Vec<_> = parse_matrix(suite.matrix)
        .iter()
        .map(|r| r.witness.to_owned())
        .collect();
    rows.swap(0, 1);
    assert_ne!(tests, rows);
    // This is a synthetic harness mutation, not evidence of an R0.9 defect.
}

#[test]
fn frozen_reference_empty_scope_closes_without_side_effects() {
    let (mut model, _permit) = Model::bootstrap(
        0x5231_0001,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        3,
    );
    let root = model.root();
    let result = model.apply(Command::Close { scope: root });
    assert!(
        matches!(result, Ok(Receipt::Closed(ref report)) if report.outcome == ScopeOutcome::Succeeded)
    );
    assert_eq!(model.event_count(), 1);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn frozen_reference_budget_rejection_has_no_mutation() {
    let (mut model, _permit) = Model::bootstrap(
        0x5231_0002,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        3,
    );
    let before = model.clone();
    let root = model.root();
    assert_eq!(
        model.apply(Command::DeclareTask { scope: root }),
        Err(ModelError::Exhausted)
    );
    assert_eq!(model, before);
    assert_eq!(model.event_count(), 0);
}

#[test]
fn post_closure_rejection_preserves_frozen_model_and_replay() {
    let (mut model, _permit) = Model::bootstrap(
        0x5231_0003,
        ScopeBudget {
            tasks: 1,
            resources: 0,
            children: 0,
        },
        4,
    );
    let root = model.root();
    assert!(matches!(
        model.apply(Command::Close { scope: root }),
        Ok(Receipt::Closed(_))
    ));
    let before = model.clone();
    assert_eq!(
        model.apply(Command::DeclareTask { scope: root }),
        Err(ModelError::ClosedScope)
    );
    assert_eq!(model, before);
    assert_eq!(model.replay_checked(), Ok(()));
}
