//! R0.9 — independent bounded sibling-branch resource/task conformance.
//! TEST-ONLY. No production concurrency, scheduling, runtime integration or authority.
//! The oracle independently predicts transitions for two sibling branches and leaves.

#[allow(dead_code)]
#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{
    Command, GrantId, HostPermit, Model, ModelError, Receipt, ResourceEffect, ResourceId,
    ScopeBudget, ScopeId, ScopeOutcome, TaskId, TaskOutcome,
};

const DOMAIN: u64 = 0x5230_3901;
const OTHER_DOMAIN: u64 = 0x5230_3902;
const SEEDS: usize = 40;
const STEPS: usize = 160;
const LIMIT: usize = 144;
const WORD_ALPHABET: usize = 7;
const WORD_DEPTH: usize = 5;

fn budget(tasks: usize, resources: usize, children: usize) -> ScopeBudget {
    ScopeBudget {
        tasks,
        resources,
        children,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Child {
        parent: usize,
    },
    Declare {
        scope: usize,
        effect: ResourceEffect,
    },
    Grant {
        scope: usize,
        effect: ResourceEffect,
    },
    BadPermit {
        scope: usize,
    },
    Acquire {
        scope: usize,
        grant: usize,
        effect: ResourceEffect,
    },
    Use {
        scope: usize,
        resource: usize,
        grant: usize,
        effect: ResourceEffect,
    },
    Revoke {
        grant: usize,
    },
    Release {
        scope: usize,
        resource: usize,
    },
    Task {
        scope: usize,
    },
    Admit {
        task: usize,
    },
    Start {
        task: usize,
    },
    Cancel {
        task: usize,
    },
    Succeed {
        task: usize,
    },
    Fail {
        task: usize,
    },
    Acknowledge {
        task: usize,
    },
    Join {
        task: usize,
    },
    Close {
        scope: usize,
    },
}

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
    Scope(usize),
    Grant(usize),
    Resource(usize),
    Task(usize),
    Closed {
        scope: usize,
        outcome: ScopeOutcome,
        tasks: Vec<(usize, TaskOutcome)>,
        children: Vec<(usize, ScopeOutcome)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OScope {
    budget: ScopeBudget,
    effects: Vec<ResourceEffect>,
    children: Vec<usize>,
    resources: Vec<usize>,
    tasks: Vec<usize>,
    closed: Option<Signature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OGrant {
    scope: usize,
    effect: ResourceEffect,
    revoked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OResource {
    scope: usize,
    effect: ResourceEffect,
    live: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OTask {
    scope: usize,
    phase: Phase,
    cancel: bool,
    outcome: Option<TaskOutcome>,
}

/// Separate transition specification: no R0.2 state inspection or Model helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Oracle {
    scopes: Vec<OScope>,
    grants: Vec<OGrant>,
    resources: Vec<OResource>,
    tasks: Vec<OTask>,
    events: usize,
    limit: usize,
}

impl Oracle {
    fn new(limit: usize) -> Self {
        Self {
            scopes: vec![OScope {
                budget: budget(2, 2, 2),
                effects: vec![],
                children: vec![],
                resources: vec![],
                tasks: vec![],
                closed: None,
            }],
            grants: vec![],
            resources: vec![],
            tasks: vec![],
            events: 0,
            limit,
        }
    }

    fn open(&self, index: usize) -> Result<(), ModelError> {
        let scope = self.scopes.get(index).ok_or(ModelError::UnknownScope)?;
        if scope.closed.is_some() {
            Err(ModelError::ClosedScope)
        } else {
            Ok(())
        }
    }

    fn effect(&self, scope: usize, effect: ResourceEffect) -> Result<(), ModelError> {
        if effect == ResourceEffect::ExternalIo {
            return Err(ModelError::UnsupportedEffect);
        }
        self.open(scope)?;
        if !self.scopes[scope].effects.contains(&effect) {
            return Err(ModelError::MissingDeclaration);
        }
        Ok(())
    }

    fn grant(&self, grant: usize, scope: usize, effect: ResourceEffect) -> Result<(), ModelError> {
        let record = self.grants.get(grant).ok_or(ModelError::UnknownGrant)?;
        if record.revoked {
            return Err(ModelError::RevokedGrant);
        }
        if record.scope != scope || record.effect != effect {
            return Err(ModelError::GrantMismatch);
        }
        Ok(())
    }

    fn task_open(&self, index: usize) -> Result<Phase, ModelError> {
        let task = self.tasks.get(index).ok_or(ModelError::UnknownTask)?;
        self.open(task.scope)?;
        Ok(task.phase)
    }

    fn step(&mut self, act: Act) -> Result<Signature, ModelError> {
        // Host permit is checked before the event budget in the frozen model.
        if matches!(act, Act::BadPermit { .. }) {
            return Err(ModelError::WrongHostPermit);
        }
        if self.events >= self.limit {
            return Err(ModelError::Exhausted);
        }
        let mut candidate = self.clone();
        let receipt = candidate.decide(act)?;
        candidate.events += 1;
        *self = candidate;
        Ok(receipt)
    }

    fn decide(&mut self, act: Act) -> Result<Signature, ModelError> {
        match act {
            Act::Child { parent } => {
                self.open(parent)?;
                if self.scopes[parent].children.len() >= self.scopes[parent].budget.children {
                    return Err(ModelError::Exhausted);
                }
                let id = self.scopes.len();
                self.scopes[parent].children.push(id);
                self.scopes.push(OScope {
                    budget: if parent == 0 {
                        budget(2, 2, 1)
                    } else {
                        budget(2, 2, 0)
                    },
                    effects: vec![],
                    children: vec![],
                    resources: vec![],
                    tasks: vec![],
                    closed: None,
                });
                Ok(Signature::Scope(id))
            }
            Act::Declare { scope, effect } => {
                self.open(scope)?;
                if effect == ResourceEffect::ExternalIo {
                    return Err(ModelError::UnsupportedEffect);
                }
                if self.scopes[scope].effects.contains(&effect) {
                    return Err(ModelError::InvalidTransition);
                }
                self.scopes[scope].effects.push(effect);
                Ok(Signature::Unit)
            }
            Act::Grant { scope, effect } => {
                self.open(scope)?;
                if effect == ResourceEffect::ExternalIo {
                    return Err(ModelError::UnsupportedEffect);
                }
                let id = self.grants.len();
                self.grants.push(OGrant {
                    scope,
                    effect,
                    revoked: false,
                });
                Ok(Signature::Grant(id))
            }
            Act::BadPermit { .. } => Err(ModelError::WrongHostPermit),
            Act::Acquire {
                scope,
                grant,
                effect,
            } => {
                self.effect(scope, effect)?;
                self.grant(grant, scope, effect)?;
                if self.scopes[scope].resources.len() >= self.scopes[scope].budget.resources {
                    return Err(ModelError::Exhausted);
                }
                let id = self.resources.len();
                self.resources.push(OResource {
                    scope,
                    effect,
                    live: true,
                });
                self.scopes[scope].resources.push(id);
                Ok(Signature::Resource(id))
            }
            Act::Use {
                scope,
                resource,
                grant,
                effect,
            } => {
                self.effect(scope, effect)?;
                let item = self
                    .resources
                    .get(resource)
                    .ok_or(ModelError::UnknownResource)?;
                if item.scope != scope {
                    return Err(ModelError::WrongScope);
                }
                if !item.live {
                    return Err(ModelError::ReleasedResource);
                }
                if item.effect != effect {
                    return Err(ModelError::GrantMismatch);
                }
                self.grant(grant, scope, effect)?;
                Ok(Signature::Unit)
            }
            Act::Revoke { grant } => {
                let item = self.grants.get_mut(grant).ok_or(ModelError::UnknownGrant)?;
                if item.revoked {
                    return Err(ModelError::InvalidTransition);
                }
                item.revoked = true;
                Ok(Signature::Unit)
            }
            Act::Release { scope, resource } => {
                self.open(scope)?;
                let item = self
                    .resources
                    .get_mut(resource)
                    .ok_or(ModelError::UnknownResource)?;
                if item.scope != scope {
                    return Err(ModelError::WrongScope);
                }
                if !item.live {
                    return Err(ModelError::ReleasedResource);
                }
                item.live = false;
                Ok(Signature::Unit)
            }
            Act::Task { scope } => {
                self.open(scope)?;
                if self.scopes[scope].tasks.len() >= self.scopes[scope].budget.tasks {
                    return Err(ModelError::Exhausted);
                }
                let id = self.tasks.len();
                self.tasks.push(OTask {
                    scope,
                    phase: Phase::Declared,
                    cancel: false,
                    outcome: None,
                });
                self.scopes[scope].tasks.push(id);
                Ok(Signature::Task(id))
            }
            Act::Admit { task } => {
                if self.task_open(task)? != Phase::Declared {
                    return Err(ModelError::InvalidTransition);
                }
                self.tasks[task].phase = Phase::Admitted;
                Ok(Signature::Unit)
            }
            Act::Start { task } => {
                let phase = self.task_open(task)?;
                if self.tasks[task].cancel || phase != Phase::Admitted {
                    return Err(ModelError::InvalidTransition);
                }
                self.tasks[task].phase = Phase::Running;
                Ok(Signature::Unit)
            }
            Act::Cancel { task } => {
                let phase = self.task_open(task)?;
                if !matches!(phase, Phase::Admitted | Phase::Running) || self.tasks[task].cancel {
                    return Err(ModelError::InvalidTransition);
                }
                self.tasks[task].cancel = true;
                Ok(Signature::Unit)
            }
            Act::Succeed { task } | Act::Fail { task } | Act::Acknowledge { task } => {
                if matches!(act, Act::Acknowledge { .. }) {
                    let target = self.tasks.get(task).ok_or(ModelError::UnknownTask)?;
                    if !target.cancel {
                        return Err(ModelError::InvalidTransition);
                    }
                }
                let phase = self.task_open(task)?;
                if !matches!(phase, Phase::Admitted | Phase::Running) {
                    return Err(ModelError::InvalidTransition);
                }
                if matches!(act, Act::Succeed { .. }) && self.tasks[task].cancel {
                    return Err(ModelError::InvalidTransition);
                }
                let outcome = match act {
                    Act::Succeed { .. } => TaskOutcome::Succeeded,
                    Act::Fail { .. } => TaskOutcome::Failed(17),
                    Act::Acknowledge { .. } => TaskOutcome::Cancelled,
                    _ => unreachable!(),
                };
                self.tasks[task].phase = Phase::Terminal;
                self.tasks[task].outcome = Some(outcome);
                Ok(Signature::Unit)
            }
            Act::Join { task } => {
                if self.task_open(task)? != Phase::Terminal {
                    return Err(ModelError::InvalidTransition);
                }
                self.tasks[task].phase = Phase::Joined;
                Ok(Signature::Unit)
            }
            Act::Close { scope } => {
                self.open(scope)?;
                let node = &self.scopes[scope];
                if node
                    .children
                    .iter()
                    .any(|&c| self.scopes[c].closed.is_none())
                {
                    return Err(ModelError::OutstandingChildren);
                }
                if node.resources.iter().any(|&r| self.resources[r].live) {
                    return Err(ModelError::OutstandingResources);
                }
                if node
                    .tasks
                    .iter()
                    .any(|&t| self.tasks[t].phase != Phase::Joined)
                {
                    return Err(ModelError::OutstandingTasks);
                }
                let tasks: Vec<_> = node
                    .tasks
                    .iter()
                    .map(|&id| (id, self.tasks[id].outcome.expect("joined has outcome")))
                    .collect();
                let children: Vec<_> = node
                    .children
                    .iter()
                    .map(|&id| {
                        let Some(Signature::Closed { outcome, .. }) = &self.scopes[id].closed
                        else {
                            unreachable!()
                        };
                        (id, *outcome)
                    })
                    .collect();
                let failure = tasks
                    .iter()
                    .any(|(_, t)| matches!(t, TaskOutcome::Failed(_)))
                    || children.iter().any(|(_, c)| *c == ScopeOutcome::Failed);
                let cancellation = tasks.iter().any(|(_, t)| *t == TaskOutcome::Cancelled)
                    || children.iter().any(|(_, c)| *c == ScopeOutcome::Cancelled);
                let outcome = if failure {
                    ScopeOutcome::Failed
                } else if cancellation {
                    ScopeOutcome::Cancelled
                } else {
                    ScopeOutcome::Succeeded
                };
                let result = Signature::Closed {
                    scope,
                    outcome,
                    tasks,
                    children,
                };
                self.scopes[scope].closed = Some(result.clone());
                Ok(result)
            }
        }
    }
}

struct Trial {
    model: Model,
    host: HostPermit,
    foreign_permit: HostPermit,
    foreign_scope: ScopeId,
    foreign_task: TaskId,
    foreign_resource: ResourceId,
    foreign_grant: GrantId,
    scopes: Vec<ScopeId>,
    grants: Vec<GrantId>,
    resources: Vec<ResourceId>,
    tasks: Vec<TaskId>,
    oracle: Oracle,
}

impl Trial {
    fn new(limit: usize) -> Self {
        let (model, host) = Model::bootstrap(DOMAIN, budget(2, 2, 2), limit);
        let root = model.root();
        let (mut foreign, foreign_permit) = Model::bootstrap(OTHER_DOMAIN, budget(1, 1, 0), 16);
        let foreign_scope = foreign.root();
        let foreign_grant = foreign
            .host_grant(&foreign_permit, foreign_scope, ResourceEffect::Read)
            .expect("foreign host grant");
        foreign
            .apply(Command::DeclareEffect {
                scope: foreign_scope,
                effect: ResourceEffect::Read,
            })
            .expect("foreign effect");
        let foreign_resource = match foreign
            .apply(Command::Acquire {
                scope: foreign_scope,
                grant: foreign_grant,
                effect: ResourceEffect::Read,
            })
            .expect("foreign resource")
        {
            Receipt::Resource(id) => id,
            _ => unreachable!(),
        };
        let foreign_task = match foreign
            .apply(Command::DeclareTask {
                scope: foreign_scope,
            })
            .expect("foreign task")
        {
            Receipt::Task(id) => id,
            _ => unreachable!(),
        };
        Self {
            model,
            host,
            foreign_permit,
            foreign_scope,
            foreign_task,
            foreign_resource,
            foreign_grant,
            scopes: vec![root],
            grants: vec![],
            resources: vec![],
            tasks: vec![],
            oracle: Oracle::new(limit),
        }
    }

    fn scope(&self, i: usize) -> ScopeId {
        self.scopes.get(i).copied().unwrap_or(self.foreign_scope)
    }
    fn grant(&self, i: usize) -> GrantId {
        self.grants.get(i).copied().unwrap_or(self.foreign_grant)
    }
    fn resource(&self, i: usize) -> ResourceId {
        self.resources
            .get(i)
            .copied()
            .unwrap_or(self.foreign_resource)
    }
    fn task(&self, i: usize) -> TaskId {
        self.tasks.get(i).copied().unwrap_or(self.foreign_task)
    }

    fn command(&self, act: Act) -> Command {
        match act {
            Act::Child { parent } => Command::OpenChild {
                parent: self.scope(parent),
                budget: if parent == 0 {
                    budget(2, 2, 1)
                } else {
                    budget(2, 2, 0)
                },
            },
            Act::Declare { scope, effect } => Command::DeclareEffect {
                scope: self.scope(scope),
                effect,
            },
            Act::Acquire {
                scope,
                grant,
                effect,
            } => Command::Acquire {
                scope: self.scope(scope),
                grant: self.grant(grant),
                effect,
            },
            Act::Use {
                scope,
                resource,
                grant,
                effect,
            } => Command::Use {
                scope: self.scope(scope),
                resource: self.resource(resource),
                grant: self.grant(grant),
                effect,
            },
            Act::Revoke { grant } => Command::RevokeGrant {
                grant: self.grant(grant),
            },
            Act::Release { scope, resource } => Command::Release {
                scope: self.scope(scope),
                resource: self.resource(resource),
            },
            Act::Task { scope } => Command::DeclareTask {
                scope: self.scope(scope),
            },
            Act::Admit { task } => Command::Admit {
                task: self.task(task),
            },
            Act::Start { task } => Command::Start {
                task: self.task(task),
            },
            Act::Cancel { task } => Command::RequestCancel {
                task: self.task(task),
            },
            Act::Succeed { task } => Command::Succeed {
                task: self.task(task),
            },
            Act::Fail { task } => Command::Fail {
                task: self.task(task),
                code: 17,
            },
            Act::Acknowledge { task } => Command::AcknowledgeCancel {
                task: self.task(task),
            },
            Act::Join { task } => Command::Join {
                task: self.task(task),
            },
            Act::Close { scope } => Command::Close {
                scope: self.scope(scope),
            },
            Act::Grant { .. } | Act::BadPermit { .. } => {
                unreachable!("host ops dispatched separately")
            }
        }
    }

    fn normalize(&mut self, receipt: Receipt) -> Signature {
        match receipt {
            Receipt::Unit => Signature::Unit,
            Receipt::Scope(id) => {
                assert_eq!(id.parts(), (DOMAIN, self.scopes.len() as u64 + 1));
                self.scopes.push(id);
                Signature::Scope(self.scopes.len() - 1)
            }
            Receipt::Resource(id) => {
                assert_eq!(id.parts(), (DOMAIN, self.resources.len() as u64 + 1));
                self.resources.push(id);
                Signature::Resource(self.resources.len() - 1)
            }
            Receipt::Task(id) => {
                assert_eq!(id.parts(), (DOMAIN, self.tasks.len() as u64 + 1));
                self.tasks.push(id);
                Signature::Task(self.tasks.len() - 1)
            }
            Receipt::Closed(report) => Signature::Closed {
                scope: self
                    .scopes
                    .iter()
                    .position(|id| *id == report.scope)
                    .expect("known scope report"),
                outcome: report.outcome,
                tasks: report
                    .tasks
                    .into_iter()
                    .map(|(id, outcome)| {
                        (
                            self.tasks
                                .iter()
                                .position(|known| *known == id)
                                .expect("known task report"),
                            outcome,
                        )
                    })
                    .collect(),
                children: report
                    .children
                    .into_iter()
                    .map(|(id, outcome)| {
                        (
                            self.scopes
                                .iter()
                                .position(|known| *known == id)
                                .expect("known child report"),
                            outcome,
                        )
                    })
                    .collect(),
            },
        }
    }

    fn step(&mut self, act: Act) -> Result<Signature, ModelError> {
        let before = self.model.clone();
        let oracle_before = self.oracle.clone();
        let events = self.model.event_count();
        let expected = self.oracle.step(act);
        let actual = match act {
            Act::Grant { scope, effect } => {
                let target = self.scope(scope);
                self.model.host_grant(&self.host, target, effect).map(|id| {
                    assert_eq!(id.parts(), (DOMAIN, self.grants.len() as u64 + 1));
                    self.grants.push(id);
                    Signature::Grant(self.grants.len() - 1)
                })
            }
            Act::BadPermit { scope } => {
                let target = self.scope(scope);
                self.model
                    .host_grant(&self.foreign_permit, target, ResourceEffect::Read)
                    .map(|_| panic!("foreign host permit must never succeed"))
            }
            _ => {
                let command = self.command(act);
                self.model
                    .apply(command)
                    .map(|receipt| self.normalize(receipt))
            }
        };
        assert_eq!(actual, expected, "R0.9 oracle mismatch act={act:?}");
        if actual.is_ok() {
            assert_eq!(self.model.event_count(), events + 1);
            assert_eq!(self.model.replay_checked(), Ok(()));
        } else {
            assert_eq!(self.model, before, "rejected event mutated SUT act={act:?}");
            assert_eq!(
                self.oracle, oracle_before,
                "rejected event mutated oracle act={act:?}"
            );
            assert_eq!(self.model.event_count(), events);
        }
        actual
    }
}

fn run(script: &[Act], limit: usize) -> Vec<Result<Signature, ModelError>> {
    let mut trial = Trial::new(limit);
    script.iter().map(|&act| trial.step(act)).collect()
}

// Stable, intentionally trivial deterministic sequence generator. No OS randomness.
fn generated(seed: u64, size: usize) -> Vec<Act> {
    let mut x = seed ^ 0xD09A_734E_A119_C3F5;
    (0..size)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let scope = ((x >> 8) % 6) as usize;
            let slot = ((x >> 16) % 10) as usize;
            let effect = if (x & 2) == 0 {
                ResourceEffect::Read
            } else {
                ResourceEffect::Write
            };
            match x % 21 {
                0 => Act::Child { parent: scope },
                1 => Act::Declare { scope, effect },
                2 => Act::Grant { scope, effect },
                3 => Act::Acquire {
                    scope,
                    grant: slot,
                    effect,
                },
                4 => Act::Use {
                    scope,
                    resource: slot,
                    grant: slot,
                    effect,
                },
                5 => Act::Release {
                    scope,
                    resource: slot,
                },
                6 => Act::Revoke { grant: slot },
                7 => Act::Task { scope },
                8 => Act::Admit { task: slot },
                9 => Act::Start { task: slot },
                10 => Act::Cancel { task: slot },
                11 => Act::Succeed { task: slot },
                12 => Act::Fail { task: slot },
                13 => Act::Acknowledge { task: slot },
                14 => Act::Join { task: slot },
                15 => Act::Close { scope },
                16 => Act::BadPermit { scope },
                17 => Act::Declare {
                    scope,
                    effect: ResourceEffect::ExternalIo,
                },
                18 => Act::Grant {
                    scope,
                    effect: ResourceEffect::ExternalIo,
                },
                19 => Act::Acquire {
                    scope,
                    grant: 99,
                    effect: ResourceEffect::Read,
                },
                _ => Act::Use {
                    scope,
                    resource: 99,
                    grant: 99,
                    effect: ResourceEffect::Read,
                },
            }
        })
        .collect()
}

fn base() -> Vec<Act> {
    vec![
        Act::Child { parent: 0 },
        Act::Child { parent: 0 },
        Act::Child { parent: 1 },
        Act::Child { parent: 2 },
    ]
}

// Each R0.9 witness uses Trial::step(), which checks predicted and actual
// results, exact error classes, rejection atomicity and causal-log replay.

#[test]
fn generated_sibling_tree_matches_independent_oracle() {
    let mut comparisons = 0;
    for seed in 1..=SEEDS as u64 {
        let mut script = base();
        script.extend(generated(seed, STEPS));
        assert_eq!(run(&script, LIMIT).len(), STEPS + 4);
        comparisons += STEPS;
    }
    assert_eq!(comparisons, 6_400);
    println!("R09_WITNESS version=1 kind=sibling_tree_oracle seeds={SEEDS} steps={STEPS} comparisons={comparisons} outcome=PASS");
}

#[test]
fn finite_seven_action_words_match_independent_oracle() {
    let alphabet = [
        Act::Child { parent: 0 },
        Act::Child { parent: 1 },
        Act::Child { parent: 2 },
        Act::Close { scope: 3 },
        Act::Close { scope: 4 },
        Act::Close { scope: 1 },
        Act::Close { scope: 0 },
    ];
    let mut count = 0;
    for mut encoded in 0..WORD_ALPHABET.pow(WORD_DEPTH as u32) {
        let mut word = Vec::new();
        for _ in 0..WORD_DEPTH {
            word.push(alphabet[encoded % WORD_ALPHABET]);
            encoded /= WORD_ALPHABET;
        }
        assert_eq!(run(&word, LIMIT).len(), WORD_DEPTH);
        count += 1;
    }
    assert_eq!(count, 16_807);
    println!(
        "R09_WITNESS version=1 kind=branch_words sequences={count} alphabet=7 depth=5 outcome=PASS"
    );
}

#[test]
fn root_has_exactly_two_sibling_child_slots() {
    let results = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(results[0], Ok(Signature::Scope(1)));
    assert_eq!(results[1], Ok(Signature::Scope(2)));
    assert_eq!(results[2], Err(ModelError::Exhausted));
    assert_eq!(
        results[5],
        Ok(Signature::Closed {
            scope: 0,
            outcome: ScopeOutcome::Succeeded,
            tasks: vec![],
            children: vec![(1, ScopeOutcome::Succeeded), (2, ScopeOutcome::Succeeded)],
        })
    );
}

#[test]
fn closing_first_sibling_does_not_replenish_child_quota() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Close { scope: 1 },
            Act::Child { parent: 0 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[3], Err(ModelError::Exhausted));
    assert!(matches!(out[5], Ok(Signature::Closed { .. })));
}

#[test]
fn siblings_each_own_one_nonrecyclable_leaf_slot() {
    let mut actions = base();
    actions.extend([
        Act::Child { parent: 1 },
        Act::Child { parent: 2 },
        Act::Child { parent: 3 },
        Act::Child { parent: 4 },
        Act::Child { parent: 0 },
    ]);
    let out = run(&actions, LIMIT);
    assert_eq!(
        &out[..4],
        &[
            Ok(Signature::Scope(1)),
            Ok(Signature::Scope(2)),
            Ok(Signature::Scope(3)),
            Ok(Signature::Scope(4)),
        ]
    );
    for result in &out[4..] {
        assert_eq!(result, &Err(ModelError::Exhausted));
    }
}

#[test]
fn closing_root_requires_both_sibling_branches() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Close { scope: 0 },
            Act::Close { scope: 1 },
            Act::Close { scope: 0 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[2], Err(ModelError::OutstandingChildren));
    assert_eq!(out[4], Err(ModelError::OutstandingChildren));
    assert!(matches!(out[6], Ok(Signature::Closed { .. })));
}

