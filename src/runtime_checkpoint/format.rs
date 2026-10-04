use std::collections::{BTreeMap, BTreeSet};

use crate::{
    effect_audit::{hash::sha256, EffectAuditHash},
    effect_completion::{EffectCompletionSequence, EffectCompletionSourceId},
    effect_dispatch::{EffectDeliveryKey, EffectDeliveryNamespace, EffectIntentId},
    input::InputSequence,
    nair::{AtomSlot, RenderNodeSlot},
    program_upgrade::{ProgramEpoch, RuntimeUpgradeHash, RuntimeUpgradeLineageRecord},
    runtime::RuntimeAtomSnapshot,
    time::{LogicalDuration, LogicalTime, TimerId, TimerSnapshot},
    value::Value,
};

use super::{
    CompletionCheckpointState, PersistentRuntimeCheckpointState, RenderRevisionCheckpoint,
    RuntimeCheckpointError, RuntimeCheckpointResult, TimeCheckpointState,
};

const MAGIC: &[u8; 8] = b"NDRTSM01";
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 1;
const CHECKPOINT_HASH_DOMAIN_V1_0: &[u8] = b"NORDOI-RUNTIME-SEMANTIC-CHECKPOINT-1.0";
const CHECKPOINT_HASH_DOMAIN_V1_1: &[u8] = b"NORDOI-RUNTIME-SEMANTIC-CHECKPOINT-1.1";

pub const MAX_RUNTIME_CHECKPOINT_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_RUNTIME_CHECKPOINT_ATOMS: usize = 262_144;
pub const MAX_RUNTIME_CHECKPOINT_TIMERS: usize = 262_144;
pub const MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES: usize = 65_536;
pub const MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES: usize = 262_144;
pub const MAX_RUNTIME_CHECKPOINT_TEXT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSemanticCheckpoint {
    namespace: EffectDeliveryNamespace,
    program_hash: [u8; 32],
    program_epoch: ProgramEpoch,
    upgrade_chain_root: RuntimeUpgradeHash,
    last_upgrade: Option<RuntimeUpgradeLineageRecord>,
    audit_events: u64,
    audit_root: EffectAuditHash,
    effect_next_intent_id: u64,
    cycle: u64,
    event_replay_state: u64,
    runtime: PersistentRuntimeCheckpointState,
    time: TimeCheckpointState,
    completions: CompletionCheckpointState,
}

