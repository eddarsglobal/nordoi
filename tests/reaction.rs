use nordoi_kernel::{
    ActionSpec, AtomicError, AtomicInputCore, AtomicKernel, AtomicReactionCore, Capability,
    CapabilitySet, Effect, InputBatch, InputDeviceId, InputEvent, InputPayload, InputSelector,
    InputSequence, InputSignal, InputSource, InputTarget, LogicalTime, ReactionError, ReactionSpec,
    ReactionStep, ReactionTrigger, ReactionValue, TimerFire, TimerId, TimerSelector, Value,
};

fn key_selector(code: u32) -> InputSelector {
    InputSelector::any(InputSignal::KeyPressed { code })
}

fn state_action(name: &str) -> ActionSpec {
    ActionSpec::new(name).declare(Effect::StateWrite)
}

fn key_batch(code: u32, pressed: bool) -> InputBatch {
    let mut input = AtomicInputCore::new();
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

#[test]
fn reaction_ids_are_monotonic_and_define_stable_order() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    let root = kernel.root_domain();
    let trigger = ReactionTrigger::input(key_selector(7));
    let mut reactions = AtomicReactionCore::new();

    let first = reactions
        .register(
            &kernel,
            ReactionSpec::new("first", root, trigger, state_action("first-action"))
                .step(ReactionStep::set(atom, ReactionValue::literal(1_i64))),
        )
        .unwrap();
    let second = reactions
        .register(
            &kernel,
            ReactionSpec::new("second", root, trigger, state_action("second-action"))
                .step(ReactionStep::set(atom, ReactionValue::literal(2_i64))),
        )
        .unwrap();

    assert!(first < second);
    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(7, true))
        .unwrap();

    assert_eq!(kernel.get(atom).unwrap(), &Value::Int(2));
    assert_eq!(kernel.version(atom).unwrap(), 2);
    assert_eq!(kernel.pending_work(), 1);
    assert_eq!(report.matched_reactions, 2);
    assert_eq!(report.transactions, 2);
}

#[test]
fn unmatched_input_creates_zero_nam_work() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "space",
                root,
                ReactionTrigger::input(key_selector(32)),
                state_action("space-action"),
            )
            .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(13, true))
        .unwrap();

    assert_eq!(report.causes, 1);
    assert_eq!(report.matched_reactions, 0);
    assert_eq!(report.transactions, 0);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn input_value_can_project_into_owned_atom() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "key-state",
                root,
                ReactionTrigger::input(key_selector(4)),
                state_action("project-key"),
            )
            .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(4, true))
        .unwrap();

    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(true));
    assert_eq!(report.changed_atoms, 1);
}

#[test]
fn identical_projection_creates_no_downstream_work() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(true);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "same",
                root,
                ReactionTrigger::input(key_selector(4)),
                state_action("same-value"),
            )
            .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(4, true))
        .unwrap();

    assert_eq!(report.changed_atoms, 0);
    assert_eq!(kernel.version(atom).unwrap(), 0);
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn failed_late_reaction_does_not_publish_earlier_candidate_state() {
    let mut kernel = AtomicKernel::new();
    let domain = kernel.create_domain("reaction-domain").unwrap();
    let other = kernel.create_domain("other-domain").unwrap();
    let first_atom = kernel.create_atom_owned(domain, 0_i64).unwrap();
    let second_atom = kernel.create_atom_owned(domain, 0_i64).unwrap();
    let trigger = ReactionTrigger::input(key_selector(9));
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new("first", domain, trigger, state_action("first"))
                .step(ReactionStep::set(first_atom, ReactionValue::literal(1_i64))),
        )
        .unwrap();
    reactions
        .register(
            &kernel,
            ReactionSpec::new("second", domain, trigger, state_action("second")).step(
                ReactionStep::set(second_atom, ReactionValue::literal(1_i64)),
            ),
        )
        .unwrap();

    kernel.transfer_atom(second_atom, domain, other).unwrap();

    let result = reactions.apply_input_batch(&mut kernel, &key_batch(9, true));
    assert!(matches!(
        result,
        Err(ReactionError::Atomic(
            AtomicError::OwnershipViolation { .. }
        ))
    ));
    assert_eq!(kernel.get(first_atom).unwrap(), &Value::Int(0));
    assert_eq!(kernel.get(second_atom).unwrap(), &Value::Int(0));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn state_write_must_be_declared_by_action() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    let result = reactions.register(
        &kernel,
        ReactionSpec::new(
            "invalid",
            root,
            ReactionTrigger::input(key_selector(1)),
            ActionSpec::pure("pure"),
        )
        .step(ReactionStep::set(atom, ReactionValue::InputValue)),
    );

    assert_eq!(
        result,
        Err(ReactionError::Atomic(AtomicError::UndeclaredEffect(
            Effect::StateWrite
        )))
    );
}

#[test]
fn external_effect_intent_requires_declaration_and_authority() {
    let kernel = AtomicKernel::new();
    let root = kernel.root_domain();
    let trigger = ReactionTrigger::input(key_selector(1));
    let effect = Effect::Network("api.example".into());
    let mut reactions = AtomicReactionCore::new();

    let undeclared = reactions.register(
        &kernel,
        ReactionSpec::new("net", root, trigger, ActionSpec::pure("net-action"))
            .with_authority({
                let mut authority = CapabilitySet::new();
                authority.allow(Capability::Network("api.example".into()));
                authority
            })
            .step(ReactionStep::emit(effect.clone())),
    );
    assert_eq!(
        undeclared,
        Err(ReactionError::Atomic(AtomicError::UndeclaredEffect(
            effect.clone()
        )))
    );

    let denied = reactions.register(
        &kernel,
        ReactionSpec::new(
            "net",
            root,
            trigger,
            ActionSpec::new("net-action").declare(effect.clone()),
        )
        .step(ReactionStep::emit(effect.clone())),
    );
    assert_eq!(
        denied,
        Err(ReactionError::Atomic(AtomicError::CapabilityDenied(
            Capability::Network("api.example".into())
        )))
    );
}