#[test]
fn leaf_must_close_before_its_own_branch() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Child { parent: 1 },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Close { scope: 3 },
            Act::Close { scope: 1 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[3], Err(ModelError::OutstandingChildren));
    assert!(matches!(&out[7], Ok(Signature::Closed { children, .. })
        if children == &vec![(1, ScopeOutcome::Succeeded), (2, ScopeOutcome::Succeeded)]));
}

#[test]
fn sibling_grants_are_not_ambient() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Declare {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 2,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 2,
                grant: 1,
                effect: ResourceEffect::Read,
            },
        ],
        LIMIT,
    );
    assert_eq!(out[5], Err(ModelError::GrantMismatch));
    assert_eq!(out[7], Ok(Signature::Resource(0)));
}

#[test]
fn sibling_resources_reject_foreign_use_and_release() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Declare {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Use {
                scope: 2,
                resource: 0,
                grant: 1,
                effect: ResourceEffect::Read,
            },
            Act::Release {
                scope: 2,
                resource: 0,
            },
            Act::Use {
                scope: 1,
                resource: 0,
                grant: 0,
                effect: ResourceEffect::Read,
            },
        ],
        LIMIT,
    );
    assert_eq!(out[7], Err(ModelError::WrongScope));
    assert_eq!(out[8], Err(ModelError::WrongScope));
    assert_eq!(out[9], Ok(Signature::Unit));
}

