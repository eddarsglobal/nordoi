#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{
    Command, GrantId, HostPermit, Model, ModelError, Receipt, ResourceEffect, ResourceId,
    ScopeBudget, ScopeId, ScopeOutcome, TaskId, TaskOutcome, TaskPhase,
};

fn budget() -> ScopeBudget {
    ScopeBudget {
        tasks: 3,
        resources: 3,
        children: 2,
    }
}

fn setup() -> (Model, HostPermit, ScopeId) {
    let (model, host) = Model::bootstrap(20261009, budget(), 100);
    let root = model.root();
    (model, host, root)
}

fn unit(model: &mut Model, command: Command) {
    assert_eq!(model.apply(command), Ok(Receipt::Unit));
}

fn task(model: &mut Model, scope: ScopeId) -> TaskId {
    match model.apply(Command::DeclareTask { scope }).unwrap() {
        Receipt::Task(id) => id,
        unexpected => panic!("expected task, got {unexpected:?}"),
    }
}

fn child(model: &mut Model, scope: ScopeId) -> ScopeId {
    match model
        .apply(Command::OpenChild {
            parent: scope,
            budget: budget(),
        })
        .unwrap()
    {
        Receipt::Scope(id) => id,
        unexpected => panic!("expected child scope, got {unexpected:?}"),
    }
}

fn resource(
    model: &mut Model,
    scope: ScopeId,
    grant: GrantId,
    effect: ResourceEffect,
) -> ResourceId {
    match model
        .apply(Command::Acquire {
            scope,
            grant,
            effect,
        })
        .unwrap()
    {
        Receipt::Resource(id) => id,
        unexpected => panic!("expected resource, got {unexpected:?}"),
    }
}

fn finish_join(model: &mut Model, id: TaskId, command: Command) {
    unit(model, command);
    unit(model, Command::Join { task: id });
}

fn close(model: &mut Model, scope: ScopeId) -> reference::ScopeReport {
    match model.apply(Command::Close { scope }).unwrap() {
        Receipt::Closed(report) => report,
        unexpected => panic!("expected scope report, got {unexpected:?}"),
    }
}

#[test]
fn i01_acquire_requires_explicit_grant() {
    let (mut m, host, scope) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Read,
        },
    );
    let before = m.clone();
    let incorrect_grant = m.host_grant(&host, scope, ResourceEffect::Write).unwrap();
    assert_eq!(
        m.apply(Command::Acquire {
            scope,
            grant: incorrect_grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::GrantMismatch)
    );
    // The failed acquire never adds a resource. The successful host grant is an
    // independently authorized and recorded step, not a guest-created grant.
    assert_eq!(m.event_count(), before.event_count() + 1);
    assert!(m.replay_checked().is_ok());
}

#[test]
fn i01_host_witness_is_scope_domain_specific() {
    let (mut one, permit_one, root_one) = setup();
    let (two, permit_two) = Model::bootstrap(444, budget(), 100);
    assert_eq!(
        one.host_grant(&permit_two, root_one, ResourceEffect::Read),
        Err(ModelError::WrongHostPermit)
    );
    assert_eq!(one.event_count(), 0);
    assert!(one
        .host_grant(&permit_one, root_one, ResourceEffect::Read)
        .is_ok());
    assert_ne!(one.root(), two.root());
}

#[test]
fn i02_owner_is_unique_across_scopes() {
    let (mut m, host, root) = setup();
    let nested = child(&mut m, root);
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: nested,
            effect: ResourceEffect::Write,
        },
    );
    let grant = m.host_grant(&host, nested, ResourceEffect::Write).unwrap();
    let handle = resource(&mut m, nested, grant, ResourceEffect::Write);
    assert_eq!(
        m.apply(Command::Use {
            scope: root,
            resource: handle,
            grant,
            effect: ResourceEffect::Write
        }),
        Err(ModelError::MissingDeclaration)
    );
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Write,
        },
    );
    assert_eq!(
        m.apply(Command::Release {
            scope: root,
            resource: handle
        }),
        Err(ModelError::WrongScope)
    );
    unit(
        &mut m,
        Command::Release {
            scope: nested,
            resource: handle,
        },
    );
    assert_eq!(m.parent(nested), Some(root));
    close(&mut m, nested);
    close(&mut m, root);
}

#[test]
fn i03_double_release_rejected_atomically() {
    let (mut m, host, scope) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Read,
        },
    );
    let grant = m.host_grant(&host, scope, ResourceEffect::Read).unwrap();
    let handle = resource(&mut m, scope, grant, ResourceEffect::Read);
    unit(
        &mut m,
        Command::Release {
            scope,
            resource: handle,
        },
    );
    let before = m.clone();
    assert_eq!(
        m.apply(Command::Release {
            scope,
            resource: handle
        }),
        Err(ModelError::ReleasedResource)
    );
    assert_eq!(m, before);
    close(&mut m, scope);
}

