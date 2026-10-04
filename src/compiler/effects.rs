use super::error::{CompilerError, CompilerResult};
use super::hir::SemanticName;

pub const MAX_SEMANTIC_EFFECT_REQUIREMENTS: u32 = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEffectSet {
    effects: Vec<SemanticName>,
}

impl SemanticEffectSet {
    pub fn empty() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn new(mut effects: Vec<SemanticName>) -> CompilerResult<Self> {
        let count = effects.len() as u32;
        if count > MAX_SEMANTIC_EFFECT_REQUIREMENTS {
            return Err(CompilerError::TooManyEffectRequirements {
                count,
                maximum: MAX_SEMANTIC_EFFECT_REQUIREMENTS,
            });
        }
        effects.sort();
        for pair in effects.windows(2) {
            if pair[0] == pair[1] {
                return Err(CompilerError::DuplicateEffectRequirement {
                    name: pair[0].as_str().to_owned(),
                });
            }
        }
        Ok(Self { effects })
    }

    pub fn effects(&self) -> &[SemanticName] {
        &self.effects
    }

    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        const DOMAIN: &[u8] = b"NORDOI-L0.4-EFFECT-SET\0";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(DOMAIN);
        bytes.extend_from_slice(&(self.effects.len() as u32).to_be_bytes());
        for effect in &self.effects {
            let raw = effect.as_str().as_bytes();
            bytes.extend_from_slice(&(raw.len() as u32).to_be_bytes());
            bytes.extend_from_slice(raw);
        }
        bytes
    }
}

impl Default for SemanticEffectSet {
    fn default() -> Self {
        Self::empty()
    }
}
