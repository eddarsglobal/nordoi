use std::{cell::RefCell, rc::Rc};

use nordoi_kernel::{
    AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, Effect,
    EffectAttestationAlgorithmId, EffectAttestationBackendError, EffectAttestationCommitReceipt,
    EffectAttestationError, EffectAttestationKeyId, EffectAttestationSigner,
    EffectAttestationStore, EffectAttestationStoreError, EffectAttestationVerifier,
    EffectAuditAttestation, EffectAuditAttestationStatement, EffectAuditCheckpoint, EffectBackend,
    EffectBackendError, EffectBackendReceipt, EffectDeliveryFence, EffectDeliveryNamespace,
    EffectDispatchAuthority, EffectFenceStoreError, EffectJournalCommitReceipt, EffectJournalLease,
    EffectJournalWriterId, EffectRetryPolicy, EffectRetryTick, EffectTrustEpoch,
    FencedEffectJournalStore, GovernedAuditedEffectJournal, GovernedEffectAttestor,
    GovernedEffectDispatcher, InputDeviceId, InputPayload, InputSequence, InputSignal, InputSource,
    InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram, NairReactionAuthority,
    NairReactionStep, NairReactionTrigger, QueuedEffectIntent, ReactionSlot,
    MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES,
};

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
}

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}
fn writer(seed: u8) -> EffectJournalWriterId {
    EffectJournalWriterId::new([seed; 16])
}
fn key(seed: u8) -> EffectAttestationKeyId {
    EffectAttestationKeyId::new([seed; 16])
}
fn policy() -> EffectRetryPolicy {
    EffectRetryPolicy::new(3, 5, 20, 0).unwrap()
}

fn keyboard_batch(code: u32, sequence: u64) -> nordoi_kernel::InputBatch {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            InputDeviceId(1),
            None,
            InputPayload::Key {
                code,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    let mut batch = input.drain();
    batch.events[0].sequence = InputSequence(sequence);
    batch
}

fn program() -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "attestation-reaction".into(),
            domain: nordoi_kernel::DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "attestation-action".into(),
            declared_effects: effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect { effect }],
        },
        Instruction::Halt,
    ]);
    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network("api.example.test".into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);
    (program, authority)
}

fn boot() -> AtomicEventLoop {
    let (program, authority) = program();
    AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap()
}

fn dispatcher() -> GovernedEffectDispatcher {
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network("api.example.test".into()));
    GovernedEffectDispatcher::new(authority)
}

#[derive(Debug, Default)]
struct JournalState {
    bytes: Option<Vec<u8>>,
    next_fence: u64,
    active: Option<EffectJournalLease>,
    commits: usize,
}

#[derive(Debug, Clone, Default)]
struct JournalStore {
    state: Rc<RefCell<JournalState>>,
}

impl JournalStore {
    fn active(&self) -> Option<EffectJournalLease> {
        self.state.borrow().active
    }
    fn bytes(&self) -> Option<Vec<u8>> {
        self.state.borrow().bytes.clone()
    }

    fn require_active(
        state: &JournalState,
        lease: EffectJournalLease,
    ) -> Result<(), EffectFenceStoreError> {
        if state.active == Some(lease) {
            Ok(())
        } else {
            Err(EffectFenceStoreError::new("stale or inactive fence"))
        }
    }
}

