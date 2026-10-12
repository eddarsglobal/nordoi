//! P0.2 research candidate: verify signed envelopes, sync a bounded append-only chain,
//! then publish the advanced in-memory gate. Journal recovery re-verifies every Ed25519
//! signature and reconstructs epochs, counters and revocations.
//!
//! NOT production authorization: a local journal cannot reject a complete rollback
//! of a previously valid journal without a separately trusted monotonic witness.
//! The old P0.1 API remains available and there is no runtime-effect integration.

use super::{GateError, Intent, SignedEnvelope, TrustGate};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8] = b"NORDOI:P0.2:SIGNED-JOURNAL:V1\0";
const BOOT_DOMAIN: &[u8] = b"NORDOI:P0.2:BOOTSTRAP:V1\0";
const MAX_FRAME: usize = 512;
const MAX_JOURNAL_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrustJournalWitness {
    bytes: u64,
    digest: [u8; 32],
}

impl TrustJournalWitness {
    pub const fn from_parts(bytes: u64, digest: [u8; 32]) -> Self {
        Self { bytes, digest }
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Debug)]
pub enum DurableError {
    Crypto(GateError),
    Io(std::io::ErrorKind),
    Corrupt,
    BootstrapMismatch,
    MissingJournal,
    JournalExists,
    Busy,
    Poisoned,
    Full,
}

impl From<GateError> for DurableError {
    fn from(err: GateError) -> Self {
        Self::Crypto(err)
    }
}
impl From<std::io::Error> for DurableError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.kind())
    }
}

struct ExclusiveLock {
    path: PathBuf,
}
impl ExclusiveLock {
    fn acquire(path: &Path) -> Result<Self, DurableError> {
        let lock_path = path.with_extension("p02.lock");
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&lock_path) {
            Ok(mut file) => {
                file.write_all(
                    b"NORDOI P0.2 exclusive writer lock; inspect before removing after crash\n",
                )?;
                file.sync_all()?;
                Ok(Self { path: lock_path })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(DurableError::Busy),
            Err(e) => Err(e.into()),
        }
    }
}
impl Drop for ExclusiveLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn hash32(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn bootstrap_hash(roots: &[(u64, [u8; 32], u64)]) -> [u8; 32] {
    let mut ordered = BTreeMap::new();
    for (id, key, epoch) in roots {
        ordered.insert(*id, (*key, *epoch));
    }
    let mut bytes = Vec::with_capacity(BOOT_DOMAIN.len() + roots.len() * 48);
    bytes.extend_from_slice(BOOT_DOMAIN);
    for (id, (key, epoch)) in ordered {
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend_from_slice(&epoch.to_be_bytes());
        bytes.extend_from_slice(&key);
    }
    hash32(&bytes)
}
fn initial_digest(bootstrap: &[u8; 32]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(MAGIC.len() + 32);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(bootstrap);
    hash32(&bytes)
}
fn next_digest(prev: &[u8; 32], body: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(prev);
    h.update(body);
    h.finalize().into()
}
fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], DurableError> {
    if input.len() < len {
        return Err(DurableError::Corrupt);
    }
    let (prefix, suffix) = input.split_at(len);
    *input = suffix;
    Ok(prefix)
}
fn get_u64(input: &mut &[u8]) -> Result<u64, DurableError> {
    let bytes: [u8; 8] = take(input, 8)?
        .try_into()
        .map_err(|_| DurableError::Corrupt)?;
    Ok(u64::from_be_bytes(bytes))
}
fn get_u16(input: &mut &[u8]) -> Result<usize, DurableError> {
    let bytes: [u8; 2] = take(input, 2)?
        .try_into()
        .map_err(|_| DurableError::Corrupt)?;
    Ok(u16::from_be_bytes(bytes) as usize)
}
fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}
fn get_str(input: &mut &[u8]) -> Result<String, DurableError> {
    let n = get_u16(input)?;
    if n == 0 || n > 128 {
        return Err(DurableError::Corrupt);
    }
    String::from_utf8(take(input, n)?.to_vec()).map_err(|_| DurableError::Corrupt)
}
fn encode(e: &SignedEnvelope) -> Result<Vec<u8>, DurableError> {
    let mut out = Vec::with_capacity(400);
    out.extend_from_slice(&e.root_id.to_be_bytes());
    out.extend_from_slice(&e.epoch.to_be_bytes());
    out.extend_from_slice(&e.sequence.to_be_bytes());
    match &e.intent {
        Intent::Execute {
            capability,
            resource,
            payload_sha256,
        } => {
            // Validate exactly the existing P0.1 signing profile.
            e.signing_bytes()?;
            out.push(1);
            put_str(&mut out, capability);
            put_str(&mut out, resource);
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
    out.extend_from_slice(&e.signature);
    if out.len() > MAX_FRAME {
        return Err(DurableError::Corrupt);
    }
    Ok(out)
}
fn decode(mut input: &[u8]) -> Result<SignedEnvelope, DurableError> {
    let root_id = get_u64(&mut input)?;
    let epoch = get_u64(&mut input)?;
    let sequence = get_u64(&mut input)?;
    let tag = *take(&mut input, 1)?.first().ok_or(DurableError::Corrupt)?;
    let intent = match tag {
        1 => {
            let capability = get_str(&mut input)?;
            let resource = get_str(&mut input)?;
            let payload_sha256 = take(&mut input, 32)?
                .try_into()
                .map_err(|_| DurableError::Corrupt)?;
            Intent::Execute {
                capability,
                resource,
                payload_sha256,
            }
        }
        2 => {
            let next_epoch = get_u64(&mut input)?;
            let next_public_key = take(&mut input, 32)?
                .try_into()
                .map_err(|_| DurableError::Corrupt)?;
            Intent::Rotate {
                next_epoch,
                next_public_key,
            }
        }
        3 => Intent::Revoke,
        _ => return Err(DurableError::Corrupt),
    };
    let signature = take(&mut input, 64)?
        .try_into()
        .map_err(|_| DurableError::Corrupt)?;
    if !input.is_empty() {
        return Err(DurableError::Corrupt);
    }
    let envelope = SignedEnvelope {
        root_id,
        epoch,
        sequence,
        intent,
        signature,
    };
    envelope
        .signing_bytes()
        .map_err(|_| DurableError::Corrupt)?;
    Ok(envelope)
}
fn replay(gate: &mut TrustGate, e: &SignedEnvelope) -> Result<(), DurableError> {
    match &e.intent {
        Intent::Execute { .. } => {
            gate.verify(e).map_err(|_| DurableError::Corrupt)?;
            let root = gate
                .roots
                .get_mut(&e.root_id)
                .ok_or(DurableError::Corrupt)?;
            root.last_sequence = e.sequence;
        }
        Intent::Rotate { .. } => gate.rotate(e).map_err(|_| DurableError::Corrupt)?,
        Intent::Revoke => gate.revoke(e).map_err(|_| DurableError::Corrupt)?,
    }
    Ok(())
}
fn check_non_symlink(path: &Path) -> Result<(), DurableError> {
    match fs::symlink_metadata(path) {
        Ok(md) if md.file_type().is_symlink() || !md.is_file() => Err(DurableError::Corrupt),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(DurableError::MissingJournal),
        Err(e) => Err(e.into()),
    }
}
#[cfg(unix)]
fn sync_parent(path: &Path) -> Result<(), DurableError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    File::open(parent)?.sync_all()?;
    Ok(())
}
#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> Result<(), DurableError> {
    Ok(())
}

