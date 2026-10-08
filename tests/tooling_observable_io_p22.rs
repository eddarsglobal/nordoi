use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_source(label: &str, text: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nordoi-p22-{label}-{}-{nonce}.noi",
        std::process::id()
    ));
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn help_lists_p22_dynamic_console_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("dynamic-console-run <path|-> <key-code> [--grant-console]"));
    assert!(stdout.contains("P2.2"));
    assert!(stdout.contains("ConsoleWrite"));
}

#[test]
fn dynamic_console_without_grant_emits_zero_program_output_and_fails_closed() {
    let path = temp_source(
        "denied",
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["dynamic-console-run", path.to_str().unwrap(), "40"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("capability denied"));
    assert!(stderr.contains("ConsoleWrite"));
    let _ = fs::remove_file(path);
}

#[test]
fn explicit_grant_emits_runtime_value_and_deterministic_receipt_metadata() {
    let path = temp_source(
        "granted",
        "module app.main; effect ConsoleWrite; input key_code; const bias = 2; entry main emits key_code + bias;",
    );
    let first = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "dynamic-console-run",
            path.to_str().unwrap(),
            "40",
            "--grant-console",
        ])
        .output()
        .unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "dynamic-console-run",
            path.to_str().unwrap(),
            "40",
            "--grant-console",
        ])
        .output()
        .unwrap();
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, b"42");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let receipt = String::from_utf8(first.stderr).unwrap();
    assert!(receipt.contains("status=EMITTED"));
    assert!(receipt.contains("result=INT(42)"));
    assert!(receipt.contains("authority=EXPLICIT"));
    assert!(receipt.contains("ambient-authority=NONE"));
    assert!(receipt.contains("runtime-receipt-sha256="));
    let _ = fs::remove_file(path);
}

#[test]
fn static_or_malformed_dynamic_output_fails_before_emission_and_version_stays_frozen() {
    let path = temp_source(
        "static",
        "module app.main; effect ConsoleWrite; input key_code; entry main emits 42;",
    );
    let denied = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "dynamic-console-run",
            path.to_str().unwrap(),
            "40",
            "--grant-console",
        ])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(4));
    assert!(denied.stdout.is_empty());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("requires output to depend"));

    let version = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
    let _ = fs::remove_file(path);
}
