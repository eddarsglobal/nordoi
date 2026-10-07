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
        "nordoi-v12-{label}-{}-{id}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn help_lists_v12_function_control_run() {
    let output = Command::new(nordoi()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("nordoi function-control-run <path|-> <key-code>"));
    assert!(stdout.contains("NAIR 0.14"));
}

#[test]
fn function_control_run_executes_true_path() {
    let path = source_file(
        "true",
        "module demo.control; fn bias(x) returns if x > 40 { x + 100 } else { x + 200 }; input key_code; entry main returns bias(key_code);",
    );
    let output = Command::new(nordoi())
        .args(["function-control-run", path.to_str().unwrap(), "41"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(141)"));
    assert!(stdout.contains("runtime-calls=1 runtime-branches=1"));
    assert!(stdout.contains("max-call-depth=1 certified-max-call-depth=1"));
    assert!(stdout.contains("nair-minor=0.14"));
    assert!(stdout.contains("IF(GT(PARAM(0), INT(40))"));
    assert!(stdout.contains("authority=NONE input-boundary=CANONICAL"));
}

#[test]
fn function_control_run_executes_false_path() {
    let path = source_file(
        "false",
        "fn bias(x) returns if x > 40 { x + 100 } else { x + 200 }; input key_code; entry main returns bias(key_code);",
    );
    let output = Command::new(nordoi())
        .args(["function-control-run", path.to_str().unwrap(), "39"])
        .output()
        .unwrap();
    cleanup(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(239)"));
    assert!(stdout.contains("runtime-calls=1 runtime-branches=1"));
    assert!(stdout.contains("nair-minor=0.14"));
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
