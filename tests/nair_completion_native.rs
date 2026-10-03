use nordoi_kernel::{
    execute_nair, AtomId, AtomSlot, AtomicEventLoop, AtomicKernel, CompletionSlot, DomainId,
    DomainRef, EffectCompletionError, EffectCompletionProjection, EffectCompletionProjectionValue,
    EffectCompletionSourceId, EffectDeliveryNamespace, EventLoopError, Instruction,
    NairCompletionAuthority, NairCompletionBinding, NairCompletionProjection,
    NairCompletionProjectionValue, NairError, NairProgram, NairReactionAuthority,
    PersistentAtomicRuntime, RegisterId, RuntimeError, Value, NAIR_FORMAT_MAJOR, NAIR_FORMAT_MINOR,
};

fn namespace(seed: u8) -> EffectDeliveryNamespace {
    EffectDeliveryNamespace::new([seed; 16])
}

fn native_program(projections: Vec<NairCompletionProjection>) -> NairProgram {
    NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Text("initial".into()),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "native-completion".into(),
            domain: DomainRef::Root,
            projections,
        },
        Instruction::Halt,
    ])
}

fn one_projection_program(value: NairCompletionProjectionValue) -> NairProgram {
    native_program(vec![NairCompletionProjection::new(AtomSlot(0), value)])
}

fn authority(source: u64, seed: u8) -> NairCompletionAuthority {
    let mut authority = NairCompletionAuthority::new();
    authority.bind(
        CompletionSlot(0),
        EffectCompletionSourceId(source),
        namespace(seed),
    );
    authority
}

#[test]
fn nair_0_6_is_current_and_0_5_programs_still_decode() {
    assert_eq!(NAIR_FORMAT_MAJOR, 0);
    assert_eq!(NAIR_FORMAT_MINOR, 6);

    let program = NairProgram::from_instructions(vec![Instruction::Halt]);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 5;
    bytes[7] = 0;

    let decoded = NairProgram::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, program);
    let reencoded = decoded.canonical_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([reencoded[6], reencoded[7]]), 6);
}

#[test]
fn completion_opcode_is_rejected_under_declared_nair_0_5() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let mut bytes = program.canonical_bytes().unwrap();
    bytes[6] = 5;
    bytes[7] = 0;

    assert_eq!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidOpcode(0x70))
    );
}

#[test]
fn native_completion_round_trip_is_byte_stable_for_all_projection_values() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(1),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::Const {
            dst: RegisterId(2),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(2),
            owner: DomainRef::Root,
            value: RegisterId(2),
        },
        Instruction::Const {
            dst: RegisterId(3),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(3),
            owner: DomainRef::Root,
            value: RegisterId(3),
        },
        Instruction::Const {
            dst: RegisterId(4),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(4),
            owner: DomainRef::Root,
            value: RegisterId(4),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(4),
            name: "all-values".into(),
            domain: DomainRef::Root,
            projections: vec![
                NairCompletionProjection::new(
                    AtomSlot(0),
                    NairCompletionProjectionValue::OutcomeValue,
                ),
                NairCompletionProjection::new(
                    AtomSlot(1),
                    NairCompletionProjectionValue::Succeeded,
                ),
                NairCompletionProjection::new(AtomSlot(2), NairCompletionProjectionValue::IntentId),
                NairCompletionProjection::new(
                    AtomSlot(3),
                    NairCompletionProjectionValue::AttemptId,
                ),
                NairCompletionProjection::new(
                    AtomSlot(4),
                    NairCompletionProjectionValue::SourceSequence,
                ),
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
fn completion_slots_are_single_assignment() {
    let first = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let declaration = first.instructions()[2].clone();
    let program = NairProgram::from_instructions(vec![
        first.instructions()[0].clone(),
        first.instructions()[1].clone(),
        declaration.clone(),
        declaration,
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateCompletionSlot(CompletionSlot(0)))
    );
}

#[test]
fn completion_name_must_not_be_empty() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "   ".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::EmptyCompletionName(CompletionSlot(0)))
    );
}

#[test]
fn completion_requires_at_least_one_projection() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(2),
            name: "empty".into(),
            domain: DomainRef::Root,
            projections: vec![],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::EmptyCompletionProjections(CompletionSlot(2)))
    );
}

#[test]
fn completion_projection_atom_must_exist_before_declaration() {
    let program = NairProgram::from_instructions(vec![
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "bad-atom".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(9),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::Halt,
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::UnknownAtomSlot(AtomSlot(9)))
    );
}

