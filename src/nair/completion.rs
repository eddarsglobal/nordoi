use std::collections::BTreeMap;

use crate::{
    effect_completion::{
        AtomicEffectCompletionCore, EffectCompletionProjection, EffectCompletionProjectionValue,
        EffectCompletionSourceId,
    },
    effect_dispatch::EffectDeliveryNamespace,
    kernel::AtomicKernel,
    ownership::DomainId,
};

use super::{
    AtomSlot, CompletionSlot, DomainRef, DomainSlot, Instruction, NairError, NairProgram,
    NairResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NairCompletionProjectionValue {
    OutcomeValue,
    Succeeded,
    IntentId,
    AttemptId,
    SourceSequence,
}

impl From<NairCompletionProjectionValue> for EffectCompletionProjectionValue {
    fn from(value: NairCompletionProjectionValue) -> Self {
        match value {
            NairCompletionProjectionValue::OutcomeValue => Self::OutcomeValue,
            NairCompletionProjectionValue::Succeeded => Self::Succeeded,
            NairCompletionProjectionValue::IntentId => Self::IntentId,
            NairCompletionProjectionValue::AttemptId => Self::AttemptId,
            NairCompletionProjectionValue::SourceSequence => Self::SourceSequence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NairCompletionProjection {
    pub atom: AtomSlot,
    pub value: NairCompletionProjectionValue,
}

impl NairCompletionProjection {
    pub const fn new(atom: AtomSlot, value: NairCompletionProjectionValue) -> Self {
        Self { atom, value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NairCompletionBinding {
    pub source: EffectCompletionSourceId,
    pub namespace: EffectDeliveryNamespace,
}

impl NairCompletionBinding {
    pub const fn new(source: EffectCompletionSourceId, namespace: EffectDeliveryNamespace) -> Self {
        Self { source, namespace }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NairCompletionAuthority {
    by_slot: BTreeMap<CompletionSlot, NairCompletionBinding>,
}

impl NairCompletionAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, slot: CompletionSlot, binding: NairCompletionBinding) {
        self.by_slot.insert(slot, binding);
    }

    pub fn bind(
        &mut self,
        slot: CompletionSlot,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) {
        self.set(slot, NairCompletionBinding::new(source, namespace));
    }

    pub fn remove(&mut self, slot: CompletionSlot) -> Option<NairCompletionBinding> {
        self.by_slot.remove(&slot)
    }

    pub fn binding_for(&self, slot: CompletionSlot) -> Option<NairCompletionBinding> {
        self.by_slot.get(&slot).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.by_slot.is_empty()
    }
}

pub(crate) fn bootstrap_native_completions(
    program: &NairProgram,
    kernel: &AtomicKernel,
    domain_bindings: &BTreeMap<DomainSlot, DomainId>,
    atom_bindings: &BTreeMap<AtomSlot, crate::atom::AtomId>,
    authority: &NairCompletionAuthority,
) -> NairResult<(
    AtomicEffectCompletionCore,
    BTreeMap<CompletionSlot, NairCompletionBinding>,
)> {
    let mut core = AtomicEffectCompletionCore::new();
    let mut bindings = BTreeMap::new();

    for instruction in program.instructions() {
        let Instruction::DefineEffectCompletion {
            dst,
            domain,
            projections,
            ..
        } = instruction
        else {
            continue;
        };

        let binding = authority
            .binding_for(*dst)
            .ok_or(NairError::CompletionBindingRequired(*dst))?;
        if binding.source.0 == 0 {
            return Err(NairError::CompletionBindingRequired(*dst));
        }
        let domain = resolve_domain(kernel, *domain, domain_bindings)?;
        core.authority_mut()
            .grant(binding.source, binding.namespace)
            .map_err(NairError::Completion)?;

        for projection in projections {
            let atom = atom_bindings
                .get(&projection.atom)
                .copied()
                .ok_or(NairError::UnknownAtomSlot(projection.atom))?;
            core.register_projection(
                kernel,
                EffectCompletionProjection::new(
                    binding.source,
                    binding.namespace,
                    domain,
                    atom,
                    projection.value.into(),
                ),
            )
            .map_err(NairError::Completion)?;
        }

        bindings.insert(*dst, binding);
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
