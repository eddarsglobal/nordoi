use nordoi_kernel::{
    execute_nair, AtomSlot, AtomicError, AtomicEventLoop, AtomicInputCore, AtomicKernel,
    Capability, CapabilitySet, DomainRef, Effect, EventLoopError, InputDeviceId, InputPayload,
    InputSignal, InputSource, InputTargetRef, Instruction, LogicalTime, NairEffectSet, NairError,
    NairProgram, NairReactionAuthority, NairReactionStep, NairReactionTrigger, NairReactionValue,
    ReactionError, ReactionSlot, RegisterId, RuntimeError, TimerSlot, Value, NAIR_FORMAT_MAJOR,
    NAIR_FORMAT_MINOR,
};

fn effects(effects: impl IntoIterator<Item = Effect>) -> NairEffectSet {
    NairEffectSet::from_effects(effects)
}

fn keyboard_batch(code: u32, pressed: bool) -> nordoi_kernel::InputBatch {
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

fn input_reaction_program(code: u32) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "keyboard-state".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code },
            },
            action_name: "set-key-state".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::InputValue,
            }],
        },
        Instruction::Halt,
    ])
}

#[test]
fn nair_0_5_is_current_and_0_4_programs_still_decode() {
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 5);

    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 4;
    bytes[7] = 0;

    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    let reencoded = decoded.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([reencoded[6], reencoded[7]]), 5);
}

#[test]
fn reaction_opcode_is_rejected_under_declared_nair_0_4() {
    let program = input_reaction_program(32);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 4;
    bytes[7] = 0;

    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x60))
    );
}

#[test]
fn reaction_slots_are_single_assignment() {
    let mut program = input_reaction_program(32);
    let duplicate = program.instructions()[2].clone();
    let mut instructions = program.instructions()[..3].to_vec();
    instructions.push(duplicate);
    instructions.push(Instruction::Halt);
    program = NairProgram::from_instructions(instructions);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateReactionSlot(ReactionSlot(0)))
    );
}

#[test]
fn reaction_set_atom_must_exist_before_definition() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "bad".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: None,
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::PointerX,
            },
            action_name: "bad-write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(9),
                value: NairReactionValue::InputValue,
            }],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownAtomSlot(AtomSlot(9)))
    );
}

#[test]
fn timer_reaction_requires_existing_timer_slot() {
    let program = NairProgram::from_instructions(vec![
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
            name: "timer".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(7)),
                occurrence: None,
            },
            action_name: "timer-write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::TimerOccurrence,
            }],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownTimerSlot(TimerSlot(7)))
    );
}

#[test]
fn reaction_value_source_must_match_trigger() {
    let program = NairProgram::from_instructions(vec![
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
            name: "bad-projection".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: None,
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::PointerX,
            },
            action_name: "write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::TimerOccurrence,
            }],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::ReactionValueSourceMismatch(ReactionSlot(0)))
    );
}

#[test]
fn reaction_state_write_must_be_declared() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "undeclared".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: None,
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 32 },
            },
            action_name: "write".into(),
            declared_effects: NairEffectSet::new(),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::InputValue,
            }],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::ReactionUndeclaredEffect {
            slot: ReactionSlot(0),
            effect: Effect::StateWrite,
        })
    );
}

#[test]
fn native_reaction_program_canonical_round_trip_is_byte_stable() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(9),
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
            name: "timer-network-intent".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: Some(1),
            },
            action_name: "project-and-intend".into(),
            declared_effects: effects([
                Effect::StateWrite,
                Effect::Network("api.example.test".into()),
            ]),
            steps: vec![
                NairReactionStep::Set {
                    atom: AtomSlot(0),
                    value: NairReactionValue::TimerDeadlineTicks,
                },
                NairReactionStep::EmitEffect {
                    effect: Effect::Network("api.example.test".into()),
                },
            ],
        },
        Instruction::Halt,
    ]);

    let bytes = program.canonical_bytes().unwrap();
    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    assert_eq!(decoded.canonical_bytes().unwrap(), bytes);
}

#[test]
fn legacy_executor_rejects_native_reaction_before_execution() {
    let program = input_reaction_program(32);
    let mut kernel = AtomicKernel::new();

    assert_eq!(
        execute_nair(&mut kernel, &program),
        Err(NairError::ReactionContextRequired)
    );
    assert!(kernel.get(nordoi_kernel::AtomId(1)).is_err());
}

#[test]
fn event_loop_boots_and_executes_native_input_reaction() {
    let program = input_reaction_program(32);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();

    assert_eq!(loop_.native_reaction_count(), 1);
    assert_eq!(
        loop_.native_reaction_id(ReactionSlot(0)).unwrap().value(),
        1
    );

    let report = loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(32, true))
        .unwrap();

    assert_eq!(report.reactions.input.matched_reactions, 1);
    assert_eq!(report.reactions.input.changed_atoms, 1);
    assert_eq!(report.reactions.timers.matched_reactions, 0);
    assert_eq!(
        loop_.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(true)
    );
}

#[test]
fn native_timer_reaction_projects_occurrence_and_deadline() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(12),
        },
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Int(0),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Int(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(1),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "timer-projection".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: None,
            },
            action_name: "project".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![
                NairReactionStep::Set {
                    atom: AtomSlot(0),
                    value: NairReactionValue::TimerOccurrence,
                },
                NairReactionStep::Set {
                    atom: AtomSlot(1),
                    value: NairReactionValue::TimerDeadlineTicks,
                },
            ],
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();
    let report = loop_
        .cycle_to(LogicalTime(12), &Default::default())
        .unwrap();
    let snapshot = loop_.snapshot().unwrap();

    assert_eq!(report.reactions.timers.matched_reactions, 1);
    assert_eq!(snapshot[&AtomSlot(0)].value, Value::Int(1));
    assert_eq!(snapshot[&AtomSlot(1)].value, Value::Int(12));
}