#[test]
fn duplicate_projection_atom_inside_one_native_route_is_rejected() {
    let program = native_program(vec![
        NairCompletionProjection::new(AtomSlot(0), NairCompletionProjectionValue::OutcomeValue),
        NairCompletionProjection::new(AtomSlot(0), NairCompletionProjectionValue::Succeeded),
    ]);

    assert_eq!(
        program.validate(),
        Err(NairError::DuplicateCompletionProjectionAtom {
            slot: CompletionSlot(0),
            atom: AtomSlot(0),
        })
    );
}

#[test]
fn direct_nair_executor_rejects_native_completion_context_before_mutation() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let mut kernel = AtomicKernel::new();

    assert_eq!(
        execute_nair(&mut kernel, &program),
        Err(NairError::CompletionContextRequired)
    );
    assert!(kernel.get(AtomId(1)).is_err());
}

#[test]
fn persistent_runtime_rejects_native_completion_without_event_loop_bootstrap() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    assert!(matches!(
        PersistentAtomicRuntime::boot(&program),
        Err(RuntimeError::Nair(NairError::CompletionContextRequired))
    ));
}

#[test]
fn event_loop_requires_explicit_host_binding_for_native_completion_slot() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    assert!(matches!(
        AtomicEventLoop::boot(&program),
        Err(EventLoopError::Runtime(RuntimeError::Nair(
            NairError::CompletionBindingRequired(CompletionSlot(0))
        )))
    ));
}

#[test]
fn event_loop_boots_native_completion_with_exact_host_binding() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let completion_authority = authority(7, 3);
    let event_loop = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &completion_authority,
    )
    .unwrap();

    assert_eq!(event_loop.native_completion_count(), 1);
    assert_eq!(
        event_loop.native_completion_binding(CompletionSlot(0)),
        Some(NairCompletionBinding::new(
            EffectCompletionSourceId(7),
            namespace(3)
        ))
    );
}

#[test]
fn native_completion_binding_grants_exact_runtime_authority() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let completion_authority = authority(11, 4);
    let mut event_loop = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &completion_authority,
    )
    .unwrap();

    assert!(event_loop.revoke_effect_completion_source(EffectCompletionSourceId(11), namespace(4)));
    assert!(!event_loop.revoke_effect_completion_source(EffectCompletionSourceId(11), namespace(4)));
}

#[test]
fn native_completion_projection_is_registered_in_completion_core() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let completion_authority = authority(12, 5);
    let mut event_loop = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &completion_authority,
    )
    .unwrap();
    let atom = event_loop.snapshot().unwrap()[&AtomSlot(0)].id;

    assert_eq!(
        event_loop.register_effect_completion_projection(EffectCompletionProjection::new(
            EffectCompletionSourceId(12),
            namespace(5),
            DomainId(0),
            atom,
            EffectCompletionProjectionValue::OutcomeValue,
        )),
        Err(EffectCompletionError::DuplicateProjection {
            source: EffectCompletionSourceId(12),
            namespace: namespace(5),
            atom,
        })
    );
}

#[test]
fn zero_source_binding_fails_closed() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let completion_authority = authority(0, 6);

    assert!(matches!(
        AtomicEventLoop::boot_with_authorities(
            &program,
            &NairReactionAuthority::new(),
            &completion_authority,
        ),
        Err(EventLoopError::Runtime(RuntimeError::Nair(
            NairError::CompletionBindingRequired(CompletionSlot(0))
        )))
    ));
}

#[test]
fn host_binding_does_not_change_canonical_nair_bytes() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let before = program.canonical_bytes().unwrap();
    let _a = authority(1, 1);
    let _b = authority(99, 9);
    assert_eq!(program.canonical_bytes().unwrap(), before);
}

#[test]
fn different_native_projection_semantics_change_canonical_bytes() {
    let outcome = one_projection_program(NairCompletionProjectionValue::OutcomeValue)
        .canonical_bytes()
        .unwrap();
    let succeeded = one_projection_program(NairCompletionProjectionValue::Succeeded)
        .canonical_bytes()
        .unwrap();

    assert_ne!(outcome, succeeded);
}

#[test]
fn different_native_completion_names_change_canonical_bytes() {
    let first = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let mut second_instructions = first.instructions().to_vec();
    if let Instruction::DefineEffectCompletion { name, .. } = &mut second_instructions[2] {
        *name = "native-completion-v2".into();
    }
    let second = NairProgram::from_instructions(second_instructions);

    assert_ne!(
        first.canonical_bytes().unwrap(),
        second.canonical_bytes().unwrap()
    );
}

#[test]
fn invalid_completion_projection_tag_is_rejected() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let mut bytes = program.canonical_bytes().unwrap();
    let projection_tag = bytes
        .iter()
        .rposition(|byte| *byte == 0x00)
        .expect("program contains completion projection tag");
    bytes[projection_tag] = 0xfe;

    assert!(matches!(
        NairProgram::from_canonical_bytes(&bytes),
        Err(NairError::InvalidCompletionProjectionTag(0xfe))
    ));
}

