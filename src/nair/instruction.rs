use crate::value::Value;

use super::id::{AtomSlot, DomainSlot, RegisterId, TransactionSlot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainRef {
    Root,
    Slot(DomainSlot),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Const {
        dst: RegisterId,
        value: Value,
    },
    CreateDomain {
        dst: DomainSlot,
        name: String,
    },
    CreateAtom {
        dst: AtomSlot,
        owner: DomainRef,
        value: RegisterId,
    },
    Connect {
        source: AtomSlot,
        dependent: AtomSlot,
    },
    BeginTransaction {
        dst: TransactionSlot,
        domain: DomainRef,
    },
    TxSet {
        tx: TransactionSlot,
        atom: AtomSlot,
        value: RegisterId,
    },
    Commit {
        tx: TransactionSlot,
    },
    Rollback {
        tx: TransactionSlot,
    },
    Halt,
}
