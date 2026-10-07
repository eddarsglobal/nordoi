use nordoi_kernel::{
    compile_project_v14, inspect_project_package_v14, parse_project_manifest_v14, SourceId,
    SourceText,
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

fn basic_sources() -> Vec<SourceText> {
    vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(
            2,
            "main.noi",
            "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);",
        ),
    ]
}

#[test]
fn simple_project_builds_existing_nair_012_package() {
    let build = compile_project_v14(&manifest(), &basic_sources()).unwrap();
    assert_eq!(build.module_plan().module_count(), 2);
    assert_eq!(build.module_plan().import_count(), 1);
    assert_eq!(build.lowering().nair_format_minor(), 12);
    assert!(build.lock_text().contains("nair = \"0.12\""));
    assert!(!build.package_bytes().is_empty());
}

#[test]
fn static_project_preserves_base_nair_06() {
    let sources = vec![
        src(
            1,
            "math.noi",
            "module app.math; fn add_100(x) returns x + 100;",
        ),
        src(
            2,
            "main.noi",
            "module app.main; import app.math; entry main returns math.add_100(41);",
        ),
    ];
    let build = compile_project_v14(&manifest(), &sources).unwrap();
    assert_eq!(build.lowering().nair_format_minor(), 6);
}

#[test]
fn transitive_project_preserves_existing_nair_013() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "mid.noi", "module app.mid; import app.math; fn plus(x) returns math.add_100(x);"),
        src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.plus(key_code);"),
    ];
    let build = compile_project_v14(&manifest(), &sources).unwrap();
    assert_eq!(build.lowering().nair_format_minor(), 13);
}

#[test]
fn structured_control_project_preserves_existing_nair_014() {
    let sources = vec![
        src(1, "rules.noi", "module app.rules; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.bias(key_code);"),
    ];
    let build = compile_project_v14(&manifest(), &sources).unwrap();
    assert_eq!(build.lowering().nair_format_minor(), 14);
}

#[test]
fn source_order_does_not_change_package_or_lock() {
    let sources = basic_sources();
    let reverse = vec![sources[1].clone(), sources[0].clone()];
    let a = compile_project_v14(&manifest(), &sources).unwrap();
    let b = compile_project_v14(&manifest(), &reverse).unwrap();
    assert_eq!(a.package_bytes(), b.package_bytes());
    assert_eq!(a.lock_text(), b.lock_text());
    assert_eq!(
        a.canonical_v14_build_witness_bytes(),
        b.canonical_v14_build_witness_bytes()
    );
}

#[test]
fn manifest_metadata_changes_package_identity() {
    let a = compile_project_v14(&manifest(), &basic_sources()).unwrap();
    let other = parse_project_manifest_v14(
        "[project]\nname = \"demo2\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap();
    let b = compile_project_v14(&other, &basic_sources()).unwrap();
    assert_ne!(a.package_bytes(), b.package_bytes());
    assert_ne!(
        a.canonical_v14_build_witness_bytes(),
        b.canonical_v14_build_witness_bytes()
    );
}

#[test]
fn package_can_be_inspected_without_sources() {
    let build = compile_project_v14(&manifest(), &basic_sources()).unwrap();
    let info = inspect_project_package_v14(build.package_bytes()).unwrap();
    assert_eq!(info.name(), "demo");
    assert_eq!(info.version(), "0.1.0");
    assert_eq!(info.entry_module(), "app.main");
    assert_eq!(info.source_root(), "src");
    assert_eq!(
        info.module_order(),
        &["app.main".to_owned(), "app.math".to_owned()]
    );
    assert_eq!(info.import_edges().len(), 1);
    assert_eq!(info.nair_format_minor(), 12);
    assert_eq!(info.nair_instruction_count(), 3);
    assert_eq!(
        info.build_witness(),
        build.canonical_v14_build_witness_bytes()
    );
}

#[test]
fn corrupted_or_noncanonical_package_is_rejected() {
    let build = compile_project_v14(&manifest(), &basic_sources()).unwrap();
    let mut corrupt = build.package_bytes().to_vec();
    corrupt[0] = b'X';
    assert!(inspect_project_package_v14(&corrupt).is_err());

    let mut trailing = build.package_bytes().to_vec();
    trailing.push(0);
    let error = inspect_project_package_v14(&trailing).unwrap_err();
    assert!(format!("{error}").contains("trailing bytes"));
}
