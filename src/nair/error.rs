use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::error::AtomicError;

use super::id::{AtomSlot, DomainSlot, RegisterId, TransactionSlot};

#[derive(Debug, PartialEq, Eq)]
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
    MissingHalt,
    InstructionAfterHalt { index: usize },
    NonFiniteFloat(RegisterId),
    InvalidMagic,
    UnsupportedFormat { major: u16, minor: u16 },
    UnexpectedEof,
    InvalidOpcode(u8),
    InvalidValueTag(u8),
    InvalidDomainRefTag(u8),
    InvalidUtf8,
    TrailingBytes(usize),
    LengthOverflow,
    Kernel(AtomicError),
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
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in NAIR binary"),
            Self::TrailingBytes(count) => {
                write!(f, "NAIR binary contains {count} trailing byte(s)")
            }
            Self::LengthOverflow => write!(f, "NAIR value is too large for canonical encoding"),
            Self::Kernel(err) => write!(f, "NAM rejected NAIR execution: {err}"),
        }
    }
}

impl Error for NairError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(err) => Some(err),
            _ => None,
        }
    }
}

impl From<AtomicError> for NairError {
    fn from(value: AtomicError) -> Self {
        Self::Kernel(value)
    }
}

pub type NairResult<T> = Result<T, NairError>;
