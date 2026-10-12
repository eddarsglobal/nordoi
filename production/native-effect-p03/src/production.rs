//! P0.4: a fail-closed, host-installed effect backend with a second persistent
//! delivery-identity ledger. This is **not** a universal kernel-wide policy.
//!
//! The ledger reserves a delivery ID and fsyncs BEFORE P0.3 consumes a signed
//! authorization or calls the downstream backend. Missing/invalid authorization
//! may leave a conservative IN_DOUBT record. No automatic retry or refund exists.
//! The chain does NOT defend against restoring an older *entire* valid ledger;
//! a separately trusted monotonic anchor is still required.

use crate::{canonical_dispatch_bytes, GuardedEffectBackend, SignedEnvelopeSource};
use nordoi_kernel::{
    Effect, EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDispatchRequest,
    QueuedEffectIntent,
};
use nordoi_native_trust_p01::{
    durable::{DurableError, TrustJournalWitness},
    sha256_payload,
};
use nordoi_rollback_anchor_p06::{
    initialize_production_anchor, verify_production_recovery, AnchorOpenMode, ChainWitness,
    MonotonicAnchor, ProductionWitnessSource, RecoveryError,
};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8] = b"NORDOI:P0.4:DELIVERY-STATE:V1\n";
const RECORD_BODY: usize = 1 + 24 + 32;
const RECORD_BYTES: usize = RECORD_BODY + 32;
const MAX_LEDGER_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryLedgerWitness {
    bytes: u64,
    digest: [u8; 32],
}

