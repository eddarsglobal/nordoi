use nordoi_kernel::{
    AtomSlot, AtomicEventLoop, AtomicInputCore, Capability, CapabilitySet, DomainRef, Effect,
    EffectBackend, EffectBackendError, EffectBackendReceipt, EffectDispatchAuthority,
    EffectDispatchError, EventLoopError, GovernedEffectDispatcher, InputDeviceId, InputPayload,
    InputSignal, InputSource, InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairProgram,
    NairReactionAuthority, NairReactionStep, NairReactionTrigger, NairReactionValue,
    QueuedEffectIntent, ReactionError, ReactionSlot, RegisterId, RuntimeError, TimerSlot, Value,
};

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
}

fn keyboard_batch(code: u32, pressed: bool) -> nordoi_kernel::InputBatch {
    let mut input = AtomicInputCore::new();
    submit_key(&mut input, code, pressed)
}

fn submit_key(input: &mut AtomicInputCore, code: u32, pressed: bool) -> nordoi_kernel::InputBatch {
    input
        .submit(
            InputSource::Keyboard,
            InputDeviceId(1),
            None,
            InputPayload::Key {
                code,
                pressed,
                repeat: false,
            },
        )
        .unwrap();
    input.drain()
}

fn network_program(code: u32, scope: &str) -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network(scope.into());
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "network-reaction".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code },
            },
            action_name: "network-action".into(),
            declared_effects: effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: effect.clone(),
            }],
        },
        Instruction::Halt,
    ]);

    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network(scope.into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);

    (program, authority)
}

fn input_and_timer_effect_program(scope: &str) -> (NairProgram, NairReactionAuthority) {
    let effect = Effect::Network(scope.into());
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(5),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "input-network".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 5 },
            },
            action_name: "input-effect".into(),
            declared_effects: effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: effect.clone(),
            }],
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(1),
            name: "timer-network".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: None,
            },
            action_name: "timer-effect".into(),
            declared_effects: effects([effect.clone()]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: effect.clone(),
            }],
        },
        Instruction::Halt,
    ]);

    let mut authority = NairReactionAuthority::new();
    for slot in [ReactionSlot(0), ReactionSlot(1)] {
        let mut capabilities = CapabilitySet::new();
        capabilities.allow(Capability::Network(scope.into()));
        authority.set(slot, capabilities);
    }

    (program, authority)
}

fn dispatch_authority(scope: &str) -> EffectDispatchAuthority {
    let mut authority = EffectDispatchAuthority::new();
    authority.grant(Capability::Network(scope.into()));
    authority
}

#[derive(Debug, Default)]
struct RecordingBackend {
    supported: Option<Effect>,
    fail: bool,
    seen: Vec<QueuedEffectIntent>,
}

impl RecordingBackend {
    fn supporting(effect: Effect) -> Self {
        Self {
            supported: Some(effect),
            fail: false,
            seen: Vec::new(),
        }
    }

    fn failing(effect: Effect) -> Self {
        Self {
            supported: Some(effect),
            fail: true,
            seen: Vec::new(),
        }
    }
}

impl EffectBackend for RecordingBackend {
    fn supports(&self, effect: &Effect) -> bool {
        self.supported
            .as_ref()
            .is_none_or(|supported| supported == effect)
    }

    fn execute(
        &mut self,
        request: &QueuedEffectIntent,
    ) -> Result<EffectBackendReceipt, EffectBackendError> {
        self.seen.push(request.clone());
        if self.fail {
            Err(EffectBackendError::new("synthetic backend failure"))
        } else {
            Ok(EffectBackendReceipt::new(format!(
                "receipt-{}",
                request.id.value()
            )))
        }
    }
}

#[test]
fn committed_effect_intent_is_enqueued_with_stable_identity() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();

    let report = loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();

    assert_eq!(report.effects.len(), 1);
    assert_eq!(report.effects.enqueued[0].id.value(), 1);
    assert_eq!(report.effects.enqueued[0].cycle, 1);
    assert_eq!(report.effects.enqueued[0].ordinal, 0);
    assert_eq!(report.effects.enqueued[0].intent.reaction.value(), 1);
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn successive_cycles_allocate_monotonic_effect_ids() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut input = AtomicInputCore::new();

    let first = loop_
        .cycle_to(LogicalTime::ZERO, &submit_key(&mut input, 7, true))
        .unwrap();
    let second = loop_
        .cycle_to(LogicalTime::ZERO, &submit_key(&mut input, 7, false))
        .unwrap();

    assert_eq!(first.effects.enqueued[0].id.value(), 1);
    assert_eq!(second.effects.enqueued[0].id.value(), 2);
    assert_eq!(loop_.pending_effect_count(), 2);
}

