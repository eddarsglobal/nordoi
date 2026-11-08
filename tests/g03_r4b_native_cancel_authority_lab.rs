//! G0.3 R4B — research-only cooperative native task checkpoint.
//!
//! The parent alone owns the NAM kernel and the existing R0.2 reference model.
//! Children execute a tiny fixed pure enum, not arbitrary .noi/Rust code.
//! No host I/O, grant material, resource handles or NAM pointers reach workers.
//! This is NOT a public executor or a nonforgeable authorization boundary.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use nordoi_kernel::{AtomId, AtomicError, AtomicKernel, Capability, Value};
use reference::{
    Command, Model, ModelError, Receipt, ResourceEffect, ScopeBudget, ScopeOutcome, ScopeReport,
    TaskId, TaskOutcome,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::thread;

const MAX_TASKS: usize = 4;
const ERROR_OVERFLOW: u16 = 111;
const ERROR_PANIC: u16 = 112;

#[derive(Clone, Copy, Debug)]
enum PurePlan {
    Add(i64, i64),
    Multiply(i64, i64),
    CheckpointAdd(i64, i64),
    Reject(u16),
    SyntheticPanic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkerOutcome {
    Value(i64),
    Cancelled,
    Failed(u16),
}

fn worker(
    plan: PurePlan,
    reached: mpsc::SyncSender<()>,
    checkpoint_release: Arc<Barrier>,
    cancelled: Arc<AtomicBool>,
) -> WorkerOutcome {
    match plan {
        PurePlan::Add(a, b) => a
            .checked_add(b)
            .map(WorkerOutcome::Value)
            .unwrap_or(WorkerOutcome::Failed(ERROR_OVERFLOW)),
        PurePlan::Multiply(a, b) => a
            .checked_mul(b)
            .map(WorkerOutcome::Value)
            .unwrap_or(WorkerOutcome::Failed(ERROR_OVERFLOW)),
        PurePlan::CheckpointAdd(a, b) => {
            // The worker is actually running before the parent can request
            // cancellation. It waits at ONE explicit cooperative checkpoint.
            if reached.send(()).is_err() {
                return WorkerOutcome::Failed(ERROR_PANIC);
            }
            checkpoint_release.wait();
            if cancelled.load(Ordering::Acquire) {
                return WorkerOutcome::Cancelled;
            }
            a.checked_add(b)
                .map(WorkerOutcome::Value)
                .unwrap_or(WorkerOutcome::Failed(ERROR_OVERFLOW))
        }
        PurePlan::Reject(code) => WorkerOutcome::Failed(code),
        PurePlan::SyntheticPanic => panic!("R4B deliberate test-only worker panic"),
    }
}

#[derive(Debug, Eq, PartialEq)]
enum LabFault {
    TooManyTasks,
    TooManyCheckpoints,
    InvalidCancelPlan,
    CheckpointProtocol,
    Reference(ModelError),
}

#[derive(Debug)]
struct Outcome {
    report: ScopeReport,
    values: Option<Vec<i64>>,
    workers: usize,
    replay_verified: bool,
    cancel_requested: bool,
}

/// Exactly one optional cancellation target, which must be a checkpoint task.
/// Validate plans before creating *any* native worker. The scoped threads join
/// before a terminal task event is written to the reference model.
fn execute(plans: &[PurePlan], cancel_index: Option<usize>) -> Result<Outcome, LabFault> {
    if plans.len() > MAX_TASKS {
        return Err(LabFault::TooManyTasks);
    }
    let checkpoint_count = plans
        .iter()
        .filter(|p| matches!(p, PurePlan::CheckpointAdd(_, _)))
        .count();
    if checkpoint_count > 1 {
        return Err(LabFault::TooManyCheckpoints);
    }
    if let Some(index) = cancel_index {
        if index >= plans.len() || !matches!(plans[index], PurePlan::CheckpointAdd(_, _)) {
            return Err(LabFault::InvalidCancelPlan);
        }
    }

    let (mut model, _host) = Model::bootstrap(
        0x2026_1108_4B,
        ScopeBudget {
            tasks: MAX_TASKS,
            resources: 0,
            children: 0,
        },
        64,
    );
    let root = model.root();
    let mut ids: Vec<TaskId> = Vec::with_capacity(plans.len());
    for _ in plans {
        let id = match model
            .apply(Command::DeclareTask { scope: root })
            .map_err(LabFault::Reference)?
        {
            Receipt::Task(id) => id,
            _ => return Err(LabFault::CheckpointProtocol),
        };
        model
            .apply(Command::Admit { task: id })
            .map_err(LabFault::Reference)?;
        model
            .apply(Command::Start { task: id })
            .map_err(LabFault::Reference)?;
        ids.push(id);
    }

    let (results, protocol_error) = thread::scope(|scope| {
        let (reached_tx, reached_rx) = mpsc::sync_channel::<()>(0);
        let checkpoint_release = Arc::new(Barrier::new(2));
        let cancelled = Arc::new(AtomicBool::new(false));
        let handles: Vec<_> = plans
            .iter()
            .copied()
            .map(|plan| {
                let signal = reached_tx.clone();
                let release = Arc::clone(&checkpoint_release);
                let cancel = Arc::clone(&cancelled);
                scope.spawn(move || worker(plan, signal, release, cancel))
            })
            .collect();
        drop(reached_tx);

        let mut error = None;
        if checkpoint_count == 1 {
            // Zero timers and zero scheduler-order guesses. A zero-capacity
            // channel proves the worker reached the live checkpoint.
            if reached_rx.recv().is_err() {
                error = Some(LabFault::CheckpointProtocol);
            } else {
                if let Some(index) = cancel_index {
                    match model.apply(Command::RequestCancel { task: ids[index] }) {
                        Ok(Receipt::Unit) => cancelled.store(true, Ordering::Release),
                        Ok(_) => error = Some(LabFault::CheckpointProtocol),
                        Err(problem) => error = Some(LabFault::Reference(problem)),
                    }
                }
                // Always release the worker, including on an unexpected
                // model error; never strand a scoped thread at its barrier.
                checkpoint_release.wait();
            }
        }
        let results = handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or(WorkerOutcome::Failed(ERROR_PANIC))
            })
            .collect::<Vec<_>>();
        (results, error)
    });
    if let Some(error) = protocol_error {
        return Err(error);
    }

    let mut values = Vec::with_capacity(results.len());
    let mut all_ok = true;
    for (&id, result) in ids.iter().zip(results) {
        let terminal = match result {
            WorkerOutcome::Value(value) => {
                values.push(value);
                Command::Succeed { task: id }
            }
            WorkerOutcome::Cancelled => {
                all_ok = false;
                Command::AcknowledgeCancel { task: id }
            }
            WorkerOutcome::Failed(code) => {
                all_ok = false;
                Command::Fail { task: id, code }
            }
        };
        model.apply(terminal).map_err(LabFault::Reference)?;
        model
            .apply(Command::Join { task: id })
            .map_err(LabFault::Reference)?;
    }
    let report = match model
        .apply(Command::Close { scope: root })
        .map_err(LabFault::Reference)?
    {
        Receipt::Closed(report) => report,
        _ => return Err(LabFault::CheckpointProtocol),
    };
    Ok(Outcome {
        report,
        values: all_ok.then_some(values),
        workers: plans.len(),
        replay_verified: model.replay_checked().is_ok(),
        cancel_requested: cancel_index.is_some(),
    })
}

