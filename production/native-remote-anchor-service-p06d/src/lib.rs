#![forbid(unsafe_code)]
//! NORDOI Production Track P0.6D: authenticated durable remote monotonic-anchor service.
//!
//! P0.6D adds the server counterpart to the P0.6C client without modifying the certified
//! client or the P0.6A/P0.6B protected path. The service speaks the exact bounded P0.6C wire
//! protocol and persists accepted anchor transitions to an authenticated append-only journal.
//!
//! Every journal entry is Ed25519-signed by the configured server key and chained to the
//! previous entry. A successful compare-and-advance response is emitted only after the new
//! entry has been written and `sync_all()` has completed.
//!
//! IMPORTANT SCOPE:
//! - P0.6D can certify service logic, protocol compatibility, durable append ordering,
//!   authenticated journal replay, fail-closed corruption handling, and restart recovery.
//! - P0.6D does NOT prove that a deployed service is in an independent failure domain.
//! - P0.6D does NOT make a single server filesystem intrinsically rollback-resistant.
//!   Restoring the entire authenticated journal to an older valid snapshot remains possible
//!   until an external monotonic/root-of-trust mechanism is separately certified.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use nordoi_rollback_anchor_p06::{ChainWitness, ProductionAnchorRecord};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

const REQUEST_MAGIC: [u8; 8] = *b"NRAREQ01";
const RESPONSE_MAGIC: [u8; 8] = *b"NRARES01";
const PROTOCOL_VERSION: u8 = 1;
const OP_READ: u8 = 1;
const OP_COMPARE_AND_ADVANCE: u8 = 2;

const STATUS_OK: u8 = 0;
const STATUS_CONFLICT: u8 = 1;
const STATUS_CORRUPT: u8 = 2;
const STATUS_UNAVAILABLE: u8 = 3;

const RECORD_BYTES: usize = 8 + 8 + 32 + 8 + 32;
const SIGNATURE_BYTES: usize = 64;
const REQUEST_ID_BYTES: usize = 32;
const REQUEST_SIGNED_BYTES: usize =
    REQUEST_MAGIC.len() + 1 + 1 + REQUEST_ID_BYTES + 1 + RECORD_BYTES + RECORD_BYTES;
const REQUEST_BYTES: usize = REQUEST_SIGNED_BYTES + SIGNATURE_BYTES;
const RESPONSE_SIGNED_BYTES: usize =
    RESPONSE_MAGIC.len() + 1 + 1 + REQUEST_ID_BYTES + 1 + RECORD_BYTES;
const RESPONSE_BYTES: usize = RESPONSE_SIGNED_BYTES + SIGNATURE_BYTES;

const STORE_MAGIC: [u8; 8] = *b"NRAST001";
const STORE_VERSION: u8 = 1;
const STORE_HEADER_BYTES: usize = 16;
const JOURNAL_PREV_HASH_BYTES: usize = 32;
const JOURNAL_ENTRY_BYTES: usize = JOURNAL_PREV_HASH_BYTES + RECORD_BYTES + SIGNATURE_BYTES;
const ROOT_DOMAIN: &[u8] = b"NORDOI-P06D-ROOT-V1";
const ENTRY_SIGNATURE_DOMAIN: &[u8] = b"NORDOI-P06D-ENTRY-SIGNATURE-V1";
const ENTRY_HASH_DOMAIN: &[u8] = b"NORDOI-P06D-ENTRY-HASH-V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceError {
    InvalidConfig,
    Corrupt,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdvanceOutcome {
    Committed,
    Conflict(Option<ProductionAnchorRecord>),
}

/// Append-only authenticated anchor journal.
///
/// The journal protects against accidental corruption and offline rewriting without the server
/// signing key. It does not, by itself, detect restoration of the complete file to an older valid
/// version; that requires a separately certified monotonic root of trust.
pub struct DurableAnchorStore {
    path: PathBuf,
    file: File,
    current: Option<ProductionAnchorRecord>,
    last_hash: [u8; 32],
    journal_secret: [u8; 32],
    poisoned: bool,
}

