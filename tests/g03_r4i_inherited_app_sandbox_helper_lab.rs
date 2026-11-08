//! G0.3 R4I — real App Sandbox inheritance from signed parent to helper.
//!
//! A disposable ad-hoc signed macOS .app uses posix_spawn for an embedded,
//! separately signed helper with exactly app-sandbox + inherit entitlements.
//! This is research-only, NOT production signing, notarization, XPC or a
//! general untrusted .noi process executor. Other OS explicitly unsupported.
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

    const MAX_STDOUT: u64 = 4096;
    static SERIAL: AtomicU64 = AtomicU64::new(1);

    const HELPER_SOURCE: &str = r#"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    if (argc != 3) return 61;
    int rd=0, wr=0, cr=0;
    int fd = open(argv[1], O_RDONLY);
    if (fd >= 0) {
        char c;
        rd = (read(fd, &c, 1) == 1);
        close(fd);
    }
    fd = open(argv[1], O_WRONLY);
    if (fd >= 0) { wr=1; close(fd); }
    fd = open(argv[2], O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (fd >= 0) { cr=1; close(fd); unlink(argv[2]); }
    printf("R4I|HELPER|READ=%d|WRITE=%d|CREATE=%d\n", rd, wr, cr);
    fflush(stdout);
    return 0;
}
"#;
    const PARENT_SOURCE: &str = r#"
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
#include <spawn.h>
#include <sys/wait.h>
#include <string.h>
#include <errno.h>
extern char **environ;
int main(int argc, char **argv) {
    if (argc != 4) return 70;
    int rd=0, wr=0, cr=0;
    int fd=open(argv[1], O_RDONLY);
    if (fd >= 0) {
        char c;
        rd = (read(fd, &c, 1) == 1);
        close(fd);
    }
    fd=open(argv[1], O_WRONLY);
    if (fd >= 0) { wr=1; close(fd); }
    fd=open(argv[2], O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (fd >= 0) { cr=1; close(fd); unlink(argv[2]); }
    printf("R4I|PARENT|READ=%d|WRITE=%d|CREATE=%d\n", rd, wr, cr);
    fflush(stdout);

    int pipefd[2];
    if (pipe(pipefd) != 0) return 71;
    posix_spawn_file_actions_t actions;
    if (posix_spawn_file_actions_init(&actions) != 0) return 72;
    if (posix_spawn_file_actions_adddup2(&actions, pipefd[1], STDOUT_FILENO) != 0) return 73;
    if (posix_spawn_file_actions_addclose(&actions, pipefd[0]) != 0) return 74;
    if (posix_spawn_file_actions_addclose(&actions, pipefd[1]) != 0) return 75;

    pid_t child = -1;
    char *args[] = {argv[3], argv[1], argv[2], NULL};
    int spawned = posix_spawn(&child, argv[3], &actions, NULL, args, environ);
    posix_spawn_file_actions_destroy(&actions);
    close(pipefd[1]);
    if (spawned != 0) {
        fprintf(stderr, "R4I|SPAWN_ERROR|%d\n", spawned);
        close(pipefd[0]);
        return 76;
    }
    char buf[256] = {0};
    ssize_t n = read(pipefd[0], buf, sizeof(buf)-1);
    close(pipefd[0]);
    int status=0;
    if (waitpid(child, &status, 0) != child) return 77;
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        fprintf(stderr, "R4I|HELPER_EXIT|%d\n", status);
        return 78;
    }
    if (n <= 0 || n >= (ssize_t)(sizeof(buf)-1)) return 79;
    if (strncmp(buf, "R4I|HELPER|", 11) != 0) return 80;
    if (memchr(buf, '\n', n) != (buf + n - 1)) return 81;
    fwrite(buf, 1, (size_t)n, stdout);
    fflush(stdout);
    return 0;
}
"#;
    const APP_ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>com.apple.security.app-sandbox</key><true/>
</dict></plist>
"#;
    const HELPER_ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>com.apple.security.app-sandbox</key><true/>
