//! P0.1: real Ed25519 signatures with in-memory trusted roots and monotonic counters.
//! NOT a production-enabling authorization backend: trusted bootstrap, durable anti-replay,
//! failure-atomic external effect dispatch, and independent verification are still required.

use std::collections::BTreeMap;

use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

const DOMAIN: &[u8] = b"NORDOI:P0.1:ADMISSION:V1\0";
const MAX_ID_LEN: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    MissingRoots,
    DuplicateRoot,
    DuplicateKey,
    InvalidRoot,
    InvalidKey,
    UnknownRoot,
    Revoked,
    WrongEpoch,
    WrongSequence,
    InvalidScope,
    WrongScope,
    WrongPayload,
    BadSignature,
    WrongOperation,
    InvalidRotation,
}

/// Capability and resource are logical identifiers, not paths or executable instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Execute {
        capability: String,
        resource: String,
        payload_sha256: [u8; 32],
    },
    Rotate {
        next_epoch: u64,
        next_public_key: [u8; 32],
    },
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedEnvelope {
    pub root_id: u64,
    pub epoch: u64,
    pub sequence: u64,
    pub intent: Intent,
    pub signature: [u8; 64],
}

impl SignedEnvelope {
    /// Exact canonical Ed25519 message. Callers sign these bytes, not an ad hoc JSON form.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, GateError> {
        if self.root_id == 0 || self.epoch == 0 || self.sequence == 0 {
            return Err(GateError::InvalidRoot);
        }
        let mut out = Vec::with_capacity(384);
        out.extend_from_slice(DOMAIN);
        out.extend_from_slice(&self.root_id.to_be_bytes());
        out.extend_from_slice(&self.epoch.to_be_bytes());
        out.extend_from_slice(&self.sequence.to_be_bytes());
        match &self.intent {
            Intent::Execute {
                capability,
                resource,
                payload_sha256,
            } => {
                valid_identifier(capability)?;
                valid_identifier(resource)?;
                out.push(1);
                let cap = capability.as_bytes();
                let res = resource.as_bytes();
                out.extend_from_slice(&(cap.len() as u16).to_be_bytes());
                out.extend_from_slice(cap);
                out.extend_from_slice(&(res.len() as u16).to_be_bytes());
                out.extend_from_slice(res);
                out.extend_from_slice(payload_sha256);
            }
            Intent::Rotate {
                next_epoch,
                next_public_key,
            } => {
                out.push(2);
                out.extend_from_slice(&next_epoch.to_be_bytes());
                out.extend_from_slice(next_public_key);
            }
            Intent::Revoke => out.push(3),
        }
        Ok(out)
    }
}

fn valid_identifier(s: &str) -> Result<(), GateError> {
    if s.is_empty()
        || s.split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || s.len() > MAX_ID_LEN
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:/-".contains(&b))
    {
        return Err(GateError::InvalidScope);
    }
    Ok(())
}

pub fn sha256_payload(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
}

#[derive(Clone)]
struct RootState {
    key: VerifyingKey,
    epoch: u64,
    last_sequence: u64,
    revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    root_id: u64,
    epoch: u64,
    sequence: u64,
    capability: String,
    resource: String,
    payload_sha256: [u8; 32],
}

impl ExecutionReceipt {
    pub fn root_id(&self) -> u64 {
        self.root_id
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn capability(&self) -> &str {
        &self.capability
    }
    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn payload_sha256(&self) -> &[u8; 32] {
        &self.payload_sha256
    }
}

/// Host-bootstrapped trust. Only authenticated signed messages may mutate an existing root.
/// P0.1 does not persist any state; do not use this to authorize real external side effects.
pub struct TrustGate {
    roots: BTreeMap<u64, RootState>,
}

impl TrustGate {
    /// The caller MUST supply publicly trusted root IDs/keys from a separately protected source.
    /// A key supplied by an untrusted client is NOT a trusted root.
    pub fn from_trusted_bootstrap(roots: &[(u64, [u8; 32], u64)]) -> Result<Self, GateError> {
        if roots.is_empty() {
            return Err(GateError::MissingRoots);
        }
        let mut trusted = BTreeMap::new();
        for (id, bytes, epoch) in roots {
            if *id == 0 || *epoch == 0 {
                return Err(GateError::InvalidRoot);
            }
            if trusted.contains_key(id) {
                return Err(GateError::DuplicateRoot);
            }
            let key = VerifyingKey::from_bytes(bytes).map_err(|_| GateError::InvalidKey)?;
            if trusted.values().any(|root: &RootState| root.key == key) {
                return Err(GateError::DuplicateKey);
            }
            trusted.insert(
                *id,
                RootState {
                    key,
                    epoch: *epoch,
                    last_sequence: 0,
                    revoked: false,
                },
            );
        }
        Ok(Self { roots: trusted })
    }

