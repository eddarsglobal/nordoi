use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l07_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_l07_expr_command_without_claiming_execution() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi expr <path|->"));
    assert!(stdout.contains("Print the L0.7 pure-expression boundary."));
    assert!(stdout.contains(
        "L0.7 expr is additive: it evaluates only pure checked integer addition and does not plan, lower, or execute runtime work."
    ));
}

#[test]
fn expr_command_reports_calculated_value_and_postfix_ops() {
    let path = source_file(
        "add",
        "module demo; type A; effect Net; entry main returns 20 + 22;",
    );
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "expr module=\"demo\" form=ENTRY name=\"main\" pure=true ops=[INT(20),INT(22),ADD] value=INT(42) nodes=3 effects=0"
    ));
    assert!(stdout.contains("c02="));
    assert!(stdout.contains("l07="));
    assert!(stdout.contains("expression="));
    assert!(
        stdout.contains("planning=NOT_DEFINED nair=UNCHANGED runtime=NOT_INVOKED authority=NONE")
    );
}

#[test]
fn expr_command_reports_parenthesized_postfix_order() {
    let path = source_file("group", "entry main returns 1 + (2 + 3);");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(1),INT(2),INT(3),ADD,ADD]"));
    assert!(stdout.contains("value=INT(6)"));
}

#[test]
fn expr_command_supports_l06_literal_as_one_node_expression() {
    let path = source_file("literal", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(42)] value=INT(42) nodes=1"));
}

#[test]
fn expr_command_supports_l05_entry_as_no_expression() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=ENTRY name=\"main\" pure=true ops=NONE value=NONE nodes=0 effects=0"));
}

#[test]
fn overflow_returns_frontend_exit_code() {
    let path = source_file("overflow", "entry main returns 9223372036854775807 + 1;");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[expr]:"));
    assert!(stderr.contains("overflowed"));
}

#[test]
fn unsupported_operator_returns_frontend_exit_code() {
    let path = source_file("multiply", "entry main returns 6 * 7;");
    let output = Command::new(nordoi())
        .args(["expr", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[expr]:"));
    assert!(stderr.contains("only '+'"));
}

#[test]
fn certified_l06_result_command_still_rejects_l07_addition() {
    let path = source_file("l06", "entry main returns 20 + 22;");
    let output = Command::new(nordoi())
        .args(["result", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[result]:"));
}

#[test]
fn certified_result_plan_lower_run_boundaries_still_reject_l07_addition() {
    let path = source_file("frozen", "entry main returns 20 + 22;");
    for command in ["result-plan", "result-lower", "result-run"] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
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
