use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{
    compile_pure_expression_nair_boundary, CompilerError, PureExpressionNairArtifact,
    PureExpressionPlanForm, SemanticPureExpressionOp,
};
use crate::frontend::{SourceSpan, SourceText};
use crate::input::{InputBatch, InputError};
use crate::nair::{Instruction, RegisterId};
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V03_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.3-PURE-EXPRESSION-EXECUTION-RECEIPT\0";

#[derive(Debug)]
pub enum PureExpressionExecutionError {
    Compiler(CompilerError),
    Runtime(RuntimeError),
    InvariantViolation { invariant: &'static str },
}

impl PureExpressionExecutionError {
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

impl Display for PureExpressionExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::InvariantViolation { invariant } => {
                write!(
                    f,
                    "V0.3 pure-expression execution invariant failed: {invariant}"
                )
            }
        }
    }
}

impl Error for PureExpressionExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::InvariantViolation { .. } => None,
        }
    }
}

impl From<CompilerError> for PureExpressionExecutionError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<RuntimeError> for PureExpressionExecutionError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for PureExpressionExecutionError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type PureExpressionExecutionResult<T> = Result<T, PureExpressionExecutionError>;

/// V0.3 source-to-runtime evidence for a pure expression.
///
/// The report binds the certified C0.8 lowering artifact to one closed runtime
/// execution with canonical empty input and validates every transient SSA register
/// produced by the postfix expression. It is not a program format, checkpoint,
/// host output channel, effect, capability, or authority grant.
#[derive(Debug, Clone, PartialEq)]
pub struct PureExpressionExecutionReport {
    lowering: PureExpressionNairArtifact,
    canonical_input: Vec<u8>,
    runtime: RuntimeObservedReport,
    result: Option<Value>,
}

impl PureExpressionExecutionReport {
    fn new(
        lowering: PureExpressionNairArtifact,
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

    pub fn lowering(&self) -> &PureExpressionNairArtifact {
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

    /// Deterministic V0.3 execution receipt.
    ///
    /// The receipt commits to the exact C0.8 compiler witness, canonical empty input,
    /// runtime replay key, every validated transient SSA register/value pair, and the
    /// closed zero-authority execution summary.
    pub fn canonical_v03_receipt_bytes(&self) -> Vec<u8> {
        let c08 = self.lowering.canonical_c08_bytes();
        let runtime = self.runtime.runtime();
        let execution = &runtime.execution.execution;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(V03_RECEIPT_DOMAIN);
        push_component(&mut bytes, &c08);
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

        bytes.push(u8::from(self.runtime.is_quiescent()));
        bytes
    }
}

/// Execute the first certified source-level pure expression end-to-end.
///
/// V0.3 compiles through C0.8, supplies canonical empty input, executes the exact
/// NAIR 0.6/0.7 program through the closed runtime with transient register
/// observation, and validates every SSA register against the certified postfix plan.
pub fn execute_pure_expression_source_v03(
    source: &SourceText,
) -> PureExpressionExecutionResult<PureExpressionExecutionReport> {
    let lowering = compile_pure_expression_nair_boundary(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;

    let result = validate_v03_execution(&lowering, &runtime)?;

    Ok(PureExpressionExecutionReport::new(
        lowering,
        canonical_input,
        runtime,
        result,
    ))
}

/// Validate the exact closed-runtime shape certified by V0.3 and return the
/// observed source-level result after successful validation.
pub fn validate_v03_execution(
    lowering: &PureExpressionNairArtifact,
    observed: &RuntimeObservedReport,
) -> PureExpressionExecutionResult<Option<Value>> {
    if lowering.semantic_work_item_count() != 0 {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "semantic work item count must remain zero",
        });
    }
    if !lowering.plan().required_effects().is_empty() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "required semantic effect count must remain zero",
        });
    }
    if lowering.requires_host_authority() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "host authority requirement must remain absent",
        });
    }

    let expected_result = lowering.result_i64();
    let result = match lowering.plan().expression() {
        Some(expression) => {
            let (expected_instructions, expected_registers, expected_result_register) =
                expected_expression_execution(expression.ops())?;

            if lowering.program().instructions() != expected_instructions.as_slice() {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant: "C0.8 NAIR must preserve the exact postfix instruction sequence",
                });
            }
            if lowering.result_register() != Some(expected_result_register) {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant: "C0.8 result register must equal the final postfix SSA register",
                });
            }
            if observed.final_registers() != &expected_registers {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant:
                        "runtime final registers must match every certified postfix SSA value",
                });
            }

            let observed_value = observed.register(expected_result_register).ok_or(
                PureExpressionExecutionError::InvariantViolation {
                    invariant: "certified expression result register must exist after execution",
                },
            )?;
            let expected_value =
                expected_result.ok_or(PureExpressionExecutionError::InvariantViolation {
                    invariant: "expression plan must carry a semantic result value",
                })?;
            if observed_value != &Value::Int(expected_value) {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant: "runtime expression result register must equal semantic result",
                });
            }
            Some(observed_value.clone())
        }
        None => {
            if lowering.result_register().is_some() || expected_result.is_some() {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant:
                        "result-free expression plan must not bind a result register or value",
                });
            }
            if lowering.program().instructions() != [Instruction::Halt] {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant: "result-free C0.8 program must contain exactly one HALT instruction",
                });
            }
            if !observed.final_registers().is_empty() {
                return Err(PureExpressionExecutionError::InvariantViolation {
                    invariant: "result-free V0.3 execution must publish zero final registers",
                });
            }
            None
        }
    };

    let runtime = observed.runtime();
    if runtime.input_events != 0 {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must consume zero input events",
        });
    }
    if !runtime.final_atoms.is_empty() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must publish zero final atoms",
        });
    }

    let execution = &runtime.execution.execution;
    if execution.executed_instructions != lowering.nair_instruction_count() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must execute the exact C0.8 instruction count",
        });
    }
    if execution.created_domains != 0 || !execution.domain_bindings.is_empty() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must create zero domains",
        });
    }
    if execution.created_atoms != 0 || !execution.atom_bindings.is_empty() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must create zero atoms",
        });
    }
    if execution.committed_transactions != 0
        || execution.rolled_back_transactions != 0
        || !execution.transaction_reports.is_empty()
    {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must perform zero transactions",
        });
    }
    if execution.scheduled_work != 0 || !observed.is_quiescent() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must end quiescent with zero scheduled work",
        });
    }
    if !runtime.execution.render_bindings.is_empty() || !runtime.execution.frames.is_empty() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must create zero render state or frames",
        });
    }
    if runtime.execution.created_input_bridges != 0
        || !runtime.execution.input_applications.is_empty()
    {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "closed V0.3 execution must create or apply zero input bridges",
        });
    }

    if matches!(lowering.plan().form(), PureExpressionPlanForm::Empty) && result.is_some() {
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "empty pure-expression plan must not publish a result",
        });
    }

    Ok(result)
}

