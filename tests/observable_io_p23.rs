use nordoi_kernel::{
    compile_structured_observable_output_plan_p23, execute_structured_observable_output_p23,
    Capability, CapabilitySet, DynamicValueKind, SourceId, SourceText, MAX_P23_OUTPUT_BYTES,
};

fn source(id: u32, name: &str, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority
}

#[test]
fn prefix_plus_dynamic_integer_compiles_to_one_structured_plan() {
    let src = source(
        1,
        "value.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"value=\" + key_code;",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    assert_eq!(plan.prefix(), "value=");
    assert_eq!(plan.suffix(), "");
    assert_eq!(plan.result_kind(), DynamicValueKind::Int);
    assert_eq!(plan.input_name(), "key_code");
}

#[test]
fn prefix_plus_dynamic_boolean_compiles_to_one_structured_plan() {
    let src = source(
        1,
        "bool.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"accepted=\" + (key_code >= 40);",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    assert_eq!(plan.prefix(), "accepted=");
    assert_eq!(plan.result_kind(), DynamicValueKind::Bool);
}

#[test]
fn optional_suffix_is_part_of_the_canonical_structured_plan() {
    let src = source(
        1,
        "suffix.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"[\" + key_code + \"]\";",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    assert_eq!(plan.prefix(), "[");
    assert_eq!(plan.suffix(), "]");
}

#[test]
fn arithmetic_stays_inside_the_certified_runtime_segment() {
    let src = source(
        1,
        "arithmetic.noi",
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits \"value=\" + (key_code + bias);",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    let receipt = execute_structured_observable_output_p23(&plan, 40, &authority()).unwrap();
    assert_eq!(receipt.output(), "value=42");
    assert_eq!(receipt.result().as_text(), "INT(42)");
}

#[test]
fn explicit_grant_emits_prefix_runtime_value_and_suffix() {
    let src = source(
        1,
        "full.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"code=[\" + key_code + \"]\";",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    let receipt = execute_structured_observable_output_p23(&plan, 41, &authority()).unwrap();
    assert_eq!(receipt.output(), "code=[41]");
    assert!(receipt.render_text().contains("status=EMITTED"));
    assert!(receipt.render_text().contains("authority=EXPLICIT"));
}

#[test]
fn explicit_grant_emits_canonical_runtime_boolean_text() {
    let src = source(
        1,
        "bool-output.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"accepted=\" + (key_code >= 40);",
    );
    let plan = compile_structured_observable_output_plan_p23(&src).unwrap();
    let receipt = execute_structured_observable_output_p23(&plan, 41, &authority()).unwrap();
    assert_eq!(receipt.output(), "accepted=true");
}

#[test]
fn runtime_segment_must_depend_on_explicit_input() {
    let src = source(
        1,
        "static.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"value=\" + 42;",
    );
    let error = compile_structured_observable_output_plan_p23(&src).unwrap_err();
    assert!(error
        .to_string()
        .contains("must depend on explicit runtime input"));
}

#[test]
fn structured_output_quota_is_proved_before_authority() {
    let prefix = "x".repeat(MAX_P23_OUTPUT_BYTES);
    let src = source(
        1,
        "quota.noi",
        format!(
            "module app.main; effect ConsoleWrite; input key_code; entry main emits \"{prefix}\" + key_code;"
        ),
    );
    let error = compile_structured_observable_output_plan_p23(&src).unwrap_err();
    assert!(error.to_string().contains("exceeding bound"));
}
