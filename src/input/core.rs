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
