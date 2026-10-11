//! P0.5A opt-in fenced production event loop.
//!
//! Owns the *actual* kernel event loop, fenced journal and P0.4 durable
//! delivery backend. This profile has no public getter, escape hatch or
//! arbitrary-backend dispatch method. Outside this profile, historical
//! dispatch and observable I/O APIs still exist: not kernel-wide enforcement.

use crate::production::ProductionEffectBackend;
use crate::SignedEnvelopeSource;
use nordoi_kernel::{
    AtomicEventLoop, EffectBackend, EffectDispatchReceipt, EffectFencingResult, EffectJournalLease,
    EventLoopCycleReport, EventLoopResult, FencedEffectJournalStore, GovernedEffectDispatcher,
    GovernedFencedEffectJournal, InputBatch, LogicalTime,
};

/// Narrow host-owned profile: the event loop and backend cannot be removed
/// or replaced through this facade's public interface.
///
/// Before external execution, the fenced journal supplies a checked lease,
/// delivery key and fence, and P0.4 records the delivery identity. Only then
/// may P0.3 authenticate the signed envelope and call the destination.
/// Recipient-side exactly-once delivery is NOT proven.
pub struct FencedProductionEffects<Store, Backend, Source> {
    event_loop: AtomicEventLoop,
    journal: GovernedFencedEffectJournal<Store>,
    dispatcher: GovernedEffectDispatcher,
    backend: ProductionEffectBackend<Backend, Source>,
}

impl<Store, Backend, Source> FencedProductionEffects<Store, Backend, Source> {
    pub fn new(
        event_loop: AtomicEventLoop,
        journal: GovernedFencedEffectJournal<Store>,
        dispatcher: GovernedEffectDispatcher,
        backend: ProductionEffectBackend<Backend, Source>,
    ) -> Self {
        Self {
            event_loop,
            journal,
            dispatcher,
            backend,
        }
    }

    pub fn pending_effect_count(&self) -> usize {
        self.event_loop.pending_effect_count()
    }
}

impl<Store, Backend, Source> FencedProductionEffects<Store, Backend, Source>
where
    Store: FencedEffectJournalStore,
    Backend: EffectBackend,
    Source: SignedEnvelopeSource,
{
    pub fn acquire(&mut self) -> EffectFencingResult<EffectJournalLease> {
        self.journal.acquire()
    }

    pub fn recover(&mut self) -> EffectFencingResult<bool> {
        self.event_loop
            .recover_effects_from_fenced_journal(&mut self.journal)
    }

    /// Publish an event-loop cycle only after its outbox has been checkpointed
    /// by the currently active fenced journal writer.
    pub fn cycle_to(
        &mut self,
        target: LogicalTime,
        input: &InputBatch,
    ) -> EventLoopResult<EventLoopCycleReport> {
        self.event_loop
            .cycle_to_with_fenced_effect_journal(target, input, &mut self.journal)
    }

    /// Only externally executing entrypoint of this facade. The concrete
    /// backend is P0.4, not an arbitrary caller-provided `EffectBackend`.
    pub fn dispatch_next(&mut self) -> EffectFencingResult<Option<EffectDispatchReceipt>> {
        self.event_loop.dispatch_next_effect_with_fenced_journal(
            &mut self.journal,
            &self.dispatcher,
            &mut self.backend,
        )
    }

    pub fn release(&mut self) -> EffectFencingResult<EffectJournalLease> {
        self.journal.release()
    }
}
