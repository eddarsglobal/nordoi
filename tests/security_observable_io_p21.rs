use nordoi_kernel::{
    compile_observable_output_plan_p21, execute_observable_output_p21,
    required_observable_capability_p21, Capability, CapabilitySet, ObservableIoError,
    P21ObservableEffect, P21ObservableEffectGuard, SourceId, SourceText,
};

fn plan(text: &str) -> nordoi_kernel::P21ObservableOutputPlan {
    let source = SourceText::new(
        SourceId::new(1),
        "secure.noi",
        format!("module secure.main; effect ConsoleWrite; entry main emits \"{text}\";"),
    )
    .unwrap();
    compile_observable_output_plan_p21(&source).unwrap()
}

#[test]
fn console_write_effect_maps_to_exact_console_write_capability() {
    assert_eq!(
        required_observable_capability_p21(P21ObservableEffect::ConsoleWrite),
        Capability::ConsoleWrite
    );
}

#[test]
fn capability_without_declared_p21_effect_is_denied() {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let guard = P21ObservableEffectGuard::new(false, authority);
    let error = guard.check(P21ObservableEffect::ConsoleWrite).unwrap_err();
    assert!(matches!(error, ObservableIoError::Policy { .. }));
    assert!(error.to_string().contains("was not declared"));
}

#[test]
fn declared_effect_without_capability_is_denied() {
    let guard = P21ObservableEffectGuard::new(true, CapabilitySet::new());
    assert!(matches!(
        guard.check(P21ObservableEffect::ConsoleWrite),
        Err(ObservableIoError::CapabilityDenied(
            Capability::ConsoleWrite
        ))
    ));
}

#[test]
fn unrelated_capability_cannot_authorize_console_output() {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::Gpu);
    let guard = P21ObservableEffectGuard::new(true, authority);
    assert!(matches!(
        guard.check(P21ObservableEffect::ConsoleWrite),
        Err(ObservableIoError::CapabilityDenied(
            Capability::ConsoleWrite
        ))
    ));
}

#[test]
fn exact_console_capability_authorizes_only_after_declaration() {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let guard = P21ObservableEffectGuard::new(true, authority);
    assert!(guard.check(P21ObservableEffect::ConsoleWrite).is_ok());
}

#[test]
fn revoked_console_capability_fails_closed() {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    authority.revoke(&Capability::ConsoleWrite);
    let error = execute_observable_output_p21(&plan("Hello"), &authority).unwrap_err();
    assert!(matches!(
        error,
        ObservableIoError::CapabilityDenied(Capability::ConsoleWrite)
    ));
}

#[test]
fn source_file_name_and_host_context_do_not_change_receipt_identity() {
    let a = SourceText::new(
        SourceId::new(1),
        "/tmp/a/main.noi",
        "module app.main; effect ConsoleWrite; entry main emits \"Same\";",
    )
    .unwrap();
    let b = SourceText::new(
        SourceId::new(999),
        "C:\\host\\different.noi",
        "module app.main; effect ConsoleWrite; entry main emits \"Same\";",
    )
    .unwrap();
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let ra =
        execute_observable_output_p21(&compile_observable_output_plan_p21(&a).unwrap(), &authority)
            .unwrap();
    let rb =
        execute_observable_output_p21(&compile_observable_output_plan_p21(&b).unwrap(), &authority)
            .unwrap();
    assert_eq!(ra.receipt_sha256(), rb.receipt_sha256());
    assert_eq!(ra.canonical_receipt_bytes(), rb.canonical_receipt_bytes());
}

#[test]
fn changing_observable_output_changes_plan_and_receipt_identity() {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let a_plan = plan("A");
    let b_plan = plan("B");
    let a = execute_observable_output_p21(&a_plan, &authority).unwrap();
    let b = execute_observable_output_p21(&b_plan, &authority).unwrap();
    assert_ne!(a_plan.plan_sha256(), b_plan.plan_sha256());
    assert_ne!(a.receipt_sha256(), b.receipt_sha256());
}
