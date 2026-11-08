//! G0.3 R4H — actual macOS App Sandbox entitlement on an ad-hoc signed helper.
//!
//! Research lab only: disposable fixture, signed C helper, OS filesystem probe.
//! An ad-hoc signature is NOT Developer ID, notarization or production signing.
//! No untrusted .noi guest, no public runtime/NAIR/kernel/CLI changes.
//! R0.2 ExternalIo is STILL unsupported; no file/NAM atomicity.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod macos {
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    static NEXT: AtomicU64 = AtomicU64::new(1);
    const MAX_OUTPUT: u64 = 4096;
    const HELPER_C: &str = r#"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    if (argc != 3) return 17;
    const char *sentinel = argv[1], *creation = argv[2];
    int rd = 0, wr = 0, cr = 0;
    int fd = open(sentinel, O_RDONLY);
    if (fd >= 0) {
        char c;
        rd = read(fd, &c, 1) == 1;
        close(fd);
    }
    fd = open(sentinel, O_WRONLY);
    if (fd >= 0) { wr = 1; close(fd); }
    fd = open(creation, O_WRONLY | O_CREAT | O_EXCL, 0600);
    if (fd >= 0) {
        cr = 1;
        close(fd);
        unlink(creation);
    }
    printf("R4H|OBS|READ=%d|WRITE=%d|CREATE=%d\n", rd, wr, cr);
    fflush(stdout);
    return 0;
}
"#;
    const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.app-sandbox</key>
  <true/>
