use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn nordoi() -> &'static str {
    env!("CARGO_BIN_EXE_nordoi")
}

fn source_file(name: &str, text: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("nordoi_c010_{}_{}.noi", std::process::id(), name));
    fs::write(&path, text).expect("test source must be writable");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c010_bindings_lower_and_preserves_prior_boundaries() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi bindings-lower <path|->"));
    assert!(stdout.contains("C0.10 bindings-lower erases immutable binding references"));
    assert!(stdout.contains("C0.9 bindings-plan preserves canonical binding identities"));
    assert!(stdout.contains("V0.3 expr-run executes C0.8"));
}

#[test]
fn bindings_lower_reports_zero_cost_binding_erasure_to_nair_07() {
    let path = source_file(
        "add",
        "module demo.bindings; type User; effect Network; const y = 22; const x = 20; entry main returns x + y;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bindings=[#1:x=INT(20),#2:y=INT(22)] count=2"));
    assert!(stdout.contains("ops=[BINDING(1),BINDING(2),ADD] value=INT(42) nodes=3"));
    assert!(stdout.contains("work=0 storage=0 effects=0 authority=NONE"));
    assert!(stdout.contains("nair-instructions=4 result-register=r2 nair-minor=0.7"));
    assert!(stdout.contains(
        "instructions=[CONST r0 INT(20),CONST r1 INT(22),ADD_INT_CHECKED r2 r0 r1,HALT]"
    ));
    assert!(
        stdout.contains("binding-runtime-storage=0 binding-runtime-lookups=0 runtime=NOT_INVOKED")
    );
    assert!(stdout.contains("c09="));
    assert!(stdout.contains("c010="));
}

#[test]
fn unused_binding_adds_no_nair_instruction() {
    let path = source_file("unused", "const unused = 999; entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("count=1 ops=[INT(42)] value=INT(42) nodes=1"));
    assert!(stdout.contains("nair-instructions=2 result-register=r0 nair-minor=0.6"));
    assert!(stdout.contains("instructions=[CONST r0 INT(42),HALT]"));
    assert!(stdout.contains("binding-runtime-storage=0 binding-runtime-lookups=0"));
}

#[test]
fn single_binding_reference_stays_nair_06() {
    let path = source_file("single", "const answer = 42; entry main returns answer;");
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[BINDING(1)] value=INT(42) nodes=1"));
    assert!(stdout.contains("nair-instructions=2 result-register=r0 nair-minor=0.6"));
    assert!(stdout.contains("instructions=[CONST r0 INT(42),HALT]"));
}

#[test]
fn grouped_binding_expression_preserves_postfix_ssa_order() {
    let path = source_file(
        "group",
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    );
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[BINDING(1),BINDING(2),BINDING(3),ADD,ADD] value=INT(6) nodes=5"));
    assert!(stdout.contains("result-register=r4 nair-minor=0.7"));
    assert!(stdout.contains("instructions=[CONST r0 INT(1),CONST r1 INT(2),CONST r2 INT(3),ADD_INT_CHECKED r3 r1 r2,ADD_INT_CHECKED r4 r0 r3,HALT]"));
}

#[test]
fn overflow_fails_closed_before_bindings_lower_publication() {
    let path = source_file(
        "overflow",
        "const max = 9223372036854775807; entry main returns max + 1;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[bindings-lower]:"));
    assert!(stderr.contains("overflowed"));
}

#[test]
fn stdin_bindings_lower_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["bindings-lower", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("ADD_INT_CHECKED r2 r0 r1"));
}

#[test]
fn c09_bindings_plan_remains_non_lowering_boundary() {
    let path = source_file("c09", "const x = 20; entry main returns x + 22;");
    let output = Command::new(nordoi())
        .args(["bindings-plan", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"));
    assert!(!stdout.contains("c010="));
}

#[test]
fn older_expression_lower_and_run_remain_frozen_for_const_source() {
    let path = source_file("frozen", "const x = 20; entry main returns x + 22;");
    for command in ["expr-lower", "expr-run"] {
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