#[test]
fn cousin_resources_and_grants_are_scope_exact() {
    let mut actions = base();
    actions.extend([
        Act::Declare {
            scope: 3,
            effect: ResourceEffect::Read,
        },
        Act::Declare {
            scope: 4,
            effect: ResourceEffect::Read,
        },
        Act::Grant {
            scope: 3,
            effect: ResourceEffect::Read,
        },
        Act::Grant {
            scope: 4,
            effect: ResourceEffect::Read,
        },
        Act::Acquire {
            scope: 3,
            grant: 0,
            effect: ResourceEffect::Read,
        },
        Act::Use {
            scope: 4,
            resource: 0,
            grant: 1,
            effect: ResourceEffect::Read,
        },
        Act::Acquire {
            scope: 4,
            grant: 0,
            effect: ResourceEffect::Read,
        },
        Act::Release {
            scope: 4,
            resource: 0,
        },
    ]);
    let out = run(&actions, LIMIT);
    assert_eq!(out[9], Err(ModelError::WrongScope));
    assert_eq!(out[10], Err(ModelError::GrantMismatch));
    assert_eq!(out[11], Err(ModelError::WrongScope));
}

#[test]
fn ancestor_grant_does_not_authorize_leaf_acquisition() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Child { parent: 1 },
            Act::Declare {
                scope: 0,
                effect: ResourceEffect::Read,
            },
            Act::Declare {
                scope: 3,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 0,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 3,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 3,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 3,
                grant: 1,
                effect: ResourceEffect::Read,
            },
        ],
        LIMIT,
    );
    assert_eq!(out[6], Err(ModelError::GrantMismatch));
    assert_eq!(out[8], Ok(Signature::Resource(0)));
}

