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
        "nordoi-v03-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v03_expr_run_without_rewriting_prior_commands() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi expr-run <path|->"));
    assert!(stdout.contains("V0.3 expr-run executes C0.8"));
    assert!(stdout.contains("nordoi expr-lower <path|->"));
    assert!(stdout.contains("nordoi result-run <path|->"));
}

#[test]
fn expr_run_executes_addition_and_reports_observed_result() {
    let path = source_file(
        "add",
        "module demo; type User; effect Network; entry main returns 20 + 22;",
    );
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(20),INT(22),ADD] result=INT(42) result-register=r2"));
    assert!(stdout
        .contains("nodes=3 work=0 effects=0 authority=NONE nair-instructions=4 nair-minor=0.7"));
    assert!(stdout.contains("executed=4 input=0 registers=3"));
    assert!(stdout.contains("domains=0 atoms=0 transactions=0 frames=0 bridges=0 scheduled=0"));
    assert!(stdout.contains("quiescent=true result=INT(42)"));
    assert!(stdout.contains("c08="));
    assert!(stdout.contains("receipt="));
}

#[test]
fn expr_run_preserves_grouped_postfix_execution() {
    let path = source_file("group", "entry main returns 1 + (2 + 3);");
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(1),INT(2),INT(3),ADD,ADD] result=INT(6) result-register=r4"));
    assert!(stdout.contains("nair-instructions=6 nair-minor=0.7"));
    assert!(stdout.contains("executed=6 input=0 registers=5"));
}

#[test]
fn literal_only_expr_run_remains_nair_06() {
    let path = source_file("literal", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(42) result-register=r0 nodes=1"));
    assert!(stdout.contains("nair-instructions=2 nair-minor=0.6"));
    assert!(stdout.contains("executed=2 input=0 registers=1"));
}

#[test]
fn expr_run_reports_no_expression_without_registers() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=NONE result=NONE result-register=NONE nodes=0"));
    assert!(stdout.contains("nair-instructions=1 nair-minor=0.6"));
    assert!(stdout.contains("executed=1 input=0 registers=0"));
    assert!(stdout.contains("quiescent=true result=NONE"));
}

#[test]
fn overflow_fails_with_frontend_exit_code_before_runtime() {
    let path = source_file("overflow", "entry main returns 9223372036854775807 + 1;");
    let output = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[expr-run]:"));
}

#[test]
fn repeated_expr_run_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 20 + 22;");
    let first = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["expr-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn stdin_expr_run_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["expr-run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"entry main returns 20 + 22;")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("quiescent=true result=INT(42)"));
}

#[test]
fn certified_result_run_remains_frozen_for_addition_source() {
    let path = source_file("result-run-frozen", "entry main returns 20 + 22;");
    let output = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
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
