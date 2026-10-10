//! R0.4 deterministic generative/adversarial verification of frozen R0.2 model.
//! TEST-ONLY. This is not a scheduler, production authority, formal proof, or fuzz corpus.

#[allow(dead_code)] // Frozen independent R0.2 APIs are not all exercised here.
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{
    Command, GrantId, HostPermit, Model, ModelError, Receipt, ResourceEffect, ResourceId,
    ScopeBudget, ScopeId, ScopeOutcome, TaskId, TaskOutcome, TaskPhase,
};

const DOMAIN: u64 = 0x5230_3454;
const MAX_GENERATED_STEPS: usize = 96;
const MAX_ACCEPTED_EVENTS: usize = 48;
const SEEDS: [u64; 16] = [
    0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987,
];

fn budget() -> ScopeBudget {
    ScopeBudget {
        tasks: 3,
        resources: 3,
        children: 2,
    }
}

// Explicitly chosen, versioned pseudo-random algorithm: deterministic inputs,
// no dependency on host entropy or external RNG packages.
struct Sequence(u64);

impl Sequence {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.0;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }
}

fn choose<T: Copy>(items: &[T], seq: &mut Sequence) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items[(seq.next() as usize) % items.len()])
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Model(Command),
    Grant {
        scope: ScopeId,
        effect: ResourceEffect,
    },
}

struct Harness {
    model: Model,
    permit: HostPermit,
    foreign_scope: ScopeId,
    scopes: Vec<ScopeId>,
    tasks: Vec<(TaskId, ScopeId)>,
    resources: Vec<(ResourceId, ScopeId, ResourceEffect, bool)>,
    grants: Vec<(GrantId, ScopeId, ResourceEffect)>,
}

impl Harness {
    fn new() -> Self {
        let (model, permit) = Model::bootstrap(DOMAIN, budget(), MAX_ACCEPTED_EVENTS);
        let (foreign, _) = Model::bootstrap(DOMAIN + 1, budget(), MAX_ACCEPTED_EVENTS);
        let root = model.root();
        Self {
            model,
            permit,
            foreign_scope: foreign.root(),
            scopes: vec![root],
            tasks: Vec::new(),
            resources: Vec::new(),
            grants: Vec::new(),
        }
    }

