use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn nordoi() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nordoi"))
}

fn source_file(label: &str, text: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nordoi-c08-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_c08_expr_lower_without_rewriting_old_commands() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi expr-lower <path|->"));
    assert!(stdout.contains("C0.8 expr-lower is additive"));
    assert!(stdout.contains("C0.7 expr-plan is additive"));
    assert!(stdout.contains("V0.2 result-run executes C0.6"));
}

#[test]
fn expr_lower_reports_faithful_nair_07_for_addition() {
    let path = source_file(
        "add",
        "module demo.expr; type User; effect Network; entry main returns 20 + 22;",
    );
    let output = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout
        .contains("form=ENTRY entry=\"main\" ops=[INT(20),INT(22),ADD] value=INT(42) nodes=3"));
    assert!(stdout.contains("work=0 effects=0 authority=NONE"));
    assert!(stdout.contains("nair-instructions=4 result-register=r2 nair-minor=0.7"));
    assert!(stdout.contains(
        "instructions=[CONST r0 INT(20),CONST r1 INT(22),ADD_INT_CHECKED r2 r0 r1,HALT]"
    ));
    assert!(stdout.contains("runtime=NOT_INVOKED"));
    assert!(stdout.contains("c07="));
    assert!(stdout.contains("c08="));
}

#[test]
fn expr_lower_preserves_grouped_postfix_order() {
    let path = source_file("group", "entry main returns 1 + (2 + 3);");
    let output = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=[INT(1),INT(2),INT(3),ADD,ADD] value=INT(6) nodes=5"));
    assert!(stdout.contains("result-register=r4 nair-minor=0.7"));
    assert!(stdout.contains("instructions=[CONST r0 INT(1),CONST r1 INT(2),CONST r2 INT(3),ADD_INT_CHECKED r3 r1 r2,ADD_INT_CHECKED r4 r0 r3,HALT]"));
}

#[test]
fn literal_only_expr_lower_stays_nair_06() {
    let path = source_file("literal", "entry main returns 42;");
    let output = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("value=INT(42)"));
    assert!(stdout.contains("nair-instructions=2 result-register=r0 nair-minor=0.6"));
    assert!(stdout.contains("nair version=0.6 instructions=[CONST r0 INT(42),HALT]"));
}

#[test]
fn entry_without_expression_stays_halt_only_06() {
    let path = source_file("none", "entry main;");
    let output = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ops=NONE value=NONE nodes=0"));
    assert!(stdout.contains("nair-instructions=1 result-register=NONE nair-minor=0.6"));
    assert!(stdout.contains("instructions=[HALT]"));
}

#[test]
fn certified_result_lower_and_result_run_remain_frozen_for_addition_source() {
    let path = source_file("frozen", "entry main returns 20 + 22;");
    let result_lower = Command::new(nordoi())
        .args(["result-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    let result_run = Command::new(nordoi())
        .args(["result-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(result_lower.status.code(), Some(4));
    assert_eq!(result_run.status.code(), Some(4));
}

#[test]
fn overflow_fails_closed_at_expr_lower_boundary() {
    let path = source_file("overflow", "entry main returns 9223372036854775807 + 1;");
    let output = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[expr-lower]:"));
    assert!(stderr.contains("overflowed"));
}

#[test]
fn repeated_expr_lower_output_is_deterministic() {
    let path = source_file("deterministic", "module demo; entry main returns 20 + 22;");
    let first = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    let second = Command::new(nordoi())
        .args(["expr-lower", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn stdin_expr_lower_is_supported() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(nordoi())
        .args(["expr-lower", "-"])
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
        .contains("ADD_INT_CHECKED r2 r0 r1"));
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
