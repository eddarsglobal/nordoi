use std::collections::BTreeMap;

use crate::effect_dispatch::{EffectDeliveryKey, EffectIntentId, QueuedEffectIntent};

use super::EffectRetryTick;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectRetryRecord {
    pub intent: EffectIntentId,
    pub failed_attempts: u32,
    pub next_eligible_tick: EffectRetryTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDeadLetterReason {
    PermanentBackendFailure,
    AttemptsExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetteredEffect {
    pub request: QueuedEffectIntent,
    pub delivery_key: EffectDeliveryKey,
    pub failed_attempts: u32,
    pub reason: EffectDeadLetterReason,
    pub last_error: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectRetryLedger {
    retries: BTreeMap<EffectIntentId, EffectRetryRecord>,
    dead_letters: BTreeMap<EffectIntentId, DeadLetteredEffect>,
    last_tick: EffectRetryTick,
}

impl EffectRetryLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_tick(&self) -> EffectRetryTick {
        self.last_tick
    }

    pub fn retry(&self, id: EffectIntentId) -> Option<&EffectRetryRecord> {
        self.retries.get(&id)
    }

    pub fn retries(&self) -> impl Iterator<Item = &EffectRetryRecord> {
        self.retries.values()
    }

    pub fn retry_count(&self) -> usize {
        self.retries.len()
    }

    pub fn dead_letter(&self, id: EffectIntentId) -> Option<&DeadLetteredEffect> {
        self.dead_letters.get(&id)
    }

    pub fn dead_letters(&self) -> impl Iterator<Item = &DeadLetteredEffect> {
        self.dead_letters.values()
    }

    pub fn dead_letter_count(&self) -> usize {
        self.dead_letters.len()
    }

    pub(crate) fn set_last_tick(&mut self, tick: EffectRetryTick) {
        self.last_tick = tick;
    }

    pub(crate) fn set_retry(&mut self, record: EffectRetryRecord) {
        self.retries.insert(record.intent, record);
    }

    pub(crate) fn clear_retry(&mut self, id: EffectIntentId) {
        self.retries.remove(&id);
    }

    pub(crate) fn insert_dead_letter(&mut self, dead: DeadLetteredEffect) {
        self.retries.remove(&dead.request.id);
        self.dead_letters.insert(dead.request.id, dead);
    }

    pub(crate) fn remove_dead_letter(&mut self, id: EffectIntentId) -> Option<DeadLetteredEffect> {
        self.dead_letters.remove(&id)
    }

    pub(crate) fn from_parts(
        last_tick: EffectRetryTick,
        retries: Vec<EffectRetryRecord>,
        dead_letters: Vec<DeadLetteredEffect>,
    ) -> Self {
        Self {
            retries: retries
                .into_iter()
                .map(|record| (record.intent, record))
                .collect(),
            dead_letters: dead_letters
                .into_iter()
                .map(|dead| (dead.request.id, dead))
                .collect(),
            last_tick,
        }
    }
}
