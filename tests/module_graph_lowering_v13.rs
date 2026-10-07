use nordoi_kernel::{
    compile_module_graph_v13, execute_module_graph_v13, lower_module_graph_v13, DynamicValue,
    InputBatch, InputDeviceId, InputEvent, InputPayload, InputSequence, InputSource, InputTarget,
    SourceId, SourceText,
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

#[test]
fn modules_are_erased_before_nair_and_simple_call_stays_012() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);"),
    ];
    let plan = compile_module_graph_v13(&sources, "app.main").unwrap();
    let lowering = lower_module_graph_v13(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 12);
    assert_eq!(lowering.nair_instruction_count(), 3);
}

#[test]
fn transitive_imported_call_graph_uses_existing_013() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "mid.noi", "module app.mid; import app.math; fn plus(x) returns math.add_100(x);"),
        src(3, "main.noi", "module app.main; import app.mid; input key_code; entry main returns mid.plus(key_code);"),
    ];
    let plan = compile_module_graph_v13(&sources, "app.main").unwrap();
    let lowering = lower_module_graph_v13(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 13);
}

#[test]
fn imported_structured_control_uses_existing_014() {
    let sources = vec![
        src(1, "rules.noi", "module app.rules; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.bias(key_code);"),
    ];
    let report = execute_module_graph_v13(&sources, "app.main", &key(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.nair_format_minor(), 14);
}

#[test]
fn static_imported_call_folds_to_base_06() {
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
    let plan = compile_module_graph_v13(&sources, "app.main").unwrap();
    let lowering = lower_module_graph_v13(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
}

#[test]
fn imported_function_runtime_metrics_remain_exact() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);"),
    ];
    let report = execute_module_graph_v13(&sources, "app.main", &key(41)).unwrap();
    assert_eq!(report.runtime_calls(), 1);
    assert_eq!(report.runtime_branches(), 0);
    assert_eq!(report.max_call_depth(), 1);
}

#[test]
fn imported_structured_control_keeps_lazy_branching() {
    let sources = vec![
        src(1, "rules.noi", "module app.rules; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 };"),
        src(2, "main.noi", "module app.main; import app.rules; input key_code; entry main returns rules.bias(key_code);"),
    ];
    let a = execute_module_graph_v13(&sources, "app.main", &key(41)).unwrap();
    let b = execute_module_graph_v13(&sources, "app.main", &key(39)).unwrap();
    assert_eq!(a.result(), DynamicValue::Int(141));
    assert_eq!(b.result(), DynamicValue::Int(239));
    assert_eq!(a.runtime_branches(), 1);
    assert_eq!(b.runtime_branches(), 1);
}

#[test]
fn module_graph_lowering_witness_is_deterministic() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);"),
    ];
    let plan_a = compile_module_graph_v13(&sources, "app.main").unwrap();
    let plan_b =
        compile_module_graph_v13(&[sources[1].clone(), sources[0].clone()], "app.main").unwrap();
    let a = lower_module_graph_v13(&plan_a).unwrap();
    let b = lower_module_graph_v13(&plan_b).unwrap();
    assert_eq!(
        a.canonical_v13_lowering_witness_bytes(),
        b.canonical_v13_lowering_witness_bytes()
    );
}

#[test]
fn qualified_unknown_function_is_rejected_before_runtime() {
    let sources = vec![
        src(1, "math.noi", "module app.math; fn add_100(x) returns x + 100;"),
        src(2, "main.noi", "module app.main; import app.math; input key_code; entry main returns math.missing(key_code);"),
    ];
    let error = compile_module_graph_v13(&sources, "app.main").unwrap_err();
    assert!(format!("{error}").contains("does not exist"));
}
