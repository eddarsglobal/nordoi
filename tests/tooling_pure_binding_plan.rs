use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c09_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c09_bindings_plan_command_without_claiming_lowering() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi bindings-plan <path|->"));
    assert!(stdout.contains("Print the C0.9 pure-binding execution plan."));
    assert!(stdout.contains(
        "C0.9 bindings-plan preserves canonical binding identities and postfix references with zero runtime storage, without NAIR lowering or runtime execution."
    ));
}

#[test]
fn bindings_plan_reports_registry_references_value_and_zero_storage() {
    let path = source_file(
        "main",
        "module demo; type A; effect Net; const y = 22; const x = 20; entry main returns x + y;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bindings-plan module=\"demo\" form=ENTRY entry=\"main\""));
    assert!(stdout.contains("bindings=[#1:x=INT(20),#2:y=INT(22)] count=2"));
    assert!(stdout.contains("ops=[BINDING(1),BINDING(2),ADD] value=INT(42) nodes=3"));
    assert!(stdout.contains("work=0 storage=0 effects=0 authority=NONE"));
    assert!(stdout.contains("l08="));
    assert!(stdout.contains("c09="));
    assert!(stdout.contains("lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
}

#[test]
fn declaration_order_produces_identical_bindings_plan_output() {
    let a = source_file(
        "order_a",
        "const x = 20; const y = 22; entry main returns x + y;",
    );
    let b = source_file(
        "order_b",
        "const y = 22; const x = 20; entry main returns x + y;",
    );
    let out_a = Command::new(nordoi())
        .args(["bindings-plan", a.to_str().unwrap()])
        .output()
        .unwrap();
    let out_b = Command::new(nordoi())
        .args(["bindings-plan", b.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&a);
    cleanup(&b);
    assert!(out_a.status.success());
    assert!(out_b.status.success());
    assert_eq!(out_a.stdout, out_b.stdout);
}

#[test]
fn bindings_plan_accepts_stdin() {
    let mut child = Command::new(nordoi())
        .args(["bindings-plan", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"const x = 20; entry main returns x + 22;")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("value=INT(42)"));
    assert!(stdout.contains("storage=0"));
}

#[test]
fn bindings_plan_supports_entry_without_expression() {
    let path = source_file("none", "const x = 20; entry main;");
    let output = Command::new(nordoi())
        .args(["bindings-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("form=ENTRY entry=\"main\""));
    assert!(
        stdout.contains("ops=NONE value=NONE nodes=0 work=0 storage=0 effects=0 authority=NONE")
    );
}

#[test]
fn unknown_binding_returns_frontend_exit_code() {
    let path = source_file("unknown", "entry main returns missing + 1;");
    let output = Command::new(nordoi())
        .args(["bindings-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[bindings-plan]:"));
    assert!(stderr.contains("undeclared pure binding 'missing'"));
}

#[test]
fn overflow_returns_frontend_exit_code() {
    let path = source_file(
        "overflow",
        "const max = 9223372036854775807; entry main returns max + 1;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("overflowed"));
}

#[test]
fn l08_bindings_command_remains_semantic_only() {
    let path = source_file("l08", "const x = 20; entry main returns x + 22;");
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ));
    assert!(!stdout.contains("c09="));
}

#[test]
fn older_expression_plan_lower_and_run_remain_frozen_for_const_source() {
    let path = source_file("frozen", "const x = 20; entry main returns x + 22;");
    for command in ["expr-plan", "expr-lower", "expr-run"] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
}

#[test]
fn certified_version_output_remains_exactly_frozen() {
    let output = Command::new(nordoi()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
