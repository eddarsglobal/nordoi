use crate::{
    effect_audit::EffectAuditHash,
    effect_dispatch::{EffectDeliveryFence, EffectDeliveryNamespace},
    effect_fencing::EffectJournalWriterId,
};

use super::{
    EffectAttestationAlgorithmId, EffectAttestationError, EffectAttestationKeyId,
    EffectAttestationResult, EffectAuditAttestation, EffectAuditAttestationStatement,
    EffectTrustEpoch, MAX_EFFECT_ATTESTATION_BYTES, MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
};

const MAGIC: &[u8; 8] = b"NDEFXT01";
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 0;
const ENVELOPE_HASH_DOMAIN: &[u8] = b"NORDOI-EFFECT-AUDIT-ATTESTATION-ENVELOPE-1.0";

impl EffectAuditAttestation {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let statement = &self.statement;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_MAJOR.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        bytes.extend_from_slice(&statement.namespace.0);
        bytes.extend_from_slice(&statement.writer.0);
        bytes.extend_from_slice(&statement.fence.0.to_le_bytes());
        bytes.extend_from_slice(&statement.trust_epoch.0.to_le_bytes());
        bytes.extend_from_slice(&statement.key_id.0);
        bytes.extend_from_slice(&statement.algorithm_id.0.to_le_bytes());
        bytes.extend_from_slice(&statement.audit_root.0);
        bytes.extend_from_slice(&statement.audit_records.to_le_bytes());
        bytes.extend_from_slice(&statement.checkpoint_hash.0);
        bytes.extend_from_slice(&(self.signature.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&self.signature);
        let digest = envelope_digest(&bytes);
        bytes.extend_from_slice(&digest);
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> EffectAttestationResult<Self> {
        if bytes.len() > MAX_EFFECT_ATTESTATION_BYTES {
            return Err(EffectAttestationError::AttestationTooLarge {
                bytes: bytes.len(),
                limit: MAX_EFFECT_ATTESTATION_BYTES,
            });
        }
        if bytes.len() < 32 {
            return Err(EffectAttestationError::AttestationTruncated);
        }
        let (payload, digest_bytes) = bytes.split_at(bytes.len() - 32);
        if envelope_digest(payload).as_slice() != digest_bytes {
            return Err(EffectAttestationError::AttestationDigestMismatch);
        }
        let mut reader = Reader::new(payload);
        if reader.read_exact(8)? != MAGIC {
            return Err(EffectAttestationError::InvalidAttestationMagic);
        }
        let major = reader.read_u16()?;
        let minor = reader.read_u16()?;
        if major != FORMAT_MAJOR || minor > FORMAT_MINOR {
            return Err(EffectAttestationError::UnsupportedAttestationVersion { major, minor });
        }
        let namespace = EffectDeliveryNamespace::new(reader.read_array_16()?);
        let writer = EffectJournalWriterId::new(reader.read_array_16()?);
        let fence = EffectDeliveryFence(reader.read_u64()?);
        let trust_epoch = EffectTrustEpoch(reader.read_u64()?);
        let key_id = EffectAttestationKeyId::new(reader.read_array_16()?);
        let algorithm_id = EffectAttestationAlgorithmId(reader.read_u16()?);
        let audit_root = EffectAuditHash(reader.read_array_32()?);
        let audit_records = reader.read_u64()?;
        let checkpoint_hash = EffectAuditHash(reader.read_array_32()?);
        let signature_len_u64 = reader.read_u64()?;
        if signature_len_u64 > MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES as u64 {
            return Err(EffectAttestationError::SignatureTooLarge {
                bytes: usize::try_from(signature_len_u64).unwrap_or(usize::MAX),
                limit: MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
            });
        }
        let signature_len = usize::try_from(signature_len_u64)
            .map_err(|_| EffectAttestationError::AttestationTruncated)?;
        let signature = reader.read_exact(signature_len)?.to_vec();
        if !reader.is_finished() {
            return Err(EffectAttestationError::AttestationTruncated);
        }
        let statement = EffectAuditAttestationStatement {
            namespace,
            writer,
            fence,
            trust_epoch,
            key_id,
            algorithm_id,
            audit_root,
            audit_records,
            checkpoint_hash,
        };
        if statement.fence.0 == 0 {
            return Err(EffectAttestationError::ZeroFence);
        }
        if statement.trust_epoch.0 == 0 {
            return Err(EffectAttestationError::ZeroTrustEpoch);
        }
        if statement.algorithm_id.0 == 0 {
            return Err(EffectAttestationError::ZeroAlgorithmId);
        }
        EffectAuditAttestation::new(statement, signature)
    }
}

fn envelope_digest(payload: &[u8]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(ENVELOPE_HASH_DOMAIN.len() + payload.len());
    bytes.extend_from_slice(ENVELOPE_HASH_DOMAIN);
    bytes.extend_from_slice(payload);
    crate::effect_audit::hash::sha256(&bytes)
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
    fn read_exact(&mut self, count: usize) -> EffectAttestationResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(EffectAttestationError::AttestationTruncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(EffectAttestationError::AttestationTruncated)?;
        self.offset = end;
        Ok(value)
    }
    fn read_u16(&mut self) -> EffectAttestationResult<u16> {
        Ok(u16::from_le_bytes(self.read_exact(2)?.try_into().map_err(
            |_| EffectAttestationError::AttestationTruncated,
        )?))
    }
    fn read_u64(&mut self) -> EffectAttestationResult<u64> {
        Ok(u64::from_le_bytes(self.read_exact(8)?.try_into().map_err(
            |_| EffectAttestationError::AttestationTruncated,
        )?))
    }
    fn read_array_16(&mut self) -> EffectAttestationResult<[u8; 16]> {
        self.read_exact(16)?
            .try_into()
            .map_err(|_| EffectAttestationError::AttestationTruncated)
    }
    fn read_array_32(&mut self) -> EffectAttestationResult<[u8; 32]> {
        self.read_exact(32)?
            .try_into()
            .map_err(|_| EffectAttestationError::AttestationTruncated)
    }
}