#[derive(Debug, Eq, PartialEq)]
enum CommitFault {
    Incomplete,
    Arity,
    DuplicateTargets,
    Kernel(AtomicError),
}

/// A *parent-only* witness: all worker threads have completed and a canonical
/// R0.2 report exists before any NAM mutation is attempted.
fn commit_owner(
    kernel: &mut AtomicKernel,
    atoms: &[AtomId],
    outcome: &Outcome,
) -> Result<usize, CommitFault> {
    if outcome.report.outcome != ScopeOutcome::Succeeded || !outcome.replay_verified {
        return Err(CommitFault::Incomplete);
    }
    let values = outcome.values.as_ref().ok_or(CommitFault::Incomplete)?;
    if atoms.len() != values.len() {
        return Err(CommitFault::Arity);
    }
    if atoms.iter().collect::<BTreeSet<_>>().len() != atoms.len() {
        return Err(CommitFault::DuplicateTargets);
    }
    if atoms.is_empty() {
        return Ok(0);
    }
    let mut tx = kernel
        .begin_transaction(kernel.root_domain())
        .map_err(CommitFault::Kernel)?;
    for (&atom, &value) in atoms.iter().zip(values) {
        tx.set(kernel, atom, value).map_err(CommitFault::Kernel)?;
    }
    Ok(kernel.commit(tx).map_err(CommitFault::Kernel)?.changed_atoms)
}

