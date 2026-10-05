use super::binding_plan::{compile_pure_binding_execution_plan_boundary, PureBindingExecutionPlan};
use super::error::{CompilerError, CompilerResult};
use super::pure_binding::{SemanticPureBindingExpressionOp, SemanticPureBindingId};
use crate::frontend::SourceText;
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::value::Value;

/// C0.10 compiler artifact binding the certified C0.9 pure-binding execution plan
/// to canonical NAIR without introducing runtime binding storage or lookup.
///
/// Pure binding references are erased at compile time to their certified immutable
/// values. `result_register` is compiler metadata only and grants no authority.
#[derive(Debug, Clone, PartialEq)]
pub struct PureBindingNairArtifact {
    plan: PureBindingExecutionPlan,
    program: NairProgram,
    canonical_nair: Vec<u8>,
    result_register: Option<RegisterId>,
}

impl PureBindingNairArtifact {
    fn new(
        plan: PureBindingExecutionPlan,
        program: NairProgram,
        canonical_nair: Vec<u8>,
        result_register: Option<RegisterId>,
    ) -> Self {
        Self {
            plan,
            program,
            canonical_nair,
            result_register,
        }
    }

    pub fn plan(&self) -> &PureBindingExecutionPlan {
        &self.plan
    }

    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn result_register(&self) -> Option<RegisterId> {
        self.result_register
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.plan.result_i64()
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.program.len()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.program.required_format_minor()
    }

    pub fn semantic_work_item_count(&self) -> usize {
        self.plan.work_item_count()
    }

    pub fn runtime_storage_item_count(&self) -> usize {
        self.plan.runtime_storage_item_count()
    }

    pub fn requires_host_authority(&self) -> bool {
        self.plan.requires_host_authority()
    }

    /// C0.10 compiler witness. The C0.9 plan preserves binding identity while
    /// the embedded canonical NAIR proves the zero-runtime-binding-cost lowering.
    /// This witness is compiler identity and is not itself a NAIR program.
    pub fn canonical_c010_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.10-PURE-BINDING-NAIR\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let plan = self.plan.canonical_c09_bytes();
        bytes.extend_from_slice(&(plan.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&plan);

        match self.result_register {
            None => bytes.push(0),
            Some(register) => {
                bytes.push(1);
                bytes.extend_from_slice(&register.0.to_be_bytes());
            }
        }

        bytes.extend_from_slice(&(self.canonical_nair.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.canonical_nair);
        bytes
    }
}

fn allocate_register(next: &mut u32) -> CompilerResult<RegisterId> {
    let register = RegisterId(*next);
    *next = (*next)
        .checked_add(1)
        .ok_or(CompilerError::PureBindingNairRegisterSpaceExhausted)?;
    Ok(register)
}

fn resolve_binding_value(
    plan: &PureBindingExecutionPlan,
    id: SemanticPureBindingId,
) -> CompilerResult<i64> {
    let raw = id.get();
    let index =
        raw.checked_sub(1)
            .ok_or(CompilerError::PureBindingNairUnknownBindingId { id: raw })? as usize;
    let binding = plan
        .binding_semantics()
        .bindings()
        .bindings()
        .get(index)
        .ok_or(CompilerError::PureBindingNairUnknownBindingId { id: raw })?;
    if binding.id() != id {
        return Err(CompilerError::PureBindingNairUnknownBindingId { id: raw });
    }
    Ok(binding.value())
}

/// Lower one already-validated C0.9 pure-binding execution plan to existing NAIR.
///
/// This is compile-time binding erasure, not runtime binding materialization:
/// - postfix `INT(v)` emits one `Const`;
/// - postfix `BINDING(id)` resolves the immutable L0.8 value at compile time and
///   emits exactly the same `Const` that the equivalent literal would emit;
/// - postfix `ADD` emits one existing N0.7 `IntAddChecked`;
/// - unused bindings emit no NAIR instruction at all;
/// - no binding table, lookup, atom, stack slot, heap object or new opcode exists.
pub fn lower_pure_binding_plan_to_nair(
    plan: PureBindingExecutionPlan,
) -> CompilerResult<PureBindingNairArtifact> {
    if plan.work_item_count() != 0 {
        return Err(CompilerError::PureBindingNairLoweringRequiresZeroWork {
            count: plan.work_item_count() as u32,
        });
    }
    if plan.runtime_storage_item_count() != 0 {
        return Err(CompilerError::PureBindingNairLoweringRequiresZeroStorage {
            count: plan.runtime_storage_item_count() as u32,
        });
    }
    if !plan.required_effects().is_empty() {
        return Err(CompilerError::PureBindingNairLoweringRequiresPurePlan {
            count: plan.required_effects().len() as u32,
        });
    }
    if plan.requires_host_authority() {
        return Err(CompilerError::PureBindingNairLoweringRequiresNoAuthority);
    }

    let mut instructions = Vec::new();
    let mut result_register = None;

    if let Some(expression) = plan.expression() {
        let mut next_register = 0_u32;
        let mut stack: Vec<RegisterId> = Vec::new();

        for op in expression.ops() {
            match op {
                SemanticPureBindingExpressionOp::Int(value) => {
                    let dst = allocate_register(&mut next_register)?;
                    instructions.push(Instruction::Const {
                        dst,
                        value: Value::Int(*value),
                    });
                    stack.push(dst);
                }
                SemanticPureBindingExpressionOp::Binding(id) => {
                    let value = resolve_binding_value(&plan, *id)?;
                    let dst = allocate_register(&mut next_register)?;
                    instructions.push(Instruction::Const {
                        dst,
                        value: Value::Int(value),
                    });
                    stack.push(dst);
                }
                SemanticPureBindingExpressionOp::Add => {
                    let rhs = stack
                        .pop()
                        .ok_or(CompilerError::PureBindingNairInvalidPostfix { depth: 0 })?;
                    let lhs = stack
                        .pop()
                        .ok_or(CompilerError::PureBindingNairInvalidPostfix { depth: 1 })?;
                    let dst = allocate_register(&mut next_register)?;
                    instructions.push(Instruction::IntAddChecked { dst, lhs, rhs });
                    stack.push(dst);
                }
            }
        }

        if stack.len() != 1 {
            return Err(CompilerError::PureBindingNairInvalidPostfix {
                depth: stack.len() as u32,
            });
        }
        result_register = stack.pop();
    }

    instructions.push(Instruction::Halt);
    let program = NairProgram::from_instructions(instructions);
    program.validate().map_err(
        |error| CompilerError::PureBindingNairLoweringValidationFailed {
            message: error.to_string(),
        },
    )?;
    let canonical_nair = program.canonical_bytes().map_err(|error| {
        CompilerError::PureBindingNairLoweringValidationFailed {
            message: error.to_string(),
        }
    })?;

    Ok(PureBindingNairArtifact::new(
        plan,
        program,
        canonical_nair,
        result_register,
    ))
}

/// C0.10 source-to-NAIR pure-binding lowering boundary. It compiles through
/// certified L0.8/C0.9, erases immutable binding names to compile-time values,
/// reuses only existing NAIR 0.6/0.7 instructions, and never invokes runtime
/// execution, performs I/O, dispatches effects, or grants host authority.
pub fn compile_pure_binding_nair_boundary(
    source: &SourceText,
) -> CompilerResult<PureBindingNairArtifact> {
    lower_pure_binding_plan_to_nair(compile_pure_binding_execution_plan_boundary(source)?)
}
