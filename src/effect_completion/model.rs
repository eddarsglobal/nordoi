use crate::{
    atom::AtomId,
    effect_audit::EffectAttemptId,
    effect_dispatch::{EffectDeliveryKey, EffectDeliveryNamespace},
    ownership::DomainId,
    value::Value,
};

use super::{EffectCompletionSequence, EffectCompletionSourceId};

pub const MAX_EFFECT_COMPLETION_BATCH: usize = 4_096;
pub const MAX_EFFECT_COMPLETION_TEXT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq)]
pub enum EffectCompletionOutcome {
    Success(Value),
    Failure(Value),
}

impl EffectCompletionOutcome {
    pub fn success(value: impl Into<Value>) -> Self {
        Self::Success(value.into())
    }

    pub fn failure(value: impl Into<Value>) -> Self {
        Self::Failure(value.into())
    }

    pub fn value(&self) -> &Value {
        match self {
            Self::Success(value) | Self::Failure(value) => value,
        }
    }

    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Success(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCompletion {
    pub source: EffectCompletionSourceId,
    pub sequence: EffectCompletionSequence,
    pub delivery_key: EffectDeliveryKey,
    pub attempt: EffectAttemptId,
    pub outcome: EffectCompletionOutcome,
}

impl EffectCompletion {
    pub fn new(
        source: EffectCompletionSourceId,
        sequence: EffectCompletionSequence,
        delivery_key: EffectDeliveryKey,
        attempt: EffectAttemptId,
        outcome: EffectCompletionOutcome,
    ) -> Self {
        Self {
            source,
            sequence,
            delivery_key,
            attempt,
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectCompletionBatch {
    pub completions: Vec<EffectCompletion>,
}

impl EffectCompletionBatch {
    pub fn new(completions: Vec<EffectCompletion>) -> Self {
        Self { completions }
    }

    pub fn push(&mut self, completion: EffectCompletion) {
        self.completions.push(completion);
    }

    pub fn len(&self) -> usize {
        self.completions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.completions.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCompletionProjectionValue {
    OutcomeValue,
    Succeeded,
    IntentId,
    AttemptId,
    SourceSequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectCompletionProjection {
    pub source: EffectCompletionSourceId,
    pub namespace: EffectDeliveryNamespace,
    pub domain: DomainId,
    pub atom: AtomId,
    pub value: EffectCompletionProjectionValue,
}

impl EffectCompletionProjection {
    pub const fn new(
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
        domain: DomainId,
        atom: AtomId,
        value: EffectCompletionProjectionValue,
    ) -> Self {
        Self {
            source,
            namespace,
            domain,
            atom,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCompletionWriteReport {
    pub domain: DomainId,
    pub atom: AtomId,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCompletionApplicationReport {
    pub source: EffectCompletionSourceId,
    pub sequence: EffectCompletionSequence,
    pub delivery_key: EffectDeliveryKey,
    pub attempt: EffectAttemptId,
    pub outcome: EffectCompletionOutcome,
    pub writes: Vec<EffectCompletionWriteReport>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectCompletionBatchReport {
    pub causes: usize,
    pub accepted: usize,
    pub transactions: usize,
    pub staged_writes: usize,
    pub changed_atoms: usize,
    pub applications: Vec<EffectCompletionApplicationReport>,
}
