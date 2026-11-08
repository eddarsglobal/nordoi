//! G0.3 R4A — bounded *real native thread* research witness.
//!
//! This integration test is NOT linked into NORDOI binaries, does NOT create a
//! user-facing executor/NAIR opcode or trust boundary, and runs no host I/O.
//! Thread tasks execute a fixed pure instruction enum (not arbitrary callbacks).
//! Each worker is joined before the single owner-side NAM commit is considered.
//! R0.2 reference outcomes are semantic witnesses, NOT a native certification.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use nordoi_kernel::{AtomId, AtomicError, AtomicKernel, Capability, Value};
use reference::{
    Command, Model, ModelError, Receipt, ScopeBudget, ScopeOutcome, ScopeReport, TaskId,
    TaskOutcome,
};
use std::thread;

const MAX_TASKS: usize = 4;
const ERROR_OVERFLOW: u16 = 101;
const ERROR_PANIC: u16 = u16::MAX;

#[derive(Debug, Clone, Copy)]
enum PureProgram {
    CheckedAdd(i64, i64),
    CheckedMul(i64, i64),
    YieldThenAdd(i64, i64),
    Fail(u16),
    SyntheticPanic,
}

fn compute(program: PureProgram) -> Result<i64, u16> {
    match program {
        PureProgram::CheckedAdd(a, b) => a.checked_add(b).ok_or(ERROR_OVERFLOW),
        PureProgram::CheckedMul(a, b) => a.checked_mul(b).ok_or(ERROR_OVERFLOW),
        PureProgram::YieldThenAdd(a, b) => {
            thread::yield_now();
            a.checked_add(b).ok_or(ERROR_OVERFLOW)
        }
        PureProgram::Fail(code) => Err(code),
        PureProgram::SyntheticPanic => panic!("synthetic native child panic, contained by join"),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LabError {
    TooManyTasks,
    Reference(ModelError),
}

#[derive(Debug, PartialEq, Eq)]
enum CommitError {
    IncompleteScope,
    ArityMismatch,
    Kernel(AtomicError),
}

#[derive(Debug)]
struct LabResult {
    report: ScopeReport,
    values: Option<Vec<i64>>,
    launched_workers: usize,
    replay_verified: bool,
}

/// All worker threads finish before this function returns. Children are modeled
/// before native execution, and completed/failed/joined in stable TaskId order.
/// These pure fixed operations grant no authority to a guest NORDOI program.
fn execute(programs: &[PureProgram]) -> Result<LabResult, LabError> {
    if programs.len() > MAX_TASKS {
        return Err(LabError::TooManyTasks);
    }
    let (mut model, _host_permit) = Model::bootstrap(
        0x2026_1108,
        ScopeBudget {
            tasks: MAX_TASKS,
            resources: 0,
            children: 0,
        },
        64,
    );
    let root = model.root();
    let mut ids: Vec<TaskId> = Vec::with_capacity(programs.len());
    for _ in programs {
        let id = match model
            .apply(Command::DeclareTask { scope: root })
            .map_err(LabError::Reference)?
        {
            Receipt::Task(id) => id,
            other => panic!("unexpected reference DeclareTask receipt: {other:?}"),
        };
        model
            .apply(Command::Admit { task: id })
            .map_err(LabError::Reference)?;
        model
            .apply(Command::Start { task: id })
            .map_err(LabError::Reference)?;
        ids.push(id);
    }

    // Scoped threads: the Rust host cannot leave this scope while a child runs.
    // No NAM, grant, resource handle, or reference model is shared with workers.
    let outcomes: Vec<Result<i64, u16>> = thread::scope(|scope| {
        let workers: Vec<_> = programs
            .iter()
            .copied()
            .map(|program| scope.spawn(move || compute(program)))
            .collect();
        workers
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(result) => result,
                Err(_) => Err(ERROR_PANIC),
            })
            .collect()
    });

    let mut values = Vec::with_capacity(outcomes.len());
    let mut all_succeeded = true;
    for (id, outcome) in ids.iter().copied().zip(outcomes) {
        let terminal = match outcome {
            Ok(value) => {
                values.push(value);
                Command::Succeed { task: id }
            }
            Err(code) => {
                all_succeeded = false;
                Command::Fail { task: id, code }
            }
        };
        model.apply(terminal).map_err(LabError::Reference)?;
        model
            .apply(Command::Join { task: id })
            .map_err(LabError::Reference)?;
    }
    let report = match model
        .apply(Command::Close { scope: root })
        .map_err(LabError::Reference)?
    {
        Receipt::Closed(report) => report,
        other => panic!("unexpected reference Close receipt: {other:?}"),
    };
    let replay_verified = model.replay_checked().is_ok();
    Ok(LabResult {
        report,
        values: all_succeeded.then_some(values),
        launched_workers: programs.len(),
        replay_verified,
    })
}

/// Deliberately separate from native execution. Failed worker / wrong arity /
/// failed ownership checks prevent every NAM mutation; commit is transactional.
/// The thread lab itself has no access to NORDOI runtime authority.
fn commit_successful(
    kernel: &mut AtomicKernel,
    target_atoms: &[AtomId],
    outcome: &LabResult,
) -> Result<usize, CommitError> {
    let values = outcome
        .values
        .as_ref()
        .ok_or(CommitError::IncompleteScope)?;
    if outcome.report.outcome != ScopeOutcome::Succeeded || !outcome.replay_verified {
        return Err(CommitError::IncompleteScope);
    }
    if values.len() != target_atoms.len() {
        return Err(CommitError::ArityMismatch);
    }
    if values.is_empty() {
        return Ok(0); // truly zero work in the zero-task case
    }
    let mut tx = kernel
        .begin_transaction(kernel.root_domain())
        .map_err(CommitError::Kernel)?;
    for (&atom, &value) in target_atoms.iter().zip(values) {
        tx.set(kernel, atom, value).map_err(CommitError::Kernel)?;
    }
    let receipt = kernel.commit(tx).map_err(CommitError::Kernel)?;
    Ok(receipt.changed_atoms)
}

