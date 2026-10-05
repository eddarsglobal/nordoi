use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{compile_nair_lowering_boundary, CompilerError, NairLoweringArtifact};
use crate::frontend::{SourceSpan, SourceText};
use crate::input::{InputBatch, InputError};
use crate::nair::Instruction;
use crate::runtime::{run_closed, RuntimeError, RuntimeReport};

const V01_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.1-EXECUTION-RECEIPT\0";

#[derive(Debug)]
pub enum SourceExecutionError {
    Compiler(CompilerError),
    Runtime(RuntimeError),
    InvariantViolation { invariant: &'static str },
}

impl SourceExecutionError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Compiler(error) => error.primary_span(),
            Self::Runtime(_) | Self::InvariantViolation { .. } => None,
        }
    }

    pub fn is_compiler_failure(&self) -> bool {
        matches!(self, Self::Compiler(_))
    }
}

impl Display for SourceExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::InvariantViolation { invariant } => {
                write!(f, "V0.1 execution invariant failed: {invariant}")
            }
        }
    }
}

impl Error for SourceExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::InvariantViolation { .. } => None,
        }
    }
}

impl From<CompilerError> for SourceExecutionError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<RuntimeError> for SourceExecutionError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for SourceExecutionError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type SourceExecutionResult<T> = Result<T, SourceExecutionError>;

/// V0.1 source-to-runtime evidence artifact.
///
/// This report binds the already-certified C0.4 lowering artifact to one closed-runtime
/// execution using canonical empty input. It is execution evidence, not a program format,
/// source identity, NAIR encoding, runtime checkpoint, or authority grant.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceExecutionReport {
    lowering: NairLoweringArtifact,
    canonical_input: Vec<u8>,
    runtime: RuntimeReport,
}

impl SourceExecutionReport {
    fn new(
        lowering: NairLoweringArtifact,
        canonical_input: Vec<u8>,
        runtime: RuntimeReport,
    ) -> Self {
        Self {
            lowering,
            canonical_input,
            runtime,
        }
    }

    pub fn lowering(&self) -> &NairLoweringArtifact {
        &self.lowering
    }

    pub fn canonical_input_bytes(&self) -> &[u8] {
        &self.canonical_input
    }

    pub fn runtime(&self) -> &RuntimeReport {
        &self.runtime
    }

    pub fn is_quiescent(&self) -> bool {
        self.runtime.is_quiescent()
    }

    /// Deterministic V0.1 execution receipt.
    ///
    /// This receipt commits to compiler provenance (C0.4), canonical empty input,
    /// the runtime replay key, and the bounded execution summary verified by V0.1.
    /// It is deliberately not part of source semantics or the NAIR wire format.
    pub fn canonical_v01_receipt_bytes(&self) -> Vec<u8> {
        let c04 = self.lowering.canonical_c04_bytes();
        let execution = &self.runtime.execution.execution;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(V01_RECEIPT_DOMAIN);
        push_component(&mut bytes, &c04);
        push_component(&mut bytes, &self.canonical_input);
        bytes.extend_from_slice(&self.runtime.replay_key.value().to_be_bytes());
        bytes.extend_from_slice(&(self.runtime.input_events as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.executed_instructions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.created_domains as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.created_atoms as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.committed_transactions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.rolled_back_transactions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.scheduled_work as u64).to_be_bytes());
        bytes.extend_from_slice(&(self.runtime.execution.frames.len() as u64).to_be_bytes());
        bytes.extend_from_slice(
            &(self.runtime.execution.created_input_bridges as u64).to_be_bytes(),
        );
        bytes.push(u8::from(self.runtime.is_quiescent()));
        bytes
    }
}

/// Execute the first certified `.noi` subset end-to-end.
///
/// V0.1 compiles through C0.4, supplies canonical empty input, executes the exact
/// NAIR 0.6 program through the existing closed runtime, then validates the expected
/// zero-work HALT-only outcome before publishing a report.
pub fn execute_source_v01(source: &SourceText) -> SourceExecutionResult<SourceExecutionReport> {
    let lowering = compile_nair_lowering_boundary(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed(lowering.program(), &input)?;

    validate_v01_execution(&lowering, &runtime)?;

    Ok(SourceExecutionReport::new(
        lowering,
        canonical_input,
        runtime,
    ))
}

/// Validate the exact runtime shape currently certified by V0.1.
///
/// This is public so adversarial tests and future higher layers can verify a report
/// before trusting it as V0.1 execution evidence.
pub fn validate_v01_execution(
    lowering: &NairLoweringArtifact,
    runtime: &RuntimeReport,
) -> SourceExecutionResult<()> {
    if lowering.program().instructions() != [Instruction::Halt] {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "lowered program must contain exactly one HALT instruction",
        });
    }
    if lowering.semantic_work_item_count() != 0 {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "semantic work item count must remain zero",
        });
    }
    if !lowering.plan().required_effects().is_empty() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "required semantic effect count must remain zero",
        });
    }
    if lowering.requires_host_authority() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "host authority requirement must remain absent",
        });
    }
    if runtime.input_events != 0 {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must consume zero input events",
        });
    }
    if !runtime.final_atoms.is_empty() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must publish zero final atoms",
        });
    }

    let execution = &runtime.execution.execution;
    if execution.executed_instructions != 1 {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must execute exactly one instruction",
        });
    }
    if execution.created_domains != 0 || !execution.domain_bindings.is_empty() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must create zero domains",
        });
    }
    if execution.created_atoms != 0 || !execution.atom_bindings.is_empty() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must create zero atoms",
        });
    }
    if execution.committed_transactions != 0
        || execution.rolled_back_transactions != 0
        || !execution.transaction_reports.is_empty()
    {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must perform zero transactions",
        });
    }
    if execution.scheduled_work != 0 || !runtime.is_quiescent() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must end quiescent with zero scheduled work",
        });
    }
    if !runtime.execution.render_bindings.is_empty() || !runtime.execution.frames.is_empty() {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must create zero render state or frames",
        });
    }
    if runtime.execution.created_input_bridges != 0
        || !runtime.execution.input_applications.is_empty()
    {
        return Err(SourceExecutionError::InvariantViolation {
            invariant: "closed V0.1 execution must create or apply zero input bridges",
        });
    }

    Ok(())
}

fn push_component(bytes: &mut Vec<u8>, component: &[u8]) {
    bytes.extend_from_slice(&(component.len() as u32).to_be_bytes());
    bytes.extend_from_slice(component);
}
