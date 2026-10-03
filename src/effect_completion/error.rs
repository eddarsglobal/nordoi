use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    atom::AtomId,
    effect_audit::EffectAttemptId,
    effect_dispatch::{EffectDeliveryKey, EffectDeliveryNamespace},
    error::AtomicError,
};

use super::{EffectCompletionSequence, EffectCompletionSourceId};

#[derive(Debug, PartialEq)]
pub enum EffectCompletionError {
    Atomic(AtomicError),
    InvalidSourceId,
    InvalidSequence,
    BatchTooLarge {
        actual: usize,
        limit: usize,
    },
    DuplicateSourceSequence {
        source: EffectCompletionSourceId,
        sequence: EffectCompletionSequence,
    },
    UnauthorizedSource {
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    },
    NonMonotonicSequence {
        source: EffectCompletionSourceId,
        previous: EffectCompletionSequence,
        current: EffectCompletionSequence,
    },
    DeliveryAlreadyCompleted(EffectDeliveryKey),
    UnknownAttempt(EffectAttemptId),
    DeliveryKeyMismatch {
        attempt: EffectAttemptId,
        expected: EffectDeliveryKey,
        actual: EffectDeliveryKey,
    },
    AttemptIntentMismatch {
        attempt: EffectAttemptId,
    },
    AttemptNotDelivered(EffectAttemptId),
    NoProjection {
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    },
    DuplicateProjection {
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
        atom: AtomId,
    },
    NonFiniteFloat,
    TextTooLarge {
        actual: usize,
        limit: usize,
    },
    IntegerProjectionOverflow(u64),
    CanonicalLengthOverflow,
}

impl Display for EffectCompletionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atomic(error) => write!(f, "effect completion NAM failure: {error}"),
            Self::InvalidSourceId => write!(f, "effect completion source id must be non-zero"),
            Self::InvalidSequence => write!(f, "effect completion sequence must be non-zero"),
            Self::BatchTooLarge { actual, limit } => write!(
                f,
                "effect completion batch contains {actual} causes, exceeding limit {limit}"
            ),
            Self::DuplicateSourceSequence { source, sequence } => write!(
                f,
                "effect completion batch repeats source {source} sequence {sequence}"
            ),
            Self::UnauthorizedSource { source, namespace } => write!(
                f,
                "effect completion source {source} has no authority for namespace {namespace}"
            ),
            Self::NonMonotonicSequence { source, previous, current } => write!(
                f,
                "effect completion source {source} sequence moved backward or repeated: previous={previous}, current={current}"
            ),
            Self::DeliveryAlreadyCompleted(key) => write!(
                f,
                "effect delivery key {key} already produced a semantic completion"
            ),
            Self::UnknownAttempt(attempt) => write!(
                f,
                "effect completion references unknown audit attempt {attempt}"
            ),
            Self::DeliveryKeyMismatch { attempt, expected, actual } => write!(
                f,
                "effect completion delivery key mismatch for attempt {attempt}: expected {expected}, got {actual}"
            ),
            Self::AttemptIntentMismatch { attempt } => write!(
                f,
                "effect completion intent does not match the prepared audit attempt {attempt}"
            ),
            Self::AttemptNotDelivered(attempt) => write!(
                f,
                "effect completion attempt {attempt} has no delivered or assumed-delivered audit resolution"
            ),
            Self::NoProjection { source, namespace } => write!(
                f,
                "effect completion route source={source}, namespace={namespace} has no semantic projection"
            ),
            Self::DuplicateProjection { source, namespace, atom } => write!(
                f,
                "effect completion route source={source}, namespace={namespace} already projects atom {}",
                atom.0
            ),
            Self::NonFiniteFloat => write!(
                f,
                "effect completion values must use finite floating-point numbers"
            ),
            Self::TextTooLarge { actual, limit } => write!(
                f,
                "effect completion text contains {actual} bytes, exceeding limit {limit}"
            ),
            Self::IntegerProjectionOverflow(value) => write!(
                f,
                "effect completion integer projection {value} does not fit in NORDOI Value::Int"
            ),
            Self::CanonicalLengthOverflow => write!(
                f,
                "effect completion canonical length does not fit in u64"
            ),
        }
    }
}

impl Error for EffectCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Atomic(error) => Some(error),
            _ => None,
        }
    }
}

impl From<AtomicError> for EffectCompletionError {
    fn from(value: AtomicError) -> Self {
        Self::Atomic(value)
    }
}

pub type EffectCompletionResult<T> = Result<T, EffectCompletionError>;