impl RuntimeSemanticCheckpoint {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        namespace: EffectDeliveryNamespace,
        program_hash: [u8; 32],
        program_epoch: ProgramEpoch,
        upgrade_chain_root: RuntimeUpgradeHash,
        last_upgrade: Option<RuntimeUpgradeLineageRecord>,
        audit_events: u64,
        audit_root: EffectAuditHash,
        effect_next_intent_id: u64,
        cycle: u64,
        event_replay_state: u64,
        runtime: PersistentRuntimeCheckpointState,
        time: TimeCheckpointState,
        completions: CompletionCheckpointState,
    ) -> RuntimeCheckpointResult<Self> {
        let checkpoint = Self {
            namespace,
            program_hash,
            program_epoch,
            upgrade_chain_root,
            last_upgrade,
            audit_events,
            audit_root,
            effect_next_intent_id,
            cycle,
            event_replay_state,
            runtime,
            time,
            completions,
        };
        checkpoint.validate()?;
        Ok(checkpoint)
    }

    pub fn namespace(&self) -> EffectDeliveryNamespace {
        self.namespace
    }
    pub fn program_hash(&self) -> [u8; 32] {
        self.program_hash
    }
    pub fn program_epoch(&self) -> ProgramEpoch {
        self.program_epoch
    }
    pub fn upgrade_chain_root(&self) -> RuntimeUpgradeHash {
        self.upgrade_chain_root
    }
    pub fn last_upgrade(&self) -> Option<&RuntimeUpgradeLineageRecord> {
        self.last_upgrade.as_ref()
    }
    pub fn audit_events(&self) -> u64 {
        self.audit_events
    }
    pub fn audit_root(&self) -> EffectAuditHash {
        self.audit_root
    }
    pub fn effect_next_intent_id(&self) -> u64 {
        self.effect_next_intent_id
    }
    pub fn cycle(&self) -> u64 {
        self.cycle
    }
    pub fn event_replay_state(&self) -> u64 {
        self.event_replay_state
    }
    pub fn runtime_tick(&self) -> u64 {
        self.runtime.tick
    }
    pub fn runtime_replay_state(&self) -> u64 {
        self.runtime.replay_state
    }
    pub fn logical_time(&self) -> LogicalTime {
        self.time.now
    }
    pub fn atom_count(&self) -> usize {
        self.runtime.atoms.len()
    }
    pub fn timer_count(&self) -> usize {
        self.time.timers.len()
    }
    pub fn completion_source_count(&self) -> usize {
        self.completions.last_sequences.len()
    }
    pub fn completed_delivery_count(&self) -> usize {
        self.completions.completed_deliveries.len()
    }

    pub(crate) fn runtime_state(&self) -> &PersistentRuntimeCheckpointState {
        &self.runtime
    }
    pub(crate) fn time_state(&self) -> &TimeCheckpointState {
        &self.time
    }
    pub(crate) fn completion_state(&self) -> &CompletionCheckpointState {
        &self.completions
    }

    pub fn canonical_bytes(&self) -> RuntimeCheckpointResult<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_MAJOR.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        bytes.extend_from_slice(&self.namespace.0);
        bytes.extend_from_slice(&self.program_hash);
        bytes.extend_from_slice(&self.program_epoch.0.to_le_bytes());
        bytes.extend_from_slice(&self.upgrade_chain_root.0);
        match &self.last_upgrade {
            Some(record) => {
                bytes.push(1);
                bytes.extend_from_slice(&record.source_program_hash);
                bytes.extend_from_slice(&record.target_program_hash);
                bytes.extend_from_slice(&record.plan_hash.0);
                bytes.extend_from_slice(&record.source_checkpoint_hash);
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&self.audit_events.to_le_bytes());
        bytes.extend_from_slice(&self.audit_root.0);
        bytes.extend_from_slice(&self.effect_next_intent_id.to_le_bytes());
        bytes.extend_from_slice(&self.cycle.to_le_bytes());
        bytes.extend_from_slice(&self.event_replay_state.to_le_bytes());

        bytes.extend_from_slice(&self.runtime.tick.to_le_bytes());
        bytes.extend_from_slice(&self.runtime.replay_state.to_le_bytes());
        match self.runtime.last_input_sequence {
            Some(sequence) => {
                bytes.push(1);
                bytes.extend_from_slice(&sequence.0.to_le_bytes());
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&self.runtime.next_transaction_id.to_le_bytes());
        push_len(&mut bytes, self.runtime.atoms.len())?;
        for (slot, snapshot) in &self.runtime.atoms {
            bytes.extend_from_slice(&slot.0.to_le_bytes());
            bytes.extend_from_slice(&snapshot.id.0.to_le_bytes());
            bytes.extend_from_slice(&snapshot.version.to_le_bytes());
            push_value(&mut bytes, &snapshot.value)?;
        }
        push_len(&mut bytes, self.runtime.render_revisions.len())?;
        for revision in &self.runtime.render_revisions {
            bytes.extend_from_slice(&revision.slot.0.to_le_bytes());
            bytes.extend_from_slice(&revision.revision.to_le_bytes());
        }

        bytes.extend_from_slice(&self.time.now.0.to_le_bytes());
        bytes.extend_from_slice(&self.time.next_timer_id.to_le_bytes());
        let fire_budget = u64::try_from(self.time.fire_budget).map_err(|_| {
            RuntimeCheckpointError::InternalState("timer fire budget does not fit u64".into())
        })?;
        bytes.extend_from_slice(&fire_budget.to_le_bytes());
        push_len(&mut bytes, self.time.timers.len())?;
        for timer in &self.time.timers {
            bytes.extend_from_slice(&timer.id.0.to_le_bytes());
            bytes.extend_from_slice(&timer.next_deadline.0.to_le_bytes());
            match timer.interval {
                Some(interval) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&interval.0.to_le_bytes());
                }
                None => bytes.push(0),
            }
            bytes.extend_from_slice(&timer.occurrences.to_le_bytes());
        }

        push_len(&mut bytes, self.completions.last_sequences.len())?;
        for (source, sequence) in &self.completions.last_sequences {
            bytes.extend_from_slice(&source.0.to_le_bytes());
            bytes.extend_from_slice(&sequence.0.to_le_bytes());
        }
        push_len(&mut bytes, self.completions.completed_deliveries.len())?;
        for delivery in &self.completions.completed_deliveries {
            bytes.extend_from_slice(&delivery.namespace.0);
            bytes.extend_from_slice(&delivery.intent.0.to_le_bytes());
        }

        let digest = checkpoint_digest(&bytes);
        bytes.extend_from_slice(&digest);
        if bytes.len() > MAX_RUNTIME_CHECKPOINT_BYTES {
            return Err(RuntimeCheckpointError::CheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_RUNTIME_CHECKPOINT_BYTES,
            });
        }
        Ok(bytes)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> RuntimeCheckpointResult<Self> {
        if bytes.len() > MAX_RUNTIME_CHECKPOINT_BYTES {
            return Err(RuntimeCheckpointError::CheckpointTooLarge {
                bytes: bytes.len(),
                limit: MAX_RUNTIME_CHECKPOINT_BYTES,
            });
        }
        if bytes.len() < 32 {
            return Err(RuntimeCheckpointError::Truncated);
        }
        let (payload, digest_bytes) = bytes.split_at(bytes.len() - 32);
        let expected: [u8; 32] = digest_bytes
            .try_into()
            .map_err(|_| RuntimeCheckpointError::Truncated)?;
        if payload.len() < 12 {
            return Err(RuntimeCheckpointError::Truncated);
        }
        if &payload[..8] != MAGIC {
            return Err(RuntimeCheckpointError::InvalidMagic);
        }
        let minor = u16::from_le_bytes([payload[10], payload[11]]);
        let actual = checkpoint_digest_for_minor(payload, minor);
        if expected != actual {
            return Err(RuntimeCheckpointError::DigestMismatch { expected, actual });
        }

        let mut reader = Reader::new(payload);
        if reader.read_exact(8)? != MAGIC {
            return Err(RuntimeCheckpointError::InvalidMagic);
        }
        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != FORMAT_MAJOR || minor > FORMAT_MINOR {
            return Err(RuntimeCheckpointError::UnsupportedVersion { major, minor });
        }
        let namespace = EffectDeliveryNamespace::new(reader.read_array_16()?);
        let program_hash = reader.read_array_32()?;
        let (program_epoch, upgrade_chain_root, last_upgrade) = if minor >= 1 {
            let program_epoch = ProgramEpoch(reader.read_u64()?);
            let upgrade_chain_root = RuntimeUpgradeHash(reader.read_array_32()?);
            let last_upgrade = match reader.read_u8()? {
                0 => None,
                1 => Some(RuntimeUpgradeLineageRecord {
                    source_program_hash: reader.read_array_32()?,
                    target_program_hash: reader.read_array_32()?,
                    plan_hash: RuntimeUpgradeHash(reader.read_array_32()?),
                    source_checkpoint_hash: reader.read_array_32()?,
                }),
                _ => return Err(RuntimeCheckpointError::Truncated),
            };
            (program_epoch, upgrade_chain_root, last_upgrade)
        } else {
            (ProgramEpoch(0), RuntimeUpgradeHash::ZERO, None)
        };
        let audit_events = reader.read_u64()?;
        let audit_root = EffectAuditHash(reader.read_array_32()?);
        let effect_next_intent_id = reader.read_u64()?;
        let cycle = reader.read_u64()?;
        let event_replay_state = reader.read_u64()?;

        let tick = reader.read_u64()?;
        let runtime_replay_state = reader.read_u64()?;
        let last_input_sequence = match reader.read_u8()? {
            0 => None,
            1 => Some(InputSequence(reader.read_u64()?)),
            _ => return Err(RuntimeCheckpointError::Truncated),
        };
        let next_transaction_id = reader.read_u64()?;
        let atom_count = reader.read_len()?;
        if atom_count > MAX_RUNTIME_CHECKPOINT_ATOMS {
            return Err(RuntimeCheckpointError::AtomLimitExceeded {
                entries: atom_count,
                limit: MAX_RUNTIME_CHECKPOINT_ATOMS,
            });
        }
        let mut atoms = BTreeMap::new();
        let mut previous_atom: Option<AtomSlot> = None;
        for _ in 0..atom_count {
            let slot = AtomSlot(reader.read_u32()?);
            if previous_atom.is_some_and(|previous| slot <= previous) {
                return Err(RuntimeCheckpointError::DuplicateOrUnorderedAtom(slot));
            }
            let atom_id = reader.read_u64()?;
            if atom_id == 0 {
                return Err(RuntimeCheckpointError::InvalidAtomId(atom_id));
            }
            let version = reader.read_u64()?;
            let value = reader.read_value()?;
            atoms.insert(
                slot,
                RuntimeAtomSnapshot {
                    id: crate::atom::AtomId(atom_id),
                    version,
                    value,
                },
            );
            previous_atom = Some(slot);
        }
        let render_count = reader.read_len()?;
        if render_count > MAX_RUNTIME_CHECKPOINT_ATOMS {
            return Err(RuntimeCheckpointError::AtomLimitExceeded {
                entries: render_count,
                limit: MAX_RUNTIME_CHECKPOINT_ATOMS,
            });
        }
        let mut render_revisions = Vec::with_capacity(render_count);
        let mut previous_render: Option<RenderNodeSlot> = None;
        for _ in 0..render_count {
            let slot = RenderNodeSlot(reader.read_u32()?);
            if previous_render.is_some_and(|previous| slot <= previous) {
                return Err(RuntimeCheckpointError::InternalState(
                    "duplicate or unordered render slot".into(),
                ));
            }
            render_revisions.push(RenderRevisionCheckpoint {
                slot,
                revision: reader.read_u64()?,
            });
            previous_render = Some(slot);
        }
        let runtime = PersistentRuntimeCheckpointState {
            tick,
            replay_state: runtime_replay_state,
            last_input_sequence,
            next_transaction_id,
            atoms,
            render_revisions,
        };

        let now = LogicalTime(reader.read_u64()?);
        let next_timer_id = reader.read_u64()?;
        let fire_budget_u64 = reader.read_u64()?;
        let fire_budget = usize::try_from(fire_budget_u64).map_err(|_| {
            RuntimeCheckpointError::InternalState("timer fire budget does not fit usize".into())
        })?;
        let timer_count = reader.read_len()?;
        if timer_count > MAX_RUNTIME_CHECKPOINT_TIMERS {
            return Err(RuntimeCheckpointError::TimerLimitExceeded {
                entries: timer_count,
                limit: MAX_RUNTIME_CHECKPOINT_TIMERS,
            });
        }
        let mut timers = Vec::with_capacity(timer_count);
        let mut previous_timer = 0_u64;
        for _ in 0..timer_count {
            let id = reader.read_u64()?;
            if id == 0 || id <= previous_timer {
                return Err(RuntimeCheckpointError::DuplicateOrUnorderedTimer(id));
            }
            let next_deadline = LogicalTime(reader.read_u64()?);
            let interval = match reader.read_u8()? {
                0 => None,
                1 => Some(LogicalDuration(reader.read_u64()?)),
                _ => return Err(RuntimeCheckpointError::Truncated),
            };
            let occurrences = reader.read_u64()?;
            timers.push(TimerSnapshot {
                id: TimerId(id),
                next_deadline,
                interval,
                occurrences,
            });
            previous_timer = id;
        }
        let time = TimeCheckpointState {
            now,
            next_timer_id,
            fire_budget,
            timers,
        };

        let source_count = reader.read_len()?;
        if source_count > MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES {
            return Err(RuntimeCheckpointError::CompletionSourceLimitExceeded {
                entries: source_count,
                limit: MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES,
            });
        }
        let mut last_sequences = BTreeMap::new();
        let mut previous_source = 0_u64;
        for _ in 0..source_count {
            let source_value = reader.read_u64()?;
            if source_value == 0 || source_value <= previous_source {
                return Err(
                    RuntimeCheckpointError::DuplicateOrUnorderedCompletionSource(source_value),
                );
            }
            let sequence_value = reader.read_u64()?;
            last_sequences.insert(
                EffectCompletionSourceId(source_value),
                EffectCompletionSequence(sequence_value),
            );
            previous_source = source_value;
        }
        let delivery_count = reader.read_len()?;
        if delivery_count > MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES {
            return Err(RuntimeCheckpointError::CompletedDeliveryLimitExceeded {
                entries: delivery_count,
                limit: MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES,
            });
        }
        let mut completed_deliveries = BTreeSet::new();
        let mut previous_delivery: Option<EffectDeliveryKey> = None;
        for _ in 0..delivery_count {
            let key = EffectDeliveryKey::new(
                EffectDeliveryNamespace::new(reader.read_array_16()?),
                EffectIntentId(reader.read_u64()?),
            );
            if previous_delivery.is_some_and(|previous| key <= previous) {
                return Err(RuntimeCheckpointError::DuplicateOrUnorderedDelivery);
            }
            completed_deliveries.insert(key);
            previous_delivery = Some(key);
        }
        if !reader.is_finished() {
            return Err(RuntimeCheckpointError::Truncated);
        }

        Self::from_parts(
            namespace,
            program_hash,
            program_epoch,
            upgrade_chain_root,
            last_upgrade,
            audit_events,
            audit_root,
            effect_next_intent_id,
            cycle,
            event_replay_state,
            runtime,
            time,
            CompletionCheckpointState {
                last_sequences,
                completed_deliveries,
            },
        )
    }

    fn validate(&self) -> RuntimeCheckpointResult<()> {
        if self.program_epoch.0 == 0 {
            if self.upgrade_chain_root != RuntimeUpgradeHash::ZERO || self.last_upgrade.is_some() {
                return Err(RuntimeCheckpointError::InvalidUpgradeLineage);
            }
        } else {
            let record = self
                .last_upgrade
                .as_ref()
                .ok_or(RuntimeCheckpointError::InvalidUpgradeLineage)?;
            if self.upgrade_chain_root == RuntimeUpgradeHash::ZERO
                || record.target_program_hash != self.program_hash
            {
                return Err(RuntimeCheckpointError::InvalidUpgradeLineage);
            }
        }
        if self.effect_next_intent_id == 0 {
            return Err(RuntimeCheckpointError::InvalidEffectNextIntentId(
                self.effect_next_intent_id,
            ));
        }
        if self.cycle != self.runtime.tick {
            return Err(RuntimeCheckpointError::CycleTickMismatch {
                cycle: self.cycle,
                tick: self.runtime.tick,
            });
        }
        if self.audit_events == 0 && self.audit_root != EffectAuditHash::ZERO {
            return Err(RuntimeCheckpointError::InvalidAuditPrefix);
        }
        if self.audit_events > 0 && self.audit_root == EffectAuditHash::ZERO {
            return Err(RuntimeCheckpointError::InvalidAuditPrefix);
        }
        if self.runtime.next_transaction_id == 0 {
            return Err(RuntimeCheckpointError::InvalidNextTransactionId(
                self.runtime.next_transaction_id,
            ));
        }
        if let Some(sequence) = self.runtime.last_input_sequence {
            if sequence.0 == 0 {
                return Err(RuntimeCheckpointError::InvalidLastInputSequence(sequence.0));
            }
        }
        if self.runtime.atoms.len() > MAX_RUNTIME_CHECKPOINT_ATOMS {
            return Err(RuntimeCheckpointError::AtomLimitExceeded {
                entries: self.runtime.atoms.len(),
                limit: MAX_RUNTIME_CHECKPOINT_ATOMS,
            });
        }
        if self.runtime.render_revisions.len() > MAX_RUNTIME_CHECKPOINT_ATOMS {
            return Err(RuntimeCheckpointError::AtomLimitExceeded {
                entries: self.runtime.render_revisions.len(),
                limit: MAX_RUNTIME_CHECKPOINT_ATOMS,
            });
        }
        for snapshot in self.runtime.atoms.values() {
            if snapshot.id.0 == 0 {
                return Err(RuntimeCheckpointError::InvalidAtomId(snapshot.id.0));
            }
            validate_value(&snapshot.value)?;
        }
        if self.time.fire_budget == 0 {
            return Err(RuntimeCheckpointError::InternalState(
                "timer fire budget is zero".into(),
            ));
        }
        if self.time.next_timer_id == 0 {
            return Err(RuntimeCheckpointError::InvalidNextTimerId(
                self.time.next_timer_id,
            ));
        }
        if self.time.timers.len() > MAX_RUNTIME_CHECKPOINT_TIMERS {
            return Err(RuntimeCheckpointError::TimerLimitExceeded {
                entries: self.time.timers.len(),
                limit: MAX_RUNTIME_CHECKPOINT_TIMERS,
            });
        }
        let mut max_timer = 0_u64;
        for timer in &self.time.timers {
            if timer.id.0 == 0 || timer.id.0 <= max_timer {
                return Err(RuntimeCheckpointError::DuplicateOrUnorderedTimer(
                    timer.id.0,
                ));
            }
            if timer.next_deadline < self.time.now {
                return Err(RuntimeCheckpointError::TimerDeadlineBeforeLogicalTime {
                    timer: timer.id.0,
                    deadline: timer.next_deadline.0,
                    logical_time: self.time.now.0,
                });
            }
            if timer.interval.is_some_and(|interval| interval.is_zero()) {
                return Err(RuntimeCheckpointError::InvalidTimerInterval);
            }
            max_timer = timer.id.0;
        }
        if self.time.next_timer_id <= max_timer {
            return Err(RuntimeCheckpointError::InvalidNextTimerId(
                self.time.next_timer_id,
            ));
        }
        if self.completions.last_sequences.len() > MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES {
            return Err(RuntimeCheckpointError::CompletionSourceLimitExceeded {
                entries: self.completions.last_sequences.len(),
                limit: MAX_RUNTIME_CHECKPOINT_COMPLETION_SOURCES,
            });
        }
        for (source, sequence) in &self.completions.last_sequences {
            if source.0 == 0 {
                return Err(RuntimeCheckpointError::InvalidCompletionSource(source.0));
            }
            if sequence.0 == 0 {
                return Err(RuntimeCheckpointError::InvalidCompletionSequence(
                    sequence.0,
                ));
            }
        }
        if self.completions.completed_deliveries.len() > MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES
        {
            return Err(RuntimeCheckpointError::CompletedDeliveryLimitExceeded {
                entries: self.completions.completed_deliveries.len(),
                limit: MAX_RUNTIME_CHECKPOINT_COMPLETED_DELIVERIES,
            });
        }
        for delivery in &self.completions.completed_deliveries {
            if delivery.intent.0 == 0 {
                return Err(RuntimeCheckpointError::InvalidEffectIntentId(
                    delivery.intent.0,
                ));
            }
        }
        Ok(())
    }
}

