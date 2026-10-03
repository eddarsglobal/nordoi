use crate::{
    effect::Effect,
    effect_dispatch::{
        EffectDeliveryFence, EffectDeliveryKey, EffectDeliveryNamespace, EffectIntentId,
        QueuedEffectIntent,
    },
    effect_persistence::MAX_EFFECT_JOURNAL_STRING_BYTES,
    effect_retry::{
        EffectDeadLetterReason, EffectRetryCheckpoint, EffectRetryPolicy, EffectRetryTick,
        MAX_EFFECT_RETRY_CHECKPOINT_BYTES,
    },
    reaction::{EffectIntent, ReactionId},
};

use super::{
    hash::sha256, EffectAttemptId, EffectAuditError, EffectAuditEvent, EffectAuditHash,
    EffectAuditLedger, EffectAuditRecord, EffectAuditResult, EffectAuditSequence,
    EffectInDoubtAttempt,
};

const MAGIC: &[u8; 8] = b"NDEFXA01";
const LEGACY_RETRY_MAGIC: &[u8; 8] = b"NDEFXR01";
const LEGACY_OUTBOX_MAGIC: &[u8; 8] = b"NDEFXJ01";
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 0;
const CHECKPOINT_HASH_DOMAIN: &[u8] = b"NORDOI-EFFECT-AUDIT-CHECKPOINT-1.0";

pub const MAX_EFFECT_AUDIT_CHECKPOINT_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_EFFECT_AUDIT_EVENTS: usize = 262_144;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAuditCheckpoint {
    namespace: EffectDeliveryNamespace,
    retry: EffectRetryCheckpoint,
    audit: EffectAuditLedger,
}

