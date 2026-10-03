use crate::{
    effect::Effect,
    effect_dispatch::{
        AtomicEffectOutbox, EffectDeliveryKey, EffectDeliveryNamespace, EffectIntentId,
        QueuedEffectIntent,
    },
    effect_persistence::{
        EffectOutboxCheckpoint, MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES, MAX_EFFECT_JOURNAL_PENDING,
        MAX_EFFECT_JOURNAL_STRING_BYTES,
    },
    reaction::{EffectIntent, ReactionId},
};

use super::{
    DeadLetteredEffect, EffectDeadLetterReason, EffectRetryError, EffectRetryLedger,
    EffectRetryPolicy, EffectRetryRecord, EffectRetryResult, EffectRetryTick,
};

const MAGIC: &[u8; 8] = b"NDEFXR01";
const LEGACY_MAGIC: &[u8; 8] = b"NDEFXJ01";
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 0;
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001B3;

pub const MAX_EFFECT_RETRY_CHECKPOINT_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_EFFECT_RETRY_ENTRIES: usize = MAX_EFFECT_JOURNAL_PENDING;
pub const MAX_EFFECT_DEAD_LETTERS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectRetryCheckpoint {
    namespace: EffectDeliveryNamespace,
    policy: Option<EffectRetryPolicy>,
    outbox: EffectOutboxCheckpoint,
    ledger: EffectRetryLedger,
}

