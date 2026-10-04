use std::fmt::{Display, Formatter};

use crate::{
    effect::Effect,
    effect_attestation::{
        EffectAttestationCommitReceipt, EffectAttestationResult, EffectAttestationSigner,
        EffectAttestationStore, EffectAttestationVerifier, EffectAuditAttestation,
        GovernedEffectAttestor,
    },
    effect_audit::{
        hash::sha256, EffectAuditCheckpoint, EffectAuditDispatchOutcome, EffectAuditLedger,
        EffectAuditResult, GovernedAuditedEffectJournal,
    },
    effect_completion::{
        AtomicEffectCompletionCore, EffectCompletionBatch, EffectCompletionBatchReport,
        EffectCompletionProjection, EffectCompletionResult, EffectCompletionSequence,
        EffectCompletionSourceId,
    },
    effect_dispatch::{
        AtomicEffectOutbox, EffectBackend, EffectDeliveryKey, EffectDeliveryNamespace,
        EffectDispatchReceipt, EffectDispatchResult, EffectIntentId, EffectOutboxStageReport,
        GovernedEffectDispatcher, QueuedEffectIntent,
    },
    effect_fencing::{
        EffectFencingResult, EffectJournalLease, FencedEffectJournalStore,
        GovernedFencedEffectJournal,
    },
    effect_persistence::{
        EffectJournalStore, EffectOutboxCheckpoint, EffectPersistenceResult, GovernedEffectJournal,
    },
    effect_retry::{
        DeadLetteredEffect, EffectRetryDispatchOutcome, EffectRetryResult, EffectRetryTick,
        GovernedRetryEffectJournal,
    },
    input::InputBatch,
    nair::{
        bootstrap_native_completions, bootstrap_native_reactions, CompletionSlot, Instruction,
        NairCompletionAuthority, NairCompletionBinding, NairError, NairProgram,
        NairReactionAuthority, NairReactionCycleReport, ReactionSlot, TimerSlot,
    },
    program_upgrade::{
        next_lineage_root, ProgramEpoch, RuntimeTimerAwareUpgradePlan, RuntimeTimerUpgradePlan,
        RuntimeUpgradeAuthority, RuntimeUpgradeError, RuntimeUpgradeHash,
        RuntimeUpgradeLineageRecord, RuntimeUpgradePlan, RuntimeUpgradeReport,
    },
    reaction::{AtomicReactionCore, ReactionId},
    runtime::{
        hash_bytes, hash_component, PersistentAtomicRuntime, PersistentRuntimeTickReport,
        RuntimeAtomSnapshot, RuntimeError, FNV_OFFSET_BASIS,
    },
    runtime_checkpoint::{
        FencedRuntimeCheckpointStore, RuntimeCheckpointCommitReceipt, RuntimeCheckpointError,
        RuntimeCheckpointRecoveryReport, RuntimeCheckpointResult, RuntimeSemanticCheckpoint,
        TimeCheckpointState,
    },
    AtomSlot,
};

use std::collections::{BTreeMap, BTreeSet};

use super::{
    AtomicTimeCore, EventLoopResult, LogicalDuration, LogicalTime, TimeAdvanceReport, TimeError,
    TimerId, TimerSnapshot, DEFAULT_TIMER_FIRE_BUDGET,
};

const EVENT_LOOP_REPLAY_DOMAIN: &[u8] = b"NORDOI-ATOMIC-EVENT-LOOP-1.16";
const RUNTIME_UPGRADE_REPLAY_DOMAIN: &[u8] = b"NORDOI-RUNTIME-PROGRAM-UPGRADE-1.1";
const RUNTIME_PROGRAM_HASH_DOMAIN: &[u8] = b"NORDOI-RUNTIME-PROGRAM-1.0";
const OP_SCHEDULE_ONCE: u8 = 0x01;
const OP_SCHEDULE_REPEATING: u8 = 0x02;
const OP_CANCEL: u8 = 0x03;
const OP_CYCLE: u8 = 0x04;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventLoopReplayKey(pub u64);

impl EventLoopReplayKey {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl Display for EventLoopReplayKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventLoopCycleReport {
    pub cycle: u64,
    pub logical_time: LogicalTime,
    pub time: TimeAdvanceReport,
    pub runtime: PersistentRuntimeTickReport,
    pub reactions: NairReactionCycleReport,
    pub effects: EffectOutboxStageReport,
    pub completions: EffectCompletionBatchReport,
    pub replay_key: EventLoopReplayKey,
}

#[derive(Clone, Copy)]
enum RuntimeUpgradePlanRef<'a> {
    AtomOnly(&'a RuntimeUpgradePlan),
    TimerAware(&'a RuntimeTimerAwareUpgradePlan),
}

impl<'a> RuntimeUpgradePlanRef<'a> {
    fn atom_plan(self) -> &'a RuntimeUpgradePlan {
        match self {
            Self::AtomOnly(plan) => plan,
            Self::TimerAware(plan) => plan.atom_plan(),
        }
    }

    fn timer_plan(self) -> Option<&'a RuntimeTimerUpgradePlan> {
        match self {
            Self::AtomOnly(_) => None,
            Self::TimerAware(plan) => Some(plan.timer_plan()),
        }
    }

