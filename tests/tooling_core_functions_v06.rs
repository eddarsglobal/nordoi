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
        "nordoi-v06-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v06_core_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi core-run <path|->"));
    assert!(stdout.contains("pure functions, parameters"));
    assert!(stdout.contains("minimal CONST+HALT NAIR"));
}

#[test]
fn core_run_executes_function_and_reports_zero_runtime_calls() {
    let path = source_file(
        "function",
        "fn add(a,b){a+b} entry main returns add(20,22);",
    );
    let output = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("functions=1"));
    assert!(stdout.contains("calls=1 inlined-calls=1"));
    assert!(
        stdout.contains("result=INT(42) constant-folded=true runtime-calls=0 runtime-branches=0")
    );
    assert!(stdout.contains("nair-instructions=2 nair-minor=0.6"));
    assert!(stdout.contains("[CONST r0 INT(42),HALT]"));
    assert!(stdout.contains("quiescent=true result=INT(42)"));
}

#[test]
fn core_run_supports_boolean_logic() {
    let path = source_file("bool", "entry main returns !(false || false) && 20 < 22;");
    let output = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=BOOL(true)"));
    assert!(stdout.contains("[CONST r0 BOOL(true),HALT]"));
}

#[test]
fn core_run_supports_local_binding_and_if_inside_function() {
    let path = source_file(
        "local-if",
        "fn choose(x){ const doubled = x * 2; if doubled >= 40 { doubled + 2 } else { 0 } } entry main returns choose(20);",
    );
    let output = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("static-ifs=1 result=INT(42)"));
    assert!(stdout.contains("runtime-branches=0"));
}

#[test]
fn stdin_core_run_is_supported() {
    let mut child = Command::new(nordoi())
        .args(["core-run", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"fn f(x){x*2} entry main returns f(21);")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("result=INT(42)"));
}

#[test]
fn division_by_zero_is_frontend_failure() {
    let path = source_file("divzero", "entry main returns 42 / 0;");
    let output = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("division by zero"));
}

#[test]
fn recursion_is_frontend_failure() {
    let path = source_file(
        "recursion",
        "fn loop(x){loop(x)} entry main returns loop(1);",
    );
    let output = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    cleanup(&path);
    assert_eq!(output.status.code(), Some(4));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("recursive pure function"));
}

#[test]
fn legacy_pipelines_remain_closed_to_function_syntax() {
    let path = source_file("legacy", "fn add(a,b){a+b} entry main returns add(20,22);");
    for command in ["expr-run", "bindings-run", "if-run"] {
        let output = Command::new(nordoi())
            .args([command, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(4), "command={command}");
    }
    cleanup(&path);
}

#[test]
fn repeated_core_run_output_is_deterministic() {
    let path = source_file("deterministic", "fn f(x){x*2} entry main returns f(21);");
    let a = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
        .output()
        .unwrap();
    let b = Command::new(nordoi())
        .args(["core-run", path.to_str().unwrap()])
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
