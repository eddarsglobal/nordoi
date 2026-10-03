use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{error::AtomicError, input::InputError, render::RenderError};

use super::id::{
    AtomSlot, DomainSlot, InputBridgeSlot, RegisterId, RenderNodeSlot, TransactionSlot,
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
    MissingHalt,
    InstructionAfterHalt { index: usize },
    NonFiniteFloat(RegisterId),
    InvalidMagic,
    UnsupportedFormat { major: u16, minor: u16 },
    UnexpectedEof,
    InvalidOpcode(u8),
    InvalidValueTag(u8),
    InvalidDomainRefTag(u8),
    InvalidRenderPrimitiveTag(u8),
    InvalidRenderSpaceTag(u8),
    InvalidDirtyMask(u8),
    InvalidBoolTag(u8),
    InvalidOptionTag(u8),
    InvalidInputSourceTag(u8),
    InvalidInputTargetTag(u8),
    InvalidInputSignalTag(u8),
    InvalidUtf8,
    TrailingBytes(usize),
    LengthOverflow,
    Kernel(AtomicError),
    Render(RenderError),
    Input(InputError),
}

impl Display for NairError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRegister(id) => {
                write!(f, "NAIR register {id:?} is defined more than once")
            }
            Self::UnknownRegister(id) => {
                write!(f, "NAIR register {id:?} is used before definition")
            }
            Self::DuplicateDomainSlot(id) => {
                write!(f, "NAIR domain slot {id:?} is defined more than once")
            }
            Self::UnknownDomainSlot(id) => {
                write!(f, "NAIR domain slot {id:?} is used before definition")
            }
            Self::EmptyDomainName(id) => write!(f, "NAIR domain slot {id:?} has an empty name"),
            Self::DuplicateAtomSlot(id) => {
                write!(f, "NAIR atom slot {id:?} is defined more than once")
            }
            Self::UnknownAtomSlot(id) => {
                write!(f, "NAIR atom slot {id:?} is used before definition")
            }
            Self::SelfDependency(id) => write!(f, "NAIR atom slot {id:?} cannot depend on itself"),
            Self::DuplicateTransactionSlot(id) => {
                write!(f, "NAIR transaction slot {id:?} is defined more than once")
            }
            Self::InactiveTransaction(id) => {
                write!(f, "NAIR transaction slot {id:?} is not active")
            }
            Self::UnclosedTransactions(ids) => {
                write!(f, "NAIR program halts with active transactions: {ids:?}")
            }
            Self::DuplicateRenderNodeSlot(id) => {
                write!(f, "NAIR render node slot {id:?} is defined more than once")
            }
            Self::UnknownRenderNodeSlot(id) => {
                write!(f, "NAIR render node slot {id:?} is used before definition")
            }
            Self::EmptyRenderDirtyMask => {
                write!(f, "NAIR render binding cannot use an empty dirty mask")
            }
            Self::InvalidRenderOpacity(value) => write!(
                f,
                "NAIR render opacity must be finite and within 0..=1, got {value}"
            ),
            Self::NonFiniteRenderPosition(id) => {
                write!(f, "NAIR render node slot {id:?} has a non-finite position")
            }
            Self::RenderContextRequired => write!(
                f,
                "NAIR program contains render instructions but no render context was provided"
            ),
            Self::DuplicateInputBridgeSlot(id) => {
                write!(f, "NAIR input bridge slot {id:?} is defined more than once")
            }
            Self::UnknownInputBridgeSlot(id) => {
                write!(f, "NAIR input bridge slot {id:?} is used before definition")
            }
            Self::InputContextRequired => write!(
                f,
                "NAIR program contains input instructions but no input batch was provided"
            ),
            Self::MissingHalt => write!(f, "NAIR program must end with HALT"),
            Self::InstructionAfterHalt { index } => {
                write!(f, "NAIR instruction at index {index} appears after HALT")
            }
            Self::NonFiniteFloat(id) => {
                write!(f, "NAIR register {id:?} contains a non-finite float")
            }
            Self::InvalidMagic => write!(f, "invalid NAIR magic header"),
            Self::UnsupportedFormat { major, minor } => {
                write!(f, "unsupported NAIR format {major}.{minor}")
            }
            Self::UnexpectedEof => write!(f, "unexpected end of NAIR binary"),
            Self::InvalidOpcode(opcode) => write!(f, "invalid NAIR opcode 0x{opcode:02x}"),
            Self::InvalidValueTag(tag) => write!(f, "invalid NAIR value tag 0x{tag:02x}"),
            Self::InvalidDomainRefTag(tag) => {
                write!(f, "invalid NAIR domain reference tag 0x{tag:02x}")
            }
            Self::InvalidRenderPrimitiveTag(tag) => {
                write!(f, "invalid NAIR render primitive tag 0x{tag:02x}")
            }
            Self::InvalidRenderSpaceTag(tag) => {
                write!(f, "invalid NAIR render space tag 0x{tag:02x}")
            }
            Self::InvalidDirtyMask(bits) => {
                write!(f, "invalid NAIR render dirty mask 0x{bits:02x}")
            }
            Self::InvalidBoolTag(tag) => write!(f, "invalid NAIR boolean tag 0x{tag:02x}"),
            Self::InvalidOptionTag(tag) => write!(f, "invalid NAIR option tag 0x{tag:02x}"),
            Self::InvalidInputSourceTag(tag) => {
                write!(f, "invalid NAIR input source tag 0x{tag:02x}")
            }
            Self::InvalidInputTargetTag(tag) => {
                write!(f, "invalid NAIR input target tag 0x{tag:02x}")
            }
            Self::InvalidInputSignalTag(tag) => {
                write!(f, "invalid NAIR input signal tag 0x{tag:02x}")
            }
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in NAIR binary"),
            Self::TrailingBytes(count) => {
                write!(f, "NAIR binary contains {count} trailing byte(s)")
            }
            Self::LengthOverflow => write!(f, "NAIR value is too large for canonical encoding"),
            Self::Kernel(err) => write!(f, "NAM rejected NAIR execution: {err}"),
            Self::Render(err) => write!(f, "Atomic Render Core rejected NAIR execution: {err}"),
            Self::Input(err) => write!(f, "Atomic Input Core rejected NAIR execution: {err}"),
        }
    }
}

impl Error for NairError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(err) => Some(err),
            Self::Render(err) => Some(err),
            Self::Input(err) => Some(err),
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

pub type NairResult<T> = Result<T, NairError>;
