use super::error::{CompilerError, CompilerResult};
use super::hir::SemanticName;
use super::pure_expression::{
    compile_pure_expression_boundary, NsirPureExpressionForm, NsirPureExpressionUnit,
    SemanticPureExpressionOp,
};
use super::symbols::{ResolvedEffectSet, SemanticEffectId};
use crate::frontend::SourceText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPureExpression {
    ops: Vec<SemanticPureExpressionOp>,
    value: i64,
}

impl PlannedPureExpression {
    fn new(ops: Vec<SemanticPureExpressionOp>, value: i64) -> Self {
        Self { ops, value }
    }

    pub fn ops(&self) -> &[SemanticPureExpressionOp] {
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
pub struct PureExpressionEntryPlan {
    name: SemanticName,
    required_effects: ResolvedEffectSet,
    expression: Option<PlannedPureExpression>,
}

impl PureExpressionEntryPlan {
    fn new(
        name: SemanticName,
        required_effects: ResolvedEffectSet,
        expression: Option<PlannedPureExpression>,
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

    pub fn expression(&self) -> Option<&PlannedPureExpression> {
        self.expression.as_ref()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.expression.as_ref().map(PlannedPureExpression::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureExpressionPlanForm {
    Empty,
    Entry(PureExpressionEntryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureExpressionExecutionPlan {
    expression_semantics: NsirPureExpressionUnit,
    form: PureExpressionPlanForm,
}

impl PureExpressionExecutionPlan {
    fn new(expression_semantics: NsirPureExpressionUnit, form: PureExpressionPlanForm) -> Self {
        Self {
            expression_semantics,
            form,
        }
    }

    pub fn expression_semantics(&self) -> &NsirPureExpressionUnit {
        &self.expression_semantics
    }

    pub fn form(&self) -> &PureExpressionPlanForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&PureExpressionEntryPlan> {
        match &self.form {
            PureExpressionPlanForm::Empty => None,
            PureExpressionPlanForm::Entry(entry) => Some(entry),
        }
    }

    pub fn expression(&self) -> Option<&PlannedPureExpression> {
        self.entry().and_then(PureExpressionEntryPlan::expression)
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.entry().and_then(PureExpressionEntryPlan::result_i64)
    }

    pub fn work_item_count(&self) -> usize {
        0
    }

    pub fn required_effects(&self) -> &[SemanticEffectId] {
        match &self.form {
            PureExpressionPlanForm::Empty => &[],
            PureExpressionPlanForm::Entry(entry) => entry.required_effects().effects(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects().is_empty()
    }

    pub fn requires_host_authority(&self) -> bool {
        false
    }

    /// C0.7 pure-expression execution-plan witness. It is compiler identity only.
    /// It preserves the exact L0.7 postfix evaluation order but does not lower to NAIR,
    /// prove runtime execution, encode source spans, or grant host authority.
    pub fn canonical_c07_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.7-PURE-EXPRESSION-PLAN\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let l07 = self.expression_semantics.canonical_l07_bytes();
        bytes.extend_from_slice(&(l07.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&l07);

        match &self.form {
            PureExpressionPlanForm::Empty => bytes.push(0),
            PureExpressionPlanForm::Entry(entry) => {
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
                                SemanticPureExpressionOp::Int(value) => {
                                    bytes.push(1);
                                    bytes.extend_from_slice(&value.to_be_bytes());
                                }
                                SemanticPureExpressionOp::Add => bytes.push(2),
                            }
                        }
                        bytes.extend_from_slice(&expression.value().to_be_bytes());
                    }
                }
            }
        }

        bytes.extend_from_slice(&(self.work_item_count() as u32).to_be_bytes());
        bytes.extend_from_slice(&(self.required_effects().len() as u32).to_be_bytes());
        for effect in self.required_effects() {
            bytes.extend_from_slice(&effect.get().to_be_bytes());
        }
        bytes.push(u8::from(self.requires_host_authority()));
        bytes
    }
}

pub fn validate_pure_expression_execution_plan(
    expression_semantics: NsirPureExpressionUnit,
) -> CompilerResult<PureExpressionExecutionPlan> {
    let form = match expression_semantics.form() {
        NsirPureExpressionForm::Empty => PureExpressionPlanForm::Empty,
        NsirPureExpressionForm::Entry(entry) => {
            if !entry.required_effects().is_pure() {
                return Err(CompilerError::PureExpressionPlanEntryMustBePure {
                    name: entry.name().as_str().to_owned(),
                });
            }

            let expression = entry.expression().map(|expression| {
                PlannedPureExpression::new(expression.ops().to_vec(), expression.value())
            });
            PureExpressionPlanForm::Entry(PureExpressionEntryPlan::new(
                entry.name().clone(),
                entry.required_effects().clone(),
                expression,
            ))
        }
    };

    Ok(PureExpressionExecutionPlan::new(expression_semantics, form))
}

/// C0.7 pure-expression execution-plan boundary. It plans the fully understood L0.7 postfix
/// semantics without changing C0.5/C0.6/V0.2, lowering to NAIR, invoking the runtime,
/// performing I/O, dispatching effects, consulting capabilities, or granting host authority.
pub fn compile_pure_expression_execution_plan_boundary(
    source: &SourceText,
) -> CompilerResult<PureExpressionExecutionPlan> {
    validate_pure_expression_execution_plan(compile_pure_expression_boundary(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{
        compile_pure_expression_boundary, resolve_effect_set, NsirPureExpressionEntry,
        SemanticEffectSet,
    };
    use crate::frontend::{SourceId, SourceText};

    #[test]
    fn impure_manual_expression_plan_fails_closed() {
        let source = SourceText::new(
            SourceId::new(970),
            "impure-expression.noi",
            "effect Net; entry main returns 1 + 2;",
        )
        .unwrap();
        let compiled = compile_pure_expression_boundary(&source).unwrap();
        let semantic = compiled.semantic().clone();
        let original = compiled.entry().unwrap();
        let requirements = SemanticEffectSet::new(vec![SemanticName::new("Net").unwrap()]).unwrap();
        let resolved = resolve_effect_set(semantic.registry(), &requirements).unwrap();
        let impure = NsirPureExpressionEntry::new(
            original.name().clone(),
            original.origin_span(),
            resolved,
            original.expression().cloned(),
        );
        let malformed =
            NsirPureExpressionUnit::new(semantic, NsirPureExpressionForm::Entry(impure));

        assert!(matches!(
            validate_pure_expression_execution_plan(malformed),
            Err(CompilerError::PureExpressionPlanEntryMustBePure { .. })
        ));
    }
}
