use std::collections::BTreeMap;

use crate::{
    atom::AtomId,
    kernel::AtomicKernel,
    ownership::DomainId,
    transaction::{AtomicTransaction, TransactionReport},
    value::Value,
};

use super::{
    error::{NairError, NairResult},
    id::{AtomSlot, DomainSlot, RegisterId, TransactionSlot},
    instruction::{DomainRef, Instruction},
    program::NairProgram,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NairExecutionReport {
    pub executed_instructions: usize,
    pub created_domains: usize,
    pub created_atoms: usize,
    pub committed_transactions: usize,
    pub rolled_back_transactions: usize,
    pub scheduled_work: usize,
    pub domain_bindings: BTreeMap<DomainSlot, DomainId>,
    pub atom_bindings: BTreeMap<AtomSlot, AtomId>,
    pub transaction_reports: Vec<TransactionReport>,
}

pub fn execute_nair(
    kernel: &mut AtomicKernel,
    program: &NairProgram,
) -> NairResult<NairExecutionReport> {
    program.validate()?;

    let mut registers: BTreeMap<RegisterId, Value> = BTreeMap::new();
    let mut domains: BTreeMap<DomainSlot, DomainId> = BTreeMap::new();
    let mut atoms: BTreeMap<AtomSlot, AtomId> = BTreeMap::new();
    let mut transactions: BTreeMap<TransactionSlot, AtomicTransaction> = BTreeMap::new();
    let mut transaction_reports = Vec::new();
    let mut committed_transactions = 0usize;
    let mut rolled_back_transactions = 0usize;
    let mut executed_instructions = 0usize;

    for instruction in program.instructions() {
        executed_instructions += 1;

        match instruction {
            Instruction::Const { dst, value } => {
                registers.insert(*dst, value.clone());
            }
            Instruction::CreateDomain { dst, name } => {
                let domain = kernel.create_domain(name.clone())?;
                domains.insert(*dst, domain);
            }
            Instruction::CreateAtom { dst, owner, value } => {
                let domain = resolve_domain(kernel, *owner, &domains)?;
                let value = registers
                    .get(value)
                    .cloned()
                    .ok_or(NairError::UnknownRegister(*value))?;
                let atom = kernel.create_atom_owned(domain, value)?;
                atoms.insert(*dst, atom);
            }
            Instruction::Connect { source, dependent } => {
                let source_id = *atoms
                    .get(source)
                    .ok_or(NairError::UnknownAtomSlot(*source))?;
                let dependent_id = *atoms
                    .get(dependent)
                    .ok_or(NairError::UnknownAtomSlot(*dependent))?;
                kernel.connect(source_id, dependent_id)?;
            }
            Instruction::BeginTransaction { dst, domain } => {
                let domain = resolve_domain(kernel, *domain, &domains)?;
                let tx = kernel.begin_transaction(domain)?;
                transactions.insert(*dst, tx);
            }
            Instruction::TxSet { tx, atom, value } => {
                let atom_id = *atoms.get(atom).ok_or(NairError::UnknownAtomSlot(*atom))?;
                let value = registers
                    .get(value)
                    .cloned()
                    .ok_or(NairError::UnknownRegister(*value))?;
                let transaction = transactions
                    .get_mut(tx)
                    .ok_or(NairError::InactiveTransaction(*tx))?;
                transaction.set(kernel, atom_id, value)?;
            }
            Instruction::Commit { tx } => {
                let transaction = transactions
                    .remove(tx)
                    .ok_or(NairError::InactiveTransaction(*tx))?;
                let report = kernel.commit(transaction)?;
                transaction_reports.push(report);
                committed_transactions += 1;
            }
            Instruction::Rollback { tx } => {
                let transaction = transactions
                    .remove(tx)
                    .ok_or(NairError::InactiveTransaction(*tx))?;
                transaction_reports.push(transaction.rollback());
                rolled_back_transactions += 1;
            }
            Instruction::Halt => break,
        }
    }

    Ok(NairExecutionReport {
        executed_instructions,
        created_domains: domains.len(),
        created_atoms: atoms.len(),
        committed_transactions,
        rolled_back_transactions,
        scheduled_work: kernel.pending_work(),
        domain_bindings: domains,
        atom_bindings: atoms,
        transaction_reports,
    })
}

fn resolve_domain(
    kernel: &AtomicKernel,
    domain: DomainRef,
    domains: &BTreeMap<DomainSlot, DomainId>,
) -> NairResult<DomainId> {
    match domain {
        DomainRef::Root => Ok(kernel.root_domain()),
        DomainRef::Slot(slot) => domains
            .get(&slot)
            .copied()
            .ok_or(NairError::UnknownDomainSlot(slot)),
    }
}