impl DurableAnchorStore {
    pub fn open_or_create(
        path: impl AsRef<Path>,
        journal_secret: [u8; 32],
    ) -> Result<Self, ServiceError> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|_| ServiceError::Unavailable)?;

        let mut len = file
            .metadata()
            .map_err(|_| ServiceError::Unavailable)?
            .len();

        if len == 0 {
            let header = store_header();
            file.seek(SeekFrom::Start(0))
                .map_err(|_| ServiceError::Unavailable)?;
            file.write_all(&header)
                .map_err(|_| ServiceError::Unavailable)?;
            file.sync_all().map_err(|_| ServiceError::Unavailable)?;
            len = STORE_HEADER_BYTES as u64;
        } else if len < STORE_HEADER_BYTES as u64 {
            return Err(ServiceError::Corrupt);
        }

        file.seek(SeekFrom::Start(0))
            .map_err(|_| ServiceError::Unavailable)?;
        let mut header = [0_u8; STORE_HEADER_BYTES];
        file.read_exact(&mut header)
            .map_err(|_| ServiceError::Corrupt)?;
        if header != store_header() {
            return Err(ServiceError::Corrupt);
        }

        let payload = len - STORE_HEADER_BYTES as u64;
        let complete_entries = payload / JOURNAL_ENTRY_BYTES as u64;
        let remainder = payload % JOURNAL_ENTRY_BYTES as u64;

        if remainder != 0 {
            let recovered_len =
                STORE_HEADER_BYTES as u64 + complete_entries * JOURNAL_ENTRY_BYTES as u64;
            file.set_len(recovered_len)
                .map_err(|_| ServiceError::Unavailable)?;
            file.sync_all().map_err(|_| ServiceError::Unavailable)?;
            len = recovered_len;
        }

        let verifying_key = SigningKey::from_bytes(&journal_secret).verifying_key();
        let mut last_hash = root_hash();
        let mut current = None;

        file.seek(SeekFrom::Start(STORE_HEADER_BYTES as u64))
            .map_err(|_| ServiceError::Unavailable)?;

        let entry_count = (len - STORE_HEADER_BYTES as u64) / JOURNAL_ENTRY_BYTES as u64;
        for _ in 0..entry_count {
            let mut entry = [0_u8; JOURNAL_ENTRY_BYTES];
            file.read_exact(&mut entry)
                .map_err(|_| ServiceError::Corrupt)?;

            let prev_hash: [u8; 32] = entry[..32].try_into().map_err(|_| ServiceError::Corrupt)?;
            if prev_hash != last_hash {
                return Err(ServiceError::Corrupt);
            }

            let record_start = 32;
            let record_end = record_start + RECORD_BYTES;
            let record = decode_record(&entry[record_start..record_end])?;

            let signature_bytes: [u8; SIGNATURE_BYTES] = entry[record_end..]
                .try_into()
                .map_err(|_| ServiceError::Corrupt)?;
            let signature = Signature::from_bytes(&signature_bytes);
            let signed = journal_signed_bytes(&prev_hash, &entry[record_start..record_end]);
            verifying_key
                .verify_strict(&signed, &signature)
                .map_err(|_| ServiceError::Corrupt)?;

            if !is_monotonic(current, record) {
                return Err(ServiceError::Corrupt);
            }

            last_hash = journal_entry_hash(&entry);
            current = Some(record);
        }

        file.seek(SeekFrom::End(0))
            .map_err(|_| ServiceError::Unavailable)?;

        Ok(Self {
            path,
            file,
            current,
            last_hash,
            journal_secret,
            poisoned: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self) -> Result<Option<ProductionAnchorRecord>, ServiceError> {
        if self.poisoned {
            return Err(ServiceError::Unavailable);
        }
        Ok(self.current)
    }

    fn compare_and_advance(
        &mut self,
        expected: Option<ProductionAnchorRecord>,
        next: ProductionAnchorRecord,
    ) -> Result<AdvanceOutcome, ServiceError> {
        if self.poisoned {
            return Err(ServiceError::Unavailable);
        }

        if self.current != expected || !is_monotonic(self.current, next) {
            return Ok(AdvanceOutcome::Conflict(self.current));
        }

        let record_bytes = encode_record(next);
        let mut entry = [0_u8; JOURNAL_ENTRY_BYTES];
        entry[..32].copy_from_slice(&self.last_hash);
        entry[32..32 + RECORD_BYTES].copy_from_slice(&record_bytes);

        let signing_key = SigningKey::from_bytes(&self.journal_secret);
        let signed = journal_signed_bytes(&self.last_hash, &record_bytes);
        let signature = signing_key.sign(&signed);
        entry[32 + RECORD_BYTES..].copy_from_slice(&signature.to_bytes());

        let io_result = self
            .file
            .seek(SeekFrom::End(0))
            .and_then(|_| self.file.write_all(&entry))
            .and_then(|_| self.file.sync_all());

        if io_result.is_err() {
            self.poisoned = true;
            return Err(ServiceError::Unavailable);
        }

        self.last_hash = journal_entry_hash(&entry);
        self.current = Some(next);
        Ok(AdvanceOutcome::Committed)
    }
}

