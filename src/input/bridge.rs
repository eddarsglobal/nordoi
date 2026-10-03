use crate::{atom::AtomId, kernel::AtomicKernel, ownership::DomainId, value::Value};

use super::{
    core::InputBatch,
    error::InputResult,
    event::{InputEvent, InputPayload, InputSource, InputTarget},
    id::InputDeviceId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSignal {
    KeyPressed { code: u32 },
    PointerX,
    PointerY,
    PointerButtonPressed { button: u16 },
    ButtonPressed { button: u16 },
    Axis { axis: u16 },
    PoseX,
    PoseY,
    PoseZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputSelector {
    pub source: Option<InputSource>,
    pub device: Option<InputDeviceId>,
    pub target: Option<InputTarget>,
    pub signal: InputSignal,
}

impl InputSelector {
    pub fn any(signal: InputSignal) -> Self {
        Self {
            source: None,
            device: None,
            target: None,
            signal,
        }
    }

    fn extract(&self, event: &InputEvent) -> Option<Value> {
        if self.source.is_some_and(|source| source != event.source)
            || self.device.is_some_and(|device| device != event.device)
            || self.target.is_some_and(|target| target != event.target)
        {
            return None;
        }

        match (self.signal, &event.payload) {
            (
                InputSignal::KeyPressed { code },
                InputPayload::Key {
                    code: actual,
                    pressed,
                    ..
                },
            ) if code == *actual => Some(Value::Bool(*pressed)),
            (InputSignal::PointerX, InputPayload::PointerMove { position, .. }) => {
                Some(Value::Float(f64::from(position[0])))
            }
            (InputSignal::PointerY, InputPayload::PointerMove { position, .. }) => {
                Some(Value::Float(f64::from(position[1])))
            }
            (
                InputSignal::PointerButtonPressed { button },
                InputPayload::PointerButton {
                    button: actual,
                    pressed,
                    ..
                },
            ) if button == *actual => Some(Value::Bool(*pressed)),
            (
                InputSignal::ButtonPressed { button },
                InputPayload::Button {
                    button: actual,
                    pressed,
                },
            ) if button == *actual => Some(Value::Bool(*pressed)),
            (
                InputSignal::Axis { axis },
                InputPayload::Axis {
                    axis: actual,
                    value,
                },
            ) if axis == *actual => Some(Value::Float(f64::from(*value))),
            (InputSignal::PoseX, InputPayload::Pose { position, .. }) => {
                Some(Value::Float(f64::from(position[0])))
            }
            (InputSignal::PoseY, InputPayload::Pose { position, .. }) => {
                Some(Value::Float(f64::from(position[1])))
            }
            (InputSignal::PoseZ, InputPayload::Pose { position, .. }) => {
                Some(Value::Float(f64::from(position[2])))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputAtomBinding {
    selector: InputSelector,
    atom: AtomId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputBridgeReport {
    pub events: usize,
    pub matched_bindings: usize,
    pub staged_writes: usize,
    pub changed_atoms: usize,
    pub scheduled_atoms: usize,
}

#[derive(Debug, Clone)]
pub struct InputAtomBridge {
    domain: DomainId,
    bindings: Vec<InputAtomBinding>,
}

impl InputAtomBridge {
    pub fn new(domain: DomainId) -> Self {
        Self {
            domain,
            bindings: Vec::new(),
        }
    }

    pub fn domain(&self) -> DomainId {
        self.domain
    }

    pub fn bind(
        &mut self,
        kernel: &AtomicKernel,
        selector: InputSelector,
        atom: AtomId,
    ) -> InputResult<()> {
        kernel.require_owner(atom, self.domain)?;
        self.bindings.push(InputAtomBinding { selector, atom });
        Ok(())
    }

    pub fn apply_batch(
        &self,
        kernel: &mut AtomicKernel,
        batch: &InputBatch,
    ) -> InputResult<InputBridgeReport> {
        let mut planned = Vec::new();
        let mut matched_bindings = 0;

        for event in &batch.events {
            for binding in &self.bindings {
                if let Some(value) = binding.selector.extract(event) {
                    kernel.require_owner(binding.atom, self.domain)?;
                    matched_bindings += 1;
                    planned.push((binding.atom, value));
                }
            }
        }

        if planned.is_empty() {
            return Ok(InputBridgeReport {
                events: batch.len(),
                ..InputBridgeReport::default()
            });
        }

        let mut transaction = kernel.begin_transaction(self.domain)?;
        for (atom, value) in planned {
            transaction.set(kernel, atom, value)?;
        }
        let transaction_report = kernel.commit(transaction)?;

        Ok(InputBridgeReport {
            events: batch.len(),
            matched_bindings,
            staged_writes: transaction_report.staged_writes,
            changed_atoms: transaction_report.changed_atoms,
            scheduled_atoms: transaction_report.scheduled_atoms,
        })
    }
}
