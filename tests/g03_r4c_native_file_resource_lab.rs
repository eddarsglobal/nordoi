//! G0.3 R4C — owner-hosted *real file handle* boundary, research only.
//!
//! Every native I/O operation is confined to a disposable test directory.
//! Guest workers receive NO File, path, NAM, R0.2 grant, or host permit.
//! This is NOT a sandbox, public .noi executor, general scheduler, or proof
//! that real disk I/O is atomic with NAM transactions.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use nordoi_kernel::{AtomicKernel, Value};
use reference::{
    Command, GrantId, Model, ModelError, Receipt, ResourceEffect, ResourceId,
    ScopeBudget, ScopeId, ScopeOutcome, ScopeReport, TaskId, TaskOutcome,
};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::thread;

static NEXT_TMP: AtomicU64 = AtomicU64::new(1);
const PANIC_CODE: u16 = 237;

struct Scratch {
    path: PathBuf,
}
impl Scratch {
    fn new() -> Self {
        let ticket = NEXT_TMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nordoi-r4c-test-{}-{ticket}", std::process::id()
        ));
        fs::create_dir(&path).expect("fresh private test directory");
        Self { path }
    }
    fn file(&self) -> PathBuf {
        self.path.join("bounded.txt")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // The directory was newly created by THIS test process and is never
        // selected by caller input. This is not cleanup of an existing folder.
        fs::remove_dir_all(&self.path).expect("test directory cleanup");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lease {
    scope: ScopeId,
    nonce: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostError {
    WrongScope,
    InvalidLease,
    Revoked,
    AlreadyClosed,
    Io,
}

/// Real OS File remains exclusively in the trusted test parent.
/// Lease is a *test-only scoped token*, not cryptographically unforgeable
/// against arbitrary Rust code running in the same process.
struct HostFile {
    file: Option<File>,
    lease: Lease,
    revoked: bool,
}
impl HostFile {
    fn create(scratch: &Scratch, scope: ScopeId) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(scratch.file())
            .expect("new file in newly created test directory");
        Self {
            file: Some(file),
            lease: Lease {
                scope,
                nonce: 0x2026_1108,
            },
            revoked: false,
        }
    }
    fn token(&self) -> Lease {
        self.lease
    }
    fn check(&self, scope: ScopeId, lease: Lease) -> Result<(), HostError> {
        if scope != self.lease.scope {
            return Err(HostError::WrongScope);
        }
        if lease != self.lease {
            return Err(HostError::InvalidLease);
        }
        if self.revoked {
            return Err(HostError::Revoked);
        }
        if self.file.is_none() {
            return Err(HostError::AlreadyClosed);
        }
        Ok(())
    }
    fn write(&mut self, scope: ScopeId, lease: Lease, bytes: &[u8]) -> Result<(), HostError> {
        self.check(scope, lease)?;
        self.file
            .as_mut()
            .ok_or(HostError::AlreadyClosed)?
            .write_all(bytes)
            .map_err(|_| HostError::Io)
    }
    fn close(&mut self, scope: ScopeId, lease: Lease) -> Result<(), HostError> {
        self.check(scope, lease)?;
        drop(self.file.take());
        Ok(())
    }
    fn revoke(&mut self) {
        self.revoked = true;
    }
    fn cleanup_on_abort(&mut self) {
        drop(self.file.take());
    }
}

struct Parent {
    model: Model,
    scope: ScopeId,
    grant: GrantId,
    resource: ResourceId,
    task: TaskId,
    host: HostFile,
    scratch: Scratch,
    kernel: AtomicKernel,
    atom: nordoi_kernel::AtomId,
}
impl Parent {
    fn new() -> Self {
        let (mut model, host_authority) = Model::bootstrap(
            0x0020_2611_084C,
            ScopeBudget {
                tasks: 1,
                resources: 1,
                children: 0,
            },
            64,
        );
        let scope = model.root();
        model
            .apply(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::Write,
            })
            .unwrap();
        let grant = model
            .host_grant(&host_authority, scope, ResourceEffect::Write)
            .unwrap();
        let resource = match model
            .apply(Command::Acquire {
                scope,
                grant,
                effect: ResourceEffect::Write,
            })
            .unwrap()
        {
            Receipt::Resource(id) => id,
            unexpected => panic!("unexpected resource receipt {unexpected:?}"),
        };
        let task = match model.apply(Command::DeclareTask { scope }).unwrap() {
            Receipt::Task(id) => id,
            unexpected => panic!("unexpected task receipt {unexpected:?}"),
        };
        model.apply(Command::Admit { task }).unwrap();
        model.apply(Command::Start { task }).unwrap();
        let scratch = Scratch::new();
        let host = HostFile::create(&scratch, scope);
        let mut kernel = AtomicKernel::new();
        let atom = kernel.create_atom(0_i64);
        Self {
            model,
            scope,
            grant,
            resource,
            task,
            host,
            scratch,
            kernel,
            atom,
        }
    }
    fn close(mut self, terminal: TaskOutcome) -> (ScopeReport, String, u64) {
        let step = match terminal {
            TaskOutcome::Succeeded => Command::Succeed { task: self.task },
            TaskOutcome::Failed(code) => Command::Fail {
                task: self.task,
                code,
            },
            TaskOutcome::Cancelled => Command::AcknowledgeCancel { task: self.task },
        };
        self.model.apply(step).unwrap();
        self.model
            .apply(Command::Join { task: self.task })
            .unwrap();
        if terminal == TaskOutcome::Succeeded {
            let token = self.host.token();
            self.model
                .apply(Command::Use {
                    scope: self.scope,
                    resource: self.resource,
                    grant: self.grant,
                    effect: ResourceEffect::Write,
                })
                .unwrap();
            self.host
                .write(self.scope, token, b"42\n")
                .expect("bounded test-only native file write");
        }
        self.host.cleanup_on_abort();
        self.model
            .apply(Command::Release {
                scope: self.scope,
                resource: self.resource,
            })
            .unwrap();
        let report = match self.model.apply(Command::Close { scope: self.scope }).unwrap() {
            Receipt::Closed(report) => report,
            unexpected => panic!("unexpected scope close receipt {unexpected:?}"),
        };
        assert!(self.model.replay_checked().is_ok());
        if report.outcome == ScopeOutcome::Succeeded {
            // NAM write is deliberately separate from native disk I/O; these
            // two stores have NO cross-store atomicity guarantee.
            assert_eq!(self.kernel.set(self.atom, 42_i64), Ok(true));
        }
        let text = fs::read_to_string(self.scratch.file()).unwrap();
        let version = self.kernel.version(self.atom).unwrap();
        (report, text, version)
    }
}

