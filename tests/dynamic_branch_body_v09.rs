use nordoi_kernel::{
    compile_dynamic_branch_body_plan_v09, execute_dynamic_branch_body_source_v09,
    lower_dynamic_branch_body_plan_v09, DynamicValue, InputBatch, InputDeviceId, InputEvent,
    InputPayload, InputSequence, InputSource, InputTarget, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v09.noi", text).unwrap()
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
fn dynamic_true_branch_body_computes_141() {
    let src = source(
        "input key_code; entry main returns if key_code > 40 { key_code + 100 } else { key_code + 200 };",
    );
    let report = execute_dynamic_branch_body_source_v09(&src, &key_batch(41)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(141));
    assert_eq!(report.runtime_branches(), 1);
    assert_eq!(report.selected_branch_instructions(), 3);
    assert_eq!(report.discarded_branch_instructions(), 0);
    assert_eq!(report.lowering().nair_format_minor(), 11);
    assert_eq!(report.lowering().selective_branch_count(), 1);
}

#[test]
fn dynamic_false_branch_body_computes_239() {
    let src = source(
        "input key_code; entry main returns if key_code > 40 { key_code + 100 } else { key_code + 200 };",
    );
    let report = execute_dynamic_branch_body_source_v09(&src, &key_batch(39)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(239));
    assert_eq!(report.selected_branch_instructions(), 3);
    assert_eq!(report.discarded_branch_instructions(), 0);
}

#[test]
fn constant_branch_values_preserve_nair_010() {
    let src = source("input key_code; entry main returns if key_code > 40 { 100 } else { 200 };");
    let lowering =
        lower_dynamic_branch_body_plan_v09(&compile_dynamic_branch_body_plan_v09(&src).unwrap())
            .unwrap();
    assert_eq!(lowering.nair_format_minor(), 10);
    assert_eq!(lowering.selective_branch_count(), 0);
    assert_eq!(lowering.dynamic_branch_count(), 1);
}

#[test]
fn static_if_still_erases_to_base_nair() {
    let src = source("entry main returns if 2 > 1 { 42 } else { 7 };");
    let plan = compile_dynamic_branch_body_plan_v09(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_dynamic_branch_body_plan_v09(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.dynamic_branch_count(), 0);
}

#[test]
fn static_condition_selects_dynamic_expression_without_runtime_branch() {
    let src = source("input key_code; entry main returns if 2 > 1 { key_code + 2 } else { 0 };");
    let report = execute_dynamic_branch_body_source_v09(&src, &key_batch(40)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(42));
    assert_eq!(report.runtime_branches(), 0);
    assert_eq!(report.lowering().nair_format_minor(), 9);
}

#[test]
fn unselected_dynamic_overflow_is_not_evaluated() {
    let src = source(
        "input key_code; entry main returns if key_code > 40 { key_code + 9223372036854775807 } else { key_code + 200 };",
    );
    let report = execute_dynamic_branch_body_source_v09(&src, &key_batch(39)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(239));
    assert_eq!(report.discarded_branch_instructions(), 0);
}

#[test]
fn nested_dynamic_branch_body_is_deferred_fail_closed() {
    let src = source(
        "input key_code; entry main returns if key_code > 40 { if key_code > 50 { 1 } else { 2 } } else { 3 };",
    );
    let error =
        lower_dynamic_branch_body_plan_v09(&compile_dynamic_branch_body_plan_v09(&src).unwrap())
            .unwrap_err();
    assert!(format!("{error}").contains("do not yet permit nested dynamic if/else"));
}

#[test]
fn repeated_selective_execution_is_byte_deterministic() {
    let src = source(
        "input key_code; entry main returns if key_code >= 40 { key_code + 2 } else { key_code + 3 };",
    );
    let a = execute_dynamic_branch_body_source_v09(&src, &key_batch(40)).unwrap();
    let b = execute_dynamic_branch_body_source_v09(&src, &key_batch(40)).unwrap();
    assert_eq!(
        a.canonical_v09_receipt_bytes(),
        b.canonical_v09_receipt_bytes()
    );
    assert_eq!(
        a.lowering().canonical_nair_bytes(),
        b.lowering().canonical_nair_bytes()
    );
}
