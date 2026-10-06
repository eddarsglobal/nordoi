use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{
    compile_pure_binding_nair_boundary, CompilerError, PureBindingNairArtifact,
    PureBindingPlanForm, SemanticPureBindingExpressionOp, SemanticPureBindingId,
};
use crate::frontend::{SourceSpan, SourceText};
use crate::input::{InputBatch, InputError};
use crate::nair::{Instruction, RegisterId};
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V04_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.4-PURE-BINDING-EXECUTION-RECEIPT\0";

#[derive(Debug)]
pub enum PureBindingExecutionError {
    Compiler(CompilerError),
    Runtime(RuntimeError),
    InvariantViolation { invariant: &'static str },
}

impl PureBindingExecutionError {
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

impl Display for PureBindingExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::InvariantViolation { invariant } => {
                write!(
                    f,
                    "V0.4 pure-binding execution invariant failed: {invariant}"
                )
            }
        }
    }
}

impl Error for PureBindingExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::InvariantViolation { .. } => None,
        }
    }
}

impl From<CompilerError> for PureBindingExecutionError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<RuntimeError> for PureBindingExecutionError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for PureBindingExecutionError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type PureBindingExecutionResult<T> = Result<T, PureBindingExecutionError>;

/// V0.4 source-to-runtime evidence for pure immutable bindings.
///
/// Binding names are erased by C0.10 before runtime execution. The report binds the
/// exact C0.10 compiler witness to one closed observed runtime execution and proves
/// that no binding-specific runtime storage, lookup, authority, effect, atom or other
/// persistent state was introduced.
#[derive(Debug, Clone, PartialEq)]
pub struct PureBindingExecutionReport {
    lowering: PureBindingNairArtifact,
    canonical_input: Vec<u8>,
    runtime: RuntimeObservedReport,
    result: Option<Value>,
}

