//! NORDOI R0.11 — TEST-ONLY production-boundary readiness advisory.
//! No production authority, native scheduler, compiler/NAIR change or real approval.

#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Command, Model, Receipt, ScopeBudget, ScopeOutcome};
use std::collections::BTreeSet;

const BASELINE_SHA: &str = "16de3cf71b4be39118eb2bb4b963c64ceeb5a65a";
const GATE_COUNT: usize = 12;
const AREAS: [&str; GATE_COUNT] = [
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
const REGISTRY: &str = include_str!("../governance/r11_native_readiness_gates_v1.tsv");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Gate<'a> {
    id: &'a str,
    area: &'a str,
    status: &'a str,
    evidence: &'a str,
    criterion: &'a str,
    limitation: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Advisory {
    Invalid,
    Blocked,
    ReviewEligible,
}

fn parse_registry(src: &str) -> Result<Vec<Gate<'_>>, &'static str> {
    let mut lines = src.lines();
    if lines.next() != Some("id\tarea\tstatus\tevidence_scope\tacceptance_criterion\tlimitation") {
        return Err("unknown readiness registry schema");
    }
    let mut result = Vec::new();
    for line in lines {
        if line.is_empty() {
            return Err("empty registry row");
        }
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() != 6 {
            return Err("incorrect field count");
        }
        if !matches!(parts[2], "UNMET" | "VERIFIED") {
            return Err("unknown status");
        }
        if !matches!(parts[3], "RESEARCH_ONLY" | "NATIVE_VERIFIED") {
            return Err("unknown evidence scope");
        }
        result.push(Gate {
            id: parts[0],
            area: parts[1],
            status: parts[2],
            evidence: parts[3],
            criterion: parts[4],
            limitation: parts[5],
        });
    }
    Ok(result)
}

fn registry() -> Vec<Gate<'static>> {
    parse_registry(REGISTRY).expect("canonical test fixture must parse")
}

fn assess(gates: &[Gate<'_>], mock_approval: bool, mock_green_ci: bool) -> Advisory {
    if gates.len() != GATE_COUNT {
        return Advisory::Invalid;
    }
    let mut seen = BTreeSet::new();
    for (index, gate) in gates.iter().enumerate() {
        let expected_id = format!("R11-G{:02}", index + 1);
        if gate.id != expected_id.as_str()
            || !seen.insert(gate.id)
            || gate.area != AREAS[index]
            || gate.criterion.trim().len() < 30
            || gate.limitation.trim().len() < 30
            || !matches!(gate.status, "UNMET" | "VERIFIED")
            || !matches!(gate.evidence, "RESEARCH_ONLY" | "NATIVE_VERIFIED")
        {
            return Advisory::Invalid;
        }
    }
    if gates
        .iter()
        .any(|gate| gate.status != "VERIFIED" || gate.evidence != "NATIVE_VERIFIED")
    {
        return Advisory::Blocked;
    }
    if !mock_approval || !mock_green_ci {
        return Advisory::Blocked;
    }
    // This test-only value NEVER means a real authenticated or authorized release.
    Advisory::ReviewEligible
}

fn synthetic_verified() -> Vec<Gate<'static>> {
    let mut gates = registry();
    for gate in &mut gates {
        gate.status = "VERIFIED";
        gate.evidence = "NATIVE_VERIFIED";
    }
    gates
}

#[test]
fn certified_r010_baseline_identity_is_explicit() {
    assert_eq!(BASELINE_SHA, "16de3cf71b4be39118eb2bb4b963c64ceeb5a65a");
}

#[test]
fn native_readiness_registry_has_twelve_gates() {
    assert_eq!(registry().len(), GATE_COUNT);
}

#[test]
fn native_gate_identifiers_are_unique_and_ordered() {
    let gates = registry();
    let mut seen = BTreeSet::new();
    for (i, gate) in gates.iter().enumerate() {
        assert_eq!(gate.id, format!("R11-G{:02}", i + 1));
        assert!(seen.insert(gate.id));
        assert_eq!(gate.area, AREAS[i]);
    }
    assert_eq!(seen.len(), GATE_COUNT);
}

#[test]
fn every_gate_contains_acceptance_and_limitation() {
    for gate in registry() {
        assert!(gate.area.len() >= 5);
        assert!(gate.criterion.len() >= 30);
        assert!(gate.limitation.len() >= 30);
    }
}

