use std::collections::BTreeMap;

use nordoi_kernel::{
    detect_import_cycle_v15, diagnostic_from_project_error_v15, import_trace_from_parents_v15,
    parse_project_manifest_v14, ByteOffset, SourceId, SourceText, V15Diagnostic, NDX_IMPORT_CYCLE,
    NDX_MANIFEST, NDX_SOURCE, V15_DIAGNOSTIC_SCHEMA,
};

fn src(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "app/main.noi", text).unwrap()
}

#[test]
fn diagnostic_schema_and_codes_are_stable() {
    assert_eq!(V15_DIAGNOSTIC_SCHEMA, "nordoi.diagnostic.v1");
    assert_eq!(NDX_MANIFEST, "NDX1001");
    assert_eq!(NDX_SOURCE, "NDX2002");
    assert_eq!(NDX_IMPORT_CYCLE, "NDX2005");
}

#[test]
fn text_renderer_includes_code_precise_location_and_excerpt() {
    let source = src("module app.main;\nentry main returns 1;\n");
    let start = source.text().find("entry").unwrap() as u32;
    let span = source
        .span(ByteOffset::new(start), ByteOffset::new(start + 5))
        .unwrap();
    let diagnostic =
        V15Diagnostic::new(NDX_SOURCE, "source", "example").with_location(&source, span);
    let rendered = diagnostic.render_text();
    assert!(rendered.contains("error[NDX2002] source: example"));
    assert!(rendered.contains("app/main.noi:2:1"));
    assert!(rendered.contains("2 | entry main returns 1;"));
    assert!(rendered.contains("^^^^^"));
}

#[test]
fn json_renderer_is_single_schema_versioned_object_and_escapes_text() {
    let diagnostic = V15Diagnostic::new(NDX_SOURCE, "source", "bad \"name\"\nnext");
    let rendered = diagnostic.render_json();
    assert!(rendered.starts_with("{\"schema\":\"nordoi.diagnostic.v1\""));
    assert!(rendered.contains("\"status\":\"error\""));
    assert!(rendered.contains("bad \\\"name\\\"\\nnext"));
    assert!(!rendered.contains('\n'));
}

#[test]
fn import_trace_removes_only_adjacent_duplicates_and_preserves_order() {
    let diagnostic = V15Diagnostic::new(NDX_SOURCE, "source", "x").with_import_trace(vec![
        "app.main".into(),
        "app.main".into(),
        "app.mid".into(),
        "app.math".into(),
    ]);
    assert_eq!(
        diagnostic.import_trace(),
        &[
            "app.main".to_owned(),
            "app.mid".to_owned(),
            "app.math".to_owned(),
        ]
    );
}

#[test]
fn parent_map_produces_entry_to_leaf_trace() {
    let mut parents = BTreeMap::new();
    parents.insert("app.mid".to_owned(), "app.main".to_owned());
    parents.insert("app.math".to_owned(), "app.mid".to_owned());
    assert_eq!(
        import_trace_from_parents_v15(&parents, "app.math"),
        vec![
            "app.main".to_owned(),
            "app.mid".to_owned(),
            "app.math".to_owned(),
        ]
    );
}

#[test]
fn cycle_detection_is_deterministic_and_closes_the_cycle() {
    let mut graph = BTreeMap::new();
    graph.insert("app.a".to_owned(), vec!["app.b".to_owned()]);
    graph.insert("app.b".to_owned(), vec!["app.a".to_owned()]);
    let cycle = detect_import_cycle_v15(&graph).unwrap();
    assert_eq!(
        cycle,
        vec!["app.a".to_owned(), "app.b".to_owned(), "app.a".to_owned()]
    );
}

#[test]
fn acyclic_graph_has_no_cycle_diagnostic() {
    let mut graph = BTreeMap::new();
    graph.insert("app.main".to_owned(), vec!["app.mid".to_owned()]);
    graph.insert("app.mid".to_owned(), vec!["app.math".to_owned()]);
    graph.insert("app.math".to_owned(), vec![]);
    assert!(detect_import_cycle_v15(&graph).is_none());
}

#[test]
fn manifest_errors_map_to_stable_manifest_code() {
    let error = parse_project_manifest_v14("name = \"demo\"\n").unwrap_err();
    let diagnostic = diagnostic_from_project_error_v15(&error, &[], vec![]);
    assert_eq!(diagnostic.code(), NDX_MANIFEST);
    assert_eq!(diagnostic.stage(), "manifest");
}
