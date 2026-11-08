//! G0.3 R4D: one-lock native host-adapter witness (research/test only).
//! Trusted in-process Rust only. Ticket is NOT a nonforgeable OS capability.
//! No arbitrary .noi program, general scheduler, ExternalIo grant or sandbox.
//! Real disk writes may persist when NAM later rejects a transaction.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use nordoi_kernel::{AtomId, AtomicError, AtomicKernel, Value};
use reference::{
    Command, GrantId, Model, ModelError, Receipt, ResourceEffect, ResourceId, ScopeBudget, ScopeId,
    ScopeOutcome, ScopeReport, TaskId,
};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Barrier, Mutex};
use std::thread;

static NEXT: AtomicU64 = AtomicU64::new(1);
const PANIC_CODE: u16 = 221;

struct Scratch {
    dir: PathBuf,
}
impl Scratch {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("nordoi-r4d-{}-{id}", std::process::id()));
        fs::create_dir(&dir).expect("new private fixture directory");
        Self { dir }
    }
    fn file(&self) -> PathBuf {
        self.dir.join("owned.txt")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("remove only newly created fixture");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ticket {
    scope: ScopeId,
    epoch: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Completion {
    Success,
    Failure,
    Cancelled,
}
#[derive(Debug, PartialEq, Eq)]
enum Denial {
    WrongScope,
    NoTicket,
    Stale,
    Revoked,
    Closed,
    Quota,
    Incomplete,
    Model(ModelError),
    Kernel(AtomicError),
    Io,
}
struct State {
    model: Model,
    scope: ScopeId,
    grant: GrantId,
    resource: ResourceId,
    task: TaskId,
    file: Option<File>,
    active: Option<Ticket>,
    epoch: u64,
    revoked: bool,
    cancel_requested: bool,
    sealed: bool,
    final_outcome: Option<ScopeOutcome>,
    used: usize,
    max_bytes: usize,
}

/// All model authority checks, real writes, revokes, and file close share
/// ONE mutex. This prevents in-process revoke from interleaving between the
/// reference model's Use event and the actual File::write_all call.
/// This does NOT stop malicious same-process code calling std::fs itself.
struct HostAdapter {
    state: Mutex<State>,
    // File in state is dropped before this scratch directory is removed.
    scratch: Scratch,
}
impl HostAdapter {
    fn new(max_bytes: usize) -> Self {
        let (mut model, host) = Model::bootstrap(
            0x2026_1108,
            ScopeBudget {
                tasks: 1,
                resources: 1,
                children: 0,
            },
            128,
        );
        let scope = model.root();
        model
            .apply(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::Write,
            })
            .unwrap();
        let grant = model
            .host_grant(&host, scope, ResourceEffect::Write)
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
            x => panic!("unexpected modeled resource {x:?}"),
        };
        let task = match model.apply(Command::DeclareTask { scope }).unwrap() {
            Receipt::Task(id) => id,
            x => panic!("unexpected modeled task {x:?}"),
        };
        model.apply(Command::Admit { task }).unwrap();
        model.apply(Command::Start { task }).unwrap();
        let scratch = Scratch::new();
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(scratch.file())
            .expect("new file inside disposable directory");
        Self {
            state: Mutex::new(State {
                model,
                scope,
                grant,
                resource,
                task,
                file: Some(file),
                active: None,
                epoch: 0,
                revoked: false,
                cancel_requested: false,
                sealed: false,
                final_outcome: None,
                used: 0,
                max_bytes,
            }),
            scratch,
        }
    }

    fn scope(&self) -> ScopeId {
        self.state.lock().unwrap().scope
    }
    /// Trusted host-side issuance only, not authentication of hostile Rust.
    fn issue(&self, scope: ScopeId) -> Result<Ticket, Denial> {
        let mut s = self.state.lock().unwrap();
        if scope != s.scope {
            return Err(Denial::WrongScope);
        }
        if s.sealed {
            return Err(Denial::Closed);
        }
        if s.revoked {
            return Err(Denial::Revoked);
        }
        s.epoch = s.epoch.checked_add(1).ok_or(Denial::Quota)?;
        let ticket = Ticket {
            scope,
            epoch: s.epoch,
        };
        s.active = Some(ticket);
        Ok(ticket)
    }

    fn write(&self, scope: ScopeId, ticket: Ticket, bytes: &[u8]) -> Result<usize, Denial> {
        let mut s = self.state.lock().unwrap();
        if scope != s.scope {
            return Err(Denial::WrongScope);
        }
        if s.sealed {
            return Err(Denial::Closed);
        }
        if s.revoked {
            return Err(Denial::Revoked);
        }
        if s.active.is_none() {
            return Err(Denial::NoTicket);
        }
        if s.active != Some(ticket) {
            return Err(Denial::Stale);
        }
        let next = s.used.checked_add(bytes.len()).ok_or(Denial::Quota)?;
        if next > s.max_bytes {
            return Err(Denial::Quota);
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        let (scope, resource, grant) = (s.scope, s.resource, s.grant);
        s.model
            .apply(Command::Use {
                scope,
                resource,
                grant,
                effect: ResourceEffect::Write,
            })
            .map_err(Denial::Model)?;
        s.file
            .as_mut()
            .ok_or(Denial::Closed)?
            .write_all(bytes)
            .map_err(|_| Denial::Io)?;
        s.used = next;
        Ok(bytes.len())
    }

    fn revoke(&self) -> Result<(), Denial> {
        let mut s = self.state.lock().unwrap();
        if s.sealed {
            return Err(Denial::Closed);
        }
        if s.revoked {
            return Err(Denial::Revoked);
        }
        let grant = s.grant;
        s.model
            .apply(Command::RevokeGrant { grant })
            .map_err(Denial::Model)?;
        s.revoked = true;
        s.active = None;
        Ok(())
    }

    fn cancel(&self) -> Result<(), Denial> {
        let mut s = self.state.lock().unwrap();
        if s.sealed {
            return Err(Denial::Closed);
        }
        if s.cancel_requested {
            return Err(Denial::Model(ModelError::InvalidTransition));
        }
        let task = s.task;
        s.model
            .apply(Command::RequestCancel { task })
            .map_err(Denial::Model)?;
        s.cancel_requested = true;
        if !s.revoked {
            let grant = s.grant;
            s.model
                .apply(Command::RevokeGrant { grant })
                .map_err(Denial::Model)?;
            s.revoked = true;
            s.active = None;
        }
        Ok(())
    }

    fn seal(&self, completion: Completion) -> Result<ScopeReport, Denial> {
        let mut s = self.state.lock().unwrap();
        if s.sealed {
            return Err(Denial::Closed);
        }
        let task = s.task;
        let term = match completion {
            Completion::Success => Command::Succeed { task },
            Completion::Failure => Command::Fail {
                task,
                code: PANIC_CODE,
            },
            Completion::Cancelled => Command::AcknowledgeCancel { task },
        };
        s.model.apply(term).map_err(Denial::Model)?;
        s.model
            .apply(Command::Join { task })
            .map_err(Denial::Model)?;
        drop(s.file.take());
        let (scope, resource) = (s.scope, s.resource);
        s.model
            .apply(Command::Release { scope, resource })
            .map_err(Denial::Model)?;
        let report = match s
            .model
            .apply(Command::Close { scope })
            .map_err(Denial::Model)?
        {
            Receipt::Closed(report) => report,
            _ => return Err(Denial::Incomplete),
        };
        s.model.replay_checked().map_err(Denial::Model)?;
        s.sealed = true;
        s.final_outcome = Some(report.outcome);
        Ok(report)
    }

    fn commit_to_nam(
        &self,
        kernel: &mut AtomicKernel,
        atom: AtomId,
        value: i64,
    ) -> Result<bool, Denial> {
        let s = self.state.lock().unwrap();
        if !s.sealed || s.final_outcome != Some(ScopeOutcome::Succeeded) {
            return Err(Denial::Incomplete);
        }
        kernel.set(atom, value).map_err(Denial::Kernel)
    }
    fn bytes(&self) -> Vec<u8> {
        fs::read(self.scratch.file()).expect("own disposable file is readable")
    }
    fn used(&self) -> usize {
        self.state.lock().unwrap().used
    }
    fn replay_ok(&self) -> bool {
        self.state.lock().unwrap().model.replay_checked().is_ok()
    }
}

#[test]
fn successful_model_check_real_file_write_and_parent_only_nam() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    assert_eq!(host.write(scope, ticket, b"42\n"), Ok(3));
    let report = host.seal(Completion::Success).unwrap();
    assert_eq!(report.outcome, ScopeOutcome::Succeeded);
    assert_eq!(host.bytes(), b"42\n");
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(host.commit_to_nam(&mut kernel, atom, 42), Ok(true));
    assert_eq!(kernel.get(atom), Ok(&Value::Int(42)));
    assert_eq!(kernel.version(atom), Ok(1));
    assert!(host.replay_ok());
}