impl DeliveryLedgerWitness {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryState {
    /// May or may not have reached the destination. NEVER auto-dispatch again.
    InDoubt,
    /// The local backend returned success; NOT proof of recipient-side commit.
    LocallyAcknowledged,
}

#[derive(Debug)]
pub enum LedgerError {
    Io(std::io::Error),
    Missing,
    Exists,
    Locked,
    Corrupt,
    Full,
    Poisoned,
    DuplicateDelivery,
}
impl From<std::io::Error> for LedgerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

struct LedgerLock(PathBuf);
impl LedgerLock {
    fn acquire(path: &Path) -> Result<Self, LedgerError> {
        let path = path.with_extension("p04.lock");
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&path) {
            Ok(mut file) => {
                if let Err(error) = (|| -> std::io::Result<()> {
                    file.write_all(b"NORDOI P0.4 exclusive owner\n")?;
                    file.sync_all()
                })() {
                    let _ = fs::remove_file(&path);
                    return Err(LedgerError::Io(error));
                }
                Ok(Self(path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(LedgerError::Locked),
            Err(e) => Err(LedgerError::Io(e)),
        }
    }
}
impl Drop for LedgerLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn sync_parent(path: &Path) -> Result<(), LedgerError> {
    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        File::open(parent)?.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn ensure_regular(path: &Path) -> Result<(), LedgerError> {
    match fs::symlink_metadata(path) {
        Ok(md) if md.file_type().is_symlink() || !md.is_file() => Err(LedgerError::Corrupt),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(LedgerError::Missing),
        Err(e) => Err(LedgerError::Io(e)),
    }
}
fn digest(previous: &[u8; 32], body: &[u8]) -> [u8; 32] {
    let mut data = Vec::with_capacity(previous.len() + body.len());
    data.extend_from_slice(previous);
    data.extend_from_slice(body);
    sha256_payload(&data)
}
fn key(request: &EffectDispatchRequest) -> Result<[u8; 24], &'static str> {
    let delivery = request.delivery_key.ok_or("missing delivery key")?;
    if delivery.intent != request.queued.id || delivery.namespace.bytes() == [0; 16] {
        return Err("invalid delivery key");
    }
    let mut id = [0_u8; 24];
    id[..16].copy_from_slice(&delivery.namespace.bytes());
    id[16..].copy_from_slice(&delivery.intent.value().to_be_bytes());
    Ok(id)
}

/// Local WAL scoped to one explicit production host and protected against
/// normal concurrent writers. A crash-stale lock is NOT silently removed;
/// operator-supervised recovery must verify that the former owner is dead.
struct DeliveryLedger {
    _lock: LedgerLock,
    file: File,
    last: [u8; 32],
    size: u64,
    entries: BTreeMap<[u8; 24], ([u8; 32], DeliveryState)>,
    witnesses: BTreeMap<u64, [u8; 32]>,
    poisoned: bool,
}
impl DeliveryLedger {
    fn initialize(path: &Path) -> Result<Self, LedgerError> {
        let lock = LedgerLock::acquire(path)?;
        let mut opts = OpenOptions::new();
        opts.read(true).append(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = match opts.open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(LedgerError::Exists)
            }
            Err(e) => return Err(LedgerError::Io(e)),
        };
        file.write_all(MAGIC)?;
        file.sync_all()?;
        sync_parent(path)?;
        let size = MAGIC.len() as u64;
        let last = sha256_payload(MAGIC);
        let mut witnesses = BTreeMap::new();
        witnesses.insert(size, last);
        Ok(Self {
            _lock: lock,
            file,
            last,
            size,
            entries: BTreeMap::new(),
            witnesses,
            poisoned: false,
        })
    }
    fn open(path: &Path) -> Result<Self, LedgerError> {
        let lock = LedgerLock::acquire(path)?;
        ensure_regular(path)?;
        let mut file = OpenOptions::new().read(true).append(true).open(path)?;
        if file.metadata()?.len() > MAX_LEDGER_BYTES {
            return Err(LedgerError::Full);
        }
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        if !data.starts_with(MAGIC) {
            return Err(LedgerError::Corrupt);
        }
        let records = &data[MAGIC.len()..];
        if records.len() % RECORD_BYTES != 0 {
            return Err(LedgerError::Corrupt);
        }
        let mut entries = BTreeMap::new();
        let mut previous = sha256_payload(MAGIC);
        let mut position = MAGIC.len() as u64;
        let mut witnesses = BTreeMap::new();
        witnesses.insert(position, previous);
        for record in records.as_chunks::<RECORD_BYTES>().0 {
            let body = &record[..RECORD_BODY];
            let expected = digest(&previous, body);
            if &record[RECORD_BODY..] != expected.as_slice() {
                return Err(LedgerError::Corrupt);
            }
            let delivery: [u8; 24] = body[1..25].try_into().map_err(|_| LedgerError::Corrupt)?;
            let binding: [u8; 32] = body[25..57].try_into().map_err(|_| LedgerError::Corrupt)?;
            match body[0] {
                1 => {
                    if entries
                        .insert(delivery, (binding, DeliveryState::InDoubt))
                        .is_some()
                    {
                        return Err(LedgerError::Corrupt);
                    }
                }
                2 => {
                    let state = entries.get_mut(&delivery).ok_or(LedgerError::Corrupt)?;
                    if state.0 != binding || state.1 != DeliveryState::InDoubt {
                        return Err(LedgerError::Corrupt);
                    }
                    state.1 = DeliveryState::LocallyAcknowledged;
                }
                _ => return Err(LedgerError::Corrupt),
            }
            previous = expected;
            position = position
                .checked_add(RECORD_BYTES as u64)
                .ok_or(LedgerError::Corrupt)?;
            witnesses.insert(position, previous);
        }
        if position != data.len() as u64 {
            return Err(LedgerError::Corrupt);
        }
        Ok(Self {
            _lock: lock,
            file,
            last: previous,
            size: data.len() as u64,
            entries,
            witnesses,
            poisoned: false,
        })
    }
    fn append(
        &mut self,
        tag: u8,
        delivery: &[u8; 24],
        binding: &[u8; 32],
    ) -> Result<(), LedgerError> {
        if self.poisoned {
            return Err(LedgerError::Poisoned);
        }
        if self.size + RECORD_BYTES as u64 > MAX_LEDGER_BYTES {
            return Err(LedgerError::Full);
        }
        let mut body = [0_u8; RECORD_BODY];
        body[0] = tag;
        body[1..25].copy_from_slice(delivery);
        body[25..57].copy_from_slice(binding);
        let next = digest(&self.last, &body);
        let result = (|| -> std::io::Result<()> {
            self.file.write_all(&body)?;
            self.file.write_all(&next)?;
            self.file.sync_all()
        })();
        if let Err(error) = result {
            self.poisoned = true;
            return Err(LedgerError::Io(error));
        }
        self.last = next;
        self.size += RECORD_BYTES as u64;
        self.witnesses.insert(self.size, self.last);
        Ok(())
    }
    fn reserve(&mut self, id: [u8; 24], binding: [u8; 32]) -> Result<(), LedgerError> {
        if self.poisoned {
            return Err(LedgerError::Poisoned);
        }
        if self.entries.contains_key(&id) {
            return Err(LedgerError::DuplicateDelivery);
        }
        self.append(1, &id, &binding)?;
        self.entries.insert(id, (binding, DeliveryState::InDoubt));
        Ok(())
    }
    fn acknowledged(&mut self, id: &[u8; 24], binding: &[u8; 32]) -> Result<(), LedgerError> {
        if self.entries.get(id) != Some(&(*binding, DeliveryState::InDoubt)) {
            return Err(LedgerError::Corrupt);
        }
        self.append(2, id, binding)?;
        if let Some(entry) = self.entries.get_mut(id) {
            entry.1 = DeliveryState::LocallyAcknowledged;
        }
        Ok(())
    }
    fn status(&self, id: &[u8; 24]) -> Option<DeliveryState> {
        self.entries.get(id).map(|entry| entry.1)
    }

    fn witness(&self) -> DeliveryLedgerWitness {
        DeliveryLedgerWitness::from_parts(self.size, self.last)
    }

    fn contains_witness(&self, witness: DeliveryLedgerWitness) -> Result<bool, LedgerError> {
        if self.poisoned {
            return Err(LedgerError::Poisoned);
        }
        Ok(self.witnesses.get(&witness.bytes) == Some(&witness.digest))
    }
}

/// Opt-in production host backend: the underlying backend and admission gate
/// cannot be accessed through this wrapper's public API. This does not make
/// other independently constructed `EffectBackend` values impossible.
pub struct ProductionEffectBackend<B, S> {
    inner: GuardedEffectBackend<B, S>,
    ledger: DeliveryLedger,
    anchor: Option<Box<dyn MonotonicAnchor>>,
}
impl<B, S> ProductionEffectBackend<B, S> {
    /// Initialize a brand new local ledger (never truncate an existing one).
    pub fn initialize(inner: GuardedEffectBackend<B, S>, path: &Path) -> Result<Self, LedgerError> {
        Ok(Self {
            inner,
            ledger: DeliveryLedger::initialize(path)?,
            anchor: None,
        })
    }
    /// On restart, verify the ENTIRE ledger; never auto-release IN_DOUBT.
    pub fn open(inner: GuardedEffectBackend<B, S>, path: &Path) -> Result<Self, LedgerError> {
        Ok(Self {
            inner,
            ledger: DeliveryLedger::open(path)?,
            anchor: None,
        })
    }
    pub fn status(&self, request: &EffectDispatchRequest) -> Option<DeliveryState> {
        key(request).ok().and_then(|id| self.ledger.status(&id))
    }