#[test]
fn empty_plan_runs_zero_threads_and_commits_zero_work() {
    let outcome = execute(&[], None).unwrap();
    assert_eq!(outcome.workers, 0);
    assert_eq!(outcome.values, Some(Vec::new()));
    assert_eq!(outcome.report.outcome, ScopeOutcome::Succeeded);
    let mut kernel = AtomicKernel::new();
    assert_eq!(commit_owner(&mut kernel, &[], &outcome), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn real_scoped_threads_commit_only_after_successful_join_and_replay() {
    let outcome = execute(&[PurePlan::Add(5, 9), PurePlan::Multiply(3, 8)], None).unwrap();
    assert_eq!(outcome.workers, 2);
    assert_eq!(outcome.values, Some(vec![14, 24]));
    assert!(outcome.replay_verified);
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    let b = kernel.create_atom(0_i64);
    assert_eq!(commit_owner(&mut kernel, &[a, b], &outcome), Ok(2));
    assert_eq!(kernel.get(a), Ok(&Value::Int(14)));
    assert_eq!(kernel.get(b), Ok(&Value::Int(24)));
    assert_eq!(kernel.version(a), Ok(1));
    assert_eq!(kernel.version(b), Ok(1));
}

#[test]
fn worker_waits_at_real_checkpoint_and_completes_without_cancellation() {
    let result = execute(&[PurePlan::CheckpointAdd(20, 22)], None).unwrap();
    assert_eq!(result.values, Some(vec![42]));
    assert_eq!(result.report.outcome, ScopeOutcome::Succeeded);
    assert!(!result.cancel_requested);
    assert!(result.replay_verified);
}

#[test]
fn parent_requests_cancel_after_checkpoint_and_worker_acknowledges() {
    let result = execute(&[PurePlan::CheckpointAdd(20, 22)], Some(0)).unwrap();
    assert_eq!(result.workers, 1);
    assert_eq!(result.report.outcome, ScopeOutcome::Cancelled);
    assert_eq!(result.report.tasks[0].1, TaskOutcome::Cancelled);
    assert_eq!(result.values, None);
    assert!(result.cancel_requested && result.replay_verified);
}

#[test]
fn cancelled_child_blocks_atomic_multi_output_commit() {
    let result = execute(
        &[PurePlan::Add(1, 2), PurePlan::CheckpointAdd(30, 40)],
        Some(1),
    )
    .unwrap();
    assert_eq!(result.report.outcome, ScopeOutcome::Cancelled);
    let mut kernel = AtomicKernel::new();
    let first = kernel.create_atom(0_i64);
    let second = kernel.create_atom(0_i64);
    assert_eq!(
        commit_owner(&mut kernel, &[first, second], &result),
        Err(CommitFault::Incomplete)
    );
    assert_eq!(kernel.version(first), Ok(0));
    assert_eq!(kernel.version(second), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn worker_failure_or_panic_blocks_every_nam_write() {
    for bad in [PurePlan::Reject(9), PurePlan::SyntheticPanic] {
        let result = execute(&[PurePlan::Add(1, 2), bad], None).unwrap();
        assert_eq!(result.report.outcome, ScopeOutcome::Failed);
        assert!(result.values.is_none());
        assert!(result.replay_verified);
        let mut kernel = AtomicKernel::new();
        let first = kernel.create_atom(0_i64);
        let second = kernel.create_atom(0_i64);
        assert_eq!(
            commit_owner(&mut kernel, &[first, second], &result),
            Err(CommitFault::Incomplete)
        );
        assert_eq!(kernel.pending_work(), 0);
    }
}

#[test]
fn checked_overflow_is_failure_not_success() {
    let result = execute(&[PurePlan::Add(i64::MAX, 1)], None).unwrap();
    assert_eq!(result.report.tasks[0].1, TaskOutcome::Failed(ERROR_OVERFLOW));
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
}

#[test]
fn invalid_cancel_plan_and_too_many_workers_are_rejected_before_spawn() {
    assert!(matches!(
        execute(&[PurePlan::Add(1, 2)], Some(0)),
        Err(LabFault::InvalidCancelPlan)
    ));
    assert!(matches!(
        execute(&[PurePlan::CheckpointAdd(1, 2)], Some(1)),
        Err(LabFault::InvalidCancelPlan)
    ));
    assert!(matches!(
        execute(&[PurePlan::Add(1, 2); MAX_TASKS + 1], None),
        Err(LabFault::TooManyTasks)
    ));
    assert!(matches!(
        execute(
            &[PurePlan::CheckpointAdd(1, 2), PurePlan::CheckpointAdd(3, 4)],
            None,
        ),
        Err(LabFault::TooManyCheckpoints)
    ));
}

#[test]
fn duplicate_output_targets_are_rejected_before_any_mutation() {
    let result = execute(&[PurePlan::Add(1, 2), PurePlan::Multiply(2, 2)], None).unwrap();
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(
        commit_owner(&mut kernel, &[atom, atom], &result),
        Err(CommitFault::DuplicateTargets)
    );
    assert_eq!(kernel.get(atom), Ok(&Value::Int(0)));
    assert_eq!(kernel.version(atom), Ok(0));
}

#[test]
fn mismatched_targets_are_rejected_before_any_mutation() {
    let result = execute(&[PurePlan::Add(1, 2)], None).unwrap();
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    assert_eq!(commit_owner(&mut kernel, &[atom, atom], &result), Err(CommitFault::Arity));
    assert_eq!(kernel.version(atom), Ok(0));
}

#[test]
fn foreign_owner_denial_is_atomic_for_multi_atom_commit() {
    let result = execute(&[PurePlan::Add(1, 2), PurePlan::Multiply(2, 3)], None).unwrap();
    let mut kernel = AtomicKernel::new();
    let home = kernel.create_atom(0_i64);
    let foreign = kernel.create_domain("foreign").unwrap();
    let other = kernel.create_atom_owned(foreign, 0_i64).unwrap();
    assert!(matches!(
        commit_owner(&mut kernel, &[home, other], &result),
        Err(CommitFault::Kernel(AtomicError::OwnershipViolation { .. }))
    ));
    assert_eq!(kernel.version(home), Ok(0));
    assert_eq!(kernel.version(other), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn console_authority_remains_default_denied_after_native_threads() {
    let mut kernel = AtomicKernel::new();
    let capability = Capability::ConsoleWrite;
    assert_eq!(
        kernel.require(&capability),
        Err(AtomicError::CapabilityDenied(capability.clone()))
    );
    let result = execute(&[PurePlan::CheckpointAdd(1, 1)], None).unwrap();
    let output = kernel.create_atom(0_i64);
    assert_eq!(commit_owner(&mut kernel, &[output], &result), Ok(1));
    assert_eq!(
        kernel.require(&capability),
        Err(AtomicError::CapabilityDenied(capability))
    );
}

#[test]
fn reference_rejects_late_or_duplicate_cancellation_without_state_changes() {
    let (mut model, _) = Model::bootstrap(
        47,
        ScopeBudget { tasks: 2, resources: 0, children: 0 },
        32,
    );
    let root = model.root();
    let task = match model.apply(Command::DeclareTask { scope: root }) {
        Ok(Receipt::Task(id)) => id,
        unexpected => panic!("unexpected reference receipt {unexpected:?}"),
    };
    model.apply(Command::Admit { task }).unwrap();
    model.apply(Command::Start { task }).unwrap();
    model.apply(Command::RequestCancel { task }).unwrap();
    let before = model.clone();
    assert_eq!(
        model.apply(Command::RequestCancel { task }),
        Err(ModelError::InvalidTransition)
    );
    assert_eq!(model, before);
    assert_eq!(
        model.apply(Command::Succeed { task }),
        Err(ModelError::InvalidTransition)
    );
    model.apply(Command::AcknowledgeCancel { task }).unwrap();
    model.apply(Command::Join { task }).unwrap();
    let after = model.clone();
    assert_eq!(
        model.apply(Command::RequestCancel { task }),
        Err(ModelError::InvalidTransition)
    );
    assert_eq!(model, after);
    assert!(model.replay_checked().is_ok());
}

#[test]
fn reference_scope_cannot_close_until_join() {
    let (mut model, _) = Model::bootstrap(
        48,
        ScopeBudget { tasks: 1, resources: 0, children: 0 },
        32,
    );
    let root = model.root();
    let task = match model.apply(Command::DeclareTask { scope: root }) {
        Ok(Receipt::Task(id)) => id,
        unexpected => panic!("unexpected reference receipt {unexpected:?}"),
    };
    model.apply(Command::Admit { task }).unwrap();
    model.apply(Command::Start { task }).unwrap();
    assert_eq!(
        model.apply(Command::Close { scope: root }),
        Err(ModelError::OutstandingTasks)
    );
    model.apply(Command::Succeed { task }).unwrap();
    assert_eq!(
        model.apply(Command::Close { scope: root }),
        Err(ModelError::OutstandingTasks)
    );
    model.apply(Command::Join { task }).unwrap();
    assert!(matches!(
        model.apply(Command::Close { scope: root }),
        Ok(Receipt::Closed(_))
    ));
}

#[test]
fn model_grant_revocation_rejects_use_but_never_authorizes_host_io() {
    let (mut model, host) = Model::bootstrap(
        49,
        ScopeBudget { tasks: 0, resources: 1, children: 0 },
        32,
    );
    let root = model.root();
    model
        .apply(Command::DeclareEffect { scope: root, effect: ResourceEffect::Read })
        .unwrap();
    let grant = model.host_grant(&host, root, ResourceEffect::Read).unwrap();
    let resource = match model
        .apply(Command::Acquire { scope: root, grant, effect: ResourceEffect::Read })
    {
        Ok(Receipt::Resource(resource)) => resource,
        unexpected => panic!("unexpected acquire {unexpected:?}"),
    };
    model
        .apply(Command::Use { scope: root, resource, grant, effect: ResourceEffect::Read })
        .unwrap();
    model.apply(Command::RevokeGrant { grant }).unwrap();
    let before = model.clone();
    assert_eq!(
        model.apply(Command::Use { scope: root, resource, grant, effect: ResourceEffect::Read }),
        Err(ModelError::RevokedGrant)
    );
    assert_eq!(model, before);
    assert_eq!(
        model.host_grant(&host, root, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    model.apply(Command::Release { scope: root, resource }).unwrap();
    assert!(matches!(
        model.apply(Command::Close { scope: root }),
        Ok(Receipt::Closed(_))
    ));
    assert!(model.replay_checked().is_ok());
}
