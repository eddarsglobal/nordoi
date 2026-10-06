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
        "nordoi-v05-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_vertical_v05_commands() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi condition-run <path|->"));
    assert!(stdout.contains("nordoi if-run <path|->"));
    assert!(stdout.contains("erases the dead branch before NAIR"));
}

#[test]
fn condition_run_executes_comparison_through_nair_08() {
    let path = source_file("condition", "entry main returns 20 <= 22;");
    let output = Command::new(nordoi())
        .args(["condition-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=INT_COMPARE(20<=22) result=BOOL(true)"));
    assert!(stdout.contains("nair-instructions=4 nair-minor=0.8"));
    assert!(stdout.contains("INT_LE r2 r0 r1"));
    assert!(stdout.contains("quiescent=true result=BOOL(true)"));
}

#[test]
fn condition_run_bool_literal_stays_nair_06() {
    let path = source_file("bool", "entry main returns true;");
    let output = Command::new(nordoi())
        .args(["condition-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind=BOOL(true) result=BOOL(true)"));
    assert!(stdout.contains("nair-instructions=2 nair-minor=0.6"));
    assert!(stdout.contains("CONST r0 BOOL(true)"));
}

#[test]
fn if_run_executes_then_branch_and_erases_else() {
    let path = source_file(
        "ifthen",
        "const value = 20; const limit = 22; entry main returns if value < limit { value + 22 } else { 999 };",
    );
    let output = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("condition=value<limit condition-value=true selected=THEN result=INT(42)")
    );
    assert!(stdout
        .contains("runtime-branches=0 dead-branch-eliminated=true dead-branch-instructions=0"));
    assert!(stdout.contains("nair-instructions=4 nair-minor=0.7"));
    assert!(stdout.contains("quiescent=true result=INT(42)"));
}

#[test]
fn if_run_executes_else_branch() {
    let path = source_file(
        "ifelse",
        "const value = 30; const limit = 22; entry main returns if value < limit { 999 } else { value + 12 };",
    );
    let output = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("condition-value=false selected=ELSE result=INT(42)"));
    assert!(stdout
        .contains("runtime-branches=0 dead-branch-eliminated=true dead-branch-instructions=0"));
}

#[test]
fn if_run_validates_dead_branch_before_execution() {
    let path = source_file(
        "dead-invalid",
        "entry main returns if true { 42 } else { missing + 1 };",
    );
    let output = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("error[if-run]:"));
}

#[test]
fn if_run_unknown_condition_binding_is_frontend_failure() {
    let path = source_file(
        "unknown-condition",
        "entry main returns if missing < 2 { 1 } else { 0 };",
    );
    let output = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn stdin_if_run_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["if-run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"const x = 20; entry main returns if x == 20 { 42 } else { 0 };")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("selected=THEN result=INT(42)"));
}

#[test]
fn legacy_pipelines_remain_closed_to_if_syntax() {
    let path = source_file("legacy", "entry main returns if true { 42 } else { 0 };");
    for command in ["expr-run", "bindings-run", "condition-plan"] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
}

#[test]
fn repeated_if_run_output_is_deterministic() {
    let path = source_file(
        "deterministic",
        "const x = 2; entry main returns if x >= 2 { 40 + 2 } else { 0 };",
    );
    let a = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let b = Command::new(nordoi())
        .args(["if-run", path.to_str().unwrap()])
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
