use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nordoi-v13-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(path.join("app")).unwrap();
    path
}

#[test]
fn help_lists_v13_module_graph_run() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("module-graph-run <source-root> <entry-module> <key-code>"));
    assert!(stdout.contains("V1.3"));
}

#[test]
fn module_graph_run_executes_real_two_file_program() {
    let root = temp_root("run");
    fs::write(
        root.join("app/math.noi"),
        "module app.math; fn add_100(x) returns x + 100;",
    )
    .unwrap();
    fs::write(root.join("app/main.noi"), "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["module-graph-run", root.to_str().unwrap(), "app.main", "41"])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&root);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("result=INT(141)"));
    assert!(stdout.contains("modules=2"));
    assert!(stdout.contains("imports=1"));
    assert!(stdout.contains("import-resolution=STATIC"));
    assert!(stdout.contains("runtime-fs=NONE"));
}

#[test]
fn module_graph_run_missing_file_is_io_error() {
    let root = temp_root("missing");
    fs::write(root.join("app/main.noi"), "module app.main; import app.math; input key_code; entry main returns math.add_100(key_code);").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["module-graph-run", root.to_str().unwrap(), "app.main", "41"])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn certified_public_version_remains_unchanged() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)\n"
    );
}