#[test]
fn two_native_completion_slots_can_bind_to_distinct_routes() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(1),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "route-a".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(1),
            name: "route-b".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(1),
                NairCompletionProjectionValue::Succeeded,
            )],
        },
        Instruction::Halt,
    ]);
    let mut completion_authority = NairCompletionAuthority::new();
    completion_authority.bind(CompletionSlot(0), EffectCompletionSourceId(1), namespace(1));
    completion_authority.bind(CompletionSlot(1), EffectCompletionSourceId(2), namespace(2));

    let event_loop = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &completion_authority,
    )
    .unwrap();
    assert_eq!(event_loop.native_completion_count(), 2);
}

#[test]
fn different_host_bindings_do_not_change_initial_replay_identity() {
    let program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let first = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &authority(1, 1),
    )
    .unwrap();
    let second = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &authority(99, 9),
    )
    .unwrap();

    assert_eq!(first.replay_key(), second.replay_key());
}

#[test]
fn different_native_completion_declarations_change_initial_replay_identity() {
    let outcome_program = one_projection_program(NairCompletionProjectionValue::OutcomeValue);
    let succeeded_program = one_projection_program(NairCompletionProjectionValue::Succeeded);
    let binding = authority(1, 1);

    let outcome = AtomicEventLoop::boot_with_authorities(
        &outcome_program,
        &NairReactionAuthority::new(),
        &binding,
    )
    .unwrap();
    let succeeded = AtomicEventLoop::boot_with_authorities(
        &succeeded_program,
        &NairReactionAuthority::new(),
        &binding,
    )
    .unwrap();

    assert_ne!(outcome.replay_key(), succeeded.replay_key());
}

#[test]
fn two_native_slots_may_share_one_route_when_atoms_are_distinct() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::Const {
            dst: RegisterId(1),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(1),
            owner: DomainRef::Root,
            value: RegisterId(1),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "first".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(1),
            name: "second".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(1),
                NairCompletionProjectionValue::Succeeded,
            )],
        },
        Instruction::Halt,
    ]);
    let mut completion_authority = NairCompletionAuthority::new();
    let route = NairCompletionBinding::new(EffectCompletionSourceId(3), namespace(8));
    completion_authority.set(CompletionSlot(0), route);
    completion_authority.set(CompletionSlot(1), route);

    let event_loop = AtomicEventLoop::boot_with_authorities(
        &program,
        &NairReactionAuthority::new(),
        &completion_authority,
    )
    .unwrap();
    assert_eq!(event_loop.native_completion_count(), 2);
}

#[test]
fn two_native_slots_cannot_resolve_same_route_to_same_runtime_atom() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "first".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(1),
            name: "second".into(),
            domain: DomainRef::Root,
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::Succeeded,
            )],
        },
        Instruction::Halt,
    ]);
    let mut completion_authority = NairCompletionAuthority::new();
    let route = NairCompletionBinding::new(EffectCompletionSourceId(3), namespace(8));
    completion_authority.set(CompletionSlot(0), route);
    completion_authority.set(CompletionSlot(1), route);

    assert!(matches!(
        AtomicEventLoop::boot_with_authorities(
            &program,
            &NairReactionAuthority::new(),
            &completion_authority,
        ),
        Err(EventLoopError::Runtime(RuntimeError::Nair(
            NairError::Completion(EffectCompletionError::DuplicateProjection { .. })
        )))
    ));
}

#[test]
fn native_completion_bootstrap_revalidates_atom_ownership() {
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: RegisterId(0),
            value: Value::Null,
        },
        Instruction::CreateDomain {
            dst: nordoi_kernel::DomainSlot(0),
            name: "other".into(),
        },
        Instruction::CreateAtom {
            dst: AtomSlot(0),
            owner: DomainRef::Root,
            value: RegisterId(0),
        },
        Instruction::DefineEffectCompletion {
            dst: CompletionSlot(0),
            name: "wrong-owner".into(),
            domain: DomainRef::Slot(nordoi_kernel::DomainSlot(0)),
            projections: vec![NairCompletionProjection::new(
                AtomSlot(0),
                NairCompletionProjectionValue::OutcomeValue,
            )],
        },
        Instruction::Halt,
    ]);

    assert!(matches!(
        AtomicEventLoop::boot_with_authorities(
            &program,
            &NairReactionAuthority::new(),
            &authority(5, 5),
        ),
        Err(EventLoopError::Runtime(RuntimeError::Nair(
            NairError::Completion(EffectCompletionError::Atomic(_))
        )))
    ));
}
