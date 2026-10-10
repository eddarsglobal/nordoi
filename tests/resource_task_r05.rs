//! R0.5 — resource/task hardening witnesses for the frozen R0.2 reference model.
//! TEST-ONLY: no scheduler, production authority, concurrency runtime or formal proof.

#[allow(dead_code)] // Independent R0.2 reference API intentionally exposes more than these tests use.
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{
    Command, Model, ModelError, Receipt, ResourceEffect, ResourceId, ScopeBudget, ScopeId,
    ScopeOutcome, ScopeReport, TaskId, TaskOutcome, TaskPhase,
};

const DOMAIN: u64 = 0x5230_3556;
const MAX_EVENTS: usize = 96;
const EXPECTED_INTERLEAVINGS: usize = 90;

fn budget(tasks: usize, resources: usize, children: usize) -> ScopeBudget {
    ScopeBudget {
        tasks,
        resources,
        children,
    }
}

fn unit(model: &mut Model, command: Command) {
    assert_eq!(
        model.apply(command),
        Ok(Receipt::Unit),
        "command={command:?}"
    );
    assert_eq!(model.replay_checked(), Ok(()));
}

fn declare_task(model: &mut Model, scope: ScopeId) -> TaskId {
    match model.apply(Command::DeclareTask { scope }) {
        Ok(Receipt::Task(id)) => id,
        other => panic!("task creation failed: {other:?}"),
    }
}

fn open_child(model: &mut Model, parent: ScopeId, limit: ScopeBudget) -> ScopeId {
    match model.apply(Command::OpenChild {
        parent,
        budget: limit,
    }) {
        Ok(Receipt::Scope(id)) => id,
        other => panic!("child creation failed: {other:?}"),
    }
}

fn acquire(model: &mut Model, scope: ScopeId, grant: reference::GrantId) -> ResourceId {
    match model.apply(Command::Acquire {
        scope,
        grant,
        effect: ResourceEffect::Read,
    }) {
        Ok(Receipt::Resource(id)) => id,
        other => panic!("acquisition failed: {other:?}"),
    }
}

fn close(model: &mut Model, scope: ScopeId) -> ScopeReport {
    match model.apply(Command::Close { scope }) {
        Ok(Receipt::Closed(report)) => report,
        other => panic!("close failed: {other:?}"),
    }
}

fn rejected(model: &mut Model, command: Command, error: ModelError) {
    let before = model.clone();
    let count = model.event_count();
    assert_eq!(model.apply(command), Err(error), "command={command:?}");
    assert_eq!(
        *model, before,
        "rejected command changed state: {command:?}"
    );
    assert_eq!(model.event_count(), count);
    assert_eq!(model.replay_checked(), Ok(()));
}

fn setup_three_tasks() -> (Model, [TaskId; 3]) {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(3, 0, 0), MAX_EVENTS);
    let root = model.root();
    let tasks = [
        declare_task(&mut model, root),
        declare_task(&mut model, root),
        declare_task(&mut model, root),
    ];
    for task in tasks {
        unit(&mut model, Command::Admit { task });
    }
    unit(&mut model, Command::RequestCancel { task: tasks[2] });
    (model, tasks)
}

// Six operations: terminal + join for each of three tasks. The only ordering
// constraint is that join follows its OWN terminal event. There are 6!/2^3 = 90.
fn walk_interleavings(
    model: Model,
    tasks: [TaskId; 3],
    progress: [u8; 3],
    trace: &mut Vec<String>,
    count: &mut usize,
    expected: &mut Option<ScopeReport>,
) {
    if progress == [2, 2, 2] {
        let mut finished = model;
        let root = finished.root();
        let report = close(&mut finished, root);
        assert_eq!(report.outcome, ScopeOutcome::Failed, "trace={trace:?}");
        assert_eq!(
            report.tasks,
            vec![
                (tasks[0], TaskOutcome::Succeeded),
                (tasks[1], TaskOutcome::Failed(23)),
                (tasks[2], TaskOutcome::Cancelled),
            ],
            "trace={trace:?}"
        );
        assert_eq!(finished.replay_checked(), Ok(()), "trace={trace:?}");
        if let Some(prior) = expected {
            assert_eq!(&report, &*prior, "report drift: trace={trace:?}");
        } else {
            *expected = Some(report);
        }
        *count += 1;
        return;
    }
    for index in 0..3 {
        if progress[index] == 2 {
            continue;
        }
        let task = tasks[index];
        let operation = match (index, progress[index]) {
            (0, 0) => Command::Succeed { task },
            (1, 0) => Command::Fail { task, code: 23 },
            (2, 0) => Command::AcknowledgeCancel { task },
            (_, 1) => Command::Join { task },
            _ => unreachable!("progress is bounded to 0..=2"),
        };
        let mut next = model.clone();
        let prior_count = next.event_count();
        assert_eq!(next.apply(operation), Ok(Receipt::Unit), "trace={trace:?}");
        assert_eq!(next.event_count(), prior_count + 1);
        assert_eq!(next.replay_checked(), Ok(()), "trace={trace:?}");
        let mut advanced = progress;
        advanced[index] += 1;
        trace.push(format!("{operation:?}"));
        walk_interleavings(next, tasks, advanced, trace, count, expected);
        trace.pop();
    }
}

