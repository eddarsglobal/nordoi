use crate::{
    authority::EffectGuard,
    capability::CapabilitySet,
    effect::{Effect, EffectSet},
    error::AtomicResult,
};

#[derive(Debug, Clone)]
pub struct ActionSpec {
    name: String,
    effects: EffectSet,
}

impl ActionSpec {
    pub fn pure(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            effects: EffectSet::pure(),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            effects: EffectSet::new(),
        }
    }

    pub fn declare(mut self, effect: Effect) -> Self {
        self.effects.declare(effect);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn effects(&self) -> &EffectSet {
        &self.effects
    }

    pub fn guard(&self, authority: CapabilitySet) -> EffectGuard {
        EffectGuard::new(self.effects.clone(), authority)
    }

    pub fn validate_effect(
        &self,
        authority: CapabilitySet,
        effect: &Effect,
    ) -> AtomicResult<()> {
        self.guard(authority).check(effect)
    }
}