#[test]
fn authorized_external_effect_is_emitted_as_intent_only() {
    let mut kernel = AtomicKernel::new();
    let root = kernel.root_domain();
    let effect = Effect::Network("api.example".into());
    let mut authority = CapabilitySet::new();
    authority.allow(Capability::Network("api.example".into()));
    let mut reactions = AtomicReactionCore::new();

    let id = reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "net",
                root,
                ReactionTrigger::input(key_selector(1)),
                ActionSpec::new("net-action").declare(effect.clone()),
            )
            .with_authority(authority)
            .step(ReactionStep::emit(effect.clone())),
        )
        .unwrap();

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(1, true))
        .unwrap();

    assert_eq!(report.transactions, 0);
    assert_eq!(kernel.pending_work(), 0);
    assert_eq!(report.effect_intents.len(), 1);
    assert_eq!(report.effect_intents[0].reaction, id);
    assert_eq!(report.effect_intents[0].effect, effect);
}

#[test]
fn timer_fires_are_canonicalized_before_reaction_execution() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "timer",
                root,
                ReactionTrigger::timer(TimerSelector::any()),
                state_action("timer-occurrence"),
            )
            .step(ReactionStep::set(atom, ReactionValue::TimerOccurrence)),
        )
        .unwrap();

    let fires = [
        TimerFire {
            timer: TimerId(2),
            deadline: LogicalTime(20),
            occurrence: 2,
        },
        TimerFire {
            timer: TimerId(1),
            deadline: LogicalTime(10),
            occurrence: 1,
        },
    ];

    let report = reactions.apply_timer_fires(&mut kernel, &fires).unwrap();
    assert_eq!(report.causes, 2);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Int(2));
}

#[test]
fn timer_deadline_and_occurrence_can_be_projected() {
    let mut kernel = AtomicKernel::new();
    let occurrence = kernel.create_atom(0_i64);
    let deadline = kernel.create_atom(0_i64);
    let root = kernel.root_domain();
    let timer = TimerId(8);
    let mut reactions = AtomicReactionCore::new();

    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "timer",
                root,
                ReactionTrigger::timer(TimerSelector::timer(timer)),
                state_action("timer-values"),
            )
            .step(ReactionStep::set(
                occurrence,
                ReactionValue::TimerOccurrence,
            ))
            .step(ReactionStep::set(
                deadline,
                ReactionValue::TimerDeadlineTicks,
            )),
        )
        .unwrap();

    reactions
        .apply_timer_fires(
            &mut kernel,
            &[TimerFire {
                timer,
                deadline: LogicalTime(55),
                occurrence: 3,
            }],
        )
        .unwrap();

    assert_eq!(kernel.get(occurrence).unwrap(), &Value::Int(3));
    assert_eq!(kernel.get(deadline).unwrap(), &Value::Int(55));
}

#[test]
fn incompatible_value_source_is_rejected_at_registration() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0_i64);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();

    let result = reactions.register(
        &kernel,
        ReactionSpec::new(
            "bad-source",
            root,
            ReactionTrigger::input(key_selector(1)),
            state_action("bad-source"),
        )
        .step(ReactionStep::set(atom, ReactionValue::TimerOccurrence)),
    );

    assert_eq!(result, Err(ReactionError::ValueSourceMismatch));
}

#[test]
fn public_non_monotonic_input_batch_is_rejected_before_publication() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let mut reactions = AtomicReactionCore::new();
    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "key",
                root,
                ReactionTrigger::input(key_selector(1)),
                state_action("key"),
            )
            .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();

    let event = |sequence| InputEvent {
        sequence: InputSequence(sequence),
        source: InputSource::Keyboard,
        device: InputDeviceId(1),
        target: InputTarget::Global,
        payload: InputPayload::Key {
            code: 1,
            pressed: true,
            repeat: false,
        },
    };
    let batch = InputBatch {
        events: vec![event(2), event(1)],
    };

    let result = reactions.apply_input_batch(&mut kernel, &batch);
    assert!(matches!(result, Err(ReactionError::Input(_))));
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn source_device_and_target_filtering_remain_exact() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let selector = InputSelector {
        source: Some(InputSource::Keyboard),
        device: Some(InputDeviceId(7)),
        target: Some(InputTarget::Global),
        signal: InputSignal::KeyPressed { code: 1 },
    };
    let mut reactions = AtomicReactionCore::new();
    reactions
        .register(
            &kernel,
            ReactionSpec::new(
                "exact",
                root,
                ReactionTrigger::input(selector),
                state_action("exact"),
            )
            .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(1, true))
        .unwrap();
    assert_eq!(report.matched_reactions, 0);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
}

#[test]
fn unregister_removes_future_reaction_work_without_reusing_identity() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let root = kernel.root_domain();
    let trigger = ReactionTrigger::input(key_selector(1));
    let mut reactions = AtomicReactionCore::new();

    let id = reactions
        .register(
            &kernel,
            ReactionSpec::new("one", root, trigger, state_action("one"))
                .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();
    assert!(reactions.unregister(id));
    assert!(!reactions.unregister(id));

    let next = reactions
        .register(
            &kernel,
            ReactionSpec::new("two", root, trigger, state_action("two"))
                .step(ReactionStep::set(atom, ReactionValue::InputValue)),
        )
        .unwrap();
    assert!(next > id);

    let report = reactions
        .apply_input_batch(&mut kernel, &key_batch(1, true))
        .unwrap();
    assert_eq!(report.matched_reactions, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(true));
}
