use nordoi_kernel::{
    compile_dynamic_observable_output_plan_p22, execute_dynamic_observable_output_p22, Capability,
    CapabilitySet, SourceId, SourceText,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn plan(name: &str) -> nordoi_kernel::P22DynamicObservableOutputPlan {
    let src = source(
        1,
        name,
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;",
    );
    compile_dynamic_observable_output_plan_p22(&src).unwrap()
}

#[test]
fn missing_console_grant_is_denied_before_output_materialization() {
    let plan = plan("denied.noi");
    let error =
        execute_dynamic_observable_output_p22(&plan, 40, &CapabilitySet::new()).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
    assert!(error.to_string().contains("ConsoleWrite"));
}

#[test]
fn unrelated_capability_cannot_authorize_dynamic_console_output() {
    let plan = plan("unrelated.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::Camera);
    let error = execute_dynamic_observable_output_p22(&plan, 40, &authority).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
}

#[test]
fn exact_console_capability_authorizes_dynamic_output() {
    let plan = plan("granted.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let receipt = execute_dynamic_observable_output_p22(&plan, 40, &authority).unwrap();
    assert_eq!(receipt.output(), "42");
    assert!(receipt.render_text().contains("authority=EXPLICIT"));
    assert!(receipt.render_text().contains("ambient-authority=NONE"));
}

#[test]
fn revoked_console_capability_fails_closed() {
    let plan = plan("revoked.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority.revoke(&Capability::ConsoleWrite);
    assert!(execute_dynamic_observable_output_p22(&plan, 40, &authority).is_err());
}

#[test]
fn equal_input_produces_equal_runtime_and_p22_receipts() {
    let plan = plan("deterministic.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let first = execute_dynamic_observable_output_p22(&plan, 40, &authority).unwrap();
    let second = execute_dynamic_observable_output_p22(&plan, 40, &authority).unwrap();
    assert_eq!(first.output(), second.output());
    assert_eq!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_eq!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    assert_eq!(
        first.runtime_receipt_sha256_hex(),
        second.runtime_receipt_sha256_hex()
    );
}

#[test]
fn changing_runtime_input_changes_receipt_and_observable_output() {
    let plan = plan("input-change.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let first = execute_dynamic_observable_output_p22(&plan, 40, &authority).unwrap();
    let second = execute_dynamic_observable_output_p22(&plan, 41, &authority).unwrap();
    assert_eq!(first.output(), "42");
    assert_eq!(second.output(), "43");
    assert_ne!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    assert_ne!(
        first.runtime_receipt_sha256_hex(),
        second.runtime_receipt_sha256_hex()
    );
}

#[test]
fn host_source_name_does_not_change_p22_plan_identity() {
    let text =
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;";
    let first = source(1, "/tmp/a.noi", text);
    let second = source(99, "C:\\different\\host\\b.noi", text);
    let first = compile_dynamic_observable_output_plan_p22(&first).unwrap();
    let second = compile_dynamic_observable_output_plan_p22(&second).unwrap();
    assert_eq!(first.plan_sha256_hex(), second.plan_sha256_hex());
    assert_eq!(first.canonical_plan_bytes(), second.canonical_plan_bytes());
}

#[test]
fn capability_grant_does_not_bypass_source_policy() {
    let src = source(
        1,
        "bad.noi",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits 42;",
    );
    assert!(compile_dynamic_observable_output_plan_p22(&src).is_err());
}
