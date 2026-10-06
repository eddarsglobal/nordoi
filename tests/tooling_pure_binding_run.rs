use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn nordoi() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nordoi"))
}

fn source_file(label: &str, text: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nordoi-v04-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v04_bindings_run_without_rewriting_prior_commands() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi bindings-run <path|->"));
    assert!(stdout.contains("V0.4 bindings-run executes C0.10"));
    assert!(stdout.contains("nordoi bindings-lower <path|->"));
    assert!(stdout.contains("nordoi expr-run <path|->"));
}

#[test]
fn bindings_run_executes_addition_and_reports_zero_binding_runtime_cost() {
    let path = source_file(
        "add",
        "module demo.bindings; type User; effect Network; const y = 22; const x = 20; entry main returns x + y;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bindings=[#1:x=INT(20),#2:y=INT(22)] count=2"));
    assert!(stdout.contains("ops=[BINDING(1),BINDING(2),ADD] result=INT(42) result-register=r2"));
    assert!(stdout
        .contains("work=0 storage=0 effects=0 authority=NONE nair-instructions=4 nair-minor=0.7"));
    assert!(stdout.contains("executed=4 input=0 registers=3"));
    assert!(stdout.contains("domains=0 atoms=0 transactions=0 frames=0 bridges=0 scheduled=0"));
    assert!(stdout.contains(
        "quiescent=true result=INT(42) binding-runtime-storage=0 binding-runtime-lookups=0"
    ));
    assert!(stdout.contains("c010="));
    assert!(stdout.contains("receipt="));
}

#[test]
fn unused_binding_executes_with_literal_only_cost() {
    let path = source_file("unused", "const unused = 999; entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("count=1 ops=[INT(42)] result=INT(42) result-register=r0 nodes=1"));
    assert!(stdout.contains("nair-instructions=2 nair-minor=0.6"));
    assert!(stdout.contains("executed=2 input=0 registers=1"));
    assert!(stdout.contains("binding-runtime-storage=0 binding-runtime-lookups=0"));
}

#[test]
fn single_binding_reference_remains_nair_06() {
    let path = source_file("single", "const answer = 42; entry main returns answer;");
    let output = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[BINDING(1)] result=INT(42) result-register=r0 nodes=1"));
    assert!(stdout.contains("nair-instructions=2 nair-minor=0.6"));
    assert!(stdout.contains("executed=2 input=0 registers=1"));
}

#[test]
fn grouped_binding_expression_preserves_postfix_execution() {
    let path = source_file(
        "group",
        "const x = 1; const y = 2; const z = 3; entry main returns x + (y + z);",
    );
    let output = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "ops=[BINDING(1),BINDING(2),BINDING(3),ADD,ADD] result=INT(6) result-register=r4"
    ));
    assert!(stdout.contains("nair-instructions=6 nair-minor=0.7"));
    assert!(stdout.contains("executed=6 input=0 registers=5"));
}

#[test]
fn overflow_fails_with_frontend_exit_code_before_runtime() {
    let path = source_file(
        "overflow",
        "const max = 9223372036854775807; entry main returns max + 1;",
    );
    let output = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[bindings-run]:"));
}

#[test]
fn repeated_bindings_run_output_is_deterministic() {
    let path = source_file(
        "deterministic",
        "module demo; const x = 20; entry main returns x + 22;",
    );
    let first = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["bindings-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn stdin_bindings_run_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["bindings-run", "-"])
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
    assert!(String::from_utf8(output.stdout).unwrap().contains(
        "quiescent=true result=INT(42) binding-runtime-storage=0 binding-runtime-lookups=0"
    ));
}

#[test]
fn c010_bindings_lower_remains_non_executing_boundary() {
    let path = source_file("c010", "const x = 20; entry main returns x + 22;");
    let output = Command::new(nordoi())
        .args(["bindings-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("runtime=NOT_INVOKED"));
    assert!(!stdout.contains("receipt="));
}

#[test]
fn certified_expr_run_remains_frozen_for_const_source() {
    let path = source_file(
        "expr-run-frozen",
        "const x = 20; entry main returns x + 22;",
    );
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
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
