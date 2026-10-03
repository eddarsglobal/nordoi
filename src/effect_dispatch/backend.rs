use crate::{
    authority::required_capability,
    capability::{Capability, CapabilitySet},
    effect::Effect,
};

use super::{
    AtomicEffectOutbox, EffectBackendError, EffectDeliveryFence, EffectDeliveryKey,
    EffectDispatchError, EffectDispatchResult, QueuedEffectIntent,
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
    /// Optional K1.8 fencing epoch. Fence-aware destinations may reject stale writers.
    /// Legacy backends may ignore this field through the default adapter.
    pub delivery_fence: Option<EffectDeliveryFence>,
}

pub trait EffectBackend {
    fn supports(&self, effect: &Effect) -> bool;

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError>;

    /// K1.7 retry-aware / K1.8 fence-aware execution surface.
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

    pub(crate) fn require(&self, effect: &Effect) -> EffectDispatchResult<()> {
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
    pub delivery_fence: Option<EffectDeliveryFence>,
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
        self.dispatch_next_with_delivery_context(outbox, backend, None, None)
    }

    pub(crate) fn dispatch_next_with_delivery_key<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        backend: &mut B,
        delivery_key: Option<EffectDeliveryKey>,
    ) -> EffectDispatchResult<Option<EffectDispatchReceipt>> {
        self.dispatch_next_with_delivery_context(outbox, backend, delivery_key, None)
    }

    pub(crate) fn dispatch_next_with_delivery_context<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        backend: &mut B,
        delivery_key: Option<EffectDeliveryKey>,
        delivery_fence: Option<EffectDeliveryFence>,
    ) -> EffectDispatchResult<Option<EffectDispatchReceipt>> {
        let Some(request) = outbox.peek().cloned() else {
            return Ok(None);
        };
        self.dispatch_intent_with_delivery_context(
            outbox,
            request.id,
            backend,
            delivery_key,
            delivery_fence,
        )
        .map(Some)
    }

    pub(crate) fn preflight_intent<B: EffectBackend>(
        &self,
        outbox: &AtomicEffectOutbox,
        intent: super::EffectIntentId,
        backend: &B,
    ) -> EffectDispatchResult<QueuedEffectIntent> {
        let request = outbox
            .get(intent)
            .cloned()
            .ok_or(EffectDispatchError::UnknownIntent(intent))?;
        self.authority.require(&request.intent.effect)?;
        if !backend.supports(&request.intent.effect) {
            return Err(EffectDispatchError::BackendUnsupported {
                intent: request.id,
                effect: request.intent.effect.clone(),
            });
        }
        Ok(request)
    }

    pub(crate) fn execute_preflighted_intent_with_delivery_context<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        request: QueuedEffectIntent,
        backend: &mut B,
        delivery_key: Option<EffectDeliveryKey>,
        delivery_fence: Option<EffectDeliveryFence>,
    ) -> EffectDispatchResult<EffectDispatchReceipt> {
        let dispatch_request = EffectDispatchRequest {
            queued: request.clone(),
            delivery_key,
            delivery_fence,
        };
        let backend_receipt = backend
            .execute_with_context(&dispatch_request)
            .map_err(|error| EffectDispatchError::BackendFailed {
                intent: request.id,
                error,
            })?;
        let acknowledged = outbox.acknowledge(request.id)?;
        debug_assert_eq!(acknowledged, request);
        Ok(EffectDispatchReceipt {
            request,
            delivery_key,
            delivery_fence,
            backend: backend_receipt,
        })
    }

    pub(crate) fn dispatch_intent_with_delivery_context<B: EffectBackend>(
        &self,
        outbox: &mut AtomicEffectOutbox,
        intent: super::EffectIntentId,
        backend: &mut B,
        delivery_key: Option<EffectDeliveryKey>,
        delivery_fence: Option<EffectDeliveryFence>,
    ) -> EffectDispatchResult<EffectDispatchReceipt> {
        let request = self.preflight_intent(outbox, intent, backend)?;
        self.execute_preflighted_intent_with_delivery_context(
            outbox,
            request,
            backend,
            delivery_key,
            delivery_fence,
        )
    }
}
