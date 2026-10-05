use super::error::{CompilerError, CompilerResult};
use super::result_plan::{
    compile_pure_result_execution_plan_boundary, PureResultExecutionPlan, PureResultPlanValue,
};
use crate::frontend::SourceText;
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::value::Value;

/// C0.6 compiler artifact binding a certified C0.5 pure-result execution plan
/// to the exact canonical NAIR 0.6 program used to carry its optional result.
///
/// `result_register` is compiler metadata identifying which NAIR register holds
/// the source-level pure result. It is not host authority, external output,
/// persistent state, or a new NAIR wire-format field.
#[derive(Debug, Clone, PartialEq)]
pub struct PureResultNairArtifact {
    plan: PureResultExecutionPlan,
    program: NairProgram,
    canonical_nair: Vec<u8>,
    result_register: Option<RegisterId>,
}

impl PureResultNairArtifact {
    fn new(
        plan: PureResultExecutionPlan,
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

    pub fn plan(&self) -> &PureResultExecutionPlan {
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

    pub fn semantic_work_item_count(&self) -> usize {
        self.plan.work_item_count()
    }

    pub fn requires_host_authority(&self) -> bool {
        self.plan.requires_host_authority()
    }

    /// C0.6 compiler witness binding the exact C0.5 plan, explicit result
    /// register binding, and exact canonical NAIR 0.6 bytes.
    ///
    /// This witness is compiler identity only. It is not itself a NAIR program
    /// and must never be supplied to the NAIR decoder.
    pub fn canonical_c06_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.6-PURE-RESULT-NAIR\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let plan = self.plan.canonical_c05_bytes();
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

/// Lower one already-validated C0.5 pure-result execution plan to NAIR 0.6.
///
/// C0.6 does not add a NAIR opcode. NAIR 0.6 already has `Const`, so a pure
/// integer result is represented canonically as `Const r0, Int(value)` followed
/// by the mandatory terminal `Halt`. Plans without a result remain `Halt` only.
/// The temporary register is not an atom, durable state, I/O, effect, capability,
/// or host authority.
pub fn lower_pure_result_plan_to_nair(
    plan: PureResultExecutionPlan,
) -> CompilerResult<PureResultNairArtifact> {
    if plan.work_item_count() != 0 {
        return Err(CompilerError::PureResultNairLoweringRequiresZeroWork {
            count: plan.work_item_count() as u32,
        });
    }
    if !plan.required_effects().is_empty() {
        return Err(CompilerError::PureResultNairLoweringRequiresPurePlan {
            count: plan.required_effects().len() as u32,
        });
    }
    if plan.requires_host_authority() {
        return Err(CompilerError::PureResultNairLoweringRequiresNoAuthority);
    }

    let (instructions, result_register) = match plan.result() {
        Some(PureResultPlanValue::Int(value)) => {
            let result_register = RegisterId(0);
            (
                vec![
                    Instruction::Const {
                        dst: result_register,
                        value: Value::Int(value),
                    },
                    Instruction::Halt,
                ],
                Some(result_register),
            )
        }
        None => (vec![Instruction::Halt], None),
    };

    let program = NairProgram::from_instructions(instructions);
    program.validate().map_err(
        |error| CompilerError::PureResultNairLoweringValidationFailed {
            message: error.to_string(),
        },
    )?;
    let canonical_nair = program.canonical_bytes().map_err(|error| {
        CompilerError::PureResultNairLoweringValidationFailed {
            message: error.to_string(),
        }
    })?;

    Ok(PureResultNairArtifact::new(
        plan,
        program,
        canonical_nair,
        result_register,
    ))
}

/// C0.6 source-to-NAIR pure-result lowering boundary. It performs frontend and
/// semantic validation through C0.5, lowers only the certified pure zero-work
/// subset to existing NAIR 0.6 primitives, and never invokes the runtime.
pub fn compile_pure_result_nair_boundary(
    source: &SourceText,
) -> CompilerResult<PureResultNairArtifact> {
    lower_pure_result_plan_to_nair(compile_pure_result_execution_plan_boundary(source)?)
}
