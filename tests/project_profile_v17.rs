use nordoi_kernel::{
    certify_production_profile1_v17, compile_project_v14, distribution_plan_v16,
    parse_project_manifest_v14, verify_release_candidate_v16, ProductionProfileCertificationError,
    ReleaseCandidateError, SourceId, SourceText,
};

fn manifest(version: &str) -> nordoi_kernel::V14ProjectManifest {
    parse_project_manifest_v14(&format!(
        "[project]\nname = \"demo\"\nversion = \"{version}\"\nentry = \"app.main\"\nsource-root = \"src\"\n"
    ))
    .unwrap()
}

fn src(id: u32, name: &str, text: impl AsRef<str>) -> SourceText {
    SourceText::new(SourceId::new(id), name, text.as_ref()).unwrap()
}

fn certify(
    manifest: &nordoi_kernel::V14ProjectManifest,
    sources: &[SourceText],
) -> nordoi_kernel::V17ProductionProfileCertificate {
    let build = compile_project_v14(manifest, sources).unwrap();
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
fn static_project_final_certification_preserves_nair_06() {
    let certificate = certify(
        &manifest("0.1.0"),
        &[
            src(
                1,
                "math.noi",
                "module app.math; fn plus(x) returns x + 100;",
            ),
            src(
                2,
                "main.noi",
                "module app.main; import app.math; entry main returns math.plus(41);",
            ),
        ],
    );
    assert_eq!(certificate.nair_format_minor(), 6);
}

#[test]
fn direct_dynamic_project_final_certification_preserves_nair_012() {
    let certificate = certify(
        &manifest("0.1.0"),
        &[
            src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
            src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
        ],
    );
    assert_eq!(certificate.nair_format_minor(), 12);
}

#[test]
fn transitive_project_final_certification_preserves_nair_013() {
    let certificate = certify(
        &manifest("0.1.0"),
        &[
            src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
            src(2, "mid.noi", "module app.mid; import app.math; fn mid(x) returns math.plus(x);"),
            src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.mid(key_code);"),
        ],
    );
    assert_eq!(certificate.nair_format_minor(), 13);
}

#[test]
fn structured_project_final_certification_preserves_nair_014() {
    let certificate = certify(
        &manifest("0.1.0"),
        &[
            src(1, "rules.noi", "module app.rules; fn classify(x) returns if x > 40 { x + 100 } else { x + 200 };"),
            src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.classify(key_code);"),
        ],
    );
    assert_eq!(certificate.nair_format_minor(), 14);
}

#[test]
fn source_order_does_not_change_final_certification_identity() {
    let a = src(
        1,
        "rules.noi",
        "module app.rules; fn classify(x) returns if x > 40 { x + 100 } else { x + 200 };",
    );
    let b = src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.classify(key_code);");
    let first = certify(&manifest("0.1.0"), &[a.clone(), b.clone()]);
    let second = certify(&manifest("0.1.0"), &[b, a]);
    assert_eq!(first.certification_sha256(), second.certification_sha256());
}

#[test]
fn project_version_changes_final_certification_identity() {
    let sources = [
        src(1, "rules.noi", "module app.rules; fn classify(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.classify(key_code);"),
    ];
    let first = certify(&manifest("0.1.0"), &sources);
    let second = certify(&manifest("0.2.0"), &sources);
    assert_ne!(first.certification_sha256(), second.certification_sha256());
}

#[test]
fn profile1_reference_application_shape_certifies_at_nair_014() {
    let reference_manifest = parse_project_manifest_v14(
        "[project]\nname = \"profile1-reference\"\nversion = \"1.0.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap();
    let certificate = certify(
        &reference_manifest,
        &[
            src(1, "rules.noi", "module app.rules; fn classify(x) returns if x > 40 { x + 100 } else { x + 200 };"),
            src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.classify(key_code);"),
        ],
    );
    assert_eq!(certificate.project(), "profile1-reference");
    assert_eq!(certificate.nair_format_minor(), 14);
}

#[test]
fn corrupted_build_package_fails_before_distribution_exactness_is_considered() {
    let sources = [
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ];
    let build = compile_project_v14(&manifest("0.1.0"), &sources).unwrap();
    let release =
        verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap();
    let plan = distribution_plan_v16(&release, build.package_bytes()).unwrap();
    let mut corrupted = build.package_bytes().to_vec();
    corrupted[0] ^= 1;
    let error = certify_production_profile1_v17(
        &build,
        build.lock_text(),
        &corrupted,
        plan.package_bytes(),
        plan.checksum_text(),
        plan.provenance_text(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProductionProfileCertificationError::ReleaseCandidate(
            ReleaseCandidateError::PackageInvalid { .. }
        ) | ProductionProfileCertificationError::ReleaseCandidate(
            ReleaseCandidateError::PackageMismatch
        )
    ));
}