fn checkpoint_digest(payload: &[u8]) -> [u8; 32] {
    checkpoint_digest_with_domain(payload, CHECKPOINT_HASH_DOMAIN_V1_1)
}

fn checkpoint_digest_for_minor(payload: &[u8], minor: u16) -> [u8; 32] {
    let domain = if minor == 0 {
        CHECKPOINT_HASH_DOMAIN_V1_0
    } else {
        CHECKPOINT_HASH_DOMAIN_V1_1
    };
    checkpoint_digest_with_domain(payload, domain)
}

fn checkpoint_digest_with_domain(payload: &[u8], domain: &[u8]) -> [u8; 32] {
    let mut input = Vec::with_capacity(domain.len() + payload.len());
    input.extend_from_slice(domain);
    input.extend_from_slice(payload);
    sha256(&input)
}

fn validate_value(value: &Value) -> RuntimeCheckpointResult<()> {
    match value {
        Value::Float(value) if !value.is_finite() => Err(RuntimeCheckpointError::NonFiniteFloat),
        Value::Text(value) if value.len() > MAX_RUNTIME_CHECKPOINT_TEXT_BYTES => {
            Err(RuntimeCheckpointError::TextTooLarge {
                bytes: value.len(),
                limit: MAX_RUNTIME_CHECKPOINT_TEXT_BYTES,
            })
        }
        _ => Ok(()),
    }
}

