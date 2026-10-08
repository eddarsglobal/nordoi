use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_project(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("nordoi-v16-{label}-{}-{nonce}", std::process::id()));
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

fn build(root: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn help_lists_v16_release_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("release-check <project-root> [--json]"));
    assert!(stdout.contains("release <project-root> [--json]"));
    assert!(stdout.contains("V1.6"));
    assert!(stdout.contains("provenance"));
}

#[test]
fn release_check_json_is_side_effect_free_and_exact() {
    let root = temp_project("check");
    build(&root);
    let output = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["release-check", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"schema\":\"nordoi.release.v1\""));
    assert!(stdout.contains("\"status\":\"pass\""));
    assert!(stdout.contains("\"lock\":\"exact\""));
    assert!(stdout.contains("\"packageMatch\":\"exact\""));
    assert!(stdout.contains("\"platformNeutral\":true"));
    assert!(!root.join("dist").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn release_publishes_reproducible_package_checksum_and_provenance() {
    let root = temp_project("publish");
    build(&root);
    let source_package = fs::read(root.join("build/demo-0.1.0.npkg")).unwrap();

    let first = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["release", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let stdout = String::from_utf8(first.stdout).unwrap();
    assert!(stdout.contains("status=PUBLISHED"));
    assert!(stdout.contains("platform-neutral=true"));

    let dist_package = root.join("dist/demo-0.1.0.npkg");
    let checksum = root.join("dist/demo-0.1.0.npkg.sha256");
    let provenance = root.join("dist/demo-0.1.0.provenance");
    assert_eq!(fs::read(&dist_package).unwrap(), source_package);
    let checksum_first = fs::read(&checksum).unwrap();
    let provenance_first = fs::read(&provenance).unwrap();
    assert!(String::from_utf8_lossy(&checksum_first).contains("demo-0.1.0.npkg"));
    assert!(String::from_utf8_lossy(&provenance_first).contains("nordoi.release.provenance.v1"));

    let second = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["release", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(fs::read(&dist_package).unwrap(), source_package);
    assert_eq!(fs::read(&checksum).unwrap(), checksum_first);
    assert_eq!(fs::read(&provenance).unwrap(), provenance_first);
    let json = String::from_utf8(second.stdout).unwrap();
    assert!(json.contains("\"status\":\"published\""));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn release_rejects_drift_and_public_version_stays_frozen() {
    let root = temp_project("drift");
    build(&root);
    fs::write(
        root.join("src/app/math.noi"),
        "module app.math; fn add_100(x) returns x + 101;",
    )
    .unwrap();
    let release = Command::new(env!("CARGO_BIN_EXE_nordoi"))
        .args(["release-check", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(release.status.code(), Some(4));
    let stdout = String::from_utf8(release.stdout).unwrap();
    assert!(stdout.contains("\"schema\":\"nordoi.release.v1\""));
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("\"stage\":\"lock\""));
    assert!(!root.join("dist").exists());

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