#[test]
fn ninety_valid_three_task_interleavings_have_same_closure_report() {
    let (model, tasks) = setup_three_tasks();
    let mut count = 0;
    let mut expected = None;
    walk_interleavings(
        model,
        tasks,
        [0, 0, 0],
        &mut Vec::new(),
        &mut count,
        &mut expected,
    );
    assert_eq!(count, EXPECTED_INTERLEAVINGS);
    assert!(expected.is_some());
    println!("R05_WITNESS version=1 kind=three_task_interleavings explored={count} limit={EXPECTED_INTERLEAVINGS} outcome=PASS");
}

#[test]
fn nested_scope_failure_dominates_cancellation_for_both_close_orders() {
    let mut canonical = None;
    for order in [[0_usize, 1], [1, 0]] {
        let (mut model, _) = Model::bootstrap(DOMAIN, budget(0, 0, 2), MAX_EVENTS);
        let root = model.root();
        let middle = open_child(&mut model, root, budget(0, 0, 1));
        let leaf = open_child(&mut model, middle, budget(1, 0, 0));
        let sibling = open_child(&mut model, root, budget(1, 0, 0));
        let failed = declare_task(&mut model, leaf);
        let cancelled = declare_task(&mut model, sibling);
        unit(&mut model, Command::Admit { task: failed });
        unit(&mut model, Command::Admit { task: cancelled });
        unit(
            &mut model,
            Command::Fail {
                task: failed,
                code: 9,
            },
        );
        unit(&mut model, Command::RequestCancel { task: cancelled });
        unit(&mut model, Command::AcknowledgeCancel { task: cancelled });
        unit(&mut model, Command::Join { task: failed });
        unit(&mut model, Command::Join { task: cancelled });
        rejected(
            &mut model,
            Command::Close { scope: root },
            ModelError::OutstandingChildren,
        );
        rejected(
            &mut model,
            Command::Close { scope: middle },
            ModelError::OutstandingChildren,
        );
        for index in order {
            if index == 0 {
                assert_eq!(close(&mut model, leaf).outcome, ScopeOutcome::Failed);
                assert_eq!(close(&mut model, middle).outcome, ScopeOutcome::Failed);
            } else {
                assert_eq!(close(&mut model, sibling).outcome, ScopeOutcome::Cancelled);
            }
        }
        let report = close(&mut model, root);
        assert_eq!(report.outcome, ScopeOutcome::Failed);
        assert_eq!(
            report.children,
            vec![
                (middle, ScopeOutcome::Failed),
                (sibling, ScopeOutcome::Cancelled),
            ]
        );
        assert_eq!(model.replay_checked(), Ok(()));
        if let Some(previous) = &canonical {
            assert_eq!(&report, previous);
        } else {
            canonical = Some(report);
        }
    }
}

#[test]
fn cross_scope_grants_do_not_transfer_resource_authority() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(0, 1, 1), MAX_EVENTS);
    let root = model.root();
    let child = open_child(&mut model, root, budget(0, 1, 0));
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: child,
            effect: ResourceEffect::Read,
        },
    );
    let host_root = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("root grant");
    let host_child = model
        .host_grant(&permit, child, ResourceEffect::Read)
        .expect("child grant");
    rejected(
        &mut model,
        Command::Acquire {
            scope: root,
            grant: host_child,
            effect: ResourceEffect::Read,
        },
        ModelError::GrantMismatch,
    );
    rejected(
        &mut model,
        Command::Acquire {
            scope: child,
            grant: host_root,
            effect: ResourceEffect::Read,
        },
        ModelError::GrantMismatch,
    );
    let resource = acquire(&mut model, child, host_child);
    rejected(
        &mut model,
        Command::Use {
            scope: root,
            resource,
            grant: host_root,
            effect: ResourceEffect::Read,
        },
        ModelError::WrongScope,
    );
    rejected(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
        ModelError::WrongScope,
    );
    unit(
        &mut model,
        Command::Release {
            scope: child,
            resource,
        },
    );
    close(&mut model, child);
    close(&mut model, root);
}

