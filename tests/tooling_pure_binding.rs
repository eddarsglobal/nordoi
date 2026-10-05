use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_l08_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_l08_bindings_command_and_zero_runtime_claim() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi bindings <path|->"));
    assert!(stdout.contains("Print the L0.8 pure named-binding semantic boundary."));
    assert!(stdout.contains("without planning, NAIR, runtime work, storage, effects, or authority"));
}

#[test]
fn bindings_command_reports_canonical_registry_and_value() {
    let path = source_file(
        "main",
        "module demo; type A; effect Net; const y = 22; const x = 20; entry main returns x + y;",
    );
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bindings module=\"demo\" form=ENTRY entry=\"main\""));
    assert!(stdout.contains("bindings=[#1:x=INT(20),#2:y=INT(22)] count=2"));
    assert!(stdout.contains("ops=[BINDING(1),BINDING(2),ADD] value=INT(42) nodes=3"));
    assert!(stdout.contains("effects=0 authority=NONE l08="));
    assert!(stdout.contains(
        "planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ));
}

#[test]
fn declaration_order_produces_identical_cli_output() {
    let a = source_file(
        "order_a",
        "const x = 20; const y = 22; entry main returns x + y;",
    );
    let b = source_file(
        "order_b",
        "const y = 22; const x = 20; entry main returns x + y;",
    );
    let out_a = Command::new(nordoi())
        .args(["bindings", a.to_str().unwrap()])
        .output()
        .unwrap();
    let out_b = Command::new(nordoi())
        .args(["bindings", b.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&a);
    cleanup(&b);
    assert!(out_a.status.success());
    assert!(out_b.status.success());
    assert_eq!(out_a.stdout, out_b.stdout);
}

#[test]
fn unknown_binding_returns_frontend_exit_code() {
    let path = source_file("unknown", "entry main returns missing + 1;");
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[bindings]:"));
    assert!(stderr.contains("undeclared pure binding 'missing'"));
}

#[test]
fn duplicate_binding_returns_frontend_exit_code() {
    let path = source_file(
        "duplicate",
        "const x = 1; const x = 2; entry main returns x;",
    );
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("duplicate L0.8 pure binding 'x'"));
}

#[test]
fn certified_l07_and_later_expression_boundaries_reject_l08_const_source() {
    let path = source_file("frozen", "const x = 20; entry main returns x + 22;");
    for command in ["expr", "expr-plan", "expr-lower", "expr-run"] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
}

#[test]
fn bindings_accepts_stdin() {
    let mut child = Command::new(nordoi())
        .args(["bindings", "-"])
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
}

#[test]
fn bindings_without_const_still_accepts_l07_shape_but_does_not_plan() {
    let path = source_file("legacy_shape", "entry main returns 20 + 22;");
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bindings=[] count=0"));
    assert!(stdout.contains("ops=[INT(20),INT(22),ADD] value=INT(42)"));
    assert!(stdout.contains("runtime=NOT_INVOKED"));
}

#[test]
fn overflow_fails_closed_at_bindings_boundary() {
    let path = source_file(
        "overflow",
        "const max = 9223372036854775807; entry main returns max + 1;",
    );
    let output = Command::new(nordoi())
        .args(["bindings", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("overflowed"));
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