impl FencedEffectJournalStore for JournalStore {
    fn acquire(
        &mut self,
        namespace: EffectDeliveryNamespace,
        writer: EffectJournalWriterId,
    ) -> Result<EffectJournalLease, EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        state.next_fence = state
            .next_fence
            .checked_add(1)
            .ok_or_else(|| EffectFenceStoreError::new("fence exhausted"))?;
        let lease =
            EffectJournalLease::new(namespace, writer, EffectDeliveryFence(state.next_fence));
        state.active = Some(lease);
        Ok(lease)
    }

    fn assert_active(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        Self::require_active(&self.state.borrow(), lease)
    }

    fn load_fenced(
        &mut self,
        lease: EffectJournalLease,
    ) -> Result<Option<Vec<u8>>, EffectFenceStoreError> {
        let state = self.state.borrow();
        Self::require_active(&state, lease)?;
        Ok(state.bytes.clone())
    }

    fn commit_fenced(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectJournalCommitReceipt, EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectJournalCommitReceipt::new(format!(
            "journal-{}",
            state.commits
        )))
    }

    fn release(&mut self, lease: EffectJournalLease) -> Result<(), EffectFenceStoreError> {
        let mut state = self.state.borrow_mut();
        Self::require_active(&state, lease)?;
        state.active = None;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct AnchorState {
    bytes: Option<Vec<u8>>,
    commits: usize,
    fail_commit: bool,
}

#[derive(Debug, Clone)]
struct AnchorStore {
    journal_state: Rc<RefCell<JournalState>>,
    state: Rc<RefCell<AnchorState>>,
}

impl AnchorStore {
    fn new(journal: &JournalStore) -> Self {
        Self {
            journal_state: journal.state.clone(),
            state: Rc::new(RefCell::new(AnchorState::default())),
        }
    }

    fn commits(&self) -> usize {
        self.state.borrow().commits
    }
    fn bytes(&self) -> Option<Vec<u8>> {
        self.state.borrow().bytes.clone()
    }
    fn fail_next_commit(&self) {
        self.state.borrow_mut().fail_commit = true;
    }
}

impl EffectAttestationStore for AnchorStore {
    fn load_attestation(
        &mut self,
        _namespace: EffectDeliveryNamespace,
    ) -> Result<Option<Vec<u8>>, EffectAttestationStoreError> {
        Ok(self.state.borrow().bytes.clone())
    }

    fn commit_fenced_attestation(
        &mut self,
        lease: EffectJournalLease,
        bytes: &[u8],
    ) -> Result<EffectAttestationCommitReceipt, EffectAttestationStoreError> {
        if self.journal_state.borrow().active != Some(lease) {
            return Err(EffectAttestationStoreError::new("stale attestation fence"));
        }
        let mut state = self.state.borrow_mut();
        if state.fail_commit {
            state.fail_commit = false;
            return Err(EffectAttestationStoreError::new(
                "synthetic attestation commit failure",
            ));
        }
        state.commits += 1;
        state.bytes = Some(bytes.to_vec());
        Ok(EffectAttestationCommitReceipt::new(bytes.len() as u64))
    }
}

#[derive(Debug)]
struct TestSigner {
    key_id: EffectAttestationKeyId,
    algorithm_id: EffectAttestationAlgorithmId,
    fail: bool,
    calls: usize,
}

impl TestSigner {
    fn new(key_seed: u8, algorithm: u16) -> Self {
        Self {
            key_id: key(key_seed),
            algorithm_id: EffectAttestationAlgorithmId::new(algorithm),
            fail: false,
            calls: 0,
        }
    }

    fn failing(key_seed: u8, algorithm: u16) -> Self {
        Self {
            key_id: key(key_seed),
            algorithm_id: EffectAttestationAlgorithmId::new(algorithm),
            fail: true,
            calls: 0,
        }
    }
}

impl EffectAttestationSigner for TestSigner {
    fn key_id(&self) -> EffectAttestationKeyId {
        self.key_id
    }
    fn algorithm_id(&self) -> EffectAttestationAlgorithmId {
        self.algorithm_id
    }
    fn sign(&mut self, statement: &[u8]) -> Result<Vec<u8>, EffectAttestationBackendError> {
        self.calls += 1;
        if self.fail {
            Err(EffectAttestationBackendError::new(
                "synthetic signer failure",
            ))
        } else {
            Ok(statement.to_vec())
        }
    }
}

#[derive(Debug)]
struct TestVerifier {
    accept: bool,
    fail: bool,
    expected_key: EffectAttestationKeyId,
    expected_epoch: EffectTrustEpoch,
    expected_algorithm: EffectAttestationAlgorithmId,
}

impl TestVerifier {
    fn accepting(key_seed: u8, epoch: u64, algorithm: u16) -> Self {
        Self {
            accept: true,
            fail: false,
            expected_key: key(key_seed),
            expected_epoch: EffectTrustEpoch::new(epoch),
            expected_algorithm: EffectAttestationAlgorithmId::new(algorithm),
        }
    }
}

impl EffectAttestationVerifier for TestVerifier {
    fn verify(
        &self,
        key_id: EffectAttestationKeyId,
        trust_epoch: EffectTrustEpoch,
        algorithm_id: EffectAttestationAlgorithmId,
        statement: &[u8],
        signature: &[u8],
    ) -> Result<bool, EffectAttestationBackendError> {
        if self.fail {
            return Err(EffectAttestationBackendError::new(
                "synthetic verifier failure",
            ));
        }
        Ok(self.accept
            && key_id == self.expected_key
            && trust_epoch == self.expected_epoch
            && algorithm_id == self.expected_algorithm
            && signature == statement)
    }
}

#[derive(Debug)]
struct SuccessBackend {
    reference: &'static str,
}

impl EffectBackend for SuccessBackend {
    fn supports(&self, _effect: &Effect) -> bool {
        true
    }
    fn execute(
        &mut self,
        _request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        Ok(EffectBackendReceipt::new(self.reference))
    }
}

fn active_journal(
    seed: u8,
    writer_seed: u8,
    store: JournalStore,
) -> GovernedAuditedEffectJournal<JournalStore> {
    let mut journal =
        GovernedAuditedEffectJournal::new(namespace(seed), writer(writer_seed), store, policy());
    journal.acquire().unwrap();
    journal
}

fn stage_one(
    loop_: &mut AtomicEventLoop,
    journal: &mut GovernedAuditedEffectJournal<JournalStore>,
) {
    loop_
        .cycle_to_with_audited_effect_journal(LogicalTime::ZERO, &keyboard_batch(7, 1), journal)
        .unwrap();
}

fn audited_checkpoint(
    seed: u8,
    receipt: &'static str,
) -> (
    JournalStore,
    GovernedAuditedEffectJournal<JournalStore>,
    AtomicEventLoop,
    EffectAuditCheckpoint,
) {
    let store = JournalStore::default();
    let mut journal = active_journal(seed, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let mut backend = SuccessBackend { reference: receipt };
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(),
            &mut backend,
        )
        .unwrap();
    let checkpoint = loop_.current_effect_audit_checkpoint(&journal);
    (store, journal, loop_, checkpoint)
}

#[test]
fn attestation_statement_binds_checkpoint_and_lease() {
    let (_store, journal, loop_, checkpoint) = audited_checkpoint(1, "ok");
    let lease = journal.lease().unwrap();
    let statement = EffectAuditAttestationStatement::for_checkpoint(
        &checkpoint,
        lease.writer,
        lease.fence,
        EffectTrustEpoch::new(1),
        key(9),
        EffectAttestationAlgorithmId::new(7),
    )
    .unwrap();
    assert!(statement.matches_checkpoint(&loop_.current_effect_audit_checkpoint(&journal)));
    assert_eq!(statement.namespace, namespace(1));
    assert_eq!(statement.writer, writer(1));
    assert_eq!(statement.fence, lease.fence);
    assert_eq!(statement.audit_records, 2);
}

#[test]
fn canonical_attestation_round_trip_is_byte_stable() {
    let (_store, journal, _loop, checkpoint) = audited_checkpoint(2, "ok");
    let lease = journal.lease().unwrap();
    let statement = EffectAuditAttestationStatement::for_checkpoint(
        &checkpoint,
        lease.writer,
        lease.fence,
        EffectTrustEpoch::new(1),
        key(1),
        EffectAttestationAlgorithmId::new(1),
    )
    .unwrap();
    let attestation =
        EffectAuditAttestation::new(statement.clone(), statement.canonical_bytes()).unwrap();
    let bytes = attestation.canonical_bytes();
    let decoded = EffectAuditAttestation::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, attestation);
    assert_eq!(decoded.canonical_bytes(), bytes);
}