#[test]
fn i04_no_use_after_release() {
    let (mut m, host, scope) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Write,
        },
    );
    let grant = m.host_grant(&host, scope, ResourceEffect::Write).unwrap();
    let handle = resource(&mut m, scope, grant, ResourceEffect::Write);
    unit(
        &mut m,
        Command::Use {
            scope,
            resource: handle,
            grant,
            effect: ResourceEffect::Write,
        },
    );
    unit(
        &mut m,
        Command::Release {
            scope,
            resource: handle,
        },
    );
    assert_eq!(
        m.apply(Command::Use {
            scope,
            resource: handle,
            grant,
            effect: ResourceEffect::Write
        }),
        Err(ModelError::ReleasedResource)
    );
}

#[test]
fn i05_effect_must_be_declared() {
    let (mut m, host, scope) = setup();
    let grant = m.host_grant(&host, scope, ResourceEffect::Read).unwrap();
    assert_eq!(
        m.apply(Command::Acquire {
            scope,
            grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::MissingDeclaration)
    );
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Read,
        },
    );
    let handle = resource(&mut m, scope, grant, ResourceEffect::Read);
    unit(
        &mut m,
        Command::Release {
            scope,
            resource: handle,
        },
    );
}

#[test]
fn i06_no_external_io_or_irreversible_rollback_claim() {
    let (mut m, host, scope) = setup();
    let before = m.clone();
    assert_eq!(
        m.host_grant(&host, scope, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(
        m.apply(Command::DeclareEffect {
            scope,
            effect: ResourceEffect::ExternalIo
        }),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(m, before);
}

#[test]
fn i07_children_cannot_outlive_parent_scope() {
    let (mut m, _, root) = setup();
    let child_scope = child(&mut m, root);
    assert_eq!(
        m.apply(Command::Close { scope: root }),
        Err(ModelError::OutstandingChildren)
    );
    close(&mut m, child_scope);
    close(&mut m, root);
    assert_eq!(
        m.apply(Command::OpenChild {
            parent: root,
            budget: budget()
        }),
        Err(ModelError::ClosedScope)
    );
}

#[test]
fn i08_cancel_request_is_not_silent_success() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    unit(&mut m, Command::Admit { task: id });
    unit(&mut m, Command::Start { task: id });
    unit(&mut m, Command::RequestCancel { task: id });
    assert_eq!(
        m.apply(Command::Succeed { task: id }),
        Err(ModelError::InvalidTransition)
    );
    assert_eq!(
        m.apply(Command::Join { task: id }),
        Err(ModelError::InvalidTransition)
    );
    finish_join(&mut m, id, Command::AcknowledgeCancel { task: id });
    assert_eq!(m.outcome(id), Some(TaskOutcome::Cancelled));
    let report = close(&mut m, root);
    assert_eq!(report.outcome, ScopeOutcome::Cancelled);
}

#[test]
fn i09_failure_is_propagated_and_accounted() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    unit(&mut m, Command::Admit { task: id });
    finish_join(&mut m, id, Command::Fail { task: id, code: 17 });
    let report = close(&mut m, root);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert_eq!(report.tasks, vec![(id, TaskOutcome::Failed(17))]);
    assert_eq!(m.phase(id), Some(TaskPhase::Joined));
}

#[test]
fn i10_task_admission_is_bounded() {
    let (mut m, _, root) = setup();
    let a = task(&mut m, root);
    let b = task(&mut m, root);
    let c = task(&mut m, root);
    for id in [a, b, c] {
        unit(&mut m, Command::Admit { task: id });
    }
    let before = m.clone();
    assert_eq!(
        m.apply(Command::DeclareTask { scope: root }),
        Err(ModelError::Exhausted)
    );
    assert_eq!(m, before);
}

fn two_task_run(reverse: bool) -> reference::ScopeReport {
    let (mut m, _, root) = setup();
    let a = task(&mut m, root);
    let b = task(&mut m, root);
    for id in [a, b] {
        unit(&mut m, Command::Admit { task: id });
        unit(&mut m, Command::Start { task: id });
    }
    let order = if reverse { [b, a] } else { [a, b] };
    for id in order {
        unit(
            &mut m,
            if id == a {
                Command::Fail { task: id, code: 9 }
            } else {
                Command::Succeed { task: id }
            },
        );
        unit(&mut m, Command::Join { task: id });
    }
    let report = close(&mut m, root);
    assert_eq!(m.report(root), Some(&report));
    assert!(m.replay_checked().is_ok());
    report
}

#[test]
fn i11_join_is_independent_of_host_completion_order() {
    let first = two_task_run(false);
    let second = two_task_run(true);
    assert_eq!(first, second);
    assert_eq!(first.outcome, ScopeOutcome::Failed);
}

#[test]
fn i12_no_scope_close_with_unjoined_child_task() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    assert_eq!(
        m.apply(Command::Close { scope: root }),
        Err(ModelError::OutstandingTasks)
    );
    unit(&mut m, Command::Admit { task: id });
    unit(&mut m, Command::Succeed { task: id });
    assert_eq!(
        m.apply(Command::Close { scope: root }),
        Err(ModelError::OutstandingTasks)
    );
    unit(&mut m, Command::Join { task: id });
    close(&mut m, root);
}