#[test]
fn all_canonical_gates_are_explicitly_unmet() {
    assert!(registry().iter().all(|gate| gate.status == "UNMET"));
}

#[test]
fn research_witnesses_are_not_native_evidence() {
    assert!(registry()
        .iter()
        .all(|gate| gate.evidence == "RESEARCH_ONLY"));
}

#[test]
fn native_advisory_denies_by_default() {
    assert_eq!(assess(&registry(), false, false), Advisory::Blocked);
}

#[test]
fn governance_approval_alone_cannot_bypass_gates() {
    assert_eq!(assess(&registry(), true, false), Advisory::Blocked);
}

#[test]
fn green_ci_alone_cannot_bypass_gates() {
    assert_eq!(assess(&registry(), false, true), Advisory::Blocked);
    assert_eq!(assess(&registry(), true, true), Advisory::Blocked);
}

#[test]
fn missing_gate_must_fail_closed() {
    let mut gates = registry();
    gates.pop();
    assert_eq!(assess(&gates, true, true), Advisory::Invalid);
}

#[test]
fn duplicate_gate_must_fail_closed() {
    let mut gates = registry();
    gates[1] = gates[0].clone();
    assert_eq!(assess(&gates, true, true), Advisory::Invalid);
    let mut swapped_area = synthetic_verified();
    swapped_area[0].area = "SIMPLIFIED";
    assert_eq!(assess(&swapped_area, true, true), Advisory::Invalid);
}

#[test]
fn unrecognized_status_cannot_be_interpreted_as_verified() {
    let corrupted = REGISTRY.replacen("\tUNMET\t", "\tFORGED\t", 1);
    assert!(parse_registry(&corrupted).is_err());
    let mut gates = registry();
    gates[0].status = "FORGED";
    assert_eq!(assess(&gates, true, true), Advisory::Invalid);
}

#[test]
fn blank_acceptance_criterion_cannot_be_verified() {
    let mut gates = synthetic_verified();
    gates[0].criterion = "";
    assert_eq!(assess(&gates, true, true), Advisory::Invalid);
}

#[test]
fn evidence_scope_missing_prevents_review() {
    let mut gates = synthetic_verified();
    gates[0].evidence = "RESEARCH_ONLY";
    assert_eq!(assess(&gates, true, true), Advisory::Blocked);
}

#[test]
fn synthetic_completion_without_approval_stays_blocked() {
    let gates = synthetic_verified();
    assert_eq!(assess(&gates, false, true), Advisory::Blocked);
    assert_eq!(assess(&gates, true, false), Advisory::Blocked);
}

#[test]
fn synthetic_completion_with_approval_stays_review_only() {
    let gates = synthetic_verified();
    assert_eq!(assess(&gates, true, true), Advisory::ReviewEligible);
    // ReviewEligible exists only in an unauthenticated test fixture.
    // There is deliberately no state allowing production deployment.
    println!(
        "R11_WITNESS version=1 kind=synthetic_advisory gates=12 status=REVIEW_ONLY outcome=PASS"
    );
}

#[test]
fn one_unmet_gate_blocks_synthetic_complete_candidate() {
    let mut gates = synthetic_verified();
    gates[11].status = "UNMET";
    assert_eq!(assess(&gates, true, true), Advisory::Blocked);
}

#[test]
fn permuted_gate_order_fails_closed() {
    let mut gates = synthetic_verified();
    gates.swap(0, 1);
    assert_eq!(assess(&gates, true, true), Advisory::Invalid);
}

#[test]
fn frozen_reference_is_still_pure_and_replayable() {
    let (mut model, _host) = Model::bootstrap(
        0x5231_0011,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        2,
    );
    let root = model.root();
    let receipt = model.apply(Command::Close { scope: root });
    assert!(
        matches!(receipt, Ok(Receipt::Closed(ref report)) if report.outcome == ScopeOutcome::Succeeded)
    );
    assert_eq!(model.event_count(), 1);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn no_native_authority_is_exported_by_r011() {
    let lib = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");
    for forbidden in ["resource_task_r11", "r11_native_readiness", "r11_readiness"] {
        assert!(!lib.contains(forbidden));
        assert!(!manifest.contains(forbidden));
    }
    assert!(REGISTRY.contains("R11-G12\tINDEPENDENT_REVIEW\tUNMET"));
}
