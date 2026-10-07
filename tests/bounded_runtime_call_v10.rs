use nordoi_kernel::{
    compile_bounded_runtime_call_plan_v10, execute_bounded_runtime_call_source_v10,
    lower_bounded_runtime_call_plan_v10, DynamicValue, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v10.noi", text).unwrap()
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
fn dynamic_direct_call_computes_141() {
    let src = source(
        "fn add_bias(x) returns x + 100; input key_code; entry main returns add_bias(key_code);",
    );
    let report = execute_bounded_runtime_call_source_v10(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.runtime_calls(), 1);
    assert_eq!(report.runtime_branches(), 0);
    assert_eq!(report.call_body_instructions(), 3);
    assert_eq!(report.max_call_depth(), 1);
    assert_eq!(report.lowering().nair_format_minor(), 12);
}

#[test]
fn multi_parameter_direct_call_is_supported() {
    let src = source(
        "fn plus(x, y) returns x + y; input key_code; entry main returns plus(key_code, 1);",
    );
    let report = execute_bounded_runtime_call_source_v10(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(42));
    assert_eq!(report.runtime_calls(), 1);
    assert_eq!(report.max_call_depth(), 1);
}

#[test]
fn static_call_is_folded_before_nair() {
    let src = source("fn add_bias(x) returns x + 100; entry main returns add_bias(41);");
    let plan = compile_bounded_runtime_call_plan_v10(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_bounded_runtime_call_plan_v10(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.runtime_call_count(), 0);
}

#[test]
fn unknown_function_is_rejected() {
    let src = source(
        "fn add_bias(x) returns x + 100; input key_code; entry main returns missing(key_code);",
    );
    let error = compile_bounded_runtime_call_plan_v10(&src).unwrap_err();
    assert!(format!("{error}").contains("unknown function 'missing'"));
}

#[test]
fn arity_mismatch_is_rejected() {
    let src = source(
        "fn add_bias(x) returns x + 100; input key_code; entry main returns add_bias(key_code, 1);",
    );
    let error = compile_bounded_runtime_call_plan_v10(&src).unwrap_err();
    assert!(format!("{error}").contains("expects 1 argument"));
}

#[test]
fn function_body_call_is_rejected() {
    let src = source(
        "fn inner(x) returns x + 1; fn outer(x) returns inner(x); input key_code; entry main returns outer(key_code);",
    );
    let error = compile_bounded_runtime_call_plan_v10(&src).unwrap_err();
    assert!(format!("{error}").contains("function bodies cannot call functions"));
}

#[test]
fn nested_entry_call_is_rejected() {
    let src = source(
        "fn add_one(x) returns x + 1; fn add_two(x) returns x + 2; input key_code; entry main returns add_two(add_one(key_code));",
    );
    let error = compile_bounded_runtime_call_plan_v10(&src).unwrap_err();
    assert!(
        format!("{error}").contains("function-call")
            || format!("{error}").contains("nested")
            || format!("{error}").contains("cannot call functions")
    );
}

#[test]
fn declaration_order_does_not_change_canonical_semantics() {
    let a = source(
        "fn zed(x) returns x + 2; fn alpha(x) returns x + 1; input key_code; entry main returns alpha(key_code);",
    );
    let b = source(
        "fn alpha(x) returns x + 1; fn zed(x) returns x + 2; input key_code; entry main returns alpha(key_code);",
    );
    let a = compile_bounded_runtime_call_plan_v10(&a).unwrap();
    let b = compile_bounded_runtime_call_plan_v10(&b).unwrap();
    assert_eq!(
        a.canonical_v10_semantic_bytes(),
        b.canonical_v10_semantic_bytes()
    );
}
