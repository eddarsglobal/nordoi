use crate::render::RenderNodeId;

use super::{
    error::InputError,
    id::{InputDeviceId, InputSequence, PointerId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputSource {
    Keyboard,
    Mouse,
    Touch,
    Pen,
    Gamepad,
    XrController,
    XrHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputTarget {
    Global,
    RenderNode(RenderNodeId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputPayload {
    Key {
        code: u32,
        pressed: bool,
        repeat: bool,
    },
    PointerMove {
        pointer: PointerId,
        position: [f32; 2],
        delta: [f32; 2],
    },
    PointerButton {
        pointer: PointerId,
        button: u16,
        pressed: bool,
    },
    Button {
        button: u16,
        pressed: bool,
    },
    Scroll {
        delta: [f32; 2],
    },
    Axis {
        axis: u16,
        value: f32,
    },
    Pose {
        position: [f32; 3],
        orientation: [f32; 4],
    },
}

impl InputPayload {
    pub(crate) fn normalized(self) -> Result<Self, InputError> {
        match self {
            Self::Key { .. } | Self::PointerButton { .. } | Self::Button { .. } => Ok(self),
            Self::PointerMove {
                pointer,
                position,
                delta,
            } => Ok(Self::PointerMove {
                pointer,
                position: normalize_finite_array(position)?,
                delta: normalize_finite_array(delta)?,
            }),
            Self::Scroll { delta } => Ok(Self::Scroll {
                delta: normalize_finite_array(delta)?,
            }),
            Self::Axis { axis, value } => {
                let value = canonical_f32(value)?;
                if !(-1.0..=1.0).contains(&value) {
                    return Err(InputError::AxisOutOfRange(value));
                }
                Ok(Self::Axis { axis, value })
            }
            Self::Pose {
                position,
                orientation,
            } => {
                let position = normalize_finite_array(position)?;
                let orientation = normalize_finite_array(orientation)?;
                let scale = orientation
                    .iter()
                    .map(|value| value.abs())
                    .fold(0.0_f32, f32::max);
                if scale == 0.0 {
                    return Err(InputError::ZeroOrientation);
                }
                let scaled = orientation.map(|value| value / scale);
                let norm = scaled.iter().map(|value| value * value).sum::<f32>().sqrt();
                let orientation =
                    canonical_quaternion(scaled.map(|value| canonical_zero(value / norm)));
                Ok(Self::Pose {
                    position,
                    orientation,
                })
            }
        }
    }

    pub(crate) fn coalesce(previous: &Self, next: &Self) -> Result<Option<Self>, InputError> {
        match (previous, next) {
            (
                Self::PointerMove {
                    pointer: a,
                    delta: old_delta,
                    ..
                },
                Self::PointerMove {
                    pointer: b,
                    position,
                    delta: new_delta,
                },
            ) if a == b => {
                let delta = [
                    canonical_f32(old_delta[0] + new_delta[0])?,
                    canonical_f32(old_delta[1] + new_delta[1])?,
                ];
                Ok(Some(Self::PointerMove {
                    pointer: *b,
                    position: *position,
                    delta,
                }))
            }
            (Self::Axis { axis: a, .. }, Self::Axis { axis: b, .. }) if a == b => {
                Ok(Some(next.clone()))
            }
            (Self::Pose { .. }, Self::Pose { .. }) => Ok(Some(next.clone())),
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputEvent {
    pub sequence: InputSequence,
    pub source: InputSource,
    pub device: InputDeviceId,
    pub target: InputTarget,
    pub payload: InputPayload,
}

fn canonical_f32(value: f32) -> Result<f32, InputError> {
    if !value.is_finite() {
        return Err(InputError::NonFiniteValue);
    }
    Ok(canonical_zero(value))
}

fn canonical_zero(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

fn canonical_quaternion(mut value: [f32; 4]) -> [f32; 4] {
    let should_flip = value[3] < 0.0
        || (value[3] == 0.0
            && value[..3]
                .iter()
                .find(|component| **component != 0.0)
                .is_some_and(|component| *component < 0.0));

    if should_flip {
        for component in &mut value {
            *component = canonical_zero(-*component);
        }
    }
    value
}

fn normalize_finite_array<const N: usize>(values: [f32; N]) -> Result<[f32; N], InputError> {
    let mut output = values;
    for value in &mut output {
        *value = canonical_f32(*value)?;
    }
    Ok(output)
}
