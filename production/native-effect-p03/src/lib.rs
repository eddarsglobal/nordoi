//! NORDOI Production Track P0.3: opt-in, fail-closed adapter implementing the actual
//! `nordoi_kernel::EffectBackend` trait. Does NOT replace/bypass-proof all legacy backends.
//! A signed admission is fsynced *before* downstream backend execution; failure or crash
//! leaves an in-doubt consumed sequence, NOT an exactly-once external effect guarantee.
//! Use only for test/research until kernel-wide routing, rollback anchoring and audits exist.

use nordoi_kernel::{
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDispatchRequest,
    QueuedEffectIntent,
};
use nordoi_native_trust_p01::{durable::DurableTrustGate, SignedEnvelope};

const DOMAIN: &[u8] = b"NORDOI:P0.3:KERNEL-EFFECT-BINDING:V1\0";
const CAPABILITY: &str = "nordoi/effect/execute";
const RESOURCE: &str = "nordoi/dispatch/v1";
const MAX_TEXT_BYTES: usize = 256;

/// A host-supplied, pre-authorized signature source; must not give signing
/// keys to untrusted programs or models. A missing envelope always denies.
pub trait SignedEnvelopeSource {
    fn take(&mut self, request: &EffectDispatchRequest) -> Option<SignedEnvelope>;
}

fn put_text(out: &mut Vec<u8>, text: &str) -> Result<(), &'static str> {
    if text.is_empty() || text.len() > MAX_TEXT_BYTES {
        return Err("invalid bounded text");
    }
    let size = u16::try_from(text.len()).map_err(|_| "text overflow")?;
    out.extend_from_slice(&size.to_be_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

/// Canonical binary commitment to a *specific* native kernel delivery, not to
/// an ad-hoc provider string. No Debug formatting, JSON or implicit defaults.
/// A missing/zero/mismatched durable delivery context is never accepted.
pub fn canonical_dispatch_bytes(request: &EffectDispatchRequest) -> Result<Vec<u8>, &'static str> {
    let key = request.delivery_key.ok_or("delivery key required")?;
    let fence = request.delivery_fence.ok_or("delivery fence required")?;
    let queued = &request.queued;
    if key.intent != queued.id
        || key.namespace.bytes() == [0; 16]
        || queued.id.value() == 0
        || queued.cycle == 0
        || fence.value() == 0
    {
        return Err("invalid delivery context");
    }
    let mut out = Vec::with_capacity(512);
    out.extend_from_slice(DOMAIN);
    out.extend_from_slice(&key.namespace.bytes());
    out.extend_from_slice(&key.intent.value().to_be_bytes());
    out.extend_from_slice(&fence.value().to_be_bytes());
    out.extend_from_slice(&queued.cycle.to_be_bytes());
    out.extend_from_slice(&queued.ordinal.to_be_bytes());
    out.extend_from_slice(&queued.intent.reaction.0.to_be_bytes());
    put_text(&mut out, &queued.intent.action_name)?;
    let (tag, scope): (u8, Option<&str>) = match &queued.intent.effect {
        Effect::Network(scope) => (1, Some(scope)),
        Effect::FileRead(scope) => (2, Some(scope)),
        Effect::FileWrite(scope) => (3, Some(scope)),
        Effect::Camera => (4, None),
        Effect::Microphone => (5, None),
        Effect::Location => (6, None),
        Effect::Gpu => (7, None),
        Effect::Xr => (8, None),
        Effect::Process => (9, None),
        Effect::Pure | Effect::StateRead | Effect::StateWrite => {
            return Err("internal effect is not externally dispatchable");
        }
    };
    out.push(tag);
    if let Some(scope) = scope {
        put_text(&mut out, scope)?;
    } else {
        out.extend_from_slice(&0_u16.to_be_bytes());
    }
    Ok(out)
}

/// Sealed, owned backend adapter. The inner backend is not exposed mutably.
/// Hosts must install this as the *only* external backend; that integration
/// mandate is outside this candidate's scope and NOT certified here.
pub struct GuardedEffectBackend<B, S> {
    backend: B,
    trust: DurableTrustGate,
    source: S,
}

impl<B, S> GuardedEffectBackend<B, S> {
    pub fn new(backend: B, trust: DurableTrustGate, source: S) -> Self {
        Self {
            backend,
            trust,
            source,
        }
    }

    pub fn root_status(&self, root_id: u64) -> Option<(u64, u64, bool)> {
        self.trust.root_status(root_id)
    }
}

impl<B: EffectBackend, S: SignedEnvelopeSource> EffectBackend for GuardedEffectBackend<B, S> {
    fn supports(&self, effect: &Effect) -> bool {
        !matches!(
            effect,
            Effect::Pure | Effect::StateRead | Effect::StateWrite
        ) && self.backend.supports(effect)
    }

    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Err(EffectBackendError::permanent(
            "P0.3 denies legacy dispatch without delivery key and fencing token",
        ))
    }

    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        if !self.supports(&request.queued.intent.effect) {
            return Err(EffectBackendError::permanent("P0.3 backend unsupported"));
        }
        let binding = canonical_dispatch_bytes(request)
            .map_err(|_| EffectBackendError::permanent("P0.3 invalid delivery context"))?;
        let envelope = self
            .source
            .take(request)
            .ok_or_else(|| EffectBackendError::permanent("P0.3 missing signed authorization"))?;
        // P0.2 commits the signed operation to disk before a side effect is attempted.
        // On subsequent backend failure do NOT roll back or reissue the signed sequence.
        self.trust
            .verify_execution(&envelope, CAPABILITY, RESOURCE, &binding)
            .map_err(|_| EffectBackendError::permanent("P0.3 signature/admission rejected"))?;
        self.backend.execute_with_context(request)
    }
}
