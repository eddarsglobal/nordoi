use nordoi_kernel::{
    compile_module_graph_v13, execute_module_graph_v13, DynamicValue, InputBatch, InputDeviceId,
    InputEvent, InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
};

fn src(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn key(code: u32) -> InputBatch {
    InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        }],
    }
}

fn basic_sources() -> Vec<SourceText> {
    vec![
        src(
            1,
            "math.noi",
            "module app.math; fn add_100(x) returns x + 100;",
        ),
        src(
            2,
            "main.noi",
            "module app.main; type User; effect Network; import app.math; input key_code; entry main returns math.add_100(key_code);",
        ),
    ]
}

#[test]
fn real_import_executes_imported_function() {
    let report = execute_module_graph_v13(&basic_sources(), "app.main", &key(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.plan().module_count(), 2);
    assert_eq!(report.plan().import_count(), 1);
    assert_eq!(report.runtime_calls(), 1);
}

#[test]
fn transitive_imports_resolve_statically() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "mid.noi", "module app.mid; import app.math; fn plus(x) returns math.add_100(x);"),
        src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.plus(key_code);"),
    ];
    let report = execute_module_graph_v13(&sources, "app.main", &key(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.plan().module_count(), 3);
    assert_eq!(report.plan().import_count(), 2);
    assert_eq!(report.runtime_calls(), 2);
    assert_eq!(report.max_call_depth(), 2);
}

#[test]
fn import_cycle_is_rejected_before_lowering() {
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
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("cyclic import graph"));
}

#[test]
fn missing_imported_module_is_rejected() {
    let sources = vec![src(
        1,
        "main.noi",
        "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);",
    )];
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("imports missing module 'app.math'"));
}

#[test]
fn duplicate_module_identity_is_rejected() {
    let sources = vec![
        src(
            1,
            "one.noi",
            "module app.main; input key_code; entry main returns key_code;",
        ),
        src(
            2,
            "two.noi",
            "module app.main; input key_code; entry main returns key_code;",
        ),
    ];
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("duplicate canonical module identity"));
}

#[test]
fn ambiguous_import_alias_is_rejected() {
    let sources = vec![
        src(1, "left.noi", "module left.math; fn a(x) returns x + 1;"),
        src(2, "right.noi", "module right.math; fn b(x) returns x + 2;"),
        src(3, "main.noi", "module app.main; import left.math; import right.math; input key_code; entry main returns key_code;"),
    ];
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("ambiguous import alias 'math'"));
}

#[test]
fn imported_module_cannot_declare_input_or_entry() {
    let sources = vec![
        src(
            1,
            "lib.noi",
            "module app.lib; input key_code; entry lib returns key_code;",
        ),
        src(
            2,
            "main.noi",
            "module app.main; import app.lib; input key_code; entry main returns key_code;",
        ),
    ];
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("library-only"));
}

#[test]
fn source_order_does_not_change_module_graph_witness() {
    let forward = basic_sources();
    let reverse = vec![forward[1].clone(), forward[0].clone()];
    let a = compile_module_graph_v13(&forward, "app.main").unwrap();
    let b = compile_module_graph_v13(&reverse, "app.main").unwrap();
    assert_eq!(
        a.canonical_v13_witness_bytes(),
        b.canonical_v13_witness_bytes()
    );
    assert_eq!(a.module_order(), b.module_order());
}
