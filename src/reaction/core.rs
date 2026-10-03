use std::collections::BTreeMap;

use crate::{
    action::ActionSpec, atom::AtomId, capability::CapabilitySet, effect::Effect, input::InputBatch,
    kernel::AtomicKernel, ownership::DomainId, time::TimerFire, value::Value,
};

use super::{
    ReactionError, ReactionId, ReactionResult, ReactionStep, ReactionTrigger, ReactionValue,
};

#[derive(Debug, Clone)]
pub struct ReactionSpec {
    name: String,
    domain: DomainId,
    trigger: ReactionTrigger,
    action: ActionSpec,
    authority: CapabilitySet,
    steps: Vec<ReactionStep>,
}

impl ReactionSpec {
    pub fn new(
        name: impl Into<String>,
        domain: DomainId,
        trigger: ReactionTrigger,
        action: ActionSpec,
    ) -> Self {
        Self {
            name: name.into(),
            domain,
            trigger,
            action,
            authority: CapabilitySet::new(),
            steps: Vec::new(),
        }
    }

    pub fn with_authority(mut self, authority: CapabilitySet) -> Self {
        self.authority = authority;
        self
    }

    pub fn step(mut self, step: ReactionStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn domain(&self) -> DomainId {
        self.domain
    }

    pub fn trigger(&self) -> ReactionTrigger {
        self.trigger
    }

    pub fn action(&self) -> &ActionSpec {
        &self.action
    }

    pub fn authority(&self) -> &CapabilitySet {
        &self.authority
    }

    pub fn steps(&self) -> &[ReactionStep] {
        &self.steps
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectIntent {
    pub reaction: ReactionId,
    pub action_name: String,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReactionBatchReport {
    pub causes: usize,
    pub matched_reactions: usize,
    pub transactions: usize,
    pub staged_writes: usize,
    pub changed_atoms: usize,
    pub effect_intents: Vec<EffectIntent>,
}

#[derive(Debug, Clone)]
pub struct AtomicReactionCore {
    reactions: BTreeMap<ReactionId, ReactionSpec>,
    next_reaction_id: u64,
}

impl Default for AtomicReactionCore {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicReactionCore {
    pub fn new() -> Self {
        Self {
            reactions: BTreeMap::new(),
            next_reaction_id: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.reactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reactions.is_empty()
    }

    pub fn reaction(&self, id: ReactionId) -> Option<&ReactionSpec> {
        self.reactions.get(&id)
    }

    pub fn register(
        &mut self,
        kernel: &AtomicKernel,
        spec: ReactionSpec,
    ) -> ReactionResult<ReactionId> {
        validate_spec(kernel, &spec)?;

        let id = ReactionId(self.next_reaction_id);
        self.next_reaction_id = self
            .next_reaction_id
            .checked_add(1)
            .ok_or(ReactionError::ReactionIdExhausted)?;
        self.reactions.insert(id, spec);
        Ok(id)
    }

    pub fn unregister(&mut self, id: ReactionId) -> bool {
        self.reactions.remove(&id).is_some()
    }

    pub fn apply_input_batch(
        &self,
        kernel: &mut AtomicKernel,
        batch: &InputBatch,
    ) -> ReactionResult<ReactionBatchReport> {
        let canonical = batch.canonicalized()?;
        let mut report = ReactionBatchReport {
            causes: canonical.len(),
            ..ReactionBatchReport::default()
        };
        let matched = canonical.events.iter().any(|event| {
            self.reactions.values().any(|spec| match spec.trigger {
                ReactionTrigger::Input(selector) => selector.extract(event).is_some(),
                ReactionTrigger::Timer(_) => false,
            })
        });
        if !matched {
            return Ok(report);
        }

        let mut candidate = kernel.clone();
        for event in &canonical.events {
            for (id, spec) in &self.reactions {
                let ReactionTrigger::Input(selector) = spec.trigger else {
                    continue;
                };
                let Some(input_value) = selector.extract(event) else {
                    continue;
                };

                apply_reaction(
                    &mut candidate,
                    *id,
                    spec,
                    Cause::Input(input_value),
                    &mut report,
                )?;
            }
        }

        *kernel = candidate;
        Ok(report)
    }

    pub fn apply_timer_fires(
        &self,
        kernel: &mut AtomicKernel,
        fires: &[TimerFire],
    ) -> ReactionResult<ReactionBatchReport> {
        let mut canonical = fires.to_vec();
        canonical.sort_by_key(|fire| (fire.deadline, fire.timer, fire.occurrence));

        let mut report = ReactionBatchReport {
            causes: canonical.len(),
            ..ReactionBatchReport::default()
        };
        let matched = canonical.iter().any(|fire| {
            self.reactions.values().any(|spec| match spec.trigger {
                ReactionTrigger::Timer(selector) => selector.matches(fire),
                ReactionTrigger::Input(_) => false,
            })
        });
        if !matched {
            return Ok(report);
        }

        let mut candidate = kernel.clone();
        for fire in &canonical {
            for (id, spec) in &self.reactions {
                let ReactionTrigger::Timer(selector) = spec.trigger else {
                    continue;
                };
                if !selector.matches(fire) {
                    continue;
                }

                apply_reaction(&mut candidate, *id, spec, Cause::Timer(fire), &mut report)?;
            }
        }

        *kernel = candidate;
        Ok(report)
    }
}

#[derive(Debug, Clone)]
enum Cause<'a> {
    Input(Value),
    Timer(&'a TimerFire),
}

fn validate_spec(kernel: &AtomicKernel, spec: &ReactionSpec) -> ReactionResult<()> {
    kernel.domain(spec.domain)?;
    if spec.steps.is_empty() {
        return Err(ReactionError::EmptyAction);
    }

    for step in &spec.steps {
        match step {
            ReactionStep::Set { atom, value } => {
                kernel.require_owner(*atom, spec.domain)?;
                spec.action
                    .validate_effect(spec.authority.clone(), &Effect::StateWrite)?;
                validate_value_source(spec.trigger, value)?;
            }
            ReactionStep::EmitEffect { effect } => {
                spec.action
                    .validate_effect(spec.authority.clone(), effect)?;
            }
        }
    }

    Ok(())
}

fn validate_value_source(trigger: ReactionTrigger, value: &ReactionValue) -> ReactionResult<()> {
    let compatible = matches!(value, ReactionValue::Literal(_))
        || matches!(
            (trigger, value),
            (ReactionTrigger::Input(_), ReactionValue::InputValue)
        )
        || matches!(
            (trigger, value),
            (
                ReactionTrigger::Timer(_),
                ReactionValue::TimerOccurrence | ReactionValue::TimerDeadlineTicks
            )
        );

    if compatible {
        Ok(())
    } else {
        Err(ReactionError::ValueSourceMismatch)
    }
}

fn apply_reaction(
    kernel: &mut AtomicKernel,
    id: ReactionId,
    spec: &ReactionSpec,
    cause: Cause<'_>,
    report: &mut ReactionBatchReport,
) -> ReactionResult<()> {
    let mut planned_writes: Vec<(AtomId, Value)> = Vec::new();
    let mut planned_intents = Vec::new();

    for step in &spec.steps {
        match step {
            ReactionStep::Set { atom, value } => {
                kernel.require_owner(*atom, spec.domain)?;
                spec.action
                    .validate_effect(spec.authority.clone(), &Effect::StateWrite)?;
                planned_writes.push((*atom, resolve_value(value, &cause)?));
            }
            ReactionStep::EmitEffect { effect } => {
                spec.action
                    .validate_effect(spec.authority.clone(), effect)?;
                planned_intents.push(EffectIntent {
                    reaction: id,
                    action_name: spec.action.name().to_owned(),
                    effect: effect.clone(),
                });
            }
        }
    }

    report.matched_reactions += 1;

    if !planned_writes.is_empty() {
        let mut tx = kernel.begin_transaction(spec.domain)?;
        for (atom, value) in planned_writes {
            tx.set(kernel, atom, value)?;
        }
        let tx_report = kernel.commit(tx)?;
        report.transactions += 1;
        report.staged_writes += tx_report.staged_writes;
        report.changed_atoms += tx_report.changed_atoms;
    }

    report.effect_intents.extend(planned_intents);
    Ok(())
}

fn resolve_value(value: &ReactionValue, cause: &Cause<'_>) -> ReactionResult<Value> {
    match (value, cause) {
        (ReactionValue::Literal(value), _) => Ok(value.clone()),
        (ReactionValue::InputValue, Cause::Input(value)) => Ok(value.clone()),
        (ReactionValue::TimerOccurrence, Cause::Timer(fire)) => project_u64(fire.occurrence),
        (ReactionValue::TimerDeadlineTicks, Cause::Timer(fire)) => {
            project_u64(fire.deadline.ticks())
        }
        _ => Err(ReactionError::ValueSourceMismatch),
    }
}

fn project_u64(value: u64) -> ReactionResult<Value> {
    let value =
        i64::try_from(value).map_err(|_| ReactionError::IntegerProjectionOverflow(value))?;
    Ok(Value::Int(value))
}
