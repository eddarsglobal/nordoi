use std::collections::VecDeque;

use super::{
    error::{InputError, InputResult},
    event::{InputEvent, InputPayload, InputSource, InputTarget},
    id::{InputDeviceId, InputSequence},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InputBatch {
    pub events: Vec<InputEvent>,
}

impl InputBatch {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Produces the canonical semantic representation accepted at the runtime boundary.
    /// Publicly constructed batches are normalized exactly like events submitted through
    /// `AtomicInputCore`, and event sequence order must remain strictly increasing.
    pub fn canonicalized(&self) -> InputResult<Self> {
        let mut previous = None;
        let mut events = Vec::with_capacity(self.events.len());

        for event in &self.events {
            if let Some(previous_sequence) = previous {
                if event.sequence <= previous_sequence {
                    return Err(InputError::NonMonotonicSequence {
                        previous: previous_sequence,
                        current: event.sequence,
                    });
                }
            }

            previous = Some(event.sequence);
            events.push(InputEvent {
                sequence: event.sequence,
                source: event.source,
                device: event.device,
                target: event.target,
                payload: event.payload.clone().normalized()?,
            });
        }

        Ok(Self { events })
    }

    /// Canonical binary identity for deterministic runtime replay.
    /// This is an internal semantic format, not a hardware packet format.
    pub fn canonical_bytes(&self) -> InputResult<Vec<u8>> {
        let canonical = self.canonicalized()?;
        let mut out = Vec::new();
        out.extend_from_slice(b"NINP");
        write_u16(&mut out, 1);
        write_u64(&mut out, canonical.events.len() as u64);

        for event in &canonical.events {
            encode_event(&mut out, event);
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct AtomicInputCore {
    queue: VecDeque<InputEvent>,
    focus: Option<InputTarget>,
    next_sequence: u64,
}

impl Default for AtomicInputCore {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicInputCore {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            focus: None,
            next_sequence: 1,
        }
    }

    pub fn set_focus(&mut self, target: Option<InputTarget>) {
        self.focus = target;
    }

    pub fn focus(&self) -> Option<InputTarget> {
        self.focus
    }

    pub fn pending_events(&self) -> usize {
        self.queue.len()
    }

    pub fn submit(
        &mut self,
        source: InputSource,
        device: InputDeviceId,
        explicit_target: Option<InputTarget>,
        payload: InputPayload,
    ) -> InputResult<InputSequence> {
        let payload = payload.normalized()?;
        let target = explicit_target.unwrap_or_else(|| self.default_target(source));
        let sequence = InputSequence(self.next_sequence);
        let following_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(InputError::SequenceExhausted)?;

        let event = InputEvent {
            sequence,
            source,
            device,
            target,
            payload,
        };

        if let Some(previous) = self.queue.back_mut() {
            if same_route(previous, &event) {
                if let Some(payload) = InputPayload::coalesce(&previous.payload, &event.payload)? {
                    previous.sequence = event.sequence;
                    previous.payload = payload;
                    self.next_sequence = following_sequence;
                    return Ok(sequence);
                }
            }
        }

        self.queue.push_back(event);
        self.next_sequence = following_sequence;
        Ok(sequence)
    }

    pub fn drain(&mut self) -> InputBatch {
        InputBatch {
            events: self.queue.drain(..).collect(),
        }
    }

    fn default_target(&self, source: InputSource) -> InputTarget {
        match source {
            InputSource::Keyboard | InputSource::Gamepad => {
                self.focus.unwrap_or(InputTarget::Global)
            }
            _ => InputTarget::Global,
        }
    }
}

fn same_route(a: &InputEvent, b: &InputEvent) -> bool {
    a.source == b.source && a.device == b.device && a.target == b.target
}

fn encode_event(out: &mut Vec<u8>, event: &InputEvent) {
    write_u64(out, event.sequence.0);
    out.push(source_tag(event.source));
    write_u64(out, event.device.0);

    match event.target {
        InputTarget::Global => out.push(0x00),
        InputTarget::RenderNode(node) => {
            out.push(0x01);
            write_u64(out, node.0);
        }
    }

    encode_payload(out, &event.payload);
}

fn encode_payload(out: &mut Vec<u8>, payload: &InputPayload) {
    match payload {
        InputPayload::Key {
            code,
            pressed,
            repeat,
        } => {
            out.push(0x00);
            write_u32(out, *code);
            write_bool(out, *pressed);
            write_bool(out, *repeat);
        }
        InputPayload::PointerMove {
            pointer,
            position,
            delta,
        } => {
            out.push(0x01);
            write_u64(out, pointer.0);
            write_f32(out, position[0]);
            write_f32(out, position[1]);
            write_f32(out, delta[0]);
            write_f32(out, delta[1]);
        }
        InputPayload::PointerButton {
            pointer,
            button,
            pressed,
        } => {
            out.push(0x02);
            write_u64(out, pointer.0);
            write_u16(out, *button);
            write_bool(out, *pressed);
        }
        InputPayload::Button { button, pressed } => {
            out.push(0x03);
            write_u16(out, *button);
            write_bool(out, *pressed);
        }
        InputPayload::Scroll { delta } => {
            out.push(0x04);
            write_f32(out, delta[0]);
            write_f32(out, delta[1]);
        }
        InputPayload::Axis { axis, value } => {
            out.push(0x05);
            write_u16(out, *axis);
            write_f32(out, *value);
        }
        InputPayload::Pose {
            position,
            orientation,
        } => {
            out.push(0x06);
            for value in position {
                write_f32(out, *value);
            }
            for value in orientation {
                write_f32(out, *value);
            }
        }
    }
}

const fn source_tag(source: InputSource) -> u8 {
    match source {
        InputSource::Keyboard => 0x00,
        InputSource::Mouse => 0x01,
        InputSource::Touch => 0x02,
        InputSource::Pen => 0x03,
        InputSource::Gamepad => 0x04,
        InputSource::XrController => 0x05,
        InputSource::XrHand => 0x06,
    }
}

fn write_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}
