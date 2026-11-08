//! G0.3 R4G — macOS Seatbelt filesystem-denial experiment (test only).
//!
//! Uses Apple's DEPRECATED /usr/bin/sandbox-exec only for a bounded research
//! witness. It is NOT an approved production isolation mechanism. A future
//! signed App Sandbox helper is a separate design with separate evidence.
//! Real parent-controlled disposable paths; actual same-user unsandboxed
//! control first, then kernel-denied child read/write/open/create.
//! No public .noi executor, no trusted arbitrary code, no general sandbox.
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

    const CORRELATION: &str = "r4g-owner-controlled-test-v1";
    const FRAME_PREFIX: &str = "R4G|PROBE|";
    const MAX_CHILD_STDERR: u64 = 4096;
    static NEXT: AtomicU64 = AtomicU64::new(1);

    pub struct Scratch {
        dir: PathBuf,
    }
    impl Scratch {
        pub fn new() -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let prefix = format!("nordoi-r4g-{}-{id}", std::process::id());
            let dir = env::temp_dir().join(prefix);
            fs::create_dir(&dir).expect("create new disposable owner test directory");
            let dir = fs::canonicalize(dir).expect("canonicalize owner test path");
            Self { dir }
        }
        pub fn sentinel(&self) -> PathBuf {
            self.dir.join("owned-sentinel.txt")
        }
        pub fn creation_target(&self) -> PathBuf {
            self.dir.join("must-not-exist.txt")
        }
        pub fn prepare(&self) {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.sentinel())
                .expect("create parent-owned sentinel");
            file.write_all(b"R4G_PARENT_ONLY\n")
                .expect("write sentinel");
        }
        pub fn verify(&self) {
            assert_eq!(fs::read(self.sentinel()).unwrap(), b"R4G_PARENT_ONLY\n");
            assert!(!self.creation_target().exists());
        }
        fn profile(&self) -> String {
            let dir = self.dir.to_str().expect("UTF-8 temp fixture path");
            assert!(
                !dir.contains('"') && !dir.contains('\\') && !dir.contains('\n'),
                "fail closed on unexpected profile path characters"
            );
            // Research-only per-path denial. All other access is deliberately
            // allowed; this is NOT a deny-by-default production sandbox.
            format!(
                "(version 1)(allow default)(deny file-read-data (subpath \"{dir}\"))(deny file-write* (subpath \"{dir}\"))"
            )
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("remove only fresh test fixture");
        }
    }

    fn observe(sentinel: &Path, creation: &Path) -> (bool, bool, bool) {
        let read = matches!(
            fs::read(sentinel),
            Ok(content) if content == b"R4G_PARENT_ONLY\n"
        );
        // Open for write only; never write to protected sentinel in child.
        let write = OpenOptions::new().write(true).open(sentinel).is_ok();
        let create = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(creation)
        {
            Ok(file) => {
                drop(file);
                fs::remove_file(creation).expect("control-only creation cleanup");
                true
            }
            Err(_) => false,
        };
        (read, write, create)
    }

    /// The exact Rust integration-test executable serves as the known child.
    /// No general .noi program, kernel object, file FD or host grant is passed.
    pub fn child_entry() {
        let sentinel = PathBuf::from(env::var_os("NORDOI_R4G_SENTINEL").expect("test sentinel"));
        let create = PathBuf::from(env::var_os("NORDOI_R4G_CREATE").expect("test create"));
        let (read, write, creation) = observe(&sentinel, &create);
        eprintln!("{FRAME_PREFIX}{CORRELATION}|READ={read}|WRITE={write}|CREATE={creation}");
    }

    #[derive(Debug)]
    pub struct Witness {
        pub exit_ok: bool,
        pub timed_out: bool,
        pub truncated: bool,
        pub transcript: String,
    }

    fn reap(child: &mut Child) -> (bool, bool) {
        let start = Instant::now();
        loop {
            if let Some(exit) = child.try_wait().expect("test subprocess status") {
                return (exit.success(), false);
            }
            if start.elapsed() >= Duration::from_secs(12) {
                let _ = child.kill();
                let _ = child.wait().expect("must reap timed-out test child");
                return (false, true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn run(scratch: &Scratch, sandboxed: bool) -> Witness {
        let executable = env::current_exe().expect("current exact pinned test executable");
        let mut command = if sandboxed {
            let mut cmd = Command::new("/usr/bin/sandbox-exec");
            cmd.arg("-p").arg(scratch.profile()).arg(executable);
            cmd
        } else {
            Command::new(executable)
        };
        let mut child = command
            .args([
                "--exact",
                "r4g_child_entry",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("NORDOI_R4G_CHILD", "FIXED_TEST_ONLY")
            .env("NORDOI_R4G_SENTINEL", scratch.sentinel())
            .env("NORDOI_R4G_CREATE", scratch.creation_target())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start exact fixed child process; unsupported profile fails closed");
        let stderr = child.stderr.take().expect("owned stderr pipe");
        // Reader starts concurrently; OS pipe never fills for the fixed child.
        // Read is physically capped; excess data will not be accepted as PASS.
        let reader = thread::spawn(move || {
            let mut text = String::new();
            let mut capped = stderr.take(MAX_CHILD_STDERR + 1);
            let result = capped.read_to_string(&mut text);
            (result, text)
        });
        let (exit_ok, timed_out) = reap(&mut child);
        let (result, transcript) = reader.join().expect("bounded stderr reader joins");
        result.expect("child transcript must be UTF-8");
        let truncated = transcript.len() as u64 > MAX_CHILD_STDERR;
        Witness {
            exit_ok,
            timed_out,
            truncated,
            transcript,
        }
    }

    /// We assert exact measured conditions, NOT a synthetic success constant.
    pub fn expect(w: &Witness, allowed: bool) {
        assert!(w.exit_ok, "fixed child must exit cleanly: {}", w.transcript);
        assert!(
            !w.timed_out && !w.truncated,
            "child deadline/byte limit enforced"
        );
        let frames: Vec<&str> = w
            .transcript
            .lines()
            .filter(|line| line.starts_with(FRAME_PREFIX))
            .collect();
        assert_eq!(
            frames.len(),
            1,
            "one actual child observation required: {}",
            w.transcript
        );
        let expect = if allowed {
            format!("{FRAME_PREFIX}{CORRELATION}|READ=true|WRITE=true|CREATE=true")
        } else {
            format!("{FRAME_PREFIX}{CORRELATION}|READ=false|WRITE=false|CREATE=false")
        };
        assert_eq!(
            frames[0], expect,
            "real OS operation observations must match"
        );
    }

    pub fn probe_once() {
        let scratch = Scratch::new();
        scratch.prepare();
        // Control proves the same user and child can access this exact path
        // when no mandatory Seatbelt policy is installed.
        let ordinary = run(&scratch, false);
        expect(&ordinary, true);
        scratch.verify();
        // Seatbelt must deny these SAME direct syscalls, or the test FAILS.
        let restricted = run(&scratch, true);
        expect(&restricted, false);
        scratch.verify();
    }
}

#[test]
#[ignore = "invoked only by the exact R4G test process launcher"]
fn r4g_child_entry() {
    #[cfg(target_os = "macos")]
    if std::env::var("NORDOI_R4G_CHILD").ok().as_deref() == Some("FIXED_TEST_ONLY") {
        macos::child_entry();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_real_kernel_file_denial_after_unrestricted_positive_control() {
    macos::probe_once();
}

#[cfg(target_os = "macos")]
#[test]
fn macos_repeated_same_user_process_controls_and_seatbelt_denials() {
    for _ in 0..3 {
        macos::probe_once();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_parent_retains_native_file_access_after_child_sandbox() {
    use std::fs::OpenOptions;
    use std::io::Write;
    let scratch = macos::Scratch::new();
    scratch.prepare();
    macos::expect(&macos::run(&scratch, false), true);
    macos::expect(&macos::run(&scratch, true), false);
    let mut parent = OpenOptions::new()
        .append(true)
        .open(scratch.sentinel())
        .unwrap();
    parent.write_all(b"PARENT_CAN_APPEND\n").unwrap();
    assert_eq!(
        std::fs::read(scratch.sentinel()).unwrap(),
        b"R4G_PARENT_ONLY\nPARENT_CAN_APPEND\n"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn macos_forged_duplicate_timeout_and_truncated_reports_are_rejected() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for transcript in [
        "R4G|PROBE|wrong|READ=false|WRITE=false|CREATE=false",
        "R4G|PROBE|r4g-owner-controlled-test-v1|READ=false|WRITE=true|CREATE=false",
        "R4G|PROBE|r4g-owner-controlled-test-v1|READ=false|WRITE=false|CREATE=false\nR4G|PROBE|r4g-owner-controlled-test-v1|READ=false|WRITE=false|CREATE=false",
    ] {
        let fabricated = macos::Witness {
            exit_ok: true,
            timed_out: false,
            truncated: false,
            transcript: transcript.into(),
        };
        assert!(catch_unwind(AssertUnwindSafe(|| macos::expect(&fabricated, false))).is_err());
    }
    let fabricated = macos::Witness {
        exit_ok: true,
        timed_out: true,
        truncated: false,
        transcript: "R4G|PROBE|r4g-owner-controlled-test-v1|READ=false|WRITE=false|CREATE=false"
            .into(),
    };
    assert!(catch_unwind(AssertUnwindSafe(|| macos::expect(&fabricated, false))).is_err());
}

#[cfg(not(target_os = "macos"))]
#[test]
fn non_macos_must_not_claim_seatbelt_and_external_io_stays_denied() {
    assert_ne!(std::env::consts::OS, "macos");
    let (mut model, permit) = Model::bootstrap(
        0x47,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let root = model.root();
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
}

#[test]
fn resource_reference_model_does_not_grant_external_io_implicitly() {
    let (mut model, host) = Model::bootstrap(
        0x47_2026,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let scope = model.root();
    assert_eq!(
        model.host_grant(&host, scope, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
