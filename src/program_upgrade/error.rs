use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::nair::AtomSlot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeUpgradeError {
    SameProgram,
    UnauthorizedTransition,
    SourceProgramMismatch,
    TargetProgramMismatch,
    SourceEpochMismatch {
        expected: u64,
        actual: u64,
    },
    ProgramEpochExhausted,
    UpgradeRequiresDurableRuntimeBinding,
    PlanRuleLimitExceeded {
        rules: usize,
        limit: usize,
    },
    PendingTimersUnsupported {
        count: usize,
    },
    TargetTimerDeadlineBeforeUpgradeTime {
        timer: u64,
        deadline: u64,
        logical_time: u64,
    },
    DuplicateSourceAtomDisposition(AtomSlot),
    DuplicateTargetAtomDisposition(AtomSlot),
    UnknownSourceAtom(AtomSlot),
    UnknownTargetAtom(AtomSlot),
    MissingSourceAtomDisposition(AtomSlot),
    MissingTargetAtomDisposition(AtomSlot),
    UpgradeHashMismatch,
    AtomVersionExhausted(AtomSlot),
    InternalState(String),
}

impl Display for RuntimeUpgradeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SameProgram => write!(f, "runtime upgrade source and target programs are identical"),
            Self::UnauthorizedTransition => write!(f, "runtime program upgrade transition is not explicitly authorized"),
            Self::SourceProgramMismatch => write!(f, "runtime upgrade source program hash does not match the live source program"),
            Self::TargetProgramMismatch => write!(f, "runtime upgrade target program hash does not match the booted target program"),
            Self::SourceEpochMismatch { expected, actual } => write!(f, "runtime upgrade source epoch mismatch: plan expects {expected}, live epoch is {actual}"),
            Self::ProgramEpochExhausted => write!(f, "runtime program epoch space is exhausted"),
            Self::UpgradeRequiresDurableRuntimeBinding => write!(f, "runtime upgrade requires an established K1.14 durable runtime lineage"),
            Self::PlanRuleLimitExceeded { rules, limit } => write!(f, "runtime upgrade plan has {rules} rules, limit is {limit}"),
            Self::PendingTimersUnsupported { count } => write!(f, "K1.15 upgrade-safe boundary requires zero pending source timers; found {count}"),
            Self::TargetTimerDeadlineBeforeUpgradeTime { timer, deadline, logical_time } => write!(f, "target timer {timer} deadline {deadline} is before upgrade logical time {logical_time}"),
            Self::DuplicateSourceAtomDisposition(slot) => write!(f, "source atom slot {} has more than one upgrade disposition", slot.0),
            Self::DuplicateTargetAtomDisposition(slot) => write!(f, "target atom slot {} has more than one upgrade disposition", slot.0),
            Self::UnknownSourceAtom(slot) => write!(f, "runtime upgrade references unknown source atom slot {}", slot.0),
            Self::UnknownTargetAtom(slot) => write!(f, "runtime upgrade references unknown target atom slot {}", slot.0),
            Self::MissingSourceAtomDisposition(slot) => write!(f, "source atom slot {} has no explicit copy/drop disposition", slot.0),
            Self::MissingTargetAtomDisposition(slot) => write!(f, "target atom slot {} has no explicit copy/default disposition", slot.0),
            Self::UpgradeHashMismatch => write!(f, "runtime upgrade plan canonical hash mismatch"),
            Self::AtomVersionExhausted(slot) => write!(f, "runtime upgrade atom version exhausted for target slot {}", slot.0),
            Self::InternalState(message) => write!(f, "runtime upgrade state rejected: {message}"),
        }
    }
}

impl Error for RuntimeUpgradeError {}

pub type RuntimeUpgradeResult<T> = Result<T, RuntimeUpgradeError>;
