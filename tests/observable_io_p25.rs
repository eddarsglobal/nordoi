use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nordoi_kernel::{
    authorize_file_output_p25, compile_file_output_plan_p25, materialize_file_output_p25,
    Capability, CapabilitySet, SourceId, SourceText, MAX_P25_FILE_NAME_BYTES,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(1), "p25.noi", text).unwrap()
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nordoi-p25-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn static_file_output_compiles_to_one_bounded_plan() {
    let src = source(
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"hello\\n\";",
    );
    let plan = compile_file_output_plan_p25(&src).unwrap();
    assert_eq!(plan.module(), Some("app.main"));
    assert_eq!(plan.entry_name(), "main");
    assert_eq!(plan.file_name(), "report.txt");
    assert_eq!(plan.output(), "hello\n");
}

#[test]
fn file_output_requires_explicit_file_write_effect() {
    let src = source("module app.main; entry main writes \"report.txt\" emits \"hello\";");
    let error = compile_file_output_plan_p25(&src).unwrap_err();
    assert!(error.to_string().contains("effect FileWrite"));
}

#[test]
fn console_write_effect_cannot_substitute_for_file_write() {
    let src = source(
        "module app.main; effect ConsoleWrite; entry main writes \"report.txt\" emits \"hello\";",
    );
    let error = compile_file_output_plan_p25(&src).unwrap_err();
    assert!(error.to_string().contains("permits only effect FileWrite"));
}

#[test]
fn path_separators_and_traversal_are_rejected_before_authority() {
    for target in ["../escape.txt", "nested/report.txt", "nested\\\\report.txt"] {
        let text = format!(
            "module app.main; effect FileWrite; entry main writes \"{target}\" emits \"hello\";"
        );
        let error = compile_file_output_plan_p25(&source(&text)).unwrap_err();
        assert!(error.to_string().contains("path separators"));
    }
}

#[test]
fn file_name_length_is_bounded_before_authority() {
    let target = format!("{}.txt", "a".repeat(MAX_P25_FILE_NAME_BYTES));
    let text = format!(
        "module app.main; effect FileWrite; entry main writes \"{target}\" emits \"hello\";"
    );
    let error = compile_file_output_plan_p25(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("output file name"));
    assert!(error.to_string().contains("exceeding bound"));
}

#[test]
fn file_output_bytes_are_bounded_before_authority() {
    let output = "x".repeat(4097);
    let text = format!(
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"{output}\";"
    );
    let error = compile_file_output_plan_p25(&source(&text)).unwrap_err();
    assert!(error.to_string().contains("exceeding bound 4096"));
}

#[test]
fn exact_target_capability_authorizes_without_materializing() {
    let src = source(
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"hello\";",
    );
    let plan = compile_file_output_plan_p25(&src).unwrap();
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("report.txt".to_owned()));
    let authorized = authorize_file_output_p25(&plan, &authority).unwrap();
    assert_eq!(authorized.plan().file_name(), "report.txt");
}

#[test]
fn authorized_file_output_materializes_exact_bytes_and_receipt() {
    let src = source(
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"hello from P2.5\\n\";",
    );
    let plan = compile_file_output_plan_p25(&src).unwrap();
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::FileWrite("report.txt".to_owned()));
    let authorized = authorize_file_output_p25(&plan, &authority).unwrap();
    let dir = temp_dir("materialize");
    let receipt = materialize_file_output_p25(&authorized, &dir).unwrap();
    assert_eq!(
        fs::read(dir.join("report.txt")).unwrap(),
        b"hello from P2.5\n"
    );
    assert_eq!(receipt.file_name(), "report.txt");
    assert_eq!(receipt.output(), "hello from P2.5\n");
    assert!(receipt.render_text().contains("status=WRITTEN"));
    assert!(receipt.render_text().contains("overwrite=DENIED"));
    let _ = fs::remove_dir_all(dir);
}
