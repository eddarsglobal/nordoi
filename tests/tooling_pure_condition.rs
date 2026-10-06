use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l09_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_l09_condition_without_claiming_execution() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi condition <path|->"));
    assert!(stdout.contains("Print the L0.9 pure boolean/comparison semantic boundary."));
    assert!(stdout.contains("without planning, NAIR, runtime work, storage, effects, or authority"));
}

#[test]
fn condition_reports_boolean_literal() {
    let path = source_file("true", "entry main returns true;");
    let output = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=BOOL(true) value=BOOL(true)"));
    assert!(stdout.contains("effects=0 authority=NONE l09="));
    assert!(stdout.contains(
        "planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ));
}

#[test]
fn condition_reports_integer_comparison() {
    let path = source_file(
        "compare",
        "module demo; effect Net; entry main returns 20 <= 22;",
    );
    let output = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("condition module=\"demo\" form=ENTRY entry=\"main\" pure=true"));
    assert!(stdout.contains("kind=INT_COMPARE(20<=22) value=BOOL(true) effects=0 authority=NONE"));
}

#[test]
fn false_comparison_is_reported_as_false_not_none() {
    let path = source_file("false", "entry main returns 9 < 3;");
    let output = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("value=BOOL(false)"));
}

#[test]
fn stdin_condition_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["condition", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"entry main returns 2 >= 2;")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("value=BOOL(true)"));
}

#[test]
fn invalid_condition_returns_frontend_exit_code() {
    let path = source_file("invalid", "entry main returns 1 = 1;");
    let output = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[condition]:"));
    assert!(stderr.contains("comparator"));
}

#[test]
fn old_expr_command_still_rejects_boolean_literal() {
    let path = source_file("exprfrozen", "entry main returns true;");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn old_bindings_pipeline_still_rejects_comparison() {
    let path = source_file("bindingsfrozen", "entry main returns 1 < 2;");
    for command in [
        "bindings",
        "bindings-plan",
        "bindings-lower",
        "bindings-run",
    ] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
}

#[test]
fn repeated_condition_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 4 != 5;");
    let a = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    let b = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(a.status.success() && b.status.success());
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn certified_version_output_remains_unchanged() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
