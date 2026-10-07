use nordoi_kernel::{
    compile_dynamic_plan_v07, execute_dynamic_source_v07, lower_dynamic_plan_v07, DynamicValue,
    InputBatch, InputDeviceId, InputEvent, InputPayload, InputSequence, InputSource, InputTarget,
    SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "v07.noi", text).unwrap()
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
fn dynamic_source_executes_key_code_plus_constant_to_42() {
    let src = source(
        "module demo.dynamic; type User; effect Network; input key_code; const bias = 2; entry main returns key_code + bias;",
    );
    let report = execute_dynamic_source_v07(&src, &key_batch(40)).unwrap();
    assert_eq!(report.result(), DynamicValue::Int(42));
    assert!(report.runtime_computed());
    assert_eq!(report.lowering().nair_format_minor(), 9);
    assert_eq!(report.lowering().nair_instruction_count(), 4);
}

#[test]
fn dynamic_comparison_executes_at_runtime() {
    let src = source("input key_code; entry main returns key_code >= 40;");
    assert_eq!(
        execute_dynamic_source_v07(&src, &key_batch(40))
            .unwrap()
            .result(),
        DynamicValue::Bool(true)
    );
    assert_eq!(
        execute_dynamic_source_v07(&src, &key_batch(39))
            .unwrap()
            .result(),
        DynamicValue::Bool(false)
    );
}

#[test]
fn static_v07_subset_still_collapses_to_const_halt() {
    let src = source("const a = 20; const b = 22; entry main returns a + b;");
    let plan = compile_dynamic_plan_v07(&src).unwrap();
    assert!(!plan.is_dynamic());
    let lowering = lower_dynamic_plan_v07(&plan).unwrap();
    assert_eq!(lowering.nair_format_minor(), 6);
    assert_eq!(lowering.nair_instruction_count(), 2);
}

#[test]
fn unused_input_does_not_force_runtime_work() {
    let src = source("input key_code; entry main returns 42;");
    let plan = compile_dynamic_plan_v07(&src).unwrap();
    assert!(!plan.is_dynamic());
    assert_eq!(
        lower_dynamic_plan_v07(&plan).unwrap().nair_format_minor(),
        6
    );
}

#[test]
fn constant_cannot_depend_on_runtime_input() {
    let src = source("input key_code; const bad = key_code + 1; entry main returns bad;");
    let error = compile_dynamic_plan_v07(&src).unwrap_err();
    assert!(format!("{error}").contains("cannot depend on runtime input"));
}

#[test]
fn duplicate_dynamic_scope_name_is_rejected() {
    let src = source("input value; const value = 1; entry main returns value;");
    let error = compile_dynamic_plan_v07(&src).unwrap_err();
    assert!(format!("{error}").contains("duplicate dynamic-scope name"));
}

#[test]
fn unknown_name_is_rejected() {
    let src = source("input key_code; entry main returns missing + 1;");
    let error = compile_dynamic_plan_v07(&src).unwrap_err();
    assert!(format!("{error}").contains("unknown name 'missing'"));
}

#[test]
fn repeated_execution_is_byte_deterministic() {
    let src = source("input key_code; const bias = 2; entry main returns key_code + bias;");
    let a = execute_dynamic_source_v07(&src, &key_batch(40)).unwrap();
    let b = execute_dynamic_source_v07(&src, &key_batch(40)).unwrap();
    assert_eq!(
        a.canonical_v07_receipt_bytes(),
        b.canonical_v07_receipt_bytes()
    );
    assert_eq!(
        a.lowering().canonical_nair_bytes(),
        b.lowering().canonical_nair_bytes()
    );
}