#[test]
fn i13_model_is_backend_neutral() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    assert_eq!(m.phase(id), Some(TaskPhase::Declared));
    // No threads, promises, executor handles or host descriptors are present.
    assert_eq!(m.event_count(), 1);
}

#[test]
fn i14_accepted_causal_log_replays_exactly() {
    let (mut m, host, root) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = m.host_grant(&host, root, ResourceEffect::Read).unwrap();
    let res = resource(&mut m, root, grant, ResourceEffect::Read);
    unit(
        &mut m,
        Command::Use {
            scope: root,
            resource: res,
            grant,
            effect: ResourceEffect::Read,
        },
    );
    unit(
        &mut m,
        Command::Release {
            scope: root,
            resource: res,
        },
    );
    let id = task(&mut m, root);
    unit(&mut m, Command::Admit { task: id });
    finish_join(&mut m, id, Command::Succeed { task: id });
    close(&mut m, root);
    assert!(m.replay_checked().is_ok());
}

#[test]
fn i15_no_tasks_means_no_scheduler_or_grant() {
    let (mut m, _, root) = setup();
    assert_eq!(m.event_count(), 0);
    assert_eq!(close(&mut m, root).outcome, ScopeOutcome::Succeeded);
    assert_eq!(m.event_count(), 1);
    assert!(m.replay_checked().is_ok());
}

#[test]
fn i16_reference_is_not_exported_by_certified_runtime() {
    // The additive package installs a test-only #[path] import and does not
    // modify src/lib.rs, kernel.rs, NAIR, scheduler.rs or Cargo.toml.
    let baseline_source = include_str!("../src/resource_task_r02.rs");
    assert!(baseline_source.contains("intentionally NOT exported from lib.rs"));
    assert!(baseline_source.contains("No host effects"));
}

#[test]
fn revoked_grant_denies_use_but_cannot_prevent_release() {
    let (mut m, host, scope) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Write,
        },
    );
    let grant = m.host_grant(&host, scope, ResourceEffect::Write).unwrap();
    let handle = resource(&mut m, scope, grant, ResourceEffect::Write);
    unit(&mut m, Command::RevokeGrant { grant });
    assert_eq!(
        m.apply(Command::Use {
            scope,
            resource: handle,
            grant,
            effect: ResourceEffect::Write
        }),
        Err(ModelError::RevokedGrant)
    );
    unit(
        &mut m,
        Command::Release {
            scope,
            resource: handle,
        },
    );
    close(&mut m, scope);
}

#[test]
fn scope_does_not_close_with_live_resource() {
    let (mut m, host, scope) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope,
            effect: ResourceEffect::Read,
        },
    );
    let grant = m.host_grant(&host, scope, ResourceEffect::Read).unwrap();
    let handle = resource(&mut m, scope, grant, ResourceEffect::Read);
    assert_eq!(
        m.apply(Command::Close { scope }),
        Err(ModelError::OutstandingResources)
    );
    unit(
        &mut m,
        Command::Release {
            scope,
            resource: handle,
        },
    );
    close(&mut m, scope);
}

#[test]
fn nested_failure_is_visible_to_parent() {
    let (mut m, _, root) = setup();
    let nested = child(&mut m, root);
    let id = task(&mut m, nested);
    unit(&mut m, Command::Admit { task: id });
    finish_join(&mut m, id, Command::Fail { task: id, code: 7 });
    assert_eq!(close(&mut m, nested).outcome, ScopeOutcome::Failed);
    let report = close(&mut m, root);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert_eq!(report.children, vec![(nested, ScopeOutcome::Failed)]);
}