#[test]
fn revocation_on_one_branch_does_not_revoke_other_branch() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Declare {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 2,
                grant: 1,
                effect: ResourceEffect::Read,
            },
            Act::Revoke { grant: 0 },
            Act::Use {
                scope: 1,
                resource: 0,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Use {
                scope: 2,
                resource: 1,
                grant: 1,
                effect: ResourceEffect::Read,
            },
        ],
        LIMIT,
    );
    assert_eq!(out[9], Err(ModelError::RevokedGrant));
    assert_eq!(out[10], Ok(Signature::Unit));
}

#[test]
fn revoked_branch_authority_cannot_block_cleanup() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Revoke { grant: 0 },
            Act::Release {
                scope: 1,
                resource: 0,
            },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[6], Ok(Signature::Unit));
    assert!(matches!(out[9], Ok(Signature::Closed { .. })));
}

#[test]
fn local_resource_quota_is_never_recycled_or_shared() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 1,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Release {
                scope: 1,
                resource: 0,
            },
            Act::Acquire {
                scope: 1,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Declare {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 2,
                grant: 1,
                effect: ResourceEffect::Read,
            },
        ],
        LIMIT,
    );
    assert_eq!(out[7], Err(ModelError::Exhausted));
    assert_eq!(out[10], Ok(Signature::Resource(2)));
}

