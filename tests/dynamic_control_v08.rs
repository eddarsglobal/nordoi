use nordoi_kernel::{
    compile_dynamic_control_plan_v08, execute_dynamic_control_source_v08,
    lower_dynamic_control_plan_v08, DynamicValue, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v08.noi", text).unwrap()
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
fn dynamic_if_true_executes_then_path() {
    let src = source("input key_code; entry main returns if key_code > 40 { 100 } else { 200 };");
    let report = execute_dynamic_control_source_v08(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(100));
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.lowering().dynamic_branch_count(), 1);
    assert_eq!(report.lowering().nair_format_minor(), 10);
}

#[test]
fn dynamic_if_false_executes_else_path() {
    let src = source("input key_code; entry main returns if key_code > 40 { 100 } else { 200 };");
    let report = execute_dynamic_control_source_v08(&src, &key_batch(39)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(200));
    assert_eq!(report.runtime_branches(), 1);
}

#[test]
fn static_if_still_erases_before_nair() {
    let src = source("entry main returns if 2 > 1 { 42 } else { 7 };");
    let plan = compile_dynamic_control_plan_v08(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_dynamic_control_plan_v08(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.dynamic_branch_count(), 0);
}

#[test]
fn static_condition_can_select_dynamic_path_without_runtime_branch() {
    let src = source("input key_code; entry main returns if 2 > 1 { key_code + 2 } else { 0 };");
    let report = execute_dynamic_control_source_v08(&src, &key_batch(40)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(42));
    assert_eq!(report.runtime_branches(), 0);
    assert_eq!(report.lowering().nair_format_minor(), 9);
}

#[test]
fn dynamic_branch_arms_must_be_statically_reducible_in_v08() {
    let src =
        source("input key_code; entry main returns if key_code > 40 { key_code + 1 } else { 0 };");
    let error = lower_dynamic_control_plan_v08(&compile_dynamic_control_plan_v08(&src).unwrap())
        .unwrap_err();
    assert!(format!("{error}").contains("branch arms must be statically reducible"));
}

#[test]
fn branch_result_kinds_must_match() {
    let src = source("input key_code; entry main returns if key_code > 40 { 1 } else { 2 == 2 };");
    let error = compile_dynamic_control_plan_v08(&src).unwrap_err();
    assert!(format!("{error}").contains("branches must produce the same value kind"));
}

#[test]
fn static_nested_if_inside_dynamic_arm_is_folded() {
    let src = source(
        "input key_code; entry main returns if key_code > 40 { if 2 > 1 { 100 } else { 101 } } else { 200 };",
    );
    let report = execute_dynamic_control_source_v08(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(100));
    assert_eq!(report.runtime_branches(), 1);
}

#[test]
fn repeated_dynamic_control_execution_is_byte_deterministic() {
    let src = source("input key_code; entry main returns if key_code >= 40 { 100 } else { 200 };");
    let a = execute_dynamic_control_source_v08(&src, &key_batch(40)).unwrap();
    let b = execute_dynamic_control_source_v08(&src, &key_batch(40)).unwrap();
    assert_eq!(
        a.canonical_v08_receipt_bytes(),
        b.canonical_v08_receipt_bytes()
    );
    assert_eq!(
        a.lowering().canonical_nair_bytes(),
        b.lowering().canonical_nair_bytes()
    );
}
