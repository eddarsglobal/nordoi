use nordoi_kernel::{
    compile_dynamic_observable_output_plan_p22, execute_dynamic_observable_output_p22, Capability,
    CapabilitySet, DynamicObservableIoError, DynamicValueKind, SourceId, SourceText,
    P22_OUTPUT_SCHEMA,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority
}

#[test]
fn dynamic_integer_expression_compiles_to_p22_plan() {
    let src = source(
        1,
        "dynamic.noi",
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;",
    );
    let plan = compile_dynamic_observable_output_plan_p22(&src).unwrap();
    assert_eq!(plan.module(), Some("app.main"));
    assert_eq!(plan.entry_name(), "main");
    assert_eq!(plan.input_name(), "key_code");
    assert_eq!(plan.result_kind(), DynamicValueKind::Int);
    assert_eq!(plan.plan_sha256().len(), 32);
}

#[test]
fn dynamic_boolean_expression_compiles_to_p22_plan() {
    let src = source(
        1,
        "dynamic.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits key_code >= 40;",
    );
    let plan = compile_dynamic_observable_output_plan_p22(&src).unwrap();
    assert_eq!(plan.result_kind(), DynamicValueKind::Bool);
}

#[test]
fn explicit_grant_emits_runtime_integer_result() {
    let src = source(
        1,
        "dynamic.noi",
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;",
    );
    let plan = compile_dynamic_observable_output_plan_p22(&src).unwrap();
    let receipt = execute_dynamic_observable_output_p22(&plan, 40, &authority()).unwrap();
    assert_eq!(receipt.output(), "42");
    assert_eq!(receipt.output_bytes(), 2);
    assert_eq!(receipt.key_code(), 40);
    assert!(receipt.render_json().contains(P22_OUTPUT_SCHEMA));
}

#[test]
fn explicit_grant_emits_runtime_boolean_result() {
    let src = source(
        1,
        "dynamic.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits key_code >= 40;",
    );
    let plan = compile_dynamic_observable_output_plan_p22(&src).unwrap();
    let true_receipt = execute_dynamic_observable_output_p22(&plan, 41, &authority()).unwrap();
    let false_receipt = execute_dynamic_observable_output_p22(&plan, 39, &authority()).unwrap();
    assert_eq!(true_receipt.output(), "true");
    assert_eq!(false_receipt.output(), "false");
}

#[test]
fn static_expression_is_rejected_from_p22() {
    let src = source(
        1,
        "static.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits 42;",
    );
    let error = compile_dynamic_observable_output_plan_p22(&src).unwrap_err();
    assert!(error.to_string().contains("requires output to depend"));
}

#[test]
fn quoted_text_remains_p21_and_is_rejected_from_p22() {
    let src = source(
        1,
        "quoted.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"42\";",
    );
    let error = compile_dynamic_observable_output_plan_p22(&src).unwrap_err();
    assert!(error.to_string().contains("quoted text remains P2.1"));
}

#[test]
fn missing_console_effect_is_rejected() {
    let src = source(
        1,
        "missing.noi",
        "module app.main; input key_code; entry main emits key_code + 1;",
    );
    let error = compile_dynamic_observable_output_plan_p22(&src).unwrap_err();
    assert!(error.to_string().contains("requires explicit"));
}

#[test]
fn duplicate_or_unrelated_effect_is_rejected_fail_closed() {
    let duplicate = source(
        1,
        "duplicate.noi",
        "module app.main; effect ConsoleWrite; effect ConsoleWrite; input key_code; entry main emits key_code + 1;",
    );
    assert!(compile_dynamic_observable_output_plan_p22(&duplicate)
        .unwrap_err()
        .to_string()
        .contains("duplicate effect"));

    let unrelated = source(
        2,
        "unrelated.noi",
        "module app.main; effect Network; input key_code; entry main emits key_code + 1;",
    );
    let error = compile_dynamic_observable_output_plan_p22(&unrelated).unwrap_err();
    assert!(matches!(error, DynamicObservableIoError::Policy { .. }));
    assert!(error
        .to_string()
        .contains("permits only effect ConsoleWrite"));
}