#[test]
fn failure_in_one_sibling_dominates_cancellation_in_other() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Task { scope: 1 },
            Act::Task { scope: 2 },
            Act::Admit { task: 0 },
            Act::Admit { task: 1 },
            Act::Fail { task: 0 },
            Act::Cancel { task: 1 },
            Act::Acknowledge { task: 1 },
            Act::Join { task: 1 },
            Act::Join { task: 0 },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert!(matches!(
        out[11],
        Ok(Signature::Closed {
            outcome: ScopeOutcome::Failed,
            ..
        })
    ));
    assert!(matches!(
        out[12],
        Ok(Signature::Closed {
            outcome: ScopeOutcome::Cancelled,
            ..
        })
    ));
    assert_eq!(
        out[13],
        Ok(Signature::Closed {
            scope: 0,
            outcome: ScopeOutcome::Failed,
            tasks: vec![],
            children: vec![(1, ScopeOutcome::Failed), (2, ScopeOutcome::Cancelled)],
        })
    );
}

#[test]
fn cancellations_from_both_leaf_branches_propagate_upwards() {
    let mut actions = base();
    actions.extend([
        Act::Task { scope: 3 },
        Act::Task { scope: 4 },
        Act::Admit { task: 0 },
        Act::Admit { task: 1 },
        Act::Cancel { task: 0 },
        Act::Cancel { task: 1 },
        Act::Acknowledge { task: 0 },
        Act::Acknowledge { task: 1 },
        Act::Join { task: 0 },
        Act::Join { task: 1 },
        Act::Close { scope: 3 },
        Act::Close { scope: 4 },
        Act::Close { scope: 1 },
        Act::Close { scope: 2 },
        Act::Close { scope: 0 },
    ]);
    let out = run(&actions, LIMIT);
    for receipt in &out[14..=18] {
        assert!(matches!(
            receipt,
            Ok(Signature::Closed {
                outcome: ScopeOutcome::Cancelled,
                ..
            })
        ));
    }
    assert_eq!(
        out[18],
        Ok(Signature::Closed {
            scope: 0,
            outcome: ScopeOutcome::Cancelled,
            tasks: vec![],
            children: vec![(1, ScopeOutcome::Cancelled), (2, ScopeOutcome::Cancelled)],
        })
    );
}

