use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    compile_dynamic_file_output_plan_p26, execute_dynamic_file_output_p26, Capability,
    CapabilitySet, DynamicValueKind, SourceId, SourceText,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "p26.noi", text).unwrap()
}

fn canonical() -> &'static str {
    "module app.main; effect FileWrite; input key_code; const bias = 1; entry main writes \"report.txt\" emits \"input=\" + key_code + \", next=\" + (key_code + bias) + \", accepted=\" + (key_code >= 40);"
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nordoi-p26-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).unwrap();
    path
}

fn exact_authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("report.txt".to_owned()));
    authority
}

#[test]
fn dynamic_file_output_compiles_to_p24_renderer_and_p25_target_policy() {
    let plan = compile_dynamic_file_output_plan_p26(&source(canonical())).unwrap();
    assert_eq!(plan.module(), Some("app.main"));
    assert_eq!(plan.entry_name(), "main");
    assert_eq!(plan.input_name(), "key_code");
    assert_eq!(plan.file_name(), "report.txt");
    assert_eq!(plan.runtime_segment_count(), 3);
    assert_eq!(
        plan.result_kinds(),
        vec![
            DynamicValueKind::Int,
            DynamicValueKind::Int,
            DynamicValueKind::Bool
        ]
    );
}

#[test]
fn dynamic_file_output_requires_explicit_file_write_effect() {
    let text = canonical().replace("effect FileWrite; ", "");
    let error = compile_dynamic_file_output_plan_p26(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("effect FileWrite"));
}

#[test]
fn console_write_cannot_substitute_for_file_write() {
    let text = canonical().replace("effect FileWrite", "effect ConsoleWrite");
    let error = compile_dynamic_file_output_plan_p26(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("permits only effect FileWrite"));
}

#[test]
fn dynamic_file_target_keeps_p25_traversal_policy() {
    let text = canonical().replace("report.txt", "../escape.txt");
    let error = compile_dynamic_file_output_plan_p26(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("path separators"));
}

#[test]
fn one_runtime_segment_is_rejected_to_preserve_p24_boundary() {
    let text = "module app.main; effect FileWrite; input key_code; entry main writes \"report.txt\" emits \"value=\" + key_code;";
    let error = compile_dynamic_file_output_plan_p26(&source(text)).unwrap_err();
    assert!(error.to_string().contains("at least two runtime segments"));
}

#[test]
fn more_than_eight_runtime_segments_are_rejected_before_authority() {
    let text = "module app.main; effect FileWrite; input key_code; entry main writes \"report.txt\" emits \"a=\" + key_code + \",b=\" + key_code + \",c=\" + key_code + \",d=\" + key_code + \",e=\" + key_code + \",f=\" + key_code + \",g=\" + key_code + \",h=\" + key_code + \",i=\" + key_code;";
    let error = compile_dynamic_file_output_plan_p26(&source(text)).unwrap_err();
    assert!(error.to_string().contains("at most 8 runtime segments"));
}

#[test]
fn explicit_file_grant_materializes_dynamic_multi_segment_bytes() {
    let plan = compile_dynamic_file_output_plan_p26(&source(canonical())).unwrap();
    let dir = temp_dir("write-40");
    let receipt = execute_dynamic_file_output_p26(&plan, 40, &exact_authority(), &dir).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join("report.txt")).unwrap(),
        "input=40, next=41, accepted=true"
    );
    assert_eq!(receipt.output(), "input=40, next=41, accepted=true");
    assert!(receipt.render_text().contains("segments=3"));
    assert!(receipt.render_text().contains("status=WRITTEN"));
    assert!(!receipt.render_text().contains("ConsoleWrite"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn runtime_input_changes_dynamic_file_bytes_and_receipt() {
    let plan = compile_dynamic_file_output_plan_p26(&source(canonical())).unwrap();
    let first_dir = temp_dir("input-40");
    let second_dir = temp_dir("input-41");
    let first = execute_dynamic_file_output_p26(&plan, 40, &exact_authority(), &first_dir).unwrap();
    let second =
        execute_dynamic_file_output_p26(&plan, 41, &exact_authority(), &second_dir).unwrap();
    assert_eq!(first.output(), "input=40, next=41, accepted=true");
    assert_eq!(second.output(), "input=41, next=42, accepted=true");
    assert_ne!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_ne!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}