fn push_len(bytes: &mut Vec<u8>, len: usize) -> RuntimeCheckpointResult<()> {
    let len = u64::try_from(len).map_err(|_| {
        RuntimeCheckpointError::InternalState("collection length does not fit u64".into())
    })?;
    bytes.extend_from_slice(&len.to_le_bytes());
    Ok(())
}

fn push_value(bytes: &mut Vec<u8>, value: &Value) -> RuntimeCheckpointResult<()> {
    validate_value(value)?;
    match value {
        Value::Null => bytes.push(0x00),
        Value::Bool(false) => bytes.push(0x01),
        Value::Bool(true) => bytes.push(0x02),
        Value::Int(value) => {
            bytes.push(0x03);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Value::Float(value) => {
            bytes.push(0x04);
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        Value::Text(value) => {
            bytes.push(0x05);
            push_len(bytes, value.len())?;
            bytes.extend_from_slice(value.as_bytes());
        }
    }
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    fn is_finished(&self) -> bool {
        self.cursor == self.bytes.len()
    }

    fn read_exact(&mut self, len: usize) -> RuntimeCheckpointResult<&'a [u8]> {
        let end = self
            .cursor
            .checked_add(len)
            .ok_or(RuntimeCheckpointError::Truncated)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or(RuntimeCheckpointError::Truncated)?;
        self.cursor = end;
        Ok(value)
    }

    fn read_u8(&mut self) -> RuntimeCheckpointResult<u8> {
        Ok(self.read_exact(1)?[0])
    }
    fn read_u16(&mut self) -> RuntimeCheckpointResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_exact(2)?
                .try_into()
                .map_err(|_| RuntimeCheckpointError::Truncated)?,
        ))
    }
    fn read_u32(&mut self) -> RuntimeCheckpointResult<u32> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?
                .try_into()
                .map_err(|_| RuntimeCheckpointError::Truncated)?,
        ))
    }
    fn read_u64(&mut self) -> RuntimeCheckpointResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?
                .try_into()
                .map_err(|_| RuntimeCheckpointError::Truncated)?,
        ))
    }
    fn read_i64(&mut self) -> RuntimeCheckpointResult<i64> {
        Ok(i64::from_le_bytes(
            self.read_exact(8)?
                .try_into()
                .map_err(|_| RuntimeCheckpointError::Truncated)?,
        ))
    }
    fn read_array_16(&mut self) -> RuntimeCheckpointResult<[u8; 16]> {
        self.read_exact(16)?
            .try_into()
            .map_err(|_| RuntimeCheckpointError::Truncated)
    }
    fn read_array_32(&mut self) -> RuntimeCheckpointResult<[u8; 32]> {
        self.read_exact(32)?
            .try_into()
            .map_err(|_| RuntimeCheckpointError::Truncated)
    }

    fn read_len(&mut self) -> RuntimeCheckpointResult<usize> {
        let raw = self.read_u64()?;
        usize::try_from(raw).map_err(|_| RuntimeCheckpointError::Truncated)
    }

    fn read_value(&mut self) -> RuntimeCheckpointResult<Value> {
        let value = match self.read_u8()? {
            0x00 => Value::Null,
            0x01 => Value::Bool(false),
            0x02 => Value::Bool(true),
            0x03 => Value::Int(self.read_i64()?),
            0x04 => {
                let value = f64::from_bits(self.read_u64()?);
                if !value.is_finite() {
                    return Err(RuntimeCheckpointError::NonFiniteFloat);
                }
                Value::Float(value)
            }
            0x05 => {
                let len = self.read_len()?;
                if len > MAX_RUNTIME_CHECKPOINT_TEXT_BYTES {
                    return Err(RuntimeCheckpointError::TextTooLarge {
                        bytes: len,
                        limit: MAX_RUNTIME_CHECKPOINT_TEXT_BYTES,
                    });
                }
                let text = std::str::from_utf8(self.read_exact(len)?)
                    .map_err(|_| RuntimeCheckpointError::InvalidUtf8)?;
                Value::Text(text.to_owned())
            }
            tag => return Err(RuntimeCheckpointError::UnknownValueTag(tag)),
        };
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program_upgrade::{ProgramEpoch, RuntimeUpgradeHash, RuntimeUpgradeLineageRecord};

    fn minimal_checkpoint() -> RuntimeSemanticCheckpoint {
        RuntimeSemanticCheckpoint::from_parts(
            EffectDeliveryNamespace::new([7; 16]),
            [3; 32],
            ProgramEpoch(0),
            RuntimeUpgradeHash::ZERO,
            None,
            0,
            EffectAuditHash::ZERO,
            1,
            0,
            11,
            PersistentRuntimeCheckpointState {
                tick: 0,
                replay_state: 13,
                last_input_sequence: None,
                next_transaction_id: 1,
                atoms: BTreeMap::new(),
                render_revisions: Vec::new(),
            },
            TimeCheckpointState {
                now: LogicalTime::ZERO,
                next_timer_id: 1,
                fire_budget: 1,
                timers: Vec::new(),
            },
            CompletionCheckpointState::default(),
        )
        .unwrap()
    }

    #[test]
    fn legacy_k114_minor_zero_checkpoint_decodes_as_epoch_zero() {
        let current = minimal_checkpoint().canonical_bytes().unwrap();
        let current_payload = &current[..current.len() - 32];
        const EXTENSION_START: usize = 60;
        const EPOCH_ZERO_EXTENSION_LEN: usize = 41;
        let mut legacy = Vec::new();
        legacy.extend_from_slice(&current_payload[..EXTENSION_START]);
        legacy[10..12].copy_from_slice(&0_u16.to_le_bytes());
        legacy.extend_from_slice(&current_payload[EXTENSION_START + EPOCH_ZERO_EXTENSION_LEN..]);
        let digest = checkpoint_digest_with_domain(&legacy, CHECKPOINT_HASH_DOMAIN_V1_0);
        legacy.extend_from_slice(&digest);

        let decoded = RuntimeSemanticCheckpoint::from_canonical_bytes(&legacy).unwrap();
        assert_eq!(decoded.program_epoch(), ProgramEpoch(0));
        assert_eq!(decoded.upgrade_chain_root(), RuntimeUpgradeHash::ZERO);
        assert!(decoded.last_upgrade().is_none());
        assert_ne!(decoded.canonical_bytes().unwrap(), legacy);
    }

    #[test]
    fn upgraded_lineage_round_trip_is_byte_stable() {
        let record = RuntimeUpgradeLineageRecord {
            source_program_hash: [1; 32],
            target_program_hash: [2; 32],
            plan_hash: RuntimeUpgradeHash([3; 32]),
            source_checkpoint_hash: [4; 32],
        };
        let checkpoint = RuntimeSemanticCheckpoint::from_parts(
            EffectDeliveryNamespace::new([8; 16]),
            [2; 32],
            ProgramEpoch(1),
            RuntimeUpgradeHash([5; 32]),
            Some(record.clone()),
            0,
            EffectAuditHash::ZERO,
            1,
            0,
            17,
            PersistentRuntimeCheckpointState {
                tick: 0,
                replay_state: 19,
                last_input_sequence: None,
                next_transaction_id: 1,
                atoms: BTreeMap::new(),
                render_revisions: Vec::new(),
            },
            TimeCheckpointState {
                now: LogicalTime::ZERO,
                next_timer_id: 1,
                fire_budget: 1,
                timers: Vec::new(),
            },
            CompletionCheckpointState::default(),
        )
        .unwrap();
        let bytes = checkpoint.canonical_bytes().unwrap();
        let decoded = RuntimeSemanticCheckpoint::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
        assert_eq!(decoded.program_epoch(), ProgramEpoch(1));
        assert_eq!(decoded.last_upgrade(), Some(&record));
    }
}
