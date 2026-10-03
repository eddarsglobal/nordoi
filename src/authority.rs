use crate::{
    capability::{Capability, CapabilitySet},
    effect::{Effect, EffectSet},
    error::{AtomicError, AtomicResult},
};

#[derive(Debug, Clone)]
pub struct EffectGuard {
    declared: EffectSet,
    authority: CapabilitySet,
}

impl EffectGuard {
    pub fn new(declared: EffectSet, authority: CapabilitySet) -> Self {
        Self {
            declared,
            authority,
        }
    }

    pub fn declared(&self) -> &EffectSet {
        &self.declared
    }

    pub fn check(&self, effect: &Effect) -> AtomicResult<()> {
        if !self.declared.contains(effect) {
            return Err(AtomicError::UndeclaredEffect(effect.clone()));
        }

        match required_capability(effect) {
            None => Ok(()),
            Some(capability) => {
                if self.authority.contains(&capability) {
                    Ok(())
                } else {
                    Err(AtomicError::CapabilityDenied(capability))
                }
            }
        }
    }
}

pub fn required_capability(effect: &Effect) -> Option<Capability> {
    match effect {
        Effect::Pure | Effect::StateRead | Effect::StateWrite => None,

        Effect::Network(scope) => Some(Capability::Network(scope.clone())),
        Effect::FileRead(scope) => Some(Capability::FileRead(scope.clone())),
        Effect::FileWrite(scope) => Some(Capability::FileWrite(scope.clone())),

        Effect::Camera => Some(Capability::Camera),
        Effect::Microphone => Some(Capability::Microphone),
        Effect::Location => Some(Capability::Location),
        Effect::Gpu => Some(Capability::Gpu),
        Effect::Xr => Some(Capability::Xr),
        Effect::Process => Some(Capability::Process),
    }
}