fn expected_expression_execution(
    ops: &[SemanticPureExpressionOp],
) -> PureExpressionExecutionResult<(
    Vec<Instruction>,
    std::collections::BTreeMap<RegisterId, Value>,
    RegisterId,
)> {
    let mut instructions = Vec::with_capacity(ops.len() + 1);
    let mut registers = std::collections::BTreeMap::new();
    let mut stack: Vec<(RegisterId, i64)> = Vec::new();

    for (index, op) in ops.iter().enumerate() {
        let register_index =
            u32::try_from(index).map_err(|_| PureExpressionExecutionError::InvariantViolation {
                invariant: "pure-expression node index must fit in a NAIR register id",
            })?;
        let dst = RegisterId(register_index);

        match op {
            SemanticPureExpressionOp::Int(value) => {
                instructions.push(Instruction::Const {
                    dst,
                    value: Value::Int(*value),
                });
                registers.insert(dst, Value::Int(*value));
                stack.push((dst, *value));
            }
            SemanticPureExpressionOp::Add => {
                let (rhs_register, rhs_value) =
                    stack
                        .pop()
                        .ok_or(PureExpressionExecutionError::InvariantViolation {
                            invariant: "postfix ADD must have a right operand",
                        })?;
                let (lhs_register, lhs_value) =
                    stack
                        .pop()
                        .ok_or(PureExpressionExecutionError::InvariantViolation {
                            invariant: "postfix ADD must have a left operand",
                        })?;
                let value = lhs_value.checked_add(rhs_value).ok_or(
                    PureExpressionExecutionError::InvariantViolation {
                        invariant: "certified pure expression must not overflow i64",
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
        return Err(PureExpressionExecutionError::InvariantViolation {
            invariant: "certified postfix expression must end with exactly one result",
        });
    }
    let (result_register, _) = stack.pop().expect("length checked above");
    instructions.push(Instruction::Halt);

    Ok((instructions, registers, result_register))
}

fn push_component(bytes: &mut Vec<u8>, component: &[u8]) {
    bytes.extend_from_slice(&(component.len() as u32).to_be_bytes());
    bytes.extend_from_slice(component);
}