<key>com.apple.security.inherit</key><true/>
</dict></plist>
"#;

    pub struct Lab {
        pub dir: PathBuf,
    }
    impl Lab {
        pub fn new() -> Self {
            let seq = SERIAL.fetch_add(1, Ordering::Relaxed);
            let dir = env::temp_dir().join(format!("nordoi-r4i-{}-{seq}", std::process::id()));
            fs::create_dir(&dir).expect("create new unique disposable test directory");
            Self {
                dir: fs::canonicalize(dir).expect("canonicalize disposable directory"),
            }
        }
        pub fn app(&self) -> PathBuf {
            self.dir.join("R4IParent.app")
        }
        pub fn contents(&self) -> PathBuf {
            self.app().join("Contents")
        }
        pub fn parent(&self) -> PathBuf {
            self.contents().join("MacOS/r4i-parent")
        }
        pub fn inherited_helper(&self) -> PathBuf {
            self.contents().join("Helpers/r4i-helper")
        }
        pub fn control(&self) -> PathBuf {
            self.dir.join("r4i-unsigned-entitlement-control")
        }
        pub fn sentinel(&self) -> PathBuf {
            self.dir.join("parent-owned-sentinel.txt")
        }
        pub fn creation(&self) -> PathBuf {
            self.dir.join("must-not-create.txt")
        }
        fn source(&self, name: &str) -> PathBuf {
            self.dir.join(name)
        }
        pub fn prepare(&self) {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.sentinel())
                .expect("fresh owned sentinel");
            f.write_all(b"R4I_PARENT_ONLY\n").unwrap();

            let contents = self.contents();
            fs::create_dir_all(contents.join("MacOS")).unwrap();
            fs::create_dir_all(contents.join("Helpers")).unwrap();
            let info = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
                "<plist version=\"1.0\"><dict>",
                "<key>CFBundleIdentifier</key><string>org.eddarsglobal.nordoi.r4i.parent</string>",
                "<key>CFBundleExecutable</key><string>r4i-parent</string>",
                "<key>CFBundlePackageType</key><string>APPL</string>",
                "<key>CFBundleVersion</key><string>1</string>",
                "<key>CFBundleShortVersionString</key><string>0.1</string>",
                "<key>LSBackgroundOnly</key><true/>",
                "</dict></plist>"
            );
            fs::write(contents.join("Info.plist"), info).unwrap();
            fs::write(self.source("helper.c"), HELPER_SOURCE).unwrap();
            fs::write(self.source("parent.c"), PARENT_SOURCE).unwrap();
            fs::write(self.source("app.entitlements"), APP_ENTITLEMENTS).unwrap();
            fs::write(self.source("helper.entitlements"), HELPER_ENTITLEMENTS).unwrap();

            Self::compile(&self.source("helper.c"), &self.control());
            fs::copy(self.control(), self.inherited_helper()).unwrap();
            Self::compile(&self.source("parent.c"), &self.parent());

            // Sign nested code BEFORE signing the containing bundle.
            Self::sign(&self.control(), None, "org.eddarsglobal.nordoi.r4i.control");
            Self::sign(
                &self.inherited_helper(),
                Some(&self.source("helper.entitlements")),
                "org.eddarsglobal.nordoi.r4i.helper",
            );
            Self::sign(
                &self.app(),
                Some(&self.source("app.entitlements")),
                "org.eddarsglobal.nordoi.r4i.parent",
            );
            self.check_signing();
            self.verify_fixture();
        }
        fn compile(source: &Path, target: &Path) {
            let build = Command::new("/usr/bin/cc")
                .args(["-std=c11", "-O2", "-x", "c"])
                .arg(source)
                .arg("-o")
                .arg(target)
                .output()
                .expect("host C compiler is needed for test fixture");
            assert!(
                build.status.success(),
                "compilation of fixed test program: {}",
                String::from_utf8_lossy(&build.stderr)
            );
        }
        fn sign(target: &Path, entitlement: Option<&Path>, id: &str) {
            let mut cmd = Command::new("/usr/bin/codesign");
            cmd.args(["--force", "--sign", "-", "--identifier", id]);
            if let Some(entitlement) = entitlement {
                cmd.arg("--entitlements").arg(entitlement);
            }
            let result = cmd.arg(target).output().expect("system codesign");
            assert!(
                result.status.success(),
                "ad-hoc signing must succeed: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        fn verified(path: &Path) -> bool {
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict", "--verbose=2"])
                .arg(path)
                .output()
                .expect("system codesign verify")
                .status
                .success()
        }
        fn keys(path: &Path) -> String {
            let res = Command::new("/usr/bin/codesign")
                .args(["--display", "--entitlements", ":-"])
                .arg(path)
                .output()
                .expect("inspect actual signed entitlements");
            assert!(res.status.success(), "signed entitlements must be inspectable");
            String::from_utf8_lossy(&res.stdout).into_owned()
        }
        pub fn check_signing(&self) {
            assert!(Self::verified(&self.app()), "parent bundle signature");
            assert!(Self::verified(&self.parent()), "signed main Mach-O");
            assert!(Self::verified(&self.inherited_helper()), "signed child Mach-O");
            assert!(Self::verified(&self.control()), "signed unsandboxed control");
            let app = Self::keys(&self.app());
            let helper = Self::keys(&self.inherited_helper());
            let control = Self::keys(&self.control());
            assert!(
                app.contains("<key>com.apple.security.app-sandbox</key>")
                    && !app.contains("<key>com.apple.security.inherit</key>"),
                "parent must start a sandbox: {app}"
            );
            assert_eq!(
                app.matches("<key>").count(),
                1,
                "parent must have exactly one entitlement"
            );
            assert!(
                helper.contains("<key>com.apple.security.app-sandbox</key>")
                    && helper.contains("<key>com.apple.security.inherit</key>"),
                "child must inherit App Sandbox: {helper}"
            );
            assert_eq!(
                helper.matches("<key>").count(),
                2,
                "child must have exactly two entitlements"
            );
            assert!(
                !control.contains("com.apple.security.app-sandbox")
                    && !control.contains("com.apple.security.inherit"),
                "control must have neither sandbox entitlement"
            );
        }
        pub fn verify_fixture(&self) {
            assert_eq!(fs::read(self.sentinel()).unwrap(), b"R4I_PARENT_ONLY\n");
            assert!(!self.creation().exists());
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("remove only disposable test directory");
        }
    }

    pub struct Receipt {
        pub success: bool,
        pub timeout: bool,
        pub stdout: String,
        pub stderr: String,
    }
    fn reap(child: &mut Child) -> (bool, bool) {
        let start = Instant::now();
        loop {
            if let Some(exit) = child.try_wait().expect("test subprocess status") {
                return (exit.success(), false);
            }
            if start.elapsed() > Duration::from_secs(15) {
                let _ = child.kill();
                let _ = child.wait().expect("reap test subprocess");
                return (false, true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn run_bin(bin: &Path, args: &[&Path]) -> Receipt {
        let mut child = Command::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("launch only fixed signed research fixture");
        let (success, timeout) = reap(&mut child);
        let mut stdout = String::new();
        let mut stderr = String::new();
        child
            .stdout
            .take()
            .expect("stdout pipe")
            .take(MAX_STDOUT + 1)
            .read_to_string(&mut stdout)
            .expect("bounded UTF-8 stdout");
        child
            .stderr
            .take()
            .expect("stderr pipe")
            .take(MAX_STDOUT + 1)
            .read_to_string(&mut stderr)
            .expect("bounded UTF-8 stderr");
        Receipt {
            success,
            timeout,
            stdout,
            stderr,
        }
    }
    pub fn control(lab: &Lab) -> Receipt {
        run_bin(&lab.control(), &[&lab.sentinel(), &lab.creation()])
    }
    pub fn parent_and_child(lab: &Lab) -> Receipt {
        run_bin(
            &lab.parent(),
            &[&lab.sentinel(), &lab.creation(), &lab.inherited_helper()],
        )
    }
    pub fn expect_control(w: &Receipt) {
        assert!(w.success && !w.timeout, "same-UID baseline must succeed");
        assert_eq!(
            w.stdout,
            "R4I|HELPER|READ=1|WRITE=1|CREATE=1\n",
            "unrestricted control must truly access all three file operations"
        );
    }
    pub fn expect_inherited(w: &Receipt) {
        assert!(
            w.success && !w.timeout,
            "actual sandbox parent and inherited child must exit cleanly: {}",
            w.stderr
        );
        assert!(
            w.stdout.len() as u64 <= MAX_STDOUT,
            "OS receipt size must remain bounded"
        );
        assert_eq!(
            w.stdout,
            "R4I|PARENT|READ=0|WRITE=0|CREATE=0\nR4I|HELPER|READ=0|WRITE=0|CREATE=0\n",
            "both actual OS processes must be blocked from direct file operations"
        );
    }
    pub fn exercise() {
        let lab = Lab::new();
        lab.prepare();
        expect_control(&control(&lab));
        lab.verify_fixture();
        expect_inherited(&parent_and_child(&lab));
        lab.verify_fixture();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_inherited_helper_denied_after_unsigned_entitlement_control() {
    macos::exercise();
}

#[cfg(target_os = "macos")]
#[test]
fn macos_parent_and_signed_embedded_child_both_denied_real_file_access() {
    macos::exercise();
}

#[cfg(target_os = "macos")]
#[test]
fn macos_real_signed_parent_and_child_have_exact_entitlement_sets() {
    let lab = macos::Lab::new();
    lab.prepare();
    lab.check_signing();
    lab.verify_fixture();
}

#[cfg(target_os = "macos")]
#[test]
fn macos_parent_remains_separate_from_unsandboxed_rust_test_driver() {
    use std::fs::OpenOptions;
    use std::io::Write;
    let lab = macos::Lab::new();
    lab.prepare();
    macos::expect_inherited(&macos::parent_and_child(&lab));
    let mut file = OpenOptions::new().append(true).open(lab.sentinel()).unwrap();
    file.write_all(b"PARENT_DRIVER_CAN_APPEND\n").unwrap();
    assert_eq!(
        std::fs::read(lab.sentinel()).unwrap(),
        b"R4I_PARENT_ONLY\nPARENT_DRIVER_CAN_APPEND\n"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn macos_repeated_inherited_helper_executions_remain_denied() {
    for _ in 0..2 {
        macos::exercise();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_tampered_signed_child_cannot_pass_verification() {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    let lab = macos::Lab::new();
    lab.prepare();
    let data = std::fs::read(lab.inherited_helper()).unwrap();
    let target = lab.dir.join("tampered-child");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .unwrap();
    file.write_all(&data).unwrap();
    file.seek(SeekFrom::Start(256)).unwrap();
    file.write_all(&[data[256] ^ 0x20]).unwrap();
    drop(file);
    let check = std::process::Command::new("/usr/bin/codesign")
        .args(["--verify", "--strict"])
        .arg(target)
        .output()
        .unwrap();
    assert!(!check.status.success(), "tampered helper must not verify");
}

#[cfg(target_os = "macos")]
#[test]
fn macos_forged_or_duplicate_inherited_report_fails_closed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for forged in [
        "R4I|PARENT|READ=0|WRITE=0|CREATE=0\nR4I|HELPER|READ=1|WRITE=0|CREATE=0\n",
        "R4I|PARENT|READ=0|WRITE=0|CREATE=0\nR4I|HELPER|READ=0|WRITE=0|CREATE=0\nR4I|HELPER|READ=0|WRITE=0|CREATE=0\n",
        "R4I|PARENT|READ=0|WRITE=0|CREATE=0\n",
    ] {
        let bad = macos::Receipt {
            success: true,
            timeout: false,
            stdout: forged.into(),
            stderr: String::new(),
        };
        assert!(catch_unwind(AssertUnwindSafe(|| macos::expect_inherited(&bad))).is_err());
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_os_do_not_claim_macos_entitlement_inheritance() {
    assert_ne!(std::env::consts::OS, "macos");
}

#[test]
fn r02_external_io_grant_remains_unsupported() {
    let (mut model, permit) = Model::bootstrap(
        0x49,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let scope = model.root();
    assert_eq!(
        model.host_grant(&permit, scope, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
