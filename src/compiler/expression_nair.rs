use super::error::{CompilerError, CompilerResult};
use super::expression_plan::{
    compile_pure_expression_execution_plan_boundary, PureExpressionExecutionPlan,
};
use super::pure_expression::SemanticPureExpressionOp;
use crate::frontend::SourceText;
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::value::Value;

/// C0.8 compiler artifact binding a certified C0.7 pure-expression execution plan
/// to the exact canonical NAIR program that preserves its postfix calculation order.
///
/// `result_register` is compiler metadata naming the final SSA register containing
/// the source-level pure result. It grants no authority and is not a NAIR wire field.
#[derive(Debug, Clone, PartialEq)]
pub struct PureExpressionNairArtifact {
    plan: PureExpressionExecutionPlan,
    program: NairProgram,
    canonical_nair: Vec<u8>,
    result_register: Option<RegisterId>,
}

impl PureExpressionNairArtifact {
    fn new(
        plan: PureExpressionExecutionPlan,
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

    pub fn plan(&self) -> &PureExpressionExecutionPlan {
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

    pub fn requires_host_authority(&self) -> bool {
        self.plan.requires_host_authority()
    }

    /// C0.8 compiler witness binding the exact C0.7 plan, final result register,
    /// and exact canonical NAIR bytes. This is compiler identity only and is not
    /// itself a NAIR program.
    pub fn canonical_c08_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.8-PURE-EXPRESSION-NAIR\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let plan = self.plan.canonical_c07_bytes();
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
        .ok_or(CompilerError::PureExpressionNairRegisterSpaceExhausted)?;
    Ok(register)
}

/// Lower one already-validated C0.7 pure-expression execution plan to NAIR.
///
/// The lowering is structurally faithful and performs no hidden constant folding:
/// every postfix `INT(v)` becomes one `Const`, every postfix `ADD` becomes one
/// `IntAddChecked`, and a final `Halt` terminates the program. Register allocation
/// is deterministic SSA order (`r0`, `r1`, ...). Expressions not using arithmetic
/// retain the certified NAIR 0.6 wire header; expressions using `IntAddChecked`
/// require N0.7 / NAIR 0.7 automatically through `NairProgram::required_format_minor()`.
pub fn lower_pure_expression_plan_to_nair(
    plan: PureExpressionExecutionPlan,
) -> CompilerResult<PureExpressionNairArtifact> {
    if plan.work_item_count() != 0 {
        return Err(CompilerError::PureExpressionNairLoweringRequiresZeroWork {
            count: plan.work_item_count() as u32,
        });
    }
    if !plan.required_effects().is_empty() {
        return Err(CompilerError::PureExpressionNairLoweringRequiresPurePlan {
            count: plan.required_effects().len() as u32,
        });
    }
    if plan.requires_host_authority() {
        return Err(CompilerError::PureExpressionNairLoweringRequiresNoAuthority);
    }

    let mut instructions = Vec::new();
    let mut result_register = None;

    if let Some(expression) = plan.expression() {
        let mut next_register = 0_u32;
        let mut stack: Vec<RegisterId> = Vec::new();

        for op in expression.ops() {
            match op {
                SemanticPureExpressionOp::Int(value) => {
                    let dst = allocate_register(&mut next_register)?;
                    instructions.push(Instruction::Const {
                        dst,
                        value: Value::Int(*value),
                    });
                    stack.push(dst);
                }
                SemanticPureExpressionOp::Add => {
                    let rhs = stack
                        .pop()
                        .ok_or(CompilerError::PureExpressionNairInvalidPostfix { depth: 0 })?;
                    let lhs = stack
                        .pop()
                        .ok_or(CompilerError::PureExpressionNairInvalidPostfix { depth: 1 })?;
                    let dst = allocate_register(&mut next_register)?;
                    instructions.push(Instruction::IntAddChecked { dst, lhs, rhs });
                    stack.push(dst);
                }
            }
        }

        if stack.len() != 1 {
            return Err(CompilerError::PureExpressionNairInvalidPostfix {
                depth: stack.len() as u32,
            });
        }
        result_register = stack.pop();
    }

    instructions.push(Instruction::Halt);
    let program = NairProgram::from_instructions(instructions);
    program.validate().map_err(|error| {
        CompilerError::PureExpressionNairLoweringValidationFailed {
            message: error.to_string(),
        }
    })?;
    let canonical_nair = program.canonical_bytes().map_err(|error| {
        CompilerError::PureExpressionNairLoweringValidationFailed {
            message: error.to_string(),
        }
    })?;

    Ok(PureExpressionNairArtifact::new(
        plan,
        program,
        canonical_nair,
        result_register,
    ))
}

/// C0.8 source-to-NAIR pure-expression lowering boundary. It compiles through
/// certified L0.7/C0.7, preserves the exact postfix calculation order in NAIR,
/// and never invokes the runtime, performs I/O, dispatches effects, consults
/// capabilities, or grants host authority.
pub fn compile_pure_expression_nair_boundary(
    source: &SourceText,
) -> CompilerResult<PureExpressionNairArtifact> {
    lower_pure_expression_plan_to_nair(compile_pure_expression_execution_plan_boundary(source)?)
}
