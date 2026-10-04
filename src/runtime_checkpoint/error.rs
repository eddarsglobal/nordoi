use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    effect_audit::{EffectAuditError, EffectAuditHash},
    effect_dispatch::EffectDeliveryNamespace,
    effect_fencing::EffectFencingError,
    nair::AtomSlot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCheckpointStoreError {
    message: String,
}

impl RuntimeCheckpointStoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for RuntimeCheckpointStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RuntimeCheckpointStoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCheckpointError {
    Store(RuntimeCheckpointStoreError),
    Fencing(EffectFencingError),
    Audit(EffectAuditError),
    InvalidMagic,
    UnsupportedVersion {
        major: u16,
        minor: u16,
    },
    Truncated,
    CheckpointTooLarge {
        bytes: usize,
        limit: usize,
    },
    TextTooLarge {
        bytes: usize,
        limit: usize,
    },
    AtomLimitExceeded {
        entries: usize,
        limit: usize,
    },
    TimerLimitExceeded {
        entries: usize,
        limit: usize,
    },
    CompletionSourceLimitExceeded {
        entries: usize,
        limit: usize,
    },
    CompletedDeliveryLimitExceeded {
        entries: usize,
        limit: usize,
    },
    DigestMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    InvalidUtf8,
    UnknownValueTag(u8),
    NonFiniteFloat,
    DuplicateOrUnorderedAtom(AtomSlot),
    InvalidAtomId(u64),
    DuplicateOrUnorderedTimer(u64),
    DuplicateOrUnorderedCompletionSource(u64),
    DuplicateOrUnorderedDelivery,
    InvalidNextTransactionId(u64),
    InvalidNextTimerId(u64),
    InvalidCompletionSource(u64),
    InvalidCompletionSequence(u64),
    InvalidEffectIntentId(u64),
    InvalidLastInputSequence(u64),
    InvalidTimerInterval,
    TimerDeadlineBeforeLogicalTime {
        timer: u64,
        deadline: u64,
        logical_time: u64,
    },
    InvalidAuditPrefix,
    InvalidEffectNextIntentId(u64),
    InvalidUpgradeLineage,
    CycleTickMismatch {
        cycle: u64,
        tick: u64,
    },
    NamespaceMismatch {
        expected: EffectDeliveryNamespace,
        actual: EffectDeliveryNamespace,
    },
    ProgramMismatch,
    AuditHistoryTooShort {
        required: u64,
        actual: u64,
    },
    AuditPrefixMismatch {
        expected: EffectAuditHash,
        actual: EffectAuditHash,
    },
    EffectIntentSequenceMismatch {
        expected_next: u64,
        actual_next: u64,
    },
    RecoveryAfterCycleStarted {
        cycle: u64,
    },
    RecoveryRequired,
    RecoveryBundleIncomplete,
    AtomShapeMismatch,
    RenderShapeMismatch,
    FireBudgetMismatch {
        expected: usize,
        actual: usize,
    },
    InternalState(String),
}