#[test]
fn revocation_denies_future_use_but_cannot_block_cleanup() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(0, 1, 0), MAX_EVENTS);
    let root = model.root();
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("host grant");
    let resource = acquire(&mut model, root, grant);
    unit(&mut model, Command::RevokeGrant { grant });
    rejected(
        &mut model,
        Command::Use {
            scope: root,
            resource,
            grant,
            effect: ResourceEffect::Read,
        },
        ModelError::RevokedGrant,
    );
    rejected(
        &mut model,
        Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read,
        },
        ModelError::RevokedGrant,
    );
    unit(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
    );
    assert_eq!(close(&mut model, root).outcome, ScopeOutcome::Succeeded);
}

#[test]
fn released_handle_cannot_reanimate_or_double_release() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(0, 1, 0), MAX_EVENTS);
    let root = model.root();
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("host grant");
    let resource = acquire(&mut model, root, grant);
    unit(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
    );
    rejected(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
        ModelError::ReleasedResource,
    );
    rejected(
        &mut model,
        Command::Use {
            scope: root,
            resource,
            grant,
            effect: ResourceEffect::Read,
        },
        ModelError::ReleasedResource,
    );
    close(&mut model, root);
    rejected(
        &mut model,
        Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read,
        },
        ModelError::ClosedScope,
    );
}

#[test]
fn resource_and_task_budgets_are_cumulative_not_reusable() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(1, 1, 0), MAX_EVENTS);
    let root = model.root();
    let task = declare_task(&mut model, root);
    rejected(
        &mut model,
        Command::DeclareTask { scope: root },
        ModelError::Exhausted,
    );
    unit(&mut model, Command::Admit { task });
    unit(&mut model, Command::Succeed { task });
    unit(&mut model, Command::Join { task });
    rejected(
        &mut model,
        Command::DeclareTask { scope: root },
        ModelError::Exhausted,
    );
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("host grant");
    let resource = acquire(&mut model, root, grant);
    unit(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
    );
    rejected(
        &mut model,
        Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read,
        },
        ModelError::Exhausted,
    );
    close(&mut model, root);
}

#[test]
fn event_budget_zero_one_and_two_fail_closed_without_partial_logs() {
    for max_events in 0..=2 {
        let (mut model, permit) = Model::bootstrap(DOMAIN, budget(1, 1, 0), max_events);
        let root = model.root();
        if max_events > 0 {
            unit(
                &mut model,
                Command::DeclareEffect {
                    scope: root,
                    effect: ResourceEffect::Read,
                },
            );
        }
        if max_events > 1 {
            model
                .host_grant(&permit, root, ResourceEffect::Read)
                .expect("budgeted host grant");
        }
        let snapshot = model.clone();
        rejected(
            &mut model,
            Command::DeclareTask { scope: root },
            ModelError::Exhausted,
        );
        assert_eq!(model.event_count(), max_events);
        assert_eq!(model, snapshot);
    }
}

#[test]
fn cancellation_then_failure_requires_join_and_reports_failure() {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(1, 0, 0), MAX_EVENTS);
    let root = model.root();
    let task = declare_task(&mut model, root);
    unit(&mut model, Command::Admit { task });
    unit(&mut model, Command::RequestCancel { task });
    rejected(
        &mut model,
        Command::Succeed { task },
        ModelError::InvalidTransition,
    );
    unit(&mut model, Command::Fail { task, code: 11 });
    rejected(
        &mut model,
        Command::AcknowledgeCancel { task },
        ModelError::InvalidTransition,
    );
    rejected(
        &mut model,
        Command::Close { scope: root },
        ModelError::OutstandingTasks,
    );
    unit(&mut model, Command::Join { task });
    let report = close(&mut model, root);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert_eq!(report.tasks, vec![(task, TaskOutcome::Failed(11))]);
}

#[test]
fn invalid_task_phase_edges_do_not_mutate_state() {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(1, 0, 0), MAX_EVENTS);
    let root = model.root();
    let task = declare_task(&mut model, root);
    for command in [
        Command::Start { task },
        Command::Succeed { task },
        Command::Join { task },
        Command::RequestCancel { task },
        Command::AcknowledgeCancel { task },
    ] {
        rejected(&mut model, command, ModelError::InvalidTransition);
    }
    unit(&mut model, Command::Admit { task });
    unit(&mut model, Command::Start { task });
    rejected(
        &mut model,
        Command::Start { task },
        ModelError::InvalidTransition,
    );
    unit(&mut model, Command::Succeed { task });
    rejected(
        &mut model,
        Command::Succeed { task },
        ModelError::InvalidTransition,
    );
    unit(&mut model, Command::Join { task });
    assert_eq!(model.phase(task), Some(TaskPhase::Joined));
    rejected(
        &mut model,
        Command::Join { task },
        ModelError::InvalidTransition,
    );
    close(&mut model, root);
}