#[test]
fn tampered_attestation_envelope_is_rejected() {
    let (_store, journal, _loop, checkpoint) = audited_checkpoint(3, "ok");
    let lease = journal.lease().unwrap();
    let statement = EffectAuditAttestationStatement::for_checkpoint(
        &checkpoint,
        lease.writer,
        lease.fence,
        EffectTrustEpoch::new(1),
        key(1),
        EffectAttestationAlgorithmId::new(1),
    )
    .unwrap();
    let mut bytes = EffectAuditAttestation::new(statement.clone(), statement.canonical_bytes())
        .unwrap()
        .canonical_bytes();
    bytes[24] ^= 0x5a;
    assert!(matches!(
        EffectAuditAttestation::from_canonical_bytes(&bytes),
        Err(EffectAttestationError::AttestationDigestMismatch)
    ));
}

#[test]
fn zero_trust_epoch_is_rejected() {
    let journal_store = JournalStore::default();
    let anchor = AnchorStore::new(&journal_store);
    assert!(matches!(
        GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(0)),
        Err(EffectAttestationError::ZeroTrustEpoch)
    ));
}

#[test]
fn zero_algorithm_is_rejected_before_signing() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(4, "ok");
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 0);
    let result = loop_.attest_current_effect_audit(&mut journal, &mut attestor, &mut signer);
    assert!(matches!(
        result,
        Err(EffectAttestationError::ZeroAlgorithmId)
    ));
    assert_eq!(signer.calls, 0);
}