/// P0.6D service counterpart for the P0.6C `RemoteAnchorClient`.
///
/// The store is mutex-protected so request handling preserves atomic compare-and-advance
/// semantics even if a future transport wrapper dispatches connections concurrently.
pub struct RemoteAnchorService {
    store: Mutex<DurableAnchorStore>,
    client_key: VerifyingKey,
    server_secret: [u8; 32],
    timeout: Duration,
}

impl RemoteAnchorService {
    pub fn open(
        store_path: impl AsRef<Path>,
        client_public: [u8; 32],
        server_secret: [u8; 32],
        timeout: Duration,
    ) -> Result<Self, ServiceError> {
        if timeout.is_zero() {
            return Err(ServiceError::InvalidConfig);
        }
        let client_key =
            VerifyingKey::from_bytes(&client_public).map_err(|_| ServiceError::InvalidConfig)?;
        let store = DurableAnchorStore::open_or_create(store_path, server_secret)?;
        Ok(Self {
            store: Mutex::new(store),
            client_key,
            server_secret,
            timeout,
        })
    }

    /// Serve a single TCP connection until the client closes it or the read timeout expires.
    pub fn serve_stream(&self, mut stream: TcpStream) -> Result<(), ServiceError> {
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|_| ServiceError::Unavailable)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|_| ServiceError::Unavailable)?;
        stream
            .set_nodelay(true)
            .map_err(|_| ServiceError::Unavailable)?;

        loop {
            let mut request = [0_u8; REQUEST_BYTES];
            match stream.read_exact(&mut request) {
                Ok(()) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::UnexpectedEof
                            | io::ErrorKind::ConnectionReset
                            | io::ErrorKind::BrokenPipe
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::WouldBlock
                    ) =>
                {
                    return Ok(());
                }
                Err(_) => return Err(ServiceError::Unavailable),
            }

            let response = self.handle_request(&request);
            stream
                .write_all(&response)
                .and_then(|_| stream.flush())
                .map_err(|_| ServiceError::Unavailable)?;
        }
    }

    /// Minimal blocking service loop.
    ///
    /// P0.6D deliberately avoids an async runtime and a general RPC framework. One connection is
    /// served at a time; the P0.6C client reuses its healthy connection, while read timeouts prevent
    /// a dead peer from holding the listener indefinitely.
    pub fn run(&self, bind: SocketAddr) -> Result<(), ServiceError> {
        let listener = TcpListener::bind(bind).map_err(|_| ServiceError::Unavailable)?;
        for incoming in listener.incoming() {
            let stream = incoming.map_err(|_| ServiceError::Unavailable)?;
            self.serve_stream(stream)?;
        }
        Ok(())
    }

    fn handle_request(&self, request: &[u8; REQUEST_BYTES]) -> [u8; RESPONSE_BYTES] {
        let request_id: [u8; REQUEST_ID_BYTES] = request[10..42]
            .try_into()
            .unwrap_or([0_u8; REQUEST_ID_BYTES]);

        if request[..8] != REQUEST_MAGIC[..] || request[8] != PROTOCOL_VERSION {
            return self.response(STATUS_CORRUPT, request_id, None);
        }

        let signature_bytes: [u8; SIGNATURE_BYTES] =
            match request[REQUEST_SIGNED_BYTES..].try_into() {
                Ok(value) => value,
                Err(_) => return self.response(STATUS_CORRUPT, request_id, None),
            };
        let signature = Signature::from_bytes(&signature_bytes);
        if self
            .client_key
            .verify_strict(&request[..REQUEST_SIGNED_BYTES], &signature)
            .is_err()
        {
            return self.response(STATUS_UNAVAILABLE, request_id, None);
        }

        match request[9] {
            OP_READ => self.handle_read(request, request_id),
            OP_COMPARE_AND_ADVANCE => self.handle_compare_and_advance(request, request_id),
            _ => self.response(STATUS_CORRUPT, request_id, None),
        }
    }

    fn handle_read(
        &self,
        request: &[u8; REQUEST_BYTES],
        request_id: [u8; REQUEST_ID_BYTES],
    ) -> [u8; RESPONSE_BYTES] {
        let expected_slot = &request[43..43 + RECORD_BYTES];
        let next_start = 43 + RECORD_BYTES;
        let next_slot = &request[next_start..next_start + RECORD_BYTES];

        if request[42] != 0
            || expected_slot.iter().any(|byte| *byte != 0)
            || next_slot.iter().any(|byte| *byte != 0)
        {
            return self.response(STATUS_CORRUPT, request_id, None);
        }

        match self.store.lock() {
            Ok(store) => match store.read() {
                Ok(record) => self.response(STATUS_OK, request_id, record),
                Err(ServiceError::Corrupt) => self.response(STATUS_CORRUPT, request_id, None),
                Err(_) => self.response(STATUS_UNAVAILABLE, request_id, None),
            },
            Err(_) => self.response(STATUS_UNAVAILABLE, request_id, None),
        }
    }

    fn handle_compare_and_advance(
        &self,
        request: &[u8; REQUEST_BYTES],
        request_id: [u8; REQUEST_ID_BYTES],
    ) -> [u8; RESPONSE_BYTES] {
        let expected_slot = &request[43..43 + RECORD_BYTES];
        let next_start = 43 + RECORD_BYTES;
        let next_slot = &request[next_start..next_start + RECORD_BYTES];

        let expected = match request[42] {
            0 => {
                if expected_slot.iter().any(|byte| *byte != 0) {
                    return self.response(STATUS_CORRUPT, request_id, None);
                }
                None
            }
            1 => match decode_record(expected_slot) {
                Ok(record) => Some(record),
                Err(_) => return self.response(STATUS_CORRUPT, request_id, None),
            },
            _ => return self.response(STATUS_CORRUPT, request_id, None),
        };

        let next = match decode_record(next_slot) {
            Ok(record) => record,
            Err(_) => return self.response(STATUS_CORRUPT, request_id, None),
        };

        match self.store.lock() {
            Ok(mut store) => match store.compare_and_advance(expected, next) {
                Ok(AdvanceOutcome::Committed) => self.response(STATUS_OK, request_id, Some(next)),
                Ok(AdvanceOutcome::Conflict(current)) => {
                    self.response(STATUS_CONFLICT, request_id, current)
                }
                Err(ServiceError::Corrupt) => self.response(STATUS_CORRUPT, request_id, None),
                Err(_) => self.response(STATUS_UNAVAILABLE, request_id, None),
            },
            Err(_) => self.response(STATUS_UNAVAILABLE, request_id, None),
        }
    }

    fn response(
        &self,
        status: u8,
        request_id: [u8; REQUEST_ID_BYTES],
        record: Option<ProductionAnchorRecord>,
    ) -> [u8; RESPONSE_BYTES] {
        let mut response = [0_u8; RESPONSE_BYTES];
        response[..8].copy_from_slice(&RESPONSE_MAGIC);
        response[8] = PROTOCOL_VERSION;
        response[9] = status;
        response[10..42].copy_from_slice(&request_id);
        response[42] = u8::from(record.is_some());
        if let Some(record) = record {
            response[43..43 + RECORD_BYTES].copy_from_slice(&encode_record(record));
        }

        let server_key = SigningKey::from_bytes(&self.server_secret);
        let signature = server_key.sign(&response[..RESPONSE_SIGNED_BYTES]);
        response[RESPONSE_SIGNED_BYTES..].copy_from_slice(&signature.to_bytes());
        response
    }
}