#[test]
fn native_reaction_ids_follow_program_definition_order() {
    let mut instructions = input_reaction_program(1).instructions().to_vec();
    instructions.pop();
    instructions.push(Instruction::DefineReaction {
        dst: ReactionSlot(99),
        name: "second".into(),
        domain: DomainRef::Root,
        trigger: NairReactionTrigger::Input {
            source: None,
            device: None,
            target: InputTargetRef::Any,
            signal: InputSignal::KeyPressed { code: 2 },
        },
        action_name: "second-action".into(),
        declared_effects: effects([Effect::StateWrite]),
        steps: vec![NairReactionStep::Set {
            atom: AtomSlot(0),
            value: NairReactionValue::InputValue,
        }],
    });
    instructions.push(Instruction::Halt);
    let loop_ = AtomicEventLoop::boot(&NairProgram::from_instructions(instructions)).unwrap();

    assert_eq!(
        loop_.native_reaction_id(ReactionSlot(0)).unwrap().value(),
        1
    );
    assert_eq!(
        loop_.native_reaction_id(ReactionSlot(99)).unwrap().value(),
        2
    );
}

#[test]
fn privileged_effect_is_denied_without_host_authority() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "network".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: None,
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "network-intent".into(),
            declared_effects: effects([Effect::Network("api.example.test".into())]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: Effect::Network("api.example.test".into()),
            }],
        },
        Instruction::Halt,
    ]);

    assert!(matches!(
        AtomicEventLoop::boot(&program),
        Err(EventLoopError::Runtime(RuntimeError::Nair(NairError::Reaction(
            ReactionError::Atomic(AtomicError::CapabilityDenied(Capability::Network(scope)))
        )))) if scope == "api.example.test"
    ));
}

#[test]
fn host_authority_allows_validated_intent_without_executing_effect() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "network".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 7 },
            },
            action_name: "network-intent".into(),
            declared_effects: effects([Effect::Network("api.example.test".into())]),
            steps: vec![NairReactionStep::EmitEffect {
                effect: Effect::Network("api.example.test".into()),
            }],
        },
        Instruction::Halt,
    ]);
    let mut capability = CapabilitySet::new();
    capability.allow(Capability::Network("api.example.test".into()));
    let mut authority = NairReactionAuthority::new();
    authority.set(ReactionSlot(0), capability);

    let mut loop_ = AtomicEventLoop::boot_with_reaction_authority(&program, &authority).unwrap();
    let report = loop_
        .cycle_to(LogicalTime::ZERO, &keyboard_batch(7, true))
        .unwrap();

    assert_eq!(report.reactions.input.effect_intents.len(), 1);
    assert_eq!(
        report.reactions.input.effect_intents[0].effect,
        Effect::Network("api.example.test".into())
    );
    assert!(loop_.snapshot().unwrap().is_empty());
}

#[test]
fn input_reactions_precede_timer_reactions_within_one_cycle() {
    let program = NairProgram::from_instructions(vec![
        Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(0),
            deadline: LogicalTime(5),
        },
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Bool(false),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(0),
            name: "input-first".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Input {
                source: Some(InputSource::Keyboard),
                device: None,
                target: InputTargetRef::Any,
                signal: InputSignal::KeyPressed { code: 5 },
            },
            action_name: "input-write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::InputValue,
            }],
        },
        Instruction::DefineReaction {
            dst: ReactionSlot(1),
            name: "timer-second".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: None,
            },
            action_name: "timer-write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::Literal(Value::Bool(false)),
            }],
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();

    let report = loop_
        .cycle_to(LogicalTime(5), &keyboard_batch(5, true))
        .unwrap();

    assert_eq!(report.reactions.input.matched_reactions, 1);
    assert_eq!(report.reactions.timers.matched_reactions, 1);
    assert_eq!(
        loop_.snapshot().unwrap()[&AtomSlot(0)].value,
        Value::Bool(false)
    );
}

#[test]
fn reaction_failure_keeps_time_and_runtime_unpublished() {
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
            name: "overflow".into(),
            domain: DomainRef::Root,
            trigger: NairReactionTrigger::Timer {
                timer: Some(TimerSlot(0)),
                occurrence: None,
            },
            action_name: "overflow-write".into(),
            declared_effects: effects([Effect::StateWrite]),
            steps: vec![NairReactionStep::Set {
                atom: AtomSlot(0),
                value: NairReactionValue::TimerDeadlineTicks,
            }],
        },
        Instruction::Halt,
    ]);
    let mut loop_ = AtomicEventLoop::boot(&program).unwrap();
    let before = loop_.replay_key();

    assert!(matches!(
        loop_.cycle_to(LogicalTime(u64::MAX), &Default::default()),
        Err(EventLoopError::Runtime(RuntimeError::Reaction(
            ReactionError::IntegerProjectionOverflow(u64::MAX)
        )))
    ));
    assert_eq!(loop_.logical_time(), LogicalTime::ZERO);
    assert_eq!(loop_.cycle_index(), 0);
    assert_eq!(loop_.replay_key(), before);
    assert_eq!(loop_.snapshot().unwrap()[&AtomSlot(0)].value, Value::Int(0));
}