impl EffectRetryCheckpoint {
    pub fn capture(
        namespace: EffectDeliveryNamespace,
        policy: EffectRetryPolicy,
        outbox: &AtomicEffectOutbox,
        ledger: &EffectRetryLedger,
    ) -> Self {
        Self {
            namespace,
            policy: Some(policy),
            outbox: EffectOutboxCheckpoint::capture(namespace, outbox),
            ledger: ledger.clone(),
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
    }

    pub fn policy(&self) -> Option<EffectRetryPolicy> {
        self.policy
    }

    pub fn outbox(&self) -> &EffectOutboxCheckpoint {
        &self.outbox
    }

    pub fn ledger(&self) -> &EffectRetryLedger {
        &self.ledger
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let outbox_bytes = self.outbox.canonical_bytes();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_MAJOR.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        bytes.extend_from_slice(&self.namespace.0);
        match self.policy {
            Some(policy) => {
                bytes.push(1);
                bytes.extend_from_slice(&u64::from(policy.max_attempts()).to_le_bytes());
                bytes.extend_from_slice(&policy.initial_backoff_ticks().to_le_bytes());
                bytes.extend_from_slice(&policy.max_backoff_ticks().to_le_bytes());
                bytes.extend_from_slice(&policy.jitter_ticks().to_le_bytes());
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&(outbox_bytes.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&outbox_bytes);
        bytes.extend_from_slice(&self.ledger.last_tick().0.to_le_bytes());

        bytes.extend_from_slice(&(self.ledger.retry_count() as u64).to_le_bytes());
        for record in self.ledger.retries() {
            bytes.extend_from_slice(&record.intent.0.to_le_bytes());
            bytes.extend_from_slice(&u64::from(record.failed_attempts).to_le_bytes());
            bytes.extend_from_slice(&record.next_eligible_tick.0.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.ledger.dead_letter_count() as u64).to_le_bytes());
        for dead in self.ledger.dead_letters() {
            push_queued(&mut bytes, &dead.request);
            bytes.extend_from_slice(&u64::from(dead.failed_attempts).to_le_bytes());
            bytes.push(match dead.reason {
                EffectDeadLetterReason::PermanentBackendFailure => 0x01,
                EffectDeadLetterReason::AttemptsExhausted => 0x02,
            });
            push_string(&mut bytes, &dead.last_error);
        }

        let checksum = checksum(&bytes);
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes
    }

    /// Decodes K1.9 retry checkpoints and migrates certified K1.7/K1.8 outbox-only
    /// checkpoints with an empty retry/dead-letter ledger.
    pub fn from_canonical_bytes(bytes: &[u8]) -> EffectRetryResult<Self> {
        if bytes.len() > MAX_EFFECT_RETRY_CHECKPOINT_BYTES {
            return Err(EffectRetryError::RetryCheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_RETRY_CHECKPOINT_BYTES,
            });
        }

        if bytes.starts_with(LEGACY_MAGIC) {
            if bytes.len() > MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES {
                return Err(EffectRetryError::Persistence(
                    crate::effect_persistence::EffectPersistenceError::CheckpointTooLarge {
                        bytes: bytes.len(),
                        limit: MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES,
                    },
                ));
            }
            let outbox = EffectOutboxCheckpoint::from_canonical_bytes(bytes)?;
            return Ok(Self {
                namespace: outbox.namespace(),
                policy: None,
                outbox,
                ledger: EffectRetryLedger::new(),
            });
        }

        if bytes.len() < 8 {
            return Err(EffectRetryError::RetryCheckpointTruncated);
        }
        let payload_len = bytes
            .len()
            .checked_sub(8)
            .ok_or(EffectRetryError::RetryCheckpointTruncated)?;
        let (payload, checksum_bytes) = bytes.split_at(payload_len);
        let expected = u64::from_le_bytes(
            checksum_bytes
                .try_into()
                .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?,
        );
        let actual = checksum(payload);
        if expected != actual {
            return Err(EffectRetryError::RetryCheckpointChecksumMismatch { expected, actual });
        }

        let mut reader = Reader::new(payload);
        if reader.read_exact(8)? != MAGIC.as_slice() {
            return Err(EffectRetryError::InvalidRetryCheckpointMagic);
        }
        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != FORMAT_MAJOR || minor != FORMAT_MINOR {
            return Err(EffectRetryError::UnsupportedRetryCheckpointVersion { major, minor });
        }

        let namespace = EffectDeliveryNamespace(
            reader
                .read_exact(16)?
                .try_into()
                .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?,
        );
        let policy = match reader.read_u8()? {
            0 => None,
            1 => {
                let max_attempts_u64 = reader.read_u64()?;
                let max_attempts = u32::try_from(max_attempts_u64).map_err(|_| {
                    EffectRetryError::InvalidPolicy("max_attempts exceeds u32".into())
                })?;
                let initial_backoff_ticks = reader.read_u64()?;
                let max_backoff_ticks = reader.read_u64()?;
                let jitter_ticks = reader.read_u64()?;
                Some(EffectRetryPolicy::new(
                    max_attempts,
                    initial_backoff_ticks,
                    max_backoff_ticks,
                    jitter_ticks,
                )?)
            }
            flag => {
                return Err(EffectRetryError::InvalidPolicy(format!(
                    "invalid persisted policy flag {flag}"
                )))
            }
        };
        let outbox_len_u64 = reader.read_u64()?;
        if outbox_len_u64 > MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES as u64 {
            return Err(EffectRetryError::Persistence(
                crate::effect_persistence::EffectPersistenceError::CheckpointTooLarge {
                    bytes: usize::try_from(outbox_len_u64).unwrap_or(usize::MAX),
                    limit: MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES,
                },
            ));
        }
        let outbox_len = usize::try_from(outbox_len_u64)
            .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?;
        let outbox = EffectOutboxCheckpoint::from_canonical_bytes(reader.read_exact(outbox_len)?)?;
        if outbox.namespace() != namespace {
            return Err(EffectRetryError::RetryNamespaceMismatch);
        }

        let last_tick = EffectRetryTick(reader.read_u64()?);
        let retry_count_u64 = reader.read_u64()?;
        if retry_count_u64 > MAX_EFFECT_RETRY_ENTRIES as u64 {
            return Err(EffectRetryError::RetryEntryLimitExceeded {
                entries: retry_count_u64,
                limit: MAX_EFFECT_RETRY_ENTRIES,
            });
        }
        let retry_count = usize::try_from(retry_count_u64)
            .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?;
        let mut retries = Vec::with_capacity(retry_count);
        let mut previous_retry = 0_u64;
        for _ in 0..retry_count {
            let intent = EffectIntentId(reader.read_u64()?);
            if intent.0 == 0 || intent.0 <= previous_retry {
                return Err(EffectRetryError::DuplicateOrUnorderedRetryIntent(intent));
            }
            let failed_u64 = reader.read_u64()?;
            let failed_attempts = u32::try_from(failed_u64)
                .map_err(|_| EffectRetryError::InvalidFailureCount(u32::MAX))?;
            if failed_attempts == 0 {
                return Err(EffectRetryError::InvalidFailureCount(0));
            }
            let next_eligible_tick = EffectRetryTick(reader.read_u64()?);
            if outbox.pending().iter().all(|request| request.id != intent) {
                return Err(EffectRetryError::RetryIntentNotPending(intent));
            }
            retries.push(EffectRetryRecord {
                intent,
                failed_attempts,
                next_eligible_tick,
            });
            previous_retry = intent.0;
        }

        let dead_count_u64 = reader.read_u64()?;
        if dead_count_u64 > MAX_EFFECT_DEAD_LETTERS as u64 {
            return Err(EffectRetryError::DeadLetterLimitExceeded {
                entries: dead_count_u64,
                limit: MAX_EFFECT_DEAD_LETTERS,
            });
        }
        let dead_count = usize::try_from(dead_count_u64)
            .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?;
        let mut dead_letters = Vec::with_capacity(dead_count);
        let mut previous_dead = 0_u64;
        for _ in 0..dead_count {
            let request = reader.read_queued()?;
            if request.id.0 <= previous_dead {
                return Err(EffectRetryError::DuplicateOrUnorderedDeadLetter(request.id));
            }
            if outbox
                .pending()
                .iter()
                .any(|pending| pending.id == request.id)
            {
                return Err(EffectRetryError::RetryIntentNotPending(request.id));
            }
            let failed_u64 = reader.read_u64()?;
            let failed_attempts = u32::try_from(failed_u64)
                .map_err(|_| EffectRetryError::InvalidFailureCount(u32::MAX))?;
            if failed_attempts == 0 {
                return Err(EffectRetryError::InvalidFailureCount(0));
            }
            let reason = match reader.read_u8()? {
                0x01 => EffectDeadLetterReason::PermanentBackendFailure,
                0x02 => EffectDeadLetterReason::AttemptsExhausted,
                tag => return Err(EffectRetryError::InvalidDeadLetterReason(tag)),
            };
            let last_error = reader.read_string()?;
            dead_letters.push(DeadLetteredEffect {
                delivery_key: EffectDeliveryKey::new(namespace, request.id),
                request,
                failed_attempts,
                reason,
                last_error,
            });
            previous_dead = dead_letters.last().unwrap().request.id.0;
        }

        if !reader.is_finished() {
            return Err(EffectRetryError::RetryCheckpointTruncated);
        }

        Ok(Self {
            namespace,
            policy,
            outbox,
            ledger: EffectRetryLedger::from_parts(last_tick, retries, dead_letters),
        })
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

fn checksum(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_exact(&mut self, len: usize) -> EffectRetryResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(EffectRetryError::RetryCheckpointTruncated)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(EffectRetryError::RetryCheckpointTruncated)?;
        self.offset = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> EffectRetryResult<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> EffectRetryResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_exact(2)?
                .try_into()
                .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?,
        ))
    }

    fn read_u64(&mut self) -> EffectRetryResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?
                .try_into()
                .map_err(|_| EffectRetryError::RetryCheckpointTruncated)?,
        ))
    }

    fn read_string(&mut self) -> EffectRetryResult<String> {
        let len_u64 = self.read_u64()?;
        if len_u64 > MAX_EFFECT_JOURNAL_STRING_BYTES as u64 {
            return Err(EffectRetryError::RetryCheckpointStringTooLarge {
                bytes: len_u64,
                limit: MAX_EFFECT_JOURNAL_STRING_BYTES,
            });
        }
        let len =
            usize::try_from(len_u64).map_err(|_| EffectRetryError::RetryCheckpointTruncated)?;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| EffectRetryError::RetryCheckpointInvalidUtf8)
    }

    fn read_effect(&mut self) -> EffectRetryResult<Effect> {
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
            tag => Err(EffectRetryError::Persistence(
                crate::effect_persistence::EffectPersistenceError::UnknownEffectTag(tag),
            )),
        }
    }

    fn read_queued(&mut self) -> EffectRetryResult<QueuedEffectIntent> {
        let id = EffectIntentId(self.read_u64()?);
        if id.0 == 0 {
            return Err(EffectRetryError::UnknownDeadLetter(id));
        }
        let cycle = self.read_u64()?;
        let ordinal = self.read_u64()?;
        let reaction = self.read_u64()?;
        if reaction == 0 {
            return Err(EffectRetryError::RetryCheckpointTruncated);
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

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