#[test]
fn zero_tasks_means_zero_native_workers_and_zero_nam_work() {
    let result = execute(&[]).unwrap();
    assert_eq!(result.launched_workers, 0);
    assert_eq!(result.values, Some(Vec::new()));
    assert_eq!(result.report.outcome, ScopeOutcome::Succeeded);
    assert!(result.replay_verified);
    let mut kernel = AtomicKernel::new();
    assert_eq!(commit_successful(&mut kernel, &[], &result), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn two_real_threads_join_then_single_atomic_nam_commit() {
    let result = execute(&[
        PureProgram::YieldThenAdd(3, 5),
        PureProgram::CheckedMul(3, 4),
    ])
    .unwrap();
    assert_eq!(result.launched_workers, 2);
    assert_eq!(result.values, Some(vec![8, 12]));
    assert_eq!(result.report.outcome, ScopeOutcome::Succeeded);
    assert_eq!(result.report.tasks.len(), 2);
    assert!(result.replay_verified);
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    let b = kernel.create_atom(0_i64);
    assert_eq!(commit_successful(&mut kernel, &[a, b], &result), Ok(2));
    assert_eq!(kernel.get(a), Ok(&Value::Int(8)));
    assert_eq!(kernel.get(b), Ok(&Value::Int(12)));
    assert_eq!(kernel.version(a), Ok(1));
    assert_eq!(kernel.version(b), Ok(1));
}

#[test]
fn child_failure_disallows_any_nam_mutation() {
    let result = execute(&[PureProgram::CheckedAdd(3, 5), PureProgram::Fail(7)]).unwrap();
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.report.tasks[1].1, TaskOutcome::Failed(7));
    assert_eq!(result.values, None);
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    let b = kernel.create_atom(0_i64);
    assert_eq!(
        commit_successful(&mut kernel, &[a, b], &result),
        Err(CommitError::IncompleteScope)
    );
    assert_eq!(kernel.version(a), Ok(0));
    assert_eq!(kernel.version(b), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn native_child_panic_is_joined_and_blocks_nam_commit() {
    let result = execute(&[PureProgram::SyntheticPanic]).unwrap();
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(result.report.tasks[0].1, TaskOutcome::Failed(ERROR_PANIC));
    assert_eq!(result.values, None);
    assert_eq!(result.launched_workers, 1);
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    assert_eq!(
        commit_successful(&mut kernel, &[a], &result),
        Err(CommitError::IncompleteScope)
    );
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn task_quota_rejected_before_any_thread_spawn() {
    let programs = [PureProgram::CheckedAdd(0, 1); MAX_TASKS + 1];
    assert!(matches!(execute(&programs), Err(LabError::TooManyTasks)));
}

#[test]
fn overflow_cannot_commit_and_scope_reports_failure() {
    let result = execute(&[PureProgram::CheckedAdd(i64::MAX, 1)]).unwrap();
    assert_eq!(result.report.outcome, ScopeOutcome::Failed);
    assert_eq!(
        result.report.tasks[0].1,
        TaskOutcome::Failed(ERROR_OVERFLOW)
    );
    assert!(result.replay_verified);
}

#[test]
fn wrong_target_count_fails_without_mutating_nam() {
    let result = execute(&[PureProgram::CheckedAdd(2, 2)]).unwrap();
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    assert_eq!(
        commit_successful(&mut kernel, &[a, a], &result),
        Err(CommitError::ArityMismatch)
    );
    assert_eq!(kernel.version(a), Ok(0));
}

#[test]
fn nam_ownership_denial_prevents_partial_multi_atom_commit() {
    let result = execute(&[PureProgram::CheckedAdd(1, 2), PureProgram::CheckedAdd(3, 4)]).unwrap();
    let mut kernel = AtomicKernel::new();
    let a = kernel.create_atom(0_i64);
    let foreign = kernel.create_domain("foreign").unwrap();
    let b = kernel.create_atom_owned(foreign, 0_i64).unwrap();
    assert!(matches!(
        commit_successful(&mut kernel, &[a, b], &result),
        Err(CommitError::Kernel(AtomicError::OwnershipViolation { .. }))
    ));
    assert_eq!(kernel.get(a), Ok(&Value::Int(0)));
    assert_eq!(kernel.get(b), Ok(&Value::Int(0)));
    assert_eq!(kernel.version(a), Ok(0));
    assert_eq!(kernel.version(b), Ok(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn no_amplified_ambient_host_authority_in_parent() {
    let mut kernel = AtomicKernel::new();
    let cap = Capability::ConsoleWrite;
    assert_eq!(
        kernel.require(&cap),
        Err(AtomicError::CapabilityDenied(cap.clone()))
    );
    let outcome = execute(&[PureProgram::CheckedMul(2, 3)]).unwrap();
    let target = kernel.create_atom(0_i64);
    assert_eq!(commit_successful(&mut kernel, &[target], &outcome), Ok(1));
    assert_eq!(
        kernel.require(&cap),
        Err(AtomicError::CapabilityDenied(cap))
    );
}

#[test]
fn completed_reference_report_is_stable_across_repeated_native_runs() {
    let programs = [
        PureProgram::YieldThenAdd(20, 22),
        PureProgram::CheckedMul(3, 3),
    ];
    let a = execute(&programs).unwrap();
    let b = execute(&programs).unwrap();
    assert_eq!(a.report, b.report);
    assert_eq!(a.values, b.values);
    assert!(a.replay_verified && b.replay_verified);
}
