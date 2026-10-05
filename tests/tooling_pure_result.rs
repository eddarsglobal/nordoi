use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l06_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_l06_result_command_without_claiming_execution() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi result <path|->"));
    assert!(stdout.contains("Print the L0.6 pure-result boundary."));
    assert!(stdout.contains(
        "L0.6 result is additive: it does not create a C0.3 plan, lower NAIR, or execute runtime work."
    ));
}

#[test]
fn result_command_reports_pure_integer_and_l06_witness() {
    let path = source_file(
        "int",
        "module demo; type A; effect Net; entry main returns 42;",
    );
    let output = Command::new(nordoi())
        .args(["result", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "result module=\"demo\" form=ENTRY name=\"main\" pure=true value=INT(42) effects=0"
    ));
    assert!(stdout.contains("c02="));
    assert!(stdout.contains("l06="));
    assert!(stdout.contains("result="));
    assert!(
        stdout.contains("execution=NOT_PLANNED nair=UNCHANGED runtime=NOT_INVOKED authority=NONE")
    );
}

#[test]
fn result_command_reports_l05_entry_as_no_result() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["result", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=ENTRY name=\"main\" pure=true value=NONE effects=0"));
}

#[test]
fn noncanonical_result_literal_returns_frontend_exit_code() {
    let path = source_file("bad", "entry main returns 01;");
    let output = Command::new(nordoi())
        .args(["result", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[result]:"));
    assert!(stderr.contains("canonical decimal"));
}

#[test]
fn certified_l05_body_command_still_rejects_l06_result_syntax() {
    let path = source_file("l05_compat", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["body", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[body]:"));
}

#[test]
fn c03_and_v01_do_not_silently_accept_l06_result_syntax() {
    let path = source_file("future", "entry main returns 42;");
    let plan = Command::new(nordoi())
        .args(["plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    let run = Command::new(nordoi())
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(plan.status.code(), Some(4));
    assert_eq!(run.status.code(), Some(4));
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
