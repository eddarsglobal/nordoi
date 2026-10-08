use nordoi_kernel::{
    certify_production_profile1_v17, compile_project_v14, distribution_plan_v16,
    parse_project_manifest_v14, verify_release_candidate_v16, ProductionProfileCertificationError,
    ReleaseCandidateError, SourceId, SourceText, V17_PROFILE1_SCHEMA,
};

fn manifest() -> nordoi_kernel::V14ProjectManifest {
    parse_project_manifest_v14(
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap()
}

fn src(id: u32, name: &str, text: impl AsRef<str>) -> SourceText {
    SourceText::new(SourceId::new(id), name, text.as_ref()).unwrap()
}

fn sources(addend: i64) -> Vec<SourceText> {
    vec![
        src(
            1,
            "math.noi",
            format!("module app.math; fn add_100(x) returns x + {addend};"),
        ),
        src(
            2,
            "main.noi",
            "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);",
        ),
    ]
}

fn exact_certificate() -> nordoi_kernel::V17ProductionProfileCertificate {
    let build = compile_project_v14(&manifest(), &sources(100)).unwrap();
    let release =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&release, build.package_bytes()).unwrap();
    certify_production_profile1_v17(
        &build,
        build.lock_text(),
        build.package_bytes(),
        plan.package_bytes(),
        plan.checksum_text(),
        plan.provenance_text(),
    )
    .unwrap()
}

#[test]
fn exact_complete_distribution_certifies_profile1() {
    let certificate = exact_certificate();
    assert_eq!(certificate.project(), "demo");
    assert_eq!(certificate.version(), "0.1.0");
    assert_eq!(certificate.nair_format_minor(), 12);
    assert_eq!(certificate.module_count(), 2);
    assert_eq!(certificate.import_count(), 1);
}

#[test]
fn substituted_distribution_package_fails_closed() {
    let build = compile_project_v14(&manifest(), &sources(100)).unwrap();
    let release =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&release, build.package_bytes()).unwrap();
    let mut dist = plan.package_bytes().to_vec();
    dist[0] ^= 0x01;
    let error = certify_production_profile1_v17(
        &build,
        build.lock_text(),
        build.package_bytes(),
        &dist,
        plan.checksum_text(),
        plan.provenance_text(),
    )
    .unwrap_err();
    assert_eq!(
        error,
        ProductionProfileCertificationError::DistributionPackageMismatch
    );
}

#[test]
fn checksum_drift_fails_closed() {
    let build = compile_project_v14(&manifest(), &sources(100)).unwrap();
    let release =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&release, build.package_bytes()).unwrap();
    let error = certify_production_profile1_v17(
        &build,
        build.lock_text(),
        build.package_bytes(),
        plan.package_bytes(),
        "00  demo-0.1.0.npkg\n",
        plan.provenance_text(),
    )
    .unwrap_err();
    assert_eq!(error, ProductionProfileCertificationError::ChecksumMismatch);
}

#[test]
fn provenance_drift_fails_closed() {
    let build = compile_project_v14(&manifest(), &sources(100)).unwrap();
    let release =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&release, build.package_bytes()).unwrap();
    let mut provenance = plan.provenance_text().to_owned();
    provenance.push_str("host = \"forbidden\"\n");
    let error = certify_production_profile1_v17(
        &build,
        build.lock_text(),
        build.package_bytes(),
        plan.package_bytes(),
        plan.checksum_text(),
        &provenance,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ProductionProfileCertificationError::ProvenanceMismatch
    );
}

#[test]
fn source_lock_drift_is_rejected_before_distribution_certification() {
    let original = compile_project_v14(&manifest(), &sources(100)).unwrap();
    let current = compile_project_v14(&manifest(), &sources(101)).unwrap();
    let release =
        verify_release_candidate_v16(&original, original.lock_text(), original.package_bytes())
            .unwrap();
    let plan = distribution_plan_v16(&release, original.package_bytes()).unwrap();
    let error = certify_production_profile1_v17(
        &current,
        original.lock_text(),
        original.package_bytes(),
        plan.package_bytes(),
        plan.checksum_text(),
        plan.provenance_text(),
    )
    .unwrap_err();
    assert_eq!(
        error,
        ProductionProfileCertificationError::ReleaseCandidate(ReleaseCandidateError::LockMismatch)
    );
}

#[test]
fn equal_builds_have_equal_final_certification_identity() {
    let a = exact_certificate();
    let b = exact_certificate();
    assert_eq!(a.certification_sha256(), b.certification_sha256());
    assert_eq!(a.certificate_text(), b.certificate_text());
}

#[test]
fn final_certificate_is_timeless_hostless_and_authority_free() {
    let certificate = exact_certificate();
    let text = certificate.certificate_text();
    assert!(text.contains("dependency-network = \"NONE\""));
    assert!(text.contains("runtime-fs = \"NONE\""));
    assert!(text.contains("authority = \"NONE\""));
    assert!(!text.contains("timestamp"));
    assert!(!text.contains("hostname"));
    assert!(!text.contains("/Users/"));
    assert!(!text.contains("C:\\"));
}

#[test]
fn renderers_are_schema_versioned_and_commit_all_exactness_proofs() {
    let certificate = exact_certificate();
    let text = certificate.render_text();
    assert!(text.contains("status=CERTIFIED"));
    assert!(text.contains("source-lock=EXACT"));
    assert!(text.contains("dist-package=EXACT"));
    assert!(text.contains("checksum=EXACT"));
    assert!(text.contains("provenance=EXACT"));

    let json = certificate.render_json();
    assert!(json.contains(&format!("\"schema\":\"{V17_PROFILE1_SCHEMA}\"")));
    assert!(json.contains("\"status\":\"certified\""));
    assert!(json.contains("\"distPackage\":\"exact\""));
    assert!(json.contains("\"authority\":\"NONE\""));
}