    fn generate(&self, seq: &mut Sequence) -> Operation {
        let root = self.model.root();
        let scope = choose(&self.scopes, seq).unwrap_or(root);
        let task = choose(&self.tasks, seq).map(|(id, _)| id);
        let resource = choose(&self.resources, seq);
        let grant = choose(&self.grants, seq);
        match seq.next() % 24 {
            0 => Operation::Model(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::Read,
            }),
            1 => Operation::Model(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::Write,
            }),
            2 => Operation::Grant {
                scope,
                effect: ResourceEffect::Read,
            },
            3 => Operation::Grant {
                scope,
                effect: ResourceEffect::Write,
            },
            4 => Operation::Model(Command::OpenChild {
                parent: root,
                budget: ScopeBudget {
                    tasks: 2,
                    resources: 2,
                    children: 0,
                },
            }),
            5 => Operation::Model(Command::DeclareTask { scope }),
            6 => {
                Operation::Model(
                    task.map_or(Command::DeclareTask { scope }, |id| Command::Admit {
                        task: id,
                    }),
                )
            }
            7 => {
                Operation::Model(
                    task.map_or(Command::DeclareTask { scope }, |id| Command::Start {
                        task: id,
                    }),
                )
            }
            8 => Operation::Model(task.map_or(Command::DeclareTask { scope }, |id| {
                Command::RequestCancel { task: id }
            })),
            9 => Operation::Model(task.map_or(Command::DeclareTask { scope }, |id| {
                Command::Succeed { task: id }
            })),
            10 => {
                Operation::Model(
                    task.map_or(Command::DeclareTask { scope }, |id| Command::Fail {
                        task: id,
                        code: 17,
                    }),
                )
            }
            11 => Operation::Model(task.map_or(Command::DeclareTask { scope }, |id| {
                Command::AcknowledgeCancel { task: id }
            })),
            12 => {
                Operation::Model(
                    task.map_or(Command::DeclareTask { scope }, |id| Command::Join {
                        task: id,
                    }),
                )
            }
            13 => match grant {
                Some((id, owner, effect)) => Operation::Model(Command::Acquire {
                    scope: owner,
                    grant: id,
                    effect,
                }),
                None => Operation::Grant {
                    scope,
                    effect: ResourceEffect::Read,
                },
            },
            14 => match (resource, grant) {
                (Some((id, owner, effect, _)), Some((cap, _, _))) => {
                    Operation::Model(Command::Use {
                        scope: owner,
                        resource: id,
                        grant: cap,
                        effect,
                    })
                }
                _ => Operation::Model(Command::DeclareEffect {
                    scope,
                    effect: ResourceEffect::Read,
                }),
            },
            15 => resource.map_or(
                Operation::Model(Command::DeclareTask { scope }),
                |(id, owner, _, _)| {
                    Operation::Model(Command::Release {
                        scope: owner,
                        resource: id,
                    })
                },
            ),
            16 => grant.map_or(
                Operation::Grant {
                    scope,
                    effect: ResourceEffect::Read,
                },
                |(id, _, _)| Operation::Model(Command::RevokeGrant { grant: id }),
            ),
            17 => Operation::Model(Command::Close { scope }),
            18 => Operation::Model(Command::Close {
                scope: self.foreign_scope,
            }),
            19 => Operation::Model(Command::DeclareEffect {
                scope,
                effect: ResourceEffect::ExternalIo,
            }),
            20 => Operation::Grant {
                scope,
                effect: ResourceEffect::ExternalIo,
            },
            21 => resource.map_or(
                Operation::Model(Command::Close { scope }),
                |(id, _, _, _)| {
                    Operation::Model(Command::Release {
                        scope: self.foreign_scope,
                        resource: id,
                    })
                },
            ),
            22 => match grant {
                Some((id, _, effect)) => Operation::Model(Command::Acquire {
                    scope: self.foreign_scope,
                    grant: id,
                    effect,
                }),
                None => Operation::Model(Command::DeclareTask {
                    scope: self.foreign_scope,
                }),
            },
            _ => Operation::Model(Command::Close { scope: root }),
        }
    }

    fn verify_closure(&self, scope: ScopeId) {
        let report = self
            .model
            .report(scope)
            .expect("accepted close must have report");
        assert_eq!(report.scope, scope);
        assert!(report.tasks.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(report.children.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for &(task, owner) in &self.tasks {
            if owner == scope {
                assert_eq!(self.model.phase(task), Some(TaskPhase::Joined));
                assert!(report
                    .tasks
                    .contains(&(task, self.model.outcome(task).expect("joined outcome"))));
            }
        }
        for &(_, owner, _, live) in &self.resources {
            if owner == scope {
                assert!(!live, "closed with live resource");
            }
        }
        for &child in &self.scopes {
            if self.model.parent(child) == Some(scope) {
                let outcome = self
                    .model
                    .report(child)
                    .expect("closed child must have report")
                    .outcome;
                assert!(report.children.contains(&(child, outcome)));
            }
        }
        let failed = report
            .tasks
            .iter()
            .any(|(_, result)| matches!(result, TaskOutcome::Failed(_)))
            || report
                .children
                .iter()
                .any(|(_, result)| *result == ScopeOutcome::Failed);
        let cancelled = report
            .tasks
            .iter()
            .any(|(_, result)| *result == TaskOutcome::Cancelled)
            || report
                .children
                .iter()
                .any(|(_, result)| *result == ScopeOutcome::Cancelled);
        let expected = if failed {
            ScopeOutcome::Failed
        } else if cancelled {
            ScopeOutcome::Cancelled
        } else {
            ScopeOutcome::Succeeded
        };
        assert_eq!(report.outcome, expected);
    }

    fn apply_checked(&mut self, operation: Operation, trace: &mut Vec<String>) -> bool {
        let before = self.model.clone();
        let events = before.event_count();
        let result = match operation {
            Operation::Model(command) => self.model.apply(command).map(Outcome::Command),
            Operation::Grant { scope, effect } => self
                .model
                .host_grant(&self.permit, scope, effect)
                .map(Outcome::Grant),
        };
        trace.push(format!("{operation:?} -> {result:?}"));
        match result {
            Err(_) => {
                assert_eq!(self.model, before, "rejection was not atomic: {trace:?}");
                assert_eq!(self.model.event_count(), events);
                false
            }
            Ok(receipt) => {
                assert_eq!(
                    self.model.event_count(),
                    events + 1,
                    "accepted event count: {trace:?}"
                );
                match (operation, receipt) {
                    (Operation::Grant { scope, effect }, Outcome::Grant(id)) => {
                        self.grants.push((id, scope, effect))
                    }
                    (
                        Operation::Model(Command::OpenChild { .. }),
                        Outcome::Command(Receipt::Scope(id)),
                    ) => self.scopes.push(id),
                    (
                        Operation::Model(Command::DeclareTask { scope }),
                        Outcome::Command(Receipt::Task(id)),
                    ) => self.tasks.push((id, scope)),
                    (
                        Operation::Model(Command::Acquire { scope, effect, .. }),
                        Outcome::Command(Receipt::Resource(id)),
                    ) => {
                        self.resources.push((id, scope, effect, true));
                    }
                    (
                        Operation::Model(Command::Release { resource, .. }),
                        Outcome::Command(Receipt::Unit),
                    ) => {
                        let entry = self
                            .resources
                            .iter_mut()
                            .find(|(id, _, _, _)| *id == resource)
                            .expect("released resource must be tracked");
                        assert!(entry.3, "duplicate successful release: {trace:?}");
                        entry.3 = false;
                    }
                    (
                        Operation::Model(Command::Close { scope }),
                        Outcome::Command(Receipt::Closed(_)),
                    ) => self.verify_closure(scope),
                    (_, Outcome::Command(Receipt::Unit)) => {}
                    (action, other) => panic!("receipt mismatch: {action:?} {other:?}; {trace:?}"),
                }
                assert_eq!(
                    self.model.replay_checked(),
                    Ok(()),
                    "accepted event failed replay: {trace:?}"
                );
                true
            }
        }
    }
}

#[derive(Debug)]
enum Outcome {
    Command(Receipt),
    Grant(GrantId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Run {
    accepted: usize,
    rejected: usize,
    closed: usize,
    trace: Vec<String>,
    snapshots: Vec<String>,
}

fn run(seed: u64, count: usize) -> Run {
    assert!(count <= MAX_GENERATED_STEPS);
    let mut seq = Sequence::new(seed);
    let mut harness = Harness::new();
    let mut trace = Vec::new();
    let mut snapshots = Vec::new();
    let mut accepted = 0;
    let mut rejected = 0;
    let mut closed = 0;
    for _ in 0..count {
        let operation = harness.generate(&mut seq);
        if harness.apply_checked(operation, &mut trace) {
            accepted += 1;
            if matches!(operation, Operation::Model(Command::Close { .. })) {
                closed += 1;
            }
        } else {
            rejected += 1;
        }
        // Debug representation is only a test-process fingerprint, NOT a
        // durable or cross-compiler canonical encoding.
        snapshots.push(format!("{:?}", harness.model));
    }
    assert_eq!(trace.len(), count);
    assert_eq!(accepted + rejected, count);
    Run {
        accepted,
        rejected,
        closed,
        trace,
        snapshots,
    }
}

fn unit(model: &mut Model, command: Command) {
    assert_eq!(model.apply(command), Ok(Receipt::Unit));
}

fn task(model: &mut Model, scope: ScopeId) -> TaskId {
    match model.apply(Command::DeclareTask { scope }) {
        Ok(Receipt::Task(id)) => id,
        other => panic!("expected task: {other:?}"),
    }
}

fn child(model: &mut Model, root: ScopeId) -> ScopeId {
    match model.apply(Command::OpenChild {
        parent: root,
        budget: ScopeBudget {
            tasks: 1,
            resources: 0,
            children: 0,
        },
    }) {
        Ok(Receipt::Scope(id)) => id,
        other => panic!("expected child: {other:?}"),
    }
}

fn close(model: &mut Model, scope: ScopeId) -> reference::ScopeReport {
    match model.apply(Command::Close { scope }) {
        Ok(Receipt::Closed(report)) => report,
        other => panic!("expected close report: {other:?}"),
    }
}

// Pure ddmin-style deletion reducer. It minimizes a reproducible synthetic
// predicate, not a real R0.2 defect or formal minimality certificate.
fn reduce<T: Clone>(initial: &[T], still_fails: impl Fn(&[T]) -> bool) -> Vec<T> {
    assert!(still_fails(initial));
    let mut current = initial.to_vec();
    let mut partitions = 2;
    while current.len() >= 2 {
        let chunk = current.len().div_ceil(partitions);
        let mut reduced = false;
        for offset in (0..current.len()).step_by(chunk) {
            let end = (offset + chunk).min(current.len());
            let mut candidate = current[..offset].to_vec();
            candidate.extend_from_slice(&current[end..]);
            if still_fails(&candidate) {
                current = candidate;
                partitions = (partitions - 1).max(2);
                reduced = true;
                break;
            }
        }
        if !reduced {
            if partitions >= current.len() {
                break;
            }
            partitions = (partitions * 2).min(current.len());
        }
    }
    current
}

fn synthetic_fault(bytes: &[u8]) -> bool {
    bytes.windows(3).any(|part| part == [4, 7, 9])
}

#[test]
fn fixed_seed_corpus_replays_and_rejects_atomically() {
    let mut accepted = 0;
    let mut rejected = 0;
    for seed in SEEDS {
        let report = run(seed, MAX_GENERATED_STEPS);
        accepted += report.accepted;
        rejected += report.rejected;
    }
    assert!(
        accepted > 20,
        "seed corpus did not reach enough valid transitions"
    );
    assert!(
        rejected > 100,
        "seed corpus did not reach enough invalid transitions"
    );
}

#[test]
fn identical_seed_reproduces_full_trace_and_snapshots() {
    for seed in [0, 42, 987] {
        assert_eq!(
            run(seed, MAX_GENERATED_STEPS),
            run(seed, MAX_GENERATED_STEPS)
        );
    }
}

#[test]
fn distinct_seeds_change_generated_trace() {
    assert_ne!(run(0, 64).trace, run(1, 64).trace);
}

#[test]
fn deterministic_reducer_minimizes_synthetic_counterexample() {
    let original = [1_u8, 6, 4, 7, 9, 2, 3, 8];
    let result = reduce(&original, synthetic_fault);
    assert_eq!(result, vec![4, 7, 9]);
}

#[test]
fn reducer_is_idempotent_and_preserves_failure() {
    let original = [5_u8, 4, 7, 9, 8, 3, 2, 1];
    let once = reduce(&original, synthetic_fault);
    let twice = reduce(&once, synthetic_fault);
    assert!(synthetic_fault(&once));
    assert_eq!(once, twice);
    for position in 0..once.len() {
        let mut smaller = once.clone();
        smaller.remove(position);
        assert!(!synthetic_fault(&smaller));
    }
}

#[test]
fn revocation_cannot_block_release_or_closure() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(), 24);
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
    let resource = match model.apply(Command::Acquire {
        scope: root,
        grant,
        effect: ResourceEffect::Read,
    }) {
        Ok(Receipt::Resource(id)) => id,
        other => panic!("expected acquired resource: {other:?}"),
    };
    unit(&mut model, Command::RevokeGrant { grant });
    let before = model.clone();
    assert_eq!(
        model.apply(Command::Use {
            scope: root,
            resource,
            grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::RevokedGrant)
    );
    assert_eq!(model, before);
    unit(
        &mut model,
        Command::Release {
            scope: root,
            resource,
        },
    );
    assert_eq!(close(&mut model, root).outcome, ScopeOutcome::Succeeded);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn cancellation_failure_precedence_is_explicit() {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(), 24);
    let root = model.root();
    let id = task(&mut model, root);
    unit(&mut model, Command::Admit { task: id });
    unit(&mut model, Command::RequestCancel { task: id });
    let before = model.clone();
    assert_eq!(
        model.apply(Command::Succeed { task: id }),
        Err(ModelError::InvalidTransition)
    );
    assert_eq!(model, before);
    unit(&mut model, Command::Fail { task: id, code: 17 });
    unit(&mut model, Command::Join { task: id });
    let report = close(&mut model, root);
    assert_eq!(report.outcome, ScopeOutcome::Failed);
    assert_eq!(report.tasks, vec![(id, TaskOutcome::Failed(17))]);
    assert_eq!(model.replay_checked(), Ok(()));
}

#[test]
fn child_close_permutation_preserves_canonical_report() {
    let mut expected = None;
    for order in [[0_usize, 1], [1, 0]] {
        let (mut model, _) = Model::bootstrap(
            DOMAIN,
            ScopeBudget {
                tasks: 0,
                resources: 0,
                children: 2,
            },
            48,
        );
        let root = model.root();
        let children = [child(&mut model, root), child(&mut model, root)];
        let ids = [task(&mut model, children[0]), task(&mut model, children[1])];
        for id in ids {
            unit(&mut model, Command::Admit { task: id });
        }
        unit(
            &mut model,
            Command::Fail {
                task: ids[0],
                code: 3,
            },
        );
        unit(&mut model, Command::RequestCancel { task: ids[1] });
        unit(&mut model, Command::AcknowledgeCancel { task: ids[1] });
        for index in order {
            unit(&mut model, Command::Join { task: ids[index] });
            close(&mut model, children[index]);
        }
        let report = close(&mut model, root);
        assert_eq!(report.outcome, ScopeOutcome::Failed);
        assert_eq!(
            report.children,
            vec![
                (children[0], ScopeOutcome::Failed),
                (children[1], ScopeOutcome::Cancelled),
            ]
        );
        assert_eq!(model.replay_checked(), Ok(()));
        if let Some(previous) = &expected {
            assert_eq!(&report, previous);
        } else {
            expected = Some(report);
        }
    }
}

#[test]
fn foreign_domain_identity_is_rejected_atomically() {
    let (mut model, _) = Model::bootstrap(DOMAIN, budget(), 16);
    let (foreign, _) = Model::bootstrap(DOMAIN + 1, budget(), 16);
    let before = model.clone();
    assert_eq!(
        model.apply(Command::DeclareTask {
            scope: foreign.root()
        }),
        Err(ModelError::UnknownScope)
    );
    assert_eq!(model, before);
}

#[test]
fn host_permit_is_not_an_ambient_guest_grant() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(), 16);
    let (_, foreign_permit) = Model::bootstrap(DOMAIN + 1, budget(), 16);
    let root = model.root();
    let before = model.clone();
    assert_eq!(
        model.host_grant(&foreign_permit, root, ResourceEffect::Read),
        Err(ModelError::WrongHostPermit)
    );
    assert_eq!(model, before);
    let grant = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("trusted host grant");
    let before = model.clone();
    assert_eq!(
        model.apply(Command::Acquire {
            scope: root,
            grant,
            effect: ResourceEffect::Read
        }),
        Err(ModelError::MissingDeclaration)
    );
    assert_eq!(model, before);
}

#[test]
fn event_budget_zero_denies_without_state_mutation() {
    let (mut model, permit) = Model::bootstrap(DOMAIN, budget(), 0);
    let root = model.root();
    let before = model.clone();
    assert_eq!(
        model.apply(Command::DeclareTask { scope: root }),
        Err(ModelError::Exhausted)
    );
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::Read),
        Err(ModelError::Exhausted)
    );
    assert_eq!(model, before);
    assert_eq!(model.event_count(), 0);
}

#[test]
fn r04_is_test_only_and_requires_no_external_runtime() {
    let lib = include_str!("../src/lib.rs");
    let cargo = include_str!("../Cargo.toml");
    assert!(!lib.contains("resource_task_r04"));
    assert!(!lib.contains("resource_task_r02"));
    assert!(!cargo.contains("resource_task_r04"));
    let (model, _) = Model::bootstrap(DOMAIN, budget(), MAX_ACCEPTED_EVENTS);
    assert_eq!(model.event_count(), 0);
    assert_eq!(model.replay_checked(), Ok(()));
}
