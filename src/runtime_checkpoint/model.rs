use std::collections::{BTreeMap, BTreeSet};

use crate::{
    effect_completion::{EffectCompletionSequence, EffectCompletionSourceId},
    effect_dispatch::EffectDeliveryKey,
    input::InputSequence,
    nair::{AtomSlot, RenderNodeSlot},
    runtime::RuntimeAtomSnapshot,
    time::{LogicalTime, TimerSnapshot},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeCheckpointCommitReceipt {
    pub reference: Option<String>,
}

impl RuntimeCheckpointCommitReceipt {
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
pub struct RuntimeCheckpointRecoveryReport {
    pub cycle: u64,
    pub runtime_tick: u64,
    pub logical_time: LogicalTime,
    pub audit_events: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderRevisionCheckpoint {
    pub slot: RenderNodeSlot,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PersistentRuntimeCheckpointState {
    pub tick: u64,
    pub replay_state: u64,
    pub last_input_sequence: Option<InputSequence>,
    pub next_transaction_id: u64,
    pub atoms: BTreeMap<AtomSlot, RuntimeAtomSnapshot>,
    pub render_revisions: Vec<RenderRevisionCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimeCheckpointState {
    pub now: LogicalTime,
    pub next_timer_id: u64,
    pub fire_budget: usize,
    pub timers: Vec<TimerSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct CompletionCheckpointState {
    pub last_sequences: BTreeMap<EffectCompletionSourceId, EffectCompletionSequence>,
    pub completed_deliveries: BTreeSet<EffectDeliveryKey>,
}