    fn semantic_plan_hash(self) -> RuntimeUpgradeHash {
        match self {
            Self::AtomOnly(plan) => plan.plan_hash(),
            Self::TimerAware(plan) => plan.composite_plan_hash(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomicEventLoop {
    runtime: PersistentAtomicRuntime,
    time: AtomicTimeCore,
    cycle: u64,
    replay_state: u64,
    replay_key: EventLoopReplayKey,
    program_hash: [u8; 32],
    program_epoch: ProgramEpoch,
    upgrade_chain_root: RuntimeUpgradeHash,
    last_upgrade: Option<RuntimeUpgradeLineageRecord>,
    runtime_checkpoint_bound: bool,
    native_timer_bindings: BTreeMap<TimerSlot, TimerId>,
    reactions: AtomicReactionCore,
    native_reaction_bindings: BTreeMap<ReactionSlot, ReactionId>,
    native_completion_bindings: BTreeMap<CompletionSlot, NairCompletionBinding>,
    effect_outbox: AtomicEffectOutbox,
    effect_completions: AtomicEffectCompletionCore,
}

impl AtomicEventLoop {
    pub fn boot(program: &NairProgram) -> EventLoopResult<Self> {
        let reaction_authority = NairReactionAuthority::new();
        let completion_authority = NairCompletionAuthority::new();
        Self::boot_with_fire_budget_and_authorities(
            program,
            DEFAULT_TIMER_FIRE_BUDGET,
            &reaction_authority,
            &completion_authority,
        )
    }

    pub fn boot_with_fire_budget(
        program: &NairProgram,
        fire_budget: usize,
    ) -> EventLoopResult<Self> {
        let reaction_authority = NairReactionAuthority::new();
        let completion_authority = NairCompletionAuthority::new();
        Self::boot_with_fire_budget_and_authorities(
            program,
            fire_budget,
            &reaction_authority,
            &completion_authority,
        )
    }

    pub fn boot_with_reaction_authority(
        program: &NairProgram,
        authority: &NairReactionAuthority,
    ) -> EventLoopResult<Self> {
        let completion_authority = NairCompletionAuthority::new();
        Self::boot_with_fire_budget_and_authorities(
            program,
            DEFAULT_TIMER_FIRE_BUDGET,
            authority,
            &completion_authority,
        )
    }

    pub fn boot_with_fire_budget_and_reaction_authority(
        program: &NairProgram,
        fire_budget: usize,
        authority: &NairReactionAuthority,
    ) -> EventLoopResult<Self> {
        let completion_authority = NairCompletionAuthority::new();
        Self::boot_with_fire_budget_and_authorities(
            program,
            fire_budget,
            authority,
            &completion_authority,
        )
    }

    pub fn boot_with_authorities(
        program: &NairProgram,
        reaction_authority: &NairReactionAuthority,
        completion_authority: &NairCompletionAuthority,
    ) -> EventLoopResult<Self> {
        Self::boot_with_fire_budget_and_authorities(
            program,
            DEFAULT_TIMER_FIRE_BUDGET,
            reaction_authority,
            completion_authority,
        )
    }

    pub fn boot_with_fire_budget_and_authorities(
        program: &NairProgram,
        fire_budget: usize,
        reaction_authority: &NairReactionAuthority,
        completion_authority: &NairCompletionAuthority,
    ) -> EventLoopResult<Self> {
        program.validate().map_err(RuntimeError::from)?;
        let program_bytes = program.canonical_bytes().map_err(RuntimeError::from)?;
        let mut program_hash_input =
            Vec::with_capacity(RUNTIME_PROGRAM_HASH_DOMAIN.len() + program_bytes.len());
        program_hash_input.extend_from_slice(RUNTIME_PROGRAM_HASH_DOMAIN);
        program_hash_input.extend_from_slice(&program_bytes);
        let program_hash = sha256(&program_hash_input);
        let mut time = AtomicTimeCore::with_fire_budget(fire_budget)?;
        let native_timer_bindings = apply_native_time_bootstrap(program, &mut time)?;
        let runtime_program = NairProgram::from_instructions(
            program
                .instructions()
                .iter()
                .filter(|instruction| {
                    !instruction.requires_time_context()
                        && !instruction.requires_reaction_context()
                        && !instruction.requires_completion_context()
                })
                .cloned()
                .collect(),
        );
        let runtime = PersistentAtomicRuntime::boot(&runtime_program)?;
        let (reactions, native_reaction_bindings) = bootstrap_native_reactions(
            program,
            runtime.kernel(),
            &runtime.boot_report().execution.domain_bindings,
            &runtime.boot_report().execution.atom_bindings,
            &runtime.boot_report().render_bindings,
            &native_timer_bindings,
            reaction_authority,
        )
        .map_err(RuntimeError::from)?;
        let (effect_completions, native_completion_bindings) = bootstrap_native_completions(
            program,
            runtime.kernel(),
            &runtime.boot_report().execution.domain_bindings,
            &runtime.boot_report().execution.atom_bindings,
            completion_authority,
        )
        .map_err(RuntimeError::from)?;

        let mut replay_state = FNV_OFFSET_BASIS;
        hash_bytes(&mut replay_state, EVENT_LOOP_REPLAY_DOMAIN);
        hash_component(&mut replay_state, &program_bytes);
        hash_component(
            &mut replay_state,
            &runtime.replay_key().value().to_le_bytes(),
        );
        hash_component(&mut replay_state, &(fire_budget as u64).to_le_bytes());
        let replay_key = EventLoopReplayKey(replay_state);

        Ok(Self {
            runtime,
            time,
            cycle: 0,
            replay_state,
            replay_key,
            program_hash,
            program_epoch: ProgramEpoch(0),
            upgrade_chain_root: RuntimeUpgradeHash::ZERO,
            last_upgrade: None,
            runtime_checkpoint_bound: false,
            native_timer_bindings,
            reactions,
            native_reaction_bindings,
            native_completion_bindings,
            effect_outbox: AtomicEffectOutbox::new(),
            effect_completions,
        })
    }

    pub fn logical_time(&self) -> LogicalTime {
        self.time.now()
    }

    pub fn cycle_index(&self) -> u64 {
        self.cycle
    }

    pub fn replay_key(&self) -> EventLoopReplayKey {
        self.replay_key
    }

    pub fn program_hash(&self) -> [u8; 32] {
        self.program_hash
    }

    pub fn program_epoch(&self) -> ProgramEpoch {
        self.program_epoch
    }

    pub fn upgrade_chain_root(&self) -> RuntimeUpgradeHash {
        self.upgrade_chain_root
    }

    pub fn last_upgrade(&self) -> Option<&RuntimeUpgradeLineageRecord> {
        self.last_upgrade.as_ref()
    }

    pub fn pending_timers(&self) -> usize {
        self.time.pending_timers()
    }

    pub fn native_timer_count(&self) -> usize {
        self.native_timer_bindings.len()
    }

    pub fn native_timer_id(&self, slot: TimerSlot) -> Option<TimerId> {
        self.native_timer_bindings.get(&slot).copied()
    }

    pub fn native_reaction_count(&self) -> usize {
        self.native_reaction_bindings.len()
    }

    pub fn native_reaction_id(&self, slot: ReactionSlot) -> Option<ReactionId> {
        self.native_reaction_bindings.get(&slot).copied()
    }

    pub fn native_completion_count(&self) -> usize {
        self.native_completion_bindings.len()
    }

    pub fn native_completion_binding(&self, slot: CompletionSlot) -> Option<NairCompletionBinding> {
        self.native_completion_bindings.get(&slot).copied()
    }

    pub fn pending_effect_count(&self) -> usize {
        self.effect_outbox.pending_len()
    }

    pub fn pending_effect(&self, id: EffectIntentId) -> Option<&QueuedEffectIntent> {
        self.effect_outbox.get(id)
    }

    pub fn pending_effects(&self) -> impl Iterator<Item = &QueuedEffectIntent> {
        self.effect_outbox.iter()
    }

    pub fn grant_effect_completion_source(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) -> EffectCompletionResult<bool> {
        self.effect_completions
            .authority_mut()
            .grant(source, namespace)
    }

    pub fn revoke_effect_completion_source(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
    ) -> bool {
        self.effect_completions
            .authority_mut()
            .revoke(source, namespace)
    }

    pub fn register_effect_completion_projection(
        &mut self,
        projection: EffectCompletionProjection,
    ) -> EffectCompletionResult<()> {
        self.effect_completions
            .register_projection(self.runtime.kernel(), projection)
    }

    pub fn unregister_effect_completion_projection(
        &mut self,
        source: EffectCompletionSourceId,
        namespace: EffectDeliveryNamespace,
        atom: crate::AtomId,
    ) -> bool {
        self.effect_completions
            .unregister_projection(source, namespace, atom)
    }

    pub fn effect_completion_last_sequence(
        &self,
        source: EffectCompletionSourceId,
    ) -> Option<EffectCompletionSequence> {
        self.effect_completions.last_sequence(source)
    }

    pub fn has_effect_completion(&self, key: EffectDeliveryKey) -> bool {
        self.effect_completions.has_completed(key)
    }

    pub fn dispatch_next_effect<B: EffectBackend>(
        &mut self,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectDispatchResult<Option<EffectDispatchReceipt>> {
        dispatcher.dispatch_next(&mut self.effect_outbox, backend)
    }

    pub fn effect_checkpoint(&self, namespace: EffectDeliveryNamespace) -> EffectOutboxCheckpoint {
        EffectOutboxCheckpoint::capture(namespace, &self.effect_outbox)
    }

    pub fn restore_effect_checkpoint(
        &mut self,
        checkpoint: &EffectOutboxCheckpoint,
    ) -> EffectPersistenceResult<()> {
        if self.cycle != 0 {
            return Err(
                crate::effect_persistence::EffectPersistenceError::RecoveryAfterCycleStarted {
                    cycle: self.cycle,
                },
            );
        }
        self.effect_outbox = checkpoint.to_outbox();
        hash_bytes(
            &mut self.replay_state,
            b"NORDOI-EFFECT-JOURNAL-RECOVERY-1.0",
        );
        hash_component(
            &mut self.replay_state,
            &checkpoint.next_intent_id().to_le_bytes(),
        );
        hash_component(
            &mut self.replay_state,
            &(checkpoint.pending().len() as u64).to_le_bytes(),
        );
        for queued in checkpoint.pending() {
            hash_component(&mut self.replay_state, &queued.id.0.to_le_bytes());
            hash_component(&mut self.replay_state, &queued.cycle.to_le_bytes());
            hash_component(&mut self.replay_state, &queued.ordinal.to_le_bytes());
            hash_component(
                &mut self.replay_state,
                &queued.intent.reaction.0.to_le_bytes(),
            );
            hash_component(&mut self.replay_state, queued.intent.action_name.as_bytes());
            hash_effect(&mut self.replay_state, &queued.intent.effect);
        }
        self.replay_key = EventLoopReplayKey(self.replay_state);
        Ok(())
    }

    pub fn recover_effects_from_journal<S: EffectJournalStore>(
        &mut self,
        journal: &mut GovernedEffectJournal<S>,
    ) -> EffectPersistenceResult<bool> {
        let Some(checkpoint) = journal.recover()? else {
            return Ok(false);
        };
        self.restore_effect_checkpoint(&checkpoint)?;
        Ok(true)
    }

    pub fn recover_effects_from_fenced_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedFencedEffectJournal<S>,
    ) -> EffectFencingResult<bool> {
        let Some(checkpoint) = journal.recover()? else {
            return Ok(false);
        };
        self.restore_effect_checkpoint(&checkpoint)
            .map_err(crate::effect_fencing::EffectFencingError::from)?;
        Ok(true)
    }

    pub fn dispatch_next_effect_with_journal<S: EffectJournalStore, B: EffectBackend>(
        &mut self,
        journal: &mut GovernedEffectJournal<S>,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectPersistenceResult<Option<EffectDispatchReceipt>> {
        journal.dispatch_next(&mut self.effect_outbox, dispatcher, backend)
    }

    pub fn dispatch_next_effect_with_fenced_journal<
        S: FencedEffectJournalStore,
        B: EffectBackend,
    >(
        &mut self,
        journal: &mut GovernedFencedEffectJournal<S>,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectFencingResult<Option<EffectDispatchReceipt>> {
        journal.dispatch_next(&mut self.effect_outbox, dispatcher, backend)
    }

    pub fn recover_effects_from_retry_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedRetryEffectJournal<S>,
    ) -> EffectRetryResult<bool> {
        let Some(checkpoint) = journal.recover()? else {
            return Ok(false);
        };
        self.restore_effect_checkpoint(checkpoint.outbox())
            .map_err(crate::effect_retry::EffectRetryError::from)?;
        journal.adopt_recovered_ledger(checkpoint.ledger().clone());
        Ok(true)
    }

    pub fn dispatch_next_effect_with_retry_journal<
        S: FencedEffectJournalStore,
        B: EffectBackend,
    >(
        &mut self,
        journal: &mut GovernedRetryEffectJournal<S>,
        current_tick: EffectRetryTick,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectRetryResult<Option<EffectRetryDispatchOutcome>> {
        journal.dispatch_next(&mut self.effect_outbox, current_tick, dispatcher, backend)
    }

    pub fn redrive_dead_letter_with_retry_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedRetryEffectJournal<S>,
        id: EffectIntentId,
    ) -> EffectRetryResult<QueuedEffectIntent> {
        journal.redrive_dead_letter(&mut self.effect_outbox, id)
    }

    pub fn discard_dead_letter_with_retry_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedRetryEffectJournal<S>,
        id: EffectIntentId,
    ) -> EffectRetryResult<DeadLetteredEffect> {
        journal.discard_dead_letter(&self.effect_outbox, id)
    }

    pub fn recover_effects_from_audited_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EffectAuditResult<bool> {
        let Some(checkpoint) = journal.recover()? else {
            return Ok(false);
        };
        self.restore_effect_checkpoint(checkpoint.retry().outbox())
            .map_err(crate::effect_audit::EffectAuditError::from)?;
        journal.adopt_recovered_state(
            checkpoint.retry().ledger().clone(),
            checkpoint.audit().clone(),
        );
        Ok(true)
    }

    pub fn current_effect_audit_checkpoint<S>(
        &self,
        journal: &GovernedAuditedEffectJournal<S>,
    ) -> EffectAuditCheckpoint {
        journal.capture_checkpoint(&self.effect_outbox)
    }

    pub fn attest_current_effect_audit<
        J: FencedEffectJournalStore,
        A: EffectAttestationStore,
        SIGN: EffectAttestationSigner,
    >(
        &self,
        journal: &mut GovernedAuditedEffectJournal<J>,
        attestor: &mut GovernedEffectAttestor<A>,
        signer: &mut SIGN,
    ) -> EffectAttestationResult<(EffectAuditAttestation, EffectAttestationCommitReceipt)> {
        attestor.attest_current(journal, &self.effect_outbox, signer)
    }

    pub fn verify_current_effect_audit_attestation<
        J: FencedEffectJournalStore,
        A: EffectAttestationStore,
        V: EffectAttestationVerifier,
    >(
        &self,
        journal: &mut GovernedAuditedEffectJournal<J>,
        attestor: &mut GovernedEffectAttestor<A>,
        verifier: &V,
    ) -> EffectAttestationResult<Option<EffectAuditAttestation>> {
        attestor.verify_current(journal, &self.effect_outbox, verifier)
    }

    pub fn dispatch_next_effect_with_audited_journal<
        S: FencedEffectJournalStore,
        B: EffectBackend,
    >(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
        current_tick: EffectRetryTick,
        dispatcher: &GovernedEffectDispatcher,
        backend: &mut B,
    ) -> EffectAuditResult<Option<EffectAuditDispatchOutcome>> {
        journal.dispatch_next(&mut self.effect_outbox, current_tick, dispatcher, backend)
    }

    pub fn resolve_in_doubt_effect_as_delivered<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
        resolution_tick: EffectRetryTick,
        backend_reference: Option<String>,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        journal.resolve_in_doubt_as_delivered(
            &mut self.effect_outbox,
            resolution_tick,
            backend_reference,
        )
    }

    pub fn authorize_in_doubt_effect_retry<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
        resolution_tick: EffectRetryTick,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        journal.authorize_in_doubt_retry(&self.effect_outbox, resolution_tick)
    }

    pub fn redrive_dead_letter_with_audited_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
        id: EffectIntentId,
    ) -> EffectAuditResult<QueuedEffectIntent> {
        journal.redrive_dead_letter(&mut self.effect_outbox, id)
    }

    pub fn discard_dead_letter_with_audited_journal<S: FencedEffectJournalStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
        id: EffectIntentId,
    ) -> EffectAuditResult<DeadLetteredEffect> {
        journal.discard_dead_letter(&self.effect_outbox, id)
    }

