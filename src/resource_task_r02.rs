//! R0.2: pure, bounded resource/task transition reference model.
//!
//! This file is intentionally NOT exported from lib.rs. It is compiled by the
//! dedicated integration tests only. It does not run work, schedule threads,
//! perform I/O, or grant authority to executable NORDOI programs.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeId(u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrantId(u64, u64);

impl ScopeId {
    pub fn parts(self) -> (u64, u64) {
        (self.0, self.1)
    }
}

impl TaskId {
    pub fn parts(self) -> (u64, u64) {
        (self.0, self.1)
    }
}

impl ResourceId {
    pub fn parts(self) -> (u64, u64) {
        (self.0, self.1)
    }
}

impl GrantId {
    pub fn parts(self) -> (u64, u64) {
        (self.0, self.1)
    }
}

/// Only trusted embedding code receives this witness from `Model::bootstrap`.
/// It is NOT a security boundary for untrusted native Rust callers.
pub struct HostPermit(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceEffect {
    Read,
    Write,
    ExternalIo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeBudget {
    pub tasks: usize,
    pub resources: usize,
    pub children: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    Exhausted,
    UnknownScope,
    ClosedScope,
    UnknownTask,
    UnknownResource,
    UnknownGrant,
    WrongHostPermit,
    WrongScope,
    MissingDeclaration,
    GrantMismatch,
    RevokedGrant,
    ReleasedResource,
    UnsupportedEffect,
    InvalidTransition,
    OutstandingChildren,
    OutstandingResources,
    OutstandingTasks,
    ReplayMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskPhase {
    Declared,
    Admitted,
    Running,
    Terminal,
    Joined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskOutcome {
    Succeeded,
    Failed(u16),
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeOutcome {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeReport {
    pub scope: ScopeId,
    pub outcome: ScopeOutcome,
    /// Sorted by semantic task identity, never by completion order.
    pub tasks: Vec<(TaskId, TaskOutcome)>,
    /// Sorted by semantic child-scope identity.
    pub children: Vec<(ScopeId, ScopeOutcome)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    OpenChild {
        parent: ScopeId,
        budget: ScopeBudget,
    },
    DeclareEffect {
        scope: ScopeId,
        effect: ResourceEffect,
    },
    RevokeGrant {
        grant: GrantId,
    },
    Acquire {
        scope: ScopeId,
        grant: GrantId,
        effect: ResourceEffect,
    },
    Use {
        scope: ScopeId,
        resource: ResourceId,
        grant: GrantId,
        effect: ResourceEffect,
    },
    Release {
        scope: ScopeId,
        resource: ResourceId,
    },
    DeclareTask {
        scope: ScopeId,
    },
    Admit {
        task: TaskId,
    },
    Start {
        task: TaskId,
    },
    RequestCancel {
        task: TaskId,
    },
    Succeed {
        task: TaskId,
    },
    Fail {
        task: TaskId,
        code: u16,
    },
    AcknowledgeCancel {
        task: TaskId,
    },
    Join {
        task: TaskId,
    },
    Close {
        scope: ScopeId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipt {
    Unit,
    Scope(ScopeId),
    Resource(ResourceId),
    Task(TaskId),
    Closed(ScopeReport),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Record {
    Command(Command, Receipt),
    HostGrant {
        scope: ScopeId,
        effect: ResourceEffect,
        result: GrantId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopeState {
    parent: Option<ScopeId>,
    budget: ScopeBudget,
    children: BTreeSet<ScopeId>,
    resources: BTreeSet<ResourceId>,
    tasks: BTreeSet<TaskId>,
    effects: BTreeSet<ResourceEffect>,
    closed: Option<ScopeReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaskState {
    scope: ScopeId,
    phase: TaskPhase,
    cancel_requested: bool,
    result: Option<TaskOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceState {
    scope: ScopeId,
    effect: ResourceEffect,
    live: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GrantState {
    scope: ScopeId,
    effect: ResourceEffect,
    revoked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    domain: u64,
    root_budget: ScopeBudget,
    max_events: usize,
    scopes: BTreeMap<ScopeId, ScopeState>,
    tasks: BTreeMap<TaskId, TaskState>,
    resources: BTreeMap<ResourceId, ResourceState>,
    grants: BTreeMap<GrantId, GrantState>,
    next_scope: u64,
    next_task: u64,
    next_resource: u64,
    next_grant: u64,
    log: Vec<Record>,
}

impl Model {
    /// Trusted embedding bootstrap. Root scope id 1 is not ambient authority.
    pub fn bootstrap(
        domain: u64,
        root_budget: ScopeBudget,
        max_events: usize,
    ) -> (Self, HostPermit) {
        let mut scopes = BTreeMap::new();
        scopes.insert(
            ScopeId(domain, 1),
            ScopeState {
                parent: None,
                budget: root_budget,
                children: BTreeSet::new(),
                resources: BTreeSet::new(),
                tasks: BTreeSet::new(),
                effects: BTreeSet::new(),
                closed: None,
            },
        );
        (
            Self {
                domain,
                root_budget,
                max_events,
                scopes,
                tasks: BTreeMap::new(),
                resources: BTreeMap::new(),
                grants: BTreeMap::new(),
                next_scope: 2,
                next_task: 1,
                next_resource: 1,
                next_grant: 1,
                log: Vec::new(),
            },
            HostPermit(domain),
        )
    }

    pub fn root(&self) -> ScopeId {
        ScopeId(self.domain, 1)
    }

    pub fn parent(&self, scope: ScopeId) -> Option<ScopeId> {
        self.scopes.get(&scope).and_then(|state| state.parent)
    }

    pub fn event_count(&self) -> usize {
        self.log.len()
    }

    pub fn phase(&self, task: TaskId) -> Option<TaskPhase> {
        self.tasks.get(&task).map(|state| state.phase)
    }

    pub fn outcome(&self, task: TaskId) -> Option<TaskOutcome> {
        self.tasks.get(&task).and_then(|state| state.result)
    }

    pub fn report(&self, scope: ScopeId) -> Option<&ScopeReport> {
        self.scopes
            .get(&scope)
            .and_then(|state| state.closed.as_ref())
    }

    fn open(&self, scope: ScopeId) -> Result<&ScopeState, ModelError> {
        let state = self.scopes.get(&scope).ok_or(ModelError::UnknownScope)?;
        if state.closed.is_some() {
            return Err(ModelError::ClosedScope);
        }
        Ok(state)
    }

    fn grant_matches(
        &self,
        grant: GrantId,
        scope: ScopeId,
        effect: ResourceEffect,
    ) -> Result<(), ModelError> {
        let state = self.grants.get(&grant).ok_or(ModelError::UnknownGrant)?;
        if state.revoked {
            return Err(ModelError::RevokedGrant);
        }
        if state.scope != scope || state.effect != effect {
            return Err(ModelError::GrantMismatch);
        }
        Ok(())
    }

    fn ensure_effect(&self, scope: ScopeId, effect: ResourceEffect) -> Result<(), ModelError> {
        if effect == ResourceEffect::ExternalIo {
            return Err(ModelError::UnsupportedEffect);
        }
        if !self.open(scope)?.effects.contains(&effect) {
            return Err(ModelError::MissingDeclaration);
        }
        Ok(())
    }

    /// Every accepted event is atomic in THIS PURE MODEL: work is first applied
    /// to a candidate clone, then committed only on success. No host effects.
    pub fn apply(&mut self, command: Command) -> Result<Receipt, ModelError> {
        if self.log.len() >= self.max_events {
            return Err(ModelError::Exhausted);
        }
        let mut candidate = self.clone();
        let receipt = candidate.apply_candidate(&command)?;
        candidate
            .log
            .push(Record::Command(command, receipt.clone()));
        *self = candidate;
        Ok(receipt)
    }

    /// Host-only model operation. The witness is deliberately not exposed to
    /// NORDOI source execution or scheduler code; the model has neither.
    pub fn host_grant(
        &mut self,
        permit: &HostPermit,
        scope: ScopeId,
        effect: ResourceEffect,
    ) -> Result<GrantId, ModelError> {
        if permit.0 != self.domain {
            return Err(ModelError::WrongHostPermit);
        }
        if self.log.len() >= self.max_events {
            return Err(ModelError::Exhausted);
        }
        let mut candidate = self.clone();
        candidate.open(scope)?;
        if effect == ResourceEffect::ExternalIo {
            return Err(ModelError::UnsupportedEffect);
        }
        let id = GrantId(candidate.domain, candidate.next_grant);
        candidate.next_grant = candidate
            .next_grant
            .checked_add(1)
            .ok_or(ModelError::Exhausted)?;
        candidate.grants.insert(
            id,
            GrantState {
                scope,
                effect,
                revoked: false,
            },
        );
        candidate.log.push(Record::HostGrant {
            scope,
            effect,
            result: id,
        });
        *self = candidate;
        Ok(id)
    }

    /// Re-executes the exact accepted causal sequence in a fresh pure model.
    /// Assumes the log is trusted; this is not a durable signed replay format.
    pub fn replay_checked(&self) -> Result<(), ModelError> {
        let (mut replay, permit) = Self::bootstrap(self.domain, self.root_budget, self.max_events);
        for record in &self.log {
            match record {
                Record::Command(command, expected) => {
                    let actual = replay
                        .apply(*command)
                        .map_err(|_| ModelError::ReplayMismatch)?;
                    if &actual != expected {
                        return Err(ModelError::ReplayMismatch);
                    }
                }
                Record::HostGrant {
                    scope,
                    effect,
                    result,
                } => {
                    let actual = replay
                        .host_grant(&permit, *scope, *effect)
                        .map_err(|_| ModelError::ReplayMismatch)?;
                    if &actual != result {
                        return Err(ModelError::ReplayMismatch);
                    }
                }
            }
        }
        if &replay == self {
            Ok(())
        } else {
            Err(ModelError::ReplayMismatch)
        }
    }

    fn apply_candidate(&mut self, command: &Command) -> Result<Receipt, ModelError> {
        match *command {
            Command::OpenChild { parent, budget } => {
                let current = self.open(parent)?;
                if current.children.len() >= current.budget.children {
                    return Err(ModelError::Exhausted);
                }
                let id = ScopeId(self.domain, self.next_scope);
                self.next_scope = self
                    .next_scope
                    .checked_add(1)
                    .ok_or(ModelError::Exhausted)?;
                self.scopes
                    .get_mut(&parent)
                    .ok_or(ModelError::UnknownScope)?
                    .children
                    .insert(id);
                self.scopes.insert(
                    id,
                    ScopeState {
                        parent: Some(parent),
                        budget,
                        children: BTreeSet::new(),
                        resources: BTreeSet::new(),
                        tasks: BTreeSet::new(),
                        effects: BTreeSet::new(),
                        closed: None,
                    },
                );
                Ok(Receipt::Scope(id))
            }
            Command::DeclareEffect { scope, effect } => {
                self.open(scope)?;
                if effect == ResourceEffect::ExternalIo {
                    return Err(ModelError::UnsupportedEffect);
                }
                if !self
                    .scopes
                    .get_mut(&scope)
                    .ok_or(ModelError::UnknownScope)?
                    .effects
                    .insert(effect)
                {
                    return Err(ModelError::InvalidTransition);
                }
                Ok(Receipt::Unit)
            }
            Command::RevokeGrant { grant } => {
                let state = self
                    .grants
                    .get_mut(&grant)
                    .ok_or(ModelError::UnknownGrant)?;
                if state.revoked {
                    return Err(ModelError::InvalidTransition);
                }
                state.revoked = true;
                Ok(Receipt::Unit)
            }
            Command::Acquire {
                scope,
                grant,
                effect,
            } => {
                self.ensure_effect(scope, effect)?;
                self.grant_matches(grant, scope, effect)?;
                let current = self.open(scope)?;
                if current.resources.len() >= current.budget.resources {
                    return Err(ModelError::Exhausted);
                }
                let id = ResourceId(self.domain, self.next_resource);
                self.next_resource = self
                    .next_resource
                    .checked_add(1)
                    .ok_or(ModelError::Exhausted)?;
                self.resources.insert(
                    id,
                    ResourceState {
                        scope,
                        effect,
                        live: true,
                    },
                );
                self.scopes
                    .get_mut(&scope)
                    .ok_or(ModelError::UnknownScope)?
                    .resources
                    .insert(id);
                Ok(Receipt::Resource(id))
            }
            Command::Use {
                scope,
                resource,
                grant,
                effect,
            } => {
                self.ensure_effect(scope, effect)?;
                let state = self
                    .resources
                    .get(&resource)
                    .ok_or(ModelError::UnknownResource)?;
                if state.scope != scope {
                    return Err(ModelError::WrongScope);
                }
                if !state.live {
                    return Err(ModelError::ReleasedResource);
                }
                if state.effect != effect {
                    return Err(ModelError::GrantMismatch);
                }
                self.grant_matches(grant, scope, effect)?;
                Ok(Receipt::Unit)
            }
            Command::Release { scope, resource } => {
                self.open(scope)?;
                let state = self
                    .resources
                    .get_mut(&resource)
                    .ok_or(ModelError::UnknownResource)?;
                if state.scope != scope {
                    return Err(ModelError::WrongScope);
                }
                if !state.live {
                    return Err(ModelError::ReleasedResource);
                }
                state.live = false;
                Ok(Receipt::Unit)
            }
            Command::DeclareTask { scope } => {
                let current = self.open(scope)?;
                if current.tasks.len() >= current.budget.tasks {
                    return Err(ModelError::Exhausted);
                }
                let id = TaskId(self.domain, self.next_task);
                self.next_task = self.next_task.checked_add(1).ok_or(ModelError::Exhausted)?;
                self.tasks.insert(
                    id,
                    TaskState {
                        scope,
                        phase: TaskPhase::Declared,
                        cancel_requested: false,
                        result: None,
                    },
                );
                self.scopes
                    .get_mut(&scope)
                    .ok_or(ModelError::UnknownScope)?
                    .tasks
                    .insert(id);
                Ok(Receipt::Task(id))
            }
            Command::Admit { task } => {
                let scope = self.tasks.get(&task).ok_or(ModelError::UnknownTask)?.scope;
                self.open(scope)?;
                self.transition(task, TaskPhase::Declared, TaskPhase::Admitted)?;
                Ok(Receipt::Unit)
            }
            Command::Start { task } => {
                let scope = self.tasks.get(&task).ok_or(ModelError::UnknownTask)?.scope;
                self.open(scope)?;
                if self
                    .tasks
                    .get(&task)
                    .ok_or(ModelError::UnknownTask)?
                    .cancel_requested
                {
                    return Err(ModelError::InvalidTransition);
                }
                self.transition(task, TaskPhase::Admitted, TaskPhase::Running)?;
                Ok(Receipt::Unit)
            }
            Command::RequestCancel { task } => {
                let state = self.tasks.get(&task).ok_or(ModelError::UnknownTask)?;
                self.open(state.scope)?;
                if !matches!(state.phase, TaskPhase::Admitted | TaskPhase::Running)
                    || state.cancel_requested
                {
                    return Err(ModelError::InvalidTransition);
                }
                self.tasks
                    .get_mut(&task)
                    .ok_or(ModelError::UnknownTask)?
                    .cancel_requested = true;
                Ok(Receipt::Unit)
            }
            Command::Succeed { task } => {
                self.terminate(task, TaskOutcome::Succeeded)?;
                Ok(Receipt::Unit)
            }
            Command::Fail { task, code } => {
                self.terminate(task, TaskOutcome::Failed(code))?;
                Ok(Receipt::Unit)
            }
            Command::AcknowledgeCancel { task } => {
                if !self
                    .tasks
                    .get(&task)
                    .ok_or(ModelError::UnknownTask)?
                    .cancel_requested
                {
                    return Err(ModelError::InvalidTransition);
                }
                self.terminate(task, TaskOutcome::Cancelled)?;
                Ok(Receipt::Unit)
            }
            Command::Join { task } => {
                let scope = self.tasks.get(&task).ok_or(ModelError::UnknownTask)?.scope;
                self.open(scope)?;
                self.transition(task, TaskPhase::Terminal, TaskPhase::Joined)?;
                Ok(Receipt::Unit)
            }
            Command::Close { scope } => {
                let state = self.open(scope)?;
                if state.children.iter().any(|id| {
                    self.scopes
                        .get(id)
                        .is_some_and(|child| child.closed.is_none())
                }) {
                    return Err(ModelError::OutstandingChildren);
                }
                if state
                    .resources
                    .iter()
                    .any(|id| self.resources.get(id).is_some_and(|resource| resource.live))
                {
                    return Err(ModelError::OutstandingResources);
                }
                if state.tasks.iter().any(|id| {
                    self.tasks
                        .get(id)
                        .is_some_and(|task| task.phase != TaskPhase::Joined)
                }) {
                    return Err(ModelError::OutstandingTasks);
                }
                let tasks: Vec<_> = state
                    .tasks
                    .iter()
                    .map(|id| {
                        (
                            *id,
                            self.tasks[id]
                                .result
                                .expect("joined tasks must have outcomes"),
                        )
                    })
                    .collect();
                let children: Vec<_> = state
                    .children
                    .iter()
                    .map(|id| {
                        (
                            *id,
                            self.scopes[id]
                                .closed
                                .as_ref()
                                .expect("closed child must have report")
                                .outcome,
                        )
                    })
                    .collect();
                let failed = tasks
                    .iter()
                    .any(|(_, outcome)| matches!(outcome, TaskOutcome::Failed(_)))
                    || children
                        .iter()
                        .any(|(_, outcome)| *outcome == ScopeOutcome::Failed);
                let cancelled = tasks
                    .iter()
                    .any(|(_, outcome)| *outcome == TaskOutcome::Cancelled)
                    || children
                        .iter()
                        .any(|(_, outcome)| *outcome == ScopeOutcome::Cancelled);
                let outcome = if failed {
                    ScopeOutcome::Failed
                } else if cancelled {
                    ScopeOutcome::Cancelled
                } else {
                    ScopeOutcome::Succeeded
                };
                let report = ScopeReport {
                    scope,
                    outcome,
                    tasks,
                    children,
                };
                self.scopes
                    .get_mut(&scope)
                    .ok_or(ModelError::UnknownScope)?
                    .closed = Some(report.clone());
                Ok(Receipt::Closed(report))
            }
        }
    }

    fn transition(
        &mut self,
        task: TaskId,
        from: TaskPhase,
        to: TaskPhase,
    ) -> Result<(), ModelError> {
        let state = self.tasks.get_mut(&task).ok_or(ModelError::UnknownTask)?;
        if state.phase != from {
            return Err(ModelError::InvalidTransition);
        }
        state.phase = to;
        Ok(())
    }

    fn terminate(&mut self, task: TaskId, result: TaskOutcome) -> Result<(), ModelError> {
        let state = self.tasks.get(&task).ok_or(ModelError::UnknownTask)?;
        self.open(state.scope)?;
        if !matches!(state.phase, TaskPhase::Admitted | TaskPhase::Running) {
            return Err(ModelError::InvalidTransition);
        }
        if state.cancel_requested && result == TaskOutcome::Succeeded {
            return Err(ModelError::InvalidTransition);
        }
        if !state.cancel_requested && result == TaskOutcome::Cancelled {
            return Err(ModelError::InvalidTransition);
        }
        let state = self.tasks.get_mut(&task).ok_or(ModelError::UnknownTask)?;
        state.phase = TaskPhase::Terminal;
        state.result = Some(result);
        Ok(())
    }
}