#[test]
fn input_effects_precede_timer_effects_in_outbox() {
    let (program, authority) = input_and_timer_effect_program("api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();

    let report = loop_
        .cycle_to(LogicalTime(5), &keyboard_batch(5, true))
        .unwrap();

    assert_eq!(report.effects.enqueued.len(), 2);
    assert_eq!(
        report.effects.enqueued[0].intent.action_name,
        "input-effect"
    );
    assert_eq!(report.effects.enqueued[0].ordinal, 0);
    assert_eq!(
        report.effects.enqueued[1].intent.action_name,
        "timer-effect"
    );
    assert_eq!(report.effects.enqueued[1].ordinal, 1);
}

#[test]
fn unmatched_cycle_creates_zero_effect_outbox_work() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();

    let report = loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(99, true))
        .unwrap();

    assert!(report.effects.is_empty());
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn failed_cycle_publishes_no_effect_intent() {
    let effect = Effect::Network("api.example.test".into());
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(u64::MAX),
        },
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "fail-after-plan".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: None,
            },
            action_name: "plan-effect-then-overflow".into(),
            declared_effects: effects([Effect::StateWrite, effect.clone()]),
            steps: vec![
                NairReactionStep::EmitEffect {
                    effect: effect.clone(),
                },
                NairReactionStep::Set {
                    atom: AtomSlot(0),
                    value: NairReactionValue::TimerDeadlineTicks,
                },
            ],
        },
        Instruction::Halt,
    ]);
    let mut capabilities = CapabilitySet::new();
    capabilities.allow(Capability::Network("api.example.test".into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capabilities);
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();

    assert!(matches!(
        loop_.cycle_to(LogicalTime(u64::MAX), &Default::default()),
        Err(EventLoopError::Runtime(RuntimeError::Reaction(
            ReactionError::IntegerProjectionOverflow(u64::MAX)
        )))
    ));
    assert_eq!(loop_.cycle_index(), 0);
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn dispatcher_denies_by_default_and_preserves_pending_intent() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(EffectDispatchAuthority::new());
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    assert_eq!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::CapabilityDenied(Capability::Network(
            "api.example.test".into()
        )))
    );
    assert_eq!(loop_.pending_effect_count(), 1);
    assert!(backend.seen.is_empty());
}

#[test]
fn dispatcher_requires_exact_capability_scope() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("other.example.test"));
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    assert!(matches!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::CapabilityDenied(Capability::Network(scope)))
            if scope == "api.example.test"
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn successful_dispatch_acknowledges_only_after_backend_success() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    let receipt = loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .unwrap()
        .unwrap();

    assert_eq!(receipt.request.id.value(), 1);
    assert_eq!(receipt.backend.reference.as_deref(), Some("receipt-1"));
    assert_eq!(loop_.pending_effect_count(), 0);
    assert_eq!(backend.seen.len(), 1);
}

#[test]
fn unsupported_backend_preserves_pending_intent() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = RecordingBackend::supporting(Effect::FileRead("/tmp/data".into()));

    assert!(matches!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::BackendUnsupported { intent, .. }) if intent.value() == 1
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    assert!(backend.seen.is_empty());
}

#[test]
fn backend_failure_preserves_pending_intent_for_retry() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = RecordingBackend::failing(Effect::Network("api.example.test".into()));

    assert!(matches!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::BackendFailed { intent, .. }) if intent.value() == 1
    ));
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(backend.seen.len(), 1);
}

#[test]
fn dispatch_authority_can_be_revoked_before_execution() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let capability = Capability::Network("api.example.test".into());
    let mut dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    dispatcher.authority_mut().revoke(&capability);
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    assert_eq!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::CapabilityDenied(capability))
    );
    assert_eq!(loop_.pending_effect_count(), 1);
}

#[test]
fn revoked_intent_can_execute_after_explicit_regrant() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let capability = Capability::Network("api.example.test".into());
    let mut dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    dispatcher.authority_mut().revoke(&capability);
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    assert!(loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .is_err());
    dispatcher.authority_mut().grant(capability);
    assert!(loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .unwrap()
        .is_some());
    assert_eq!(loop_.pending_effect_count(), 0);
}

#[test]
fn internal_effect_cannot_cross_external_dispatch_boundary() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "internal".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 3 },
            },
            action_name: "internal-effect".into(),
            declared_effects: effects([Effect::StateRead]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: Effect::StateRead,
            }],
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(3, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(EffectDispatchAuthority::new());
    let mut backend = RecordingBackend::default();

    assert_eq!(
        loop_.dispatch_next_effect(&dispatcher, &mut backend),
        Err(EffectDispatchError::InternalEffectNotDispatchable(
            Effect::StateRead
        ))
    );
    assert_eq!(loop_.pending_effect_count(), 1);
    assert!(backend.seen.is_empty());
}

#[test]
fn dispatch_receipt_does_not_change_deterministic_replay_identity() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let before = loop_.replay_key();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .unwrap();

    assert_eq!(loop_.replay_key(), before);
}

#[test]
fn equal_cause_traces_produce_equal_effect_envelopes() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut left = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let mut right = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();

    let left_report = left
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();
    let right_report = right
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();

    assert_eq!(left_report.effects, right_report.effects);
    assert_eq!(left.replay_key(), right.replay_key());
}

#[test]
fn pending_effect_survives_later_cycle_without_new_effects() {
    let (program, authority) = network_program(7, "api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();

    let report = loop_.cycle_to(LogicalTime(1), &Default::default()).unwrap();

    assert!(report.effects.is_empty());
    assert_eq!(loop_.pending_effect_count(), 1);
    assert_eq!(loop_.pending_effects().next().unwrap().id.value(), 1);
}

#[test]
fn dispatcher_processes_pending_effects_in_intent_id_order() {
    let (program, authority) = input_and_timer_effect_program("api.example.test");
    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    loop_
        .cycle_to(LogicalTime(5), &keyboard_batch(5, true))
        .unwrap();
    let dispatcher = GovernedEffectDispatcher::new(dispatch_authority("api.example.test"));
    let mut backend = RecordingBackend::supporting(Effect::Network("api.example.test".into()));

    let first = loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .unwrap()
        .unwrap();
    let second = loop_
        .dispatch_next_effect(&dispatcher, &mut backend)
        .unwrap()
        .unwrap();

    assert_eq!(first.request.id.value(), 1);
    assert_eq!(second.request.id.value(), 2);
    assert_eq!(first.request.intent.action_name, "input-effect");
    assert_eq!(second.request.intent.action_name, "timer-effect");
    assert_eq!(loop_.pending_effect_count(), 0);
}
