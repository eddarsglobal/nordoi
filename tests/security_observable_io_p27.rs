use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    compile_atomic_bundle_plan_p27, execute_atomic_bundle_p27, Capability, CapabilitySet, SourceId,
    SourceText,
};

fn source(id: u32, name: &str, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), name, text).unwrap()
}

fn text(bundle: &str, first: &str, second: &str) -> String {
    format!(
        "module app.main; effect FileWrite; input key_code; entry main writes \"{bundle}\" {{ \"{first}\" emits \"a=\" + key_code + \",b=\" + key_code; \"{second}\" emits \"c=\" + key_code + \",d=\" + (key_code + 1); }};"
    )
}

fn plan(name: &str) -> nordoi_kernel::P27AtomicBundlePlan {
    compile_atomic_bundle_plan_p27(&source(1, name, &text("bundle", "a.txt", "b.txt"))).unwrap()
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nordoi-p27-sec-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn exact_authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("bundle/a.txt".to_owned()));
    authority.allow(Capability::FileWrite("bundle/b.txt".to_owned()));
    authority
}

#[test]
fn missing_one_exact_grant_fails_before_any_bundle_publication() {
    let plan = plan("missing.noi");
    let dir = temp_dir("missing");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("bundle/a.txt".to_owned()));
    let error = execute_atomic_bundle_p27(&plan, 40, &authority, &dir).unwrap_err();
    assert!(error.to_string().contains("capability denied"));
    assert!(!dir.join("bundle").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn console_write_authority_cannot_authorize_bundle_output() {
    let plan = plan("console.noi");
    let dir = temp_dir("console");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::ConsoleWrite);
    assert!(execute_atomic_bundle_p27(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("bundle").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn grants_for_another_bundle_cannot_authorize() {
    let plan = plan("other.noi");
    let dir = temp_dir("other");
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("other/a.txt".to_owned()));
    authority.allow(Capability::FileWrite("other/b.txt".to_owned()));
    assert!(execute_atomic_bundle_p27(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("bundle").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn revoked_exact_grant_fails_closed() {
    let plan = plan("revoked.noi");
    let dir = temp_dir("revoked");
    let capability = Capability::FileWrite("bundle/b.txt".to_owned());
    let mut authority = exact_authority();
    authority.revoke(&capability);
    assert!(execute_atomic_bundle_p27(&plan, 40, &authority, &dir).is_err());
    assert!(!dir.join("bundle").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn existing_bundle_is_never_overwritten() {
    let plan = plan("existing.noi");
    let dir = temp_dir("existing");
    fs::create_dir(dir.join("bundle")).unwrap();
    fs::write(dir.join("bundle/original.txt"), b"original").unwrap();
    let error = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &dir).unwrap_err();
    assert!(error.to_string().contains("target already exists"));
    assert_eq!(
        fs::read(dir.join("bundle/original.txt")).unwrap(),
        b"original"
    );
    assert!(!dir.join("bundle/a.txt").exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn different_host_roots_produce_equal_bundle_receipts() {
    let ordered = text("bundle", "a.txt", "b.txt");
    let reordered = "module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" { \"b.txt\" emits \"c=\" + key_code + \",d=\" + (key_code + 1); \"a.txt\" emits \"a=\" + key_code + \",b=\" + key_code; };";
    let first_plan =
        compile_atomic_bundle_plan_p27(&source(1, "/tmp/ordered.noi", &ordered)).unwrap();
    let second_plan =
        compile_atomic_bundle_plan_p27(&source(99, "C:\\host\\reordered.noi", reordered)).unwrap();
    assert_eq!(
        first_plan.canonical_plan_bytes(),
        second_plan.canonical_plan_bytes()
    );
    assert_eq!(first_plan.plan_sha256_hex(), second_plan.plan_sha256_hex());

    let plan = first_plan;
    let first_dir = temp_dir("root-a");
    let second_dir = temp_dir("root-b");
    let first = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &first_dir).unwrap();
    let second = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &second_dir).unwrap();
    assert_eq!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_eq!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}

#[test]
fn bundle_and_file_traversal_are_rejected_before_authority() {
    let bundle_error = compile_atomic_bundle_plan_p27(&source(
        1,
        "bundle-escape.noi",
        &text("../escape", "a.txt", "b.txt"),
    ))
    .unwrap_err();
    assert!(bundle_error
        .to_string()
        .contains("bundle name must not contain path separators"));

    let file_error = compile_atomic_bundle_plan_p27(&source(
        2,
        "file-escape.noi",
        &text("bundle", "../a.txt", "b.txt"),
    ))
    .unwrap_err();
    assert!(file_error.to_string().contains("path separators"));
}

#[test]
fn exhausted_private_staging_names_publish_no_final_bundle() {
    let plan = plan("staging.noi");
    let dir = temp_dir("staging");
    let hash = plan.plan_sha256_hex();
    let prefix = &hash[..16];
    for attempt in 0..64u8 {
        fs::create_dir(dir.join(format!(".nordoi-p27-stage-{prefix}-{attempt:02x}"))).unwrap();
    }
    let error = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &dir).unwrap_err();
    assert!(error.to_string().contains("after 64 attempts"));
    assert!(!dir.join("bundle").exists());
    let _ = fs::remove_dir_all(dir);
}