/// Single-process, exclusively locked journal of signed state transitions.
/// DO NOT expose its receipts to an external-effect executor until a later
/// P0.3 implementation installs a durable transactional boundary.
pub struct DurableTrustGate {
    _lock: ExclusiveLock,
    file: File,
    gate: TrustGate,
    digest: [u8; 32],
    size: u64,
    witnesses: BTreeMap<u64, [u8; 32]>,
    poisoned: bool,
}
impl DurableTrustGate {
    /// First boot is explicit: opening a missing journal always fails closed.
    pub fn initialize(path: &Path, roots: &[(u64, [u8; 32], u64)]) -> Result<Self, DurableError> {
        let gate = TrustGate::from_trusted_bootstrap(roots)?;
        let lock = ExclusiveLock::acquire(path)?;
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = match opts.open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(DurableError::JournalExists)
            }
            Err(e) => return Err(e.into()),
        };
        let boot = bootstrap_hash(roots);
        file.write_all(MAGIC)?;
        file.write_all(&boot)?;
        file.sync_all()?;
        sync_parent(path)?;
        let size = (MAGIC.len() + 32) as u64;
        let digest = initial_digest(&boot);
        let mut witnesses = BTreeMap::new();
        witnesses.insert(size, digest);
        Ok(Self {
            _lock: lock,
            file,
            gate,
            digest,
            size,
            witnesses,
            poisoned: false,
        })
    }
    /// Re-verify all signatures from the exact bootstrapped trust roots.
    /// Corrupted, partial or reordered records block admission.
    pub fn open(path: &Path, roots: &[(u64, [u8; 32], u64)]) -> Result<Self, DurableError> {
        let mut gate = TrustGate::from_trusted_bootstrap(roots)?;
        let lock = ExclusiveLock::acquire(path)?;
        check_non_symlink(path)?;
        let mut file = OpenOptions::new().read(true).append(true).open(path)?;
        let md = file.metadata()?;
        if md.len() > MAX_JOURNAL_BYTES {
            return Err(DurableError::Full);
        }
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(0))?;
        file.read_to_end(&mut bytes)?;
        if bytes.len() < MAGIC.len() + 32 || &bytes[..MAGIC.len()] != MAGIC {
            return Err(DurableError::Corrupt);
        }
        let expected_boot = bootstrap_hash(roots);
        if &bytes[MAGIC.len()..MAGIC.len() + 32] != expected_boot.as_slice() {
            return Err(DurableError::BootstrapMismatch);
        }
        let mut prev = initial_digest(&expected_boot);
        let mut position = (MAGIC.len() + 32) as u64;
        let mut witnesses = BTreeMap::new();
        witnesses.insert(position, prev);
        let mut remainder = &bytes[MAGIC.len() + 32..];
        while !remainder.is_empty() {
            let n = u32::from_be_bytes(
                take(&mut remainder, 4)?
                    .try_into()
                    .map_err(|_| DurableError::Corrupt)?,
            ) as usize;
            if n == 0 || n > MAX_FRAME {
                return Err(DurableError::Corrupt);
            }
            let frame = take(&mut remainder, n)?;
            let digest = take(&mut remainder, 32)?;
            let expected = next_digest(&prev, frame);
            if digest != expected.as_slice() {
                return Err(DurableError::Corrupt);
            }
            let e = decode(frame)?;
            replay(&mut gate, &e)?;
            prev = expected;
            position = position
                .checked_add(4 + n as u64 + 32)
                .ok_or(DurableError::Corrupt)?;
            witnesses.insert(position, prev);
        }
        if position != bytes.len() as u64 {
            return Err(DurableError::Corrupt);
        }
        Ok(Self {
            _lock: lock,
            file,
            gate,
            digest: prev,
            size: bytes.len() as u64,
            witnesses,
            poisoned: false,
        })
    }
    pub fn root_status(&self, id: u64) -> Option<(u64, u64, bool)> {
        self.gate.root_status(id)
    }
    pub fn journal_bytes(&self) -> u64 {
        self.size
    }

    pub fn journal_witness(&self) -> TrustJournalWitness {
        TrustJournalWitness::from_parts(self.size, self.digest)
    }

    pub fn contains_journal_witness(
        &self,
        witness: TrustJournalWitness,
    ) -> Result<bool, DurableError> {
        if self.poisoned {
            return Err(DurableError::Poisoned);
        }
        Ok(self.witnesses.get(&witness.bytes) == Some(&witness.digest))
    }

    fn append(&mut self, e: &SignedEnvelope) -> Result<(), DurableError> {
        if self.poisoned {
            return Err(DurableError::Poisoned);
        }
        let frame = encode(e)?;
        let increment = 4 + frame.len() as u64 + 32;
        if !matches!(self.size.checked_add(increment), Some(n) if n <= MAX_JOURNAL_BYTES) {
            return Err(DurableError::Full);
        }
        let digest = next_digest(&self.digest, &frame);
        let result = (|| -> Result<(), DurableError> {
            self.file.write_all(&(frame.len() as u32).to_be_bytes())?;
            self.file.write_all(&frame)?;
            self.file.write_all(&digest)?;
            self.file.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
            return result;
        }
        self.digest = digest;
        self.size += increment;
        self.witnesses.insert(self.size, self.digest);
        Ok(())
    }
    pub fn verify_execution(
        &mut self,
        e: &SignedEnvelope,
        capability: &str,
        resource: &str,
        payload: &[u8],
    ) -> Result<super::ExecutionReceipt, DurableError> {
        if self.poisoned {
            return Err(DurableError::Poisoned);
        }
        let mut next = TrustGate {
            roots: self.gate.roots.clone(),
        };
        let receipt = next.verify_execution(e, capability, resource, payload)?;
        self.append(e)?;
        self.gate = next;
        Ok(receipt)
    }
    pub fn rotate(&mut self, e: &SignedEnvelope) -> Result<(), DurableError> {
        if self.poisoned {
            return Err(DurableError::Poisoned);
        }
        let mut next = TrustGate {
            roots: self.gate.roots.clone(),
        };
        next.rotate(e)?;
        self.append(e)?;
        self.gate = next;
        Ok(())
    }
    pub fn revoke(&mut self, e: &SignedEnvelope) -> Result<(), DurableError> {
        if self.poisoned {
            return Err(DurableError::Poisoned);
        }
        let mut next = TrustGate {
            roots: self.gate.roots.clone(),
        };
        next.revoke(e)?;
        self.append(e)?;
        self.gate = next;
        Ok(())
    }
}
