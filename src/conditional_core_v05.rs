use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{
    compile_pure_binding_boundary, compile_pure_condition_execution_plan_boundary,
    lower_pure_binding_plan_to_nair, validate_pure_binding_execution_plan, CompilerError,
    NsirPureBindingUnit, PureBindingNairArtifact, PureConditionExecutionPlan,
    PureConditionPlanError, PureConditionPlanForm, SemanticPureComparator, SemanticPureCondition,
};
use crate::frontend::{
    analyze_type_effect_unit, AstElement, Delimiter, Name, NameError, SourceError, SourceSpan,
    SourceText, TokenKind, TypeEffectError, MAX_PURE_BINDINGS,
};
use crate::input::{InputBatch, InputError};
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V05_CONDITION_NAIR_DOMAIN: &[u8] = b"NORDOI-V0.5-CONDITION-NAIR\0";
const V05_CONDITION_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.5-CONDITION-EXECUTION-RECEIPT\0";
const V05_STATIC_IF_PLAN_DOMAIN: &[u8] = b"NORDOI-V0.5-STATIC-IF-PLAN\0";
const V05_STATIC_IF_LOWERING_DOMAIN: &[u8] = b"NORDOI-V0.5-STATIC-IF-LOWERING\0";
const V05_STATIC_IF_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.5-STATIC-IF-EXECUTION-RECEIPT\0";

#[derive(Debug)]
pub enum ConditionalCoreError {
    ConditionPlan(PureConditionPlanError),
    Compiler(CompilerError),
    TypeEffect(TypeEffectError),
    Source(SourceError),
    Name(NameError),
    Syntax {
        message: &'static str,
        span: SourceSpan,
    },
    Semantic {
        message: String,
    },
    Runtime(RuntimeError),
    Invariant {
        message: String,
    },
}

impl ConditionalCoreError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::ConditionPlan(error) => error.primary_span(),
            Self::Compiler(error) => error.primary_span(),
            Self::TypeEffect(error) => error.primary_span(),
            Self::Source(_) => None,
            Self::Name(error) => error.primary_span(),
            Self::Syntax { span, .. } => Some(*span),
            Self::Semantic { .. } | Self::Runtime(_) | Self::Invariant { .. } => None,
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::Runtime(_) | Self::Invariant { .. })
    }
}

impl Display for ConditionalCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConditionPlan(error) => Display::fmt(error, f),
            Self::Compiler(error) => Display::fmt(error, f),
            Self::TypeEffect(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Name(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => write!(f, "V0.5 conditional syntax error: {message}"),
            Self::Semantic { message } => write!(f, "V0.5 conditional semantic error: {message}"),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Invariant { message } => {
                write!(f, "V0.5 conditional invariant failed: {message}")
            }
        }
    }
}

impl Error for ConditionalCoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConditionPlan(error) => Some(error),
            Self::Compiler(error) => Some(error),
            Self::TypeEffect(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Name(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::Syntax { .. } | Self::Semantic { .. } | Self::Invariant { .. } => None,
        }
    }
}

impl From<PureConditionPlanError> for ConditionalCoreError {
    fn from(value: PureConditionPlanError) -> Self {
        Self::ConditionPlan(value)
    }
}

impl From<CompilerError> for ConditionalCoreError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<TypeEffectError> for ConditionalCoreError {
    fn from(value: TypeEffectError) -> Self {
        Self::TypeEffect(value)
    }
}

impl From<SourceError> for ConditionalCoreError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<NameError> for ConditionalCoreError {
    fn from(value: NameError) -> Self {
        Self::Name(value)
    }
}

impl From<RuntimeError> for ConditionalCoreError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for ConditionalCoreError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type ConditionalCoreResult<T> = Result<T, ConditionalCoreError>;

#[derive(Debug, Clone, PartialEq)]
pub struct V05ConditionNairArtifact {
    plan: PureConditionExecutionPlan,
    program: NairProgram,
    canonical_nair: Vec<u8>,
    result_register: Option<RegisterId>,
}

impl V05ConditionNairArtifact {
    pub fn plan(&self) -> &PureConditionExecutionPlan {
        &self.plan
    }

    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn result_register(&self) -> Option<RegisterId> {
        self.result_register
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.plan.result_bool()
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.program.len()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.program.required_format_minor()
    }

    pub fn canonical_v05_condition_nair_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(V05_CONDITION_NAIR_DOMAIN);
        push_component(&mut bytes, &self.plan.canonical_c011_bytes());
        match self.result_register {
            None => bytes.push(0),
            Some(register) => {
                bytes.push(1);
                bytes.extend_from_slice(&register.0.to_be_bytes());
            }
        }
        push_component(&mut bytes, &self.canonical_nair);
        bytes
    }
}