impl EffectAuditCheckpoint {
    pub fn capture(
        namespace: EffectDeliveryNamespace,
        policy: EffectRetryPolicy,
        outbox: &crate::effect_dispatch::AtomicEffectOutbox,
        retry_ledger: &crate::effect_retry::EffectRetryLedger,
        audit: &EffectAuditLedger,
    ) -> Self {
        Self {
            namespace,
            retry: EffectRetryCheckpoint::capture(namespace, policy, outbox, retry_ledger),
            audit: audit.clone(),
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
    }
    pub fn retry(&self) -> &EffectRetryCheckpoint {
        &self.retry
    }
    pub fn audit(&self) -> &EffectAuditLedger {
        &self.audit
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let retry_bytes = self.retry.canonical_bytes();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_MAJOR.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        bytes.extend_from_slice(&self.namespace.0);
        bytes.extend_from_slice(&(retry_bytes.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&retry_bytes);
        bytes.extend_from_slice(&self.audit.next_attempt_id().to_le_bytes());
        bytes.extend_from_slice(&(self.audit.len() as u64).to_le_bytes());
        for record in self.audit.records() {
            bytes.extend_from_slice(&record.sequence.0.to_le_bytes());
            bytes.extend_from_slice(&record.previous_hash.0);
            push_event(&mut bytes, &record.event);
            bytes.extend_from_slice(&record.hash.0);
        }
        let digest = checkpoint_digest(&bytes);
        bytes.extend_from_slice(&digest.0);
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> EffectAuditResult<Self> {
        if bytes.len() > MAX_EFFECT_AUDIT_CHECKPOINT_BYTES {
            return Err(EffectAuditError::AuditCheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_AUDIT_CHECKPOINT_BYTES,
            });
        }

        if bytes.starts_with(LEGACY_RETRY_MAGIC) || bytes.starts_with(LEGACY_OUTBOX_MAGIC) {
            let retry = EffectRetryCheckpoint::from_canonical_bytes(bytes)?;
            return Ok(Self {
                namespace: retry.namespace(),
                retry,
                audit: EffectAuditLedger::new(),
            });
        }

        if bytes.len() < 32 {
            return Err(EffectAuditError::AuditCheckpointTruncated);
        }
        let (payload, digest_bytes) = bytes.split_at(bytes.len() - 32);
        let expected = EffectAuditHash(
            digest_bytes
                .try_into()
                .map_err(|_| EffectAuditError::AuditCheckpointTruncated)?,
        );
        let actual = checkpoint_digest(payload);
        if expected != actual {
            return Err(EffectAuditError::AuditCheckpointDigestMismatch { expected, actual });
        }

        let mut reader = Reader::new(payload);
        if reader.read_exact(8)? != MAGIC {
            return Err(EffectAuditError::InvalidAuditCheckpointMagic);
        }
        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != FORMAT_MAJOR || minor > FORMAT_MINOR {
            return Err(EffectAuditError::UnsupportedAuditCheckpointVersion { major, minor });
        }
        let namespace = EffectDeliveryNamespace::new(reader.read_array_16()?);
        let retry_len_u64 = reader.read_u64()?;
        if retry_len_u64 > MAX_EFFECT_RETRY_CHECKPOINT_BYTES as u64 {
            return Err(EffectAuditError::AuditCheckpointTooLarge {
                bytes: usize::try_from(retry_len_u64).unwrap_or(usize::MAX),
                limit: MAX_EFFECT_RETRY_CHECKPOINT_BYTES,
            });
        }
        let retry_len = usize::try_from(retry_len_u64)
            .map_err(|_| EffectAuditError::AuditCheckpointTruncated)?;
        let retry = EffectRetryCheckpoint::from_canonical_bytes(reader.read_exact(retry_len)?)?;
        if retry.namespace() != namespace {
            return Err(EffectAuditError::AuditNamespaceMismatch);
        }

        let next_attempt_id = reader.read_u64()?;
        let event_count_u64 = reader.read_u64()?;
        if event_count_u64 > MAX_EFFECT_AUDIT_EVENTS as u64 {
            return Err(EffectAuditError::AuditEventLimitExceeded {
                events: event_count_u64,
                limit: MAX_EFFECT_AUDIT_EVENTS,
            });
        }
        let event_count = usize::try_from(event_count_u64)
            .map_err(|_| EffectAuditError::AuditCheckpointTruncated)?;
        let mut records = Vec::with_capacity(event_count);
        for _ in 0..event_count {
            let sequence = EffectAuditSequence(reader.read_u64()?);
            let previous_hash = EffectAuditHash(reader.read_array_32()?);
            let event = reader.read_event(namespace)?;
            let hash = EffectAuditHash(reader.read_array_32()?);
            records.push(EffectAuditRecord {
                sequence,
                previous_hash,
                event,
                hash,
            });
        }
        if !reader.is_finished() {
            return Err(EffectAuditError::AuditCheckpointTruncated);
        }
        let audit = EffectAuditLedger::from_records(next_attempt_id, records)?;
        if let Some(open) = audit.in_doubt() {
            if retry
                .outbox()
                .pending()
                .iter()
                .all(|request| request.id != open.request.id)
            {
                return Err(EffectAuditError::InDoubtIntentNotPending(open.request.id));
            }
        }
        Ok(Self {
            namespace,
            retry,
            audit,
        })
    }
}

fn checkpoint_digest(payload: &[u8]) -> EffectAuditHash {
    let mut input = Vec::with_capacity(CHECKPOINT_HASH_DOMAIN.len() + payload.len());
    input.extend_from_slice(CHECKPOINT_HASH_DOMAIN);
    input.extend_from_slice(payload);
    EffectAuditHash(sha256(&input))
}

pub(crate) fn push_event(bytes: &mut Vec<u8>, event: &EffectAuditEvent) {
    match event {
        EffectAuditEvent::AttemptPrepared(value) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&value.attempt.0.to_le_bytes());
            push_queued(bytes, &value.request);
            bytes.extend_from_slice(&value.delivery_key.namespace.0);
            bytes.extend_from_slice(&value.delivery_key.intent.0.to_le_bytes());
            bytes.extend_from_slice(&value.delivery_fence.0.to_le_bytes());
            bytes.extend_from_slice(&value.prepared_tick.0.to_le_bytes());
            bytes.extend_from_slice(&u64::from(value.failed_attempts_before).to_le_bytes());
        }
        EffectAuditEvent::AttemptDelivered {
            attempt,
            intent,
            backend_reference,
        } => {
            bytes.push(0x02);
            bytes.extend_from_slice(&attempt.0.to_le_bytes());
            bytes.extend_from_slice(&intent.0.to_le_bytes());
            push_option_string(bytes, backend_reference.as_deref());
        }
        EffectAuditEvent::AttemptRetryScheduled {
            attempt,
            intent,
            failed_attempts,
            next_eligible_tick,
            error,
        } => {
            bytes.push(0x03);
            bytes.extend_from_slice(&attempt.0.to_le_bytes());
            bytes.extend_from_slice(&intent.0.to_le_bytes());
            bytes.extend_from_slice(&u64::from(*failed_attempts).to_le_bytes());
            bytes.extend_from_slice(&next_eligible_tick.0.to_le_bytes());
            push_string(bytes, error);
        }
        EffectAuditEvent::AttemptDeadLettered {
            attempt,
            intent,
            failed_attempts,
            reason,
            error,
        } => {
            bytes.push(0x04);
            bytes.extend_from_slice(&attempt.0.to_le_bytes());
            bytes.extend_from_slice(&intent.0.to_le_bytes());
            bytes.extend_from_slice(&u64::from(*failed_attempts).to_le_bytes());
            bytes.push(dead_reason_tag(*reason));
            push_string(bytes, error);
        }
        EffectAuditEvent::InDoubtAssumedDelivered {
            attempt,
            intent,
            resolution_tick,
            backend_reference,
        } => {
            bytes.push(0x05);
            bytes.extend_from_slice(&attempt.0.to_le_bytes());
            bytes.extend_from_slice(&intent.0.to_le_bytes());
            bytes.extend_from_slice(&resolution_tick.0.to_le_bytes());
            push_option_string(bytes, backend_reference.as_deref());
        }
        EffectAuditEvent::InDoubtRetryAuthorized {
            attempt,
            intent,
            resolution_tick,
        } => {
            bytes.push(0x06);
            bytes.extend_from_slice(&attempt.0.to_le_bytes());
            bytes.extend_from_slice(&intent.0.to_le_bytes());
            bytes.extend_from_slice(&resolution_tick.0.to_le_bytes());
        }
        EffectAuditEvent::DeadLetterRedriven { intent } => {
            bytes.push(0x07);
            bytes.extend_from_slice(&intent.0.to_le_bytes());
        }
        EffectAuditEvent::DeadLetterDiscarded { intent } => {
            bytes.push(0x08);
            bytes.extend_from_slice(&intent.0.to_le_bytes());
        }
    }
}