#[test]
fn empty_signature_is_rejected() {
    let (_store, journal, _loop, checkpoint) = audited_checkpoint(5, "ok");
    let lease = journal.lease().unwrap();
    let statement = EffectAuditAttestationStatement::for_checkpoint(
        &checkpoint,
        lease.writer,
        lease.fence,
        EffectTrustEpoch::new(1),
        key(1),
        EffectAttestationAlgorithmId::new(1),
    )
    .unwrap();
    assert!(matches!(
        EffectAuditAttestation::new(statement, Vec::new()),
        Err(EffectAttestationError::EmptySignature)
    ));
}

#[test]
fn oversized_signature_is_rejected() {
    let (_store, journal, _loop, checkpoint) = audited_checkpoint(6, "ok");
    let lease = journal.lease().unwrap();
    let statement = EffectAuditAttestationStatement::for_checkpoint(
        &checkpoint,
        lease.writer,
        lease.fence,
        EffectTrustEpoch::new(1),
        key(1),
        EffectAttestationAlgorithmId::new(1),
    )
    .unwrap();
    let signature = vec![0_u8; MAX_EFFECT_ATTESTATION_SIGNATURE_BYTES + 1];
    assert!(matches!(
        EffectAuditAttestation::new(statement, signature),
        Err(EffectAttestationError::SignatureTooLarge { .. })
    ));
}

#[test]
fn attest_current_commits_anchor_without_rewriting_journal() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(7, "ok");
    let before = store.bytes().unwrap();
    let anchor = AnchorStore::new(&store);
    let anchor_view = anchor.clone();
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    let (attestation, receipt) = loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    assert_eq!(anchor_view.commits(), 1);
    assert_eq!(receipt.bytes as usize, anchor_view.bytes().unwrap().len());
    assert_eq!(store.bytes().unwrap(), before);
    assert_eq!(attestation.statement.audit_records, 2);
}

#[test]
fn signer_failure_creates_zero_anchor_commits() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(8, "ok");
    let anchor = AnchorStore::new(&store);
    let view = anchor.clone();
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::failing(1, 1);
    let result = loop_.attest_current_effect_audit(&mut journal, &mut attestor, &mut signer);
    assert!(matches!(result, Err(EffectAttestationError::Backend(_))));
    assert_eq!(view.commits(), 0);
}