fn store_header() -> [u8; STORE_HEADER_BYTES] {
    let mut header = [0_u8; STORE_HEADER_BYTES];
    header[..8].copy_from_slice(&STORE_MAGIC);
    header[8] = STORE_VERSION;
    header
}

fn root_hash() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ROOT_DOMAIN);
    hasher.finalize().into()
}

fn journal_signed_bytes(prev_hash: &[u8; 32], record: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ENTRY_SIGNATURE_DOMAIN.len() + 32 + RECORD_BYTES);
    out.extend_from_slice(ENTRY_SIGNATURE_DOMAIN);
    out.extend_from_slice(prev_hash);
    out.extend_from_slice(record);
    out
}

fn journal_entry_hash(entry: &[u8; JOURNAL_ENTRY_BYTES]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ENTRY_HASH_DOMAIN);
    hasher.update(entry);
    hasher.finalize().into()
}

fn is_monotonic(previous: Option<ProductionAnchorRecord>, next: ProductionAnchorRecord) -> bool {
    match previous {
        None => next.generation() == 1,
        Some(previous) => {
            next.generation() == previous.generation() + 1
                && next.trust().position() >= previous.trust().position()
                && next.delivery().position() >= previous.delivery().position()
                && (next.trust().position() != previous.trust().position()
                    || next.trust().digest() == previous.trust().digest())
                && (next.delivery().position() != previous.delivery().position()
                    || next.delivery().digest() == previous.delivery().digest())
        }
    }
}

