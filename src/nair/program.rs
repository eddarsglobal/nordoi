use std::collections::BTreeSet;

use crate::{
    effect::Effect,
    input::{InputDeviceId, InputSignal, InputSource},
    render::{DirtyMask, RenderPrimitive, RenderSpace},
    time::{LogicalDuration, LogicalTime},
    value::Value,
};

use super::{
    completion::{NairCompletionProjection, NairCompletionProjectionValue},
    error::{NairError, NairResult},
    id::{
        AtomSlot, CompletionSlot, DomainSlot, InputBridgeSlot, ReactionSlot, RegisterId,
        RenderNodeSlot, TimerSlot, TransactionSlot,
    },
    instruction::{DomainRef, InputTargetRef, Instruction},
    reaction::{NairEffectSet, NairReactionStep, NairReactionTrigger, NairReactionValue},
};

pub const NAIR_MAGIC: [u8; 4] = *b"NAIR";
pub const NAIR_FORMAT_MAJOR: u16 = 0;
pub const NAIR_FORMAT_MINOR: u16 = 6;
pub const NAIR_MIN_SUPPORTED_MINOR: u16 = 1;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NairProgram {
    instructions: Vec<Instruction>,
}

impl NairProgram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_instructions(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn validate(&self) -> NairResult<()> {
        let mut registers = BTreeSet::new();
        let mut domains = BTreeSet::new();
        let mut atoms = BTreeSet::new();
        let mut transaction_slots = BTreeSet::new();
        let mut active_transactions = BTreeSet::new();
        let mut render_nodes = BTreeSet::new();
        let mut input_bridges = BTreeSet::new();
        let mut timer_slots = BTreeSet::new();
        let mut reaction_slots = BTreeSet::new();
        let mut completion_slots = BTreeSet::new();
        let mut halt_seen = false;

        for (index, instruction) in self.instructions.iter().enumerate() {
            if halt_seen {
                return Err(NairError::InstructionAfterHalt { index });
            }

            match instruction {
                Instruction::Const { dst, value } => {
                    if !registers.insert(*dst) {
                        return Err(NairError::DuplicateRegister(*dst));
                    }
                    if matches!(value, Value::Float(value) if !value.is_finite()) {
                        return Err(NairError::NonFiniteFloat(*dst));
                    }
                }
                Instruction::CreateDomain { dst, name } => {
                    if !domains.insert(*dst) {
                        return Err(NairError::DuplicateDomainSlot(*dst));
                    }
                    if name.trim().is_empty() {
                        return Err(NairError::EmptyDomainName(*dst));
                    }
                }
                Instruction::CreateAtom { dst, owner, value } => {
                    validate_domain_ref(*owner, &domains)?;
                    require_register(*value, &registers)?;
                    if !atoms.insert(*dst) {
                        return Err(NairError::DuplicateAtomSlot(*dst));
                    }
                }
                Instruction::Connect { source, dependent } => {
                    require_atom(*source, &atoms)?;
                    require_atom(*dependent, &atoms)?;
                    if source == dependent {
                        return Err(NairError::SelfDependency(*source));
                    }
                }
                Instruction::BeginTransaction { dst, domain } => {
                    validate_domain_ref(*domain, &domains)?;
                    if !transaction_slots.insert(*dst) {
                        return Err(NairError::DuplicateTransactionSlot(*dst));
                    }
                    active_transactions.insert(*dst);
                }
                Instruction::TxSet { tx, atom, value } => {
                    require_active_transaction(*tx, &active_transactions)?;
                    require_atom(*atom, &atoms)?;
                    require_register(*value, &registers)?;
                }
                Instruction::Commit { tx } | Instruction::Rollback { tx } => {
                    require_active_transaction(*tx, &active_transactions)?;
                    active_transactions.remove(tx);
                }
                Instruction::CreateRenderNode { dst, .. } => {
                    if !render_nodes.insert(*dst) {
                        return Err(NairError::DuplicateRenderNodeSlot(*dst));
                    }
                }
                Instruction::CreateRenderChild { dst, parent, .. } => {
                    require_render_node(*parent, &render_nodes)?;
                    if !render_nodes.insert(*dst) {
                        return Err(NairError::DuplicateRenderNodeSlot(*dst));
                    }
                }
                Instruction::BindRenderAtom { atom, node, dirty } => {
                    require_atom(*atom, &atoms)?;
                    require_render_node(*node, &render_nodes)?;
                    if dirty.is_empty() {
                        return Err(NairError::EmptyRenderDirtyMask);
                    }
                }
                Instruction::SetRenderVisible { node, .. } => {
                    require_render_node(*node, &render_nodes)?;
                }
                Instruction::SetRenderOpacity { node, opacity } => {
                    require_render_node(*node, &render_nodes)?;
                    if !opacity.is_finite() || !(0.0..=1.0).contains(opacity) {
                        return Err(NairError::InvalidRenderOpacity(*opacity));
                    }
                }
                Instruction::SetRenderPosition { node, position } => {
                    require_render_node(*node, &render_nodes)?;
                    if position.iter().any(|value| !value.is_finite()) {
                        return Err(NairError::NonFiniteRenderPosition(*node));
                    }
                }
                Instruction::RenderFlush => {}
                Instruction::CreateInputBridge { dst, domain } => {
                    validate_domain_ref(*domain, &domains)?;
                    if !input_bridges.insert(*dst) {
                        return Err(NairError::DuplicateInputBridgeSlot(*dst));
                    }
                }
                Instruction::BindInputAtom {
                    bridge,
                    atom,
                    target,
                    ..
                } => {
                    require_input_bridge(*bridge, &input_bridges)?;
                    require_atom(*atom, &atoms)?;
                    if let InputTargetRef::RenderNode(node) = target {
                        require_render_node(*node, &render_nodes)?;
                    }
                }
                Instruction::ApplyInput { bridge } => {
                    require_input_bridge(*bridge, &input_bridges)?;
                }
                Instruction::ScheduleTimerOnceAt { dst, .. } => {
                    if !timer_slots.insert(*dst) {
                        return Err(NairError::DuplicateTimerSlot(*dst));
                    }
                }
                Instruction::ScheduleTimerRepeatingAt { dst, interval, .. } => {
                    if !timer_slots.insert(*dst) {
                        return Err(NairError::DuplicateTimerSlot(*dst));
                    }
                    if interval.is_zero() {
                        return Err(NairError::ZeroTimerInterval(*dst));
                    }
                }
                Instruction::CancelTimer { timer } => {
                    require_timer_slot(*timer, &timer_slots)?;
                }
                Instruction::DefineEffectCompletion {
                    dst,
                    name,
                    domain,
                    projections,
                } => {
                    validate_domain_ref(*domain, &domains)?;
                    if !completion_slots.insert(*dst) {
                        return Err(NairError::DuplicateCompletionSlot(*dst));
                    }
                    if name.trim().is_empty() {
                        return Err(NairError::EmptyCompletionName(*dst));
                    }
                    if projections.is_empty() {
                        return Err(NairError::EmptyCompletionProjections(*dst));
                    }
                    let mut projected_atoms = BTreeSet::new();
                    for projection in projections {
                        require_atom(projection.atom, &atoms)?;
                        if !projected_atoms.insert(projection.atom) {
                            return Err(NairError::DuplicateCompletionProjectionAtom {
                                slot: *dst,
                                atom: projection.atom,
                            });
                        }
                    }
                }
                Instruction::DefineReaction {
                    dst,
                    name,
                    domain,
                    trigger,
                    action_name,
                    declared_effects,
                    steps,
                } => {
                    validate_domain_ref(*domain, &domains)?;
                    if !reaction_slots.insert(*dst) {
                        return Err(NairError::DuplicateReactionSlot(*dst));
                    }
                    if name.trim().is_empty() {
                        return Err(NairError::EmptyReactionName(*dst));
                    }
                    if action_name.trim().is_empty() {
                        return Err(NairError::EmptyReactionActionName(*dst));
                    }
                    if steps.is_empty() {
                        return Err(NairError::EmptyReactionSteps(*dst));
                    }
                    validate_reaction_trigger(*dst, *trigger, &render_nodes, &timer_slots)?;
                    validate_reaction_steps(*dst, *trigger, declared_effects, steps, &atoms)?;
                }
                Instruction::Halt => {
                    halt_seen = true;
                }
            }
        }

        if !halt_seen {
            return Err(NairError::MissingHalt);
        }

        if !active_transactions.is_empty() {
            return Err(NairError::UnclosedTransactions(
                active_transactions.into_iter().collect(),
            ));
        }

        Ok(())
    }

    pub fn canonical_bytes(&self) -> NairResult<Vec<u8>> {
        self.validate()?;

        let mut out = Vec::new();
        out.extend_from_slice(&NAIR_MAGIC);
        write_u16(&mut out, NAIR_FORMAT_MAJOR);
        write_u16(&mut out, NAIR_FORMAT_MINOR);
        write_len(&mut out, self.instructions.len())?;

        for instruction in &self.instructions {
            encode_instruction(&mut out, instruction)?;
        }

        Ok(out)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> NairResult<Self> {
        let mut input = Decoder::new(bytes);

        let magic = input.read_exact(4)?;
        if magic != NAIR_MAGIC {
            return Err(NairError::InvalidMagic);
        }

        let major = input.read_u16()?;
        let minor = input.read_u16()?;
        if major != NAIR_FORMAT_MAJOR
            || !(NAIR_MIN_SUPPORTED_MINOR..=NAIR_FORMAT_MINOR).contains(&minor)
        {
            return Err(NairError::UnsupportedFormat { major, minor });
        }

        let count = input.read_u32()? as usize;
        let mut instructions = Vec::with_capacity(count);
        for _ in 0..count {
            instructions.push(decode_instruction(&mut input, minor)?);
        }

        if input.remaining() != 0 {
            return Err(NairError::TrailingBytes(input.remaining()));
        }

        let program = Self::from_instructions(instructions);
        program.validate()?;
        Ok(program)
    }
}

fn validate_reaction_trigger(
    _slot: ReactionSlot,
    trigger: NairReactionTrigger,
    render_nodes: &BTreeSet<RenderNodeSlot>,
    timer_slots: &BTreeSet<TimerSlot>,
) -> NairResult<()> {
    match trigger {
        NairReactionTrigger::Input { target, .. } => {
            if let InputTargetRef::RenderNode(node) = target {
                require_render_node(node, render_nodes)?;
            }
        }
        NairReactionTrigger::Timer { timer, .. } => {
            if let Some(timer) = timer {
                require_timer_slot(timer, timer_slots)?;
            }
        }
    }
    Ok(())
}

fn validate_reaction_steps(
    slot: ReactionSlot,
    trigger: NairReactionTrigger,
    declared_effects: &NairEffectSet,
    steps: &[NairReactionStep],
    atoms: &BTreeSet<AtomSlot>,
) -> NairResult<()> {
    for step in steps {
        match step {
            NairReactionStep::Set { atom, value } => {
                require_atom(*atom, atoms)?;
                if !declared_effects.contains(&Effect::StateWrite) {
                    return Err(NairError::ReactionUndeclaredEffect {
                        slot,
                        effect: Effect::StateWrite,
                    });
                }
                match value {
                    NairReactionValue::Literal(Value::Float(value)) if !value.is_finite() => {
                        return Err(NairError::NonFiniteReactionLiteral(slot));
                    }
                    NairReactionValue::Literal(_) => {}
                    NairReactionValue::InputValue
                        if !matches!(trigger, NairReactionTrigger::Input { .. }) =>
                    {
                        return Err(NairError::ReactionValueSourceMismatch(slot));
                    }
                    NairReactionValue::TimerOccurrence | NairReactionValue::TimerDeadlineTicks
                        if !matches!(trigger, NairReactionTrigger::Timer { .. }) =>
                    {
                        return Err(NairError::ReactionValueSourceMismatch(slot));
                    }
                    _ => {}
                }
            }
            NairReactionStep::EmitEffect { effect } => {
                if !declared_effects.contains(effect) {
                    return Err(NairError::ReactionUndeclaredEffect {
                        slot,
                        effect: effect.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_domain_ref(domain: DomainRef, domains: &BTreeSet<DomainSlot>) -> NairResult<()> {
    match domain {
        DomainRef::Root => Ok(()),
        DomainRef::Slot(slot) if domains.contains(&slot) => Ok(()),
        DomainRef::Slot(slot) => Err(NairError::UnknownDomainSlot(slot)),
    }
}

fn require_register(id: RegisterId, registers: &BTreeSet<RegisterId>) -> NairResult<()> {
    if registers.contains(&id) {
        Ok(())
    } else {
        Err(NairError::UnknownRegister(id))
    }
}

fn require_atom(id: AtomSlot, atoms: &BTreeSet<AtomSlot>) -> NairResult<()> {
    if atoms.contains(&id) {
        Ok(())
    } else {
        Err(NairError::UnknownAtomSlot(id))
    }
}

fn require_render_node(id: RenderNodeSlot, nodes: &BTreeSet<RenderNodeSlot>) -> NairResult<()> {
    if nodes.contains(&id) {
        Ok(())
    } else {
        Err(NairError::UnknownRenderNodeSlot(id))
    }
}

fn require_active_transaction(
    id: TransactionSlot,
    active: &BTreeSet<TransactionSlot>,
) -> NairResult<()> {
    if active.contains(&id) {
        Ok(())
    } else {
        Err(NairError::InactiveTransaction(id))
    }
}

fn require_input_bridge(
    id: InputBridgeSlot,
    bridges: &BTreeSet<InputBridgeSlot>,
) -> NairResult<()> {
    if bridges.contains(&id) {
        Ok(())
    } else {
        Err(NairError::UnknownInputBridgeSlot(id))
    }
}

fn require_timer_slot(id: TimerSlot, timers: &BTreeSet<TimerSlot>) -> NairResult<()> {
    if timers.contains(&id) {
        Ok(())
    } else {
        Err(NairError::UnknownTimerSlot(id))
    }
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(out: &mut Vec<u8>, value: f32) {
    write_u32(out, value.to_bits());
}

fn write_len(out: &mut Vec<u8>, len: usize) -> NairResult<()> {
    let len = u32::try_from(len).map_err(|_| NairError::LengthOverflow)?;
    write_u32(out, len);
    Ok(())
}

fn write_string(out: &mut Vec<u8>, value: &str) -> NairResult<()> {
    write_len(out, value.len())?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_value(out: &mut Vec<u8>, value: &Value) -> NairResult<()> {
    match value {
        Value::Null => out.push(0x00),
        Value::Bool(false) => out.push(0x01),
        Value::Bool(true) => out.push(0x02),
        Value::Int(value) => {
            out.push(0x03);
            write_i64(out, *value);
        }
        Value::Float(value) => {
            if !value.is_finite() {
                return Err(NairError::LengthOverflow);
            }
            out.push(0x04);
            write_u64(out, value.to_bits());
        }
        Value::Text(value) => {
            out.push(0x05);
            write_string(out, value)?;
        }
    }
    Ok(())
}

fn encode_domain_ref(out: &mut Vec<u8>, domain: DomainRef) {
    match domain {
        DomainRef::Root => out.push(0x00),
        DomainRef::Slot(slot) => {
            out.push(0x01);
            write_u32(out, slot.0);
        }
    }
}

fn encode_render_primitive(out: &mut Vec<u8>, primitive: RenderPrimitive) {
    out.push(match primitive {
        RenderPrimitive::Group => 0x00,
        RenderPrimitive::Quad => 0x01,
        RenderPrimitive::Text => 0x02,
        RenderPrimitive::Mesh => 0x03,
    });
}

fn encode_render_space(out: &mut Vec<u8>, space: RenderSpace) {
    out.push(match space {
        RenderSpace::Screen => 0x00,
        RenderSpace::World => 0x01,
    });
}

fn encode_input_source(out: &mut Vec<u8>, source: InputSource) {
    out.push(match source {
        InputSource::Keyboard => 0x00,
        InputSource::Mouse => 0x01,
        InputSource::Touch => 0x02,
        InputSource::Pen => 0x03,
        InputSource::Gamepad => 0x04,
        InputSource::XrController => 0x05,
        InputSource::XrHand => 0x06,
    });
}

fn encode_optional_input_source(out: &mut Vec<u8>, source: Option<InputSource>) {
    match source {
        None => out.push(0x00),
        Some(source) => {
            out.push(0x01);
            encode_input_source(out, source);
        }
    }
}

fn encode_optional_device(out: &mut Vec<u8>, device: Option<InputDeviceId>) {
    match device {
        None => out.push(0x00),
        Some(device) => {
            out.push(0x01);
            write_u64(out, device.0);
        }
    }
}

fn encode_input_target(out: &mut Vec<u8>, target: InputTargetRef) {
    match target {
        InputTargetRef::Any => out.push(0x00),
        InputTargetRef::Global => out.push(0x01),
        InputTargetRef::RenderNode(node) => {
            out.push(0x02);
            write_u32(out, node.0);
        }
    }
}

fn encode_input_signal(out: &mut Vec<u8>, signal: InputSignal) {
    match signal {
        InputSignal::KeyPressed { code } => {
            out.push(0x00);
            write_u32(out, code);
        }
        InputSignal::PointerX => out.push(0x01),
        InputSignal::PointerY => out.push(0x02),
        InputSignal::PointerButtonPressed { button } => {
            out.push(0x03);
            write_u16(out, button);
        }
        InputSignal::ButtonPressed { button } => {
            out.push(0x04);
            write_u16(out, button);
        }
        InputSignal::Axis { axis } => {
            out.push(0x05);
            write_u16(out, axis);
        }
        InputSignal::PoseX => out.push(0x06),
        InputSignal::PoseY => out.push(0x07),
        InputSignal::PoseZ => out.push(0x08),
    }
}

fn encode_effect(out: &mut Vec<u8>, effect: &Effect) -> NairResult<()> {
    match effect {
        Effect::Pure => out.push(0x00),
        Effect::StateRead => out.push(0x01),
        Effect::StateWrite => out.push(0x02),
        Effect::Network(scope) => {
            out.push(0x10);
            write_string(out, scope)?;
        }
        Effect::FileRead(scope) => {
            out.push(0x11);
            write_string(out, scope)?;
        }
        Effect::FileWrite(scope) => {
            out.push(0x12);
            write_string(out, scope)?;
        }
        Effect::Camera => out.push(0x20),
        Effect::Microphone => out.push(0x21),
        Effect::Location => out.push(0x22),
        Effect::Gpu => out.push(0x23),
        Effect::Xr => out.push(0x24),
        Effect::Process => out.push(0x25),
    }
    Ok(())
}

fn encode_optional_timer_slot(out: &mut Vec<u8>, timer: Option<TimerSlot>) {
    match timer {
        None => out.push(0x00),
        Some(timer) => {
            out.push(0x01);
            write_u32(out, timer.0);
        }
    }
}

fn encode_optional_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        None => out.push(0x00),
        Some(value) => {
            out.push(0x01);
            write_u64(out, value);
        }
    }
}

fn encode_reaction_trigger(out: &mut Vec<u8>, trigger: NairReactionTrigger) {
    match trigger {
        NairReactionTrigger::Input {
            source,
            device,
            target,
            signal,
        } => {
            out.push(0x00);
            encode_optional_input_source(out, source);
            encode_optional_device(out, device);
            encode_input_target(out, target);
            encode_input_signal(out, signal);
        }
        NairReactionTrigger::Timer { timer, occurrence } => {
            out.push(0x01);
            encode_optional_timer_slot(out, timer);
            encode_optional_u64(out, occurrence);
        }
    }
}

fn encode_reaction_value(out: &mut Vec<u8>, value: &NairReactionValue) -> NairResult<()> {
    match value {
        NairReactionValue::Literal(value) => {
            out.push(0x00);
            encode_value(out, value)?;
        }
        NairReactionValue::InputValue => out.push(0x01),
        NairReactionValue::TimerOccurrence => out.push(0x02),
        NairReactionValue::TimerDeadlineTicks => out.push(0x03),
    }
    Ok(())
}

fn encode_reaction_step(out: &mut Vec<u8>, step: &NairReactionStep) -> NairResult<()> {
    match step {
        NairReactionStep::Set { atom, value } => {
            out.push(0x00);
            write_u32(out, atom.0);
            encode_reaction_value(out, value)?;
        }
        NairReactionStep::EmitEffect { effect } => {
            out.push(0x01);
            encode_effect(out, effect)?;
        }
    }
    Ok(())
}

fn encode_completion_projection_value(out: &mut Vec<u8>, value: NairCompletionProjectionValue) {
    out.push(match value {
        NairCompletionProjectionValue::OutcomeValue => 0x00,
        NairCompletionProjectionValue::Succeeded => 0x01,
        NairCompletionProjectionValue::IntentId => 0x02,
        NairCompletionProjectionValue::AttemptId => 0x03,
        NairCompletionProjectionValue::SourceSequence => 0x04,
    });
}

fn encode_instruction(out: &mut Vec<u8>, instruction: &Instruction) -> NairResult<()> {
    match instruction {
        Instruction::Const { dst, value } => {
            out.push(0x01);
            write_u32(out, dst.0);
            encode_value(out, value)?;
        }
        Instruction::CreateDomain { dst, name } => {
            out.push(0x10);
            write_u32(out, dst.0);
            write_string(out, name)?;
        }
        Instruction::CreateAtom { dst, owner, value } => {
            out.push(0x11);
            write_u32(out, dst.0);
            encode_domain_ref(out, *owner);
            write_u32(out, value.0);
        }
        Instruction::Connect { source, dependent } => {
            out.push(0x12);
            write_u32(out, source.0);
            write_u32(out, dependent.0);
        }
        Instruction::BeginTransaction { dst, domain } => {
            out.push(0x20);
            write_u32(out, dst.0);
            encode_domain_ref(out, *domain);
        }
        Instruction::TxSet { tx, atom, value } => {
            out.push(0x21);
            write_u32(out, tx.0);
            write_u32(out, atom.0);
            write_u32(out, value.0);
        }
        Instruction::Commit { tx } => {
            out.push(0x22);
            write_u32(out, tx.0);
        }
        Instruction::Rollback { tx } => {
            out.push(0x23);
            write_u32(out, tx.0);
        }
        Instruction::CreateRenderNode {
            dst,
            primitive,
            space,
        } => {
            out.push(0x30);
            write_u32(out, dst.0);
            encode_render_primitive(out, *primitive);
            encode_render_space(out, *space);
        }
        Instruction::CreateRenderChild {
            dst,
            parent,
            primitive,
            space,
        } => {
            out.push(0x31);
            write_u32(out, dst.0);
            write_u32(out, parent.0);
            encode_render_primitive(out, *primitive);
            encode_render_space(out, *space);
        }
        Instruction::BindRenderAtom { atom, node, dirty } => {
            out.push(0x32);
            write_u32(out, atom.0);
            write_u32(out, node.0);
            out.push(dirty.bits());
        }
        Instruction::SetRenderVisible { node, visible } => {
            out.push(0x33);
            write_u32(out, node.0);
            out.push(u8::from(*visible));
        }
        Instruction::SetRenderOpacity { node, opacity } => {
            out.push(0x34);
            write_u32(out, node.0);
            write_f32(out, *opacity);
        }
        Instruction::SetRenderPosition { node, position } => {
            out.push(0x35);
            write_u32(out, node.0);
            for value in position {
                write_f32(out, *value);
            }
        }
        Instruction::RenderFlush => out.push(0x36),
        Instruction::CreateInputBridge { dst, domain } => {
            out.push(0x40);
            write_u32(out, dst.0);
            encode_domain_ref(out, *domain);
        }
        Instruction::BindInputAtom {
            bridge,
            atom,
            source,
            device,
            target,
            signal,
        } => {
            out.push(0x41);
            write_u32(out, bridge.0);
            write_u32(out, atom.0);
            encode_optional_input_source(out, *source);
            encode_optional_device(out, *device);
            encode_input_target(out, *target);
            encode_input_signal(out, *signal);
        }
        Instruction::ApplyInput { bridge } => {
            out.push(0x42);
            write_u32(out, bridge.0);
        }
        Instruction::ScheduleTimerOnceAt { dst, deadline } => {
            out.push(0x50);
            write_u32(out, dst.0);
            write_u64(out, deadline.0);
        }
        Instruction::ScheduleTimerRepeatingAt {
            dst,
            first_deadline,
            interval,
        } => {
            out.push(0x51);
            write_u32(out, dst.0);
            write_u64(out, first_deadline.0);
            write_u64(out, interval.0);
        }
        Instruction::CancelTimer { timer } => {
            out.push(0x52);
            write_u32(out, timer.0);
        }
        Instruction::DefineEffectCompletion {
            dst,
            name,
            domain,
            projections,
        } => {
            out.push(0x70);
            write_u32(out, dst.0);
            write_string(out, name)?;
            encode_domain_ref(out, *domain);
            write_len(out, projections.len())?;
            for projection in projections {
                write_u32(out, projection.atom.0);
                encode_completion_projection_value(out, projection.value);
            }
        }
        Instruction::DefineReaction {
            dst,
            name,
            domain,
            trigger,
            action_name,
            declared_effects,
            steps,
        } => {
            out.push(0x60);
            write_u32(out, dst.0);
            write_string(out, name)?;
            encode_domain_ref(out, *domain);
            encode_reaction_trigger(out, *trigger);
            write_string(out, action_name)?;
            write_len(out, declared_effects.len())?;
            for effect in declared_effects.iter() {
                encode_effect(out, effect)?;
            }
            write_len(out, steps.len())?;
            for step in steps {
                encode_reaction_step(out, step)?;
            }
        }
        Instruction::Halt => out.push(0xff),
    }
    Ok(())
}

fn decode_instruction(input: &mut Decoder<'_>, minor: u16) -> NairResult<Instruction> {
    let opcode = input.read_u8()?;
    match opcode {
        0x01 => Ok(Instruction::Const {
            dst: RegisterId(input.read_u32()?),
            value: decode_value(input)?,
        }),
        0x10 => Ok(Instruction::CreateDomain {
            dst: DomainSlot(input.read_u32()?),
            name: input.read_string()?,
        }),
        0x11 => Ok(Instruction::CreateAtom {
            dst: AtomSlot(input.read_u32()?),
            owner: decode_domain_ref(input)?,
            value: RegisterId(input.read_u32()?),
        }),
        0x12 => Ok(Instruction::Connect {
            source: AtomSlot(input.read_u32()?),
            dependent: AtomSlot(input.read_u32()?),
        }),
        0x20 => Ok(Instruction::BeginTransaction {
            dst: TransactionSlot(input.read_u32()?),
            domain: decode_domain_ref(input)?,
        }),
        0x21 => Ok(Instruction::TxSet {
            tx: TransactionSlot(input.read_u32()?),
            atom: AtomSlot(input.read_u32()?),
            value: RegisterId(input.read_u32()?),
        }),
        0x22 => Ok(Instruction::Commit {
            tx: TransactionSlot(input.read_u32()?),
        }),
        0x23 => Ok(Instruction::Rollback {
            tx: TransactionSlot(input.read_u32()?),
        }),
        0x30 if minor >= 2 => Ok(Instruction::CreateRenderNode {
            dst: RenderNodeSlot(input.read_u32()?),
            primitive: decode_render_primitive(input)?,
            space: decode_render_space(input)?,
        }),
        0x31 if minor >= 2 => Ok(Instruction::CreateRenderChild {
            dst: RenderNodeSlot(input.read_u32()?),
            parent: RenderNodeSlot(input.read_u32()?),
            primitive: decode_render_primitive(input)?,
            space: decode_render_space(input)?,
        }),
        0x32 if minor >= 2 => {
            let atom = AtomSlot(input.read_u32()?);
            let node = RenderNodeSlot(input.read_u32()?);
            let bits = input.read_u8()?;
            let dirty = DirtyMask::from_bits(bits).ok_or(NairError::InvalidDirtyMask(bits))?;
            Ok(Instruction::BindRenderAtom { atom, node, dirty })
        }
        0x33 if minor >= 2 => {
            let node = RenderNodeSlot(input.read_u32()?);
            let visible = match input.read_u8()? {
                0 => false,
                1 => true,
                other => return Err(NairError::InvalidBoolTag(other)),
            };
            Ok(Instruction::SetRenderVisible { node, visible })
        }
        0x34 if minor >= 2 => Ok(Instruction::SetRenderOpacity {
            node: RenderNodeSlot(input.read_u32()?),
            opacity: input.read_f32()?,
        }),
        0x35 if minor >= 2 => Ok(Instruction::SetRenderPosition {
            node: RenderNodeSlot(input.read_u32()?),
            position: [input.read_f32()?, input.read_f32()?, input.read_f32()?],
        }),
        0x36 if minor >= 2 => Ok(Instruction::RenderFlush),
        0x40 if minor >= 3 => Ok(Instruction::CreateInputBridge {
            dst: InputBridgeSlot(input.read_u32()?),
            domain: decode_domain_ref(input)?,
        }),
        0x41 if minor >= 3 => Ok(Instruction::BindInputAtom {
            bridge: InputBridgeSlot(input.read_u32()?),
            atom: AtomSlot(input.read_u32()?),
            source: decode_optional_input_source(input)?,
            device: decode_optional_device(input)?,
            target: decode_input_target(input)?,
            signal: decode_input_signal(input)?,
        }),
        0x42 if minor >= 3 => Ok(Instruction::ApplyInput {
            bridge: InputBridgeSlot(input.read_u32()?),
        }),
        0x50 if minor >= 4 => Ok(Instruction::ScheduleTimerOnceAt {
            dst: TimerSlot(input.read_u32()?),
            deadline: LogicalTime(input.read_u64()?),
        }),
        0x51 if minor >= 4 => Ok(Instruction::ScheduleTimerRepeatingAt {
            dst: TimerSlot(input.read_u32()?),
            first_deadline: LogicalTime(input.read_u64()?),
            interval: LogicalDuration(input.read_u64()?),
        }),
        0x52 if minor >= 4 => Ok(Instruction::CancelTimer {
            timer: TimerSlot(input.read_u32()?),
        }),
        0x60 if minor >= 5 => {
            let dst = ReactionSlot(input.read_u32()?);
            let name = input.read_string()?;
            let domain = decode_domain_ref(input)?;
            let trigger = decode_reaction_trigger(input)?;
            let action_name = input.read_string()?;
            let effect_count = input.read_u32()? as usize;
            let mut declared_effects = NairEffectSet::new();
            for _ in 0..effect_count {
                declared_effects.declare(decode_effect(input)?);
            }
            let step_count = input.read_u32()? as usize;
            let mut steps = Vec::with_capacity(step_count);
            for _ in 0..step_count {
                steps.push(decode_reaction_step(input)?);
            }
            Ok(Instruction::DefineReaction {
                dst,
                name,
                domain,
                trigger,
                action_name,
                declared_effects,
                steps,
            })
        }
        0x70 if minor >= 6 => {
            let dst = CompletionSlot(input.read_u32()?);
            let name = input.read_string()?;
            let domain = decode_domain_ref(input)?;
            let projection_count = input.read_u32()? as usize;
            let mut projections = Vec::with_capacity(projection_count);
            for _ in 0..projection_count {
                projections.push(NairCompletionProjection {
                    atom: AtomSlot(input.read_u32()?),
                    value: decode_completion_projection_value(input)?,
                });
            }
            Ok(Instruction::DefineEffectCompletion {
                dst,
                name,
                domain,
                projections,
            })
        }
        0xff => Ok(Instruction::Halt),
        other => Err(NairError::InvalidOpcode(other)),
    }
}

fn decode_value(input: &mut Decoder<'_>) -> NairResult<Value> {
    match input.read_u8()? {
        0x00 => Ok(Value::Null),
        0x01 => Ok(Value::Bool(false)),
        0x02 => Ok(Value::Bool(true)),
        0x03 => Ok(Value::Int(input.read_i64()?)),
        0x04 => Ok(Value::Float(f64::from_bits(input.read_u64()?))),
        0x05 => Ok(Value::Text(input.read_string()?)),
        other => Err(NairError::InvalidValueTag(other)),
    }
}

fn decode_domain_ref(input: &mut Decoder<'_>) -> NairResult<DomainRef> {
    match input.read_u8()? {
        0x00 => Ok(DomainRef::Root),
        0x01 => Ok(DomainRef::Slot(DomainSlot(input.read_u32()?))),
        other => Err(NairError::InvalidDomainRefTag(other)),
    }
}

fn decode_render_primitive(input: &mut Decoder<'_>) -> NairResult<RenderPrimitive> {
    match input.read_u8()? {
        0x00 => Ok(RenderPrimitive::Group),
        0x01 => Ok(RenderPrimitive::Quad),
        0x02 => Ok(RenderPrimitive::Text),
        0x03 => Ok(RenderPrimitive::Mesh),
        other => Err(NairError::InvalidRenderPrimitiveTag(other)),
    }
}

fn decode_render_space(input: &mut Decoder<'_>) -> NairResult<RenderSpace> {
    match input.read_u8()? {
        0x00 => Ok(RenderSpace::Screen),
        0x01 => Ok(RenderSpace::World),
        other => Err(NairError::InvalidRenderSpaceTag(other)),
    }
}

fn decode_input_source(input: &mut Decoder<'_>) -> NairResult<InputSource> {
    match input.read_u8()? {
        0x00 => Ok(InputSource::Keyboard),
        0x01 => Ok(InputSource::Mouse),
        0x02 => Ok(InputSource::Touch),
        0x03 => Ok(InputSource::Pen),
        0x04 => Ok(InputSource::Gamepad),
        0x05 => Ok(InputSource::XrController),
        0x06 => Ok(InputSource::XrHand),
        other => Err(NairError::InvalidInputSourceTag(other)),
    }
}

fn decode_optional_input_source(input: &mut Decoder<'_>) -> NairResult<Option<InputSource>> {
    match input.read_u8()? {
        0x00 => Ok(None),
        0x01 => Ok(Some(decode_input_source(input)?)),
        other => Err(NairError::InvalidOptionTag(other)),
    }
}

fn decode_optional_device(input: &mut Decoder<'_>) -> NairResult<Option<InputDeviceId>> {
    match input.read_u8()? {
        0x00 => Ok(None),
        0x01 => Ok(Some(InputDeviceId(input.read_u64()?))),
        other => Err(NairError::InvalidOptionTag(other)),
    }
}

fn decode_input_target(input: &mut Decoder<'_>) -> NairResult<InputTargetRef> {
    match input.read_u8()? {
        0x00 => Ok(InputTargetRef::Any),
        0x01 => Ok(InputTargetRef::Global),
        0x02 => Ok(InputTargetRef::RenderNode(RenderNodeSlot(
            input.read_u32()?,
        ))),
        other => Err(NairError::InvalidInputTargetTag(other)),
    }
}

fn decode_input_signal(input: &mut Decoder<'_>) -> NairResult<InputSignal> {
    match input.read_u8()? {
        0x00 => Ok(InputSignal::KeyPressed {
            code: input.read_u32()?,
        }),
        0x01 => Ok(InputSignal::PointerX),
        0x02 => Ok(InputSignal::PointerY),
        0x03 => Ok(InputSignal::PointerButtonPressed {
            button: input.read_u16()?,
        }),
        0x04 => Ok(InputSignal::ButtonPressed {
            button: input.read_u16()?,
        }),
        0x05 => Ok(InputSignal::Axis {
            axis: input.read_u16()?,
        }),
        0x06 => Ok(InputSignal::PoseX),
        0x07 => Ok(InputSignal::PoseY),
        0x08 => Ok(InputSignal::PoseZ),
        other => Err(NairError::InvalidInputSignalTag(other)),
    }
}

fn decode_effect(input: &mut Decoder<'_>) -> NairResult<Effect> {
    match input.read_u8()? {
        0x00 => Ok(Effect::Pure),
        0x01 => Ok(Effect::StateRead),
        0x02 => Ok(Effect::StateWrite),
        0x10 => Ok(Effect::Network(input.read_string()?)),
        0x11 => Ok(Effect::FileRead(input.read_string()?)),
        0x12 => Ok(Effect::FileWrite(input.read_string()?)),
        0x20 => Ok(Effect::Camera),
        0x21 => Ok(Effect::Microphone),
        0x22 => Ok(Effect::Location),
        0x23 => Ok(Effect::Gpu),
        0x24 => Ok(Effect::Xr),
        0x25 => Ok(Effect::Process),
        other => Err(NairError::InvalidEffectTag(other)),
    }
}

fn decode_optional_timer_slot(input: &mut Decoder<'_>) -> NairResult<Option<TimerSlot>> {
    match input.read_u8()? {
        0x00 => Ok(None),
        0x01 => Ok(Some(TimerSlot(input.read_u32()?))),
        other => Err(NairError::InvalidOptionTag(other)),
    }
}

fn decode_optional_u64(input: &mut Decoder<'_>) -> NairResult<Option<u64>> {
    match input.read_u8()? {
        0x00 => Ok(None),
        0x01 => Ok(Some(input.read_u64()?)),
        other => Err(NairError::InvalidOptionTag(other)),
    }
}

fn decode_reaction_trigger(input: &mut Decoder<'_>) -> NairResult<NairReactionTrigger> {
    match input.read_u8()? {
        0x00 => Ok(NairReactionTrigger::Input {
            source: decode_optional_input_source(input)?,
            device: decode_optional_device(input)?,
            target: decode_input_target(input)?,
            signal: decode_input_signal(input)?,
        }),
        0x01 => Ok(NairReactionTrigger::Timer {
            timer: decode_optional_timer_slot(input)?,
            occurrence: decode_optional_u64(input)?,
        }),
        other => Err(NairError::InvalidReactionTriggerTag(other)),
    }
}

fn decode_reaction_value(input: &mut Decoder<'_>) -> NairResult<NairReactionValue> {
    match input.read_u8()? {
        0x00 => Ok(NairReactionValue::Literal(decode_value(input)?)),
        0x01 => Ok(NairReactionValue::InputValue),
        0x02 => Ok(NairReactionValue::TimerOccurrence),
        0x03 => Ok(NairReactionValue::TimerDeadlineTicks),
        other => Err(NairError::InvalidReactionValueTag(other)),
    }
}

fn decode_reaction_step(input: &mut Decoder<'_>) -> NairResult<NairReactionStep> {
    match input.read_u8()? {
        0x00 => Ok(NairReactionStep::Set {
            atom: AtomSlot(input.read_u32()?),
            value: decode_reaction_value(input)?,
        }),
        0x01 => Ok(NairReactionStep::EmitEffect {
            effect: decode_effect(input)?,
        }),
        other => Err(NairError::InvalidReactionStepTag(other)),
    }
}

fn decode_completion_projection_value(
    input: &mut Decoder<'_>,
) -> NairResult<NairCompletionProjectionValue> {
    match input.read_u8()? {
        0x00 => Ok(NairCompletionProjectionValue::OutcomeValue),
        0x01 => Ok(NairCompletionProjectionValue::Succeeded),
        0x02 => Ok(NairCompletionProjectionValue::IntentId),
        0x03 => Ok(NairCompletionProjectionValue::AttemptId),
        0x04 => Ok(NairCompletionProjectionValue::SourceSequence),
        other => Err(NairError::InvalidCompletionProjectionTag(other)),
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    fn read_exact(&mut self, len: usize) -> NairResult<&'a [u8]> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(NairError::UnexpectedEof)?;
        if end > self.bytes.len() {
            return Err(NairError::UnexpectedEof);
        }
        let slice = &self.bytes[self.position..end];
        self.position = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> NairResult<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> NairResult<u16> {
        let bytes: [u8; 2] = self
            .read_exact(2)?
            .try_into()
            .map_err(|_| NairError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> NairResult<u32> {
        let bytes: [u8; 4] = self
            .read_exact(4)?
            .try_into()
            .map_err(|_| NairError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i64(&mut self) -> NairResult<i64> {
        let bytes: [u8; 8] = self
            .read_exact(8)?
            .try_into()
            .map_err(|_| NairError::UnexpectedEof)?;
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> NairResult<u64> {
        let bytes: [u8; 8] = self
            .read_exact(8)?
            .try_into()
            .map_err(|_| NairError::UnexpectedEof)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_f32(&mut self) -> NairResult<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    fn read_string(&mut self) -> NairResult<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| NairError::InvalidUtf8)
    }
}
