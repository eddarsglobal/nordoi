use nordoi_kernel::{
    compile_observable_output_plan_p21, execute_observable_output_p21, Capability, CapabilitySet,
    ObservableIoError, SourceId, SourceText, MAX_P21_OUTPUT_BYTES, P21_CONSOLE_EFFECT_NAME,
    P21_OUTPUT_SCHEMA,
};

fn source(name: &str, text: impl Into<String>) -> SourceText {
    SourceText::new(SourceId::new(1), name, text).unwrap()
}

fn valid(text: &str) -> SourceText {
    source(
        "main.noi",
        format!("module app.main; effect ConsoleWrite; entry main emits \"{text}\";"),
    )
}

#[test]
fn explicit_console_effect_compiles_one_bounded_output_plan() {
    let plan = compile_observable_output_plan_p21(&valid("Hello")).unwrap();
    assert_eq!(plan.module(), Some("app.main"));
    assert_eq!(plan.entry_name(), "main");
    assert_eq!(plan.output(), "Hello");
    assert_eq!(plan.output_bytes(), 5);
    assert!(!plan.canonical_plan_bytes().is_empty());
    assert_eq!(plan.plan_sha256_hex().len(), 64);
}

#[test]
fn missing_console_effect_is_rejected() {
    let error = compile_observable_output_plan_p21(&source(
        "missing.noi",
        "module app.main; entry main emits \"Hello\";",
    ))
    .unwrap_err();
    assert!(matches!(error, ObservableIoError::Policy { .. }));
    assert!(error.to_string().contains("requires explicit"));
}

#[test]
fn unrelated_effect_is_rejected_fail_closed() {
    let error = compile_observable_output_plan_p21(&source(
        "network.noi",
        "module app.main; effect Network; entry main emits \"Hello\";",
    ))
    .unwrap_err();
    assert!(matches!(error, ObservableIoError::Policy { .. }));
    assert!(error.to_string().contains(P21_CONSOLE_EFFECT_NAME));
}

#[test]
fn duplicate_console_effect_is_rejected() {
    let error = compile_observable_output_plan_p21(&source(
        "duplicate.noi",
        "module app.main; effect ConsoleWrite; effect ConsoleWrite; entry main emits \"Hello\";",
    ))
    .unwrap_err();
    assert!(matches!(error, ObservableIoError::Policy { .. }));
    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn output_escapes_decode_deterministically() {
    let plan = compile_observable_output_plan_p21(&valid("A\\nB\\t\\\"C\\\"\\\\D")).unwrap();
    assert_eq!(plan.output(), "A\nB\t\"C\"\\D");
}

#[test]
fn unsupported_output_escape_is_rejected() {
    let error = compile_observable_output_plan_p21(&valid("bad\\qescape")).unwrap_err();
    assert!(matches!(error, ObservableIoError::Syntax { .. }));
    assert!(error.to_string().contains("unsupported output escape"));
}

#[test]
fn output_quota_is_enforced_before_authority() {
    let oversized = "a".repeat(MAX_P21_OUTPUT_BYTES + 1);
    let error = compile_observable_output_plan_p21(&valid(&oversized)).unwrap_err();
    assert!(matches!(error, ObservableIoError::Policy { .. }));
    assert!(error.to_string().contains("exceeding bound"));
}

#[test]
fn explicit_grant_produces_schema_versioned_deterministic_receipt() {
    let plan = compile_observable_output_plan_p21(&valid("Hello")).unwrap();
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    let first = execute_observable_output_p21(&plan, &authority).unwrap();
    let second = execute_observable_output_p21(&plan, &authority).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.output(), "Hello");
    assert_eq!(first.receipt_sha256_hex().len(), 64);
    let json = first.render_json();
    assert!(json.contains(P21_OUTPUT_SCHEMA));
    assert!(json.contains("\"authority\":\"EXPLICIT\""));
    assert!(json.contains("\"ambientAuthority\":\"NONE\""));
    assert!(execute_observable_output_p21(&plan, &CapabilitySet::new()).is_err());
    let denied = execute_observable_output_p21(&plan, &CapabilitySet::new()).unwrap_err();
    assert!(matches!(
        denied,
        ObservableIoError::CapabilityDenied(Capability::ConsoleWrite)
    ));
}
