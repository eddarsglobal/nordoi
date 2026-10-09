use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(label: &str, extension: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nordoi-p27-tool-{label}-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let path = temp_path(label, "dir");
    fs::create_dir(&path).unwrap();
    path
}

fn write_source(label: &str) -> std::path::PathBuf {
    let path = temp_path(label, "noi");
    fs::write(
        &path,
        b"module app.main; effect FileWrite; input key_code; entry main writes \"bundle\" { \"a.txt\" emits \"a=\" + key_code + \",b=\" + key_code; \"b.txt\" emits \"accepted=\" + (key_code >= 40) + \",key=\" + key_code; };",
    )
    .unwrap();
    path
}

#[test]
fn help_lists_p27_bundle_write_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bundle-write <path|-> <key-code> [--grant-output-dir <dir>]"));
    assert!(stdout.contains("P2.7"));
    assert!(stdout.contains("atomic bundle"));
}

#[test]
fn bundle_write_without_grant_creates_nothing_and_fails_closed() {
    let source = write_source("denied");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["bundle-write", source.to_str().unwrap(), "40"])
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
fn explicit_output_directory_grant_publishes_complete_bundle_and_receipt() {
    let source = write_source("granted");
    let dir = temp_dir("granted-output");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "bundle-write",
            source.to_str().unwrap(),
            "40",
            "--grant-output-dir",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(dir.join("bundle/a.txt")).unwrap(),
        "a=40,b=40"
    );
    assert_eq!(
        fs::read_to_string(dir.join("bundle/b.txt")).unwrap(),
        "accepted=true,key=40"
    );
    let receipt = String::from_utf8(output.stderr).unwrap();
    assert!(receipt.contains("bundle-write"));
    assert!(receipt.contains("files=2"));
    assert!(
        receipt.contains("results=[\"a.txt\":[INT(40),INT(40)],\"b.txt\":[BOOL(true),INT(40)]]")
    );
    assert!(receipt.contains("status=COMMITTED"));
    assert!(receipt.contains("authority=EXPLICIT"));
    assert!(receipt.contains("ambient-authority=NONE"));
    assert!(receipt.contains("partial-final-state=FORBIDDEN"));
    assert!(receipt.contains("publication=DIRECTORY-RENAME"));
    assert!(receipt.contains("crash-durability=UNCLAIMED"));
    assert!(!receipt.contains("ConsoleWrite"));
    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn existing_bundle_fails_without_overwrite_and_public_version_stays_frozen() {
    let source = write_source("existing");
    let dir = temp_dir("existing-output");
    fs::create_dir(dir.join("bundle")).unwrap();
    fs::write(dir.join("bundle/original.txt"), b"original").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args([
            "bundle-write",
            source.to_str().unwrap(),
            "40",
            "--grant-output-dir",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read(dir.join("bundle/original.txt")).unwrap(),
        b"original"
    );
    assert!(!dir.join("bundle/a.txt").exists());

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
