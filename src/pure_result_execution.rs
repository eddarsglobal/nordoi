use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{
    compile_pure_result_nair_boundary, CompilerError, PureResultNairArtifact, PureResultPlanForm,
};
use crate::frontend::{SourceSpan, SourceText};
use crate::input::{InputBatch, InputError};
use crate::nair::Instruction;
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V02_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.2-PURE-RESULT-EXECUTION-RECEIPT\0";

#[derive(Debug)]
pub enum PureResultExecutionError {
    Compiler(CompilerError),
    Runtime(RuntimeError),
    InvariantViolation { invariant: &'static str },
}

impl PureResultExecutionError {
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

impl Display for PureResultExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::InvariantViolation { invariant } => {
                write!(
                    f,
                    "V0.2 pure-result execution invariant failed: {invariant}"
                )
            }
        }
    }
}

impl Error for PureResultExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::InvariantViolation { .. } => None,
        }
    }
}

impl From<CompilerError> for PureResultExecutionError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<RuntimeError> for PureResultExecutionError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for PureResultExecutionError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type PureResultExecutionResult<T> = Result<T, PureResultExecutionError>;

/// V0.2 source-to-runtime evidence for a pure result.
///
/// The report binds the certified C0.6 lowering artifact to one closed runtime
/// execution with canonical empty input and a validated transient result-register
/// observation. It is not a program format, checkpoint, host output channel, effect,
/// capability, or authority grant.
#[derive(Debug, Clone, PartialEq)]
pub struct PureResultExecutionReport {
    lowering: PureResultNairArtifact,
    canonical_input: Vec<u8>,
    runtime: RuntimeObservedReport,
    result: Option<Value>,
}

impl PureResultExecutionReport {
    fn new(
        lowering: PureResultNairArtifact,
        canonical_input: Vec<u8>,
        runtime: RuntimeObservedReport,
        result: Option<Value>,
    ) -> Self {
        Self {
            lowering,
            canonical_input,
            runtime,
            result,
        }
    }

    pub fn lowering(&self) -> &PureResultNairArtifact {
        &self.lowering
    }

    pub fn canonical_input_bytes(&self) -> &[u8] {
        &self.canonical_input
    }

    pub fn runtime(&self) -> &RuntimeObservedReport {
        &self.runtime
    }

    pub fn result(&self) -> Option<&Value> {
        self.result.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        if let Some(Value::Int(value)) = self.result.as_ref() {
            Some(*value)
        } else {
            None
        }
    }

    pub fn is_quiescent(&self) -> bool {
        self.runtime.is_quiescent()
    }