pub fn lower_pure_condition_plan_v05(
    plan: PureConditionExecutionPlan,
) -> ConditionalCoreResult<V05ConditionNairArtifact> {
    if plan.work_item_count() != 0
        || plan.runtime_storage_item_count() != 0
        || !plan.required_effects().is_empty()
        || plan.requires_host_authority()
    {
        return Err(ConditionalCoreError::Invariant {
            message:
                "pure condition plan must remain zero-work, zero-storage, pure and authority-free"
                    .to_owned(),
        });
    }

    let mut instructions = Vec::new();
    let mut result_register = None;

    match plan.form() {
        PureConditionPlanForm::Empty => {}
        PureConditionPlanForm::Entry(entry) => {
            if let Some(planned) = entry.condition() {
                match planned.condition() {
                    SemanticPureCondition::Bool(value) => {
                        let dst = RegisterId(0);
                        instructions.push(Instruction::Const {
                            dst,
                            value: Value::Bool(*value),
                        });
                        result_register = Some(dst);
                    }
                    SemanticPureCondition::IntCompare {
                        lhs,
                        comparator,
                        rhs,
                    } => {
                        let lhs_register = RegisterId(0);
                        let rhs_register = RegisterId(1);
                        let dst = RegisterId(2);
                        instructions.push(Instruction::Const {
                            dst: lhs_register,
                            value: Value::Int(*lhs),
                        });
                        instructions.push(Instruction::Const {
                            dst: rhs_register,
                            value: Value::Int(*rhs),
                        });
                        instructions.push(comparison_instruction(
                            *comparator,
                            dst,
                            lhs_register,
                            rhs_register,
                        ));
                        result_register = Some(dst);
                    }
                }
            }
        }
    }

    instructions.push(Instruction::Halt);
    let program = NairProgram::from_instructions(instructions);
    program
        .validate()
        .map_err(|error| ConditionalCoreError::Invariant {
            message: format!("generated condition NAIR failed validation: {error}"),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| ConditionalCoreError::Invariant {
                message: format!("generated condition NAIR failed canonical encoding: {error}"),
            })?;

    Ok(V05ConditionNairArtifact {
        plan,
        program,
        canonical_nair,
        result_register,
    })
}

pub fn compile_pure_condition_nair_v05(
    source: &SourceText,
) -> ConditionalCoreResult<V05ConditionNairArtifact> {
    lower_pure_condition_plan_v05(compile_pure_condition_execution_plan_boundary(source)?)
}

#[derive(Debug, Clone, PartialEq)]
pub struct V05ConditionExecutionReport {
    lowering: V05ConditionNairArtifact,
    runtime: RuntimeObservedReport,
    canonical_input: Vec<u8>,
    result: Option<bool>,
}

impl V05ConditionExecutionReport {
    pub fn lowering(&self) -> &V05ConditionNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeObservedReport {
        &self.runtime
    }

    pub fn result_bool(&self) -> Option<bool> {
        self.result
    }

    pub fn canonical_v05_receipt_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(V05_CONDITION_RECEIPT_DOMAIN);
        push_component(
            &mut bytes,
            &self.lowering.canonical_v05_condition_nair_bytes(),
        );
        push_component(&mut bytes, &self.canonical_input);
        bytes.extend_from_slice(&self.runtime.runtime().replay_key.value().to_be_bytes());
        bytes.push(u8::from(self.runtime.is_quiescent()));
        match self.result {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                bytes.push(u8::from(value));
            }
        }
        bytes
    }
}

