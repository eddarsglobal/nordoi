use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use super::{EffectAttestationAlgorithmId, EffectAttestationKeyId, EffectTrustEpoch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectAttestationBackendError {
    message: String,
}

impl EffectAttestationBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for EffectAttestationBackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EffectAttestationBackendError {}

pub trait EffectAttestationSigner {
    fn key_id(&self) -> EffectAttestationKeyId;
    fn algorithm_id(&self) -> EffectAttestationAlgorithmId;
    fn sign(&mut self, statement: &[u8]) -> Result<Vec<u8>, EffectAttestationBackendError>;
}

pub trait EffectAttestationVerifier {
    fn verify(
        &self,
        key_id: EffectAttestationKeyId,
        trust_epoch: EffectTrustEpoch,
        algorithm_id: EffectAttestationAlgorithmId,
        statement: &[u8],
        signature: &[u8],
    ) -> Result<bool, EffectAttestationBackendError>;
}
