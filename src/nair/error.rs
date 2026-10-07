use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    effect::Effect, effect_completion::EffectCompletionError, error::AtomicError,
    input::InputError, reaction::ReactionError, render::RenderError,
};

use super::id::{
    AtomSlot, CompletionSlot, DomainSlot, InputBridgeSlot, ReactionSlot, RegisterId,
    RenderNodeSlot, TimerSlot, TransactionSlot,
};

#[derive(Debug, PartialEq)]
pub enum NairError {
    DuplicateRegister(RegisterId),
    UnknownRegister(RegisterId),
    DuplicateDomainSlot(DomainSlot),
    UnknownDomainSlot(DomainSlot),
    EmptyDomainName(DomainSlot),
    DuplicateAtomSlot(AtomSlot),
    UnknownAtomSlot(AtomSlot),
    SelfDependency(AtomSlot),
    DuplicateTransactionSlot(TransactionSlot),
    InactiveTransaction(TransactionSlot),
    UnclosedTransactions(Vec<TransactionSlot>),
    DuplicateRenderNodeSlot(RenderNodeSlot),
    UnknownRenderNodeSlot(RenderNodeSlot),
    EmptyRenderDirtyMask,
    InvalidRenderOpacity(f32),
    NonFiniteRenderPosition(RenderNodeSlot),
    RenderContextRequired,
    DuplicateInputBridgeSlot(InputBridgeSlot),
    UnknownInputBridgeSlot(InputBridgeSlot),
    InputContextRequired,
    DuplicateTimerSlot(TimerSlot),
    UnknownTimerSlot(TimerSlot),
    ZeroTimerInterval(TimerSlot),
    TimeContextRequired,
    DuplicateCompletionSlot(CompletionSlot),
    EmptyCompletionName(CompletionSlot),
    EmptyCompletionProjections(CompletionSlot),
    DuplicateCompletionProjectionAtom {
        slot: CompletionSlot,
        atom: AtomSlot,
    },
    CompletionBindingRequired(CompletionSlot),
    CompletionContextRequired,
    DuplicateReactionSlot(ReactionSlot),
    EmptyReactionName(ReactionSlot),
    EmptyReactionActionName(ReactionSlot),
    EmptyReactionSteps(ReactionSlot),
    ReactionValueSourceMismatch(ReactionSlot),
    ReactionUndeclaredEffect {
        slot: ReactionSlot,
        effect: Effect,
    },
    NonFiniteReactionLiteral(ReactionSlot),
    ReactionContextRequired,
    MissingHalt,
    InstructionAfterHalt {
        index: usize,
    },
    NonFiniteFloat(RegisterId),
    IntegerAddOperandNotInt(RegisterId),
    IntegerAddOverflow {
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntegerCompareOperandNotInt(RegisterId),
    BranchConditionNotBool(RegisterId),
    BranchArmKindMismatch,
    BranchArmKindUnsupported,
    BranchExpressionOperandNotInt,
    BranchExpressionIntegerOverflow,
    BranchExpressionTooDeep,
    BranchExpressionTooLarge,
    InputEventMissing(u32),
    InputEventNotKeyboardKey(u32),
    InvalidMagic,
    UnsupportedFormat {
        major: u16,
        minor: u16,
    },
    UnexpectedEof,
    InvalidOpcode(u8),
    InvalidValueTag(u8),
    InvalidBranchExpressionTag(u8),
    InvalidDomainRefTag(u8),
    InvalidRenderPrimitiveTag(u8),
    InvalidRenderSpaceTag(u8),
    InvalidDirtyMask(u8),
    InvalidBoolTag(u8),
    InvalidOptionTag(u8),
    InvalidInputSourceTag(u8),
    InvalidInputTargetTag(u8),
    InvalidInputSignalTag(u8),
    InvalidEffectTag(u8),
    InvalidReactionTriggerTag(u8),
    InvalidReactionValueTag(u8),
    InvalidReactionStepTag(u8),
    InvalidCompletionProjectionTag(u8),
    InvalidUtf8,
    TrailingBytes(usize),
    LengthOverflow,
    Kernel(AtomicError),
    Render(RenderError),
    Input(InputError),
    Reaction(ReactionError),
    Completion(EffectCompletionError),
}

impl Display for NairError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRegister(id) => write!(f, "NAIR register {id:?} is defined more than once"),
            Self::UnknownRegister(id) => write!(f, "NAIR register {id:?} is used before definition"),
            Self::DuplicateDomainSlot(id) => write!(f, "NAIR domain slot {id:?} is defined more than once"),
            Self::UnknownDomainSlot(id) => write!(f, "NAIR domain slot {id:?} is used before definition"),
            Self::EmptyDomainName(id) => write!(f, "NAIR domain slot {id:?} has an empty name"),
            Self::DuplicateAtomSlot(id) => write!(f, "NAIR atom slot {id:?} is defined more than once"),
            Self::UnknownAtomSlot(id) => write!(f, "NAIR atom slot {id:?} is used before definition"),
            Self::SelfDependency(id) => write!(f, "NAIR atom slot {id:?} cannot depend on itself"),
            Self::DuplicateTransactionSlot(id) => write!(f, "NAIR transaction slot {id:?} is defined more than once"),
            Self::InactiveTransaction(id) => write!(f, "NAIR transaction slot {id:?} is not active"),
            Self::UnclosedTransactions(ids) => write!(f, "NAIR program halts with active transactions: {ids:?}"),
            Self::DuplicateRenderNodeSlot(id) => write!(f, "NAIR render node slot {id:?} is defined more than once"),
            Self::UnknownRenderNodeSlot(id) => write!(f, "NAIR render node slot {id:?} is used before definition"),
            Self::EmptyRenderDirtyMask => write!(f, "NAIR render binding cannot use an empty dirty mask"),
            Self::InvalidRenderOpacity(value) => write!(f, "NAIR render opacity must be finite and within 0..=1, got {value}"),
            Self::NonFiniteRenderPosition(id) => write!(f, "NAIR render node slot {id:?} has a non-finite position"),
            Self::RenderContextRequired => write!(f, "NAIR program contains render instructions but no render context was provided"),
            Self::DuplicateInputBridgeSlot(id) => write!(f, "NAIR input bridge slot {id:?} is defined more than once"),
            Self::UnknownInputBridgeSlot(id) => write!(f, "NAIR input bridge slot {id:?} is used before definition"),
            Self::InputContextRequired => write!(f, "NAIR program contains input instructions but no input batch was provided"),
            Self::DuplicateTimerSlot(id) => write!(f, "NAIR timer slot {id:?} is defined more than once"),
            Self::UnknownTimerSlot(id) => write!(f, "NAIR timer slot {id:?} is used before definition"),
            Self::ZeroTimerInterval(id) => write!(f, "NAIR repeating timer slot {id:?} must have a non-zero interval"),
            Self::TimeContextRequired => write!(f, "NAIR program contains time instructions but no logical-time context was provided"),
            Self::DuplicateCompletionSlot(id) => write!(f, "NAIR completion slot {id:?} is defined more than once"),
            Self::EmptyCompletionName(id) => write!(f, "NAIR completion slot {id:?} has an empty name"),
            Self::EmptyCompletionProjections(id) => write!(f, "NAIR completion slot {id:?} must contain at least one projection"),
            Self::DuplicateCompletionProjectionAtom { slot, atom } => write!(f, "NAIR completion slot {slot:?} projects atom {atom:?} more than once"),
            Self::CompletionBindingRequired(id) => write!(f, "NAIR completion slot {id:?} requires an explicit host source/namespace binding"),
            Self::CompletionContextRequired => write!(f, "NAIR program contains native completion declarations but no completion context was provided"),
            Self::DuplicateReactionSlot(id) => write!(f, "NAIR reaction slot {id:?} is defined more than once"),
            Self::EmptyReactionName(id) => write!(f, "NAIR reaction slot {id:?} has an empty reaction name"),
            Self::EmptyReactionActionName(id) => write!(f, "NAIR reaction slot {id:?} has an empty action name"),
            Self::EmptyReactionSteps(id) => write!(f, "NAIR reaction slot {id:?} must contain at least one reaction step"),
            Self::ReactionValueSourceMismatch(id) => write!(f, "NAIR reaction slot {id:?} uses a value source incompatible with its trigger"),
            Self::ReactionUndeclaredEffect { slot, effect } => write!(f, "NAIR reaction slot {slot:?} uses undeclared effect {effect:?}"),
            Self::NonFiniteReactionLiteral(id) => write!(f, "NAIR reaction slot {id:?} contains a non-finite float literal"),
            Self::ReactionContextRequired => write!(f, "NAIR program contains native reaction declarations but no reaction context was provided"),
            Self::MissingHalt => write!(f, "NAIR program must end with HALT"),
            Self::InstructionAfterHalt { index } => write!(f, "NAIR instruction at index {index} appears after HALT"),
            Self::NonFiniteFloat(id) => write!(f, "NAIR register {id:?} contains a non-finite float"),
            Self::IntegerAddOperandNotInt(id) => write!(f, "NAIR checked integer add requires integer register {id:?}"),
            Self::IntegerAddOverflow { lhs, rhs } => write!(f, "NAIR checked integer add overflowed for registers {lhs:?} and {rhs:?}"),
            Self::IntegerCompareOperandNotInt(id) => write!(f, "NAIR integer comparison requires integer register {id:?}"),
            Self::BranchConditionNotBool(id) => write!(f, "NAIR dynamic branch requires boolean condition register {id:?}"),
            Self::BranchArmKindMismatch => write!(f, "NAIR dynamic branch arms must have the same value kind"),
            Self::BranchArmKindUnsupported => write!(f, "NAIR dynamic branch arms support only INT or BOOL values"),
            Self::BranchExpressionOperandNotInt => write!(f, "NAIR selective branch integer expression requires INT operands"),
            Self::BranchExpressionIntegerOverflow => write!(f, "NAIR selective branch checked integer addition overflowed"),
            Self::BranchExpressionTooDeep => write!(f, "NAIR selective branch expression exceeds maximum nesting depth"),
            Self::BranchExpressionTooLarge => write!(f, "NAIR selective branch expression exceeds maximum node count"),
            Self::InputEventMissing(index) => write!(f, "NAIR input event index {index} is not present in the canonical input batch"),
            Self::InputEventNotKeyboardKey(index) => write!(f, "NAIR input event index {index} is not a keyboard key event"),
            Self::InvalidMagic => write!(f, "invalid NAIR magic header"),
            Self::UnsupportedFormat { major, minor } => write!(f, "unsupported NAIR format {major}.{minor}"),
            Self::UnexpectedEof => write!(f, "unexpected end of NAIR binary"),
            Self::InvalidOpcode(opcode) => write!(f, "invalid NAIR opcode 0x{opcode:02x}"),
            Self::InvalidValueTag(tag) => write!(f, "invalid NAIR value tag 0x{tag:02x}"),
            Self::InvalidBranchExpressionTag(tag) => write!(f, "invalid NAIR selective branch expression tag 0x{tag:02x}"),
            Self::InvalidDomainRefTag(tag) => write!(f, "invalid NAIR domain reference tag 0x{tag:02x}"),
            Self::InvalidRenderPrimitiveTag(tag) => write!(f, "invalid NAIR render primitive tag 0x{tag:02x}"),
            Self::InvalidRenderSpaceTag(tag) => write!(f, "invalid NAIR render space tag 0x{tag:02x}"),
            Self::InvalidDirtyMask(bits) => write!(f, "invalid NAIR render dirty mask 0x{bits:02x}"),
            Self::InvalidBoolTag(tag) => write!(f, "invalid NAIR boolean tag 0x{tag:02x}"),
            Self::InvalidOptionTag(tag) => write!(f, "invalid NAIR option tag 0x{tag:02x}"),
            Self::InvalidInputSourceTag(tag) => write!(f, "invalid NAIR input source tag 0x{tag:02x}"),
            Self::InvalidInputTargetTag(tag) => write!(f, "invalid NAIR input target tag 0x{tag:02x}"),
            Self::InvalidInputSignalTag(tag) => write!(f, "invalid NAIR input signal tag 0x{tag:02x}"),
            Self::InvalidEffectTag(tag) => write!(f, "invalid NAIR effect tag 0x{tag:02x}"),
            Self::InvalidReactionTriggerTag(tag) => write!(f, "invalid NAIR reaction trigger tag 0x{tag:02x}"),
            Self::InvalidReactionValueTag(tag) => write!(f, "invalid NAIR reaction value tag 0x{tag:02x}"),
            Self::InvalidReactionStepTag(tag) => write!(f, "invalid NAIR reaction step tag 0x{tag:02x}"),
            Self::InvalidCompletionProjectionTag(tag) => write!(f, "invalid NAIR completion projection tag 0x{tag:02x}"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in NAIR binary"),
            Self::TrailingBytes(count) => write!(f, "NAIR binary contains {count} trailing byte(s)"),
            Self::LengthOverflow => write!(f, "NAIR value is too large for canonical encoding"),
            Self::Kernel(err) => write!(f, "NAM rejected NAIR execution: {err}"),
            Self::Render(err) => write!(f, "Atomic Render Core rejected NAIR execution: {err}"),
            Self::Input(err) => write!(f, "Atomic Input Core rejected NAIR execution: {err}"),
            Self::Reaction(err) => write!(f, "Atomic Reaction Core rejected NAIR bootstrap: {err}"),
            Self::Completion(err) => write!(f, "Atomic Effect Completion Core rejected NAIR bootstrap: {err}"),
        }
    }
}

impl Error for NairError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(err) => Some(err),
            Self::Render(err) => Some(err),
            Self::Input(err) => Some(err),
            Self::Reaction(err) => Some(err),
            Self::Completion(err) => Some(err),
            _ => None,
        }
    }
}

impl From<AtomicError> for NairError {
    fn from(value: AtomicError) -> Self {
        Self::Kernel(value)
    }
}

impl From<RenderError> for NairError {
    fn from(value: RenderError) -> Self {
        Self::Render(value)
    }
}

impl From<InputError> for NairError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

impl From<ReactionError> for NairError {
    fn from(value: ReactionError) -> Self {
        Self::Reaction(value)
    }
}

impl From<EffectCompletionError> for NairError {
    fn from(value: EffectCompletionError) -> Self {
        Self::Completion(value)
    }
}

pub type NairResult<T> = Result<T, NairError>;
