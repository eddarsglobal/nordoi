//! R0.3: bounded exploration of the frozen R0.2 pure reference model.
//! This test-only file is NOT exported by lib.rs, runs no tasks and grants
//! no authority to NORDOI source. All bounds and omissions are explicit.

#[allow(dead_code)] // The frozen R0.2 model exposes APIs outside this independent test.
#[path = "../src/resource_task_r02.rs"]
mod reference;

use std::collections::{BTreeSet, VecDeque};

use reference::{
    Command, GrantId, HostPermit, Model, Receipt, ResourceEffect, ResourceId, ScopeBudget, ScopeId,
    ScopeOutcome, TaskId, TaskOutcome, TaskPhase,
};

const DOMAIN: u64 = 0x5230_3353;
const MAX_VISITED: usize = 20_000;
const MAX_EVENTS: usize = 8;

#[derive(Clone, Copy, Debug)]
struct Profile {
    tasks: usize,
    resources: usize,
    children: usize,
    depth: usize,
}

#[derive(Clone, Debug)]
struct Node {
    model: Model,
    scopes: Vec<ScopeId>,
    tasks: Vec<(TaskId, ScopeId)>,
    resources: Vec<(ResourceId, ScopeId, bool)>,
    grants: Vec<(GrantId, ScopeId)>,
    trace: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
enum Action {
    Model(Command),
    HostGrant { scope: ScopeId },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Summary {
    states: usize,
    accepted: usize,
    rejected: usize,
    closed: usize,
    replayed: usize,
}

fn bootstrap(profile: Profile) -> (Node, HostPermit) {
    let (model, permit) = Model::bootstrap(
        DOMAIN,
        ScopeBudget {
            tasks: profile.tasks,
            resources: profile.resources,
            children: profile.children,
        },
        MAX_EVENTS,
    );
    let root = model.root();
    (
        Node {
            model,
            scopes: vec![root],
            tasks: Vec::new(),
            resources: Vec::new(),
            grants: Vec::new(),
            trace: Vec::new(),
        },
        permit,
    )
}

fn actions(node: &Node, profile: Profile) -> Vec<Action> {
    let mut result = Vec::new();
    let root = node.model.root();
    let read = ResourceEffect::Read;
    for &scope in &node.scopes {
        result.push(Action::Model(Command::Close { scope }));
        // Also attempt duplicates and operations against closed scopes: failures
        // must leave the entire model, counters and causal log unchanged.
        if profile.resources > 0 {
            result.push(Action::Model(Command::DeclareEffect {
                scope,
                effect: read,
            }));
            if node.grants.is_empty() {
                result.push(Action::HostGrant { scope });
            }
            for &(grant, _) in &node.grants {
                result.push(Action::Model(Command::Acquire {
                    scope,
                    grant,
                    effect: read,
                }));
            }
        }
        if profile.tasks > 0 && node.tasks.len() < profile.tasks {
            result.push(Action::Model(Command::DeclareTask { scope }));
        }
    }
    if profile.children > 0 && node.scopes.len() < profile.children + 1 {
        result.push(Action::Model(Command::OpenChild {
            parent: root,
            budget: ScopeBudget {
                tasks: profile.tasks,
                resources: profile.resources,
                children: 0,
            },
        }));
    }
    for &(task, _) in &node.tasks {
        // Invalid transitions are deliberately included at every phase.
        result.extend(
            [
                Command::Admit { task },
                Command::Start { task },
                Command::RequestCancel { task },
                Command::Succeed { task },
                Command::Fail { task, code: 7 },
                Command::AcknowledgeCancel { task },
                Command::Join { task },
            ]
            .into_iter()
            .map(Action::Model),
        );
    }
    for &(resource, owner, _) in &node.resources {
        result.push(Action::Model(Command::Release {
            scope: owner,
            resource,
        }));
        for &(grant, _) in &node.grants {
            result.push(Action::Model(Command::Use {
                scope: owner,
                resource,
                grant,
                effect: read,
            }));
        }
        // Cross-scope access attempts cannot transfer ownership.
        for &other in &node.scopes {
            if other != owner {
                result.push(Action::Model(Command::Release {
                    scope: other,
                    resource,
                }));
            }
        }
    }
    for &(grant, _) in &node.grants {
        result.push(Action::Model(Command::RevokeGrant { grant }));
    }
    result
}

fn validate_closed_report(node: &Node, scope: ScopeId, trace: &[String]) {
    let report = node
        .model
        .report(scope)
        .expect("closed receipt has a report");
    assert_eq!(report.scope, scope, "trace={trace:?}");
    assert!(report.tasks.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert!(report.children.windows(2).all(|pair| pair[0].0 < pair[1].0));
    for &(task, owner) in &node.tasks {
        if owner == scope {
            assert_eq!(
                node.model.phase(task),
                Some(TaskPhase::Joined),
                "trace={trace:?}"
            );
            let outcome = node.model.outcome(task).expect("joined task has outcome");
            assert!(report.tasks.contains(&(task, outcome)), "trace={trace:?}");
        }
    }
    assert_eq!(
        report.tasks.len(),
        node.tasks
            .iter()
            .filter(|(_, owner)| *owner == scope)
            .count(),
        "trace={trace:?}"
    );
    for &(resource, owner, live) in &node.resources {
        if owner == scope {
            assert!(
                !live,
                "resource {resource:?} live at close; trace={trace:?}"
            );
        }
    }
    for &child in &node.scopes {
        if node.model.parent(child) == Some(scope) {
            let child_report = node.model.report(child).expect("child must close first");
            assert!(
                report.children.contains(&(child, child_report.outcome)),
                "trace={trace:?}"
            );
        }
    }
    let any_failure = report
        .tasks
        .iter()
        .any(|(_, outcome)| matches!(outcome, TaskOutcome::Failed(_)))
        || report
            .children
            .iter()
            .any(|(_, outcome)| *outcome == ScopeOutcome::Failed);
    let any_cancel = report
        .tasks
        .iter()
        .any(|(_, outcome)| *outcome == TaskOutcome::Cancelled)
        || report
            .children
            .iter()
            .any(|(_, outcome)| *outcome == ScopeOutcome::Cancelled);
    let expected = if any_failure {
        ScopeOutcome::Failed
    } else if any_cancel {
        ScopeOutcome::Cancelled
    } else {
        ScopeOutcome::Succeeded
    };
    assert_eq!(report.outcome, expected, "trace={trace:?}");
}

fn explore(profile: Profile) -> Summary {
    assert!(profile.depth <= MAX_EVENTS);
    let (initial, permit) = bootstrap(profile);
    let mut visited = BTreeSet::new();
    let mut pending = VecDeque::new();
    visited.insert(format!("{:?}", initial.model));
    pending.push_back(initial);
    let mut summary = Summary::default();

    while let Some(node) = pending.pop_front() {
        summary.states += 1;
        assert!(
            summary.states <= MAX_VISITED,
            "bound exceeded (not a proof): {profile:?}"
        );
        if node.trace.len() >= profile.depth {
            continue;
        }
        for action in actions(&node, profile) {
            let mut next = node.clone();
            let before = next.model.clone();
            let previous_events = before.event_count();
            let outcome = match action {
                Action::Model(command) => next.model.apply(command).map(Applied::Command),
                Action::HostGrant { scope } => next
                    .model
                    .host_grant(&permit, scope, ResourceEffect::Read)
                    .map(Applied::Grant),
            };
            match outcome {
                Err(_) => {
                    summary.rejected += 1;
                    assert_eq!(
                        next.model, before,
                        "rejected transition mutated state: {action:?}; trace={:?}",
                        node.trace
                    );
                    assert_eq!(next.model.event_count(), previous_events);
                }
                Ok(receipt) => {
                    summary.accepted += 1;
                    assert_eq!(
                        next.model.event_count(),
                        previous_events + 1,
                        "{action:?}; trace={:?}",
                        node.trace
                    );
                    next.trace.push(format!("{action:?} -> {receipt:?}"));
                    match (action, &receipt) {
                        (
                            Action::Model(Command::OpenChild { .. }),
                            Applied::Command(Receipt::Scope(id)),
                        ) => {
                            next.scopes.push(*id);
                        }
                        (
                            Action::Model(Command::DeclareTask { scope }),
                            Applied::Command(Receipt::Task(id)),
                        ) => {
                            next.tasks.push((*id, scope));
                        }
                        (
                            Action::Model(Command::Acquire { scope, .. }),
                            Applied::Command(Receipt::Resource(id)),
                        ) => {
                            next.resources.push((*id, scope, true));
                        }
                        (
                            Action::Model(Command::Release { resource, .. }),
                            Applied::Command(Receipt::Unit),
                        ) => {
                            let (_, _, live) = next
                                .resources
                                .iter_mut()
                                .find(|(id, _, _)| *id == resource)
                                .expect("accepted release must have a tracked resource");
                            assert!(*live, "duplicate accepted release; trace={:?}", next.trace);
                            *live = false;
                        }
                        (Action::HostGrant { scope }, Applied::Grant(id)) => {
                            next.grants.push((*id, scope));
                        }
                        (
                            Action::Model(Command::Close { scope }),
                            Applied::Command(Receipt::Closed(_)),
                        ) => {
                            validate_closed_report(&next, scope, &next.trace);
                            summary.closed += 1;
                        }
                        (_, Applied::Command(Receipt::Unit)) => {}
                        (unexpected_action, unexpected_receipt) => {
                            panic!("inconsistent receipt {unexpected_receipt:?} for {unexpected_action:?}; trace={:?}", next.trace);
                        }
                    }
                    assert_eq!(
                        next.model.replay_checked(),
                        Ok(()),
                        "non-replayable path={:?}",
                        next.trace
                    );
                    summary.replayed += 1;
                    let fingerprint = format!("{:?}", next.model);
                    if visited.insert(fingerprint) {
                        assert!(
                            visited.len() <= MAX_VISITED,
                            "visited-state budget exceeded: {profile:?}; trace={:?}",
                            next.trace
                        );
                        pending.push_back(next);
                    }
                }
            }
        }
    }
    summary
}

#[derive(Debug)]
enum Applied {
    Command(Receipt),
    Grant(GrantId),
}

#[test]
fn bounded_task_lifecycle_state_space() {
    let profile = Profile {
        tasks: 1,
        resources: 0,
        children: 0,
        depth: 6,
    };
    let report = explore(profile);
    assert!(report.states > 12, "not enough explored states: {report:?}");
    assert!(
        report.rejected > 12,
        "invalid transitions not exercised: {report:?}"
    );
    assert!(
        report.closed > 0,
        "terminal scope closure not exercised: {report:?}"
    );
    assert_eq!(report.accepted, report.replayed);
}

#[test]
fn bounded_resource_and_authority_state_space() {
    let profile = Profile {
        tasks: 0,
        resources: 1,
        children: 0,
        depth: 6,
    };
    let report = explore(profile);
    assert!(report.states > 8, "not enough explored states: {report:?}");
    assert!(
        report.rejected > 8,
        "invalid authority operations not exercised: {report:?}"
    );
    assert!(report.closed > 0);
    assert_eq!(report.accepted, report.replayed);
}

#[test]
fn bounded_child_scope_state_space() {
    let profile = Profile {
        tasks: 0,
        resources: 0,
        children: 1,
        depth: 4,
    };
    let report = explore(profile);
    assert!(
        report.states >= 4,
        "child lifecycle insufficiently explored: {report:?}"
    );
    assert!(report.rejected > 0);
    assert!(report.closed > 0);
}

#[test]
fn bounded_cross_domain_state_space() {
    let profile = Profile {
        tasks: 1,
        resources: 1,
        children: 1,
        depth: 4,
    };
    let report = explore(profile);
    assert!(
        report.states > 10,
        "mixed frontier insufficiently explored: {report:?}"
    );
    assert!(report.rejected > 10);
}

#[test]
fn explorer_repeats_identical_counts_for_identical_inputs() {
    let profile = Profile {
        tasks: 1,
        resources: 0,
        children: 0,
        depth: 5,
    };
    assert_eq!(explore(profile), explore(profile));
}

fn unit(model: &mut Model, command: Command) {
    assert_eq!(model.apply(command), Ok(Receipt::Unit));
}

fn declared_task(model: &mut Model, scope: ScopeId) -> TaskId {
    match model.apply(Command::DeclareTask { scope }) {
        Ok(Receipt::Task(id)) => id,
        other => panic!("failed to declare test task: {other:?}"),
    }
}

fn terminate_join(model: &mut Model, task: TaskId, outcome: TaskOutcome) {
    if outcome == TaskOutcome::Cancelled {
        unit(model, Command::RequestCancel { task });
        unit(model, Command::AcknowledgeCancel { task });
    } else if let TaskOutcome::Failed(code) = outcome {
        unit(model, Command::Fail { task, code });
    } else {
        unit(model, Command::Succeed { task });
    }
    unit(model, Command::Join { task });
}

#[test]
fn all_two_task_completion_and_join_orders_keep_canonical_report() {
    let cases = [
        [TaskOutcome::Succeeded, TaskOutcome::Succeeded],
        [TaskOutcome::Failed(9), TaskOutcome::Succeeded],
        [TaskOutcome::Cancelled, TaskOutcome::Succeeded],
        [TaskOutcome::Cancelled, TaskOutcome::Failed(3)],
        [TaskOutcome::Failed(5), TaskOutcome::Failed(3)],
    ];
    for outcomes in cases {
        let mut baseline = None;
        for finish_order in [[0_usize, 1], [1, 0]] {
            let (mut model, _) = Model::bootstrap(
                DOMAIN,
                ScopeBudget {
                    tasks: 2,
                    resources: 0,
                    children: 0,
                },
                32,
            );
            let root = model.root();
            let tasks = [
                declared_task(&mut model, root),
                declared_task(&mut model, root),
            ];
            for task in tasks {
                unit(&mut model, Command::Admit { task });
            }
            for index in finish_order {
                terminate_join(&mut model, tasks[index], outcomes[index]);
            }
            let report = match model.apply(Command::Close { scope: root }) {
                Ok(Receipt::Closed(report)) => report,
                other => panic!("unexpected close receipt: {other:?}"),
            };
            assert_eq!(
                report.tasks,
                vec![(tasks[0], outcomes[0]), (tasks[1], outcomes[1])]
            );
            assert_eq!(model.replay_checked(), Ok(()));
            if let Some(expected) = &baseline {
                assert_eq!(&report, expected, "completion order altered report");
            } else {
                baseline = Some(report);
            }
        }
    }
}

#[test]
fn cross_domain_identity_cannot_act_as_scope_or_grant() {
    let (mut model, _) = Model::bootstrap(
        DOMAIN,
        ScopeBudget {
            tasks: 1,
            resources: 1,
            children: 0,
        },
        12,
    );
    let (mut foreign, foreign_permit) = Model::bootstrap(
        DOMAIN + 1,
        ScopeBudget {
            tasks: 1,
            resources: 1,
            children: 0,
        },
        12,
    );
    let foreign_root = foreign.root();
    let foreign_grant = foreign
        .host_grant(&foreign_permit, foreign_root, ResourceEffect::Read)
        .expect("foreign setup");
    let root = model.root();
    unit(
        &mut model,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let prior = model.clone();
    assert!(model
        .apply(Command::DeclareTask {
            scope: foreign_root
        })
        .is_err());
    assert_eq!(
        model.apply(Command::Acquire {
            scope: root,
            grant: foreign_grant,
            effect: ResourceEffect::Read,
        }),
        Err(reference::ModelError::UnknownGrant)
    );
    assert_eq!(model, prior, "domain misuse mutated the model");
}

#[test]
fn accepted_event_budget_exhaustion_is_atomic() {
    let (mut model, _) = Model::bootstrap(
        DOMAIN,
        ScopeBudget {
            tasks: 1,
            resources: 0,
            children: 0,
        },
        1,
    );
    let root = model.root();
    let task = declared_task(&mut model, root);
    let before = model.clone();
    assert!(model.apply(Command::Admit { task }).is_err());
    assert_eq!(model, before);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn r03_remains_test_only_not_a_public_runtime_or_nair_extension() {
    let lib = include_str!("../src/lib.rs");
    let cargo = include_str!("../Cargo.toml");
    assert!(!lib.contains("resource_task_r03"));
    assert!(!lib.contains("resource_task_r02"));
    assert!(!cargo.contains("resource_task_r03"));
}
