use std::collections::BTreeMap;

use crate::{
    atom::AtomId,
    input::{
        InputAtomBridge, InputBatch, InputBridgeReport, InputSelector, InputSequence, InputTarget,
    },
    kernel::AtomicKernel,
    nair::{
        execute_nair_with_render_and_input, AtomSlot, DomainRef, DomainSlot, InputBridgeSlot,
        InputTargetRef, Instruction, NairError, NairInteractiveExecutionReport, NairProgram,
        RenderNodeSlot,
    },
    ownership::DomainId,
    render::{AtomicRenderCore, NairRenderFrame, RenderNodeId},
};

use super::{
    hash_bytes, hash_component, snapshot_atoms, RuntimeAtomSnapshot, RuntimeError,
    RuntimeReplayKey, RuntimeResult, FNV_OFFSET_BASIS,
};

const PERSISTENT_REPLAY_DOMAIN: &[u8] = b"NORDOI-PERSISTENT-RUNTIME-1.1";

#[derive(Debug, Clone, PartialEq)]
pub struct PersistentRuntimeTickReport {
    pub tick: u64,
    pub replay_key: RuntimeReplayKey,
    pub input_events: usize,
    pub input_applications: Vec<InputBridgeReport>,
    pub frame: Option<NairRenderFrame>,
    pub final_atoms: BTreeMap<AtomSlot, RuntimeAtomSnapshot>,
}

#[derive(Debug, Clone)]
pub struct PersistentAtomicRuntime {
    kernel: AtomicKernel,
    render: AtomicRenderCore,
    atom_bindings: BTreeMap<AtomSlot, AtomId>,
    input_bridges: BTreeMap<InputBridgeSlot, InputAtomBridge>,
    apply_order: Vec<InputBridgeSlot>,
    boot_report: NairInteractiveExecutionReport,
    replay_state: u64,
    replay_key: RuntimeReplayKey,
    tick: u64,
    last_input_sequence: Option<InputSequence>,
}

impl PersistentAtomicRuntime {
    pub fn boot(program: &NairProgram) -> RuntimeResult<Self> {
        let program_bytes = program.canonical_bytes()?;
        let empty_input = InputBatch::default();
        let mut kernel = AtomicKernel::new();
        let mut render = AtomicRenderCore::new();
        let boot_report =
            execute_nair_with_render_and_input(&mut kernel, &mut render, &empty_input, program)?;

        ensure_quiescent(&kernel, &render)?;

        let input_bridges = rebuild_input_bridges(program, &kernel, &boot_report)?;
        let apply_order = program
            .instructions()
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::ApplyInput { bridge } => Some(*bridge),
                _ => None,
            })
            .collect();

        let mut replay_state = FNV_OFFSET_BASIS;
        hash_bytes(&mut replay_state, PERSISTENT_REPLAY_DOMAIN);
        hash_component(&mut replay_state, &program_bytes);
        let replay_key = RuntimeReplayKey(replay_state);

        Ok(Self {
            atom_bindings: boot_report.execution.atom_bindings.clone(),
            kernel,
            render,
            input_bridges,
            apply_order,
            boot_report,
            replay_state,
            replay_key,
            tick: 0,
            last_input_sequence: None,
        })
    }

    pub fn tick(&mut self, input: &InputBatch) -> RuntimeResult<PersistentRuntimeTickReport> {
        let canonical_input = input.canonicalized()?;
        validate_cross_tick_sequence(self.last_input_sequence, &canonical_input)?;
        let input_bytes = canonical_input.canonical_bytes()?;

        let mut kernel = self.kernel.clone();
        let mut render = self.render.clone();
        let mut input_applications = Vec::with_capacity(self.apply_order.len());

        for slot in &self.apply_order {
            let bridge = self
                .input_bridges
                .get(slot)
                .ok_or(RuntimeError::Nair(NairError::UnknownInputBridgeSlot(*slot)))?;
            input_applications.push(bridge.apply_batch(&mut kernel, &canonical_input)?);
        }

        let frame = if kernel.pending_work() > 0 || render.pending_nodes() > 0 {
            let scheduled_atoms = kernel.flush();
            let newly_invalidated_nodes =
                render.invalidate_atoms(scheduled_atoms.iter().copied())?;
            let batch = render.flush();
            Some(NairRenderFrame {
                scheduled_atoms,
                newly_invalidated_nodes,
                batch,
            })
        } else {
            None
        };

        ensure_quiescent(&kernel, &render)?;
        let final_atoms = snapshot_atoms(&kernel, &self.atom_bindings)?;

        let mut replay_state = self.replay_state;
        hash_component(&mut replay_state, &input_bytes);
        let replay_key = RuntimeReplayKey(replay_state);
        let tick = self.tick + 1;
        let last_input_sequence = canonical_input.events.last().map(|event| event.sequence);

        self.kernel = kernel;
        self.render = render;
        self.replay_state = replay_state;
        self.replay_key = replay_key;
        self.tick = tick;
        if last_input_sequence.is_some() {
            self.last_input_sequence = last_input_sequence;
        }

        Ok(PersistentRuntimeTickReport {
            tick,
            replay_key,
            input_events: canonical_input.len(),
            input_applications,
            frame,
            final_atoms,
        })
    }

    pub fn tick_index(&self) -> u64 {
        self.tick
    }

    pub fn replay_key(&self) -> RuntimeReplayKey {
        self.replay_key
    }

    pub fn boot_report(&self) -> &NairInteractiveExecutionReport {
        &self.boot_report
    }

    pub fn snapshot(&self) -> RuntimeResult<BTreeMap<AtomSlot, RuntimeAtomSnapshot>> {
        snapshot_atoms(&self.kernel, &self.atom_bindings)
    }

    pub fn is_quiescent(&self) -> bool {
        self.kernel.pending_work() == 0 && self.render.pending_nodes() == 0
    }
}

