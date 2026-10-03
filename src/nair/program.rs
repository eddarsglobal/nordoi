use std::collections::BTreeSet;

use crate::{
    render::{DirtyMask, RenderPrimitive, RenderSpace},
    value::Value,
};

use super::{
    error::{NairError, NairResult},
    id::{AtomSlot, DomainSlot, RegisterId, RenderNodeSlot, TransactionSlot},
    instruction::{DomainRef, Instruction},
};

pub const NAIR_MAGIC: [u8; 4] = *b"NAIR";
pub const NAIR_FORMAT_MAJOR: u16 = 0;
pub const NAIR_FORMAT_MINOR: u16 = 2;
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
