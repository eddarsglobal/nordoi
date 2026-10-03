use std::fmt::{Display, Formatter};

use crate::{
    effect::Effect,
    effect_audit::{EffectAuditDispatchOutcome, EffectAuditResult, GovernedAuditedEffectJournal},
    effect_dispatch::{
        AtomicEffectOutbox, EffectBackend, EffectDeliveryNamespace, EffectDispatchReceipt,
        EffectDispatchResult, EffectIntentId, EffectOutboxStageReport, GovernedEffectDispatcher,
        QueuedEffectIntent,
    },
    effect_fencing::{EffectFencingResult, FencedEffectJournalStore, GovernedFencedEffectJournal},
    effect_persistence::{
        EffectJournalStore, EffectOutboxCheckpoint, EffectPersistenceResult, GovernedEffectJournal,
    },
    effect_retry::{
        DeadLetteredEffect, EffectRetryDispatchOutcome, EffectRetryResult, EffectRetryTick,
        GovernedRetryEffectJournal,
    },
    input::InputBatch,
    nair::{
        bootstrap_native_reactions, Instruction, NairError, NairProgram, NairReactionAuthority,
        NairReactionCycleReport, ReactionSlot, TimerSlot,
    },
    reaction::{AtomicReactionCore, ReactionId},
    runtime::{
        hash_bytes, hash_component, PersistentAtomicRuntime, PersistentRuntimeTickReport,
        RuntimeAtomSnapshot, RuntimeError, FNV_OFFSET_BASIS,
    },
    AtomSlot,
};

use std::collections::BTreeMap;

use super::{
    AtomicTimeCore, EventLoopResult, LogicalDuration, LogicalTime, TimeAdvanceReport, TimeError,
    TimerId, TimerSnapshot, DEFAULT_TIMER_FIRE_BUDGET,
};

const EVENT_LOOP_REPLAY_DOMAIN: &[u8] = b"NORDOI-ATOMIC-EVENT-LOOP-1.7";
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
    pub replay_key: EventLoopReplayKey,
}

#[derive(Debug, Clone)]
pub struct AtomicEventLoop {
    runtime: PersistentAtomicRuntime,
    time: AtomicTimeCore,
    cycle: u64,
    replay_state: u64,
    replay_key: EventLoopReplayKey,
    native_timer_bindings: BTreeMap<TimerSlot, TimerId>,
    reactions: AtomicReactionCore,
    native_reaction_bindings: BTreeMap<ReactionSlot, ReactionId>,
    effect_outbox: AtomicEffectOutbox,
}

impl AtomicEventLoop {
    pub fn boot(program: &NairProgram) -> EventLoopResult<Self> {
        let authority = NairReactionAuthority::new();
        Self::boot_with_fire_budget_and_reaction_authority(
            program,
            DEFAULT_TIMER_FIRE_BUDGET,
            &authority,
        )
    }

    pub fn boot_with_fire_budget(
        program: &NairProgram,
        fire_budget: usize,
    ) -> EventLoopResult<Self> {
        let authority = NairReactionAuthority::new();
        Self::boot_with_fire_budget_and_reaction_authority(program, fire_budget, &authority)
    }

    pub fn boot_with_reaction_authority(
        program: &NairProgram,
        authority: &NairReactionAuthority,
    ) -> EventLoopResult<Self> {
        Self::boot_with_fire_budget_and_reaction_authority(
            program,
            DEFAULT_TIMER_FIRE_BUDGET,
            authority,
        )
    }

    pub fn boot_with_fire_budget_and_reaction_authority(
        program: &NairProgram,
        fire_budget: usize,
        authority: &NairReactionAuthority,
    ) -> EventLoopResult<Self> {
        program.validate().map_err(RuntimeError::from)?;
        let program_bytes = program.canonical_bytes().map_err(RuntimeError::from)?;
        let mut time = AtomicTimeCore::with_fire_budget(fire_budget)?;
        let native_timer_bindings = apply_native_time_bootstrap(program, &mut time)?;
        let runtime_program = NairProgram::from_instructions(
            program
                .instructions()
                .iter()
                .filter(|instruction| {
                    !instruction.requires_time_context() && !instruction.requires_reaction_context()
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
            authority,
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
            native_timer_bindings,
            reactions,
            native_reaction_bindings,
            effect_outbox: AtomicEffectOutbox::new(),
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

    pub fn pending_effect_count(&self) -> usize {
        self.effect_outbox.pending_len()
    }

    pub fn pending_effect(&self, id: EffectIntentId) -> Option<&QueuedEffectIntent> {
        self.effect_outbox.get(id)
    }

    pub fn pending_effects(&self) -> impl Iterator<Item = &QueuedEffectIntent> {
        self.effect_outbox.iter()
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
        let mut time = self.time.clone();
        let mut runtime = self.runtime.clone();
        let mut effect_outbox = self.effect_outbox.clone();

        let time_report = time.advance_to(target)?;
        let (runtime_report, input_reactions, timer_reactions) =
            runtime.tick_with_reactions(input, &time_report.fires, &self.reactions)?;
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

        self.time = time;
        self.runtime = runtime;
        self.effect_outbox = effect_outbox;
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
