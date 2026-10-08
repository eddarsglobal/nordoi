use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_project(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("nordoi-v15-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("src/app")).unwrap();
    fs::write(
        root.join("NORDOI.toml"),
        "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"app.main\"\nsource-root = \"src\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/app/math.noi"),
        "module app.math; fn add_100(x) returns x + 100;",
    )
    .unwrap();
    fs::write(
        root.join("src/app/main.noi"),
        "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);",
    )
    .unwrap();
    root
}

#[test]
fn help_lists_v15_check_and_json_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("check <project-root> [--json]"));
    assert!(stdout.contains("V1.5"));
    assert!(stdout.contains("stable diagnostics"));
}

#[test]
fn check_text_reports_clean_project_without_writing_build_outputs() {
    let root = temp_project("text");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["check", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("status=PASS"));
    assert!(stdout.contains("diagnostics=0"));
    assert!(stdout.contains("modules=2"));
    assert!(!root.join("build").exists());
    assert!(!root.join("NORDOI.lock").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn check_json_reports_schema_versioned_success_object() {
    let root = temp_project("json");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["check", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"schema\":\"nordoi.diagnostic.v1\""));
    assert!(stdout.contains("\"status\":\"pass\""));
    assert!(stdout.contains("\"diagnostics\":[]"));
    assert!(output.stderr.is_empty());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_import_json_has_stable_code_trace_and_version_remains_frozen() {
    let root = temp_project("missing");
    fs::remove_file(root.join("src/app/math.noi")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["check", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"code\":\"NDX2004\""));
    assert!(stdout.contains("\"importTrace\":[\"app.main\",\"app.math\"]"));

    let version = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
    let _ = fs::remove_dir_all(&root);
}