pub fn execute_pure_condition_source_v05(
    source: &SourceText,
) -> ConditionalCoreResult<V05ConditionExecutionReport> {
    let lowering = compile_pure_condition_nair_v05(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;
    let expected = lowering.result_bool();
    let observed = match lowering.result_register() {
        None => None,
        Some(register) => match runtime.register(register) {
            Some(Value::Bool(value)) => Some(*value),
            Some(other) => {
                return Err(ConditionalCoreError::Invariant {
                    message: format!("condition result register contains non-bool value {other:?}"),
                })
            }
            None => {
                return Err(ConditionalCoreError::Invariant {
                    message: "condition result register missing after runtime execution".to_owned(),
                })
            }
        },
    };
    if observed != expected {
        return Err(ConditionalCoreError::Invariant {
            message: "runtime boolean result differs from certified C0.11 truth".to_owned(),
        });
    }
    validate_zero_state(&runtime)?;

    Ok(V05ConditionExecutionReport {
        lowering,
        runtime,
        canonical_input,
        result: observed,
    })
}

fn comparison_instruction(
    comparator: SemanticPureComparator,
    dst: RegisterId,
    lhs: RegisterId,
    rhs: RegisterId,
) -> Instruction {
    match comparator {
        SemanticPureComparator::Eq => Instruction::IntEq { dst, lhs, rhs },
        SemanticPureComparator::Ne => Instruction::IntNe { dst, lhs, rhs },
        SemanticPureComparator::Lt => Instruction::IntLt { dst, lhs, rhs },
        SemanticPureComparator::Le => Instruction::IntLe { dst, lhs, rhs },
        SemanticPureComparator::Gt => Instruction::IntGt { dst, lhs, rhs },
        SemanticPureComparator::Ge => Instruction::IntGe { dst, lhs, rhs },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticIfOperand {
    Int(i64),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticIfCondition {
    Bool(bool),
    IntCompare {
        lhs: StaticIfOperand,
        comparator: SemanticPureComparator,
        rhs: StaticIfOperand,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticIfBranch {
    Then,
    Else,
}

impl StaticIfBranch {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Then => "THEN",
            Self::Else => "ELSE",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Then => 1,
            Self::Else => 2,
        }
    }
}

#[derive(Debug, Clone)]
struct SurfaceStaticIf {
    condition: StaticIfCondition,
    conditional_span: SourceSpan,
    then_inner_span: SourceSpan,
    else_inner_span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct V05StaticIfPlan {
    condition: StaticIfCondition,
    condition_value: bool,
    selected: StaticIfBranch,
    then_semantics: NsirPureBindingUnit,
    else_semantics: NsirPureBindingUnit,
}

impl V05StaticIfPlan {
    pub fn condition(&self) -> &StaticIfCondition {
        &self.condition
    }

    pub fn condition_value(&self) -> bool {
        self.condition_value
    }

    pub fn selected_branch(&self) -> StaticIfBranch {
        self.selected
    }

    pub fn then_semantics(&self) -> &NsirPureBindingUnit {
        &self.then_semantics
    }

    pub fn else_semantics(&self) -> &NsirPureBindingUnit {
        &self.else_semantics
    }

    pub fn selected_semantics(&self) -> &NsirPureBindingUnit {
        match self.selected {
            StaticIfBranch::Then => &self.then_semantics,
            StaticIfBranch::Else => &self.else_semantics,
        }
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.selected_semantics().result_i64()
    }

    pub fn dead_branch_eliminated(&self) -> bool {
        true
    }

    pub fn runtime_branch_count(&self) -> usize {
        0
    }

    pub fn canonical_v05_plan_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(V05_STATIC_IF_PLAN_DOMAIN);
        encode_static_condition(&self.condition, &mut bytes);
        bytes.push(u8::from(self.condition_value));
        bytes.push(self.selected.tag());
        push_component(&mut bytes, &self.then_semantics.canonical_l08_bytes());
        push_component(&mut bytes, &self.else_semantics.canonical_l08_bytes());
        bytes.push(u8::from(self.dead_branch_eliminated()));
        bytes.extend_from_slice(&(self.runtime_branch_count() as u32).to_be_bytes());
        bytes
    }
}

pub fn compile_static_if_plan_v05(source: &SourceText) -> ConditionalCoreResult<V05StaticIfPlan> {
    let surface = analyze_static_if(source)?;
    let then_source = selected_branch_source(source, &surface, StaticIfBranch::Then)?;
    let else_source = selected_branch_source(source, &surface, StaticIfBranch::Else)?;

    let then_semantics = compile_pure_binding_boundary(&then_source)?;
    let else_semantics = compile_pure_binding_boundary(&else_source)?;

    if then_semantics.bindings().canonical_bytes() != else_semantics.bindings().canonical_bytes() {
        return Err(ConditionalCoreError::Invariant {
            message: "then/else synthetic branches must preserve one canonical binding registry"
                .to_owned(),
        });
    }
    if then_semantics.semantic().canonical_c02_bytes()
        != else_semantics.semantic().canonical_c02_bytes()
    {
        return Err(ConditionalCoreError::Invariant {
            message: "then/else synthetic branches must preserve one C0.2 semantic identity"
                .to_owned(),
        });
    }

    let condition_value = evaluate_static_condition(&surface.condition, &then_semantics)?;
    let selected = if condition_value {
        StaticIfBranch::Then
    } else {
        StaticIfBranch::Else
    };

    Ok(V05StaticIfPlan {
        condition: surface.condition,
        condition_value,
        selected,
        then_semantics,
        else_semantics,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct V05StaticIfNairArtifact {
    plan: V05StaticIfPlan,
    selected_lowering: PureBindingNairArtifact,
}

impl V05StaticIfNairArtifact {
    pub fn plan(&self) -> &V05StaticIfPlan {
        &self.plan
    }

    pub fn selected_lowering(&self) -> &PureBindingNairArtifact {
        &self.selected_lowering
    }

    pub fn program(&self) -> &NairProgram {
        self.selected_lowering.program()
    }

    pub fn result_register(&self) -> Option<RegisterId> {
        self.selected_lowering.result_register()
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.plan.result_i64()
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.selected_lowering.nair_instruction_count()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.selected_lowering.nair_format_minor()
    }

    pub fn dead_branch_instruction_count(&self) -> usize {
        0
    }

    pub fn canonical_v05_lowering_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(V05_STATIC_IF_LOWERING_DOMAIN);
        push_component(&mut bytes, &self.plan.canonical_v05_plan_bytes());
        push_component(&mut bytes, &self.selected_lowering.canonical_c010_bytes());
        bytes.extend_from_slice(&(self.dead_branch_instruction_count() as u32).to_be_bytes());
        bytes
    }
}

pub fn lower_static_if_v05(
    plan: V05StaticIfPlan,
) -> ConditionalCoreResult<V05StaticIfNairArtifact> {
    let selected_semantics = plan.selected_semantics().clone();
    let selected_plan = validate_pure_binding_execution_plan(selected_semantics)?;
    let selected_lowering = lower_pure_binding_plan_to_nair(selected_plan)?;

    Ok(V05StaticIfNairArtifact {
        plan,
        selected_lowering,
    })
}

pub fn compile_static_if_nair_v05(
    source: &SourceText,
) -> ConditionalCoreResult<V05StaticIfNairArtifact> {
    lower_static_if_v05(compile_static_if_plan_v05(source)?)
}

#[derive(Debug, Clone, PartialEq)]
pub struct V05StaticIfExecutionReport {
    lowering: V05StaticIfNairArtifact,
    runtime: RuntimeObservedReport,
    canonical_input: Vec<u8>,
    result: Option<i64>,
}

impl V05StaticIfExecutionReport {
    pub fn lowering(&self) -> &V05StaticIfNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeObservedReport {
        &self.runtime
    }

    pub fn result_i64(&self) -> Option<i64> {
        self.result
    }

    pub fn canonical_v05_receipt_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(V05_STATIC_IF_RECEIPT_DOMAIN);
        push_component(&mut bytes, &self.lowering.canonical_v05_lowering_bytes());
        push_component(&mut bytes, &self.canonical_input);
        bytes.extend_from_slice(&self.runtime.runtime().replay_key.value().to_be_bytes());
        bytes.push(self.lowering.plan().selected_branch().tag());
        bytes.push(u8::from(self.lowering.plan().dead_branch_eliminated()));
        match self.result {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        bytes.push(u8::from(self.runtime.is_quiescent()));
        bytes
    }
}

pub fn execute_static_if_source_v05(
    source: &SourceText,
) -> ConditionalCoreResult<V05StaticIfExecutionReport> {
    let lowering = compile_static_if_nair_v05(source)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;

    let expected = lowering.result_i64();
    let observed = match lowering.result_register() {
        None => None,
        Some(register) => match runtime.register(register) {
            Some(Value::Int(value)) => Some(*value),
            Some(other) => {
                return Err(ConditionalCoreError::Invariant {
                    message: format!("selected if branch result is not INT: {other:?}"),
                })
            }
            None => {
                return Err(ConditionalCoreError::Invariant {
                    message: "selected if branch result register missing".to_owned(),
                })
            }
        },
    };
    if observed != expected {
        return Err(ConditionalCoreError::Invariant {
            message: "runtime result differs from selected compile-time branch value".to_owned(),
        });
    }
    validate_zero_state(&runtime)?;

    Ok(V05StaticIfExecutionReport {
        lowering,
        runtime,
        canonical_input,
        result: observed,
    })
}

fn validate_zero_state(runtime: &RuntimeObservedReport) -> ConditionalCoreResult<()> {
    let report = runtime.runtime();
    let execution = &report.execution.execution;
    if execution.created_domains != 0
        || execution.created_atoms != 0
        || execution.committed_transactions != 0
        || execution.rolled_back_transactions != 0
        || execution.scheduled_work != 0
        || !report.execution.frames.is_empty()
        || report.execution.created_input_bridges != 0
        || !runtime.is_quiescent()
    {
        return Err(ConditionalCoreError::Invariant {
            message: "pure conditional runtime must end quiescent with zero persistent state, effects and bridges"
                .to_owned(),
        });
    }
    Ok(())
}

fn analyze_static_if(source: &SourceText) -> ConditionalCoreResult<SurfaceStaticIf> {
    let type_effect = analyze_type_effect_unit(source)?;
    let body_span = type_effect.body_span();
    let mut cursor = ElementCursor::after_offset(
        type_effect.module_unit().file().elements(),
        body_span.start().get(),
    );
    let mut binding_count = 0usize;

    loop {
        let element = cursor
            .next_significant()
            .ok_or(ConditionalCoreError::Syntax {
                message: "expected contextual 'entry' after optional const declarations",
                span: type_effect.module_unit().file().eof().span(),
            })?;
        if is_keyword(source, element, "const")? {
            binding_count += 1;
            if binding_count > MAX_PURE_BINDINGS as usize {
                return Err(ConditionalCoreError::Syntax {
                    message: "too many pure bindings before V0.5 static if entry",
                    span: element.span(),
                });
            }
            consume_const(source, &mut cursor)?;
            continue;
        }
        if is_keyword(source, element, "entry")? {
            break;
        }
        return Err(ConditionalCoreError::Syntax {
            message: "only contextual const declarations may precede the V0.5 entry",
            span: element.span(),
        });
    }

    let name_element = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "expected entry name",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    let name_token = name_element
        .as_token()
        .ok_or(ConditionalCoreError::Syntax {
            message: "entry name must be an identifier",
            span: name_element.span(),
        })?;
    Name::from_token(source, name_token)?;

    let returns = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "expected contextual 'returns'",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    if !is_keyword(source, returns, "returns")? {
        return Err(ConditionalCoreError::Syntax {
            message: "V0.5 static conditional entry must continue with contextual 'returns'",
            span: returns.span(),
        });
    }

    let if_element = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "expected contextual 'if'",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    if !is_keyword(source, if_element, "if")? {
        return Err(ConditionalCoreError::Syntax {
            message: "V0.5 static conditional entry must return an if/else expression",
            span: if_element.span(),
        });
    }

    let mut condition_elements = Vec::new();
    let then_group = loop {
        let element = cursor
            .next_significant()
            .ok_or(ConditionalCoreError::Syntax {
                message: "expected '{...}' then branch",
                span: type_effect.module_unit().file().eof().span(),
            })?;
        if let Some(group) = element.as_group() {
            if group.delimiter() != Delimiter::Brace {
                return Err(ConditionalCoreError::Syntax {
                    message: "V0.5 then branch must use braces",
                    span: group.span(),
                });
            }
            break group;
        }
        condition_elements.push(element);
    };
    if condition_elements.is_empty() {
        return Err(ConditionalCoreError::Syntax {
            message: "if condition must not be empty",
            span: if_element.span(),
        });
    }

    let else_element = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "expected contextual 'else'",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    if !is_keyword(source, else_element, "else")? {
        return Err(ConditionalCoreError::Syntax {
            message: "V0.5 static conditional requires an explicit else branch",
            span: else_element.span(),
        });
    }

    let else_element_group = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "expected '{...}' else branch",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    let else_group = else_element_group
        .as_group()
        .ok_or(ConditionalCoreError::Syntax {
            message: "V0.5 else branch must use braces",
            span: else_element_group.span(),
        })?;
    if else_group.delimiter() != Delimiter::Brace {
        return Err(ConditionalCoreError::Syntax {
            message: "V0.5 else branch must use braces",
            span: else_group.span(),
        });
    }

    let terminator_element = cursor
        .next_significant()
        .ok_or(ConditionalCoreError::Syntax {
            message: "V0.5 static if expression must end with ';'",
            span: type_effect.module_unit().file().eof().span(),
        })?;
    terminator_element
        .as_token()
        .filter(|token| token.kind() == &TokenKind::Punctuation(';'))
        .ok_or(ConditionalCoreError::Syntax {
            message: "V0.5 static if expression must end with ';'",
            span: terminator_element.span(),
        })?;
    if let Some(extra) = cursor.next_significant() {
        return Err(ConditionalCoreError::Syntax {
            message: "no significant source is allowed after the V0.5 entry",
            span: extra.span(),
        });
    }

    if significant_elements(then_group.elements()).is_empty() {
        return Err(ConditionalCoreError::Syntax {
            message: "then branch must contain a pure integer expression",
            span: then_group.span(),
        });
    }
    if significant_elements(else_group.elements()).is_empty() {
        return Err(ConditionalCoreError::Syntax {
            message: "else branch must contain a pure integer expression",
            span: else_group.span(),
        });
    }

    let condition = parse_static_condition(source, &condition_elements)?;
    let conditional_span = source.span(if_element.span().start(), else_group.span().end())?;
    let then_inner_span = source.span(
        then_group.open().span().end(),
        then_group.close().span().start(),
    )?;
    let else_inner_span = source.span(
        else_group.open().span().end(),
        else_group.close().span().start(),
    )?;

    Ok(SurfaceStaticIf {
        condition,
        conditional_span,
        then_inner_span,
        else_inner_span,
    })
}

fn consume_const(source: &SourceText, cursor: &mut ElementCursor<'_>) -> ConditionalCoreResult<()> {
    let name = cursor
        .next_significant()
        .ok_or(syntax_at_eof(source, "expected binding name after const"))?;
    let token = name.as_token().ok_or(ConditionalCoreError::Syntax {
        message: "binding name must be an identifier",
        span: name.span(),
    })?;
    Name::from_token(source, token)?;

    let equals = cursor
        .next_significant()
        .ok_or(syntax_at_eof(source, "expected '=' after binding name"))?;
    require_punctuation(equals, '=', "expected '=' after binding name")?;

    let value = cursor.next_significant().ok_or(syntax_at_eof(
        source,
        "expected integer binding initializer",
    ))?;
    let value_token = value.as_token().ok_or(ConditionalCoreError::Syntax {
        message: "binding initializer must be a canonical non-negative integer literal",
        span: value.span(),
    })?;
    if value_token.kind() != &TokenKind::NumericCandidate
        || !is_canonical_non_negative_decimal(source.slice(value_token.span())?)
        || source.slice(value_token.span())?.parse::<i64>().is_err()
    {
        return Err(ConditionalCoreError::Syntax {
            message: "binding initializer must be a canonical non-negative i64 literal",
            span: value_token.span(),
        });
    }

    let terminator = cursor.next_significant().ok_or(syntax_at_eof(
        source,
        "binding declaration must end with ';'",
    ))?;
    require_punctuation(terminator, ';', "binding declaration must end with ';'")?;
    Ok(())
}

fn parse_static_condition(
    source: &SourceText,
    elements: &[&AstElement],
) -> ConditionalCoreResult<StaticIfCondition> {
    if elements.len() == 1 {
        if let Some(token) = elements[0].as_token() {
            if token.kind() == &TokenKind::Identifier {
                match source.slice(token.span())? {
                    "true" => return Ok(StaticIfCondition::Bool(true)),
                    "false" => return Ok(StaticIfCondition::Bool(false)),
                    _ => {}
                }
            }
        }
        return Err(ConditionalCoreError::Syntax {
            message: "condition must be true, false, or one integer comparison",
            span: elements[0].span(),
        });
    }

    let lhs = parse_static_operand(source, elements[0])?;
    let mut index = 1usize;
    let comparator = parse_comparator(elements, &mut index)?;
    let rhs_element = elements
        .get(index)
        .copied()
        .ok_or(ConditionalCoreError::Syntax {
            message: "comparison is missing its right operand",
            span: elements[elements.len() - 1].span(),
        })?;
    let rhs = parse_static_operand(source, rhs_element)?;
    index += 1;
    if let Some(extra) = elements.get(index) {
        return Err(ConditionalCoreError::Syntax {
            message: "unexpected tokens after pure static comparison",
            span: extra.span(),
        });
    }

    Ok(StaticIfCondition::IntCompare {
        lhs,
        comparator,
        rhs,
    })
}

fn parse_static_operand(
    source: &SourceText,
    element: &AstElement,
) -> ConditionalCoreResult<StaticIfOperand> {
    let token = element.as_token().ok_or(ConditionalCoreError::Syntax {
        message: "comparison operand must be an integer literal or pure binding name",
        span: element.span(),
    })?;
    match token.kind() {
        TokenKind::NumericCandidate => {
            let text = source.slice(token.span())?;
            if !is_canonical_non_negative_decimal(text) {
                return Err(ConditionalCoreError::Syntax {
                    message: "comparison integer must use canonical non-negative decimal form",
                    span: token.span(),
                });
            }
            let value = text
                .parse::<i64>()
                .map_err(|_| ConditionalCoreError::Syntax {
                    message: "comparison integer exceeds i64 range",
                    span: token.span(),
                })?;
            Ok(StaticIfOperand::Int(value))
        }
        TokenKind::Identifier => {
            let name = Name::from_token(source, token)?;
            Ok(StaticIfOperand::Binding(name.text(source)?.to_owned()))
        }
        _ => Err(ConditionalCoreError::Syntax {
            message: "comparison operand must be an integer literal or pure binding name",
            span: token.span(),
        }),
    }
}

fn parse_comparator(
    elements: &[&AstElement],
    index: &mut usize,
) -> ConditionalCoreResult<SemanticPureComparator> {
    let first_element = elements
        .get(*index)
        .copied()
        .ok_or(ConditionalCoreError::Syntax {
            message: "missing comparison operator",
            span: elements[0].span(),
        })?;
    let first = first_element
        .as_token()
        .ok_or(ConditionalCoreError::Syntax {
            message: "comparison operator must be punctuation",
            span: first_element.span(),
        })?;
    let TokenKind::Punctuation(first_char) = first.kind() else {
        return Err(ConditionalCoreError::Syntax {
            message: "expected one of == != < <= > >=",
            span: first.span(),
        });
    };
    *index += 1;

    let adjacent_equals = elements
        .get(*index)
        .and_then(|element| element.as_token())
        .filter(|token| {
            token.kind() == &TokenKind::Punctuation('=')
                && first.span().end().get() == token.span().start().get()
        });

    let comparator = match *first_char {
        '=' => {
            if adjacent_equals.is_none() {
                return Err(ConditionalCoreError::Syntax {
                    message: "equality comparator must be '=='",
                    span: first.span(),
                });
            }
            *index += 1;
            SemanticPureComparator::Eq
        }
        '!' => {
            if adjacent_equals.is_none() {
                return Err(ConditionalCoreError::Syntax {
                    message: "inequality comparator must be '!='",
                    span: first.span(),
                });
            }
            *index += 1;
            SemanticPureComparator::Ne
        }
        '<' => {
            if adjacent_equals.is_some() {
                *index += 1;
                SemanticPureComparator::Le
            } else {
                SemanticPureComparator::Lt
            }
        }
        '>' => {
            if adjacent_equals.is_some() {
                *index += 1;
                SemanticPureComparator::Ge
            } else {
                SemanticPureComparator::Gt
            }
        }
        _ => {
            return Err(ConditionalCoreError::Syntax {
                message: "expected one of == != < <= > >=",
                span: first.span(),
            })
        }
    };

    Ok(comparator)
}

fn evaluate_static_condition(
    condition: &StaticIfCondition,
    semantics: &NsirPureBindingUnit,
) -> ConditionalCoreResult<bool> {
    match condition {
        StaticIfCondition::Bool(value) => Ok(*value),
        StaticIfCondition::IntCompare {
            lhs,
            comparator,
            rhs,
        } => {
            let lhs = resolve_static_operand(lhs, semantics)?;
            let rhs = resolve_static_operand(rhs, semantics)?;
            Ok(match comparator {
                SemanticPureComparator::Eq => lhs == rhs,
                SemanticPureComparator::Ne => lhs != rhs,
                SemanticPureComparator::Lt => lhs < rhs,
                SemanticPureComparator::Le => lhs <= rhs,
                SemanticPureComparator::Gt => lhs > rhs,
                SemanticPureComparator::Ge => lhs >= rhs,
            })
        }
    }
}

fn resolve_static_operand(
    operand: &StaticIfOperand,
    semantics: &NsirPureBindingUnit,
) -> ConditionalCoreResult<i64> {
    match operand {
        StaticIfOperand::Int(value) => Ok(*value),
        StaticIfOperand::Binding(name) => semantics
            .bindings()
            .resolve(name)
            .map(|binding| binding.value())
            .ok_or_else(|| ConditionalCoreError::Semantic {
                message: format!("static if condition references unknown pure binding '{name}'"),
            }),
    }
}

fn selected_branch_source(
    source: &SourceText,
    surface: &SurfaceStaticIf,
    selected: StaticIfBranch,
) -> ConditionalCoreResult<SourceText> {
    let branch_span = match selected {
        StaticIfBranch::Then => surface.then_inner_span,
        StaticIfBranch::Else => surface.else_inner_span,
    };
    let start = surface.conditional_span.start().get() as usize;
    let end = surface.conditional_span.end().get() as usize;
    let branch = source.slice(branch_span)?;
    let mut text = String::with_capacity(source.text().len());
    text.push_str(&source.text()[..start]);
    text.push_str(branch);
    text.push_str(&source.text()[end..]);
    Ok(SourceText::new(source.id(), source.name(), text)?)
}

fn encode_static_condition(condition: &StaticIfCondition, bytes: &mut Vec<u8>) {
    match condition {
        StaticIfCondition::Bool(value) => {
            bytes.push(1);
            bytes.push(u8::from(*value));
        }
        StaticIfCondition::IntCompare {
            lhs,
            comparator,
            rhs,
        } => {
            bytes.push(2);
            encode_static_operand(lhs, bytes);
            bytes.push(comparator_tag(*comparator));
            encode_static_operand(rhs, bytes);
        }
    }
}

fn encode_static_operand(operand: &StaticIfOperand, bytes: &mut Vec<u8>) {
    match operand {
        StaticIfOperand::Int(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        StaticIfOperand::Binding(name) => {
            bytes.push(2);
            let raw = name.as_bytes();
            bytes.extend_from_slice(&(raw.len() as u32).to_be_bytes());
            bytes.extend_from_slice(raw);
        }
    }
}

fn comparator_tag(comparator: SemanticPureComparator) -> u8 {
    match comparator {
        SemanticPureComparator::Eq => 1,
        SemanticPureComparator::Ne => 2,
        SemanticPureComparator::Lt => 3,
        SemanticPureComparator::Le => 4,
        SemanticPureComparator::Gt => 5,
        SemanticPureComparator::Ge => 6,
    }
}

fn is_keyword(
    source: &SourceText,
    element: &AstElement,
    expected: &str,
) -> ConditionalCoreResult<bool> {
    let Some(token) = element.as_token() else {
        return Ok(false);
    };
    if token.kind() != &TokenKind::Identifier {
        return Ok(false);
    }
    Ok(source.slice(token.span())? == expected)
}

fn require_punctuation(
    element: &AstElement,
    expected: char,
    message: &'static str,
) -> ConditionalCoreResult<()> {
    let Some(token) = element.as_token() else {
        return Err(ConditionalCoreError::Syntax {
            message,
            span: element.span(),
        });
    };
    if token.kind() != &TokenKind::Punctuation(expected) {
        return Err(ConditionalCoreError::Syntax {
            message,
            span: token.span(),
        });
    }
    Ok(())
}

fn syntax_at_eof(source: &SourceText, message: &'static str) -> ConditionalCoreError {
    ConditionalCoreError::Syntax {
        message,
        span: source.full_span(),
    }
}

fn significant_elements(elements: &[AstElement]) -> Vec<&AstElement> {
    elements
        .iter()
        .filter(|element| !matches!(element, AstElement::Token(token) if token.is_trivia()))
        .collect()
}

fn is_canonical_non_negative_decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes == b"0" {
        return true;
    }
    if bytes.is_empty() || !(b'1'..=b'9').contains(&bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(|byte| byte.is_ascii_digit())
}

fn push_component(bytes: &mut Vec<u8>, component: &[u8]) {
    bytes.extend_from_slice(&(component.len() as u32).to_be_bytes());
    bytes.extend_from_slice(component);
}

#[derive(Debug)]
struct ElementCursor<'a> {
    elements: &'a [AstElement],
    index: usize,
}

impl<'a> ElementCursor<'a> {
    fn after_offset(elements: &'a [AstElement], offset: u32) -> Self {
        let mut index = 0usize;
        while index < elements.len() && elements[index].span().end().get() <= offset {
            index += 1;
        }
        Self { elements, index }
    }

    fn next_significant(&mut self) -> Option<&'a AstElement> {
        while self.index < self.elements.len() {
            let element = &self.elements[self.index];
            self.index += 1;
            match element {
                AstElement::Token(token) if token.is_trivia() => continue,
                _ => return Some(element),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::SourceId;

    fn source(text: &str) -> SourceText {
        SourceText::new(SourceId::new(905), "v05.noi", text).unwrap()
    }

    #[test]
    fn direct_comparison_lowers_to_nair_08() {
        let artifact =
            compile_pure_condition_nair_v05(&source("entry main returns 20 <= 22;")).unwrap();
        assert_eq!(artifact.nair_format_minor(), 8);
        assert_eq!(artifact.result_bool(), Some(true));
        assert_eq!(artifact.program().len(), 4);
    }

    #[test]
    fn bool_literal_stays_nair_06() {
        let artifact =
            compile_pure_condition_nair_v05(&source("entry main returns true;")).unwrap();
        assert_eq!(artifact.nair_format_minor(), 6);
        assert_eq!(artifact.program().len(), 2);
    }

    #[test]
    fn static_if_erases_dead_branch_before_nair() {
        let source = source(
            "const x = 20; const y = 22; entry main returns if x < y { x + 22 } else { 999 };",
        );
        let artifact = compile_static_if_nair_v05(&source).unwrap();
        assert_eq!(artifact.plan().selected_branch(), StaticIfBranch::Then);
        assert_eq!(artifact.result_i64(), Some(42));
        assert_eq!(artifact.dead_branch_instruction_count(), 0);
        assert_eq!(artifact.nair_instruction_count(), 4);
        assert_eq!(artifact.nair_format_minor(), 7);
    }
}