    pub fn timer_snapshot(&self, id: TimerId) -> EventLoopResult<TimerSnapshot> {
        self.time.timer(id).map_err(Into::into)
    }

    pub fn next_deadline(&self) -> Option<LogicalTime> {
        self.time.next_deadline()
    }

    pub fn is_runtime_quiescent(&self) -> bool {
        self.runtime.is_quiescent()
    }

    pub fn snapshot(&self) -> EventLoopResult<BTreeMap<AtomSlot, RuntimeAtomSnapshot>> {
        self.runtime.snapshot().map_err(Into::into)
    }

    pub fn semantic_checkpoint<S>(
        &self,
        journal: &GovernedAuditedEffectJournal<S>,
    ) -> RuntimeCheckpointResult<RuntimeSemanticCheckpoint> {
        let audit_events = u64::try_from(journal.audit_ledger().len()).map_err(|_| {
            RuntimeCheckpointError::InternalState("audit length does not fit u64".into())
        })?;
        RuntimeSemanticCheckpoint::from_parts(
            journal.namespace(),
            self.program_hash,
            self.program_epoch,
            self.upgrade_chain_root,
            self.last_upgrade.clone(),
            audit_events,
            journal.audit_ledger().root_hash(),
            self.effect_outbox.next_intent_id(),
            self.cycle,
            self.replay_state,
            self.runtime.capture_checkpoint_state()?,
            self.time.capture_checkpoint_state(),
            self.effect_completions.capture_checkpoint_state(),
        )
    }

