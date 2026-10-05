use super::error::{CompilerError, CompilerResult};
use super::plan::{compile_execution_plan_boundary, SemanticExecutionPlan};
use crate::frontend::SourceText;
use crate::nair::{Instruction, NairProgram};

/// C0.4 compiler artifact that binds a certified C0.3 semantic plan to the exact
/// canonical NAIR 0.6 program produced from that plan.
///
/// The NAIR program is operational content only. Source/module/entry provenance
/// remains compiler metadata and is committed by `canonical_c04_bytes()` rather
/// than being injected into NAIR itself.
#[derive(Debug, Clone, PartialEq)]
pub struct NairLoweringArtifact {
    plan: SemanticExecutionPlan,
    program: NairProgram,
    canonical_nair: Vec<u8>,
}

impl NairLoweringArtifact {
    fn new(plan: SemanticExecutionPlan, program: NairProgram, canonical_nair: Vec<u8>) -> Self {
        Self {
            plan,
            program,
            canonical_nair,
        }
    }

    pub fn plan(&self) -> &SemanticExecutionPlan {
        &self.plan
    }

    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
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

    /// C0.4 compiler witness binding the exact C0.3 semantic plan to the exact
    /// canonical NAIR bytes emitted by this lowering boundary.
    ///
    /// This is not the NAIR wire format and must never be supplied to the NAIR
    /// decoder as though it were a program.
    pub fn canonical_c04_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.4-NAIR-LOWERING\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let plan = self.plan.canonical_c03_bytes();
        bytes.extend_from_slice(&(plan.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&plan);

        bytes.extend_from_slice(&(self.canonical_nair.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.canonical_nair);
        bytes
    }
}

/// Lower one already-validated C0.3 zero-work plan to NAIR 0.6.
///
/// C0.4 intentionally supports only the C0.3 semantic floor: zero semantic work,
/// zero required effects and zero host authority. The only emitted NAIR
/// instruction is `Halt`, because NAIR 0.6 requires a terminal halt even for a
/// no-work program.
pub fn lower_execution_plan_to_nair(
    plan: SemanticExecutionPlan,
) -> CompilerResult<NairLoweringArtifact> {
    if plan.work_item_count() != 0 {
        return Err(CompilerError::NairLoweringRequiresZeroWork {
            count: plan.work_item_count() as u32,
        });
    }
    if !plan.required_effects().is_empty() {
        return Err(CompilerError::NairLoweringRequiresPurePlan {
            count: plan.required_effects().len() as u32,
        });
    }
    if plan.requires_host_authority() {
        return Err(CompilerError::NairLoweringRequiresNoAuthority);
    }

    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    program
        .validate()
        .map_err(|error| CompilerError::NairLoweringValidationFailed {
            message: error.to_string(),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| CompilerError::NairLoweringValidationFailed {
                message: error.to_string(),
            })?;

    Ok(NairLoweringArtifact::new(plan, program, canonical_nair))
}

/// C0.4 source-to-NAIR lowering boundary. It performs frontend + semantic
/// validation through C0.3, lowers only the certified zero-work subset to
/// canonical NAIR 0.6, and does not execute the runtime.
pub fn compile_nair_lowering_boundary(source: &SourceText) -> CompilerResult<NairLoweringArtifact> {
    lower_execution_plan_to_nair(compile_execution_plan_boundary(source)?)
}
