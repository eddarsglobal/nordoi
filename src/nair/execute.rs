use std::collections::BTreeMap;

use crate::{
    atom::AtomId,
    input::{InputAtomBridge, InputBatch, InputBridgeReport, InputSelector, InputTarget},
    kernel::AtomicKernel,
    ownership::DomainId,
    render::{AtomicRenderCore, NairRenderFrame, RenderNodeId},
    transaction::{AtomicTransaction, TransactionReport},
    value::Value,
};

use super::{
    error::{NairError, NairResult},
    id::{AtomSlot, DomainSlot, InputBridgeSlot, RegisterId, RenderNodeSlot, TransactionSlot},
    instruction::{DomainRef, InputTargetRef, Instruction},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NairInputExecutionReport {
    pub execution: NairExecutionReport,
    pub created_input_bridges: usize,
    pub input_applications: Vec<InputBridgeReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NairInteractiveExecutionReport {
    pub execution: NairExecutionReport,
    pub render_bindings: BTreeMap<RenderNodeSlot, RenderNodeId>,
    pub frames: Vec<NairRenderFrame>,
    pub created_input_bridges: usize,
    pub input_applications: Vec<InputBridgeReport>,
}

impl NairInteractiveExecutionReport {
    pub fn created_render_nodes(&self) -> usize {
        self.render_bindings.len()
    }

    pub fn last_frame(&self) -> Option<&NairRenderFrame> {
        self.frames.last()
    }
}

/// Additive execution observation used by V0.2 to validate source-level pure results.
///
/// Final registers are transient execution evidence only. They are not serialized into
/// NAIR, persisted in runtime checkpoints, exposed as host authority, or included in
/// source semantic identity.
#[derive(Debug, Clone, PartialEq)]
pub struct NairObservedInteractiveExecutionReport {
    pub execution: NairInteractiveExecutionReport,
    pub final_registers: BTreeMap<RegisterId, Value>,
}

impl NairObservedInteractiveExecutionReport {
    pub fn register(&self, id: RegisterId) -> Option<&Value> {
        self.final_registers.get(&id)
    }
}

struct ExecutionOutcome {
    execution: NairExecutionReport,
    render_bindings: BTreeMap<RenderNodeSlot, RenderNodeId>,
    frames: Vec<NairRenderFrame>,
    created_input_bridges: usize,
    input_applications: Vec<InputBridgeReport>,
    final_registers: BTreeMap<RegisterId, Value>,
}

pub fn execute_nair(
    kernel: &mut AtomicKernel,
    program: &NairProgram,
) -> NairResult<NairExecutionReport> {
    program.validate()?;
    reject_missing_contexts(program, false, false, false)?;
    Ok(execute_internal(kernel, None, None, program)?.execution)
}

pub fn execute_nair_with_render(
    kernel: &mut AtomicKernel,
    render: &mut AtomicRenderCore,
    program: &NairProgram,
) -> NairResult<NairRenderExecutionReport> {
    program.validate()?;
    reject_missing_contexts(program, true, false, false)?;
    let outcome = execute_internal(kernel, Some(render), None, program)?;

    Ok(NairRenderExecutionReport {
        execution: outcome.execution,
        render_bindings: outcome.render_bindings,
        frames: outcome.frames,
    })
}

pub fn execute_nair_with_input(
    kernel: &mut AtomicKernel,
    input_batch: &InputBatch,
    program: &NairProgram,
) -> NairResult<NairInputExecutionReport> {
    program.validate()?;
    reject_missing_contexts(program, false, true, false)?;
    let outcome = execute_internal(kernel, None, Some(input_batch), program)?;

    Ok(NairInputExecutionReport {
        execution: outcome.execution,
        created_input_bridges: outcome.created_input_bridges,
        input_applications: outcome.input_applications,
    })
}

pub fn execute_nair_with_render_and_input(
    kernel: &mut AtomicKernel,
    render: &mut AtomicRenderCore,
    input_batch: &InputBatch,
    program: &NairProgram,
) -> NairResult<NairInteractiveExecutionReport> {
    program.validate()?;
    reject_missing_contexts(program, true, true, false)?;
    let outcome = execute_internal(kernel, Some(render), Some(input_batch), program)?;

    Ok(NairInteractiveExecutionReport {
        execution: outcome.execution,
        render_bindings: outcome.render_bindings,
        frames: outcome.frames,
        created_input_bridges: outcome.created_input_bridges,
        input_applications: outcome.input_applications,
    })
}

/// Execute NAIR through the same interactive engine while retaining a transient
/// snapshot of final register values for validation/inspection layers.
pub fn execute_nair_with_render_and_input_observed(
    kernel: &mut AtomicKernel,
    render: &mut AtomicRenderCore,
    input_batch: &InputBatch,
    program: &NairProgram,
) -> NairResult<NairObservedInteractiveExecutionReport> {
    program.validate()?;
    reject_missing_contexts(program, true, true, false)?;
    let outcome = execute_internal(kernel, Some(render), Some(input_batch), program)?;

    Ok(NairObservedInteractiveExecutionReport {
        execution: NairInteractiveExecutionReport {
            execution: outcome.execution,
            render_bindings: outcome.render_bindings,
            frames: outcome.frames,
            created_input_bridges: outcome.created_input_bridges,
            input_applications: outcome.input_applications,
        },
        final_registers: outcome.final_registers,
    })
}

fn reject_missing_contexts(
    program: &NairProgram,
    has_render: bool,
    has_input: bool,
    has_time: bool,
) -> NairResult<()> {
    if !has_render
        && program
            .instructions()
            .iter()
            .any(Instruction::requires_render_context)
    {
        return Err(NairError::RenderContextRequired);
    }

    if !has_input
        && program
            .instructions()
            .iter()
            .any(Instruction::requires_input_context)
    {
        return Err(NairError::InputContextRequired);
    }

    if !has_time
        && program
            .instructions()
            .iter()
            .any(Instruction::requires_time_context)
    {
        return Err(NairError::TimeContextRequired);
    }

    if program
        .instructions()
        .iter()
        .any(Instruction::requires_reaction_context)
    {
        return Err(NairError::ReactionContextRequired);
    }

    if program
        .instructions()
        .iter()
        .any(Instruction::requires_completion_context)
    {
        return Err(NairError::CompletionContextRequired);
    }

    Ok(())
}

fn execute_internal(
    kernel: &mut AtomicKernel,
    mut render: Option<&mut AtomicRenderCore>,
    input_batch: Option<&InputBatch>,
    program: &NairProgram,
) -> NairResult<ExecutionOutcome> {
    let mut registers: BTreeMap<RegisterId, Value> = BTreeMap::new();
    let mut domains: BTreeMap<DomainSlot, DomainId> = BTreeMap::new();
    let mut atoms: BTreeMap<AtomSlot, AtomId> = BTreeMap::new();
    let mut transactions: BTreeMap<TransactionSlot, AtomicTransaction> = BTreeMap::new();
    let mut render_nodes: BTreeMap<RenderNodeSlot, RenderNodeId> = BTreeMap::new();
    let mut input_bridges: BTreeMap<InputBridgeSlot, InputAtomBridge> = BTreeMap::new();
    let mut frames = Vec::new();
    let mut input_applications = Vec::new();
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
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                let lhs_value = registers.get(lhs).ok_or(NairError::UnknownRegister(*lhs))?;
                let rhs_value = registers.get(rhs).ok_or(NairError::UnknownRegister(*rhs))?;
                let lhs_int = match lhs_value {
                    Value::Int(value) => *value,
                    _ => return Err(NairError::IntegerAddOperandNotInt(*lhs)),
                };
                let rhs_int = match rhs_value {
                    Value::Int(value) => *value,
                    _ => return Err(NairError::IntegerAddOperandNotInt(*rhs)),
                };
                let value = lhs_int
                    .checked_add(rhs_int)
                    .ok_or(NairError::IntegerAddOverflow {
                        lhs: *lhs,
                        rhs: *rhs,
                    })?;
                registers.insert(*dst, Value::Int(value));
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
            Instruction::CreateInputBridge { dst, domain } => {
                let domain = resolve_domain(kernel, *domain, &domains)?;
                input_bridges.insert(*dst, InputAtomBridge::new(domain));
            }
            Instruction::BindInputAtom {
                bridge,
                atom,
                source,
                device,
                target,
                signal,
            } => {
                let atom = *atoms.get(atom).ok_or(NairError::UnknownAtomSlot(*atom))?;
                let target = resolve_input_target(*target, &render_nodes)?;
                let bridge = input_bridges
                    .get_mut(bridge)
                    .ok_or(NairError::UnknownInputBridgeSlot(*bridge))?;
                bridge.bind(
                    kernel,
                    InputSelector {
                        source: *source,
                        device: *device,
                        target,
                        signal: *signal,
                    },
                    atom,
                )?;
            }
            Instruction::ApplyInput { bridge } => {
                let batch = input_batch.ok_or(NairError::InputContextRequired)?;
                let bridge = input_bridges
                    .get(bridge)
                    .ok_or(NairError::UnknownInputBridgeSlot(*bridge))?;
                input_applications.push(bridge.apply_batch(kernel, batch)?);
            }
            Instruction::ScheduleTimerOnceAt { .. }
            | Instruction::ScheduleTimerRepeatingAt { .. }
            | Instruction::CancelTimer { .. } => {
                return Err(NairError::TimeContextRequired);
            }
            Instruction::DefineReaction { .. } => {
                return Err(NairError::ReactionContextRequired);
            }
            Instruction::DefineEffectCompletion { .. } => {
                return Err(NairError::CompletionContextRequired);
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
        created_input_bridges: input_bridges.len(),
        input_applications,
        final_registers: registers,
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

fn resolve_input_target(
    target: InputTargetRef,
    render_nodes: &BTreeMap<RenderNodeSlot, RenderNodeId>,
) -> NairResult<Option<InputTarget>> {
    match target {
        InputTargetRef::Any => Ok(None),
        InputTargetRef::Global => Ok(Some(InputTarget::Global)),
        InputTargetRef::RenderNode(slot) => render_nodes
            .get(&slot)
            .copied()
            .map(InputTarget::RenderNode)
            .map(Some)
            .ok_or(NairError::UnknownRenderNodeSlot(slot)),
    }
}
