use crate::{
    effect::Effect,
    effect_dispatch::{
        AtomicEffectOutbox, EffectDeliveryNamespace, EffectIntentId, QueuedEffectIntent,
    },
    reaction::{EffectIntent, ReactionId},
};

use super::{EffectPersistenceError, EffectPersistenceResult};

const MAGIC: &[u8; 8] = b"NDEFXJ01";
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 0;
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001B3;
pub const MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_EFFECT_JOURNAL_PENDING: usize = 65_536;
pub const MAX_EFFECT_JOURNAL_STRING_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectOutboxCheckpoint {
    namespace: EffectDeliveryNamespace,
    next_intent_id: u64,
    pending: Vec<QueuedEffectIntent>,
}

impl EffectOutboxCheckpoint {
    pub fn capture(namespace: EffectDeliveryNamespace, outbox: &AtomicEffectOutbox) -> Self {
        Self {
            namespace,
            next_intent_id: outbox.next_intent_id(),
            pending: outbox.iter().cloned().collect(),
        }
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
    }

    pub fn next_intent_id(&self) -> u64 {
        self.next_intent_id
    }

    pub fn pending(&self) -> &[QueuedEffectIntent] {
        &self.pending
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_MAJOR.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        bytes.extend_from_slice(&self.namespace.0);
        bytes.extend_from_slice(&self.next_intent_id.to_le_bytes());
        bytes.extend_from_slice(&(self.pending.len() as u64).to_le_bytes());

        for request in &self.pending {
            bytes.extend_from_slice(&request.id.0.to_le_bytes());
            bytes.extend_from_slice(&request.cycle.to_le_bytes());
            bytes.extend_from_slice(&request.ordinal.to_le_bytes());
            bytes.extend_from_slice(&request.intent.reaction.0.to_le_bytes());
            push_string(&mut bytes, &request.intent.action_name);
            push_effect(&mut bytes, &request.intent.effect);
        }

        let checksum = checksum(&bytes);
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> EffectPersistenceResult<Self> {
        if bytes.len() > MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES {
            return Err(EffectPersistenceError::CheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_JOURNAL_CHECKPOINT_BYTES,
            });
        }
        if bytes.len() < 8 {
            return Err(EffectPersistenceError::Truncated);
        }
        let payload_len = bytes.len() - 8;
        let (payload, checksum_bytes) = bytes.split_at(payload_len);
        let expected = u64::from_le_bytes(
            checksum_bytes
                .try_into()
                .map_err(|_| EffectPersistenceError::Truncated)?,
        );
        let actual = checksum(payload);
        if expected != actual {
            return Err(EffectPersistenceError::ChecksumMismatch { expected, actual });
        }

        let mut reader = Reader::new(payload);
        if reader.read_exact(8)? != MAGIC.as_slice() {
            return Err(EffectPersistenceError::InvalidMagic);
        }
        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != FORMAT_MAJOR || minor != FORMAT_MINOR {
            return Err(EffectPersistenceError::UnsupportedVersion { major, minor });
        }

        let namespace_bytes: [u8; 16] = reader
            .read_exact(16)?
            .try_into()
            .map_err(|_| EffectPersistenceError::Truncated)?;
        let namespace = EffectDeliveryNamespace(namespace_bytes);
        let next_intent_id = reader.read_u64()?;
        if next_intent_id == 0 {
            return Err(EffectPersistenceError::InvalidNextIntentId(next_intent_id));
        }
        let count_u64 = reader.read_u64()?;
        if count_u64 > MAX_EFFECT_JOURNAL_PENDING as u64 {
            return Err(EffectPersistenceError::PendingLimitExceeded {
                pending: count_u64,
                limit: MAX_EFFECT_JOURNAL_PENDING,
            });
        }
        let count = usize::try_from(count_u64).map_err(|_| EffectPersistenceError::Truncated)?;
        let mut pending = Vec::with_capacity(count);
        let mut previous_id = 0_u64;
        for _ in 0..count {
            let id = EffectIntentId(reader.read_u64()?);
            if id.0 == 0 {
                return Err(EffectPersistenceError::InvalidIntentId(id));
            }
            if id.0 <= previous_id {
                return Err(EffectPersistenceError::DuplicateOrUnorderedIntent(id));
            }
            let cycle = reader.read_u64()?;
            let ordinal = reader.read_u64()?;
            let reaction = reader.read_u64()?;
            if reaction == 0 {
                return Err(EffectPersistenceError::InvalidReactionId(reaction));
            }
            let action_name = reader.read_string()?;
            let effect = reader.read_effect()?;
            pending.push(QueuedEffectIntent {
                id,
                cycle,
                ordinal,
                intent: EffectIntent {
                    reaction: ReactionId(reaction),
                    action_name,
                    effect,
                },
            });
            previous_id = id.0;
        }

        if !reader.is_finished() {
            return Err(EffectPersistenceError::Truncated);
        }
        if next_intent_id <= previous_id {
            return Err(EffectPersistenceError::InvalidNextIntentId(next_intent_id));
        }

        Ok(Self {
            namespace,
            next_intent_id,
            pending,
        })
    }

    pub(crate) fn to_outbox(&self) -> AtomicEffectOutbox {
        AtomicEffectOutbox::from_checkpoint_parts(self.pending.clone(), self.next_intent_id)
    }
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

    fn read_exact(&mut self, len: usize) -> EffectPersistenceResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(EffectPersistenceError::Truncated)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(EffectPersistenceError::Truncated)?;
        self.offset = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> EffectPersistenceResult<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> EffectPersistenceResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_exact(2)?
                .try_into()
                .map_err(|_| EffectPersistenceError::Truncated)?,
        ))
    }

    fn read_u64(&mut self) -> EffectPersistenceResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?
                .try_into()
                .map_err(|_| EffectPersistenceError::Truncated)?,
        ))
    }

    fn read_string(&mut self) -> EffectPersistenceResult<String> {
        let len_u64 = self.read_u64()?;
        if len_u64 > MAX_EFFECT_JOURNAL_STRING_BYTES as u64 {
            return Err(EffectPersistenceError::StringTooLarge {
                bytes: len_u64,
                limit: MAX_EFFECT_JOURNAL_STRING_BYTES,
            });
        }
        let len = usize::try_from(len_u64).map_err(|_| EffectPersistenceError::Truncated)?;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| EffectPersistenceError::InvalidUtf8)
    }

    fn read_effect(&mut self) -> EffectPersistenceResult<Effect> {
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
            tag => Err(EffectPersistenceError::UnknownEffectTag(tag)),
        }
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