#[test]
fn anchor_store_failure_does_not_change_journal() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(9, "ok");
    let journal_before = store.bytes().unwrap();
    let anchor = AnchorStore::new(&store);
    anchor.fail_next_commit();
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    let result = loop_.attest_current_effect_audit(&mut journal, &mut attestor, &mut signer);
    assert!(matches!(result, Err(EffectAttestationError::Store(_))));
    assert_eq!(store.bytes().unwrap(), journal_before);
}

#[test]
fn latest_attestation_verifies_against_current_durable_checkpoint() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(10, "ok");
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(3)).unwrap();
    let mut signer = TestSigner::new(4, 7);
    loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    let verifier = TestVerifier::accepting(4, 3, 7);
    let verified = loop_
        .verify_current_effect_audit_attestation(&mut journal, &mut attestor, &verifier)
        .unwrap()
        .unwrap();
    assert_eq!(verified.statement.trust_epoch, EffectTrustEpoch::new(3));
}

#[test]
fn verifier_rejection_fails_closed() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(11, "ok");
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    let mut verifier = TestVerifier::accepting(1, 1, 1);
    verifier.accept = false;
    assert!(matches!(
        loop_.verify_current_effect_audit_attestation(&mut journal, &mut attestor, &verifier),
        Err(EffectAttestationError::SignatureRejected)
    ));
}

#[test]
fn verifier_backend_failure_is_propagated() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(12, "ok");
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    let mut verifier = TestVerifier::accepting(1, 1, 1);
    verifier.fail = true;
    assert!(matches!(
        loop_.verify_current_effect_audit_attestation(&mut journal, &mut attestor, &verifier),
        Err(EffectAttestationError::Backend(_))
    ));
}

#[test]
fn missing_anchor_is_explicit_none() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(13, "ok");
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let verifier = TestVerifier::accepting(1, 1, 1);
    assert!(loop_
        .verify_current_effect_audit_attestation(&mut journal, &mut attestor, &verifier)
        .unwrap()
        .is_none());
}

#[test]
fn attest_current_requires_existing_durable_checkpoint() {
    let store = JournalStore::default();
    let mut journal = active_journal(14, 1, store.clone());
    let loop_ = boot();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    assert!(matches!(
        loop_.attest_current_effect_audit(&mut journal, &mut attestor, &mut signer),
        Err(EffectAttestationError::DurableCheckpointMissing)
    ));
}

