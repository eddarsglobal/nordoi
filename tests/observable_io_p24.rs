use nordoi_kernel::{
    compile_multi_segment_output_plan_p24, execute_multi_segment_output_p24, Capability,
    CapabilitySet, DynamicValueKind, SourceId, SourceText, MAX_P24_RUNTIME_SEGMENTS,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "p24.noi", text).unwrap()
}

fn grant() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority
}

#[test]
fn one_runtime_segment_remains_p23_and_is_rejected_from_p24() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"value=\" + key_code;",
    );
    let error = compile_multi_segment_output_plan_p24(&src).unwrap_err();
    assert!(error.to_string().contains("at least two runtime segments"));
    assert!(error.to_string().contains("P2.3"));
}

#[test]
fn two_dynamic_integer_segments_compile_to_one_multi_segment_plan() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"a=\" + key_code + \", b=\" + (key_code + 1);",
    );
    let plan = compile_multi_segment_output_plan_p24(&src).unwrap();
    assert_eq!(plan.runtime_segment_count(), 2);
    assert_eq!(
        plan.result_kinds(),
        vec![DynamicValueKind::Int, DynamicValueKind::Int]
    );
    let text = plan
        .static_segments()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(text, vec!["a=", ", b=", ""]);
}

#[test]
fn arithmetic_stays_inside_each_certified_p23_runtime_segment() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits \"raw=\" + key_code + \", shifted=\" + (key_code + bias);",
    );
    let plan = compile_multi_segment_output_plan_p24(&src).unwrap();
    let receipt = execute_multi_segment_output_p24(&plan, 40, &grant()).unwrap();
    assert_eq!(receipt.output(), "raw=40, shifted=42");
}

#[test]
fn mixed_integer_and_boolean_segments_are_typed_independently() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"code=\" + key_code + \", accepted=\" + (key_code >= 40);",
    );
    let plan = compile_multi_segment_output_plan_p24(&src).unwrap();
    assert_eq!(
        plan.result_kinds(),
        vec![DynamicValueKind::Int, DynamicValueKind::Bool]
    );
    let receipt = execute_multi_segment_output_p24(&plan, 41, &grant()).unwrap();
    assert_eq!(receipt.output(), "code=41, accepted=true");
}

#[test]
fn explicit_grant_emits_three_ordered_runtime_segments() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; const bias = 1; entry main emits \"input=\" + key_code + \", next=\" + (key_code + bias) + \", accepted=\" + (key_code >= 40);",
    );
    let plan = compile_multi_segment_output_plan_p24(&src).unwrap();
    let receipt = execute_multi_segment_output_p24(&plan, 40, &grant()).unwrap();
    assert_eq!(receipt.output(), "input=40, next=41, accepted=true");
    assert_eq!(receipt.results().len(), 3);
    assert_eq!(receipt.segment_receipt_sha256_hex().len(), 3);
    assert!(receipt.render_text().contains("segments=3"));
}

#[test]
fn trailing_static_segment_is_committed_to_output() {
    let src = source(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"[\" + key_code + \"|\" + (key_code + 1) + \"]\";",
    );
    let plan = compile_multi_segment_output_plan_p24(&src).unwrap();
    let receipt = execute_multi_segment_output_p24(&plan, 7, &grant()).unwrap();
    assert_eq!(receipt.output(), "[7|8]");
}

#[test]
fn runtime_segment_count_is_bounded_before_authority() {
    let mut expression = String::from("\"s0=\" + key_code");
    for index in 1..=MAX_P24_RUNTIME_SEGMENTS {
        expression.push_str(&format!(" + \", s{index}=\" + key_code"));
    }
    let text = format!(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits {expression};"
    );
    let error = compile_multi_segment_output_plan_p24(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("at most"));
    assert!(error
        .to_string()
        .contains(&MAX_P24_RUNTIME_SEGMENTS.to_string()));
}

#[test]
fn total_output_quota_is_proved_before_authority() {
    let prefix = "x".repeat(4070);
    let text = format!(
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"{prefix}\" + key_code + \",\" + key_code;"
    );
    let error = compile_multi_segment_output_plan_p24(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("exceeding bound 4096"));
}
