use nordoi_kernel::{
    compile_project_v14, parse_project_manifest_v14, verify_release_candidate_v16,
    ReleaseCandidateError, SourceId, SourceText,
};

fn manifest() -> nordoi_kernel::V14ProjectManifest {
    parse_project_manifest_v14(
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap()
}

fn src(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn verify(sources: &[SourceText]) -> nordoi_kernel::V16ReleaseCandidateReport {
    let build = compile_project_v14(&manifest(), sources).unwrap();
    verify_release_candidate_v16(&build, build.lock_text(), build.package_bytes()).unwrap()
}

#[test]
fn static_project_release_preserves_nair_06() {
    let report = verify(&[
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
    ]);
    assert_eq!(report.nair_format_minor(), 6);
}

#[test]
fn direct_dynamic_project_release_preserves_nair_012() {
    let report = verify(&[
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ]);
    assert_eq!(report.nair_format_minor(), 12);
}

#[test]
fn transitive_project_release_preserves_nair_013() {
    let report = verify(&[
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "mid.noi", "module app.mid; import app.math; fn mid(x) returns math.plus(x);"),
        src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.mid(key_code);"),
    ]);
    assert_eq!(report.nair_format_minor(), 13);
}

#[test]
fn structured_control_project_release_preserves_nair_014() {
    let report = verify(&[
        src(1, "rules.noi", "module app.rules; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.bias(key_code);"),
    ]);
    assert_eq!(report.nair_format_minor(), 14);
}

#[test]
fn source_order_does_not_change_release_identity() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ];
    let reverse = vec![sources[1].clone(), sources[0].clone()];
    let a = verify(&sources);
    let b = verify(&reverse);
    assert_eq!(a.package_sha256(), b.package_sha256());
    assert_eq!(a.provenance_sha256(), b.provenance_sha256());
}

#[test]
fn source_drift_rejects_old_lock_before_distribution() {
    let original_sources = vec![
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ];
    let drifted_sources = vec![
        src(
            1,
            "math.noi",
            "module app.math; fn plus(x) returns x + 101;",
        ),
        original_sources[1].clone(),
    ];
    let old = compile_project_v14(&manifest(), &original_sources).unwrap();
    let current = compile_project_v14(&manifest(), &drifted_sources).unwrap();
    let error =
        verify_release_candidate_v16(&current, old.lock_text(), old.package_bytes()).unwrap_err();
    assert_eq!(error, ReleaseCandidateError::LockMismatch);
}

#[test]
fn project_metadata_is_committed_into_release_identity() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ];
    let a = compile_project_v14(&manifest(), &sources).unwrap();
    let other_manifest = parse_project_manifest_v14(
        "[project]\nname = \"demo\"\nversion = \"0.2.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap();
    let b = compile_project_v14(&other_manifest, &sources).unwrap();
    let ar = verify_release_candidate_v16(&a, a.lock_text(), a.package_bytes()).unwrap();
    let br = verify_release_candidate_v16(&b, b.lock_text(), b.package_bytes()).unwrap();
    assert_ne!(ar.package_sha256(), br.package_sha256());
    assert_ne!(ar.provenance_sha256(), br.provenance_sha256());
}

#[test]
fn release_provenance_commits_no_host_authority_or_runtime_filesystem() {
    let report = verify(&[
        src(1, "math.noi", "module app.math; fn plus(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.plus(key_code);"),
    ]);
    let text = report.provenance_text();
    assert!(text.contains("dependency-network = \"NONE\""));
    assert!(text.contains("runtime-fs = \"NONE\""));
    assert!(text.contains("authority = \"NONE\""));
    assert!(text.contains("reproducible = true"));
    assert!(text.contains("platform-neutral = true"));
}
