use nordoi_kernel::{
    constitutional_conformance_report_g01, EvidenceStatus, G01_BASELINE_CI, G01_BASELINE_COMMIT,
    G01_BASELINE_TAG, G01_DELIVERY_BASIS_POINTS, G01_FUTURE_NATIVE_RULE_COUNT, G01_PRINCIPLE_COUNT,
    G01_SCHEMA,
};

#[test]
fn matrix_contains_exact_c1_through_c322_sequence() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert_eq!(report.principles().len(), G01_PRINCIPLE_COUNT);
    for (index, principle) in report.principles().iter().enumerate() {
        assert_eq!(principle.id, format!("C{}", index + 1));
    }
}

#[test]
fn evidence_counts_are_explicit_and_not_project_completion() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert_eq!(report.certified_count(), 310);
    assert_eq!(report.partial_count(), 12);
    assert_eq!(report.evidence_basis_points(), 9_627);
    assert_eq!(report.delivery_basis_points(), G01_DELIVERY_BASIS_POINTS);
    assert_ne!(
        report.evidence_basis_points(),
        report.delivery_basis_points()
    );
}

#[test]
fn global_future_principles_remain_partial_where_delivery_is_incomplete() {
    let report = constitutional_conformance_report_g01().unwrap();
    for id in ["C1", "C2", "C3", "C8", "C9", "C15", "C17", "C25"] {
        let row = report.principles().iter().find(|row| row.id == id).unwrap();
        assert_eq!(row.evidence_status, EvidenceStatus::Partial, "{id}");
    }
}

#[test]
fn certified_kernel_principles_have_traceable_evidence() {
    let report = constitutional_conformance_report_g01().unwrap();
    for row in report
        .principles()
        .iter()
        .filter(|row| row.evidence_status == EvidenceStatus::Certified)
    {
        assert!(!row.evidence_ref.is_empty());
        assert!(!row.next_action.is_empty());
    }
}

#[test]
fn delivery_model_is_weighted_to_exactly_one_hundred_percent() {
    let report = constitutional_conformance_report_g01().unwrap();
    let weight: u32 = report
        .delivery_domains()
        .iter()
        .map(|row| row.weight_percent)
        .sum();
    assert_eq!(weight, 100);
    assert_eq!(report.delivery_basis_points(), 5_835);
}

#[test]
fn future_native_gate_has_six_nonempty_rules() {
    let report = constitutional_conformance_report_g01().unwrap();
    assert_eq!(
        report.future_native_rules().len(),
        G01_FUTURE_NATIVE_RULE_COUNT
    );
    for row in report.future_native_rules() {
        assert!(!row.id.is_empty());
        assert!(!row.rule.is_empty());
        assert!(!row.description.is_empty());
    }
}

#[test]
fn baseline_identity_is_exact_p27_certification() {
    assert_eq!(G01_BASELINE_TAG, "p2.7");
    assert_eq!(
        G01_BASELINE_COMMIT,
        "0a634d6cab52074e18b42d1ffea00ce02cdba359"
    );
    assert_eq!(G01_BASELINE_CI, 37_956_122_196);
}

#[test]
fn report_rendering_is_deterministic_and_schema_versioned() {
    let first = constitutional_conformance_report_g01().unwrap();
    let second = constitutional_conformance_report_g01().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.render_text(), second.render_text());
    assert_eq!(first.render_json(), second.render_json());
    assert!(first.render_json().contains(G01_SCHEMA));
}