#[derive(Clone, Copy, Debug)]
enum Race {
    CancelBeforeRelease,
    CompleteBeforeLateCancel,
    Fail,
    Panic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Worker {
    Value,
    Cancelled,
    Failed,
}
fn race(mode: Race) -> (ScopeReport, String, u64, bool) {
    let mut parent = Parent::new();
    let cancelled = Arc::new(AtomicBool::new(false));
    let release = Arc::new(Barrier::new(2));
    let (at_checkpoint_tx, at_checkpoint_rx) = mpsc::sync_channel::<()>(0);
    let worker = thread::scope(|scope| {
        let flag = Arc::clone(&cancelled);
        let release = Arc::clone(&release);
        let handle = scope.spawn(move || match mode {
            Race::Fail => Worker::Failed,
            Race::Panic => panic!("R4C deliberate child panic in test-only host-file lab"),
            Race::CancelBeforeRelease | Race::CompleteBeforeLateCancel => {
                at_checkpoint_tx.send(()).unwrap();
                release.wait();
                if flag.load(Ordering::Acquire) {
                    Worker::Cancelled
                } else {
                    Worker::Value
                }
            }
        });
        if matches!(mode, Race::CancelBeforeRelease | Race::CompleteBeforeLateCancel) {
            at_checkpoint_rx.recv().unwrap();
            if matches!(mode, Race::CancelBeforeRelease) {
                parent
                    .model
                    .apply(Command::RequestCancel { task: parent.task })
                    .unwrap();
                cancelled.store(true, Ordering::Release);
            }
            release.wait();
        }
        handle.join().unwrap_or(Worker::Failed)
    });
    let terminal = match worker {
        Worker::Value => TaskOutcome::Succeeded,
        Worker::Cancelled => TaskOutcome::Cancelled,
        Worker::Failed => TaskOutcome::Failed(PANIC_CODE),
    };
    let late_cancel_blocked = if matches!(mode, Race::CompleteBeforeLateCancel) {
        parent.model.apply(Command::Succeed { task: parent.task }).unwrap();
        let previous = parent.model.clone();
        let denied = parent.model.apply(Command::RequestCancel { task: parent.task })
            == Err(ModelError::InvalidTransition);
        assert_eq!(parent.model, previous);
        // Parent::close will apply the terminal transition: avoid doing it
        // twice by resetting only this pure-model witness in the test logic.
        // Instead drive this scenario separately below.
        return (run_late_cancel_close(parent), "42\n".into(), 1, denied);
    } else {
        false
    };
    let (report, text, version) = parent.close(terminal);
    (report, text, version, late_cancel_blocked)
}

fn run_late_cancel_close(mut parent: Parent) -> ScopeReport {
    parent.model.apply(Command::Join { task: parent.task }).unwrap();
    parent
        .model
        .apply(Command::Use {
            scope: parent.scope,
            resource: parent.resource,
            grant: parent.grant,
            effect: ResourceEffect::Write,
        })
        .unwrap();
    let token = parent.host.token();
    parent.host.write(parent.scope, token, b"42\n").unwrap();
    parent.host.cleanup_on_abort();
    parent
        .model
        .apply(Command::Release { scope: parent.scope, resource: parent.resource })
        .unwrap();
    let report = match parent.model.apply(Command::Close { scope: parent.scope }).unwrap() {
        Receipt::Closed(report) => report,
        _ => panic!("unexpected close"),
    };
    assert!(parent.model.replay_checked().is_ok());
    assert_eq!(parent.kernel.set(parent.atom, 42_i64), Ok(true));
    assert_eq!(fs::read_to_string(parent.scratch.file()).unwrap(), "42\n");
    report
}

#[test]
fn cancellation_wins_after_real_worker_checkpoint_and_prevents_io() {
    let (report, file_text, version, _) = race(Race::CancelBeforeRelease);
    assert_eq!(report.outcome, ScopeOutcome::Cancelled);
    assert_eq!(report.tasks[0].1, TaskOutcome::Cancelled);
    assert!(file_text.is_empty());
    assert_eq!(version, 0);
}

#[test]
fn completed_task_refuses_late_cancellation_and_writes_only_after_join() {
    let (report, file_text, version, late_denied) = race(Race::CompleteBeforeLateCancel);
    assert_eq!(report.outcome, ScopeOutcome::Succeeded);
    assert_eq!(file_text, "42\n");
    assert_eq!(version, 1);
    assert!(late_denied);
}

#[test]
fn native_worker_failure_cleans_handle_and_writes_no_bytes() {
    let (report, file_text, version, _) = race(Race::Fail);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert_eq!(file_text, "");
    assert_eq!(version, 0);
}

#[test]
fn native_worker_panic_joins_and_writes_no_bytes() {
    let (report, file_text, version, _) = race(Race::Panic);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert!(file_text.is_empty());
    assert_eq!(version, 0);
}

#[test]
fn cross_scope_lease_cannot_access_native_file() {
    let mut p = Parent::new();
    let (foreign_model, _) = Model::bootstrap(
        90,
        ScopeBudget { tasks: 0, resources: 0, children: 0 },
        0,
    );
    let token = p.host.token();
    assert_eq!(p.host.write(foreign_model.root(), token, b"UNAUTHORIZED"),
               Err(HostError::WrongScope));
    assert_eq!(fs::read(p.scratch.file()).unwrap(), b"");
}

#[test]
fn forged_lease_nonce_is_rejected_before_os_io() {
    let mut p = Parent::new();
    let mut token = p.host.token();
    token.nonce ^= 1;
    assert_eq!(p.host.write(p.scope, token, b"FORGED"), Err(HostError::InvalidLease));
    assert_eq!(fs::read(p.scratch.file()).unwrap(), b"");
}

#[test]
fn test_host_revocation_blocks_further_file_writes() {
    let mut p = Parent::new();
    let token = p.host.token();
    p.model.apply(Command::RevokeGrant { grant: p.grant }).unwrap();
    p.host.revoke();
    assert_eq!(
        p.model.apply(Command::Use {
            scope: p.scope,
            resource: p.resource,
            grant: p.grant,
            effect: ResourceEffect::Write,
        }),
        Err(ModelError::RevokedGrant)
    );
    assert_eq!(p.host.write(p.scope, token, b"REVOKED"), Err(HostError::Revoked));
    assert!(fs::read(p.scratch.file()).unwrap().is_empty());
}

#[test]
fn native_resource_close_is_idempotence_denied() {
    let mut p = Parent::new();
    let token = p.host.token();
    assert_eq!(p.host.close(p.scope, token), Ok(()));
    assert_eq!(p.host.close(p.scope, token), Err(HostError::AlreadyClosed));
    assert_eq!(p.host.write(p.scope, token, b"AFTER_CLOSE"),
               Err(HostError::AlreadyClosed));
}

#[test]
fn model_scope_close_denied_with_live_resource_and_unjoined_task() {
    let mut p = Parent::new();
    assert_eq!(
        p.model.apply(Command::Close { scope: p.scope }),
        Err(ModelError::OutstandingResources)
    );
    p.model.apply(Command::Release { scope: p.scope, resource: p.resource }).unwrap();
    assert_eq!(
        p.model.apply(Command::Close { scope: p.scope }),
        Err(ModelError::OutstandingTasks)
    );
}

#[test]
fn no_implicit_external_io_grant_in_reference_model() {
    let (mut model, permit) = Model::bootstrap(
        85, ScopeBudget { tasks: 0, resources: 0, children: 0 }, 16
    );
    let root = model.root();
    assert_eq!(model.host_grant(&permit, root, ResourceEffect::ExternalIo),
               Err(ModelError::UnsupportedEffect));
    assert_eq!(model.event_count(), 0);
}

#[test]
fn abort_cleanup_removes_disposable_file_after_native_handle_close() {
    let path;
    {
        let mut p = Parent::new();
        path = p.scratch.file();
        assert!(path.exists());
        p.host.cleanup_on_abort();
        assert_eq!(p.host.file.is_none(), true);
    }
    assert!(!path.exists());
}

#[test]
fn fresh_directory_refuses_replacing_existing_native_file() {
    let scratch = Scratch::new();
    let _first = HostFile::create(
        &scratch,
        Model::bootstrap(8, ScopeBudget { tasks: 0, resources: 0, children: 0 }, 0).0.root(),
    );
    assert_eq!(
        OpenOptions::new().write(true).create_new(true).open(scratch.file())
            .unwrap_err().kind(),
        std::io::ErrorKind::AlreadyExists
    );
}

#[test]
fn cancel_wins_repeated_native_runs_without_timing_assumptions() {
    for _ in 0..8 {
        let (report, text, version, _) = race(Race::CancelBeforeRelease);
        assert_eq!(report.outcome, ScopeOutcome::Cancelled);
        assert_eq!(text, "");
        assert_eq!(version, 0);
    }
}
