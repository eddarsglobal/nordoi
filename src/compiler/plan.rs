use super::body::{compile_minimal_body_boundary, NsirBodyUnit, NsirMinimalBody};
use super::error::{CompilerError, CompilerResult};
use super::hir::SemanticName;
use super::symbols::{ResolvedEffectSet, SemanticEffectId};
use crate::frontend::SourceText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEntryPlan {
    name: SemanticName,
    required_effects: ResolvedEffectSet,
}

impl SemanticEntryPlan {
    fn new(name: SemanticName, required_effects: ResolvedEffectSet) -> Self {
        Self {
            name,
            required_effects,
        }
    }

    pub fn name(&self) -> &SemanticName {
        &self.name
    }

    pub fn required_effects(&self) -> &ResolvedEffectSet {
        &self.required_effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticPlanForm {
    Empty,
    Entry(SemanticEntryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticExecutionPlan {
    body: NsirBodyUnit,
    form: SemanticPlanForm,
}

impl SemanticExecutionPlan {
    fn new(body: NsirBodyUnit, form: SemanticPlanForm) -> Self {
        Self { body, form }
    }

    pub fn body_semantics(&self) -> &NsirBodyUnit {
        &self.body
    }

    pub fn form(&self) -> &SemanticPlanForm {
        &self.form
    }

    pub fn entry(&self) -> Option<&SemanticEntryPlan> {
        match &self.form {
            SemanticPlanForm::Empty => None,
            SemanticPlanForm::Entry(entry) => Some(entry),
        }
    }

    pub fn work_item_count(&self) -> usize {
        0
    }

    pub fn required_effects(&self) -> &[SemanticEffectId] {
        match &self.form {
            SemanticPlanForm::Empty => &[],
            SemanticPlanForm::Entry(entry) => entry.required_effects().effects(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.required_effects().is_empty()
    }

    pub fn requires_host_authority(&self) -> bool {
        false
    }

    /// C0.3 semantic execution-plan witness. This is compiler identity only; it is not NAIR,
    /// a runtime checkpoint, an authority grant, or proof that host execution occurred.
    pub fn canonical_c03_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-C0.3-PLAN\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);

        let body = self.body.canonical_l05_bytes();
        bytes.extend_from_slice(&(body.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&body);

        match &self.form {
            SemanticPlanForm::Empty => bytes.push(0),
            SemanticPlanForm::Entry(entry) => {
                bytes.push(1);
                let name = entry.name().as_str().as_bytes();
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
                bytes.extend_from_slice(name);
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

pub fn validate_execution_plan(body: NsirBodyUnit) -> CompilerResult<SemanticExecutionPlan> {
    let form = match body.body() {
        NsirMinimalBody::Empty => SemanticPlanForm::Empty,
        NsirMinimalBody::Entry(entry) => {
            if !entry.required_effects().is_pure() {
                return Err(CompilerError::ExecutablePlanEntryMustBePure {
                    name: entry.name().as_str().to_owned(),
                });
            }
            SemanticPlanForm::Entry(SemanticEntryPlan::new(
                entry.name().clone(),
                entry.required_effects().clone(),
            ))
        }
    };

    Ok(SemanticExecutionPlan::new(body, form))
}

/// C0.3 executable semantic-plan boundary. It compiles only fully understood L0.5 bodies into a
/// canonical zero-work plan. It performs no NSIR -> NAIR lowering, runtime execution, effect
/// dispatch, capability lookup, or authority grant.
pub fn compile_execution_plan_boundary(
    source: &SourceText,
) -> CompilerResult<SemanticExecutionPlan> {
    validate_execution_plan(compile_minimal_body_boundary(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{resolve_effect_set, SemanticEffectSet};
    use crate::frontend::{SourceId, SourceText};

    #[test]
    fn impure_manual_entry_plan_fails_closed() {
        let source =
            SourceText::new(SourceId::new(900), "impure.noi", "effect Net; entry main;").unwrap();
        let compiled = compile_minimal_body_boundary(&source).unwrap();
        let semantic = compiled.semantic().clone();
        let requirements = SemanticEffectSet::new(vec![SemanticName::new("Net").unwrap()]).unwrap();
        let resolved = resolve_effect_set(semantic.registry(), &requirements).unwrap();
        let original = compiled.entry().unwrap();
        let entry = super::super::body::NsirEntryPoint::new(
            original.name().clone(),
            original.origin_span(),
            resolved,
        );
        let malformed = NsirBodyUnit::new(semantic, NsirMinimalBody::Entry(entry));

        assert!(matches!(
            validate_execution_plan(malformed),
            Err(CompilerError::ExecutablePlanEntryMustBePure { .. })
        ));
    }
}
