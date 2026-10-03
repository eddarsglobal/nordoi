use std::collections::BTreeMap;

use crate::reaction::EffectIntent;

use super::{EffectDispatchError, EffectDispatchResult, EffectIntentId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedEffectIntent {
    pub id: EffectIntentId,
    pub cycle: u64,
    pub ordinal: u64,
    pub intent: EffectIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectOutboxStageReport {
    pub enqueued: Vec<QueuedEffectIntent>,
}

impl EffectOutboxStageReport {
    pub fn len(&self) -> usize {
        self.enqueued.len()
    }

    pub fn is_empty(&self) -> bool {
        self.enqueued.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct AtomicEffectOutbox {
    pending: BTreeMap<EffectIntentId, QueuedEffectIntent>,
    next_intent_id: u64,
}

impl Default for AtomicEffectOutbox {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicEffectOutbox {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            next_intent_id: 1,
        }
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn get(&self, id: EffectIntentId) -> Option<&QueuedEffectIntent> {
        self.pending.get(&id)
    }

    pub fn peek(&self) -> Option<&QueuedEffectIntent> {
        self.pending.first_key_value().map(|(_, intent)| intent)
    }

    pub fn iter(&self) -> impl Iterator<Item = &QueuedEffectIntent> {
        self.pending.values()
    }

    pub(crate) fn next_intent_id(&self) -> u64 {
        self.next_intent_id
    }

    pub(crate) fn from_checkpoint_parts(
        pending: Vec<QueuedEffectIntent>,
        next_intent_id: u64,
    ) -> Self {
        let pending = pending
            .into_iter()
            .map(|request| (request.id, request))
            .collect();
        Self {
            pending,
            next_intent_id,
        }
    }

    pub(crate) fn stage_cycle(
        &mut self,
        cycle: u64,
        intents: impl IntoIterator<Item = EffectIntent>,
    ) -> EffectDispatchResult<EffectOutboxStageReport> {
        let intents: Vec<EffectIntent> = intents.into_iter().collect();
        if intents.is_empty() {
            return Ok(EffectOutboxStageReport::default());
        }

        let count =
            u64::try_from(intents.len()).map_err(|_| EffectDispatchError::IntentIdExhausted)?;
        let next_after = self
            .next_intent_id
            .checked_add(count)
            .ok_or(EffectDispatchError::IntentIdExhausted)?;

        let mut staged = Vec::with_capacity(intents.len());
        for (offset, intent) in intents.into_iter().enumerate() {
            let offset =
                u64::try_from(offset).map_err(|_| EffectDispatchError::IntentIdExhausted)?;
            let id = EffectIntentId(
                self.next_intent_id
                    .checked_add(offset)
                    .ok_or(EffectDispatchError::IntentIdExhausted)?,
            );
            staged.push(QueuedEffectIntent {
                id,
                cycle,
                ordinal: offset,
                intent,
            });
        }

        for queued in &staged {
            self.pending.insert(queued.id, queued.clone());
        }
        self.next_intent_id = next_after;

        Ok(EffectOutboxStageReport { enqueued: staged })
    }

    pub(crate) fn acknowledge(
        &mut self,
        id: EffectIntentId,
    ) -> EffectDispatchResult<QueuedEffectIntent> {
        self.pending
            .remove(&id)
            .ok_or(EffectDispatchError::UnknownIntent(id))
    }
}