fn encode_record(record: ProductionAnchorRecord) -> [u8; RECORD_BYTES] {
    let mut out = [0_u8; RECORD_BYTES];
    out[0..8].copy_from_slice(&record.generation().to_be_bytes());
    out[8..16].copy_from_slice(&record.trust().position().to_be_bytes());
    out[16..48].copy_from_slice(&record.trust().digest());
    out[48..56].copy_from_slice(&record.delivery().position().to_be_bytes());
    out[56..88].copy_from_slice(&record.delivery().digest());
    out
}

fn decode_record(bytes: &[u8]) -> Result<ProductionAnchorRecord, ServiceError> {
    if bytes.len() != RECORD_BYTES {
        return Err(ServiceError::Corrupt);
    }

    let generation = read_u64(bytes, 0)?;
    let trust_position = read_u64(bytes, 8)?;
    let trust_digest: [u8; 32] = bytes[16..48]
        .try_into()
        .map_err(|_| ServiceError::Corrupt)?;
    let delivery_position = read_u64(bytes, 48)?;
    let delivery_digest: [u8; 32] = bytes[56..88]
        .try_into()
        .map_err(|_| ServiceError::Corrupt)?;

    Ok(ProductionAnchorRecord::new(
        generation,
        ChainWitness::from_parts(trust_position, trust_digest),
        ChainWitness::from_parts(delivery_position, delivery_digest),
    ))
}