    pub fn root_status(&self, root_id: u64) -> Option<(u64, u64, bool)> {
        self.roots
            .get(&root_id)
            .map(|r| (r.epoch, r.last_sequence, r.revoked))
    }

    fn verify(&self, envelope: &SignedEnvelope) -> Result<(), GateError> {
        let root = self
            .roots
            .get(&envelope.root_id)
            .ok_or(GateError::UnknownRoot)?;
        if root.revoked {
            return Err(GateError::Revoked);
        }
        if root.epoch != envelope.epoch {
            return Err(GateError::WrongEpoch);
        }
        if root.last_sequence.checked_add(1) != Some(envelope.sequence) {
            return Err(GateError::WrongSequence);
        }
        let message = envelope.signing_bytes()?;
        root.key
            .verify_strict(&message, &Signature::from_bytes(&envelope.signature))
            .map_err(|_| GateError::BadSignature)
    }

    /// Validates exact capability/resource/payload and consumes an in-memory sequence.
    /// The result is NOT durable and MUST NOT be used as a production authority token.
    pub fn verify_execution(
        &mut self,
        envelope: &SignedEnvelope,
        capability: &str,
        resource: &str,
        payload: &[u8],
    ) -> Result<ExecutionReceipt, GateError> {
        let (signed_cap, signed_resource, signed_hash) = match &envelope.intent {
            Intent::Execute {
                capability,
                resource,
                payload_sha256,
            } => (capability, resource, payload_sha256),
            _ => return Err(GateError::WrongOperation),
        };
        valid_identifier(capability)?;
        valid_identifier(resource)?;
        if signed_cap != capability || signed_resource != resource {
            return Err(GateError::WrongScope);
        }
        if *signed_hash != sha256_payload(payload) {
            return Err(GateError::WrongPayload);
        }
        self.verify(envelope)?;
        let root = self
            .roots
            .get_mut(&envelope.root_id)
            .ok_or(GateError::UnknownRoot)?;
        root.last_sequence = envelope.sequence;
        Ok(ExecutionReceipt {
            root_id: envelope.root_id,
            epoch: envelope.epoch,
            sequence: envelope.sequence,
            capability: signed_cap.clone(),
            resource: signed_resource.clone(),
            payload_sha256: *signed_hash,
        })
    }

    /// Rotation must be signed by the currently trusted key, not by the proposed new key.
    pub fn rotate(&mut self, envelope: &SignedEnvelope) -> Result<(), GateError> {
        let (new_epoch, key_bytes) = match &envelope.intent {
            Intent::Rotate {
                next_epoch,
                next_public_key,
            } => (*next_epoch, next_public_key),
            _ => return Err(GateError::WrongOperation),
        };
        self.verify(envelope)?;
        let root = self
            .roots
            .get(&envelope.root_id)
            .ok_or(GateError::UnknownRoot)?;
        if root.epoch.checked_add(1) != Some(new_epoch) {
            return Err(GateError::InvalidRotation);
        }
        let new_key = VerifyingKey::from_bytes(key_bytes).map_err(|_| GateError::InvalidKey)?;
        if root.key == new_key
            || self
                .roots
                .iter()
                .any(|(id, other)| *id != envelope.root_id && other.key == new_key)
        {
            return Err(GateError::DuplicateKey);
        }
        let root = self
            .roots
            .get_mut(&envelope.root_id)
            .ok_or(GateError::UnknownRoot)?;
        root.key = new_key;
        root.epoch = new_epoch;
        root.last_sequence = envelope.sequence;
        Ok(())
    }

    /// Revocation is irreversible for the lifetime of this in-memory instance.
    pub fn revoke(&mut self, envelope: &SignedEnvelope) -> Result<(), GateError> {
        if !matches!(&envelope.intent, Intent::Revoke) {
            return Err(GateError::WrongOperation);
        }
        self.verify(envelope)?;
        let root = self
            .roots
            .get_mut(&envelope.root_id)
            .ok_or(GateError::UnknownRoot)?;
        root.revoked = true;
        root.last_sequence = envelope.sequence;
        Ok(())
    }
}

/// Production-track P0.2 experimental crash/restart trust journal; NOT a production authorization boundary.
pub mod durable;
