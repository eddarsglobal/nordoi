use std::collections::BTreeMap;

use crate::{
    atom::AtomId, error::AtomicResult, kernel::AtomicKernel, ownership::DomainId, value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(pub u64);

#[derive(Debug, Clone)]
pub(crate) struct StagedWrite {
    pub expected_version: u64,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct AtomicTransaction {
    pub(crate) id: TransactionId,
    pub(crate) domain: DomainId,
    pub(crate) writes: BTreeMap<AtomId, StagedWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionReport {
    pub id: TransactionId,
    pub staged_writes: usize,
    pub changed_atoms: usize,
    pub scheduled_atoms: usize,
}

impl AtomicTransaction {
    pub(crate) fn new(id: TransactionId, domain: DomainId) -> Self {
        Self {
            id,
            domain,
            writes: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn domain(&self) -> DomainId {
        self.domain
    }

    pub fn staged_writes(&self) -> usize {
        self.writes.len()
    }

    /// Stage a write without mutating kernel state.
    /// Ownership and atom existence are checked immediately.
    /// The atom version is captured on the first staged write for conflict detection.
    pub fn set(
        &mut self,
        kernel: &AtomicKernel,
        atom: AtomId,
        next: impl Into<Value>,
    ) -> AtomicResult<()> {
        kernel.require_owner(atom, self.domain)?;

        let expected_version = match self.writes.get(&atom) {
            Some(existing) => existing.expected_version,
            None => kernel.version(atom)?,
        };

        self.writes.insert(
            atom,
            StagedWrite {
                expected_version,
                value: next.into(),
            },
        );

        Ok(())
    }

    /// Rollback is intentionally zero-work: staged values were never applied.
    pub fn rollback(self) -> TransactionReport {
        TransactionReport {
            id: self.id,
            staged_writes: self.writes.len(),
            changed_atoms: 0,
            scheduled_atoms: 0,
        }
    }
}