#[test]
fn root_child_reports_are_sorted_even_when_close_order_is_reversed() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Close { scope: 2 },
            Act::Close { scope: 1 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(
        out[4],
        Ok(Signature::Closed {
            scope: 0,
            outcome: ScopeOutcome::Succeeded,
            tasks: vec![],
            children: vec![(1, ScopeOutcome::Succeeded), (2, ScopeOutcome::Succeeded)],
        })
    );
}

#[test]
fn blockers_remain_ordered_across_sibling_tree() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Task { scope: 0 },
            Act::Declare {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Grant {
                scope: 2,
                effect: ResourceEffect::Read,
            },
            Act::Acquire {
                scope: 2,
                grant: 0,
                effect: ResourceEffect::Read,
            },
            Act::Close { scope: 0 },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Release {
                scope: 2,
                resource: 0,
            },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
            Act::Admit { task: 0 },
            Act::Succeed { task: 0 },
            Act::Join { task: 0 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[6], Err(ModelError::OutstandingChildren));
    assert_eq!(out[8], Err(ModelError::OutstandingResources));
    assert_eq!(out[11], Err(ModelError::OutstandingTasks));
    assert!(matches!(out[15], Ok(Signature::Closed { .. })));
}

#[test]
fn malformed_cross_branch_transitions_are_atomic() {
    let out = run(
        &[
            Act::Child { parent: 0 },
            Act::Child { parent: 0 },
            Act::Task { scope: 99 },
            Act::Admit { task: 99 },
            Act::BadPermit { scope: 2 },
            Act::Declare {
                scope: 1,
                effect: ResourceEffect::ExternalIo,
            },
            Act::Use {
                scope: 2,
                resource: 99,
                grant: 99,
                effect: ResourceEffect::Read,
            },
            Act::Close { scope: 1 },
            Act::Close { scope: 2 },
            Act::Close { scope: 0 },
        ],
        LIMIT,
    );
    assert_eq!(out[2], Err(ModelError::UnknownScope));
    assert_eq!(out[3], Err(ModelError::UnknownTask));
    assert_eq!(out[4], Err(ModelError::WrongHostPermit));
    assert_eq!(out[5], Err(ModelError::UnsupportedEffect));
    assert_eq!(out[6], Err(ModelError::MissingDeclaration));
    assert!(out[9].is_ok());
}

