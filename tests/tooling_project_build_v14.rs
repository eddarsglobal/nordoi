use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_project(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("nordoi-v14-{label}-{}-{nonce}", std::process::id()));
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
fn help_lists_v14_project_build_and_package_info() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("build <project-root> [--locked]"));
    assert!(stdout.contains("package-info <package.npkg>"));
    assert!(stdout.contains("V1.4"));
}

#[test]
fn build_writes_canonical_lock_and_deterministic_package() {
    let root = temp_project("build");
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("project=\"demo\""));
    assert!(stdout.contains("modules=2"));
    assert!(stdout.contains("imports=1"));
    assert!(stdout.contains("nair-minor=0.12"));
    assert!(stdout.contains("dependency-network=NONE"));
    assert!(stdout.contains("reproducible=true"));

    let lock = fs::read_to_string(root.join("NORDOI.lock")).unwrap();
    assert!(lock.contains("# NORDOI lock v1.4"));
    assert!(lock.contains("module = \"app.main\""));
    assert!(lock.contains("module = \"app.math\""));
    assert!(root.join("build/demo-0.1.0.npkg").is_file());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn locked_rebuild_is_identical_and_package_info_validates_artifact() {
    let root = temp_project("locked");
    let first = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(first.status.success());
    let package_path = root.join("build/demo-0.1.0.npkg");
    let first_bytes = fs::read(&package_path).unwrap();

    let second = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap(), "--locked"])
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(first_bytes, fs::read(&package_path).unwrap());

    let info = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["package-info", package_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        info.status.success(),
        "{}",
        String::from_utf8_lossy(&info.stderr)
    );
    let stdout = String::from_utf8(info.stdout).unwrap();
    assert!(stdout.contains("package-format=1.0"));
    assert!(stdout.contains("project=\"demo\""));
    assert!(stdout.contains("nair-minor=0.12"));
    assert!(stdout.contains("runtime-fs=NONE"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn locked_build_rejects_drift_and_public_version_stays_frozen() {
    let root = temp_project("drift");
    let first = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(first.status.success());

    fs::write(
        root.join("src/app/math.noi"),
        "module app.math; fn add_100(x) returns x + 101;",
    )
    .unwrap();
    let drift = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap(), "--locked"])
        .output()
        .unwrap();
    assert_eq!(drift.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&drift.stderr).contains("--locked rejected project drift"));

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
