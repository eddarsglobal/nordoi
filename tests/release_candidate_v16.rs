use nordoi_kernel::{
    compile_project_v14, distribution_plan_v16, parse_project_manifest_v14,
    verify_release_candidate_v16, ReleaseCandidateError, SourceId, SourceText,
    V16_PROVENANCE_SCHEMA, V16_RELEASE_SCHEMA,
};

fn manifest(name: &str, version: &str) -> nordoi_kernel::V14ProjectManifest {
    parse_project_manifest_v14(&format!(
        "[project]\nname = \"{name}\"\nversion = \"{version}\"\nentry = \"app.main\"\nsource-root = \"src\"\n"
    ))
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

#[test]
fn exact_lock_and_package_produce_release_candidate() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let report =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    assert_eq!(report.project(), "demo");
    assert_eq!(report.version(), "0.1.0");
    assert_eq!(report.module_count(), 2);
    assert_eq!(report.import_count(), 1);
    assert_eq!(report.nair_format_minor(), 12);
    assert_eq!(report.package_sha256_hex().len(), 64);
    assert!(report.render_json().contains(V16_RELEASE_SCHEMA));
}

#[test]
fn lock_drift_is_rejected_before_release() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let error =
        verify_release_candidate_v16(&build, "stale lock\n", build.package_bytes()).unwrap_err();
    assert_eq!(error, ReleaseCandidateError::LockMismatch);
}

#[test]
fn corrupted_package_is_rejected_as_invalid() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let mut corrupt = build.package_bytes().to_vec();
    corrupt[0] = b'X';
    let error = verify_release_candidate_v16(&build, build.lock_text(), &corrupt).unwrap_err();
    assert!(matches!(
        error,
        ReleaseCandidateError::PackageInvalid { .. }
    ));
}

#[test]
fn different_valid_package_is_rejected_as_mismatch() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let other = compile_project_v14(&manifest("demo", "0.1.0"), &sources(101)).unwrap();
    let error =
        verify_release_candidate_v16(&build, build.lock_text(), other.package_bytes()).unwrap_err();
    assert_eq!(error, ReleaseCandidateError::PackageMismatch);
}

#[test]
fn provenance_is_canonical_platform_neutral_and_timeless() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let report =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let provenance = report.provenance_text();
    assert!(provenance.contains(V16_PROVENANCE_SCHEMA));
    assert!(provenance.contains("platform-neutral = true"));
    assert!(provenance.contains("dependency-network = \"NONE\""));
    assert!(provenance.contains("runtime-fs = \"NONE\""));
    assert!(provenance.contains("authority = \"NONE\""));
    assert!(!provenance.contains("timestamp"));
    assert!(!provenance.contains("/tmp/"));
}

#[test]
fn equal_builds_have_equal_release_hashes_and_provenance() {
    let a_sources = sources(100);
    let b_sources = vec![a_sources[1].clone(), a_sources[0].clone()];
    let manifest = manifest("demo", "0.1.0");
    let a = compile_project_v14(&manifest, &a_sources).unwrap();
    let b = compile_project_v14(&manifest, &b_sources).unwrap();
    let ar = verify_release_candidate_v16(&a, a.lock_text(), a.package_bytes()).unwrap();
    let br = verify_release_candidate_v16(&b, b.lock_text(), b.package_bytes()).unwrap();
    assert_eq!(ar.package_sha256(), br.package_sha256());
    assert_eq!(ar.provenance_sha256(), br.provenance_sha256());
    assert_eq!(ar.provenance_text(), br.provenance_text());
}

#[test]
fn distribution_plan_contains_exact_package_checksum_and_provenance() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let report =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&report, build.package_bytes()).unwrap();
    assert_eq!(plan.package_file_name(), "demo-0.1.0.npkg");
    assert_eq!(plan.checksum_file_name(), "demo-0.1.0.npkg.sha256");
    assert_eq!(plan.provenance_file_name(), "demo-0.1.0.provenance");
    assert_eq!(plan.package_bytes(), build.package_bytes());
    assert_eq!(plan.provenance_text(), report.provenance_text());
    assert_eq!(
        plan.checksum_text(),
        format!("{}  demo-0.1.0.npkg\n", report.package_sha256_hex())
    );
}

#[test]
fn distribution_plan_rejects_package_substitution() {
    let build = compile_project_v14(&manifest("demo", "0.1.0"), &sources(100)).unwrap();
    let other = compile_project_v14(&manifest("demo", "0.1.0"), &sources(101)).unwrap();
    let report =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let error = distribution_plan_v16(&report, other.package_bytes()).unwrap_err();
    assert_eq!(error, ReleaseCandidateError::PackageMismatch);
}
