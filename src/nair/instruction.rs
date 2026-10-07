use crate::{
    input::{InputDeviceId, InputSignal, InputSource},
    render::{DirtyMask, RenderPrimitive, RenderSpace},
    time::{LogicalDuration, LogicalTime},
    value::Value,
};

use super::{
    completion::NairCompletionProjection,
    id::{
        AtomSlot, CompletionSlot, DomainSlot, InputBridgeSlot, ReactionSlot, RegisterId,
        RenderNodeSlot, TimerSlot, TransactionSlot,
    },
    reaction::{NairEffectSet, NairReactionStep, NairReactionTrigger},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainRef {
    Root,
    Slot(DomainSlot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputTargetRef {
    Any,
    Global,
    RenderNode(RenderNodeSlot),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchExpr {
    Value(Value),
    Register(RegisterId),
    IntAddChecked {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntEq {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntNe {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntLt {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntLe {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntGt {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
    IntGe {
        lhs: Box<BranchExpr>,
        rhs: Box<BranchExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Const {
        dst: RegisterId,
        value: Value,
    },
    IntAddChecked {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntEq {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntNe {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntLt {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntLe {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntGt {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    IntGe {
        dst: RegisterId,
        lhs: RegisterId,
        rhs: RegisterId,
    },
    ReadInputKeyCode {
        dst: RegisterId,
        event_index: u32,
    },
    BranchValue {
        dst: RegisterId,
        condition: RegisterId,
        then_value: Value,
        else_value: Value,
    },
    BranchEval {
        dst: RegisterId,
        condition: RegisterId,
        then_expr: BranchExpr,
        else_expr: BranchExpr,
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
    ScheduleTimerOnceAt {
        dst: TimerSlot,
        deadline: LogicalTime,
    },
    ScheduleTimerRepeatingAt {
        dst: TimerSlot,
        first_deadline: LogicalTime,
        interval: LogicalDuration,
    },
    CancelTimer {
        timer: TimerSlot,
    },
    DefineEffectCompletion {
        dst: CompletionSlot,
        name: String,
        domain: DomainRef,
        projections: Vec<NairCompletionProjection>,
    },
    DefineReaction {
        dst: ReactionSlot,
        name: String,
        domain: DomainRef,
        trigger: NairReactionTrigger,
        action_name: String,
        declared_effects: NairEffectSet,
        steps: Vec<NairReactionStep>,
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
            Self::ReadInputKeyCode { .. }
                | Self::CreateInputBridge { .. }
                | Self::BindInputAtom { .. }
                | Self::ApplyInput { .. }
        )
    }

    pub const fn requires_time_context(&self) -> bool {
        matches!(
            self,
            Self::ScheduleTimerOnceAt { .. }
                | Self::ScheduleTimerRepeatingAt { .. }
                | Self::CancelTimer { .. }
        )
    }

    pub const fn requires_reaction_context(&self) -> bool {
        matches!(self, Self::DefineReaction { .. })
    }

    pub const fn requires_completion_context(&self) -> bool {
        matches!(self, Self::DefineEffectCompletion { .. })
    }
}
