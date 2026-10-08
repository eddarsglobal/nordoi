use nordoi_kernel::{
    check_project_sources_v15, parse_project_manifest_v14, SourceId, SourceText, NDX_IMPORT_CYCLE,
    NDX_MODULE_DECLARATION, NDX_MODULE_GRAPH,
};

fn src(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn manifest() -> nordoi_kernel::V14ProjectManifest {
    parse_project_manifest_v14(
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap()
}

#[test]
fn two_file_project_checks_without_emitting_runtime_authority() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);"),
    ];
    let report = check_project_sources_v15(&manifest(), &sources).unwrap();
    assert_eq!(report.module_count(), 2);
    assert_eq!(report.import_count(), 1);
    assert_eq!(report.nair_format_minor(), 12);
    assert!(report.render_text().contains("authority=NONE"));
}

#[test]
fn transitive_project_preserves_existing_nair_013() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "mid.noi", "module app.mid; import app.math; fn plus(x) returns math.add_100(x);"),
        src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.plus(key_code);"),
    ];
    let report = check_project_sources_v15(&manifest(), &sources).unwrap();
    assert_eq!(report.module_count(), 3);
    assert_eq!(report.nair_format_minor(), 13);
}

#[test]
fn structured_control_project_preserves_existing_nair_014() {
    let sources = vec![
        src(1, "rules.noi", "module app.rules; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.bias(key_code);"),
    ];
    let report = check_project_sources_v15(&manifest(), &sources).unwrap();
    assert_eq!(report.nair_format_minor(), 14);
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
    let report = check_project_sources_v15(&manifest(), &sources).unwrap();
    assert_eq!(report.nair_format_minor(), 6);
}

#[test]
fn import_cycle_has_specific_code_and_closed_trace() {
    let sources = vec![
        src(
            1,
            "a.noi",
            "module app.a; import app.b; fn a(x) returns b.b(x);",
        ),
        src(
            2,
            "b.noi",
            "module app.b; import app.a; fn b(x) returns a.a(x);",
        ),
        src(
            3,
            "main.noi",
            "module app.main; import app.a; input key_code; entry main returns a.a(key_code);",
        ),
    ];
    let diagnostic = check_project_sources_v15(&manifest(), &sources).unwrap_err();
    assert_eq!(diagnostic.code(), NDX_IMPORT_CYCLE);
    assert_eq!(
        diagnostic.import_trace().first(),
        diagnostic.import_trace().last()
    );
}

#[test]
fn malformed_module_declaration_keeps_source_location() {
    let sources = vec![src(
        1,
        "main.noi",
        "module app.main import app.math; input key_code; entry main returns key_code;",
    )];
    let diagnostic = check_project_sources_v15(&manifest(), &sources).unwrap_err();
    assert_eq!(diagnostic.code(), NDX_MODULE_DECLARATION);
    assert!(diagnostic.location().is_some());
}

#[test]
fn unknown_qualified_function_is_semantic_diagnostic() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.missing(key_code);"),
    ];
    let diagnostic = check_project_sources_v15(&manifest(), &sources).unwrap_err();
    assert_eq!(diagnostic.code(), NDX_MODULE_GRAPH);
    assert!(diagnostic.message().contains("does not exist"));
}

#[test]
fn source_order_does_not_change_success_json() {
    let a = src(
        1,
        "math.noi",
        "module app.math; fn add_100(x) returns x + 100;",
    );
    let b = src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);");
    let first = check_project_sources_v15(&manifest(), &[a.clone(), b.clone()]).unwrap();
    let second = check_project_sources_v15(&manifest(), &[b, a]).unwrap();
    assert_eq!(first.render_json(), second.render_json());
}