fn read_u64(bytes: &[u8], start: usize) -> Result<u64, ServiceError> {
    let end = start + 8;
    let raw: [u8; 8] = bytes
        .get(start..end)
        .ok_or(ServiceError::Corrupt)?
        .try_into()
        .map_err(|_| ServiceError::Corrupt)?;
    Ok(u64::from_be_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nordoi_remote_anchor_p06c::RemoteAnchorClient;
    use nordoi_rollback_anchor_p06::{
        initialize_production_anchor, verify_production_recovery, ChainWitness, MonotonicAnchor,
        ProductionWitnessSource, RecoveryError,
    };
    use std::fs;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    const CLIENT_SECRET: [u8; 32] = [31; 32];
    const OTHER_CLIENT_SECRET: [u8; 32] = [32; 32];
    const SERVER_SECRET: [u8; 32] = [41; 32];

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_path(label: &str) -> PathBuf {
        let index = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "nordoi-p06d-{}-{}-{}.journal",
            std::process::id(),
            index,
            label
        ))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn record(generation: u64, trust: u64, delivery: u64) -> ProductionAnchorRecord {
        ProductionAnchorRecord::new(
            generation,
            ChainWitness::from_parts(trust, [trust as u8; 32]),
            ChainWitness::from_parts(delivery, [delivery as u8; 32]),
        )
    }

    fn record_with_digests(
        generation: u64,
        trust: u64,
        trust_digest: [u8; 32],
        delivery: u64,
        delivery_digest: [u8; 32],
    ) -> ProductionAnchorRecord {
        ProductionAnchorRecord::new(
            generation,
            ChainWitness::from_parts(trust, trust_digest),
            ChainWitness::from_parts(delivery, delivery_digest),
        )
    }

    fn client_public(secret: [u8; 32]) -> [u8; 32] {
        SigningKey::from_bytes(&secret).verifying_key().to_bytes()
    }

    fn spawn_service(
        path: PathBuf,
        expected_client_secret: [u8; 32],
    ) -> (SocketAddr, thread::JoinHandle<()>) {
        let service = RemoteAnchorService::open(
            &path,
            client_public(expected_client_secret),
            SERVER_SECRET,
            Duration::from_secs(2),
        )
        .expect("open service");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let address = listener.local_addr().expect("address");
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            service.serve_stream(stream).expect("serve");
        });
        (address, handle)
    }

    fn client(address: SocketAddr, secret: [u8; 32]) -> RemoteAnchorClient {
        RemoteAnchorClient::new(
            address,
            secret,
            client_public(SERVER_SECRET),
            Duration::from_secs(2),
        )
        .expect("client")
    }

    #[derive(Clone, Copy)]
    struct Source {
        trust: ChainWitness,
        delivery: ChainWitness,
    }

    impl Source {
        fn at(trust: u64, delivery: u64) -> Self {
            Self {
                trust: ChainWitness::from_parts(trust, [trust as u8; 32]),
                delivery: ChainWitness::from_parts(delivery, [delivery as u8; 32]),
            }
        }
    }

    impl ProductionWitnessSource for Source {
        type Error = ();

        fn trust_witness(&self) -> ChainWitness {
            self.trust
        }

        fn contains_trust_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
            Ok(witness == self.trust)
        }

        fn delivery_witness(&self) -> ChainWitness {
            self.delivery
        }

        fn contains_delivery_witness(&self, witness: ChainWitness) -> Result<bool, Self::Error> {
            Ok(witness == self.delivery)
        }
    }

    #[test]
    fn empty_store_is_durable() {
        let path = temp_path("empty");
        {
            let store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
            assert_eq!(store.read(), Ok(None));
        }
        {
            let store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("reopen");
            assert_eq!(store.read(), Ok(None));
        }
        cleanup(&path);
    }

    #[test]
    fn committed_record_survives_reopen() {
        let path = temp_path("reopen");
        let first = record(1, 5, 7);
        {
            let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
            assert_eq!(
                store.compare_and_advance(None, first),
                Ok(AdvanceOutcome::Committed)
            );
        }
        {
            let store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("reopen");
            assert_eq!(store.read(), Ok(Some(first)));
        }
        cleanup(&path);
    }

    #[test]
    fn conflict_does_not_append() {
        let path = temp_path("conflict");
        let first = record(1, 5, 7);
        let second = record(2, 6, 8);
        let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
        assert_eq!(
            store.compare_and_advance(None, first),
            Ok(AdvanceOutcome::Committed)
        );
        let before = fs::metadata(&path).expect("metadata").len();
        assert_eq!(
            store.compare_and_advance(None, second),
            Ok(AdvanceOutcome::Conflict(Some(first)))
        );
        let after = fs::metadata(&path).expect("metadata").len();
        assert_eq!(before, after);
        cleanup(&path);
    }

    #[test]
    fn first_commit_must_use_generation_one() {
        let path = temp_path("generation-one");
        let invalid = record(2, 5, 7);
        let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
        assert_eq!(
            store.compare_and_advance(None, invalid),
            Ok(AdvanceOutcome::Conflict(None))
        );
        assert_eq!(store.read(), Ok(None));
        cleanup(&path);
    }

    #[test]
    fn non_monotonic_generation_is_rejected() {
        let path = temp_path("generation");
        let first = record(1, 5, 7);
        let invalid = record(1, 6, 8);
        let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
        assert_eq!(
            store.compare_and_advance(None, first),
            Ok(AdvanceOutcome::Committed)
        );
        assert_eq!(
            store.compare_and_advance(Some(first), invalid),
            Ok(AdvanceOutcome::Conflict(Some(first)))
        );
        cleanup(&path);
    }

    #[test]
    fn equal_position_digest_rewrite_is_rejected() {
        let path = temp_path("digest-rewrite");
        let first = record_with_digests(1, 5, [5; 32], 7, [7; 32]);
        let invalid = record_with_digests(2, 5, [9; 32], 7, [7; 32]);
        let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
        assert_eq!(
            store.compare_and_advance(None, first),
            Ok(AdvanceOutcome::Committed)
        );
        assert_eq!(
            store.compare_and_advance(Some(first), invalid),
            Ok(AdvanceOutcome::Conflict(Some(first)))
        );
        cleanup(&path);
    }

    #[test]
    fn partial_tail_is_truncated_to_last_complete_entry() {
        let path = temp_path("partial-tail");
        let first = record(1, 5, 7);
        {
            let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
            assert_eq!(
                store.compare_and_advance(None, first),
                Ok(AdvanceOutcome::Committed)
            );
        }
        {
            let mut file = OpenOptions::new().append(true).open(&path).expect("append");
            file.write_all(&[0xA5; 17]).expect("partial write");
            file.sync_all().expect("partial sync");
        }
        {
            let store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("recover");
            assert_eq!(store.read(), Ok(Some(first)));
        }
        assert_eq!(
            fs::metadata(&path).expect("metadata").len(),
            (STORE_HEADER_BYTES + JOURNAL_ENTRY_BYTES) as u64
        );
        cleanup(&path);
    }

    #[test]
    fn corrupted_complete_entry_fails_closed() {
        let path = temp_path("corrupt-entry");
        let first = record(1, 5, 7);
        {
            let mut store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("open");
            assert_eq!(
                store.compare_and_advance(None, first),
                Ok(AdvanceOutcome::Committed)
            );
        }
        {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .expect("rewrite");
            let offset = STORE_HEADER_BYTES as u64 + 32 + 16;
            file.seek(SeekFrom::Start(offset)).expect("seek");
            file.write_all(&[0xFF]).expect("tamper");
            file.sync_all().expect("sync tamper");
        }
        assert!(matches!(
            DurableAnchorStore::open_or_create(&path, SERVER_SECRET),
            Err(ServiceError::Corrupt)
        ));
        cleanup(&path);
    }

    #[test]
    fn p06c_client_round_trip_against_p06d_service() {
        let path = temp_path("round-trip");
        let (address, handle) = spawn_service(path.clone(), CLIENT_SECRET);
        let first = record(1, 5, 7);
        let mut remote = client(address, CLIENT_SECRET);
        assert_eq!(remote.read(), Ok(None));
        assert_eq!(remote.compare_and_advance(None, first), Ok(()));
        assert_eq!(remote.read(), Ok(Some(first)));
        drop(remote);
        handle.join().expect("join");
        cleanup(&path);
    }

    #[test]
    fn wrong_client_key_cannot_change_durable_store() {
        let path = temp_path("wrong-client");
        let (address, handle) = spawn_service(path.clone(), CLIENT_SECRET);
        let mut remote = client(address, OTHER_CLIENT_SECRET);
        assert_eq!(
            remote.compare_and_advance(None, record(1, 5, 7)),
            Err(nordoi_rollback_anchor_p06::AnchorStoreError::Unavailable)
        );
        drop(remote);
        handle.join().expect("join");
        let store = DurableAnchorStore::open_or_create(&path, SERVER_SECRET).expect("reopen");
        assert_eq!(store.read(), Ok(None));
        cleanup(&path);
    }

    #[test]
    fn service_restart_preserves_remote_anchor() {
        let path = temp_path("service-restart");
        let first = record(1, 5, 7);

        let (address_one, handle_one) = spawn_service(path.clone(), CLIENT_SECRET);
        let mut remote_one = client(address_one, CLIENT_SECRET);
        assert_eq!(remote_one.compare_and_advance(None, first), Ok(()));
        drop(remote_one);
        handle_one.join().expect("join one");

        let (address_two, handle_two) = spawn_service(path.clone(), CLIENT_SECRET);
        let mut remote_two = client(address_two, CLIENT_SECRET);
        assert_eq!(remote_two.read(), Ok(Some(first)));
        drop(remote_two);
        handle_two.join().expect("join two");

        cleanup(&path);
    }

    #[test]
    fn p06a_recovery_detects_rollback_through_restarted_service() {
        let path = temp_path("rollback-restart");
        let original = Source::at(5, 7);
        let rolled_back = Source::at(4, 7);

        let (address_one, handle_one) = spawn_service(path.clone(), CLIENT_SECRET);
        let mut remote_one = client(address_one, CLIENT_SECRET);
        initialize_production_anchor(&mut remote_one, &original).expect("initialize");
        drop(remote_one);
        handle_one.join().expect("join one");

        let (address_two, handle_two) = spawn_service(path.clone(), CLIENT_SECRET);
        let mut remote_two = client(address_two, CLIENT_SECRET);
        assert_eq!(
            verify_production_recovery(&mut remote_two, &rolled_back),
            Err(RecoveryError::TrustRollback)
        );
        drop(remote_two);
        handle_two.join().expect("join two");

        cleanup(&path);
    }
}