#[test]
fn live_state_must_match_durable_state_before_attestation() {
    let store = JournalStore::default();
    let mut journal = active_journal(15, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    loop_
        .cycle_to(LogicalTime(1), &keyboard_batch(7, 2))
        .unwrap();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    assert!(matches!(
        loop_.attest_current_effect_audit(&mut journal, &mut attestor, &mut signer),
        Err(EffectAttestationError::DurableCheckpointMismatch)
    ));
}

#[test]
fn key_change_requires_trust_epoch_advance() {
    let (store, journal, _loop, checkpoint) = audited_checkpoint(16, "ok");
    let lease = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut first = TestSigner::new(1, 1);
    attestor.attest(&checkpoint, lease, &mut first).unwrap();
    let mut second = TestSigner::new(2, 1);
    assert!(matches!(
        attestor.attest(&checkpoint, lease, &mut second),
        Err(EffectAttestationError::KeyChangedWithoutEpochAdvance { .. })
    ));
}

#[test]
fn algorithm_change_requires_trust_epoch_advance() {
    let (store, journal, _loop, checkpoint) = audited_checkpoint(17, "ok");
    let lease = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut first = TestSigner::new(1, 1);
    attestor.attest(&checkpoint, lease, &mut first).unwrap();
    let mut second = TestSigner::new(1, 2);
    assert!(matches!(
        attestor.attest(&checkpoint, lease, &mut second),
        Err(EffectAttestationError::AlgorithmChangedWithoutEpochAdvance { .. })
    ));
}

#[test]
fn trust_epoch_cannot_move_backward() {
    let (store, journal, _loop, checkpoint) = audited_checkpoint(18, "ok");
    let lease = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let shared = anchor.clone();
    let mut newer = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(2)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    newer.attest(&checkpoint, lease, &mut signer).unwrap();
    let mut older = GovernedEffectAttestor::new(shared, EffectTrustEpoch::new(1)).unwrap();
    let mut signer2 = TestSigner::new(1, 1);
    assert!(matches!(
        older.attest(&checkpoint, lease, &mut signer2),
        Err(EffectAttestationError::TrustEpochRollback { .. })
    ));
}

#[test]
fn trust_epoch_advance_allows_key_and_algorithm_rotation() {
    let (store, journal, _loop, checkpoint) = audited_checkpoint(19, "ok");
    let lease = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let shared = anchor.clone();
    let mut first_attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut first = TestSigner::new(1, 1);
    first_attestor
        .attest(&checkpoint, lease, &mut first)
        .unwrap();
    let mut rotated = GovernedEffectAttestor::new(shared, EffectTrustEpoch::new(2)).unwrap();
    let mut second = TestSigner::new(2, 9);
    let (attestation, _) = rotated.attest(&checkpoint, lease, &mut second).unwrap();
    assert_eq!(attestation.statement.key_id, key(2));
    assert_eq!(
        attestation.statement.algorithm_id,
        EffectAttestationAlgorithmId::new(9)
    );
}

#[test]
fn audit_height_cannot_move_backward() {
    let store = JournalStore::default();
    let mut journal = active_journal(20, 1, store.clone());
    let mut loop_ = boot();
    stage_one(&mut loop_, &mut journal);
    let old_checkpoint = loop_.current_effect_audit_checkpoint(&journal);
    let mut backend = SuccessBackend {
        reference: "delivered",
    };
    loop_
        .dispatch_next_effect_with_audited_journal(
            &mut journal,
            EffectRetryTick(1),
            &dispatcher(),
            &mut backend,
        )
        .unwrap();
    let new_checkpoint = loop_.current_effect_audit_checkpoint(&journal);
    let lease = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    attestor
        .attest(&new_checkpoint, lease, &mut signer)
        .unwrap();
    assert!(matches!(
        attestor.attest(&old_checkpoint, lease, &mut signer),
        Err(EffectAttestationError::AuditRollback { .. })
    ));
}

#[test]
fn divergent_audit_root_at_same_height_is_rejected() {
    let (store_a, journal_a, _loop_a, checkpoint_a) = audited_checkpoint(21, "receipt-a");
    let (_store_b, _journal_b, _loop_b, checkpoint_b) = audited_checkpoint(21, "receipt-b");
    assert_eq!(checkpoint_a.audit().len(), checkpoint_b.audit().len());
    assert_ne!(
        checkpoint_a.audit().root_hash(),
        checkpoint_b.audit().root_hash()
    );
    let lease = journal_a.lease().unwrap();
    let anchor = AnchorStore::new(&store_a);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    attestor.attest(&checkpoint_a, lease, &mut signer).unwrap();
    assert!(matches!(
        attestor.attest(&checkpoint_b, lease, &mut signer),
        Err(EffectAttestationError::AuditForkAtSameHeight)
    ));
}

#[test]
fn checkpoint_mismatch_is_rejected_before_signature_verification() {
    let (store_a, journal_a, _loop_a, checkpoint_a) = audited_checkpoint(22, "receipt-a");
    let (_store_b, _journal_b, _loop_b, checkpoint_b) = audited_checkpoint(22, "receipt-b");
    let lease = journal_a.lease().unwrap();
    let anchor = AnchorStore::new(&store_a);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    attestor.attest(&checkpoint_a, lease, &mut signer).unwrap();
    let verifier = TestVerifier::accepting(1, 1, 1);
    assert!(matches!(
        attestor.verify_latest(&checkpoint_b, &verifier),
        Err(EffectAttestationError::CheckpointMismatch)
    ));
}

#[test]
fn takeover_may_change_writer_and_fence_without_changing_trust_epoch() {
    let (store, mut journal, _loop, checkpoint) = audited_checkpoint(23, "ok");
    let lease1 = journal.lease().unwrap();
    let anchor = AnchorStore::new(&store);
    let shared = anchor.clone();
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    attestor.attest(&checkpoint, lease1, &mut signer).unwrap();
    journal.release().unwrap();
    let mut takeover = active_journal(23, 2, store);
    let lease2 = takeover.lease().unwrap();
    assert!(lease2.fence.0 > lease1.fence.0);
    let mut attestor2 = GovernedEffectAttestor::new(shared, EffectTrustEpoch::new(1)).unwrap();
    let (signed, _) = attestor2.attest(&checkpoint, lease2, &mut signer).unwrap();
    assert_eq!(signed.statement.writer, writer(2));
    assert_eq!(signed.statement.fence, lease2.fence);
    takeover.release().unwrap();
}

#[test]
fn stale_lease_is_rejected_by_anchor_store() {
    let (store, mut journal, _loop, checkpoint) = audited_checkpoint(24, "ok");
    let stale = journal.lease().unwrap();
    journal.release().unwrap();
    let mut takeover = active_journal(24, 2, store.clone());
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    assert!(matches!(
        attestor.attest(&checkpoint, stale, &mut signer),
        Err(EffectAttestationError::Store(_))
    ));
    takeover.release().unwrap();
}

#[test]
fn attestation_does_not_change_event_loop_replay_identity() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(25, "ok");
    let before = loop_.replay_key();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    assert_eq!(loop_.replay_key(), before);
}

