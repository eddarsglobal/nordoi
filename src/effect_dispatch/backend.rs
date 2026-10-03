use crate::{
    authority::required_capability,
    capability::{Capability, CapabilitySet},
    effect::Effect,
};

use super::{
    AtomicEffectOutbox, EffectBackendError, EffectDispatchError, EffectDispatchResult,
    QueuedEffectIntent,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectBackendReceipt {
    pub reference: Option<String>,
}

impl EffectBackendReceipt {
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: Some(reference.into()),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

pub trait EffectBackend {
    fn supports(&self, effect: &Effect) -> bool;

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError>;
}

#[derive(Debug, Clone, Default)]
pub struct EffectDispatchAuthority {
    allowed: CapabilitySet,
}

impl EffectDispatchAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_capabilities(allowed: CapabilitySet) -> Self {
        Self { allowed }
    }

    pub fn grant(&mut self, capability: Capability) {
        self.allowed.allow(capability);
    }

    pub fn revoke(&mut self, capability: &Capability) {
        self.allowed.revoke(capability);
    }

    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }

    fn require(&self, effect: &Effect) -> EffectDispatchResult<()> {
        let Some(capability) = required_capability(effect) else {
            return Err(EffectDispatchError::InternalEffectNotDispatchable(
                effect.clone(),
            ));
        };

        if self.allowed.contains(&capability) {
            Ok(())
        } else {
            Err(EffectDispatchError::CapabilityDenied(capability))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDispatchReceipt {
    pub request: QueuedEffectIntent,
    pub backend: EffectBackendReceipt,
}

#[derive(Debug, Clone)]
pub struct GovernedEffectDispatcher {
    authority: EffectDispatchAuthority,
}

impl GovernedEffectDispatcher {
    pub fn new(authority: EffectDispatchAuthority) -> Self {
        Self { authority }
    }

    pub fn authority(&self) -> &EffectDispatchAuthority {
        &self.authority
    }

    pub fn authority_mut(&mut self) -> &mut EffectDispatchAuthority {
        &mut self.authority
    }

    pub fn dispatch_next<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        backend: &mut B,
    ) -> EffectDispatchResult<Option<EffectDispatchReceipt>> {
        let Some(request) = outbox.peek().cloned() else {
            return Ok(None);
        };

        self.authority.require(&request.intent.effect)?;

        if !backend.supports(&request.intent.effect) {
            return Err(EffectDispatchError::BackendUnsupported {
                intent: request.id,
                effect: request.intent.effect.clone(),
            });
        }

        let backend_receipt =
            backend
                .execute(&request)
                .map_err(|error| EffectDispatchError::BackendFailed {
                    intent: request.id,
                    error,
                })?;

        let acknowledged = outbox.acknowledge(request.id)?;
        debug_assert_eq!(acknowledged, request);

        Ok(Some(EffectDispatchReceipt {
            request,
            backend: backend_receipt,
        }))
    }
}
