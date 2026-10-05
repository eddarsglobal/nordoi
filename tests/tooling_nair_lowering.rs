use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c04_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c04_lower_command_without_claiming_runtime_execution() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi lower <path|->"));
    assert!(stdout.contains("Lower the C0.3 validated zero-work plan to NAIR 0.6."));
    assert!(stdout.contains("C0.4 lowering does not execute the runtime."));
}

#[test]
fn lower_command_reports_single_halt_for_entry() {
    let path = source_file("entry", "module demo; type A; effect Net; entry main;");
    let output = Command::new(nordoi())
        .args(["lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "lower module=\"demo\" form=ENTRY entry=\"main\" work=0 effects=0 authority=NONE nair-instructions=1"
    ));
    assert!(stdout.contains("c03="));
    assert!(stdout.contains("c04="));
    assert!(stdout.contains("nair version=0.6 instructions=[HALT] bytes="));
    assert!(stdout.contains("runtime=NOT_INVOKED"));
}

#[test]
fn lower_command_reports_single_halt_for_empty_body() {
    let path = source_file("empty", "module demo; // tail\n");
    let output = Command::new(nordoi())
        .args(["lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=EMPTY work=0 effects=0 authority=NONE nair-instructions=1"));
}

#[test]
fn lower_command_rejects_unknown_body_with_frontend_exit_code() {
    let path = source_file("bad", "future_body");
    let output = Command::new(nordoi())
        .args(["lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[lower]:"));
}

#[test]
fn certified_c02_version_output_remains_unchanged() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
