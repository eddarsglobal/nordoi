use super::error::{CompilerError, CompilerResult};
use super::hir::SemanticName;
use super::pure_binding::{
    compile_pure_binding_boundary, NsirPureBindingForm, NsirPureBindingUnit,
    SemanticPureBindingExpressionOp,
};
use super::symbols::{ResolvedEffectSet, SemanticEffectId};
use crate::frontend::SourceText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPureBindingExpression {
    ops: Vec<SemanticPureBindingExpressionOp>,
    value: i64,
}

impl PlannedPureBindingExpression {
    fn new(ops: Vec<SemanticPureBindingExpressionOp>, value: i64) -> Self {
        Self { ops, value }
    }

    pub fn ops(&self) -> &[SemanticPureBindingExpressionOp] {
        &self.ops
    }

    pub fn node_count(&self) -> usize {
        self.ops.len()
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureBindingEntryPlan {
    name: SemanticName,
    required_effects: ResolvedEffectSet,
    expression: Option<PlannedPureBindingExpression>,
}

impl PureBindingEntryPlan {
    fn new(
        name: SemanticName,
        required_effects: ResolvedEffectSet,
        expression: Option<PlannedPureBindingExpression>,
    ) -> Self {
        Self {
            name,
            required_effects,
            expression,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn required_effects(&self) -> &ResolvedEffectSet {
        &self.required_effects
    }

    pub fn expression(&self) -> Option<&PlannedPureBindingExpression> {
        self.expression.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.expression
            .as_ref()
            .map(PlannedPureBindingExpression::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureBindingPlanForm {
    Empty,
    Entry(PureBindingEntryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureBindingExecutionPlan {
    binding_semantics: NsirPureBindingUnit,
    form: PureBindingPlanForm,
}

impl PureBindingExecutionPlan {
    fn new(binding_semantics: NsirPureBindingUnit, form: PureBindingPlanForm) -> Self {
        Self {
            binding_semantics,
            form,
        }
    }

    pub fn binding_semantics(&self) -> &NsirPureBindingUnit {
        &self.binding_semantics
    }

    pub fn form(&self) -> &PureBindingPlanForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&PureBindingEntryPlan> {
        match &self.form {
            PureBindingPlanForm::Empty => None,
            PureBindingPlanForm::Entry(entry) => Some(entry),
        }
    }

    pub fn expression(&self) -> Option<&PlannedPureBindingExpression> {
        self.entry().and_then(PureBindingEntryPlan::expression)
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.entry().and_then(PureBindingEntryPlan::result_i64)
    }

    pub fn binding_count(&self) -> usize {
        self.binding_semantics.bindings().bindings().len()
    }

    pub fn work_item_count(&self) -> usize {
        0
    }

    pub fn runtime_storage_item_count(&self) -> usize {
        0
    }

    pub fn required_effects(&self) -> &[SemanticEffectId] {
        match &self.form {
            PureBindingPlanForm::Empty => &[],
            PureBindingPlanForm::Entry(entry) => entry.required_effects().effects(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects().is_empty()
    }

    pub fn requires_host_authority(&self) -> bool {
        false
    }

    /// C0.9 pure-binding execution-plan witness. It preserves L0.8 canonical binding
    /// identity and postfix references while defining zero runtime storage and zero work.
    /// It does not lower to NAIR, invoke runtime execution, or grant host authority.
    pub fn canonical_c09_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.9-PURE-BINDING-PLAN\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let l08 = self.binding_semantics.canonical_l08_bytes();
        bytes.extend_from_slice(&(l08.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&l08);

        match &self.form {
            PureBindingPlanForm::Empty => bytes.push(0),
            PureBindingPlanForm::Entry(entry) => {
                bytes.push(1);
                let name = entry.name().as_str().as_bytes();
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
                bytes.extend_from_slice(name);

                match entry.expression() {
                    None => bytes.push(0),
                    Some(expression) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&(expression.ops().len() as u32).to_be_bytes());
                        for op in expression.ops() {
                            match op {
                                SemanticPureBindingExpressionOp::Int(value) => {
                                    bytes.push(1);
                                    bytes.extend_from_slice(&value.to_be_bytes());
                                }
                                SemanticPureBindingExpressionOp::Binding(id) => {
                                    bytes.push(2);
                                    bytes.extend_from_slice(&id.get().to_be_bytes());
                                }
                                SemanticPureBindingExpressionOp::Add => bytes.push(3),
                            }
                        }
                        bytes.extend_from_slice(&expression.value().to_be_bytes());
                    }
                }
            }
        }

        bytes.extend_from_slice(&(self.binding_count() as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.work_item_count() as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.runtime_storage_item_count() as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.required_effects().len() as u32).to_be_bytes());
        for effect in self.required_effects() {
            bytes.extend_from_slice(&effect.get().to_be_bytes());
        }
        bytes.push(u8::from(self.requires_host_authority()));
        bytes
    }
}

pub fn validate_pure_binding_execution_plan(
    binding_semantics: NsirPureBindingUnit,
) -> CompilerResult<PureBindingExecutionPlan> {
    let form = match binding_semantics.form() {
        NsirPureBindingForm::Empty => PureBindingPlanForm::Empty,
        NsirPureBindingForm::Entry(entry) => {
            if !entry.required_effects().is_pure() {
                return Err(CompilerError::PureBindingPlanEntryMustBePure {
                    name: entry.name().as_str().to_owned(),
                });
            }

            let expression = entry.expression().map(|expression| {
                PlannedPureBindingExpression::new(expression.ops().to_vec(), expression.value())
            });

            PureBindingPlanForm::Entry(PureBindingEntryPlan::new(
                entry.name().clone(),
                entry.required_effects().clone(),
                expression,
            ))
        }
    };

    Ok(PureBindingExecutionPlan::new(binding_semantics, form))
}

/// C0.9 pure-binding execution-plan boundary. It plans fully resolved L0.8 immutable
/// binding semantics without allocating runtime storage, lowering to NAIR, invoking
/// runtime execution, performing I/O, dispatching effects, or granting authority.
pub fn compile_pure_binding_execution_plan_boundary(
    source: &SourceText,
) -> CompilerResult<PureBindingExecutionPlan> {
    validate_pure_binding_execution_plan(compile_pure_binding_boundary(source)?)
}