impl PureBindingExecutionReport {
    fn new(
        lowering: PureBindingNairArtifact,
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

    pub fn lowering(&self) -> &PureBindingNairArtifact {
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

    pub fn binding_runtime_storage_item_count(&self) -> usize {
        0
    }

    pub fn binding_runtime_lookup_count(&self) -> usize {
        0
    }

    /// Deterministic V0.4 execution receipt.
    ///
    /// The receipt commits to the exact C0.10 compiler witness, canonical empty input,
    /// runtime replay key, validated transient SSA registers and closed execution shape.
    /// Operationally equivalent programs may share a runtime replay key while retaining
    /// distinct V0.4 receipts when their certified binding semantics differ.
    pub fn canonical_v04_receipt_bytes(&self) -> Vec<u8> {
        let c010 = self.lowering.canonical_c010_bytes();
        let runtime = self.runtime.runtime();
        let execution = &runtime.execution.execution;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(V04_RECEIPT_DOMAIN);
        push_component(&mut bytes, &c010);
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

        for (register, value) in self.runtime.final_registers() {
            bytes.extend_from_slice(&register.0.to_be_bytes());
            match value {
                Value::Int(value) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&value.to_be_bytes());
                }
                _ => bytes.push(0xff),
            }
        }

        match (self.lowering.result_register(), self.result.as_ref()) {
            (None, None) => bytes.push(0),
            (Some(register), Some(Value::Int(value))) => {
                bytes.push(1);
                bytes.extend_from_slice(&register.0.to_be_bytes());
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            _ => bytes.push(0xff),
        }

        bytes.extend_from_slice(&(self.binding_runtime_storage_item_count() as u64).to_be_bytes());
        bytes.extend_from_slice(&(self.binding_runtime_lookup_count() as u64).to_be_bytes());
        bytes.push(u8::from(self.runtime.is_quiescent()));
        bytes
    }
}

/// Execute the first certified source-level program containing pure named bindings.
///
/// V0.4 compiles through C0.10, supplies canonical empty input, executes the exact
/// existing NAIR program through the closed observed runtime, and validates that the
/// runtime contains only the transient SSA values required by the erased expression.
pub fn execute_pure_binding_source_v04(
    source: &SourceText,
) -> PureBindingExecutionResult<PureBindingExecutionReport> {
    let lowering = compile_pure_binding_nair_boundary(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;
    let result = validate_v04_execution(&lowering, &runtime)?;

    Ok(PureBindingExecutionReport::new(
        lowering,
        canonical_input,
        runtime,
        result,
    ))
}

/// Validate the closed runtime execution shape certified by V0.4.
pub fn validate_v04_execution(
    lowering: &PureBindingNairArtifact,
    observed: &RuntimeObservedReport,
) -> PureBindingExecutionResult<Option<Value>> {
    if lowering.semantic_work_item_count() != 0 {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "semantic work item count must remain zero",
        });
    }
    if lowering.runtime_storage_item_count() != 0 {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "pure bindings must require zero runtime storage",
        });
    }
    if !lowering.plan().required_effects().is_empty() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "required semantic effect count must remain zero",
        });
    }
    if lowering.requires_host_authority() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "host authority requirement must remain absent",
        });
    }

    let expected_result = lowering.result_i64();
    let result = match lowering.plan().expression() {
        Some(expression) => {
            let (expected_instructions, expected_registers, expected_result_register) =
                expected_binding_execution(lowering, expression.ops())?;

            if lowering.program().instructions() != expected_instructions.as_slice() {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant: "C0.10 NAIR must equal exact compile-time binding erasure",
                });
            }
            if lowering.result_register() != Some(expected_result_register) {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant: "C0.10 result register must equal the final postfix SSA register",
                });
            }
            if observed.final_registers() != &expected_registers {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant:
                        "runtime final registers must match every erased binding postfix SSA value",
                });
            }

            let observed_value = observed.register(expected_result_register).ok_or(
                PureBindingExecutionError::InvariantViolation {
                    invariant: "certified pure-binding result register must exist after execution",
                },
            )?;
            let expected_value =
                expected_result.ok_or(PureBindingExecutionError::InvariantViolation {
                    invariant: "pure-binding execution plan must carry a semantic result value",
                })?;
            if observed_value != &Value::Int(expected_value) {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant: "runtime pure-binding result register must equal semantic result",
                });
            }
            Some(observed_value.clone())
        }
        None => {
            if lowering.result_register().is_some() || expected_result.is_some() {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant: "result-free binding plan must not bind a result register or value",
                });
            }
            if lowering.program().instructions() != [Instruction::Halt] {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant:
                        "result-free C0.10 program must contain exactly one HALT instruction",
                });
            }
            if !observed.final_registers().is_empty() {
                return Err(PureBindingExecutionError::InvariantViolation {
                    invariant: "result-free V0.4 execution must publish zero final registers",
                });
            }
            None
        }
    };

    let runtime = observed.runtime();
    if runtime.input_events != 0 {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must consume zero input events",
        });
    }
    if !runtime.final_atoms.is_empty() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must publish zero final atoms",
        });
    }

    let execution = &runtime.execution.execution;
    if execution.executed_instructions != lowering.nair_instruction_count() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must execute the exact C0.10 instruction count",
        });
    }
    if execution.created_domains != 0 || !execution.domain_bindings.is_empty() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must create zero domains",
        });
    }
    if execution.created_atoms != 0 || !execution.atom_bindings.is_empty() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must create zero atoms",
        });
    }
    if execution.committed_transactions != 0
        || execution.rolled_back_transactions != 0
        || !execution.transaction_reports.is_empty()
    {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must perform zero transactions",
        });
    }
    if execution.scheduled_work != 0 || !observed.is_quiescent() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must end quiescent with zero scheduled work",
        });
    }
    if !runtime.execution.render_bindings.is_empty() || !runtime.execution.frames.is_empty() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must create zero render state or frames",
        });
    }
    if runtime.execution.created_input_bridges != 0
        || !runtime.execution.input_applications.is_empty()
    {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "closed V0.4 execution must create or apply zero input bridges",
        });
    }

    if matches!(lowering.plan().form(), PureBindingPlanForm::Empty) && result.is_some() {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "empty pure-binding plan must not publish a result",
        });
    }

    Ok(result)
}

