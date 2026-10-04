use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    nair::{AtomSlot, TimerSlot},
    time::TimerId,
};

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
    TimerPlanRuleLimitExceeded {
        rules: usize,
        limit: usize,
    },
    DynamicTimerPlanRuleLimitExceeded {
        rules: usize,
        limit: usize,
    },
    PendingTimersUnsupported {
        count: usize,
    },
    DynamicSourceTimerUnsupported {
        timer: u64,
    },
    DuplicateDynamicSourceTimerDisposition(TimerId),
    DuplicateDynamicTargetTimerDisposition(TimerId),
    UnknownDynamicSourceTimer(TimerId),
    DynamicSourceTimerIsNative(TimerId),
    MissingDynamicSourceTimerDisposition(TimerId),
    DynamicTargetTimerConflictsWithNative(TimerId),
    DynamicTargetTimerIdentityNotFresh {
        source: TimerId,
        target: TimerId,
        minimum: u64,
    },
    InvalidDynamicTargetTimerId(TimerId),
    DynamicTargetTimerIdentityExhausted(TimerId),
    DynamicTimerUpgradeHashMismatch,
    DynamicTimerPlanSourceProgramMismatch,
    DynamicTimerPlanTargetProgramMismatch,
    DynamicTimerPlanSourceEpochMismatch {
        expected: u64,
        actual: u64,
    },
    DuplicateSourceTimerDisposition(TimerSlot),
    DuplicateTargetTimerDisposition(TimerSlot),
    UnknownSourceTimerSlot(TimerSlot),
    UnknownTargetTimerSlot(TimerSlot),
    MissingSourceTimerDisposition(TimerSlot),
    MissingTargetTimerDisposition(TimerSlot),
    TimerUpgradeHashMismatch,
    TimerPlanSourceProgramMismatch,
    TimerPlanTargetProgramMismatch,
    TimerPlanSourceEpochMismatch {
        expected: u64,
        actual: u64,
    },
    TargetTimerInactive(TimerSlot),
    TimerKindMismatch {
        source: TimerSlot,
        target: TimerSlot,
    },
    TimerIntervalMismatch {
        source: TimerSlot,
        target: TimerSlot,
        source_interval: u64,
        target_interval: u64,
    },
    SourceTimerDeadlineBeforeUpgradeTime {
        timer: u64,
        deadline: u64,
        logical_time: u64,
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
            Self::TimerPlanRuleLimitExceeded { rules, limit } => write!(f, "runtime timer upgrade plan has {rules} rules, limit is {limit}"),
            Self::DynamicTimerPlanRuleLimitExceeded { rules, limit } => write!(f, "runtime dynamic timer upgrade plan has {rules} rules, limit is {limit}"),
            Self::PendingTimersUnsupported { count } => write!(f, "legacy K1.15 upgrade path requires zero pending source timers; found {count}"),
            Self::DynamicSourceTimerUnsupported { timer } => write!(f, "source timer {timer} is pending but is not bound to a native NAIR TimerSlot"),
            Self::DuplicateDynamicSourceTimerDisposition(timer) => write!(f, "dynamic source timer {} has more than one upgrade disposition", timer.0),
            Self::DuplicateDynamicTargetTimerDisposition(timer) => write!(f, "dynamic target timer {} is claimed by more than one source timer", timer.0),
            Self::UnknownDynamicSourceTimer(timer) => write!(f, "runtime dynamic timer upgrade references unknown or inactive source timer {}", timer.0),
            Self::DynamicSourceTimerIsNative(timer) => write!(f, "runtime dynamic timer upgrade references source timer {} that is bound to a native TimerSlot", timer.0),
            Self::MissingDynamicSourceTimerDisposition(timer) => write!(f, "active dynamic source timer {} has no explicit carry/drop disposition", timer.0),
            Self::DynamicTargetTimerConflictsWithNative(timer) => write!(f, "dynamic target timer {} conflicts with a target native TimerSlot identity", timer.0),
            Self::DynamicTargetTimerIdentityNotFresh { source, target, minimum } => write!(f, "dynamic timer remap {} -> {} reuses an old identity; remapped target must be at least {minimum}", source.0, target.0),
            Self::InvalidDynamicTargetTimerId(timer) => write!(f, "dynamic target timer id {} is invalid", timer.0),
            Self::DynamicTargetTimerIdentityExhausted(timer) => write!(f, "dynamic target timer id {} cannot advance the timer allocation frontier", timer.0),
            Self::DynamicTimerUpgradeHashMismatch => write!(f, "runtime dynamic timer upgrade plan canonical hash mismatch"),
            Self::DynamicTimerPlanSourceProgramMismatch => write!(f, "runtime dynamic timer upgrade plan source program does not match the atom migration plan"),
            Self::DynamicTimerPlanTargetProgramMismatch => write!(f, "runtime dynamic timer upgrade plan target program does not match the atom migration plan"),
            Self::DynamicTimerPlanSourceEpochMismatch { expected, actual } => write!(f, "runtime dynamic timer upgrade source epoch mismatch: atom plan expects {expected}, dynamic timer plan has {actual}"),
            Self::DuplicateSourceTimerDisposition(slot) => write!(f, "source timer slot {} has more than one upgrade disposition", slot.0),
            Self::DuplicateTargetTimerDisposition(slot) => write!(f, "target timer slot {} has more than one upgrade disposition", slot.0),
            Self::UnknownSourceTimerSlot(slot) => write!(f, "runtime timer upgrade references unknown source timer slot {}", slot.0),
            Self::UnknownTargetTimerSlot(slot) => write!(f, "runtime timer upgrade references unknown target timer slot {}", slot.0),
            Self::MissingSourceTimerDisposition(slot) => write!(f, "source timer slot {} has no explicit carry/drop disposition", slot.0),
            Self::MissingTargetTimerDisposition(slot) => write!(f, "target timer slot {} has no explicit carry/default disposition", slot.0),
            Self::TimerUpgradeHashMismatch => write!(f, "runtime timer upgrade plan canonical hash mismatch"),
            Self::TimerPlanSourceProgramMismatch => write!(f, "runtime timer upgrade plan source program does not match the atom migration plan"),
            Self::TimerPlanTargetProgramMismatch => write!(f, "runtime timer upgrade plan target program does not match the atom migration plan"),
            Self::TimerPlanSourceEpochMismatch { expected, actual } => write!(f, "runtime timer upgrade source epoch mismatch: atom plan expects {expected}, timer plan has {actual}"),
            Self::TargetTimerInactive(slot) => write!(f, "target timer slot {} is canceled at bootstrap and cannot receive active carried state", slot.0),
            Self::TimerKindMismatch { source, target } => write!(f, "timer kind mismatch while carrying source slot {} to target slot {}", source.0, target.0),
            Self::TimerIntervalMismatch { source, target, source_interval, target_interval } => write!(f, "repeating timer interval mismatch while carrying source slot {} ({source_interval}) to target slot {} ({target_interval})", source.0, target.0),
            Self::SourceTimerDeadlineBeforeUpgradeTime { timer, deadline, logical_time } => write!(f, "source timer {timer} deadline {deadline} is before upgrade logical time {logical_time}"),
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