    pub fn checkpoint_runtime_with_audited_journal<S: FencedRuntimeCheckpointStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<RuntimeCheckpointCommitReceipt> {
        let lease = self.runtime_checkpoint_commit_lease(journal)?;
        let effect_checkpoint = journal.capture_checkpoint(&self.effect_outbox);
        let runtime_checkpoint = self.semantic_checkpoint(journal)?;
        let effect_bytes = effect_checkpoint.canonical_bytes();
        let runtime_bytes = runtime_checkpoint.canonical_bytes()?;
        let receipt = journal
            .store_mut()
            .commit_effect_and_runtime_fenced(lease, &effect_bytes, &runtime_bytes)
            .map_err(RuntimeCheckpointError::from)?;
        self.runtime_checkpoint_bound = true;
        Ok(receipt)
    }

    fn runtime_checkpoint_commit_lease<S: FencedRuntimeCheckpointStore>(
        &self,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EffectJournalLease> {
        let lease = journal.assert_active()?;
        if self.runtime_checkpoint_bound {
            return Ok(lease);
        }
        let effect_bytes = journal
            .store_mut()
            .load_fenced(lease)
            .map_err(crate::effect_fencing::EffectFencingError::from)?;
        let runtime_bytes = journal
            .store_mut()
            .load_runtime_fenced(lease)
            .map_err(RuntimeCheckpointError::from)?;
        match (effect_bytes, runtime_bytes) {
            (None, None) => Ok(lease),
            (Some(_), Some(_)) => Err(RuntimeCheckpointError::RecoveryRequired.into()),
            _ => Err(RuntimeCheckpointError::RecoveryBundleIncomplete.into()),
        }
    }

    pub fn recover_runtime_from_audited_journal<S: FencedRuntimeCheckpointStore>(
        &mut self,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<Option<RuntimeCheckpointRecoveryReport>> {
        if self.cycle != 0 {
            return Err(
                RuntimeCheckpointError::RecoveryAfterCycleStarted { cycle: self.cycle }.into(),
            );
        }
        let lease = journal.assert_active()?;
        let effect_bytes = journal
            .store_mut()
            .load_fenced(lease)
            .map_err(crate::effect_fencing::EffectFencingError::from)?;
        let runtime_bytes = journal
            .store_mut()
            .load_runtime_fenced(lease)
            .map_err(RuntimeCheckpointError::from)?;
        let (effect_bytes, runtime_bytes) = match (effect_bytes, runtime_bytes) {
            (None, None) => {
                self.runtime_checkpoint_bound = true;
                return Ok(None);
            }
            (Some(effect_bytes), Some(runtime_bytes)) => (effect_bytes, runtime_bytes),
            _ => return Err(RuntimeCheckpointError::RecoveryBundleIncomplete.into()),
        };

        let effect_checkpoint = EffectAuditCheckpoint::from_canonical_bytes(&effect_bytes)?;
        if effect_checkpoint.namespace() != journal.namespace() {
            return Err(RuntimeCheckpointError::NamespaceMismatch {
                expected: journal.namespace(),
                actual: effect_checkpoint.namespace(),
            }
            .into());
        }
        if let Some(policy) = effect_checkpoint.retry().policy() {
            if policy != journal.policy() {
                return Err(crate::effect_audit::EffectAuditError::AuditPolicyMismatch.into());
            }
        }
        let checkpoint = RuntimeSemanticCheckpoint::from_canonical_bytes(&runtime_bytes)?;
        self.validate_runtime_recovery_checkpoint(journal, &effect_checkpoint, &checkpoint)?;

        let mut candidate = self.clone();
        candidate
            .runtime
            .restore_checkpoint_state(checkpoint.runtime_state())?;
        candidate
            .time
            .restore_checkpoint_state(checkpoint.time_state())?;
        candidate
            .effect_completions
            .restore_checkpoint_state(checkpoint.completion_state());
        candidate.effect_outbox = effect_checkpoint.retry().outbox().to_outbox();
        candidate.cycle = checkpoint.cycle();
        candidate.replay_state = checkpoint.event_replay_state();
        candidate.replay_key = EventLoopReplayKey(checkpoint.event_replay_state());
        candidate.program_epoch = checkpoint.program_epoch();
        candidate.upgrade_chain_root = checkpoint.upgrade_chain_root();
        candidate.last_upgrade = checkpoint.last_upgrade().cloned();
        candidate.runtime_checkpoint_bound = true;

        let audit_events = u64::try_from(effect_checkpoint.audit().len()).map_err(|_| {
            RuntimeCheckpointError::InternalState("audit length does not fit u64".into())
        })?;
        let report = RuntimeCheckpointRecoveryReport {
            cycle: checkpoint.cycle(),
            runtime_tick: checkpoint.runtime_tick(),
            logical_time: checkpoint.logical_time(),
            audit_events,
        };
        journal.adopt_recovered_state(
            effect_checkpoint.retry().ledger().clone(),
            effect_checkpoint.audit().clone(),
        );
        *self = candidate;
        Ok(Some(report))
    }

    fn validate_runtime_recovery_checkpoint<S>(
        &self,
        journal: &GovernedAuditedEffectJournal<S>,
        effect_checkpoint: &EffectAuditCheckpoint,
        checkpoint: &RuntimeSemanticCheckpoint,
    ) -> RuntimeCheckpointResult<()> {
        if checkpoint.namespace() != journal.namespace() {
            return Err(RuntimeCheckpointError::NamespaceMismatch {
                expected: journal.namespace(),
                actual: checkpoint.namespace(),
            });
        }
        if checkpoint.program_hash() != self.program_hash {
            return Err(RuntimeCheckpointError::ProgramMismatch);
        }
        let actual_next = effect_checkpoint.retry().outbox().next_intent_id();
        if actual_next != checkpoint.effect_next_intent_id() {
            return Err(RuntimeCheckpointError::EffectIntentSequenceMismatch {
                expected_next: checkpoint.effect_next_intent_id(),
                actual_next,
            });
        }
        let required = checkpoint.audit_events();
        let actual = u64::try_from(effect_checkpoint.audit().len()).map_err(|_| {
            RuntimeCheckpointError::InternalState("audit length does not fit u64".into())
        })?;
        if actual < required {
            return Err(RuntimeCheckpointError::AuditHistoryTooShort { required, actual });
        }
        let actual_root = if required == 0 {
            crate::effect_audit::EffectAuditHash::ZERO
        } else {
            let index = usize::try_from(required - 1)
                .map_err(|_| RuntimeCheckpointError::InvalidAuditPrefix)?;
            effect_checkpoint
                .audit()
                .records()
                .get(index)
                .ok_or(RuntimeCheckpointError::InvalidAuditPrefix)?
                .hash
        };
        if actual_root != checkpoint.audit_root() {
            return Err(RuntimeCheckpointError::AuditPrefixMismatch {
                expected: checkpoint.audit_root(),
                actual: actual_root,
            });
        }
        Ok(())
    }

    pub fn upgrade_program_with_runtime_checkpoint<S: FencedRuntimeCheckpointStore>(
        &mut self,
        target_program: &NairProgram,
        reaction_authority: &NairReactionAuthority,
        completion_authority: &NairCompletionAuthority,
        upgrade_authority: &RuntimeUpgradeAuthority,
        plan: &RuntimeUpgradePlan,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<RuntimeUpgradeReport> {
        self.upgrade_program_with_runtime_checkpoint_internal(
            target_program,
            reaction_authority,
            completion_authority,
            upgrade_authority,
            RuntimeUpgradePlanRef::AtomOnly(plan),
            journal,
        )
    }

    pub fn upgrade_program_with_runtime_checkpoint_and_timers<S: FencedRuntimeCheckpointStore>(
        &mut self,
        target_program: &NairProgram,
        reaction_authority: &NairReactionAuthority,
        completion_authority: &NairCompletionAuthority,
        upgrade_authority: &RuntimeUpgradeAuthority,
        migration: &RuntimeTimerAwareUpgradePlan,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<RuntimeUpgradeReport> {
        self.upgrade_program_with_runtime_checkpoint_internal(
            target_program,
            reaction_authority,
            completion_authority,
            upgrade_authority,
            RuntimeUpgradePlanRef::TimerAware(migration),
            journal,
        )
    }

    fn upgrade_program_with_runtime_checkpoint_internal<S: FencedRuntimeCheckpointStore>(
        &mut self,
        target_program: &NairProgram,
        reaction_authority: &NairReactionAuthority,
        completion_authority: &NairCompletionAuthority,
        upgrade_authority: &RuntimeUpgradeAuthority,
        migration: RuntimeUpgradePlanRef<'_>,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<RuntimeUpgradeReport> {
        let plan = migration.atom_plan();
        let timer_plan = migration.timer_plan();
        if !self.runtime_checkpoint_bound {
            return Err(RuntimeUpgradeError::UpgradeRequiresDurableRuntimeBinding.into());
        }
        plan.verify_hash()?;
        if plan.source_program_hash() != self.program_hash {
            return Err(RuntimeUpgradeError::SourceProgramMismatch.into());
        }
        if plan.source_epoch() != self.program_epoch {
            return Err(RuntimeUpgradeError::SourceEpochMismatch {
                expected: plan.source_epoch().0,
                actual: self.program_epoch.0,
            }
            .into());
        }
        if !upgrade_authority.allows(plan.source_program_hash(), plan.target_program_hash()) {
            return Err(RuntimeUpgradeError::UnauthorizedTransition.into());
        }

        if let Some(timer_plan) = timer_plan {
            timer_plan.verify_hash()?;
        }

        let source_checkpoint = self.semantic_checkpoint(journal)?;
        let source_checkpoint_bytes = source_checkpoint.canonical_bytes()?;
        let source_checkpoint_hash = sha256(&source_checkpoint_bytes);
        let source_runtime = source_checkpoint.runtime_state().clone();
        let source_time = source_checkpoint.time_state().clone();
        let source_completions = source_checkpoint.completion_state().clone();
        if timer_plan.is_none() && !source_time.timers.is_empty() {
            return Err(RuntimeUpgradeError::PendingTimersUnsupported {
                count: source_time.timers.len(),
            }
            .into());
        }

        let target_epoch = ProgramEpoch(
            self.program_epoch
                .0
                .checked_add(1)
                .ok_or(RuntimeUpgradeError::ProgramEpochExhausted)?,
        );
        let mut candidate = Self::boot_with_fire_budget_and_authorities(
            target_program,
            source_time.fire_budget,
            reaction_authority,
            completion_authority,
        )?;
        if candidate.program_hash != plan.target_program_hash() {
            return Err(RuntimeUpgradeError::TargetProgramMismatch.into());
        }

        let mut target_runtime = candidate.runtime.capture_checkpoint_state()?;
        let mut target_time = candidate.time.capture_checkpoint_state();

        for source in plan.source_dispositions().keys() {
            if !source_runtime.atoms.contains_key(source) {
                return Err(RuntimeUpgradeError::UnknownSourceAtom(*source).into());
            }
        }
        for source in source_runtime.atoms.keys() {
            if !plan.source_dispositions().contains_key(source) {
                return Err(RuntimeUpgradeError::MissingSourceAtomDisposition(*source).into());
            }
        }
        for target in plan.target_defaults() {
            if !target_runtime.atoms.contains_key(target) {
                return Err(RuntimeUpgradeError::UnknownTargetAtom(*target).into());
            }
        }
        for target in plan.source_dispositions().values().flatten() {
            if !target_runtime.atoms.contains_key(target) {
                return Err(RuntimeUpgradeError::UnknownTargetAtom(*target).into());
            }
        }
        for target in target_runtime.atoms.keys() {
            let copied = plan
                .source_dispositions()
                .values()
                .any(|mapped| mapped == &Some(*target));
            if !copied && !plan.target_defaults().contains(target) {
                return Err(RuntimeUpgradeError::MissingTargetAtomDisposition(*target).into());
            }
        }

        let mut migrated_atoms = 0_usize;
        let mut dropped_atoms = 0_usize;
        for (source, target) in plan.source_dispositions() {
            match target {
                Some(target) => {
                    let source_snapshot = source_runtime
                        .atoms
                        .get(source)
                        .ok_or(RuntimeUpgradeError::UnknownSourceAtom(*source))?;
                    let target_snapshot = target_runtime
                        .atoms
                        .get_mut(target)
                        .ok_or(RuntimeUpgradeError::UnknownTargetAtom(*target))?;
                    target_snapshot.value = source_snapshot.value.clone();
                    target_snapshot.version = source_snapshot
                        .version
                        .max(target_snapshot.version)
                        .checked_add(1)
                        .ok_or(RuntimeUpgradeError::AtomVersionExhausted(*target))?;
                    migrated_atoms += 1;
                }
                None => dropped_atoms += 1,
            }
        }

        let (migrated_timers, dropped_timers, defaulted_timers, timer_plan_hash) = match timer_plan
        {
            Some(timer_plan) => {
                let (migrated, dropped, defaulted) = migrate_timer_upgrade_state(
                    &source_time,
                    &mut target_time,
                    &self.native_timer_bindings,
                    &candidate.native_timer_bindings,
                    timer_plan,
                )?;
                (migrated, dropped, defaulted, Some(timer_plan.plan_hash()))
            }
            None => {
                for timer in &target_time.timers {
                    if timer.next_deadline < source_time.now {
                        return Err(RuntimeUpgradeError::TargetTimerDeadlineBeforeUpgradeTime {
                            timer: timer.id.0,
                            deadline: timer.next_deadline.0,
                            logical_time: source_time.now.0,
                        }
                        .into());
                    }
                }
                target_time.now = source_time.now;
                target_time.next_timer_id =
                    target_time.next_timer_id.max(source_time.next_timer_id);
                (0, 0, 0, None)
            }
        };

        let atom_plan_hash = plan.plan_hash();
        let semantic_plan_hash = migration.semantic_plan_hash();
        let lineage_root = next_lineage_root(
            self.upgrade_chain_root,
            target_epoch,
            self.program_hash,
            candidate.program_hash,
            semantic_plan_hash,
            source_checkpoint_hash,
        );
        let lineage_record = RuntimeUpgradeLineageRecord {
            source_program_hash: self.program_hash,
            target_program_hash: candidate.program_hash,
            plan_hash: semantic_plan_hash,
            source_checkpoint_hash,
        };

        let mut runtime_replay_state = target_runtime.replay_state;
        hash_bytes(&mut runtime_replay_state, RUNTIME_UPGRADE_REPLAY_DOMAIN);
        hash_component(
            &mut runtime_replay_state,
            &source_runtime.replay_state.to_le_bytes(),
        );
        hash_component(&mut runtime_replay_state, &self.program_hash);
        hash_component(&mut runtime_replay_state, &candidate.program_hash);
        hash_component(&mut runtime_replay_state, &semantic_plan_hash.0);
        hash_component(&mut runtime_replay_state, &target_epoch.0.to_le_bytes());
        target_runtime.tick = source_runtime.tick;
        target_runtime.replay_state = runtime_replay_state;
        target_runtime.last_input_sequence = source_runtime.last_input_sequence;
        target_runtime.next_transaction_id = target_runtime
            .next_transaction_id
            .max(source_runtime.next_transaction_id);

        candidate
            .runtime
            .restore_checkpoint_state(&target_runtime)?;
        candidate.time.restore_checkpoint_state(&target_time)?;
        candidate
            .effect_completions
            .restore_checkpoint_state(&source_completions);
        candidate.effect_outbox = self.effect_outbox.clone();
        candidate.cycle = self.cycle;
        candidate.program_epoch = target_epoch;
        candidate.upgrade_chain_root = lineage_root;
        candidate.last_upgrade = Some(lineage_record);

        let mut event_replay_state = candidate.replay_state;
        hash_bytes(&mut event_replay_state, RUNTIME_UPGRADE_REPLAY_DOMAIN);
        hash_component(&mut event_replay_state, &self.replay_state.to_le_bytes());
        hash_component(&mut event_replay_state, &self.program_hash);
        hash_component(&mut event_replay_state, &candidate.program_hash);
        hash_component(&mut event_replay_state, &semantic_plan_hash.0);
        hash_component(&mut event_replay_state, &target_epoch.0.to_le_bytes());
        candidate.replay_state = event_replay_state;
        candidate.replay_key = EventLoopReplayKey(event_replay_state);

        let lease = journal.assert_active()?;
        let effect_checkpoint = journal.capture_checkpoint(&candidate.effect_outbox);
        let target_checkpoint = candidate.semantic_checkpoint(journal)?;
        let effect_bytes = effect_checkpoint.canonical_bytes();
        let runtime_bytes = target_checkpoint.canonical_bytes()?;
        journal
            .store_mut()
            .commit_effect_and_runtime_fenced(lease, &effect_bytes, &runtime_bytes)
            .map_err(RuntimeCheckpointError::from)?;

        candidate.runtime_checkpoint_bound = true;
        let report = RuntimeUpgradeReport {
            source_epoch: self.program_epoch,
            target_epoch,
            source_program_hash: self.program_hash,
            target_program_hash: candidate.program_hash,
            plan_hash: atom_plan_hash,
            timer_plan_hash,
            composite_plan_hash: semantic_plan_hash,
            source_checkpoint_hash,
            lineage_root,
            migrated_atoms,
            dropped_atoms,
            defaulted_atoms: plan.target_defaults().len(),
            migrated_timers,
            dropped_timers,
            defaulted_timers,
            cycle: candidate.cycle,
            logical_time: candidate.logical_time(),
        };
        *self = candidate;
        Ok(report)
    }

    pub fn schedule_once_at(&mut self, deadline: LogicalTime) -> EventLoopResult<TimerId> {
        let id = self.time.schedule_once_at(deadline)?;
        self.hash_schedule_once(id, deadline);
        Ok(id)
    }

    pub fn schedule_once_after(&mut self, delay: LogicalDuration) -> EventLoopResult<TimerId> {
        let deadline = self
            .time
            .now()
            .checked_add(delay)
            .ok_or(TimeError::TimeOverflow)?;
        self.schedule_once_at(deadline)
    }

    pub fn schedule_repeating_at(
        &mut self,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    ) -> EventLoopResult<TimerId> {
        let id = self.time.schedule_repeating_at(first_deadline, interval)?;
        self.hash_schedule_repeating(id, first_deadline, interval);
        Ok(id)
    }

    pub fn schedule_repeating_after(
        &mut self,
        initial_delay: LogicalDuration,
        interval: LogicalDuration,
    ) -> EventLoopResult<TimerId> {
        let deadline = self
            .time
            .now()
            .checked_add(initial_delay)
            .ok_or(TimeError::TimeOverflow)?;
        self.schedule_repeating_at(deadline, interval)
    }

    pub fn cancel_timer(&mut self, id: TimerId) -> bool {
        if !self.time.cancel(id) {
            return false;
        }
        hash_bytes(&mut self.replay_state, &[OP_CANCEL]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
        true
    }

    /// Advances time and the persistent runtime as one publication boundary.
    ///
    /// Time, runtime and effect-outbox state are evaluated on private clones. If
    /// candidate evaluation fails, the published event-loop state is unchanged.
    pub fn cycle_to(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
    ) -> EventLoopResult<EventLoopCycleReport> {
        self.cycle_to_internal(target, input, None)
    }

    pub fn cycle_to_with_effect_completions<S: FencedEffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        completions: &EffectCompletionBatch,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        journal.assert_active()?;
        self.cycle_to_internal(target, input, Some((completions, journal.audit_ledger())))
    }

    fn cycle_to_internal(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        completion_context: Option<(&EffectCompletionBatch, &EffectAuditLedger)>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let mut time = self.time.clone();
        let mut runtime = self.runtime.clone();
        let mut effect_outbox = self.effect_outbox.clone();
        let mut effect_completions = self.effect_completions.clone();

        let time_report = time.advance_to(target)?;
        let (runtime_report, input_reactions, timer_reactions) = match completion_context {
            Some((completions, audit)) => runtime.tick_with_reactions_and_completions(
                input,
                &time_report.fires,
                &self.reactions,
                &mut effect_completions,
                audit,
                completions,
            )?,
            None => runtime.tick_with_reactions(input, &time_report.fires, &self.reactions)?,
        };
        let reactions = NairReactionCycleReport {
            input: input_reactions,
            timers: timer_reactions,
        };
        let cycle = self.cycle.checked_add(1).ok_or(TimeError::TimeOverflow)?;
        let effects = effect_outbox.stage_cycle(
            cycle,
            reactions
                .input
                .effect_intents
                .iter()
                .chain(reactions.timers.effect_intents.iter())
                .cloned(),
        )?;

        let mut replay_state = self.replay_state;
        hash_bytes(&mut replay_state, &[OP_CYCLE]);
        hash_component(&mut replay_state, &cycle.to_le_bytes());
        hash_component(&mut replay_state, &target.0.to_le_bytes());
        hash_component(
            &mut replay_state,
            &runtime_report.replay_key.value().to_le_bytes(),
        );
        hash_component(
            &mut replay_state,
            &(time_report.fires.len() as u64).to_le_bytes(),
        );
        for fire in &time_report.fires {
            hash_component(&mut replay_state, &fire.timer.0.to_le_bytes());
            hash_component(&mut replay_state, &fire.deadline.0.to_le_bytes());
            hash_component(&mut replay_state, &fire.occurrence.to_le_bytes());
        }
        hash_effect_stage(&mut replay_state, &effects);
        let replay_key = EventLoopReplayKey(replay_state);
        let completion_report = runtime_report.completions.clone();

        self.time = time;
        self.runtime = runtime;
        self.effect_outbox = effect_outbox;
        self.effect_completions = effect_completions;
        self.cycle = cycle;
        self.replay_state = replay_state;
        self.replay_key = replay_key;

        Ok(EventLoopCycleReport {
            cycle,
            logical_time: target,
            time: time_report,
            runtime: runtime_report,
            reactions,
            effects,
            completions: completion_report,
            replay_key,
        })
    }

    pub fn cycle_to_with_effect_journal<S: EffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        journal: &mut GovernedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let mut candidate = self.clone();
        let report = candidate.cycle_to(target, input)?;
        journal.checkpoint(&candidate.effect_outbox)?;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_fenced_effect_journal<S: FencedEffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        journal: &mut GovernedFencedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        journal.assert_active()?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to(target, input)?;
        journal.checkpoint(&candidate.effect_outbox)?;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_retry_effect_journal<S: FencedEffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        journal: &mut GovernedRetryEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        journal.assert_active()?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to(target, input)?;
        journal.checkpoint(&candidate.effect_outbox)?;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_audited_effect_journal<S: FencedEffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        journal.assert_active()?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to(target, input)?;
        journal.checkpoint(&candidate.effect_outbox)?;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_audited_effect_journal_and_completions<S: FencedEffectJournalStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        completions: &EffectCompletionBatch,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        journal.assert_active()?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to_internal(
            target,
            input,
            Some((completions, journal.audit_ledger())),
        )?;
        journal.checkpoint(&candidate.effect_outbox)?;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_runtime_checkpoint<S: FencedRuntimeCheckpointStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let lease = self.runtime_checkpoint_commit_lease(journal)?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to(target, input)?;
        let effect_checkpoint = journal.capture_checkpoint(&candidate.effect_outbox);
        let runtime_checkpoint = candidate.semantic_checkpoint(journal)?;
        let runtime_bytes = runtime_checkpoint.canonical_bytes()?;
        journal
            .store_mut()
            .commit_effect_and_runtime_fenced(
                lease,
                &effect_checkpoint.canonical_bytes(),
                &runtime_bytes,
            )
            .map_err(RuntimeCheckpointError::from)?;
        candidate.runtime_checkpoint_bound = true;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_to_with_runtime_checkpoint_and_completions<S: FencedRuntimeCheckpointStore>(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
        completions: &EffectCompletionBatch,
        journal: &mut GovernedAuditedEffectJournal<S>,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let lease = self.runtime_checkpoint_commit_lease(journal)?;
        let mut candidate = self.clone();
        let report = candidate.cycle_to_internal(
            target,
            input,
            Some((completions, journal.audit_ledger())),
        )?;
        let effect_checkpoint = journal.capture_checkpoint(&candidate.effect_outbox);
        let runtime_checkpoint = candidate.semantic_checkpoint(journal)?;
        let runtime_bytes = runtime_checkpoint.canonical_bytes()?;
        journal
            .store_mut()
            .commit_effect_and_runtime_fenced(
                lease,
                &effect_checkpoint.canonical_bytes(),
                &runtime_bytes,
            )
            .map_err(RuntimeCheckpointError::from)?;
        candidate.runtime_checkpoint_bound = true;
        *self = candidate;
        Ok(report)
    }

    pub fn cycle_by(
        &mut self,
        duration: LogicalDuration,
        input: &InputBatch,
    ) -> EventLoopResult<EventLoopCycleReport> {
        let target = self
            .time
            .now()
            .checked_add(duration)
            .ok_or(TimeError::TimeOverflow)?;
        self.cycle_to(target, input)
    }

    pub fn cycle_to_next_deadline(
        &mut self,
        input: &InputBatch,
    ) -> EventLoopResult<Option<EventLoopCycleReport>> {
        let Some(deadline) = self.next_deadline() else {
            return Ok(None);
        };
        self.cycle_to(deadline, input).map(Some)
    }

    fn hash_schedule_once(&mut self, id: TimerId, deadline: LogicalTime) {
        hash_bytes(&mut self.replay_state, &[OP_SCHEDULE_ONCE]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        hash_component(&mut self.replay_state, &deadline.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
    }

    fn hash_schedule_repeating(
        &mut self,
        id: TimerId,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    ) {
        hash_bytes(&mut self.replay_state, &[OP_SCHEDULE_REPEATING]);
        hash_component(&mut self.replay_state, &id.0.to_le_bytes());
        hash_component(&mut self.replay_state, &first_deadline.0.to_le_bytes());
        hash_component(&mut self.replay_state, &interval.0.to_le_bytes());
        self.replay_key = EventLoopReplayKey(self.replay_state);
    }
}

fn hash_effect_stage(replay_state: &mut u64, report: &EffectOutboxStageReport) {
    if report.enqueued.is_empty() {
        return;
    }

    hash_bytes(replay_state, b"NORDOI-EFFECT-OUTBOX-1.0");
    hash_component(replay_state, &(report.enqueued.len() as u64).to_le_bytes());
    for queued in &report.enqueued {
        hash_component(replay_state, &queued.id.0.to_le_bytes());
        hash_component(replay_state, &queued.cycle.to_le_bytes());
        hash_component(replay_state, &queued.ordinal.to_le_bytes());
        hash_component(replay_state, &queued.intent.reaction.0.to_le_bytes());
        hash_component(replay_state, queued.intent.action_name.as_bytes());
        hash_effect(replay_state, &queued.intent.effect);
    }
}

fn hash_effect(replay_state: &mut u64, effect: &Effect) {
    match effect {
        Effect::Pure => hash_bytes(replay_state, &[0x00]),
        Effect::StateRead => hash_bytes(replay_state, &[0x01]),
        Effect::StateWrite => hash_bytes(replay_state, &[0x02]),
        Effect::Network(scope) => {
            hash_bytes(replay_state, &[0x10]);
            hash_component(replay_state, scope.as_bytes());
        }
        Effect::FileRead(scope) => {
            hash_bytes(replay_state, &[0x11]);
            hash_component(replay_state, scope.as_bytes());
        }
        Effect::FileWrite(scope) => {
            hash_bytes(replay_state, &[0x12]);
            hash_component(replay_state, scope.as_bytes());
        }
        Effect::Camera => hash_bytes(replay_state, &[0x20]),
        Effect::Microphone => hash_bytes(replay_state, &[0x21]),
        Effect::Location => hash_bytes(replay_state, &[0x22]),
        Effect::Gpu => hash_bytes(replay_state, &[0x23]),
        Effect::Xr => hash_bytes(replay_state, &[0x24]),
        Effect::Process => hash_bytes(replay_state, &[0x25]),
    }
}

fn migrate_timer_upgrade_state(
    source_time: &TimeCheckpointState,
    target_time: &mut TimeCheckpointState,
    source_bindings: &BTreeMap<TimerSlot, TimerId>,
    target_bindings: &BTreeMap<TimerSlot, TimerId>,
    timer_plan: &RuntimeTimerUpgradePlan,
) -> Result<(usize, usize, usize), RuntimeUpgradeError> {
    for source in timer_plan.source_dispositions().keys() {
        if !source_bindings.contains_key(source) {
            return Err(RuntimeUpgradeError::UnknownSourceTimerSlot(*source));
        }
    }
    for source in source_bindings.keys() {
        if !timer_plan.source_dispositions().contains_key(source) {
            return Err(RuntimeUpgradeError::MissingSourceTimerDisposition(*source));
        }
    }
    for target in timer_plan.target_defaults() {
        if !target_bindings.contains_key(target) {
            return Err(RuntimeUpgradeError::UnknownTargetTimerSlot(*target));
        }
    }
    for target in timer_plan.source_dispositions().values().flatten() {
        if !target_bindings.contains_key(target) {
            return Err(RuntimeUpgradeError::UnknownTargetTimerSlot(*target));
        }
    }
    let carried_targets: BTreeSet<TimerSlot> = timer_plan
        .source_dispositions()
        .values()
        .flatten()
        .copied()
        .collect();
    for target in target_bindings.keys() {
        if !carried_targets.contains(target) && !timer_plan.target_defaults().contains(target) {
            return Err(RuntimeUpgradeError::MissingTargetTimerDisposition(*target));
        }
    }

    let source_native_ids: BTreeSet<TimerId> = source_bindings.values().copied().collect();
    for timer in &source_time.timers {
        if timer.next_deadline < source_time.now {
            return Err(RuntimeUpgradeError::SourceTimerDeadlineBeforeUpgradeTime {
                timer: timer.id.0,
                deadline: timer.next_deadline.0,
                logical_time: source_time.now.0,
            });
        }
        if !source_native_ids.contains(&timer.id) {
            return Err(RuntimeUpgradeError::DynamicSourceTimerUnsupported { timer: timer.id.0 });
        }
    }

    let source_timers: BTreeMap<TimerId, TimerSnapshot> = source_time
        .timers
        .iter()
        .copied()
        .map(|timer| (timer.id, timer))
        .collect();
    let mut target_timers: BTreeMap<TimerId, TimerSnapshot> = target_time
        .timers
        .iter()
        .copied()
        .map(|timer| (timer.id, timer))
        .collect();

    let mut migrated = 0_usize;
    let mut dropped = 0_usize;
    for (source_slot, target_slot) in timer_plan.source_dispositions() {
        let source_id = source_bindings
            .get(source_slot)
            .copied()
            .ok_or(RuntimeUpgradeError::UnknownSourceTimerSlot(*source_slot))?;
        match target_slot {
            Some(target_slot) => {
                let target_id = target_bindings
                    .get(target_slot)
                    .copied()
                    .ok_or(RuntimeUpgradeError::UnknownTargetTimerSlot(*target_slot))?;
                match source_timers.get(&source_id).copied() {
                    Some(source_timer) => {
                        let target_timer = target_timers
                            .get(&target_id)
                            .copied()
                            .ok_or(RuntimeUpgradeError::TargetTimerInactive(*target_slot))?;
                        match (source_timer.interval, target_timer.interval) {
                            (None, None) => {}
                            (Some(source_interval), Some(target_interval))
                                if source_interval == target_interval => {}
                            (Some(source_interval), Some(target_interval)) => {
                                return Err(RuntimeUpgradeError::TimerIntervalMismatch {
                                    source: *source_slot,
                                    target: *target_slot,
                                    source_interval: source_interval.0,
                                    target_interval: target_interval.0,
                                });
                            }
                            _ => {
                                return Err(RuntimeUpgradeError::TimerKindMismatch {
                                    source: *source_slot,
                                    target: *target_slot,
                                });
                            }
                        }
                        target_timers.insert(
                            target_id,
                            TimerSnapshot {
                                id: target_id,
                                next_deadline: source_timer.next_deadline,
                                interval: source_timer.interval,
                                occurrences: source_timer.occurrences,
                            },
                        );
                    }
                    None => {
                        target_timers.remove(&target_id);
                    }
                }
                migrated += 1;
            }
            None => dropped += 1,
        }
    }

    for target_slot in timer_plan.target_defaults() {
        let target_id = target_bindings
            .get(target_slot)
            .copied()
            .ok_or(RuntimeUpgradeError::UnknownTargetTimerSlot(*target_slot))?;
        if let Some(timer) = target_timers.get(&target_id) {
            if timer.next_deadline < source_time.now {
                return Err(RuntimeUpgradeError::TargetTimerDeadlineBeforeUpgradeTime {
                    timer: timer.id.0,
                    deadline: timer.next_deadline.0,
                    logical_time: source_time.now.0,
                });
            }
        }
    }

    target_time.now = source_time.now;
    target_time.next_timer_id = target_time.next_timer_id.max(source_time.next_timer_id);
    target_time.timers = target_timers.into_values().collect();

    Ok((migrated, dropped, timer_plan.target_defaults().len()))
}

fn apply_native_time_bootstrap(
    program: &NairProgram,
    time: &mut AtomicTimeCore,
) -> EventLoopResult<BTreeMap<TimerSlot, TimerId>> {
    let mut bindings = BTreeMap::new();

    for instruction in program.instructions() {
        match instruction {
            Instruction::ScheduleTimerOnceAt { dst, deadline } => {
                let id = time.schedule_once_at(*deadline)?;
                bindings.insert(*dst, id);
            }
            Instruction::ScheduleTimerRepeatingAt {
                dst,
                first_deadline,
                interval,
            } => {
                let id = time.schedule_repeating_at(*first_deadline, *interval)?;
                bindings.insert(*dst, id);
            }
            Instruction::CancelTimer { timer } => {
                let id = bindings
                    .get(timer)
                    .copied()
                    .ok_or(RuntimeError::Nair(NairError::UnknownTimerSlot(*timer)))?;
                time.cancel(id);
            }
            _ => {}
        }
    }

    Ok(bindings)
}
