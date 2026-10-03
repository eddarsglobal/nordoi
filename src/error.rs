use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum AtomicError {
    UnknownAtom(crate::atom::AtomId),
    UnknownDomain(crate::ownership::DomainId),
    DuplicateDomain(crate::ownership::DomainId),
    OwnershipViolation {
        atom: crate::atom::AtomId,
        owner: crate::ownership::DomainId,
        attempted: crate::ownership::DomainId,
    },
    TransactionConflict {
        atom: crate::atom::AtomId,
        expected_version: u64,
        actual_version: u64,
    },
    CapabilityDenied(crate::capability::Capability),
    UndeclaredEffect(crate::effect::Effect),
    DependencyCycle,
}

impl Display for AtomicError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAtom(id) => write!(f, "atom {id:?} does not exist"),
            Self::UnknownDomain(id) => write!(f, "ownership domain {id:?} does not exist"),
            Self::DuplicateDomain(id) => write!(f, "ownership domain {id:?} already exists"),
            Self::OwnershipViolation {
                atom,
                owner,
                attempted,
            } => write!(
                f,
                "ownership violation for {atom:?}: owner={owner:?}, attempted={attempted:?}"
            ),
            Self::TransactionConflict {
                atom,
                expected_version,
                actual_version,
            } => write!(
                f,
                "transaction conflict for {atom:?}: expected version {expected_version}, actual version {actual_version}"
            ),
            Self::CapabilityDenied(capability) => {
                write!(f, "capability denied: {capability:?}")
            }
            Self::UndeclaredEffect(effect) => {
                write!(f, "effect was not declared: {effect:?}")
            }
            Self::DependencyCycle => write!(f, "dependency cycle detected"),
        }
    }
}

impl Error for AtomicError {}

pub type AtomicResult<T> = Result<T, AtomicError>;