#[test]
fn no_ticket_is_default_denied_before_model_use_and_disk_io() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let fake = Ticket { scope, epoch: 1 };
    assert_eq!(host.write(scope, fake, b"NO"), Err(Denial::NoTicket));
    assert!(host.bytes().is_empty());
}

#[test]
fn cross_scope_denied_before_file_write() {
    let host = HostAdapter::new(8);
    let ticket = host.issue(host.scope()).unwrap();
    let (other, _) = Model::bootstrap(
        99,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        0,
    );
    assert_eq!(
        host.write(other.root(), ticket, b"NO"),
        Err(Denial::WrongScope)
    );
    assert!(host.bytes().is_empty());
}

#[test]
fn mutated_and_replaced_tickets_cannot_write() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let old = host.issue(scope).unwrap();
    let mut altered = old;
    altered.epoch ^= 1;
    assert_eq!(host.write(scope, altered, b"NO"), Err(Denial::Stale));
    let latest = host.issue(scope).unwrap();
    assert_eq!(host.write(scope, old, b"NO"), Err(Denial::Stale));
    assert_eq!(host.write(scope, latest, b"OK"), Ok(2));
    assert_eq!(host.bytes(), b"OK");
}

#[test]
fn byte_quota_denies_overflow_without_new_partial_bytes() {
    let host = HostAdapter::new(3);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    assert_eq!(host.write(scope, ticket, b"ABCD"), Err(Denial::Quota));
    assert_eq!(host.used(), 0);
    assert_eq!(host.write(scope, ticket, b"YES"), Ok(3));
    assert_eq!(host.write(scope, ticket, b"Z"), Err(Denial::Quota));
    assert_eq!(host.bytes(), b"YES");
}

