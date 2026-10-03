mod attestor;
mod backend;
mod error;
mod format;
mod id;
mod model;
mod store;

pub use attestor::GovernedEffectAttestor;
pub use backend::{
    EffectAttestationBackendError, EffectAttestationSigner, EffectAttestationVerifier,
};
pub use error::{EffectAttestationError, EffectAttestationResult};
pub use id::{EffectAttestationAlgorithmId, EffectAttestationKeyId, EffectTrustEpoch};
pub use model::{EffectAuditAttestation, EffectAuditAttestationStatement};
pub use store::{
    EffectAttestationCommitReceipt, EffectAttestationStore, EffectAttestationStoreError,
};

pub const MAX_EFFECT_ATTESTATION_BYTES: usize = 128 * 1024;
pub const MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES: usize = 64 * 1024;
