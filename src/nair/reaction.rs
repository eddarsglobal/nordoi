use std::collections::{BTreeMap, BTreeSet};

use crate::{
    capability::CapabilitySet,
    effect::Effect,
    input::{InputDeviceId, InputSignal, InputSource},
    reaction::ReactionBatchReport,
    value::Value,
};

use super::{AtomSlot, InputTargetRef, ReactionSlot, TimerSlot};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NairEffectSet {
    effects: BTreeSet<Effect>,
}

impl NairEffectSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_effects(effects: impl IntoIterator<Item = Effect>) -> Self {
        Self {
            effects: effects.into_iter().collect(),
        }
    }

    pub fn declare(&mut self, effect: Effect) {
        self.effects.insert(effect);
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NairReactionTrigger {
    Input {
        source: Option<InputSource>,
        device: Option<InputDeviceId>,
        target: InputTargetRef,
        signal: InputSignal,
    },
    Timer {
        timer: Option<TimerSlot>,
        occurrence: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NairReactionValue {
    Literal(Value),
    InputValue,
    TimerOccurrence,
    TimerDeadlineTicks,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NairReactionStep {
    Set {
        atom: AtomSlot,
        value: NairReactionValue,
    },
    EmitEffect {
        effect: Effect,
    },
}

#[derive(Debug, Clone, Default)]
pub struct NairReactionAuthority {
    by_slot: BTreeMap<ReactionSlot, CapabilitySet>,
}

impl NairReactionAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, slot: ReactionSlot, authority: CapabilitySet) {
        self.by_slot.insert(slot, authority);
    }

    pub fn remove(&mut self, slot: ReactionSlot) -> Option<CapabilitySet> {
        self.by_slot.remove(&slot)
    }

    pub fn authority_for(&self, slot: ReactionSlot) -> CapabilitySet {
        self.by_slot.get(&slot).cloned().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.by_slot.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NairReactionCycleReport {
    pub input: ReactionBatchReport,
    pub timers: ReactionBatchReport,
}

impl NairReactionCycleReport {
    pub fn matched_reactions(&self) -> usize {
        self.input.matched_reactions + self.timers.matched_reactions
    }

    pub fn changed_atoms(&self) -> usize {
        self.input.changed_atoms + self.timers.changed_atoms
    }

    pub fn effect_intent_count(&self) -> usize {
        self.input.effect_intents.len() + self.timers.effect_intents.len()
    }
}

use crate::{
    action::ActionSpec,
    atom::AtomId,
    input::{InputSelector, InputTarget},
    kernel::AtomicKernel,
    ownership::DomainId,
    reaction::{
        AtomicReactionCore, ReactionId, ReactionSpec, ReactionStep, ReactionTrigger, ReactionValue,
        TimerSelector,
    },
    render::RenderNodeId,
    time::TimerId,
};

use super::{
    DomainRef, DomainSlot, Instruction, NairError, NairProgram, NairResult, RenderNodeSlot,
};

pub(crate) fn bootstrap_native_reactions(
    program: &NairProgram,
    kernel: &AtomicKernel,
    domain_bindings: &BTreeMap<DomainSlot, DomainId>,
    atom_bindings: &BTreeMap<AtomSlot, AtomId>,
    render_bindings: &BTreeMap<RenderNodeSlot, RenderNodeId>,
    timer_bindings: &BTreeMap<TimerSlot, TimerId>,
    authority: &NairReactionAuthority,
) -> NairResult<(AtomicReactionCore, BTreeMap<ReactionSlot, ReactionId>)> {
    let mut core = AtomicReactionCore::new();
    let mut bindings = BTreeMap::new();

    for instruction in program.instructions() {
        let Instruction::DefineReaction {
            dst,
            name,
            domain,
            trigger,
            action_name,
            declared_effects,
            steps,
        } = instruction
        else {
            continue;
        };

        let domain = resolve_domain(kernel, *domain, domain_bindings)?;
        let trigger = resolve_trigger(*trigger, render_bindings, timer_bindings)?;

        let mut action = ActionSpec::new(action_name.clone());
        for effect in declared_effects.iter() {
            action = action.declare(effect.clone());
        }

        let mut spec = ReactionSpec::new(name.clone(), domain, trigger, action)
            .with_authority(authority.authority_for(*dst));
        for step in steps {
            spec = spec.step(resolve_step(step, atom_bindings)?);
        }

        let id = core.register(kernel, spec)?;
        bindings.insert(*dst, id);
    }

    Ok((core, bindings))
}

fn resolve_domain(
    kernel: &AtomicKernel,
    domain: DomainRef,
    domains: &BTreeMap<DomainSlot, DomainId>,
) -> NairResult<DomainId> {
    match domain {
        DomainRef::Root => Ok(kernel.root_domain()),
        DomainRef::Slot(slot) => domains
            .get(&slot)
            .copied()
            .ok_or(NairError::UnknownDomainSlot(slot)),
    }
}

fn resolve_trigger(
    trigger: NairReactionTrigger,
    render_nodes: &BTreeMap<RenderNodeSlot, RenderNodeId>,
    timers: &BTreeMap<TimerSlot, TimerId>,
) -> NairResult<ReactionTrigger> {
    match trigger {
        NairReactionTrigger::Input {
            source,
            device,
            target,
            signal,
        } => Ok(ReactionTrigger::Input(InputSelector {
            source,
            device,
            target: resolve_target(target, render_nodes)?,
            signal,
        })),
        NairReactionTrigger::Timer { timer, occurrence } => {
            let timer = timer
                .map(|slot| {
                    timers
                        .get(&slot)
                        .copied()
                        .ok_or(NairError::UnknownTimerSlot(slot))
                })
                .transpose()?;
            Ok(ReactionTrigger::Timer(TimerSelector { timer, occurrence }))
        }
    }
}

fn resolve_target(
    target: InputTargetRef,
    render_nodes: &BTreeMap<RenderNodeSlot, RenderNodeId>,
) -> NairResult<Option<InputTarget>> {
    match target {
        InputTargetRef::Any => Ok(None),
        InputTargetRef::Global => Ok(Some(InputTarget::Global)),
        InputTargetRef::RenderNode(slot) => render_nodes
            .get(&slot)
            .copied()
            .map(InputTarget::RenderNode)
            .map(Some)
            .ok_or(NairError::UnknownRenderNodeSlot(slot)),
    }
}

fn resolve_step(
    step: &NairReactionStep,
    atoms: &BTreeMap<AtomSlot, AtomId>,
) -> NairResult<ReactionStep> {
    match step {
        NairReactionStep::Set { atom, value } => {
            let atom = atoms
                .get(atom)
                .copied()
                .ok_or(NairError::UnknownAtomSlot(*atom))?;
            Ok(ReactionStep::Set {
                atom,
                value: match value {
                    NairReactionValue::Literal(value) => ReactionValue::Literal(value.clone()),
                    NairReactionValue::InputValue => ReactionValue::InputValue,
                    NairReactionValue::TimerOccurrence => ReactionValue::TimerOccurrence,
                    NairReactionValue::TimerDeadlineTicks => ReactionValue::TimerDeadlineTicks,
                },
            })
        }
        NairReactionStep::EmitEffect { effect } => Ok(ReactionStep::EmitEffect {
            effect: effect.clone(),
        }),
    }
}