</dict>
</plist>
"#;

    pub struct Scratch {
        dir: PathBuf,
    }
    impl Scratch {
        pub fn new() -> Self {
            let serial = NEXT.fetch_add(1, Ordering::Relaxed);
            let dir = env::temp_dir().join(format!("nordoi-r4h-{}-{serial}", std::process::id()));
            fs::create_dir(&dir).expect("new private disposable directory, never reuse");
            Self { dir: fs::canonicalize(dir).expect("canonicalize new fixture") }
        }
        pub fn sentinel(&self) -> PathBuf { self.dir.join("parent-only.txt") }
        pub fn create_target(&self) -> PathBuf { self.dir.join("child-create-probe.txt") }
        pub fn control(&self) -> PathBuf { self.dir.join("r4h-control") }
        pub fn isolated(&self) -> PathBuf { self.dir.join("r4h-entitled") }
        pub fn entitlements(&self) -> PathBuf { self.dir.join("r4h.entitlements") }
        pub fn source(&self) -> PathBuf { self.dir.join("r4h_helper.c") }

        pub fn prepare(&self) {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.sentinel())
                .expect("fresh parent-owned sentinel");
            file.write_all(b"R4H_PARENT_ONLY\n").expect("write fresh sentinel");
            fs::write(self.source(), HELPER_C).expect("write self-contained C test source");
            fs::write(self.entitlements(), ENTITLEMENTS).expect("write minimal entitlement plist");

            let compiler = Command::new("/usr/bin/cc")
                .args(["-x", "c", "-std=c11", "-O2", "-o"])
                .arg(self.control())
                .arg(self.source())
                .output()
                .expect("Xcode command line C compiler must be installed");
            assert!(
                compiler.status.success(),
                "compile fixed host helper: {}",
                String::from_utf8_lossy(&compiler.stderr)
            );
            fs::copy(self.control(), self.isolated()).expect("copy exact same unsigned helper bytes");

            // Sign BOTH binaries with ad-hoc signing. Only one receives the
            // app-sandbox entitlement, so signing alone cannot explain denial.
            Self::sign(&self.control(), None);
            Self::sign(&self.isolated(), Some(&self.entitlements()));
            self.verify_signed_entitlement();
            self.verify_sentinel();
        }
        fn sign(binary: &Path, entitlement: Option<&Path>) {
            let mut cmd = Command::new("/usr/bin/codesign");
            cmd.args(["--force", "--sign", "-", "--identifier", "org.eddarsglobal.nordoi.r4h.fixture"]);
            if let Some(entitlement) = entitlement {
                cmd.arg("--entitlements").arg(entitlement);
            }
            let output = cmd.arg(binary).output().expect("codesign exists on macOS");
            assert!(
                output.status.success(),
                "ad-hoc code signing must work: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        pub fn verify_signature(binary: &Path) -> bool {
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict", "--verbose=2"])
                .arg(binary)
                .output()
                .expect("codesign verification command")
                .status
                .success()
        }
        pub fn verify_signed_entitlement(&self) {
            assert!(Self::verify_signature(&self.control()), "signed positive-control identity");
            assert!(Self::verify_signature(&self.isolated()), "signed App Sandbox helper identity");

            let signed = Command::new("/usr/bin/codesign")
                .args(["--display", "--entitlements", ":-"])
                .arg(self.isolated())
                .output()
                .expect("extract entitlements from signed Mach-O");
            let actual = format!(
                "{}{}",
                String::from_utf8_lossy(&signed.stdout),
                String::from_utf8_lossy(&signed.stderr)
            );
            assert!(
                signed.status.success() && actual.contains("com.apple.security.app-sandbox"),
                "signed executable must contain real App Sandbox entitlement: {actual}"
            );
            assert!(
                actual.contains("<true/>") || actual.contains("<true />"),
                "the signed App Sandbox entitlement must be enabled: {actual}"
            );

            let control = Command::new("/usr/bin/codesign")
                .args(["--display", "--entitlements", ":-"])
                .arg(self.control())
                .output()
                .expect("inspect unentitled control");
            let info = format!(
                "{}{}",
                String::from_utf8_lossy(&control.stdout),
                String::from_utf8_lossy(&control.stderr)
            );
            assert!(
                !info.contains("com.apple.security.app-sandbox"),
                "positive-control executable must have NO App Sandbox entitlement"
            );
        }
        pub fn verify_sentinel(&self) {
            assert_eq!(fs::read(self.sentinel()).unwrap(), b"R4H_PARENT_ONLY\n");
            assert!(!self.create_target().exists());
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("only remove own newly created fixture directory");
        }
    }

    pub struct Witness {
        pub exit_ok: bool,
        pub timed_out: bool,
        pub stdout: String,
        pub stderr: String,
    }
    fn reap(child: &mut Child) -> (bool, bool) {
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("test helper exit status") {
                return (status.success(), false);
            }
            if start.elapsed() > Duration::from_secs(10) {
                let _ = child.kill();
                child.wait().expect("reap killed test helper");
                return (false, true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn run(bin: &Path, test: &Scratch) -> Witness {
        let mut child = Command::new(bin)
            .arg(test.sentinel())
            .arg(test.create_target())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("execute exact locally built helper");
        let (exit_ok, timed_out) = reap(&mut child);
        let mut output = String::new();
        let mut errors = String::new();
        child.stdout.take().expect("stdout pipe").take(MAX_OUTPUT + 1)
            .read_to_string(&mut output).expect("UTF-8 result");
        child.stderr.take().expect("stderr pipe").take(MAX_OUTPUT + 1)
            .read_to_string(&mut errors).expect("UTF-8 diagnostic");
        Witness { exit_ok, timed_out, stdout: output, stderr: errors }
    }
    pub fn expect(w: &Witness, allowed: bool) {
        assert!(w.exit_ok && !w.timed_out, "signed helper must exit cleanly: {}", w.stderr);
        assert!(w.stdout.len() as u64 <= MAX_OUTPUT, "stdout hard limit");
        let expected = if allowed {
            "R4H|OBS|READ=1|WRITE=1|CREATE=1\n"
        } else {
            "R4H|OBS|READ=0|WRITE=0|CREATE=0\n"
        };
        assert_eq!(w.stdout, expected, "real OS file access results must match the entitlement policy");
    }
    pub fn positive_and_isolated() {
        let scratch = Scratch::new();
        scratch.prepare();
        expect(&run(&scratch.control(), &scratch), true);
        scratch.verify_sentinel();
        expect(&run(&scratch.isolated(), &scratch), false);
        scratch.verify_sentinel();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn signed_app_sandbox_denies_real_child_file_read_write_and_create() {
    macos::positive_and_isolated();
}

#[cfg(target_os = "macos")]
#[test]
fn code_signing_without_entitlement_does_not_deny_control_access() {
    let scratch = macos::Scratch::new();
    scratch.prepare();
    macos::expect(&macos::run(&scratch.control(), &scratch), true);
    scratch.verify_sentinel();
}

#[cfg(target_os = "macos")]
#[test]
fn real_signed_entitlement_and_both_signatures_verify() {
    let scratch = macos::Scratch::new();
    scratch.prepare();
    scratch.verify_signed_entitlement();
}

#[cfg(target_os = "macos")]
#[test]
fn parent_still_reads_and_appends_after_sandboxed_child() {
    use std::fs::OpenOptions;
    use std::io::Write;
    let scratch = macos::Scratch::new();
    scratch.prepare();
    macos::expect(&macos::run(&scratch.isolated(), &scratch), false);
    let mut parent = OpenOptions::new().append(true).open(scratch.sentinel()).unwrap();
    parent.write_all(b"PARENT_CAN_APPEND\n").unwrap();
    assert_eq!(
        std::fs::read(scratch.sentinel()).unwrap(),
        b"R4H_PARENT_ONLY\nPARENT_CAN_APPEND\n"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn damaged_code_signature_is_detected_and_must_not_be_trusted() {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    let scratch = macos::Scratch::new();
    scratch.prepare();
    let original = std::fs::read(scratch.isolated()).unwrap();
    let tampered = scratch.dir.join("tampered");
    let mut file = OpenOptions::new().write(true).create_new(true).open(&tampered).unwrap();
    file.write_all(&original).unwrap();
    file.seek(SeekFrom::Start(256)).unwrap();
    file.write_all(&[original[256] ^ 0x11]).unwrap();
    drop(file);
    assert!(
        !macos::Scratch::verify_signature(&tampered),
        "modified Mach-O must lose code-signature verification"
    );
    scratch.verify_sentinel();
}

#[cfg(target_os = "macos")]
#[test]
fn forged_or_duplicate_child_reports_are_rejected() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for fabricated in [
        "R4H|OBS|READ=1|WRITE=1|CREATE=1\n",
        "R4H|OBS|READ=0|WRITE=0|CREATE=0\nR4H|OBS|READ=0|WRITE=0|CREATE=0\n",
        "R4H|OBS|READ=0|WRITE=1|CREATE=0\n",
    ] {
        let frame = macos::Witness {
            exit_ok: true,
            timed_out: false,
            stdout: fabricated.to_string(),
            stderr: String::new(),
        };
        assert!(catch_unwind(AssertUnwindSafe(|| macos::expect(&frame, false))).is_err());
    }
}

#[cfg(target_os = "macos")]
#[test]
fn multiple_distinct_signed_helpers_reproduce_real_file_denial() {
    for _ in 0..3 {
        macos::positive_and_isolated();
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn no_macos_app_sandbox_backend_on_other_os() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn r02_reference_still_disallows_external_io_grant() {
    let (mut model, permit) = Model::bootstrap(
        0x48,
        ScopeBudget { tasks: 0, resources: 0, children: 0 },
        16,
    );
    let scope = model.root();
    assert_eq!(
        model.host_grant(&permit, scope, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
