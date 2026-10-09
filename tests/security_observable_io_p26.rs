use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    compile_dynamic_file_output_plan_p26, execute_dynamic_file_output_p26, Capability,
    CapabilitySet, SourceId, SourceText,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn text(target: &str) -> String {
    format!(
        "module app.main; effect FileWrite; input key_code; entry main writes \"{target}\" emits \"value=\" + key_code + \", accepted=\" + (key_code >= 40);"
    )
}

fn plan(name: &str) -> nordoi_kernel::P26DynamicFileOutputPlan {
    compile_dynamic_file_output_plan_p26(&source(1, name, &text("report.txt"))).unwrap()
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nordoi-p26-sec-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn exact_authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("report.txt".to_owned()));
    authority
}

#[test]
fn missing_file_grant_fails_before_dynamic_file_materialization() {
    let plan = plan("denied.noi");
    let dir = temp_dir("denied");
    let error =
        execute_dynamic_file_output_p26(&plan, 40, &CapabilitySet::new(), &dir).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
    assert!(!dir.join("report.txt").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn console_write_authority_cannot_authorize_dynamic_file_output() {
    let plan = plan("console.noi");
    let dir = temp_dir("console");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    assert!(execute_dynamic_file_output_p26(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("report.txt").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn file_grant_for_another_target_cannot_authorize() {
    let plan = plan("other.noi");
    let dir = temp_dir("other");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("other.txt".to_owned()));
    assert!(execute_dynamic_file_output_p26(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("report.txt").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn revoked_exact_file_grant_fails_closed() {
    let plan = plan("revoked.noi");
    let dir = temp_dir("revoked");
    let capability = Capability::FileWrite("report.txt".to_owned());
    let mut authority = CapabilitySet::new();
    authority.allow(capability.clone());
    authority.revoke(&capability);
    assert!(execute_dynamic_file_output_p26(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("report.txt").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn host_source_identity_does_not_change_p26_plan_identity() {
    let body = text("report.txt");
    let first = compile_dynamic_file_output_plan_p26(&source(1, "/tmp/a.noi", &body)).unwrap();
    let second =
        compile_dynamic_file_output_plan_p26(&source(99, "C:\\host\\b.noi", &body)).unwrap();
    assert_eq!(first.plan_sha256_hex(), second.plan_sha256_hex());
    assert_eq!(first.canonical_plan_bytes(), second.canonical_plan_bytes());
}

#[test]
fn different_host_roots_produce_equal_p26_receipts() {
    let plan = plan("roots.noi");
    let first_dir = temp_dir("root-a");
    let second_dir = temp_dir("root-b");
    let first = execute_dynamic_file_output_p26(&plan, 40, &exact_authority(), &first_dir).unwrap();
    let second =
        execute_dynamic_file_output_p26(&plan, 40, &exact_authority(), &second_dir).unwrap();
    assert_eq!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_eq!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}

#[test]
fn existing_target_is_not_overwritten_by_dynamic_file_output() {
    let plan = plan("existing.noi");
    let dir = temp_dir("existing");
    fs::write(dir.join("report.txt"), b"original").unwrap();
    let error = execute_dynamic_file_output_p26(&plan, 40, &exact_authority(), &dir).unwrap_err();
    assert!(error.to_string().contains("cannot create new target"));
    assert_eq!(fs::read(dir.join("report.txt")).unwrap(), b"original");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn changing_target_changes_plan_identity_and_required_capability() {
    let first = compile_dynamic_file_output_plan_p26(&source(1, "a.noi", &text("a.txt"))).unwrap();
    let second = compile_dynamic_file_output_plan_p26(&source(2, "b.noi", &text("b.txt"))).unwrap();
    assert_ne!(first.plan_sha256_hex(), second.plan_sha256_hex());
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("a.txt".to_owned()));
    let first_dir = temp_dir("target-a");
    let second_dir = temp_dir("target-b");
    assert!(execute_dynamic_file_output_p26(&first, 40, &authority, &first_dir).is_ok());
    assert!(execute_dynamic_file_output_p26(&second, 40, &authority, &second_dir).is_err());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}
