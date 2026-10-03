use crate::{
    effect_dispatch::{EffectDeliveryFence, EffectDeliveryKey, EffectIntentId, QueuedEffectIntent},
    effect_retry::{EffectDeadLetterReason, EffectRetryTick},
};

use super::{EffectAttemptId, EffectAuditHash, EffectAuditSequence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInDoubtAttempt {
    pub attempt: EffectAttemptId,
    pub request: QueuedEffectIntent,
    pub delivery_key: EffectDeliveryKey,
    pub delivery_fence: EffectDeliveryFence,
    pub prepared_tick: EffectRetryTick,
    pub failed_attempts_before: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectAuditEvent {
    AttemptPrepared(EffectInDoubtAttempt),
    AttemptDelivered {
        attempt: EffectAttemptId,
        intent: EffectIntentId,
        backend_reference: Option<String>,
    },
    AttemptRetryScheduled {
        attempt: EffectAttemptId,
        intent: EffectIntentId,
        failed_attempts: u32,
        next_eligible_tick: EffectRetryTick,
        error: String,
    },
    AttemptDeadLettered {
        attempt: EffectAttemptId,
        intent: EffectIntentId,
        failed_attempts: u32,
        reason: EffectDeadLetterReason,
        error: String,
    },
    InDoubtAssumedDelivered {
        attempt: EffectAttemptId,
        intent: EffectIntentId,
        resolution_tick: EffectRetryTick,
        backend_reference: Option<String>,
    },
    InDoubtRetryAuthorized {
        attempt: EffectAttemptId,
        intent: EffectIntentId,
        resolution_tick: EffectRetryTick,
    },
    DeadLetterRedriven {
        intent: EffectIntentId,
    },
    DeadLetterDiscarded {
        intent: EffectIntentId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuditRecord {
    pub sequence: EffectAuditSequence,
    pub previous_hash: EffectAuditHash,
    pub event: EffectAuditEvent,
    pub hash: EffectAuditHash,
}
