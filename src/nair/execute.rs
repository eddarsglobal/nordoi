use std::collections::BTreeMap;

use crate::{
    atom::AtomId,
    kernel::AtomicKernel,
    ownership::DomainId,
    render::{AtomicRenderCore, NairRenderFrame, RenderNodeId},
    transaction::{AtomicTransaction, TransactionReport},
    value::Value,
};

use super::{
    error::{NairError, NairResult},
    id::{AtomSlot, DomainSlot, RegisterId, RenderNodeSlot, TransactionSlot},
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

#[derive(Debug, Clone, PartialEq)]
pub struct NairRenderExecutionReport {
    pub execution: NairExecutionReport,
    pub render_bindings: BTreeMap<RenderNodeSlot, RenderNodeId>,
    pub frames: Vec<NairRenderFrame>,
}

impl NairRenderExecutionReport {
    pub fn created_render_nodes(&self) -> usize {
        self.render_bindings.len()
    }

    pub fn last_frame(&self) -> Option<&NairRenderFrame> {
        self.frames.last()
    }
}

struct ExecutionOutcome {
    execution: NairExecutionReport,
    render_bindings: BTreeMap<RenderNodeSlot, RenderNodeId>,
    frames: Vec<NairRenderFrame>,
}

pub fn execute_nair(
    kernel: &mut AtomicKernel,
    program: &NairProgram,
) -> NairResult<NairExecutionReport> {
    program.validate()?;

    if program
        .instructions()
        .iter()
        .any(Instruction::requires_render_context)
    {
        return Err(NairError::RenderContextRequired);
    }

    Ok(execute_internal(kernel, None, program)?.execution)
}

pub fn execute_nair_with_render(
    kernel: &mut AtomicKernel,
    render: &mut AtomicRenderCore,
    program: &NairProgram,
) -> NairResult<NairRenderExecutionReport> {
    program.validate()?;
    let outcome = execute_internal(kernel, Some(render), program)?;

    Ok(NairRenderExecutionReport {
        execution: outcome.execution,
        render_bindings: outcome.render_bindings,
        frames: outcome.frames,
    })
}

fn execute_internal(
    kernel: &mut AtomicKernel,
    mut render: Option<&mut AtomicRenderCore>,
    program: &NairProgram,
) -> NairResult<ExecutionOutcome> {
    let mut registers: BTreeMap<RegisterId, Value> = BTreeMap::new();
    let mut domains: BTreeMap<DomainSlot, DomainId> = BTreeMap::new();
    let mut atoms: BTreeMap<AtomSlot, AtomId> = BTreeMap::new();
    let mut transactions: BTreeMap<TransactionSlot, AtomicTransaction> = BTreeMap::new();
    let mut render_nodes: BTreeMap<RenderNodeSlot, RenderNodeId> = BTreeMap::new();
    let mut frames = Vec::new();
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
            Instruction::CreateRenderNode {
                dst,
                primitive,
                space,
            } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let node = render.create_node(*primitive, *space);
                render_nodes.insert(*dst, node);
            }
            Instruction::CreateRenderChild {
                dst,
                parent,
                primitive,
                space,
            } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let parent = *render_nodes
                    .get(parent)
                    .ok_or(NairError::UnknownRenderNodeSlot(*parent))?;
                let node = render.create_child(parent, *primitive, *space)?;
                render_nodes.insert(*dst, node);
            }
            Instruction::BindRenderAtom { atom, node, dirty } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let atom = *atoms.get(atom).ok_or(NairError::UnknownAtomSlot(*atom))?;
                let node = *render_nodes
                    .get(node)
                    .ok_or(NairError::UnknownRenderNodeSlot(*node))?;
                render.bind_atom(atom, node, *dirty)?;
            }
            Instruction::SetRenderVisible { node, visible } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let node = *render_nodes
                    .get(node)
                    .ok_or(NairError::UnknownRenderNodeSlot(*node))?;
                render.set_visible(node, *visible)?;
            }
            Instruction::SetRenderOpacity { node, opacity } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let node = *render_nodes
                    .get(node)
                    .ok_or(NairError::UnknownRenderNodeSlot(*node))?;
                render.set_opacity(node, *opacity)?;
            }
            Instruction::SetRenderPosition { node, position } => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                let node = *render_nodes
                    .get(node)
                    .ok_or(NairError::UnknownRenderNodeSlot(*node))?;
                render.set_position(node, *position)?;
            }
            Instruction::RenderFlush => {
                let render = render
                    .as_deref_mut()
                    .ok_or(NairError::RenderContextRequired)?;
                frames.push(pump_render(kernel, render)?);
            }
            Instruction::Halt => {
                if let Some(render) = render.as_deref_mut() {
                    if kernel.pending_work() > 0 || render.pending_nodes() > 0 {
                        frames.push(pump_render(kernel, render)?);
                    }
                }
                break;
            }
        }
    }

    Ok(ExecutionOutcome {
        execution: NairExecutionReport {
            executed_instructions,
            created_domains: domains.len(),
            created_atoms: atoms.len(),
            committed_transactions,
            rolled_back_transactions,
            scheduled_work: kernel.pending_work(),
            domain_bindings: domains,
            atom_bindings: atoms,
            transaction_reports,
        },
        render_bindings: render_nodes,
        frames,
    })
}

fn pump_render(
    kernel: &mut AtomicKernel,
    render: &mut AtomicRenderCore,
) -> NairResult<NairRenderFrame> {
    let scheduled_atoms = kernel.flush();
    let newly_invalidated_nodes = render.invalidate_atoms(scheduled_atoms.iter().copied())?;
    let batch = render.flush();

    Ok(NairRenderFrame {
        scheduled_atoms,
        newly_invalidated_nodes,
        batch,
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
