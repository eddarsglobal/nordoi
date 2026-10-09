use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(label: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nordoi-p25-{label}-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

fn temp_dir(label: &str) -> PathBuf {
    let path = temp_path(label, "dir");
    fs::create_dir(&path).unwrap();
    path
}

fn write_source(label: &str, text: &str) -> PathBuf {
    let path = temp_path(label, "noi");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn help_lists_p25_file_write_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("file-write <path|-> [--grant-output-dir <dir>]"));
    assert!(stdout.contains("P2.5"));
    assert!(stdout.contains("FileWrite"));
}

#[test]
fn file_write_without_grant_creates_nothing_and_fails_closed() {
    let source = write_source(
        "denied",
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"hello\";",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["file-write", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("capability denied"));
    assert!(stderr.contains("FileWrite"));
    let _ = fs::remove_file(source);
}

#[test]
fn explicit_output_directory_grant_writes_exact_file_and_receipt() {
    let source = write_source(
        "granted",
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"hello from P2.5\\n\";",
    );
    let dir = temp_dir("granted-output");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "file-write",
            source.to_str().unwrap(),
            "--grant-output-dir",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read(dir.join("report.txt")).unwrap(),
        b"hello from P2.5\n"
    );
    let receipt = String::from_utf8(output.stderr).unwrap();
    assert!(receipt.contains("target=\"report.txt\""));
    assert!(receipt.contains("status=WRITTEN"));
    assert!(receipt.contains("authority=EXPLICIT"));
    assert!(receipt.contains("ambient-authority=NONE"));
    assert!(receipt.contains("overwrite=DENIED"));
    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn existing_target_fails_without_overwrite_and_public_version_stays_frozen() {
    let source = write_source(
        "existing",
        "module app.main; effect FileWrite; entry main writes \"report.txt\" emits \"new\";",
    );
    let dir = temp_dir("existing-output");
    fs::write(dir.join("report.txt"), b"original").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "file-write",
            source.to_str().unwrap(),
            "--grant-output-dir",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert_eq!(fs::read(dir.join("report.txt")).unwrap(), b"original");
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot create new target"));

    let version = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(dir);
}
