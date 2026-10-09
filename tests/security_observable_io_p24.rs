use nordoi_kernel::{
    compile_multi_segment_output_plan_p24, execute_multi_segment_output_p24, Capability,
    CapabilitySet, SourceId, SourceText,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn plan(name: &str) -> nordoi_kernel::P24MultiSegmentOutputPlan {
    let src = source(
        1,
        name,
        "module app.main; effect ConsoleWrite; input key_code; const bias = 1; entry main emits \"input=\" + key_code + \", next=\" + (key_code + bias) + \", accepted=\" + (key_code >= 40);",
    );
    compile_multi_segment_output_plan_p24(&src).unwrap()
}

#[test]
fn missing_console_grant_is_denied_before_multi_segment_materialization() {
    let plan = plan("denied.noi");
    let error = execute_multi_segment_output_p24(&plan, 40, &CapabilitySet::new()).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
    assert!(error.to_string().contains("ConsoleWrite"));
}

#[test]
fn unrelated_capability_cannot_authorize_multi_segment_output() {
    let plan = plan("unrelated.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::Camera);
    assert!(execute_multi_segment_output_p24(&plan, 40, &authority).is_err());
}

#[test]
fn exact_console_capability_authorizes_multi_segment_output() {
    let plan = plan("granted.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let receipt = execute_multi_segment_output_p24(&plan, 40, &authority).unwrap();
    assert_eq!(receipt.output(), "input=40, next=41, accepted=true");
    assert!(receipt.render_text().contains("ambient-authority=NONE"));
}

#[test]
fn revoked_console_capability_fails_closed() {
    let plan = plan("revoked.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority.revoke(&Capability::ConsoleWrite);
    assert!(execute_multi_segment_output_p24(&plan, 40, &authority).is_err());
}

#[test]
fn equal_input_produces_equal_segment_and_p24_receipts() {
    let plan = plan("deterministic.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let first = execute_multi_segment_output_p24(&plan, 40, &authority).unwrap();
    let second = execute_multi_segment_output_p24(&plan, 40, &authority).unwrap();
    assert_eq!(first.output(), second.output());
    assert_eq!(
        first.segment_receipt_sha256_hex(),
        second.segment_receipt_sha256_hex()
    );
    assert_eq!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_eq!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
}

#[test]
fn changing_runtime_input_changes_global_receipt_and_output() {
    let plan = plan("input-change.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let first = execute_multi_segment_output_p24(&plan, 40, &authority).unwrap();
    let second = execute_multi_segment_output_p24(&plan, 41, &authority).unwrap();
    assert_eq!(first.output(), "input=40, next=41, accepted=true");
    assert_eq!(second.output(), "input=41, next=42, accepted=true");
    assert_ne!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    assert_ne!(
        first.segment_receipt_sha256_hex(),
        second.segment_receipt_sha256_hex()
    );
}

#[test]
fn host_source_name_does_not_change_p24_plan_identity() {
    let text = "module app.main; effect ConsoleWrite; input key_code; entry main emits \"a=\" + key_code + \", b=\" + (key_code + 1);";
    let first = source(1, "/tmp/a.noi", text);
    let second = source(99, "C:\\different\\host\\b.noi", text);
    let first = compile_multi_segment_output_plan_p24(&first).unwrap();
    let second = compile_multi_segment_output_plan_p24(&second).unwrap();
    assert_eq!(first.plan_sha256_hex(), second.plan_sha256_hex());
    assert_eq!(first.canonical_plan_bytes(), second.canonical_plan_bytes());
}

#[test]
fn changing_static_template_changes_p24_plan_identity() {
    let first = source(
        1,
        "one.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"a=\" + key_code + \", b=\" + (key_code + 1);",
    );
    let second = source(
        2,
        "two.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits \"x=\" + key_code + \", y=\" + (key_code + 1);",
    );
    let first = compile_multi_segment_output_plan_p24(&first).unwrap();
    let second = compile_multi_segment_output_plan_p24(&second).unwrap();
    assert_ne!(first.plan_sha256_hex(), second.plan_sha256_hex());
    assert_ne!(first.canonical_plan_bytes(), second.canonical_plan_bytes());
}
