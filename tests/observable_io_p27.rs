use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    compile_atomic_bundle_plan_p27, execute_atomic_bundle_p27, Capability, CapabilitySet, SourceId,
    SourceText, MAX_P27_FILES, MAX_P27_TOTAL_OUTPUT_BYTES,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "p27.noi", text).unwrap()
}

fn canonical() -> &'static str {
    "module app.main; effect FileWrite; input key_code; const bias = 1; entry main writes \"report-bundle\" { \"report.txt\" emits \"input=\" + key_code + \", next=\" + (key_code + bias); \"summary.txt\" emits \"accepted=\" + (key_code >= 40) + \", key=\" + key_code; };"
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nordoi-p27-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).unwrap();
    path
}

fn exact_authority() -> CapabilitySet {
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("report-bundle/report.txt".to_owned()));
    authority.allow(Capability::FileWrite(
        "report-bundle/summary.txt".to_owned(),
    ));
    authority
}

#[test]
fn atomic_bundle_compiles_two_canonical_targets() {
    let plan = compile_atomic_bundle_plan_p27(&source(canonical())).unwrap();
    assert_eq!(plan.module(), Some("app.main"));
    assert_eq!(plan.entry_name(), "main");
    assert_eq!(plan.input_name(), "key_code");
    assert_eq!(plan.bundle_name(), "report-bundle");
    assert_eq!(plan.file_count(), 2);
    assert_eq!(plan.files()[0].file_name(), "report.txt");
    assert_eq!(plan.files()[1].file_name(), "summary.txt");
    assert!(plan.maximum_total_output_bytes() <= MAX_P27_TOTAL_OUTPUT_BYTES);
}

#[test]
fn atomic_bundle_requires_explicit_file_write_effect() {
    let text = canonical().replace("effect FileWrite; ", "");
    let error = compile_atomic_bundle_plan_p27(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("effect FileWrite"));
}

#[test]
fn atomic_bundle_requires_at_least_two_files() {
    let text = "module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" { \"one.txt\" emits \"a=\" + key_code + \",b=\" + key_code; };";
    let error = compile_atomic_bundle_plan_p27(&source(text)).unwrap_err();
    assert!(error.to_string().contains("at least 2 files"));
}

#[test]
fn atomic_bundle_rejects_more_than_eight_files() {
    let mut items = String::new();
    for index in 0..=MAX_P27_FILES {
        items.push_str(&format!(
            "\"f{index}.txt\" emits \"a=\" + key_code + \",b=\" + key_code; "
        ));
    }
    let text = format!(
        "module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" {{ {items}}};"
    );
    let error = compile_atomic_bundle_plan_p27(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("at most 8 files"));
}

#[test]
fn duplicate_bundle_targets_are_rejected() {
    let text = "module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" { \"same.txt\" emits \"a=\" + key_code + \",b=\" + key_code; \"same.txt\" emits \"c=\" + key_code + \",d=\" + key_code; };";
    let error = compile_atomic_bundle_plan_p27(&source(text)).unwrap_err();
    assert!(error.to_string().contains("duplicate bundle target"));
}

#[test]
fn global_bundle_quota_is_proved_before_authority() {
    let large = "x".repeat(3400);
    let mut items = String::new();
    for index in 0..5 {
        items.push_str(&format!(
            "\"f{index}.txt\" emits \"{large}\" + key_code + \",\" + key_code; "
        ));
    }
    let text = format!(
        "module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" {{ {items}}};"
    );
    let error = compile_atomic_bundle_plan_p27(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("exceeding global bound"));
}

#[test]
fn exact_grants_publish_one_complete_bundle() {
    let plan = compile_atomic_bundle_plan_p27(&source(canonical())).unwrap();
    let dir = temp_dir("publish");
    let receipt = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &dir).unwrap();
    let bundle = dir.join("report-bundle");
    assert_eq!(
        fs::read_to_string(bundle.join("report.txt")).unwrap(),
        "input=40, next=41"
    );
    assert_eq!(
        fs::read_to_string(bundle.join("summary.txt")).unwrap(),
        "accepted=true, key=40"
    );
    assert_eq!(receipt.files().len(), 2);
    assert!(receipt.render_text().contains("status=COMMITTED"));
    assert!(receipt
        .render_text()
        .contains("partial-final-state=FORBIDDEN"));
    assert!(receipt.render_text().contains("crash-durability=UNCLAIMED"));
    assert!(!receipt.render_text().contains("ConsoleWrite"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn runtime_input_changes_bundle_bytes_and_receipt() {
    let plan = compile_atomic_bundle_plan_p27(&source(canonical())).unwrap();
    let first_dir = temp_dir("input-40");
    let second_dir = temp_dir("input-41");
    let first = execute_atomic_bundle_p27(&plan, 40, &exact_authority(), &first_dir).unwrap();
    let second = execute_atomic_bundle_p27(&plan, 41, &exact_authority(), &second_dir).unwrap();
    assert_eq!(
        fs::read_to_string(first_dir.join("report-bundle/report.txt")).unwrap(),
        "input=40, next=41"
    );
    assert_eq!(
        fs::read_to_string(second_dir.join("report-bundle/report.txt")).unwrap(),
        "input=41, next=42"
    );
    assert_ne!(
        first.canonical_receipt_bytes(),
        second.canonical_receipt_bytes()
    );
    assert_ne!(first.receipt_sha256_hex(), second.receipt_sha256_hex());
    let _ = fs::remove_dir_all(first_dir);
    let _ = fs::remove_dir_all(second_dir);
}
