use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l05_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_l05_body_command() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi body <path|->"));
    assert!(stdout.contains("L0.5 minimal body boundary"));
}

#[test]
fn body_command_reports_pure_entry_and_l05_witness() {
    let path = source_file("entry", "module demo; type A; effect Net; entry main;");
    let output = Command::new(nordoi())
        .args(["body", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("body module=\"demo\" form=ENTRY name=\"main\" pure=true effects=0"));
    assert!(stdout.contains("c02="));
    assert!(stdout.contains("l05="));
    assert!(stdout.contains("entry="));
}

#[test]
fn body_command_reports_empty_body_explicitly() {
    let path = source_file("empty", "module demo; type A; //tail\n");
    let output = Command::new(nordoi())
        .args(["body", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=EMPTY pure=true"));
}

#[test]
fn body_command_rejects_unsupported_body_with_frontend_exit_code() {
    let path = source_file("unsupported", "future_body");
    let output = Command::new(nordoi())
        .args(["body", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[body]:"));
    assert!(stderr.contains("must be empty or begin with contextual 'entry'"));
}

#[test]
fn semantic_command_remains_c02_and_reports_unlowered_for_entry_source() {
    let path = source_file("compat", "entry main;");
    let output = Command::new(nordoi())
        .args(["semantic", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("body=UNLOWERED"));
}