fn four_pair_orders() -> Vec<[usize; 8]> {
    fn visit(pos: usize, used: u16, order: &mut [usize; 8], output: &mut Vec<[usize; 8]>) {
        if pos == 8 {
            output.push(*order);
            return;
        }
        for op in 0..8 {
            let bit = 1u16 << op;
            if used & bit != 0 || ((op & 1) == 1 && used & (1u16 << (op - 1)) == 0) {
                continue;
            }
            order[pos] = op;
            visit(pos + 1, used | bit, order, output);
        }
    }
    let mut results = Vec::new();
    visit(0, 0, &mut [0usize; 8], &mut results);
    results
}

#[test]
fn all_four_task_finish_join_orders_have_canonical_reports() {
    let orders = four_pair_orders();
    assert_eq!(orders.len(), 2_520);
    let mut baseline: Option<Vec<Signature>> = None;
    for order in &orders {
        let mut trial = Trial::new(LIMIT);
        for act in base() {
            assert!(trial.step(act).is_ok());
        }
        // One task each in root, both siblings and the first grandchild.
        for scope in [0, 1, 2, 3] {
            assert!(trial.step(Act::Task { scope }).is_ok());
        }
        for task in 0..4 {
            assert!(trial.step(Act::Admit { task }).is_ok());
        }
        for &event in order {
            let task = event / 2;
            let act = if event & 1 == 0 {
                Act::Succeed { task }
            } else {
                Act::Join { task }
            };
            assert!(trial.step(act).is_ok());
        }
        let mut reports = Vec::new();
        for scope in [3, 4, 1, 2, 0] {
            let report = trial
                .step(Act::Close { scope })
                .expect("valid leaf-first close");
            reports.push(report);
        }
        if let Some(expected) = &baseline {
            assert_eq!(&reports, expected);
        } else {
            assert_eq!(
                reports[4],
                Signature::Closed {
                    scope: 0,
                    outcome: ScopeOutcome::Succeeded,
                    tasks: vec![(0, TaskOutcome::Succeeded)],
                    children: vec![(1, ScopeOutcome::Succeeded), (2, ScopeOutcome::Succeeded)],
                }
            );
            baseline = Some(reports);
        }
    }
    println!(
        "R09_WITNESS version=1 kind=four_task_interleavings explored={} limit=2520 outcome=PASS",
        orders.len()
    );
}

