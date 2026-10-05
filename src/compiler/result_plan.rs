use super::error::{CompilerError, CompilerResult};
use super::hir::SemanticName;
use super::pure_result::{compile_pure_result_boundary, NsirPureResultForm, NsirPureResultUnit};
use super::symbols::{ResolvedEffectSet, SemanticEffectId};
use crate::frontend::SourceText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PureResultPlanValue {
    Int(i64),
}

impl PureResultPlanValue {
    pub fn value(self) -> i64 {
        match self {
            Self::Int(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureResultEntryPlan {
    name: SemanticName,
    required_effects: ResolvedEffectSet,
    result: Option<PureResultPlanValue>,
}

impl PureResultEntryPlan {
    fn new(
        name: SemanticName,
        required_effects: ResolvedEffectSet,
        result: Option<PureResultPlanValue>,
    ) -> Self {
        Self {
            name,
            required_effects,
            result,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn required_effects(&self) -> &ResolvedEffectSet {
        &self.required_effects
    }

    pub fn result(&self) -> Option<PureResultPlanValue> {
        self.result
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.result.map(PureResultPlanValue::value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PureResultPlanForm {
    Empty,
    Entry(PureResultEntryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureResultExecutionPlan {
    result_semantics: NsirPureResultUnit,
    form: PureResultPlanForm,
}

impl PureResultExecutionPlan {
    fn new(result_semantics: NsirPureResultUnit, form: PureResultPlanForm) -> Self {
        Self {
            result_semantics,
            form,
        }
    }

    pub fn result_semantics(&self) -> &NsirPureResultUnit {
        &self.result_semantics
    }

    pub fn form(&self) -> &PureResultPlanForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&PureResultEntryPlan> {
        match &self.form {
            PureResultPlanForm::Empty => None,
            PureResultPlanForm::Entry(entry) => Some(entry),
        }
    }

    pub fn result(&self) -> Option<PureResultPlanValue> {
        self.entry().and_then(PureResultEntryPlan::result)
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.result().map(PureResultPlanValue::value)
    }

    pub fn work_item_count(&self) -> usize {
        0
    }

    pub fn required_effects(&self) -> &[SemanticEffectId] {
        match &self.form {
            PureResultPlanForm::Empty => &[],
            PureResultPlanForm::Entry(entry) => entry.required_effects().effects(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects().is_empty()
    }

    pub fn requires_host_authority(&self) -> bool {
        false
    }

    /// C0.5 pure-result execution-plan witness. It is compiler identity only. It does not replace
    /// L0.6 identity, encode host authority, lower to NAIR, or prove runtime execution.
    pub fn canonical_c05_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.5-PURE-RESULT-PLAN\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let l06 = self.result_semantics.canonical_l06_bytes();
        bytes.extend_from_slice(&(l06.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&l06);

        match &self.form {
            PureResultPlanForm::Empty => bytes.push(0),
            PureResultPlanForm::Entry(entry) => {
                bytes.push(1);
                let name = entry.name().as_str().as_bytes();
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
                bytes.extend_from_slice(name);

                match entry.result() {
                    None => bytes.push(0),
                    Some(PureResultPlanValue::Int(value)) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&value.to_be_bytes());
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

pub fn validate_pure_result_execution_plan(
    result_semantics: NsirPureResultUnit,
) -> CompilerResult<PureResultExecutionPlan> {
    let form = match result_semantics.form() {
        NsirPureResultForm::Empty => PureResultPlanForm::Empty,
        NsirPureResultForm::Entry(entry) => {
            if !entry.required_effects().is_pure() {
                return Err(CompilerError::PureResultPlanEntryMustBePure {
                    name: entry.name().as_str().to_owned(),
                });
            }

            let result = entry.result_i64().map(PureResultPlanValue::Int);
            PureResultPlanForm::Entry(PureResultEntryPlan::new(
                entry.name().clone(),
                entry.required_effects().clone(),
                result,
            ))
        }
    };

    Ok(PureResultExecutionPlan::new(result_semantics, form))
}

/// C0.5 pure-result execution-plan boundary. It plans the fully understood L0.6 pure-result
/// semantics without changing C0.3, lowering to NAIR, invoking the runtime, performing I/O,
/// dispatching effects, looking up capabilities, or granting host authority.
pub fn compile_pure_result_execution_plan_boundary(
    source: &SourceText,
) -> CompilerResult<PureResultExecutionPlan> {
    validate_pure_result_execution_plan(compile_pure_result_boundary(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{resolve_effect_set, NsirPureResultEntry, SemanticEffectSet};
    use crate::frontend::{SourceId, SourceText};

    #[test]
    fn impure_manual_pure_result_plan_fails_closed() {
        let source = SourceText::new(
            SourceId::new(950),
            "impure-result.noi",
            "effect Net; entry main returns 1;",
        )
        .unwrap();
        let compiled = compile_pure_result_boundary(&source).unwrap();
        let semantic = compiled.semantic().clone();
        let original = compiled.entry().unwrap();
        let requirements = SemanticEffectSet::new(vec![SemanticName::new("Net").unwrap()]).unwrap();
        let resolved = resolve_effect_set(semantic.registry(), &requirements).unwrap();
        let impure = NsirPureResultEntry::new(
            original.name().clone(),
            original.origin_span(),
            resolved,
            original.result().cloned(),
        );
        let malformed = NsirPureResultUnit::new(semantic, NsirPureResultForm::Entry(impure));

        assert!(matches!(
            validate_pure_result_execution_plan(malformed),
            Err(CompilerError::PureResultPlanEntryMustBePure { .. })
        ));
    }
}