fn validate_cross_tick_sequence(
    previous: Option<InputSequence>,
    batch: &InputBatch,
) -> RuntimeResult<()> {
    if let (Some(previous), Some(first)) = (previous, batch.events.first()) {
        if first.sequence <= previous {
            return Err(RuntimeError::Input(
                crate::input::InputError::NonMonotonicSequence {
                    previous,
                    current: first.sequence,
                },
            ));
        }
    }
    Ok(())
}

fn ensure_quiescent(kernel: &AtomicKernel, render: &AtomicRenderCore) -> RuntimeResult<()> {
    let nam = kernel.pending_work();
    let render_work = render.pending_nodes();
    if nam != 0 || render_work != 0 {
        return Err(RuntimeError::ResidualWork {
            nam,
            render: render_work,
        });
    }
    Ok(())
}

fn rebuild_input_bridges(
    program: &NairProgram,
    kernel: &AtomicKernel,
    report: &NairInteractiveExecutionReport,
) -> RuntimeResult<BTreeMap<InputBridgeSlot, InputAtomBridge>> {
    let mut bridges = BTreeMap::new();

    for instruction in program.instructions() {
        match instruction {
            Instruction::CreateInputBridge { dst, domain } => {
                let domain = resolve_domain(kernel, *domain, &report.execution.domain_bindings)?;
                bridges.insert(*dst, InputAtomBridge::new(domain));
            }
            Instruction::BindInputAtom {
                bridge,
                atom,
                source,
                device,
                target,
                signal,
            } => {
                let atom = report
                    .execution
                    .atom_bindings
                    .get(atom)
                    .copied()
                    .ok_or(RuntimeError::Nair(NairError::UnknownAtomSlot(*atom)))?;
                let target = resolve_target(*target, &report.render_bindings)?;
                let input_bridge = bridges.get_mut(bridge).ok_or(RuntimeError::Nair(
                    NairError::UnknownInputBridgeSlot(*bridge),
                ))?;
                input_bridge.bind(
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
            _ => {}
        }
    }

    Ok(bridges)
}

fn resolve_domain(
    kernel: &AtomicKernel,
    domain: DomainRef,
    domains: &BTreeMap<DomainSlot, DomainId>,
) -> RuntimeResult<DomainId> {
    match domain {
        DomainRef::Root => Ok(kernel.root_domain()),
        DomainRef::Slot(slot) => domains
            .get(&slot)
            .copied()
            .ok_or(RuntimeError::Nair(NairError::UnknownDomainSlot(slot))),
    }
}

fn resolve_target(
    target: InputTargetRef,
    render_nodes: &BTreeMap<RenderNodeSlot, RenderNodeId>,
) -> RuntimeResult<Option<InputTarget>> {
    match target {
        InputTargetRef::Any => Ok(None),
        InputTargetRef::Global => Ok(Some(InputTarget::Global)),
        InputTargetRef::RenderNode(slot) => render_nodes
            .get(&slot)
            .copied()
            .map(InputTarget::RenderNode)
            .map(Some)
            .ok_or(RuntimeError::Nair(NairError::UnknownRenderNodeSlot(slot))),
    }
}
