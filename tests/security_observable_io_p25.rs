use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    authorize_file_output_p25, compile_file_output_plan_p25, materialize_file_output_p25,
    Capability, CapabilitySet, SourceId, SourceText,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn plan(name: &str) -> nordoi_kernel::P25FileOutputPlan {
    let src = source(
        1,
        name,
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"stable payload\";",
    );
    compile_file_output_plan_p25(&src).unwrap()
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nordoi-p25-sec-{label}-{}-{nonce}",
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
fn missing_file_grant_is_denied_before_materialization() {
    let plan = plan("denied.noi");
    let error = authorize_file_output_p25(&plan, &CapabilitySet::new()).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
    assert!(error.to_string().contains("FileWrite"));
}

#[test]
fn unrelated_capability_cannot_authorize_file_output() {
    let plan = plan("unrelated.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    assert!(authorize_file_output_p25(&plan, &authority).is_err());
}

#[test]
fn file_grant_for_different_target_cannot_authorize() {
    let plan = plan("wrong-target.noi");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("other.txt".to_owned()));
    assert!(authorize_file_output_p25(&plan, &authority).is_err());
}

#[test]
fn revoked_exact_file_grant_fails_closed() {
    let plan = plan("revoked.noi");
    let capability = Capability::FileWrite("report.txt".to_owned());
    let mut authority = CapabilitySet::new();
    authority.allow(capability.clone());
    authority.revoke(&capability);
    assert!(authorize_file_output_p25(&plan, &authority).is_err());
}

#[test]
fn host_source_name_does_not_change_p25_plan_identity() {
    let text = "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"stable payload\";";
    let first = compile_file_output_plan_p25(&source(1, "/tmp/a.noi", text)).unwrap();
    let second = compile_file_output_plan_p25(&source(99, "C:\\host\\b.noi", text)).unwrap();
    assert_eq!(first.plan_sha256_hex(), second.plan_sha256_hex());
    assert_eq!(first.canonical_plan_bytes(), second.canonical_plan_bytes());
}

#[test]
fn different_host_output_roots_produce_equal_canonical_receipts() {
    let plan = plan("roots.noi");
    let authority = exact_authority();
    let authorized = authorize_file_output_p25(&plan, &authority).unwrap();
    let first_dir = temp_dir("root-a");
    let second_dir = temp_dir("root-b");
    let first = materialize_file_output_p25(&authorized, &first_dir).unwrap();
    let second = materialize_file_output_p25(&authorized, &second_dir).unwrap();
    assert_eq!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_eq!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}

#[test]
fn existing_target_is_never_overwritten() {
    let plan = plan("no-overwrite.noi");
    let authority = exact_authority();
    let authorized = authorize_file_output_p25(&plan, &authority).unwrap();
    let dir = temp_dir("existing");
    fs::write(dir.join("report.txt"), b"original").unwrap();
    let error = materialize_file_output_p25(&authorized, &dir).unwrap_err();
    assert!(error.to_string().contains("cannot create new target"));
    assert_eq!(fs::read(dir.join("report.txt")).unwrap(), b"original");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn changing_target_name_changes_plan_identity_and_required_capability() {
    let first = source(
        1,
        "one.noi",
        "module app.main; effect FileWrite; entry main writes \"a.txt\" emits \"same\";",
    );
    let second = source(
        2,
        "two.noi",
        "module app.main; effect FileWrite; entry main writes \"b.txt\" emits \"same\";",
    );
    let first = compile_file_output_plan_p25(&first).unwrap();
    let second = compile_file_output_plan_p25(&second).unwrap();
    assert_ne!(first.plan_sha256_hex(), second.plan_sha256_hex());
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("a.txt".to_owned()));
    assert!(authorize_file_output_p25(&first, &authority).is_ok());
    assert!(authorize_file_output_p25(&second, &authority).is_err());
}