#[test]
fn repeated_branch_scripts_are_byte_deterministic_as_results() {
    let mut script = base();
    script.extend([
        Act::Task { scope: 1 },
        Act::Task { scope: 2 },
        Act::Admit { task: 0 },
        Act::Admit { task: 1 },
        Act::Fail { task: 0 },
        Act::Succeed { task: 1 },
        Act::Join { task: 1 },
        Act::Join { task: 0 },
        Act::Close { scope: 3 },
        Act::Close { scope: 4 },
        Act::Close { scope: 1 },
        Act::Close { scope: 2 },
        Act::Close { scope: 0 },
    ]);
    assert_eq!(run(&script, LIMIT), run(&script, LIMIT));
}

#[test]
fn synthetic_oracle_mutation_detection_is_only_a_test_of_the_harness() {
    let original = run(&[Act::Child { parent: 0 }], LIMIT);
    assert_eq!(original[0], Ok(Signature::Scope(1)));
    let mut intentionally_wrong = original.clone();
    intentionally_wrong[0] = Ok(Signature::Scope(2));
    assert_ne!(intentionally_wrong, original);
    // A synthetic mismatch is not evidence of an actual R0.2 flaw.
}

#[test]
fn r09_is_test_only_and_cannot_export_host_authority() {
    let lib = include_str!("../src/lib.rs");
    let cargo = include_str!("../Cargo.toml");
    assert!(!lib.contains("mod resource_task_r09"));
    assert!(!lib.contains("mod resource_task_r02"));
    assert!(!cargo.contains("resource_task_r09"));
    assert!(!cargo.contains("resource_task_r02"));
}