#[test]
fn journal_active_lease_is_shared_with_attestation_commit() {
    let (store, mut journal, loop_, _checkpoint) = audited_checkpoint(26, "ok");
    let expected = store.active().unwrap();
    let anchor = AnchorStore::new(&store);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    let (attestation, _) = loop_
        .attest_current_effect_audit(&mut journal, &mut attestor, &mut signer)
        .unwrap();
    assert_eq!(attestation.statement.writer, expected.writer);
    assert_eq!(attestation.statement.fence, expected.fence);
}

#[test]
fn longer_divergent_history_is_rejected_as_non_descendant() {
    let (store_a, journal_a, _loop_a, checkpoint_a) = audited_checkpoint(27, "receipt-a");

    let store_b = JournalStore::default();
    let mut journal_b = active_journal(27, 1, store_b);
    let mut loop_b = boot();
    stage_one(&mut loop_b, &mut journal_b);
    let mut first_backend = SuccessBackend {
        reference: "receipt-b",
    };
    loop_b
        .dispatch_next_effect_with_audited_journal(
            &mut journal_b,
            EffectRetryTick(1),
            &dispatcher(),
            &mut first_backend,
        )
        .unwrap();
    loop_b
        .cycle_to_with_audited_effect_journal(LogicalTime(1), &keyboard_batch(7, 2), &mut journal_b)
        .unwrap();
    let mut second_backend = SuccessBackend {
        reference: "receipt-c",
    };
    loop_b
        .dispatch_next_effect_with_audited_journal(
            &mut journal_b,
            EffectRetryTick(2),
            &dispatcher(),
            &mut second_backend,
        )
        .unwrap();
    let checkpoint_b = loop_b.current_effect_audit_checkpoint(&journal_b);
    assert_eq!(checkpoint_a.audit().len(), 2);
    assert_eq!(checkpoint_b.audit().len(), 4);

    let lease = journal_a.lease().unwrap();
    let anchor = AnchorStore::new(&store_a);
    let mut attestor = GovernedEffectAttestor::new(anchor, EffectTrustEpoch::new(1)).unwrap();
    let mut signer = TestSigner::new(1, 1);
    attestor.attest(&checkpoint_a, lease, &mut signer).unwrap();
    assert!(matches!(
        attestor.attest(&checkpoint_b, lease, &mut signer),
        Err(EffectAttestationError::AuditHistoryNotDescendant)
    ));
}