    pub fn trust_witness(&self) -> TrustJournalWitness {
        self.inner.trust.journal_witness()
    }

    pub fn contains_trust_witness(
        &self,
        witness: TrustJournalWitness,
    ) -> Result<bool, DurableError> {
        self.inner.trust.contains_journal_witness(witness)
    }

    pub fn delivery_witness(&self) -> DeliveryLedgerWitness {
        self.ledger.witness()
    }

    pub fn contains_delivery_witness(
        &self,
        witness: DeliveryLedgerWitness,
    ) -> Result<bool, LedgerError> {
        self.ledger.contains_witness(witness)
    }

    pub(crate) fn attach_anchor<A: MonotonicAnchor + 'static>(
        &mut self,
        mut anchor: A,
        mode: AnchorOpenMode,
    ) -> Result<(), RecoveryError> {
        match mode {
            AnchorOpenMode::Initialize => {
                initialize_production_anchor(&mut anchor, self)?;
            }
            AnchorOpenMode::Recover => {
                verify_production_recovery(&mut anchor, self)?;
            }
        }
        self.anchor = Some(Box::new(anchor));
        Ok(())
    }

    fn anchor_barrier(&mut self) -> Result<(), EffectBackendError> {
        let Some(mut anchor) = self.anchor.take() else {
            // Direct P0.4 use retains its certified historical scope. The selected
            // P0.5A/P0.5C facade always installs an anchor in its constructor.
            return Ok(());
        };
        let result = verify_production_recovery(&mut *anchor, &*self);
        self.anchor = Some(anchor);
        result
            .map(|_| ())
            .map_err(|_| EffectBackendError::permanent("P0.6B monotonic anchor barrier rejected"))
    }
}

impl<B, S> ProductionWitnessSource for ProductionEffectBackend<B, S> {
    type Error = ();

