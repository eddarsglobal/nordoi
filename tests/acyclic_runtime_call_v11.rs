use nordoi_kernel::{
    compile_acyclic_runtime_call_plan_v11, execute_acyclic_runtime_call_source_v11,
    lower_acyclic_runtime_call_plan_v11, DynamicValue, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
    MAX_V11_RUNTIME_CALL_DEPTH,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v11.noi", text).unwrap()
}

fn key_batch(code: u32) -> InputBatch {
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
fn two_level_dynamic_call_graph_computes_241() {
    let src = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns add_100(x) + 100; input key_code; entry main returns add_200(key_code);",
    );
    let report = execute_acyclic_runtime_call_source_v11(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(241));
    assert_eq!(report.runtime_calls(), 2);
    assert_eq!(report.max_call_depth(), 2);
    assert_eq!(report.runtime_branches(), 0);
    assert_eq!(report.lowering().nair_format_minor(), 13);
    assert_eq!(report.plan().max_call_depth(), 2);
}

#[test]
fn three_level_dynamic_call_graph_computes_341() {
    let src = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns add_100(x) + 100; fn add_300(x) returns add_200(x) + 100; input key_code; entry main returns add_300(key_code);",
    );
    let report = execute_acyclic_runtime_call_source_v11(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(341));
    assert_eq!(report.runtime_calls(), 3);
    assert_eq!(report.max_call_depth(), 3);
}

#[test]
fn direct_recursion_is_rejected_before_lowering() {
    let src =
        source("fn loop(x) returns loop(x); input key_code; entry main returns loop(key_code);");
    let error = compile_acyclic_runtime_call_plan_v11(&src).unwrap_err();
    assert!(format!("{error}").contains("recursive or cyclic"));
}

#[test]
fn indirect_cycle_is_rejected_before_lowering() {
    let src = source(
        "fn alpha(x) returns beta(x); fn beta(x) returns alpha(x); input key_code; entry main returns alpha(key_code);",
    );
    let error = compile_acyclic_runtime_call_plan_v11(&src).unwrap_err();
    assert!(format!("{error}").contains("recursive or cyclic"));
}

#[test]
fn unknown_function_inside_function_body_is_rejected() {
    let src = source(
        "fn outer(x) returns missing(x); input key_code; entry main returns outer(key_code);",
    );
    let error = compile_acyclic_runtime_call_plan_v11(&src).unwrap_err();
    assert!(format!("{error}").contains("calls unknown function 'missing'"));
}

#[test]
fn static_acyclic_chain_folds_to_base_nair() {
    let src = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns add_100(x) + 100; entry main returns add_200(41);",
    );
    let plan = compile_acyclic_runtime_call_plan_v11(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_acyclic_runtime_call_plan_v11(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.runtime_call_count(), 0);
}

#[test]
fn declaration_order_does_not_change_canonical_semantics() {
    let a = source(
        "fn add_200(x) returns add_100(x) + 100; fn add_100(x) returns x + 100; input key_code; entry main returns add_200(key_code);",
    );
    let b = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns add_100(x) + 100; input key_code; entry main returns add_200(key_code);",
    );
    let a = compile_acyclic_runtime_call_plan_v11(&a).unwrap();
    let b = compile_acyclic_runtime_call_plan_v11(&b).unwrap();
    assert_eq!(
        a.canonical_v11_semantic_bytes(),
        b.canonical_v11_semantic_bytes()
    );
}

#[test]
fn call_graph_depth_bound_is_enforced() {
    let mut text = String::new();
    for i in 0..=MAX_V11_RUNTIME_CALL_DEPTH {
        if i == 0 {
            text.push_str("fn f0(x) returns x + 1; ");
        } else {
            text.push_str(&format!("fn f{i}(x) returns f{}(x) + 1; ", i - 1));
        }
    }
    text.push_str(&format!(
        "input key_code; entry main returns f{}(key_code);",
        MAX_V11_RUNTIME_CALL_DEPTH
    ));
    let src = source(&text);
    let error = compile_acyclic_runtime_call_plan_v11(&src).unwrap_err();
    assert!(format!("{error}").contains("exceeding the V1.1 bound"));
}