fn expected_binding_execution(
    lowering: &PureBindingNairArtifact,
    ops: &[SemanticPureBindingExpressionOp],
) -> PureBindingExecutionResult<(Vec<Instruction>, BTreeMap<RegisterId, Value>, RegisterId)> {
    let mut instructions = Vec::with_capacity(ops.len() + 1);
    let mut registers = BTreeMap::new();
    let mut stack: Vec<(RegisterId, i64)> = Vec::new();

    for (index, op) in ops.iter().enumerate() {
        let register_index =
            u32::try_from(index).map_err(|_| PureBindingExecutionError::InvariantViolation {
                invariant: "pure-binding node index must fit in a NAIR register id",
            })?;
        let dst = RegisterId(register_index);

        match op {
            SemanticPureBindingExpressionOp::Int(value) => {
                instructions.push(Instruction::Const {
                    dst,
                    value: Value::Int(*value),
                });
                registers.insert(dst, Value::Int(*value));
                stack.push((dst, *value));
            }
            SemanticPureBindingExpressionOp::Binding(id) => {
                let value = resolve_binding_value(lowering, *id)?;
                instructions.push(Instruction::Const {
                    dst,
                    value: Value::Int(value),
                });
                registers.insert(dst, Value::Int(value));
                stack.push((dst, value));
            }
            SemanticPureBindingExpressionOp::Add => {
                let (rhs_register, rhs_value) =
                    stack
                        .pop()
                        .ok_or(PureBindingExecutionError::InvariantViolation {
                            invariant: "postfix ADD must have a right operand",
                        })?;
                let (lhs_register, lhs_value) =
                    stack
                        .pop()
                        .ok_or(PureBindingExecutionError::InvariantViolation {
                            invariant: "postfix ADD must have a left operand",
                        })?;
                let value = lhs_value.checked_add(rhs_value).ok_or(
                    PureBindingExecutionError::InvariantViolation {
                        invariant: "certified pure-binding expression must not overflow i64",
                    },
                )?;
                instructions.push(Instruction::IntAddChecked {
                    dst,
                    lhs: lhs_register,
                    rhs: rhs_register,
                });
                registers.insert(dst, Value::Int(value));
                stack.push((dst, value));
            }
        }
    }

    if stack.len() != 1 {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "certified pure-binding postfix expression must end with exactly one result",
        });
    }
    let (result_register, _) = stack.pop().expect("length checked above");
    instructions.push(Instruction::Halt);

    Ok((instructions, registers, result_register))
}

fn resolve_binding_value(
    lowering: &PureBindingNairArtifact,
    id: SemanticPureBindingId,
) -> PureBindingExecutionResult<i64> {
    let raw = id.get();
    let index = raw
        .checked_sub(1)
        .ok_or(PureBindingExecutionError::InvariantViolation {
            invariant: "binding id must be non-zero",
        })? as usize;
    let binding = lowering
        .plan()
        .binding_semantics()
        .bindings()
        .bindings()
        .get(index)
        .ok_or(PureBindingExecutionError::InvariantViolation {
            invariant: "binding id must resolve inside the certified L0.8 registry",
        })?;
    if binding.id() != id {
        return Err(PureBindingExecutionError::InvariantViolation {
            invariant: "binding registry order must match canonical binding ids",
        });
    }
    Ok(binding.value())
}

fn push_component(bytes: &mut Vec<u8>, component: &[u8]) {
    bytes.extend_from_slice(&(component.len() as u32).to_be_bytes());
    bytes.extend_from_slice(component);
}
