use crate::{
    authority::required_capability,
    capability::{Capability, CapabilitySet},
    effect::Effect,
};

use super::{
    AtomicEffectOutbox, EffectBackendError, EffectDeliveryKey, EffectDispatchError,
    EffectDispatchResult, QueuedEffectIntent,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDispatchRequest {
    pub queued: QueuedEffectIntent,
    pub delivery_key: Option<EffectDeliveryKey>,
}

pub trait EffectBackend {
    fn supports(&self, effect: &Effect) -> bool;

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError>;

    /// K1.7 retry-aware execution surface.
    ///
    /// Existing K1.6 backends remain source-compatible through this default adapter.
    /// Backends that can provide destination-level idempotency may override this method
    /// and forward `delivery_key` as the destination's client request/idempotency token.
    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.execute(&request.queued)
    }
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
    pub delivery_key: Option<EffectDeliveryKey>,
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
        self.dispatch_next_with_delivery_key(outbox, backend, None)
    }

    pub(crate) fn dispatch_next_with_delivery_key<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        backend: &mut B,
        delivery_key: Option<EffectDeliveryKey>,
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

        let dispatch_request = EffectDispatchRequest {
            queued: request.clone(),
            delivery_key,
        };
        let backend_receipt = backend
            .execute_with_context(&dispatch_request)
            .map_err(|error| EffectDispatchError::BackendFailed {
                intent: request.id,
                error,
            })?;

        let acknowledged = outbox.acknowledge(request.id)?;
        debug_assert_eq!(acknowledged, request);

        Ok(Some(EffectDispatchReceipt {
            request,
            delivery_key,
            backend: backend_receipt,
        }))
    }
}