impl Display for RuntimeCheckpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(f, "runtime checkpoint store failure: {error}"),
            Self::Fencing(error) => write!(f, "runtime checkpoint fencing failure: {error}"),
            Self::Audit(error) => write!(f, "runtime checkpoint audit failure: {error}"),
            Self::InvalidMagic => write!(f, "invalid runtime checkpoint magic"),
            Self::UnsupportedVersion { major, minor } => write!(f, "unsupported runtime checkpoint version {major}.{minor}"),
            Self::Truncated => write!(f, "truncated runtime checkpoint"),
            Self::CheckpointTooLarge { bytes, limit } => write!(f, "runtime checkpoint is {bytes} bytes, limit is {limit} bytes"),
            Self::TextTooLarge { bytes, limit } => write!(f, "runtime checkpoint text is {bytes} bytes, limit is {limit} bytes"),
            Self::AtomLimitExceeded { entries, limit } => write!(f, "runtime checkpoint has {entries} atoms, limit is {limit}"),
            Self::TimerLimitExceeded { entries, limit } => write!(f, "runtime checkpoint has {entries} timers, limit is {limit}"),
            Self::CompletionSourceLimitExceeded { entries, limit } => write!(f, "runtime checkpoint has {entries} completion sources, limit is {limit}"),
            Self::CompletedDeliveryLimitExceeded { entries, limit } => write!(f, "runtime checkpoint has {entries} completed deliveries, limit is {limit}"),
            Self::DigestMismatch { .. } => write!(f, "runtime checkpoint SHA-256 digest mismatch"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in runtime checkpoint"),
            Self::UnknownValueTag(tag) => write!(f, "unknown runtime checkpoint value tag 0x{tag:02x}"),
            Self::NonFiniteFloat => write!(f, "runtime checkpoint contains non-finite floating-point state"),
            Self::DuplicateOrUnorderedAtom(slot) => write!(f, "duplicate or unordered runtime atom slot {}", slot.0),
            Self::InvalidAtomId(value) => write!(f, "runtime checkpoint atom id must be non-zero, got {value}"),
            Self::DuplicateOrUnorderedTimer(timer) => write!(f, "duplicate or unordered runtime timer {timer}"),
            Self::DuplicateOrUnorderedCompletionSource(source) => write!(f, "duplicate or unordered completion source {source}"),
            Self::DuplicateOrUnorderedDelivery => write!(f, "duplicate or unordered completed delivery key"),
            Self::InvalidNextTransactionId(value) => write!(f, "next transaction id must be non-zero, got {value}"),
            Self::InvalidNextTimerId(value) => write!(f, "next timer id must be non-zero and exceed every retained timer id, got {value}"),
            Self::InvalidCompletionSource(value) => write!(f, "completion source id must be non-zero, got {value}"),
            Self::InvalidCompletionSequence(value) => write!(f, "completion source sequence must be non-zero, got {value}"),
            Self::InvalidEffectIntentId(value) => write!(f, "effect intent id must be non-zero, got {value}"),
            Self::InvalidLastInputSequence(value) => write!(f, "last input sequence must be non-zero, got {value}"),
            Self::InvalidTimerInterval => write!(f, "repeating timer interval must be non-zero in runtime checkpoint"),
            Self::TimerDeadlineBeforeLogicalTime { timer, deadline, logical_time } => write!(f, "timer {timer} deadline {deadline} is before recovered logical time {logical_time}"),
            Self::InvalidAuditPrefix => write!(f, "runtime checkpoint audit prefix is invalid"),
            Self::InvalidEffectNextIntentId(value) => write!(f, "effect next intent id must be non-zero, got {value}"),
            Self::InvalidUpgradeLineage => write!(f, "runtime checkpoint program-upgrade lineage is invalid"),
            Self::CycleTickMismatch { cycle, tick } => write!(f, "runtime checkpoint cycle {cycle} does not equal runtime tick {tick}"),
            Self::NamespaceMismatch { expected, actual } => write!(f, "runtime checkpoint namespace mismatch: expected {expected}, actual {actual}"),
            Self::ProgramMismatch => write!(f, "runtime checkpoint was produced by a different canonical NAIR program"),
            Self::AuditHistoryTooShort { required, actual } => write!(f, "effect audit history has {actual} records but runtime checkpoint requires prefix length {required}"),
            Self::AuditPrefixMismatch { .. } => write!(f, "effect audit history does not descend from the runtime checkpoint audit prefix"),
            Self::EffectIntentSequenceMismatch { expected_next, actual_next } => write!(f, "effect intent identity frontier mismatch: runtime checkpoint expects next id {expected_next}, journal has {actual_next}"),
            Self::RecoveryAfterCycleStarted { cycle } => write!(f, "whole-runtime recovery is bootstrap-only; event loop is already at cycle {cycle}"),
            Self::RecoveryRequired => write!(f, "an existing durable runtime bundle must be recovered before it can be replaced"),
            Self::RecoveryBundleIncomplete => write!(f, "runtime/effect checkpoint bundle is incomplete"),
            Self::AtomShapeMismatch => write!(f, "runtime checkpoint atom slots do not match the booted program"),
            Self::RenderShapeMismatch => write!(f, "runtime checkpoint render slots do not match the booted program"),
            Self::FireBudgetMismatch { expected, actual } => write!(f, "runtime checkpoint timer fire budget {actual} does not match boot configuration {expected}"),
            Self::InternalState(message) => write!(f, "runtime checkpoint state rejected: {message}"),
        }
    }
}

impl Error for RuntimeCheckpointError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Fencing(error) => Some(error),
            Self::Audit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RuntimeCheckpointStoreError> for RuntimeCheckpointError {
    fn from(value: RuntimeCheckpointStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<EffectFencingError> for RuntimeCheckpointError {
    fn from(value: EffectFencingError) -> Self {
        Self::Fencing(value)
    }
}

impl From<EffectAuditError> for RuntimeCheckpointError {
    fn from(value: EffectAuditError) -> Self {
        Self::Audit(value)
    }
}

pub type RuntimeCheckpointResult<T> = Result<T, RuntimeCheckpointError>;