    /// Deterministic V0.2 execution receipt.
    ///
    /// The receipt commits to the exact C0.6 compiler witness, canonical empty
    /// input, runtime replay key, transient result-register observation, and the
    /// bounded zero-authority execution summary validated by V0.2.
    pub fn canonical_v02_receipt_bytes(&self) -> Vec<u8> {
        let c06 = self.lowering.canonical_c06_bytes();
        let runtime = self.runtime.runtime();
        let execution = &runtime.execution.execution;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(V02_RECEIPT_DOMAIN);
        push_component(&mut bytes, &c06);
        push_component(&mut bytes, &self.canonical_input);
        bytes.extend_from_slice(&runtime.replay_key.value().to_be_bytes());
        bytes.extend_from_slice(&(runtime.input_events as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.executed_instructions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.created_domains as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.created_atoms as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.committed_transactions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.rolled_back_transactions as u64).to_be_bytes());
        bytes.extend_from_slice(&(execution.scheduled_work as u64).to_be_bytes());
        bytes.extend_from_slice(&(runtime.execution.frames.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&(runtime.execution.created_input_bridges as u64).to_be_bytes());
        bytes.extend_from_slice(&(self.runtime.final_registers().len() as u64).to_be_bytes());

        match (self.lowering.result_register(), self.result.as_ref()) {
            (None, None) => bytes.push(0),
            (Some(register), Some(Value::Int(value))) => {
                bytes.push(1);
                bytes.extend_from_slice(&register.0.to_be_bytes());
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            _ => bytes.push(0xff),
        }

        bytes.push(u8::from(self.runtime.is_quiescent()));
        bytes
    }
}

/// Execute the first certified source-level pure result end-to-end.
///
/// V0.2 compiles through C0.6, supplies canonical empty input, executes the exact
/// NAIR 0.6 program through the closed runtime with transient register observation,
/// and validates the source result against the actual final result register.
pub fn execute_pure_result_source_v02(
    source: &SourceText,
) -> PureResultExecutionResult<PureResultExecutionReport> {
    let lowering = compile_pure_result_nair_boundary(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;

    let result = validate_v02_execution(&lowering, &runtime)?;

    Ok(PureResultExecutionReport::new(
        lowering,
        canonical_input,
        runtime,
        result,
    ))
}

/// Validate the exact closed-runtime shape certified by V0.2 and return the
/// observed source-level result after successful validation.
pub fn validate_v02_execution(
    lowering: &PureResultNairArtifact,
    observed: &RuntimeObservedReport,
) -> PureResultExecutionResult<Option<Value>> {
    if lowering.semantic_work_item_count() != 0 {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "semantic work item count must remain zero",
        });
    }
    if !lowering.plan().required_effects().is_empty() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "required semantic effect count must remain zero",
        });
    }
    if lowering.requires_host_authority() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "host authority requirement must remain absent",
        });
    }

    let expected_result = lowering.result_i64();
    let result = match (expected_result, lowering.result_register()) {
        (Some(value), Some(register)) => {
            let expected = [
                Instruction::Const {
                    dst: register,
                    value: Value::Int(value),
                },
                Instruction::Halt,
            ];
            if lowering.program().instructions() != expected {
                return Err(PureResultExecutionError::InvariantViolation {
                    invariant: "result-bearing C0.6 program must be exactly CONST(result-register, value), HALT",
                });
            }
            if observed.final_registers().len() != 1 {
                return Err(PureResultExecutionError::InvariantViolation {
                    invariant:
                        "result-bearing V0.2 execution must publish exactly one final register",
                });
            }
            let observed_value = observed.register(register).ok_or(
                PureResultExecutionError::InvariantViolation {
                    invariant: "certified result register must exist after runtime execution",
                },
            )?;
            if observed_value != &Value::Int(value) {
                return Err(PureResultExecutionError::InvariantViolation {
                    invariant: "runtime result register must equal the source-level pure result",
                });
            }
            Some(observed_value.clone())
        }
        (None, None) => {
            if lowering.program().instructions() != [Instruction::Halt] {
                return Err(PureResultExecutionError::InvariantViolation {
                    invariant: "result-free C0.6 program must contain exactly one HALT instruction",
                });
            }
            if !observed.final_registers().is_empty() {
                return Err(PureResultExecutionError::InvariantViolation {
                    invariant: "result-free V0.2 execution must publish zero final registers",
                });
            }
            None
        }
        _ => {
            return Err(PureResultExecutionError::InvariantViolation {
                invariant: "C0.6 result value and result-register binding must agree",
            });
        }
    };

    let runtime = observed.runtime();
    if runtime.input_events != 0 {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must consume zero input events",
        });
    }
    if !runtime.final_atoms.is_empty() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must publish zero final atoms",
        });
    }

    let execution = &runtime.execution.execution;
    let expected_instruction_count = if expected_result.is_some() { 2 } else { 1 };
    if execution.executed_instructions != expected_instruction_count {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must execute the exact C0.6 instruction count",
        });
    }
    if execution.created_domains != 0 || !execution.domain_bindings.is_empty() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must create zero domains",
        });
    }
    if execution.created_atoms != 0 || !execution.atom_bindings.is_empty() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must create zero atoms",
        });
    }
    if execution.committed_transactions != 0
        || execution.rolled_back_transactions != 0
        || !execution.transaction_reports.is_empty()
    {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must perform zero transactions",
        });
    }
    if execution.scheduled_work != 0 || !observed.is_quiescent() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must end quiescent with zero scheduled work",
        });
    }
    if !runtime.execution.render_bindings.is_empty() || !runtime.execution.frames.is_empty() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must create zero render state or frames",
        });
    }
    if runtime.execution.created_input_bridges != 0
        || !runtime.execution.input_applications.is_empty()
    {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "closed V0.2 execution must create or apply zero input bridges",
        });
    }

    if matches!(lowering.plan().form(), PureResultPlanForm::Empty) && result.is_some() {
        return Err(PureResultExecutionError::InvariantViolation {
            invariant: "empty pure-result plan must not publish a result",
        });
    }

    Ok(result)
}

fn push_component(bytes: &mut Vec<u8>, component: &[u8]) {
    bytes.extend_from_slice(&(component.len() as u32).to_be_bytes());
    bytes.extend_from_slice(component);
}
