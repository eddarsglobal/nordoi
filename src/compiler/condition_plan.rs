use super::hir::SemanticName;
use super::pure_condition::{
    compile_pure_condition_boundary, NsirPureConditionForm, NsirPureConditionUnit,
    SemanticPureComparator, SemanticPureCondition,
};
use super::symbols::{ResolvedEffectSet, SemanticEffectId};
use crate::frontend::SourceText;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPureCondition {
    condition: SemanticPureCondition,
    value: bool,
}

impl PlannedPureCondition {
    fn new(condition: SemanticPureCondition, value: bool) -> Self {
        Self { condition, value }
    }

    pub fn condition(&self) -> &SemanticPureCondition {
        &self.condition
    }

    pub fn value(&self) -> bool {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureConditionEntryPlan {
    name: SemanticName,
    required_effects: ResolvedEffectSet,
    condition: Option<PlannedPureCondition>,
}

impl PureConditionEntryPlan {
    fn new(
        name: SemanticName,
        required_effects: ResolvedEffectSet,
        condition: Option<PlannedPureCondition>,
    ) -> Self {
        Self {
            name,
            required_effects,
            condition,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn required_effects(&self) -> &ResolvedEffectSet {
        &self.required_effects
    }

    pub fn condition(&self) -> Option<&PlannedPureCondition> {
        self.condition.as_ref()
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.condition.as_ref().map(PlannedPureCondition::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureConditionPlanForm {
    Empty,
    Entry(PureConditionEntryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureConditionExecutionPlan {
    condition_semantics: NsirPureConditionUnit,
    form: PureConditionPlanForm,
}

impl PureConditionExecutionPlan {
    fn new(condition_semantics: NsirPureConditionUnit, form: PureConditionPlanForm) -> Self {
        Self {
            condition_semantics,
            form,
        }
    }

    pub fn condition_semantics(&self) -> &NsirPureConditionUnit {
        &self.condition_semantics
    }

    pub fn form(&self) -> &PureConditionPlanForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&PureConditionEntryPlan> {
        match &self.form {
            PureConditionPlanForm::Empty => None,
            PureConditionPlanForm::Entry(entry) => Some(entry),
        }
    }

    pub fn condition(&self) -> Option<&PlannedPureCondition> {
        self.entry().and_then(PureConditionEntryPlan::condition)
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.entry().and_then(PureConditionEntryPlan::result_bool)
    }

    pub fn work_item_count(&self) -> usize {
        0
    }

    pub fn runtime_storage_item_count(&self) -> usize {
        0
    }

    pub fn required_effects(&self) -> &[SemanticEffectId] {
        match &self.form {
            PureConditionPlanForm::Empty => &[],
            PureConditionPlanForm::Entry(entry) => entry.required_effects().effects(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects().is_empty()
    }

    pub fn requires_host_authority(&self) -> bool {
        false
    }

    /// C0.11 pure-condition execution-plan witness. It preserves the exact L0.9
    /// condition identity and truth value while defining zero runtime storage and
    /// zero work. It does not define branches, lower to NAIR, invoke runtime work,
    /// perform I/O, or grant host authority.
    pub fn canonical_c011_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.11-PURE-CONDITION-PLAN\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let l09 = self.condition_semantics.canonical_l09_bytes();
        bytes.extend_from_slice(&(l09.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&l09);

        match &self.form {
            PureConditionPlanForm::Empty => bytes.push(0),
            PureConditionPlanForm::Entry(entry) => {
                bytes.push(1);
                let name = entry.name().as_str().as_bytes();
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
                bytes.extend_from_slice(name);

                match entry.condition() {
                    None => bytes.push(0),
                    Some(planned) => {
                        bytes.push(1);
                        encode_condition(planned.condition(), &mut bytes);
                        bytes.push(u8::from(planned.value()));
                    }
                }
            }
        }

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

fn encode_condition(condition: &SemanticPureCondition, bytes: &mut Vec<u8>) {
    match condition {
        SemanticPureCondition::Bool(value) => {
            bytes.push(1);
            bytes.push(u8::from(*value));
        }
        SemanticPureCondition::IntCompare {
            lhs,
            comparator,
            rhs,
        } => {
            bytes.push(2);
            bytes.extend_from_slice(&lhs.to_be_bytes());
            bytes.push(comparator_tag(*comparator));
            bytes.extend_from_slice(&rhs.to_be_bytes());
        }
    }
}

const fn comparator_tag(comparator: SemanticPureComparator) -> u8 {
    match comparator {
        SemanticPureComparator::Eq => 1,
        SemanticPureComparator::Ne => 2,
        SemanticPureComparator::Lt => 3,
        SemanticPureComparator::Le => 4,
        SemanticPureComparator::Gt => 5,
        SemanticPureComparator::Ge => 6,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureConditionPlanError {
    Semantic(super::pure_condition::PureConditionCompilerError),
    EntryMustBePure { name: String },
}

impl PureConditionPlanError {
    pub fn primary_span(&self) -> Option<crate::frontend::SourceSpan> {
        match self {
            Self::Semantic(error) => error.primary_span(),
            Self::EntryMustBePure { .. } => None,
        }
    }
}

impl Display for PureConditionPlanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic(error) => Display::fmt(error, f),
            Self::EntryMustBePure { name } => {
                write!(
                f,
                "C0.11 pure-condition execution plan entry '{}' must require zero semantic effects",
                name.chars().flat_map(char::escape_default).collect::<String>()
            )
            }
        }
    }
}

impl Error for PureConditionPlanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Semantic(error) => Some(error),
            Self::EntryMustBePure { .. } => None,
        }
    }
}

impl From<super::pure_condition::PureConditionCompilerError> for PureConditionPlanError {
    fn from(value: super::pure_condition::PureConditionCompilerError) -> Self {
        Self::Semantic(value)
    }
}

pub type PureConditionPlanResult<T> = Result<T, PureConditionPlanError>;

pub fn validate_pure_condition_execution_plan(
    condition_semantics: NsirPureConditionUnit,
) -> PureConditionPlanResult<PureConditionExecutionPlan> {
    let form = match condition_semantics.form() {
        NsirPureConditionForm::Empty => PureConditionPlanForm::Empty,
        NsirPureConditionForm::Entry(entry) => {
            if !entry.required_effects().is_pure() {
                return Err(PureConditionPlanError::EntryMustBePure {
                    name: entry.name().as_str().to_owned(),
                });
            }

            let condition = entry.condition().cloned().map(|condition| {
                let value = condition.value();
                PlannedPureCondition::new(condition, value)
            });
            PureConditionPlanForm::Entry(PureConditionEntryPlan::new(
                entry.name().clone(),
                entry.required_effects().clone(),
                condition,
            ))
        }
    };

    Ok(PureConditionExecutionPlan::new(condition_semantics, form))
}

/// C0.11 pure-condition execution-plan boundary. It plans the fully understood
/// L0.9 boolean/comparison semantics without defining control flow, lowering to
/// NAIR, invoking runtime execution, allocating storage, dispatching effects,
/// consulting capabilities, or granting host authority.
pub fn compile_pure_condition_execution_plan_boundary(
    source: &SourceText,
) -> PureConditionPlanResult<PureConditionExecutionPlan> {
    validate_pure_condition_execution_plan(compile_pure_condition_boundary(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{
        compile_pure_condition_boundary, resolve_effect_set, NsirPureConditionEntry,
        NsirPureConditionForm, NsirPureConditionUnit, SemanticEffectSet, SemanticName,
    };
    use crate::frontend::{SourceId, SourceText};

    #[test]
    fn impure_manual_condition_plan_fails_closed() {
        let source = SourceText::new(
            SourceId::new(971),
            "impure-condition.noi",
            "effect Net; entry main returns 1 < 2;",
        )
        .unwrap();
        let compiled = compile_pure_condition_boundary(&source).unwrap();
        let semantic = compiled.semantic().clone();
        let original = compiled.entry().unwrap();
        let requirements = SemanticEffectSet::new(vec![SemanticName::new("Net").unwrap()]).unwrap();
        let resolved = resolve_effect_set(semantic.registry(), &requirements).unwrap();
        let impure = NsirPureConditionEntry::new(
            original.name().clone(),
            original.origin_span(),
            resolved,
            original.condition().cloned(),
        );
        let malformed = NsirPureConditionUnit::new(semantic, NsirPureConditionForm::Entry(impure));

        assert!(matches!(
            validate_pure_condition_execution_plan(malformed),
            Err(PureConditionPlanError::EntryMustBePure { .. })
        ));
    }
}