#[test]
fn foreign_domain_handles_are_rejected() {
    let (mut a, host_a, root_a) = setup();
    let (mut b, host_b) = Model::bootstrap(54321, budget(), 100);
    let root_b = b.root();
    unit(
        &mut a,
        Command::DeclareEffect {
            scope: root_a,
            effect: ResourceEffect::Read,
        },
    );
    unit(
        &mut b,
        Command::DeclareEffect {
            scope: root_b,
            effect: ResourceEffect::Read,
        },
    );
    let grant_a = a.host_grant(&host_a, root_a, ResourceEffect::Read).unwrap();
    let grant_b = b.host_grant(&host_b, root_b, ResourceEffect::Read).unwrap();
    let foreign = resource(&mut a, root_a, grant_a, ResourceEffect::Read);
    assert_eq!(
        b.apply(Command::Use {
            scope: root_b,
            resource: foreign,
            grant: grant_b,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::UnknownResource)
    );
}

#[test]
fn event_budget_rejects_before_mutation() {
    let (mut m, _, root) = {
        let (m, host) = Model::bootstrap(800, budget(), 1);
        let root = m.root();
        (m, host, root)
    };
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let before = m.clone();
    assert_eq!(
        m.apply(Command::DeclareTask { scope: root }),
        Err(ModelError::Exhausted)
    );
    assert_eq!(m, before);
}

#[test]
fn invalid_task_transitions_are_atomic() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    let before = m.clone();
    assert_eq!(
        m.apply(Command::Start { task: id }),
        Err(ModelError::InvalidTransition)
    );
    assert_eq!(m, before);
    assert_eq!(
        m.apply(Command::AcknowledgeCancel { task: id }),
        Err(ModelError::InvalidTransition)
    );
    unit(&mut m, Command::Admit { task: id });
    finish_join(&mut m, id, Command::Succeed { task: id });
    assert_eq!(
        m.apply(Command::Join { task: id }),
        Err(ModelError::InvalidTransition)
    );
}

#[test]
fn cancellation_request_does_not_instantly_preempt_task() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    unit(&mut m, Command::Admit { task: id });
    unit(&mut m, Command::RequestCancel { task: id });
    assert_eq!(m.phase(id), Some(TaskPhase::Admitted));
    assert_eq!(m.outcome(id), None);
    finish_join(&mut m, id, Command::AcknowledgeCancel { task: id });
    assert_eq!(m.outcome(id), Some(TaskOutcome::Cancelled));
}

#[test]
fn cancellation_does_not_mask_explicit_failure() {
    let (mut m, _, root) = setup();
    let id = task(&mut m, root);
    unit(&mut m, Command::Admit { task: id });
    unit(&mut m, Command::RequestCancel { task: id });
    finish_join(&mut m, id, Command::Fail { task: id, code: 92 });
    assert_eq!(close(&mut m, root).outcome, ScopeOutcome::Failed);
}

#[test]
fn scope_resource_and_child_budgets_enforced() {
    let (mut m, host, root) = setup();
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = m.host_grant(&host, root, ResourceEffect::Read).unwrap();
    let mut handles = Vec::new();
    for _ in 0..budget().resources {
        handles.push(resource(&mut m, root, grant, ResourceEffect::Read));
    }
    assert_eq!(
        m.apply(Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::Exhausted)
    );
    for handle in handles {
        unit(
            &mut m,
            Command::Release {
                scope: root,
                resource: handle,
            },
        );
    }
    assert_eq!(
        m.apply(Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::Exhausted)
    );
    let c1 = child(&mut m, root);
    let c2 = child(&mut m, root);
    assert_eq!(
        m.apply(Command::OpenChild {
            parent: root,
            budget: budget()
        }),
        Err(ModelError::Exhausted)
    );
    close(&mut m, c1);
    close(&mut m, c2);
    close(&mut m, root);
}

#[test]
fn scope_ids_and_task_ids_are_not_capability_grants() {
    let (mut m, host, root) = setup();
    let id = task(&mut m, root);
    unit(
        &mut m,
        Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        },
    );
    let grant = m.host_grant(&host, root, ResourceEffect::Read).unwrap();
    let handle = resource(&mut m, root, grant, ResourceEffect::Read);
    unit(&mut m, Command::Admit { task: id });
    unit(&mut m, Command::Succeed { task: id });
    unit(&mut m, Command::Join { task: id });
    assert_eq!(root.parts().0, 20261009);
    assert_eq!(id.parts().0, root.parts().0);
    assert_eq!(grant.parts().0, root.parts().0);
    assert_eq!(handle.parts().0, root.parts().0);
    unit(
        &mut m,
        Command::Release {
            scope: root,
            resource: handle,
        },
    );
    close(&mut m, root);
}
