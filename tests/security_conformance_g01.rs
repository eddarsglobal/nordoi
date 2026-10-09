use nordoi_kernel::{constitutional_conformance_report_g01, G01_DELIVERY_BASIS_POINTS};

#[test]
fn conformance_report_grants_no_authority() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert!(report.render_text().contains("authority=NONE"));
    assert!(report.render_json().contains("\"authority\":\"none\""));
}

#[test]
fn planning_estimate_is_never_labeled_certification() {
    let report = constitutional_conformance_report_g01().unwrap();
    let text = report.render_text();
    assert!(text.contains("delivery-estimate-kind=PLANNING_NOT_CERTIFICATION"));
    assert_eq!(report.delivery_basis_points(), G01_DELIVERY_BASIS_POINTS);
}

#[test]
fn output_contains_no_host_path_or_timestamp() {
    let report = constitutional_conformance_report_g01().unwrap();
    let text = report.render_text();
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("/tmp/"));
    assert!(!text.contains("2026-"));
}

#[test]
fn future_native_gate_rejects_feature_parity_only_in_governance_data() {
    let report = constitutional_conformance_report_g01().unwrap();
    let rule = report
        .future_native_rules()
        .iter()
        .find(|row| row.rule == "NO_FEATURE_PARITY_ONLY")
        .unwrap();
    assert!(rule.description.contains("legacy feature parity"));
}

#[test]
fn future_native_gate_requires_simplicity_gain() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert!(report
        .future_native_rules()
        .iter()
        .any(|row| row.rule == "SIMPLICITY_GAIN"));
}

#[test]
fn future_native_gate_requires_security_or_provability_gain() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert!(report
        .future_native_rules()
        .iter()
        .any(|row| row.rule == "SECURITY_OR_PROVABILITY_GAIN"));
}

#[test]
fn future_native_gate_requires_universal_architecture() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert!(report
        .future_native_rules()
        .iter()
        .any(|row| row.rule == "UNIVERSAL_ARCHITECTURE"));
}

#[test]
fn conformance_source_has_no_runtime_fs_network_process_or_env_imports() {
    let source = include_str!("../src/conformance_g01.rs");
    for forbidden in ["std::fs", "std::net", "std::process", "std::env"] {
        assert!(!source.contains(forbidden), "unexpected import {forbidden}");
    }
}