fn dead_reason_tag(reason: EffectDeadLetterReason) -> u8 {
    match reason {
        EffectDeadLetterReason::PermanentBackendFailure => 0x01,
        EffectDeadLetterReason::AttemptsExhausted => 0x02,
    }
}

fn push_option_string(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            bytes.push(1);
            push_string(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn push_queued(bytes: &mut Vec<u8>, request: &QueuedEffectIntent) {
    bytes.extend_from_slice(&request.id.0.to_le_bytes());
    bytes.extend_from_slice(&request.cycle.to_le_bytes());
    bytes.extend_from_slice(&request.ordinal.to_le_bytes());
    bytes.extend_from_slice(&request.intent.reaction.0.to_le_bytes());
    push_string(bytes, &request.intent.action_name);
    push_effect(bytes, &request.intent.effect);
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn push_effect(bytes: &mut Vec<u8>, effect: &Effect) {
    match effect {
        Effect::Pure => bytes.push(0x00),
        Effect::StateRead => bytes.push(0x01),
        Effect::StateWrite => bytes.push(0x02),
        Effect::Network(scope) => {
            bytes.push(0x10);
            push_string(bytes, scope);
        }
        Effect::FileRead(scope) => {
            bytes.push(0x11);
            push_string(bytes, scope);
        }
        Effect::FileWrite(scope) => {
            bytes.push(0x12);
            push_string(bytes, scope);
        }
        Effect::Camera => bytes.push(0x20),
        Effect::Microphone => bytes.push(0x21),
        Effect::Location => bytes.push(0x22),
        Effect::Gpu => bytes.push(0x23),
        Effect::Xr => bytes.push(0x24),
        Effect::Process => bytes.push(0x25),
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn read_exact(&mut self, len: usize) -> EffectAuditResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(EffectAuditError::AuditCheckpointTruncated)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(EffectAuditError::AuditCheckpointTruncated)?;
        self.offset = end;
        Ok(slice)
    }
    fn read_u8(&mut self) -> EffectAuditResult<u8> {
        Ok(self.read_exact(1)?[0])
    }
    fn read_u16(&mut self) -> EffectAuditResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_exact(2)?
                .try_into()
                .map_err(|_| EffectAuditError::AuditCheckpointTruncated)?,
        ))
    }
    fn read_u64(&mut self) -> EffectAuditResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?
                .try_into()
                .map_err(|_| EffectAuditError::AuditCheckpointTruncated)?,
        ))
    }
    fn read_array_16(&mut self) -> EffectAuditResult<[u8; 16]> {
        self.read_exact(16)?
            .try_into()
            .map_err(|_| EffectAuditError::AuditCheckpointTruncated)
    }
    fn read_array_32(&mut self) -> EffectAuditResult<[u8; 32]> {
        self.read_exact(32)?
            .try_into()
            .map_err(|_| EffectAuditError::AuditCheckpointTruncated)
    }
    fn read_string(&mut self) -> EffectAuditResult<String> {
        let len_u64 = self.read_u64()?;
        if len_u64 > MAX_EFFECT_JOURNAL_STRING_BYTES as u64 {
            return Err(EffectAuditError::AuditCheckpointStringTooLarge {
                bytes: len_u64,
                limit: MAX_EFFECT_JOURNAL_STRING_BYTES,
            });
        }
        let len =
            usize::try_from(len_u64).map_err(|_| EffectAuditError::AuditCheckpointTruncated)?;
        String::from_utf8(self.read_exact(len)?.to_vec())
            .map_err(|_| EffectAuditError::AuditCheckpointInvalidUtf8)
    }
    fn read_option_string(&mut self) -> EffectAuditResult<Option<String>> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_string()?)),
            tag => Err(EffectAuditError::InvalidAuditEventState(format!(
                "invalid optional string flag {tag}"
            ))),
        }
    }
    fn read_effect(&mut self) -> EffectAuditResult<Effect> {
        match self.read_u8()? {
            0x00 => Ok(Effect::Pure),
            0x01 => Ok(Effect::StateRead),
            0x02 => Ok(Effect::StateWrite),
            0x10 => Ok(Effect::Network(self.read_string()?)),
            0x11 => Ok(Effect::FileRead(self.read_string()?)),
            0x12 => Ok(Effect::FileWrite(self.read_string()?)),
            0x20 => Ok(Effect::Camera),
            0x21 => Ok(Effect::Microphone),
            0x22 => Ok(Effect::Location),
            0x23 => Ok(Effect::Gpu),
            0x24 => Ok(Effect::Xr),
            0x25 => Ok(Effect::Process),
            tag => Err(EffectAuditError::Persistence(
                crate::effect_persistence::EffectPersistenceError::UnknownEffectTag(tag),
            )),
        }
    }
    fn read_queued(&mut self) -> EffectAuditResult<QueuedEffectIntent> {
        let id = EffectIntentId(self.read_u64()?);
        if id.0 == 0 {
            return Err(EffectAuditError::InvalidAuditEventState(
                "zero queued intent id".into(),
            ));
        }
        let cycle = self.read_u64()?;
        let ordinal = self.read_u64()?;
        let reaction = self.read_u64()?;
        if reaction == 0 {
            return Err(EffectAuditError::InvalidAuditEventState(
                "zero reaction id".into(),
            ));
        }
        let action_name = self.read_string()?;
        let effect = self.read_effect()?;
        Ok(QueuedEffectIntent {
            id,
            cycle,
            ordinal,
            intent: EffectIntent {
                reaction: ReactionId(reaction),
                action_name,
                effect,
            },
        })
    }
    fn read_event(
        &mut self,
        namespace: EffectDeliveryNamespace,
    ) -> EffectAuditResult<EffectAuditEvent> {
        match self.read_u8()? {
            0x01 => {
                let attempt = EffectAttemptId(self.read_u64()?);
                let request = self.read_queued()?;
                let key_namespace = EffectDeliveryNamespace::new(self.read_array_16()?);
                let key_intent = EffectIntentId(self.read_u64()?);
                if key_namespace != namespace || key_intent != request.id {
                    return Err(EffectAuditError::AuditNamespaceMismatch);
                }
                let delivery_fence = EffectDeliveryFence(self.read_u64()?);
                let prepared_tick = EffectRetryTick(self.read_u64()?);
                let failed_u64 = self.read_u64()?;
                let failed_attempts_before = u32::try_from(failed_u64).map_err(|_| {
                    EffectAuditError::InvalidAuditEventState("failure count exceeds u32".into())
                })?;
                Ok(EffectAuditEvent::AttemptPrepared(EffectInDoubtAttempt {
                    attempt,
                    request,
                    delivery_key: EffectDeliveryKey::new(namespace, key_intent),
                    delivery_fence,
                    prepared_tick,
                    failed_attempts_before,
                }))
            }
            0x02 => Ok(EffectAuditEvent::AttemptDelivered {
                attempt: EffectAttemptId(self.read_u64()?),
                intent: EffectIntentId(self.read_u64()?),
                backend_reference: self.read_option_string()?,
            }),
            0x03 => {
                let attempt = EffectAttemptId(self.read_u64()?);
                let intent = EffectIntentId(self.read_u64()?);
                let failed_attempts = u32::try_from(self.read_u64()?).map_err(|_| {
                    EffectAuditError::InvalidAuditEventState("failure count exceeds u32".into())
                })?;
                let next_eligible_tick = EffectRetryTick(self.read_u64()?);
                let error = self.read_string()?;
                Ok(EffectAuditEvent::AttemptRetryScheduled {
                    attempt,
                    intent,
                    failed_attempts,
                    next_eligible_tick,
                    error,
                })
            }
            0x04 => {
                let attempt = EffectAttemptId(self.read_u64()?);
                let intent = EffectIntentId(self.read_u64()?);
                let failed_attempts = u32::try_from(self.read_u64()?).map_err(|_| {
                    EffectAuditError::InvalidAuditEventState("failure count exceeds u32".into())
                })?;
                let reason = match self.read_u8()? {
                    0x01 => EffectDeadLetterReason::PermanentBackendFailure,
                    0x02 => EffectDeadLetterReason::AttemptsExhausted,
                    tag => return Err(EffectAuditError::InvalidDeadLetterReason(tag)),
                };
                let error = self.read_string()?;
                Ok(EffectAuditEvent::AttemptDeadLettered {
                    attempt,
                    intent,
                    failed_attempts,
                    reason,
                    error,
                })
            }
            0x05 => Ok(EffectAuditEvent::InDoubtAssumedDelivered {
                attempt: EffectAttemptId(self.read_u64()?),
                intent: EffectIntentId(self.read_u64()?),
                resolution_tick: EffectRetryTick(self.read_u64()?),
                backend_reference: self.read_option_string()?,
            }),
            0x06 => Ok(EffectAuditEvent::InDoubtRetryAuthorized {
                attempt: EffectAttemptId(self.read_u64()?),
                intent: EffectIntentId(self.read_u64()?),
                resolution_tick: EffectRetryTick(self.read_u64()?),
            }),
            0x07 => Ok(EffectAuditEvent::DeadLetterRedriven {
                intent: EffectIntentId(self.read_u64()?),
            }),
            0x08 => Ok(EffectAuditEvent::DeadLetterDiscarded {
                intent: EffectIntentId(self.read_u64()?),
            }),
            tag => Err(EffectAuditError::InvalidAuditEventTag(tag)),
        }
    }
    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
