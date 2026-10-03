mod error;
mod persistent;

use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::{
    atom::AtomId,
    input::InputBatch,
    kernel::AtomicKernel,
    nair::{
        execute_nair_with_render_and_input, AtomSlot, NairInteractiveExecutionReport, NairProgram,
    },
    render::{AtomicRenderCore, NairRenderFrame},
    value::Value,
};

pub use error::{RuntimeError, RuntimeResult};
pub use persistent::{PersistentAtomicRuntime, PersistentRuntimeTickReport};

const REPLAY_KEY_DOMAIN: &[u8] = b"NORDOI-ATOMIC-RUNTIME-1.0";
pub(super) const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001B3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeReplayKey(pub u64);

impl RuntimeReplayKey {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for RuntimeReplayKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeAtomSnapshot {
    pub id: AtomId,
    pub version: u64,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeReport {
    pub replay_key: RuntimeReplayKey,
    pub input_events: usize,
    pub final_atoms: BTreeMap<AtomSlot, RuntimeAtomSnapshot>,
    pub execution: NairInteractiveExecutionReport,
}

impl RuntimeReport {
    pub fn is_quiescent(&self) -> bool {
        self.execution.execution.scheduled_work == 0
    }

    pub fn last_frame(&self) -> Option<&NairRenderFrame> {
        self.execution.last_frame()
    }
}

/// K1.0 closed execution capsule.
///
/// Every call starts from fresh NAM and render state. Input is canonicalized before
/// execution. If execution fails, all intermediate state is dropped with the local
/// capsule and no partial runtime report is returned.
#[derive(Debug, Default, Clone, Copy)]
pub struct AtomicRuntime;

impl AtomicRuntime {
    pub const fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        program: &NairProgram,
        input: &InputBatch,
    ) -> RuntimeResult<RuntimeReport> {
        let program_bytes = program.canonical_bytes()?;
        let input_bytes = input.canonical_bytes()?;
        let canonical_input = input.canonicalized()?;
        let replay_key = replay_key(&program_bytes, &input_bytes);

        let mut kernel = AtomicKernel::new();
        let mut render = AtomicRenderCore::new();
        let execution = execute_nair_with_render_and_input(
            &mut kernel,
            &mut render,
            &canonical_input,
            program,
        )?;

        let residual_nam = kernel.pending_work();
        let residual_render = render.pending_nodes();
        if residual_nam != 0 || residual_render != 0 {
            return Err(RuntimeError::ResidualWork {
                nam: residual_nam,
                render: residual_render,
            });
        }

        let final_atoms = snapshot_atoms(&kernel, &execution.execution.atom_bindings)?;

        Ok(RuntimeReport {
            replay_key,
            input_events: canonical_input.len(),
            final_atoms,
            execution,
        })
    }
}

pub fn run_closed(program: &NairProgram, input: &InputBatch) -> RuntimeResult<RuntimeReport> {
    AtomicRuntime::new().execute(program, input)
}

pub(super) fn snapshot_atoms(
    kernel: &AtomicKernel,
    bindings: &BTreeMap<AtomSlot, AtomId>,
) -> RuntimeResult<BTreeMap<AtomSlot, RuntimeAtomSnapshot>> {
    let mut final_atoms = BTreeMap::new();
    for (slot, id) in bindings {
        final_atoms.insert(
            *slot,
            RuntimeAtomSnapshot {
                id: *id,
                version: kernel.version(*id)?,
                value: kernel.get(*id)?.clone(),
            },
        );
    }
    Ok(final_atoms)
}

fn replay_key(program: &[u8], input: &[u8]) -> RuntimeReplayKey {
    let mut hash = FNV_OFFSET_BASIS;
    hash_bytes(&mut hash, REPLAY_KEY_DOMAIN);
    hash_component(&mut hash, program);
    hash_component(&mut hash, input);
    RuntimeReplayKey(hash)
}

pub(super) fn hash_component(hash: &mut u64, bytes: &[u8]) {
    hash_bytes(hash, &(bytes.len() as u64).to_le_bytes());
    hash_bytes(hash, bytes);
}

pub(super) fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}
