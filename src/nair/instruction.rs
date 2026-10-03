use crate::{
    input::{InputDeviceId, InputSignal, InputSource},
    render::{DirtyMask, RenderPrimitive, RenderSpace},
    value::Value,
};

use super::id::{
    AtomSlot, DomainSlot, InputBridgeSlot, RegisterId, RenderNodeSlot, TransactionSlot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainRef {
    Root,
    Slot(DomainSlot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputTargetRef {
    Any,
    Global,
    RenderNode(RenderNodeSlot),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Const {
        dst: RegisterId,
        value: Value,
    },
    CreateDomain {
        dst: DomainSlot,
        name: String,
    },
    CreateAtom {
        dst: AtomSlot,
        owner: DomainRef,
        value: RegisterId,
    },
    Connect {
        source: AtomSlot,
        dependent: AtomSlot,
    },
    BeginTransaction {
        dst: TransactionSlot,
        domain: DomainRef,
    },
    TxSet {
        tx: TransactionSlot,
        atom: AtomSlot,
        value: RegisterId,
    },
    Commit {
        tx: TransactionSlot,
    },
    Rollback {
        tx: TransactionSlot,
    },
    CreateRenderNode {
        dst: RenderNodeSlot,
        primitive: RenderPrimitive,
        space: RenderSpace,
    },
    CreateRenderChild {
        dst: RenderNodeSlot,
        parent: RenderNodeSlot,
        primitive: RenderPrimitive,
        space: RenderSpace,
    },
    BindRenderAtom {
        atom: AtomSlot,
        node: RenderNodeSlot,
        dirty: DirtyMask,
    },
    SetRenderVisible {
        node: RenderNodeSlot,
        visible: bool,
    },
    SetRenderOpacity {
        node: RenderNodeSlot,
        opacity: f32,
    },
    SetRenderPosition {
        node: RenderNodeSlot,
        position: [f32; 3],
    },
    RenderFlush,
    CreateInputBridge {
        dst: InputBridgeSlot,
        domain: DomainRef,
    },
    BindInputAtom {
        bridge: InputBridgeSlot,
        atom: AtomSlot,
        source: Option<InputSource>,
        device: Option<InputDeviceId>,
        target: InputTargetRef,
        signal: InputSignal,
    },
    ApplyInput {
        bridge: InputBridgeSlot,
    },
    Halt,
}

impl Instruction {
    pub const fn requires_render_context(&self) -> bool {
        matches!(
            self,
            Self::CreateRenderNode { .. }
                | Self::CreateRenderChild { .. }
                | Self::BindRenderAtom { .. }
                | Self::SetRenderVisible { .. }
                | Self::SetRenderOpacity { .. }
                | Self::SetRenderPosition { .. }
                | Self::RenderFlush
        )
    }

    pub const fn requires_input_context(&self) -> bool {
        matches!(
            self,
            Self::CreateInputBridge { .. } | Self::BindInputAtom { .. } | Self::ApplyInput { .. }
        )
    }
}
