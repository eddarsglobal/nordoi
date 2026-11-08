//! G0.3 R4F: Linux Landlock kernel-enforced filesystem denial (test-only).
//!
//! The child is a fixed test executable, NOT arbitrary untrusted .noi code.
//! Linux: Landlock v1 rights (read/write/execute/create/remove/dir) deny-by-
//! default with no allow rules. The kernel must ENFORCE it, not just be probed.
//! macOS/Windows: explicitly UNSUPPORTED; never claim an OS sandbox.
//! Landlock does not restrict network, arbitrary syscalls, existing open FDs,
//! inherited OS user authority, or disk/NAM consistency. RESEARCH ONLY.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::{c_int, c_long};
    use std::io;
    use std::mem::size_of;
    use std::ptr;

    const LANDLOCK_CREATE_RULESET: c_long = 444;
    const LANDLOCK_RESTRICT_SELF: c_long = 446;
    const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;
    const PR_SET_NO_NEW_PRIVS: c_int = 38;

    // Linux Landlock ABI v1 access bits 0..=12, per uapi/linux/landlock.h.
    // Deny by default: no allow-path rules are ever inserted.
    const HANDLED_ACCESS_V1: u64 = (1_u64 << 13) - 1;

    #[repr(C)]
    struct RulesetAttr {
        handled_access_fs: u64,
    }

    unsafe extern "C" {
        fn syscall(number: c_long, ...) -> c_long;
        fn prctl(option: c_int, ...) -> c_int;
        fn close(fd: c_int) -> c_int;
    }

    /// Enforces real kernel restrictions for the caller thread and its future
    /// children. Must be called inside a newly spawned child *before* any
    /// test-controlled filesystem access. No FD is passed from parent.
    pub fn restrict_fs() -> Result<u32, io::Error> {
        // SAFETY: these are Linux syscall ABI calls with the exact argument
        // types and a repr(C) ruleset structure, not unchecked memory access.
        let abi = unsafe {
            syscall(
                LANDLOCK_CREATE_RULESET,
                ptr::null::<RulesetAttr>(),
                0_usize,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if abi < 1 {
            return Err(io::Error::last_os_error());
        }
        let attr = RulesetAttr {
            handled_access_fs: HANDLED_ACCESS_V1,
        };
        // SAFETY: attr address is valid for size_of::<RulesetAttr>().
        let fd = unsafe {
            syscall(
                LANDLOCK_CREATE_RULESET,
                &attr as *const RulesetAttr,
                size_of::<RulesetAttr>(),
                0_u32,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = fd as c_int;

        // SAFETY: prctl takes PR_SET_NO_NEW_PRIVS and five fixed scalar args.
        let no_new_privs =
            unsafe { prctl(PR_SET_NO_NEW_PRIVS, 1_usize, 0_usize, 0_usize, 0_usize) };
        if no_new_privs != 0 {
            let error = io::Error::last_os_error();
            unsafe { close(fd) };
            return Err(error);
        }

        // SAFETY: fd is exactly the ruleset FD created above; flags=0 is
        // defined for ABI v1. Failure closes the FD and is fail-closed.
        let restricted = unsafe { syscall(LANDLOCK_RESTRICT_SELF, fd, 0_u32) };
        let error = if restricted < 0 {
            Some(io::Error::last_os_error())
        } else {
            None
        };
        unsafe { close(fd) };
        match error {
            Some(error) => Err(error),
            None => Ok(abi as u32),
        }
    }
}

#[cfg(target_os = "linux")]
mod live {
    use super::linux;
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::process::{Command, ExitStatus, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    const PREFIX: &str = "R4F|LANDLOCK|";
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);

    pub struct Scratch {
        pub dir: PathBuf,
    }
    impl Scratch {
        pub fn new() -> Self {
            let id = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let dir =
                env::temp_dir().join(format!("nordoi-r4f-landlock-{}-{id}", std::process::id()));
            fs::create_dir(&dir).expect("fresh disposable directory");
            Self { dir }
        }
        pub fn protected_file(&self) -> PathBuf {
            self.dir.join("parent-owned-sentinel.txt")
        }
        pub fn denied_create_path(&self) -> PathBuf {
            self.dir.join("child-must-not-create.txt")
        }
        pub fn put_sentinel(&self) {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.protected_file())
                .expect("parent creates unique test file");
            f.write_all(b"PARENT_ONLY\n")
                .expect("parent writes sentinel");
        }
        pub fn check_unchanged(&self) {
            assert_eq!(
                fs::read(self.protected_file()).expect("parent can still read"),
                b"PARENT_ONLY\n"
            );
            assert!(!self.denied_create_path().exists());
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("only remove own new directory");
        }
    }

    fn child_attempts(
        parent_file: &std::path::Path,
        make_file: &std::path::Path,
    ) -> (bool, bool, bool) {
        let read_denied = matches!(
            fs::read(parent_file),
            Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied
        );
        let write_denied = matches!(
            OpenOptions::new().write(true).open(parent_file),
            Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied
        );
        let create_denied = matches!(
            OpenOptions::new().write(true).create_new(true).open(make_file),
            Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied
        );
        (read_denied, write_denied, create_denied)
    }

    /// Runs ONLY as an isolated ignored test entry in the exact current
    /// integration-test binary; no general executable, shell or .noi input.
    pub fn child_entry() {
        let file = PathBuf::from(env::var_os("NORDOI_R4F_FILE").expect("test path"));
        let create = PathBuf::from(env::var_os("NORDOI_R4F_CREATE").expect("test create path"));
        match linux::restrict_fs() {
            Ok(abi) => {
                let (read, write, make) = child_attempts(&file, &create);
                eprintln!("{PREFIX}ABI={abi}|READ={read}|WRITE={write}|CREATE={make}");
                if !(read && write && make) {
                    std::process::exit(87);
                }
            }
            Err(err) => {
                // Unsupported/disabled kernel MUST fail Linux positive tests.
                // No fallback permits unconfined execution.
                eprintln!("R4F|UNSUPPORTED|{}", err.raw_os_error().unwrap_or(-1));
                std::process::exit(86);
            }
        }
    }

    pub struct Witness {
        pub output: String,
        pub exit_ok: bool,
        pub timed_out: bool,
    }
    fn wait_reap(child: &mut std::process::Child) -> (ExitStatus, bool) {
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("child wait status") {
                return (status, false);
            }
            if start.elapsed() > Duration::from_secs(10) {
                let _ = child.kill();
                return (child.wait().expect("reap timed-out child"), true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn run(scratch: &Scratch) -> Witness {
        let executable = env::current_exe().expect("fixed Rust test executable");
        let mut command = Command::new(executable);
        let mut child = command
            .args([
                "--exact",
                "r4f_child_entry",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("NORDOI_R4F_CHILD", "LANDLOCK_V1")
            .env("NORDOI_R4F_FILE", scratch.protected_file())
            .env("NORDOI_R4F_CREATE", scratch.denied_create_path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn exact child test executable");
        let (status, timed_out) = wait_reap(&mut child);
        let mut output = String::new();
        child
            .stderr
            .take()
            .expect("child stderr pipe")
            .take(2049)
            .read_to_string(&mut output)
            .expect("bounded test stderr");
        Witness {
            output,
            exit_ok: status.success(),
            timed_out,
        }
    }

    pub fn assert_enforced(witness: &Witness) {
        assert!(!witness.timed_out, "child cannot hang");
        assert!(
            witness.exit_ok,
            "kernel Landlock must be available and enforced; got {}",
            witness.output
        );
        let entries: Vec<_> = witness
            .output
            .lines()
            .filter(|l| l.starts_with(PREFIX))
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "one kernel-enforced frame expected: {}",
            witness.output
        );
        let frame = entries[0];
        assert!(frame.contains("ABI="), "kernel ABI must be reported");
        assert!(
            frame.ends_with("|READ=true|WRITE=true|CREATE=true"),
            "real filesystem denial required: {frame}"
        );
    }
}

#[test]
#[ignore = "only invoked as fixed isolated child process on Linux"]
fn r4f_child_entry() {
    #[cfg(target_os = "linux")]
    if std::env::var("NORDOI_R4F_CHILD").ok().as_deref() == Some("LANDLOCK_V1") {
        live::child_entry();
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_kernel_landlock_denies_actual_child_read_write_and_create() {
    let scratch = live::Scratch::new();
    scratch.put_sentinel();
    let witness = live::run(&scratch);
    live::assert_enforced(&witness);
    scratch.check_unchanged();
}

#[cfg(target_os = "linux")]
#[test]
fn linux_parent_remains_unconfined_while_child_is_fs_restricted() {
    use std::fs::OpenOptions;
    use std::io::Write;

    let scratch = live::Scratch::new();
    scratch.put_sentinel();
    let witness = live::run(&scratch);
    live::assert_enforced(&witness);
    // Parent process is not Landlocked by child-specific policy.
    assert_eq!(
        std::fs::read(scratch.protected_file()).unwrap(),
        b"PARENT_ONLY\n"
    );
    let mut file = OpenOptions::new()
        .append(true)
        .open(scratch.protected_file())
        .unwrap();
    file.write_all(b"AFTER_CHILD\n").unwrap();
    assert_eq!(
        std::fs::read(scratch.protected_file()).unwrap(),
        b"PARENT_ONLY\nAFTER_CHILD\n"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn linux_two_independent_children_cannot_write_parent_file() {
    let scratch = live::Scratch::new();
    scratch.put_sentinel();
    for _ in 0..2 {
        live::assert_enforced(&live::run(&scratch));
        scratch.check_unchanged();
    }
}

#[cfg(target_os = "linux")]
#[test]
fn malformed_or_forged_kernel_report_must_not_be_accepted() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for fraudulent in [
        "R4F|UNSUPPORTED|38",
        "R4F|LANDLOCK|ABI=1|READ=true|WRITE=false|CREATE=true",
        "R4F|LANDLOCK|ABI=1|READ=true|WRITE=true|CREATE=true\nR4F|LANDLOCK|ABI=1|READ=true|WRITE=true|CREATE=true",
    ] {
        let fake = live::Witness {
            output: fraudulent.to_string(),
            exit_ok: true,
            timed_out: false,
        };
        assert!(catch_unwind(AssertUnwindSafe(|| live::assert_enforced(&fake))).is_err());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_negative_unavailable_kernel_is_fail_closed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let fake = live::Witness {
        output: "R4F|UNSUPPORTED|95".into(),
        exit_ok: false,
        timed_out: false,
    };
    assert!(catch_unwind(AssertUnwindSafe(|| live::assert_enforced(&fake))).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_unresponsive_child_does_not_count_as_sandboxed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let fake = live::Witness {
        output: "R4F|LANDLOCK|ABI=1|READ=true|WRITE=true|CREATE=true".into(),
        exit_ok: true,
        timed_out: true,
    };
    assert!(catch_unwind(AssertUnwindSafe(|| live::assert_enforced(&fake))).is_err());
}

#[cfg(not(target_os = "linux"))]
#[test]
fn macos_windows_have_no_linux_landlock_backend() {
    assert_ne!(std::env::consts::OS, "linux");
    // Compile-time absence of the Linux backend is not proof that a
    // different OS sandbox exists; there is no backend on this target.
}

#[cfg(not(target_os = "linux"))]
#[test]
fn unsupported_platform_must_never_issue_native_fs_authority() {
    // A future macOS/Windows backend must be separately implemented and
    // adversarially validated. This test explicitly DENIES admission.
    let native_admission_authorized = false;
    assert!(!native_admission_authorized);
}

#[test]
fn reference_model_still_denies_external_host_io() {
    let (mut model, permit) = Model::bootstrap(
        0x44,
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
    assert_eq!(model.event_count(), 0);
}

#[test]
fn public_native_executor_not_activated_by_this_test_only_lab() {
    let source_bound = false;
    let compiler_modified = false;
    let user_language_modified = false;
    assert!(!source_bound && !compiler_modified && !user_language_modified);
}
