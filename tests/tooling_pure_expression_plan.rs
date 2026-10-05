use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c07_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c07_expr_plan_command_without_claiming_lowering() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi expr-plan <path|->"));
    assert!(stdout.contains("Print the C0.7 pure-expression execution plan."));
    assert!(stdout.contains(
        "C0.7 expr-plan is additive: it preserves L0.7 postfix calculation order without lowering NAIR or executing runtime work."
    ));
}

#[test]
fn expr_plan_reports_postfix_value_and_zero_work() {
    let path = source_file(
        "add",
        "module demo; type A; effect Net; entry main returns 20 + 22;",
    );
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "expr-plan module=\"demo\" form=ENTRY entry=\"main\" ops=[INT(20),INT(22),ADD] value=INT(42) nodes=3 work=0 effects=0 authority=NONE"
    ));
    assert!(stdout.contains("l07="));
    assert!(stdout.contains("c07="));
    assert!(stdout.contains("lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
}

#[test]
fn expr_plan_preserves_parenthesized_postfix_order() {
    let path = source_file("group", "entry main returns 1 + (2 + 3);");
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(1),INT(2),INT(3),ADD,ADD]"));
    assert!(stdout.contains("value=INT(6)"));
    assert!(stdout.contains("nodes=5 work=0 effects=0 authority=NONE"));
}

#[test]
fn expr_plan_supports_literal_result_without_rewriting_l06() {
    let path = source_file("literal", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(42)] value=INT(42) nodes=1 work=0"));
}

#[test]
fn expr_plan_supports_entry_without_expression() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains(
        "form=ENTRY entry=\"main\" ops=NONE value=NONE nodes=0 work=0 effects=0 authority=NONE"
    ));
}

#[test]
fn expr_plan_supports_empty_body() {
    let path = source_file("empty", "");
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("form=EMPTY ops=NONE value=NONE nodes=0 work=0 effects=0 authority=NONE"));
}

#[test]
fn overflow_returns_frontend_exit_code_from_expr_plan() {
    let path = source_file("overflow", "entry main returns 9223372036854775807 + 1;");
    let output = Command::new(nordoi())
        .args(["expr-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[expr-plan]:"));
    assert!(stderr.contains("overflowed"));
}

#[test]
fn certified_l06_c05_c06_v02_boundaries_still_reject_l07_addition() {
    let path = source_file("frozen", "entry main returns 20 + 22;");
    for command in ["result", "result-plan", "result-lower", "result-run"] {
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