    fn trust_witness(&self) -> ChainWitness {
        let witness = ProductionEffectBackend::trust_witness(self);
        ChainWitness::from_parts(witness.bytes(), witness.digest())
    }

    fn contains_trust_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
        ProductionEffectBackend::contains_trust_witness(
            self,
            TrustJournalWitness::from_parts(witness.position(), witness.digest()),
        )
        .map_err(|_| ())
    }

    fn delivery_witness(&self) -> ChainWitness {
        let witness = ProductionEffectBackend::delivery_witness(self);
        ChainWitness::from_parts(witness.bytes(), witness.digest())
    }

    fn contains_delivery_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
        ProductionEffectBackend::contains_delivery_witness(
            self,
            DeliveryLedgerWitness::from_parts(witness.position(), witness.digest()),
        )
        .map_err(|_| ())
    }
}

impl<B: EffectBackend, S: SignedEnvelopeSource> EffectBackend for ProductionEffectBackend<B, S> {
    fn supports(&self, effect: &Effect) -> bool {
        self.inner.supports(effect)
    }
    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Err(EffectBackendError::permanent(
            "P0.4 contextless host dispatch denied",
        ))
    }
    fn execute_with_context(
        &mut self,
        request: &EffectDispatchRequest,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        if !self.inner.supports(&request.queued.intent.effect) {
            return Err(EffectBackendError::permanent("P0.4 unsupported effect"));
        }
        let binding = canonical_dispatch_bytes(request)
            .map_err(|_| EffectBackendError::permanent("P0.4 malformed delivery context"))?;
        let delivery = key(request)
            .map_err(|_| EffectBackendError::permanent("P0.4 invalid delivery identity"))?;
        let binding_hash = sha256_payload(&binding);
        // Reserve before either signature consumption or real external execution.
        // This can conservatively classify an unsigned request as IN_DOUBT.
        self.ledger.reserve(delivery, binding_hash).map_err(|_| {
            EffectBackendError::permanent("P0.4 duplicate/in-doubt/journal unavailable")
        })?;

        // P0.3 performs admission separately so the signed P0.2 transition is
        // fsynced before the P0.6B pre-effect anchor barrier.
        self.inner.admit_with_context(request)?;
        self.anchor_barrier()?;

        // No external backend call is allowed before the anchor has durably
        // caught up to both the P0.2 admission and P0.4 reservation witnesses.
        let receipt = self.inner.execute_admitted_with_context(request)?;

        self.ledger
            .acknowledged(&delivery, &binding_hash)
            .map_err(|_| {
                EffectBackendError::permanent("P0.4 acknowledgement journal failed; IN_DOUBT")
            })?;

        // Anchor the local ACK as a second monotonic transition. If this fails,
        // the caller receives an error but the delivery identity is already
        // locally acknowledged and cannot be automatically executed again.
        self.anchor_barrier()?;
        Ok(receipt)
    }
}