#[test]
fn empty_payload_performs_zero_disk_bytes() {
    let host = HostAdapter::new(0);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    assert_eq!(host.write(scope, ticket, b""), Ok(0));
    assert_eq!(host.used(), 0);
    assert!(host.bytes().is_empty());
}

#[test]
fn revoke_before_real_worker_release_denies_io_and_nam() {
    let host = Arc::new(HostAdapter::new(8));
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    let gate = Arc::new(Barrier::new(2));
    let (tx, rx) = mpsc::sync_channel::<()>(0);
    let result = thread::scope(|s| {
        let worker = Arc::clone(&host);
        let worker_gate = Arc::clone(&gate);
        let child = s.spawn(move || {
            tx.send(()).unwrap();
            worker_gate.wait();
            worker.write(scope, ticket, b"LATE")
        });
        rx.recv().unwrap();
        host.cancel().unwrap();
        gate.wait();
        child.join().unwrap()
    });
    assert_eq!(result, Err(Denial::Revoked));
    let report = host.seal(Completion::Cancelled).unwrap();
    assert_eq!(report.outcome, ScopeOutcome::Cancelled);
    assert!(host.bytes().is_empty());
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(
        host.commit_to_nam(&mut kernel, atom, 42),
        Err(Denial::Incomplete)
    );
    assert_eq!(kernel.version(atom), Ok(0));
}

