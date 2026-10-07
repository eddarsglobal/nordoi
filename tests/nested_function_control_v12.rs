use nordoi_kernel::{
    compile_nested_function_control_plan_v12, execute_nested_function_control_source_v12,
    lower_nested_function_control_plan_v12, DynamicValue, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v12.noi", text).unwrap()
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
fn dynamic_true_function_branch_computes_141() {
    let src = source(
        "fn bias(x) returns if x > 40 { x + 100 } else { x + 200 }; input key_code; entry main returns bias(key_code);",
    );
    let report = execute_nested_function_control_source_v12(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.runtime_calls(), 1);
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.max_call_depth(), 1);
    assert_eq!(report.lowering().nair_format_minor(), 14);
}

#[test]
fn dynamic_false_function_branch_computes_239() {
    let src = source(
        "fn bias(x) returns if x > 40 { x + 100 } else { x + 200 }; input key_code; entry main returns bias(key_code);",
    );
    let report = execute_nested_function_control_source_v12(&src, &key_batch(39)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(239));
    assert_eq!(report.runtime_calls(), 1);
    assert_eq!(report.runtime_branches(), 1);
}

#[test]
fn selected_branch_can_call_an_acyclic_pure_function() {
    let src = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns x + 200; fn choose(x) returns if x > 40 { add_100(x) } else { add_200(x) }; input key_code; entry main returns choose(key_code);",
    );
    let report = execute_nested_function_control_source_v12(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.runtime_calls(), 2);
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.max_call_depth(), 2);
    assert_eq!(report.plan().max_call_depth(), 2);
}

#[test]
fn alternate_selected_branch_can_call_an_acyclic_pure_function() {
    let src = source(
        "fn add_100(x) returns x + 100; fn add_200(x) returns x + 200; fn choose(x) returns if x > 40 { add_100(x) } else { add_200(x) }; input key_code; entry main returns choose(key_code);",
    );
    let report = execute_nested_function_control_source_v12(&src, &key_batch(39)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(239));
    assert_eq!(report.runtime_calls(), 2);
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.max_call_depth(), 2);
}

#[test]
fn recursion_remains_rejected_before_runtime() {
    let src = source(
        "fn loop(x) returns if x > 40 { loop(x) } else { x }; input key_code; entry main returns loop(key_code);",
    );
    let error = compile_nested_function_control_plan_v12(&src).unwrap_err();
    assert!(format!("{error}").contains("recursive or cyclic"));
}

#[test]
fn indirect_cycle_remains_rejected_before_runtime() {
    let src = source(
        "fn alpha(x) returns if x > 40 { beta(x) } else { x }; fn beta(x) returns alpha(x); input key_code; entry main returns alpha(key_code);",
    );
    let error = compile_nested_function_control_plan_v12(&src).unwrap_err();
    assert!(format!("{error}").contains("recursive or cyclic"));
}

#[test]
fn function_if_arms_must_have_same_kind() {
    let src = source(
        "fn bad(x) returns if x > 40 { x + 1 } else { x > 1 }; input key_code; entry main returns bad(key_code);",
    );
    let error = compile_nested_function_control_plan_v12(&src).unwrap_err();
    assert!(format!("{error}").contains("if arms must have the same value kind"));
}

#[test]
fn fully_static_function_control_still_folds_to_base_nair() {
    let src = source(
        "fn bias(x) returns if x > 40 { x + 100 } else { x + 200 }; entry main returns bias(41);",
    );
    let plan = compile_nested_function_control_plan_v12(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_nested_function_control_plan_v12(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.runtime_call_count(), 0);
}
