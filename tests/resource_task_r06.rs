//! R0.6 — independently specified, finite single-scope conformance oracle.
//! TEST-ONLY. Not a production scheduler, runtime authority or formal proof.
//! The oracle below MUST NOT inspect private model data or use model results to
//! choose its next expected transition. The R0.2 implementation is the SUT.

#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{
    Command, GrantId, HostPermit, Model, ModelError, Receipt, ResourceEffect, ResourceId,
    ScopeBudget, ScopeId, ScopeOutcome, TaskId, TaskOutcome,
};

const DOMAIN: u64 = 0x5230_3601;
const FOREIGN_DOMAIN: u64 = 0x5230_3602;
const MAX_EVENTS: usize = 20;
const SEED_COUNT: usize = 32;
const STEPS_PER_SEED: usize = 96;
const FINITE_ALPHABET: usize = 5;
const FINITE_DEPTH: usize = 5;

fn budget(tasks: usize, resources: usize, children: usize) -> ScopeBudget {
    ScopeBudget {
        tasks,
        resources,
        children,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    DeclareRead,
    HostReadGrant,
    AcquireRead,
    UseRead,
    RevokeGrant,
    ReleaseRead,
    DeclareTask,
    Admit,
    Start,
    RequestCancel,
    Succeed,
    Fail,
    AcknowledgeCancel,
    Join,
    Close,
    ExternalIoDeclaration,
    ForeignScopeTask,
    ForeignTaskStart,
    ForeignGrantAcquire,
    ForeignResourceUse,
    WrongHostGrant,
}

const ACTIONS: [Action; 21] = [
    Action::DeclareRead,
    Action::HostReadGrant,
    Action::AcquireRead,
    Action::UseRead,
    Action::RevokeGrant,
    Action::ReleaseRead,
    Action::DeclareTask,
    Action::Admit,
    Action::Start,
    Action::RequestCancel,
    Action::Succeed,
    Action::Fail,
    Action::AcknowledgeCancel,
    Action::Join,
    Action::Close,
    Action::ExternalIoDeclaration,
    Action::ForeignScopeTask,
    Action::ForeignTaskStart,
    Action::ForeignGrantAcquire,
    Action::ForeignResourceUse,
    Action::WrongHostGrant,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Declared,
    Admitted,
    Running,
    Terminal,
    Joined,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Signature {
    Unit,
    Grant(u64),
    Resource(u64),
    Task(u64),
    Closed(ScopeOutcome, Vec<TaskOutcome>),
}

// Independent specification. This intentionally uses neither Model::phase,
// Model::outcome, Model::report, nor Model::replay_checked for predictions.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Oracle {
    events: usize,
    max_events: usize,
    task_budget: usize,
    resource_budget: usize,
    declared_read: bool,
    grants_issued: u64,
    current_grant_revoked: bool,
    has_resource: bool,
    resource_live: bool,
    task: Option<Phase>,
    cancel_requested: bool,
    task_outcome: Option<TaskOutcome>,
    closed: bool,
}

impl Oracle {
    fn new(tasks: usize, resources: usize, max_events: usize) -> Self {
        Self {
            events: 0,
            max_events,
            task_budget: tasks,
            resource_budget: resources,
            declared_read: false,
            grants_issued: 0,
            current_grant_revoked: false,
            has_resource: false,
            resource_live: false,
            task: None,
            cancel_requested: false,
            task_outcome: None,
            closed: false,
        }
    }

    fn open(&self) -> Result<(), ModelError> {
        if self.closed {
            Err(ModelError::ClosedScope)
        } else {
            Ok(())
        }
    }

    fn read_declared(&self) -> Result<(), ModelError> {
        self.open()?;
        if !self.declared_read {
            return Err(ModelError::MissingDeclaration);
        }
        Ok(())
    }

    fn current_grant(&self) -> Result<(), ModelError> {
        if self.grants_issued == 0 {
            return Err(ModelError::UnknownGrant);
        }
        if self.current_grant_revoked {
            return Err(ModelError::RevokedGrant);
        }
        Ok(())
    }

    fn active_task(&self) -> Result<Phase, ModelError> {
        let phase = self.task.ok_or(ModelError::UnknownTask)?;
        self.open()?;
        Ok(phase)
    }

    fn step(&mut self, action: Action) -> Result<Signature, ModelError> {
        // The host witness is checked before the global event budget in R0.2.
        if action == Action::WrongHostGrant {
            return Err(ModelError::WrongHostPermit);
        }
        if self.events >= self.max_events {
            return Err(ModelError::Exhausted);
        }
        let mut next = self.clone();
        let receipt = next.decide(action)?;
        next.events += 1;
        *self = next;
        Ok(receipt)
    }

    fn decide(&mut self, action: Action) -> Result<Signature, ModelError> {
        match action {
            Action::DeclareRead => {
                self.open()?;
                if self.declared_read {
                    return Err(ModelError::InvalidTransition);
                }
                self.declared_read = true;
                Ok(Signature::Unit)
            }
            Action::HostReadGrant => {
                self.open()?;
                self.grants_issued += 1;
                self.current_grant_revoked = false;
                Ok(Signature::Grant(self.grants_issued))
            }
            Action::AcquireRead => {
                self.read_declared()?;
                self.current_grant()?;
                if self.resource_budget == 0 || self.has_resource {
                    return Err(ModelError::Exhausted);
                }
                self.has_resource = true;
                self.resource_live = true;
                Ok(Signature::Resource(1))
            }
            Action::UseRead => {
                self.read_declared()?;
                if !self.has_resource {
                    return Err(ModelError::UnknownResource);
                }
                if !self.resource_live {
                    return Err(ModelError::ReleasedResource);
                }
                self.current_grant()?;
                Ok(Signature::Unit)
            }
            Action::RevokeGrant => {
                if self.grants_issued == 0 {
                    return Err(ModelError::UnknownGrant);
                }
                if self.current_grant_revoked {
                    return Err(ModelError::InvalidTransition);
                }
                self.current_grant_revoked = true;
                Ok(Signature::Unit)
            }
            Action::ReleaseRead => {
                self.open()?;
                if !self.has_resource {
                    return Err(ModelError::UnknownResource);
                }
                if !self.resource_live {
                    return Err(ModelError::ReleasedResource);
                }
                self.resource_live = false;
                Ok(Signature::Unit)
            }
            Action::DeclareTask => {
                self.open()?;
                if self.task_budget == 0 || self.task.is_some() {
                    return Err(ModelError::Exhausted);
                }
                self.task = Some(Phase::Declared);
                Ok(Signature::Task(1))
            }
            Action::Admit => {
                if self.active_task()? != Phase::Declared {
                    return Err(ModelError::InvalidTransition);
                }
                self.task = Some(Phase::Admitted);
                Ok(Signature::Unit)
            }
            Action::Start => {
                let phase = self.active_task()?;
                if self.cancel_requested || phase != Phase::Admitted {
                    return Err(ModelError::InvalidTransition);
                }
                self.task = Some(Phase::Running);
                Ok(Signature::Unit)
            }
            Action::RequestCancel => {
                let phase = self.active_task()?;
                if !matches!(phase, Phase::Admitted | Phase::Running) || self.cancel_requested {
                    return Err(ModelError::InvalidTransition);
                }
                self.cancel_requested = true;
                Ok(Signature::Unit)
            }
            Action::Succeed | Action::Fail | Action::AcknowledgeCancel => {
                if action == Action::AcknowledgeCancel && self.task.is_none() {
                    return Err(ModelError::UnknownTask);
                }
                if action == Action::AcknowledgeCancel && !self.cancel_requested {
                    return Err(ModelError::InvalidTransition);
                }
                let phase = self.active_task()?;
                if !matches!(phase, Phase::Admitted | Phase::Running) {
                    return Err(ModelError::InvalidTransition);
                }
                if action == Action::Succeed && self.cancel_requested {
                    return Err(ModelError::InvalidTransition);
                }
                self.task_outcome = Some(match action {
                    Action::Succeed => TaskOutcome::Succeeded,
                    Action::Fail => TaskOutcome::Failed(17),
                    Action::AcknowledgeCancel => TaskOutcome::Cancelled,
                    _ => unreachable!(),
                });
                self.task = Some(Phase::Terminal);
                Ok(Signature::Unit)
            }
            Action::Join => {
                if self.active_task()? != Phase::Terminal {
                    return Err(ModelError::InvalidTransition);
                }
                self.task = Some(Phase::Joined);
                Ok(Signature::Unit)
            }
            Action::Close => {
                self.open()?;
                if self.resource_live {
                    return Err(ModelError::OutstandingResources);
                }
                if self.task.is_some() && self.task != Some(Phase::Joined) {
                    return Err(ModelError::OutstandingTasks);
                }
                let outcomes: Vec<_> = self.task_outcome.into_iter().collect();
                let outcome = match self.task_outcome {
                    Some(TaskOutcome::Failed(_)) => ScopeOutcome::Failed,
                    Some(TaskOutcome::Cancelled) => ScopeOutcome::Cancelled,
                    _ => ScopeOutcome::Succeeded,
                };
                self.closed = true;
                Ok(Signature::Closed(outcome, outcomes))
            }
            Action::ExternalIoDeclaration => {
                self.open()?;
                Err(ModelError::UnsupportedEffect)
            }
            Action::ForeignScopeTask => Err(ModelError::UnknownScope),
            Action::ForeignTaskStart => Err(ModelError::UnknownTask),
            Action::ForeignGrantAcquire => {
                self.read_declared()?;
                Err(ModelError::UnknownGrant)
            }
            Action::ForeignResourceUse => {
                self.read_declared()?;
                Err(ModelError::UnknownResource)
            }
            Action::WrongHostGrant => Err(ModelError::WrongHostPermit),
        }
    }
}

// The fixture constructs real typed foreign-domain identities; no forging of
// private ID fields and no assumption that bare integer IDs grant authority.
struct Foreign {
    root: ScopeId,
    task: TaskId,
    resource: ResourceId,
    grant: GrantId,
    permit: HostPermit,
}

fn foreign_fixture() -> Foreign {
    let (mut model, permit) = Model::bootstrap(FOREIGN_DOMAIN, budget(1, 1, 0), 16);
    let root = model.root();
    let grant = model
        .host_grant(&permit, root, ResourceEffect::Read)
        .expect("foreign grant");
    model
        .apply(Command::DeclareEffect {
            scope: root,
            effect: ResourceEffect::Read,
        })
        .expect("foreign declaration");
    let resource = match model.apply(Command::Acquire {
        scope: root,
        grant,
        effect: ResourceEffect::Read,
    }) {
        Ok(Receipt::Resource(id)) => id,
        other => panic!("foreign resource setup: {other:?}"),
    };
    let task = match model.apply(Command::DeclareTask { scope: root }) {
        Ok(Receipt::Task(id)) => id,
        other => panic!("foreign task setup: {other:?}"),
    };
    Foreign {
        root,
        task,
        resource,
        grant,
        permit,
    }
}

struct Trial {
    model: Model,
    host: HostPermit,
    root: ScopeId,
    grant: Option<GrantId>,
    resource: Option<ResourceId>,
    task: Option<TaskId>,
    foreign: Foreign,
    oracle: Oracle,
}

impl Trial {
    fn new(tasks: usize, resources: usize, max_events: usize) -> Self {
        let (model, host) = Model::bootstrap(DOMAIN, budget(tasks, resources, 0), max_events);
        let root = model.root();
        Self {
            model,
            host,
            root,
            grant: None,
            resource: None,
            task: None,
            foreign: foreign_fixture(),
            oracle: Oracle::new(tasks, resources, max_events),
        }
    }

    fn step(&mut self, action: Action) -> Result<Signature, ModelError> {
        let before = self.model.clone();
        let count_before = self.model.event_count();
        let oracle_before = self.oracle.clone();
        let predicted = self.oracle.step(action);
        let actual = match action {
            Action::HostReadGrant => self
                .model
                .host_grant(&self.host, self.root, ResourceEffect::Read)
                .map(|id| {
                    self.grant = Some(id);
                    Signature::Grant(id.parts().1)
                }),
            Action::WrongHostGrant => self
                .model
                .host_grant(&self.foreign.permit, self.root, ResourceEffect::Read)
                .map(|id| Signature::Grant(id.parts().1)),
            _ => {
                let cmd = self.command(action);
                self.model.apply(cmd).map(|receipt| match receipt {
                    Receipt::Unit => Signature::Unit,
                    Receipt::Task(id) => {
                        self.task = Some(id);
                        Signature::Task(id.parts().1)
                    }
                    Receipt::Resource(id) => {
                        self.resource = Some(id);
                        Signature::Resource(id.parts().1)
                    }
                    Receipt::Closed(report) => {
                        assert_eq!(report.scope, self.root);
                        assert!(report.children.is_empty());
                        if let Some(task) = self.task {
                            assert_eq!(
                                report.tasks.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
                                vec![task]
                            );
                        } else {
                            assert!(report.tasks.is_empty());
                        }
                        Signature::Closed(
                            report.outcome,
                            report
                                .tasks
                                .into_iter()
                                .map(|(_, outcome)| outcome)
                                .collect(),
                        )
                    }
                    Receipt::Scope(_) => panic!("no child creation in R0.6 finite oracle"),
                })
            }
        };
        assert_eq!(
            actual, predicted,
            "independent oracle mismatch on action {action:?}"
        );
        if actual.is_ok() {
            assert_eq!(self.model.event_count(), count_before + 1);
            assert_eq!(self.model.replay_checked(), Ok(()));
        } else {
            assert_eq!(
                self.model, before,
                "rejection mutated implementation state: {action:?}"
            );
            assert_eq!(
                self.oracle, oracle_before,
                "rejection mutated specification state: {action:?}"
            );
            assert_eq!(self.model.event_count(), count_before);
        }
        actual
    }

    fn command(&self, action: Action) -> Command {
        let task = self.task.unwrap_or(self.foreign.task);
        let resource = self.resource.unwrap_or(self.foreign.resource);
        let grant = self.grant.unwrap_or(self.foreign.grant);
        match action {
            Action::DeclareRead => Command::DeclareEffect {
                scope: self.root,
                effect: ResourceEffect::Read,
            },
            Action::AcquireRead => Command::Acquire {
                scope: self.root,
                grant,
                effect: ResourceEffect::Read,
            },
            Action::UseRead => Command::Use {
                scope: self.root,
                resource,
                grant,
                effect: ResourceEffect::Read,
            },
            Action::RevokeGrant => Command::RevokeGrant { grant },
            Action::ReleaseRead => Command::Release {
                scope: self.root,
                resource,
            },
            Action::DeclareTask => Command::DeclareTask { scope: self.root },
            Action::Admit => Command::Admit { task },
            Action::Start => Command::Start { task },
            Action::RequestCancel => Command::RequestCancel { task },
            Action::Succeed => Command::Succeed { task },
            Action::Fail => Command::Fail { task, code: 17 },
            Action::AcknowledgeCancel => Command::AcknowledgeCancel { task },
            Action::Join => Command::Join { task },
            Action::Close => Command::Close { scope: self.root },
            Action::ExternalIoDeclaration => Command::DeclareEffect {
                scope: self.root,
                effect: ResourceEffect::ExternalIo,
            },
            Action::ForeignScopeTask => Command::DeclareTask {
                scope: self.foreign.root,
            },
            Action::ForeignTaskStart => Command::Start {
                task: self.foreign.task,
            },
            Action::ForeignGrantAcquire => Command::Acquire {
                scope: self.root,
                grant: self.foreign.grant,
                effect: ResourceEffect::Read,
            },
            Action::ForeignResourceUse => Command::Use {
                scope: self.root,
                resource: self.foreign.resource,
                grant,
                effect: ResourceEffect::Read,
            },
            Action::HostReadGrant | Action::WrongHostGrant => {
                unreachable!("host actions dispatched separately")
            }
        }
    }
}

fn run(
    actions: &[Action],
    tasks: usize,
    resources: usize,
    max_events: usize,
) -> Vec<Result<Signature, ModelError>> {
    let mut trial = Trial::new(tasks, resources, max_events);
    actions.iter().map(|action| trial.step(*action)).collect()
}

fn generated(seed: u64, steps: usize) -> Vec<Action> {
    let mut x = seed ^ 0xA5D3_3AC5_3C6D_6781;
    (0..steps)
        .map(|_| {
            // Deliberately small, platform-independent PRNG, fixed seeds.
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            ACTIONS[(x % ACTIONS.len() as u64) as usize]
        })
        .collect()
}

#[test]
fn deterministic_generated_corpus_matches_independent_oracle() {
    let mut trials = 0usize;
    for seed in 1..=SEED_COUNT as u64 {
        let script = generated(seed, STEPS_PER_SEED);
        assert_eq!(
            run(&script, 1, 1, MAX_EVENTS).len(),
            STEPS_PER_SEED,
            "seed={seed}"
        );
        trials += script.len();
    }
    assert_eq!(trials, SEED_COUNT * STEPS_PER_SEED);
    println!("R06_WITNESS version=1 kind=independent_oracle seeds={SEED_COUNT} steps={STEPS_PER_SEED} comparisons={trials} outcome=PASS");
}

#[test]
fn finite_five_action_sequences_match_independent_oracle() {
    let alphabet = [
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::AcquireRead,
        Action::ReleaseRead,
        Action::Close,
    ];
    let mut checked = 0usize;
    for mut encoded in 0..FINITE_ALPHABET.pow(FINITE_DEPTH as u32) {
        let mut actions = Vec::new();
        for _ in 0..FINITE_DEPTH {
            actions.push(alphabet[encoded % FINITE_ALPHABET]);
            encoded /= FINITE_ALPHABET;
        }
        let result = run(&actions, 1, 1, MAX_EVENTS);
        assert_eq!(result.len(), FINITE_DEPTH);
        checked += 1;
    }
    assert_eq!(checked, 3_125);
    println!("R06_WITNESS version=1 kind=finite_alphabet sequences={checked} alphabet=5 depth=5 outcome=PASS");
}

#[test]
fn read_permission_is_not_equivalent_to_effect_declaration() {
    let actions = [
        Action::HostReadGrant,
        Action::AcquireRead,
        Action::DeclareRead,
        Action::AcquireRead,
        Action::UseRead,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[1], Err(ModelError::MissingDeclaration));
    assert_eq!(results[3], Ok(Signature::Resource(1)));
    assert_eq!(results[4], Ok(Signature::Unit));
}

#[test]
fn host_grants_can_be_revoked_without_revoking_cleanup() {
    let actions = [
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::AcquireRead,
        Action::RevokeGrant,
        Action::UseRead,
        Action::ReleaseRead,
        Action::Close,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[4], Err(ModelError::RevokedGrant));
    assert_eq!(results[5], Ok(Signature::Unit));
    assert_eq!(
        results[6],
        Ok(Signature::Closed(ScopeOutcome::Succeeded, vec![]))
    );
}

#[test]
fn reissued_host_grant_allows_use_but_not_resource_reacquisition() {
    let actions = [
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::AcquireRead,
        Action::RevokeGrant,
        Action::HostReadGrant,
        Action::UseRead,
        Action::ReleaseRead,
        Action::AcquireRead,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[4], Ok(Signature::Grant(2)));
    assert_eq!(results[5], Ok(Signature::Unit));
    assert_eq!(results[7], Err(ModelError::Exhausted));
}

#[test]
fn success_path_and_closed_scope_have_an_independent_report() {
    let actions = [
        Action::DeclareTask,
        Action::Admit,
        Action::Start,
        Action::Succeed,
        Action::Join,
        Action::Close,
        Action::DeclareTask,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(
        results[5],
        Ok(Signature::Closed(
            ScopeOutcome::Succeeded,
            vec![TaskOutcome::Succeeded]
        ))
    );
    assert_eq!(results[6], Err(ModelError::ClosedScope));
}

#[test]
fn requested_cancellation_can_finish_as_failure() {
    let actions = [
        Action::DeclareTask,
        Action::Admit,
        Action::RequestCancel,
        Action::Succeed,
        Action::Fail,
        Action::Join,
        Action::Close,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[3], Err(ModelError::InvalidTransition));
    assert_eq!(
        results[6],
        Ok(Signature::Closed(
            ScopeOutcome::Failed,
            vec![TaskOutcome::Failed(17)]
        ))
    );
}

#[test]
fn acknowledged_cancellation_reports_cancelled() {
    let actions = [
        Action::DeclareTask,
        Action::Admit,
        Action::Start,
        Action::RequestCancel,
        Action::AcknowledgeCancel,
        Action::Join,
        Action::Close,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(
        results[6],
        Ok(Signature::Closed(
            ScopeOutcome::Cancelled,
            vec![TaskOutcome::Cancelled]
        ))
    );
}

#[test]
fn both_budget_zero_cases_deny_creation_without_partial_state() {
    let actions = [
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::AcquireRead,
        Action::DeclareTask,
        Action::Close,
    ];
    let results = run(&actions, 0, 0, MAX_EVENTS);
    assert_eq!(results[2], Err(ModelError::Exhausted));
    assert_eq!(results[3], Err(ModelError::Exhausted));
    assert_eq!(
        results[4],
        Ok(Signature::Closed(ScopeOutcome::Succeeded, vec![]))
    );
}

#[test]
fn unsupported_effect_is_denied_before_authority_creation() {
    let actions = [
        Action::ExternalIoDeclaration,
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::AcquireRead,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[0], Err(ModelError::UnsupportedEffect));
    assert_eq!(results[2], Ok(Signature::Grant(1)));
}

#[test]
fn foreign_domain_handles_do_not_provide_scope_resource_or_task_authority() {
    let actions = [
        Action::ForeignScopeTask,
        Action::ForeignTaskStart,
        Action::DeclareRead,
        Action::HostReadGrant,
        Action::ForeignGrantAcquire,
        Action::AcquireRead,
        Action::ForeignResourceUse,
        Action::WrongHostGrant,
    ];
    let results = run(&actions, 1, 1, MAX_EVENTS);
    assert_eq!(results[0], Err(ModelError::UnknownScope));
    assert_eq!(results[1], Err(ModelError::UnknownTask));
    assert_eq!(results[4], Err(ModelError::UnknownGrant));
    assert_eq!(results[6], Err(ModelError::UnknownResource));
    assert_eq!(results[7], Err(ModelError::WrongHostPermit));
}

#[test]
fn event_budget_does_not_hide_wrong_host_witness_denial() {
    let actions = [
        Action::HostReadGrant,
        Action::WrongHostGrant,
        Action::DeclareRead,
        Action::Close,
    ];
    let results = run(&actions, 1, 1, 1);
    assert_eq!(results[0], Ok(Signature::Grant(1)));
    assert_eq!(results[1], Err(ModelError::WrongHostPermit));
    assert_eq!(results[2], Err(ModelError::Exhausted));
    assert_eq!(results[3], Err(ModelError::Exhausted));
}

#[test]
fn identical_oracle_inputs_are_reproducible_without_platform_randomness() {
    let script = generated(0x4321, 120);
    let first = run(&script, 1, 1, MAX_EVENTS);
    let second = run(&script, 1, 1, MAX_EVENTS);
    assert_eq!(first, second);
    assert_ne!(generated(1, 30), generated(2, 30));
}

#[test]
fn synthetic_oracle_mutation_is_detected_not_a_real_bug() {
    let script = [
        Action::DeclareTask,
        Action::Admit,
        Action::Succeed,
        Action::Join,
        Action::Close,
    ];
    let correct = run(&script, 1, 1, MAX_EVENTS);
    let mut counterfeit = correct.clone();
    counterfeit[4] = Ok(Signature::Closed(
        ScopeOutcome::Failed,
        vec![TaskOutcome::Failed(17)],
    ));
    assert_ne!(correct, counterfeit, "synthetic mutation must be detected");
}

#[test]
fn r06_remains_test_only_and_does_not_export_new_authority() {
    let lib = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");
    let source = include_str!("../src/resource_task_r02.rs");
    assert!(!lib.contains("resource_task_r06"));
    assert!(!lib.contains("resource_task_r02"));
    assert!(!manifest.contains("resource_task_r06"));
    assert!(source.contains("intentionally NOT exported from lib.rs"));
}