#[test]
fn foreign_scope_ids_and_host_permits_are_rejected_atomically() {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(1, 0, 0), MAX_EVENTS);
    let (foreign, permit) = Model::bootstrap(DOMAIN + 1, budget(1, 0, 0), MAX_EVENTS);
    rejected(
        &mut model,
        Command::DeclareTask {
            scope: foreign.root(),
        },
        ModelError::UnknownScope,
    );
    let root = model.root();
    let snapshot = model.clone();
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::Read),
        Err(ModelError::WrongHostPermit)
    );
    assert_eq!(model, snapshot);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn synthetic_oracle_mutation_is_detected_without_claiming_real_defect() {
    let (mut model, _) = setup_three_tasks();
    // No introspection or monkey patching of the reference implementation.
    // The synthetic mutation modifies the *test's copy* of the report only.
    // Finish the model using one admissible order.
    let tasks = {
        let (other, tasks) = setup_three_tasks();
        assert_eq!(model, other);
        tasks
    };
    unit(&mut model, Command::Succeed { task: tasks[0] });
    unit(
        &mut model,
        Command::Fail {
            task: tasks[1],
            code: 23,
        },
    );
    unit(&mut model, Command::AcknowledgeCancel { task: tasks[2] });
    for task in tasks {
        unit(&mut model, Command::Join { task });
    }
    let root = model.root();
    let truth = close(&mut model, root);
    let mut changed = truth.clone();
    changed.tasks.reverse();
    assert_ne!(
        changed, truth,
        "the synthetic ordering fault was not detected"
    );
    changed = truth.clone();
    changed.outcome = ScopeOutcome::Succeeded;
    assert_ne!(
        changed, truth,
        "the synthetic failure-precedence fault was not detected"
    );
}

fn has_rejection(commands: &[Command]) -> bool {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(3, 0, 0), MAX_EVENTS);
    commands
        .iter()
        .any(|&command| model.apply(command).is_err())
}

fn shrink_invalid_trace(mut commands: Vec<Command>) -> Vec<Command> {
    assert!(has_rejection(&commands));
    loop {
        let mut removed = false;
        for index in 0..commands.len() {
            let mut candidate = commands.clone();
            candidate.remove(index);
            if has_rejection(&candidate) {
                commands = candidate;
                removed = true;
                break;
            }
        }
        if !removed {
            return commands;
        }
    }
}

#[test]
fn real_model_negative_trace_shrinks_to_minimal_rejection() {
    let (model, _) = Model::bootstrap(DOMAIN, budget(3, 0, 0), MAX_EVENTS);
    let root = model.root();
    let bad = Command::DeclareEffect {
        scope: root,
        effect: ResourceEffect::ExternalIo,
    };
    let original = vec![
        Command::DeclareTask { scope: root },
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
        bad,
        Command::DeclareTask { scope: root },
    ];
    let reduced = shrink_invalid_trace(original);
    assert_eq!(reduced, vec![bad]);
    assert!(has_rejection(&reduced));
    assert!(!has_rejection(&[]));
    assert_eq!(shrink_invalid_trace(reduced.clone()), reduced);
    // This minimizes an expected model rejection, NOT a discovered implementation bug.
}

#[test]
fn identical_scripts_produce_identical_model_state_and_receipts() {
    fn run() -> (Model, Vec<String>) {
        let (mut model, permit) = Model::bootstrap(DOMAIN, budget(1, 1, 0), MAX_EVENTS);
        let root = model.root();
        let mut receipts = Vec::new();
        let declaration = Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        };
        receipts.push(format!("{:?}", model.apply(declaration).expect("declare")));
        let grant = model
            .host_grant(&permit, root, ResourceEffect::Read)
            .expect("grant");
        receipts.push(format!("grant={:?}", grant.parts()));
        let resource = acquire(&mut model, root, grant);
        receipts.push(format!("resource={:?}", resource.parts()));
        unit(
            &mut model,
            Command::Release {
                scope: root,
                resource,
            },
        );
        receipts.push(format!("{:?}", close(&mut model, root)));
        assert_eq!(model.replay_checked(), Ok(()));
        (model, receipts)
    }
    assert_eq!(run(), run());
}

#[test]
fn r05_remains_test_only_without_runtime_or_nair_exports() {
    let library = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");
    for forbidden in [
        "resource_task_r02",
        "resource_task_r03",
        "resource_task_r04",
        "resource_task_r05",
    ] {
        assert!(
            !library.contains(forbidden),
            "reference model leaked to library: {forbidden}"
        );
        assert!(
            !manifest.contains(forbidden),
            "reference model leaked to manifest: {forbidden}"
        );
    }
    let (model, _) = Model::bootstrap(DOMAIN, budget(0, 0, 0), MAX_EVENTS);
    assert_eq!(model.event_count(), 0);
    assert_eq!(model.replay_checked(), Ok(()));
}
