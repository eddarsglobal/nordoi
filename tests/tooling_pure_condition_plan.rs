use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(label: &str, source: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nordoi-c011-{label}-{}-{stamp}.noi",
        std::process::id()
    ));
    fs::write(&path, source).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c011_condition_plan_without_claiming_branching() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi condition-plan <path|->"));
    assert!(stdout.contains("C0.11 pure-condition execution plan"));
    assert!(stdout.contains("without branches, NAIR lowering, or runtime execution"));
}

#[test]
fn condition_plan_reports_true_boolean_zero_cost() {
    let path = source_file("true", "entry main returns true;");
    let output = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=BOOL(true) value=BOOL(true)"));
    assert!(stdout.contains("work=0 storage=0 effects=0 authority=NONE"));
    assert!(stdout.contains("c011="));
    assert!(stdout
        .contains("branching=UNDEFINED lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
}

#[test]
fn condition_plan_reports_false_boolean() {
    let path = source_file("false", "entry main returns false;");
    let output = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=BOOL(false) value=BOOL(false)"));
}

#[test]
fn condition_plan_reports_comparison_structure_and_truth() {
    let path = source_file(
        "compare",
        "module demo.condition; entry main returns 20 <= 22;",
    );
    let output = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("module=\"demo.condition\""));
    assert!(stdout.contains("kind=INT_COMPARE(20<=22) value=BOOL(true)"));
}

#[test]
fn condition_plan_supports_empty_and_plain_entry() {
    for (label, source, expected) in [
        ("empty", "module demo;", "form=EMPTY"),
        ("entry", "entry main;", "form=ENTRY entry=\"main\""),
    ] {
        let path = source_file(label, source);
        let output = Command::new(nordoi())
            .args(["condition-plan", path.to_str().unwrap()])
            .output()
            .unwrap();
        cleanup(&path);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(expected));
        assert!(stdout.contains("value=NONE"));
    }
}

#[test]
fn stdin_condition_plan_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["condition-plan", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"entry main returns 8 >= 8;")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=INT_COMPARE(8>=8) value=BOOL(true)"));
}

#[test]
fn invalid_condition_plan_fails_with_frontend_exit_code() {
    let path = source_file("invalid", "entry main returns 1 = 1;");
    let output = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[condition-plan]:"));
}

#[test]
fn l09_condition_command_remains_semantic_only() {
    let path = source_file("l09-frozen", "entry main returns 2 < 3;");
    let output = Command::new(nordoi())
        .args(["condition", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
}

#[test]
fn older_expression_and_binding_pipelines_still_reject_condition_syntax() {
    let path = source_file("frozen", "entry main returns 1 < 2;");
    for command in [
        "expr-plan",
        "expr-lower",
        "expr-run",
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
fn repeated_condition_plan_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 4 != 5;");
    let a = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    let b = Command::new(nordoi())
        .args(["condition-plan", path.to_str().unwrap()])
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