#[test]
fn real_worker_write_before_revoke_persists_but_future_io_denied() {
    let host = Arc::new(HostAdapter::new(8));
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    let (tx, rx) = mpsc::sync_channel::<Result<usize, Denial>>(0);
    let outcome = thread::scope(|s| {
        let worker = Arc::clone(&host);
        let child = s.spawn(move || tx.send(worker.write(scope, ticket, b"FIRST")).unwrap());
        let result = rx.recv().unwrap();
        host.revoke().unwrap();
        child.join().unwrap();
        result
    });
    assert_eq!(outcome, Ok(5));
    assert_eq!(host.write(scope, ticket, b"NO"), Err(Denial::Revoked));
    assert_eq!(host.bytes(), b"FIRST");
    assert_eq!(
        host.seal(Completion::Success).unwrap().outcome,
        ScopeOutcome::Succeeded
    );
    assert!(host.replay_ok());
}

#[test]
fn duplicate_revoke_and_ticket_issuance_after_revoke_are_denied() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    host.revoke().unwrap();
    assert_eq!(host.issue(scope), Err(Denial::Revoked));
    assert_eq!(host.revoke(), Err(Denial::Revoked));
    assert_eq!(host.write(scope, ticket, b"NO"), Err(Denial::Revoked));
    assert!(host.bytes().is_empty());
}

#[test]
fn double_cancel_and_success_after_cancel_are_denied() {
    let host = HostAdapter::new(8);
    host.cancel().unwrap();
    assert_eq!(
        host.cancel(),
        Err(Denial::Model(ModelError::InvalidTransition))
    );
    assert_eq!(
        host.seal(Completion::Success),
        Err(Denial::Model(ModelError::InvalidTransition))
    );
    assert_eq!(
        host.seal(Completion::Cancelled).unwrap().outcome,
        ScopeOutcome::Cancelled
    );
}

#[test]
fn file_close_invalidates_ticket_and_duplicate_close_is_denied() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    host.seal(Completion::Success).unwrap();
    assert_eq!(host.seal(Completion::Success), Err(Denial::Closed));
    assert_eq!(host.write(scope, ticket, b"NO"), Err(Denial::Closed));
    assert_eq!(host.issue(scope), Err(Denial::Closed));
    assert_eq!(host.revoke(), Err(Denial::Closed));
}

#[test]
fn joined_worker_panic_closes_file_and_prevents_nam_commit() {
    let host = HostAdapter::new(8);
    let child = thread::scope(|s| s.spawn(|| panic!("R4D deliberate child panic")).join());
    assert!(child.is_err());
    assert_eq!(
        host.seal(Completion::Failure).unwrap().outcome,
        ScopeOutcome::Failed
    );
    assert!(host.bytes().is_empty());
    assert!(host.replay_ok());
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(
        host.commit_to_nam(&mut kernel, atom, 42),
        Err(Denial::Incomplete)
    );
}

#[test]
fn real_file_bytes_survive_failure_without_nam_rollback() {
    let host = HostAdapter::new(8);
    let scope = host.scope();
    let ticket = host.issue(scope).unwrap();
    assert_eq!(host.write(scope, ticket, b"PERSIST"), Ok(7));
    host.seal(Completion::Failure).unwrap();
    assert_eq!(host.bytes(), b"PERSIST");
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(
        host.commit_to_nam(&mut kernel, atom, 42),
        Err(Denial::Incomplete)
    );
    assert_eq!(kernel.version(atom), Ok(0));
}

#[test]
fn pure_reference_model_still_rejects_external_io() {
    let (mut model, host) = Model::bootstrap(
        77,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        20,
    );
    assert_eq!(
        model.host_grant(&host, model.root(), ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
}

#[test]
fn drop_reclaims_only_new_disposable_file_not_user_paths() {
    let path;
    {
        let host = HostAdapter::new(8);
        path = host.scratch.file();
        assert!(path.exists());
    }
    assert!(!path.exists());
}
