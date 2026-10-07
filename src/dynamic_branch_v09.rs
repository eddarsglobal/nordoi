use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{compile_resolved_semantic_boundary, CompilerError};
use crate::dynamic_input_v07::{DynamicValue, DynamicValueKind};
use crate::frontend::{lex, LexError, SourceError, SourceSpan, SourceText, Token, TokenKind};
use crate::input::{InputBatch, InputError};
use crate::nair::{BranchExpr, Instruction, NairProgram, RegisterId};
use crate::runtime::{run_closed_selective_observed, RuntimeError, RuntimeSelectiveObservedReport};
use crate::value::Value;

const V09_WITNESS_DOMAIN: &[u8] = b"NORDOI-V0.9-SELECTIVE-BRANCH-BODY\0";
const V09_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.9-SELECTIVE-BRANCH-BODY-RECEIPT\0";

pub const MAX_V09_CONSTANTS: usize = 256;
pub const MAX_V09_EXPR_NODES: usize = 4096;
pub const MAX_V09_NAME_BYTES: usize = 128;

#[derive(Debug)]
pub enum DynamicBranchBodyError {
    Compiler(CompilerError),
    Lex(LexError),
    Source(SourceError),
    Syntax {
        message: String,
        span: SourceSpan,
    },
    Semantic {
        message: String,
        span: Option<SourceSpan>,
    },
    Runtime(RuntimeError),
    Invariant {
        message: String,
    },
}

impl DynamicBranchBodyError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Compiler(error) => error.primary_span(),
            Self::Lex(error) => error.span(),
            Self::Source(_) => None,
            Self::Syntax { span, .. } => Some(*span),
            Self::Semantic { span, .. } => *span,
            Self::Runtime(_) | Self::Invariant { .. } => None,
        }
    }

    pub fn is_frontend_failure(&self) -> bool {
        !matches!(self, Self::Runtime(_) | Self::Invariant { .. })
    }
}

impl Display for DynamicBranchBodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => write!(f, "V0.9 control syntax error: {message}"),
            Self::Semantic { message, .. } => {
                write!(f, "V0.9 control semantic error: {message}")
            }
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Invariant { message } => write!(f, "V0.9 control invariant failed: {message}"),
        }
    }
}

impl Error for DynamicBranchBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Lex(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::Syntax { .. } | Self::Semantic { .. } | Self::Invariant { .. } => None,
        }
    }
}

impl From<CompilerError> for DynamicBranchBodyError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<LexError> for DynamicBranchBodyError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for DynamicBranchBodyError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<RuntimeError> for DynamicBranchBodyError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for DynamicBranchBodyError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type DynamicBranchBodyResult<T> = Result<T, DynamicBranchBodyError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CompareOp {
    const fn tag(self) -> u8 {
        match self {
            Self::Eq => 1,
            Self::Ne => 2,
            Self::Lt => 3,
            Self::Le => 4,
            Self::Gt => 5,
            Self::Ge => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ControlExpr {
    Int(i64),
    Name(String),
    Add(Box<ControlExpr>, Box<ControlExpr>),
    Compare {
        op: CompareOp,
        lhs: Box<ControlExpr>,
        rhs: Box<ControlExpr>,
    },
    If {
        condition: Box<ControlExpr>,
        then_expr: Box<ControlExpr>,
        else_expr: Box<ControlExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ControlProgram {
    input_name: Option<String>,
    constants: Vec<(String, ControlExpr)>,
    entry_name: String,
    entry: ControlExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V09BranchBodyPlan {
    module: Option<String>,
    input_name: Option<String>,
    constants: BTreeMap<String, DynamicValue>,
    entry_name: String,
    entry: ControlExpr,
    result_kind: DynamicValueKind,
    dynamic: bool,
    canonical_semantics: Vec<u8>,
}

impl V09BranchBodyPlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn input_name(&self) -> Option<&str> {
        self.input_name.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }

    pub fn constant_count(&self) -> usize {
        self.constants.len()
    }

    pub fn result_kind(&self) -> DynamicValueKind {
        self.result_kind
    }

    pub fn is_dynamic(&self) -> bool {
        self.dynamic
    }

    pub fn canonical_v09_semantic_bytes(&self) -> &[u8] {
        &self.canonical_semantics
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V09BranchBodyNairArtifact {
    program: NairProgram,
    result_register: RegisterId,
    canonical_nair: Vec<u8>,
    witness: Vec<u8>,
    dynamic_branch_count: usize,
    selective_branch_count: usize,
}

impl V09BranchBodyNairArtifact {
    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn result_register(&self) -> RegisterId {
        self.result_register
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn canonical_v09_witness_bytes(&self) -> &[u8] {
        &self.witness
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.program.len()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.program.required_format_minor()
    }

    pub fn dynamic_branch_count(&self) -> usize {
        self.dynamic_branch_count
    }

    pub fn selective_branch_count(&self) -> usize {
        self.selective_branch_count
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V09BranchBodyExecutionReport {
    plan: V09BranchBodyPlan,
    lowering: V09BranchBodyNairArtifact,
    runtime: RuntimeSelectiveObservedReport,
    result: DynamicValue,
    receipt: Vec<u8>,
}

impl V09BranchBodyExecutionReport {
    pub fn plan(&self) -> &V09BranchBodyPlan {
        &self.plan
    }

    pub fn lowering(&self) -> &V09BranchBodyNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeSelectiveObservedReport {
        &self.runtime
    }

    pub fn result(&self) -> DynamicValue {
        self.result
    }

    pub fn runtime_computed(&self) -> bool {
        self.plan.is_dynamic()
    }

    pub fn runtime_branches(&self) -> usize {
        self.runtime.runtime().execution.execution.runtime_branches
    }

    pub fn selected_branch_instructions(&self) -> usize {
        self.runtime.branch_work().selected_branch_instructions
    }

    pub fn discarded_branch_instructions(&self) -> usize {
        self.runtime.branch_work().discarded_branch_instructions
    }

    pub fn canonical_v09_receipt_bytes(&self) -> &[u8] {
        &self.receipt
    }
}

pub fn compile_dynamic_branch_body_plan_v09(
    source: &SourceText,
) -> DynamicBranchBodyResult<V09BranchBodyPlan> {
    let semantic = compile_resolved_semantic_boundary(source)?;
    let body_start = semantic.origin().body_span().start().get();
    let tokens = lex(source)?
        .into_iter()
        .filter(|token| {
            token.span().start().get() >= body_start
                && !token.is_trivia()
                && !matches!(token.kind(), TokenKind::Eof)
        })
        .collect::<Vec<_>>();

    let mut parser = ControlParser::new(source, tokens);
    let program = parser.parse_program()?;
    let mut constants = BTreeMap::new();
    let mut names = BTreeSet::new();

    if let Some(input_name) = &program.input_name {
        validate_name(input_name, None)?;
        names.insert(input_name.clone());
    }

    for (name, expr) in &program.constants {
        validate_name(name, None)?;
        if !names.insert(name.clone()) {
            return Err(DynamicBranchBodyError::Semantic {
                message: format!("duplicate dynamic-scope name '{name}'"),
                span: None,
            });
        }
        let analysis = analyze_expr(expr, &constants, program.input_name.as_deref())?;
        if analysis.dynamic {
            return Err(DynamicBranchBodyError::Semantic {
                message: format!("constant '{name}' cannot depend on runtime input"),
                span: None,
            });
        }
        let value =
            eval_static(expr, &constants, program.input_name.as_deref())?.ok_or_else(|| {
                DynamicBranchBodyError::Invariant {
                    message: format!(
                        "static constant '{name}' did not produce a compile-time value"
                    ),
                }
            })?;
        constants.insert(name.clone(), value);
    }

    let analysis = analyze_expr(&program.entry, &constants, program.input_name.as_deref())?;

    let mut canonical = Vec::new();
    canonical.extend_from_slice(V09_WITNESS_DOMAIN);
    push_component(&mut canonical, &semantic.canonical_c02_bytes());
    match &program.input_name {
        Some(name) => {
            canonical.push(1);
            encode_string(name, &mut canonical);
        }
        None => canonical.push(0),
    }
    encode_value_map(&constants, &mut canonical);
    encode_string(&program.entry_name, &mut canonical);
    encode_expr(&program.entry, &mut canonical);
    canonical.push(match analysis.kind {
        DynamicValueKind::Int => 1,
        DynamicValueKind::Bool => 2,
    });
    canonical.push(u8::from(analysis.dynamic));

    Ok(V09BranchBodyPlan {
        module: semantic.module().canonical_text(),
        input_name: program.input_name,
        constants,
        entry_name: program.entry_name,
        entry: program.entry,
        result_kind: analysis.kind,
        dynamic: analysis.dynamic,
        canonical_semantics: canonical,
    })
}

pub fn lower_dynamic_branch_body_plan_v09(
    plan: &V09BranchBodyPlan,
) -> DynamicBranchBodyResult<V09BranchBodyNairArtifact> {
    let mut lowerer = ControlLowerer::new(plan);
    let lowered = lowerer.lower_expr(&plan.entry)?;
    lowerer.instructions.push(Instruction::Halt);
    let dynamic_branch_count = lowerer.dynamic_branch_count;
    let selective_branch_count = lowerer.selective_branch_count;
    let program = NairProgram::from_instructions(lowerer.instructions);
    program
        .validate()
        .map_err(|error| DynamicBranchBodyError::Invariant {
            message: format!("V0.9 generated NAIR failed validation: {error}"),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| DynamicBranchBodyError::Invariant {
                message: format!("V0.9 generated NAIR failed canonical encoding: {error}"),
            })?;

    if selective_branch_count > 0 && program.required_format_minor() != 11 {
        return Err(DynamicBranchBodyError::Invariant {
            message: "selective V0.9 branch body must require NAIR 0.11 semantics".to_owned(),
        });
    }
    if selective_branch_count == 0
        && dynamic_branch_count > 0
        && program.required_format_minor() != 10
    {
        return Err(DynamicBranchBodyError::Invariant {
            message: "constant-arm dynamic branch must preserve NAIR 0.10 semantics".to_owned(),
        });
    }
    if !plan.is_dynamic() && (program.required_format_minor() != 6 || program.len() != 2) {
        return Err(DynamicBranchBodyError::Invariant {
            message: "fully static V0.9 source must collapse to base NAIR 0.6 CONST + HALT"
                .to_owned(),
        });
    }

    let mut witness = Vec::new();
    witness.extend_from_slice(V09_WITNESS_DOMAIN);
    push_component(&mut witness, plan.canonical_v09_semantic_bytes());
    push_component(&mut witness, &canonical_nair);

    Ok(V09BranchBodyNairArtifact {
        program,
        result_register: lowered.register,
        canonical_nair,
        witness,
        dynamic_branch_count,
        selective_branch_count,
    })
}

pub fn execute_dynamic_branch_body_source_v09(
    source: &SourceText,
    input: &InputBatch,
) -> DynamicBranchBodyResult<V09BranchBodyExecutionReport> {
    let plan = compile_dynamic_branch_body_plan_v09(source)?;
    let lowering = lower_dynamic_branch_body_plan_v09(&plan)?;
    let runtime = run_closed_selective_observed(lowering.program(), input)?;
    let value = runtime
        .register(lowering.result_register())
        .ok_or_else(|| DynamicBranchBodyError::Invariant {
            message: format!(
                "V0.9 result register r{} is missing after runtime execution",
                lowering.result_register().0
            ),
        })?;
    let result = match (plan.result_kind(), value) {
        (DynamicValueKind::Int, Value::Int(value)) => DynamicValue::Int(*value),
        (DynamicValueKind::Bool, Value::Bool(value)) => DynamicValue::Bool(*value),
        (expected, actual) => {
            return Err(DynamicBranchBodyError::Invariant {
                message: format!(
                    "V0.9 result kind mismatch: expected {}, got {actual:?}",
                    expected.as_str()
                ),
            });
        }
    };

    if runtime.runtime().execution.execution.runtime_branches != lowering.dynamic_branch_count() {
        return Err(DynamicBranchBodyError::Invariant {
            message: format!(
                "V0.9 branch observation mismatch: lowered={} executed={}",
                lowering.dynamic_branch_count(),
                runtime.runtime().execution.execution.runtime_branches
            ),
        });
    }
    if runtime.branch_work().discarded_branch_instructions != 0 {
        return Err(DynamicBranchBodyError::Invariant {
            message: "V0.9 unselected branch performed observable runtime work".to_owned(),
        });
    }
    if lowering.selective_branch_count() > 0
        && runtime.branch_work().selected_branch_instructions == 0
    {
        return Err(DynamicBranchBodyError::Invariant {
            message: "V0.9 selected branch body produced zero evaluation evidence".to_owned(),
        });
    }

    let input_bytes = input.canonical_bytes()?;
    let mut receipt = Vec::new();
    receipt.extend_from_slice(V09_RECEIPT_DOMAIN);
    push_component(&mut receipt, lowering.canonical_v09_witness_bytes());
    push_component(&mut receipt, &input_bytes);
    receipt.extend_from_slice(&runtime.runtime().replay_key.value().to_be_bytes());
    receipt.extend_from_slice(&(lowering.dynamic_branch_count() as u64).to_be_bytes());
    receipt.extend_from_slice(&(lowering.selective_branch_count() as u64).to_be_bytes());
    receipt.extend_from_slice(
        &(runtime.branch_work().selected_branch_instructions as u64).to_be_bytes(),
    );
    receipt.extend_from_slice(
        &(runtime.branch_work().discarded_branch_instructions as u64).to_be_bytes(),
    );
    encode_dynamic_value(result, &mut receipt);

    Ok(V09BranchBodyExecutionReport {
        plan,
        lowering,
        runtime,
        result,
        receipt,
    })
}

#[derive(Debug, Clone, Copy)]
struct ExprAnalysis {
    kind: DynamicValueKind,
    dynamic: bool,
}

fn analyze_expr(
    expr: &ControlExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
) -> DynamicBranchBodyResult<ExprAnalysis> {
    match expr {
        ControlExpr::Int(_) => Ok(ExprAnalysis {
            kind: DynamicValueKind::Int,
            dynamic: false,
        }),
        ControlExpr::Name(name) => {
            if input_name == Some(name.as_str()) {
                return Ok(ExprAnalysis {
                    kind: DynamicValueKind::Int,
                    dynamic: true,
                });
            }
            let value = constants
                .get(name)
                .ok_or_else(|| DynamicBranchBodyError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                })?;
            Ok(ExprAnalysis {
                kind: value.kind(),
                dynamic: false,
            })
        }
        ControlExpr::Add(lhs, rhs) => {
            let lhs = analyze_expr(lhs, constants, input_name)?;
            let rhs = analyze_expr(rhs, constants, input_name)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(DynamicBranchBodyError::Semantic {
                    message: "checked '+' requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Int,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        ControlExpr::Compare { lhs, rhs, .. } => {
            let lhs = analyze_expr(lhs, constants, input_name)?;
            let rhs = analyze_expr(rhs, constants, input_name)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(DynamicBranchBodyError::Semantic {
                    message: "integer comparison requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Bool,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        ControlExpr::If {
            condition,
            then_expr,
            else_expr,
        } => {
            let condition_analysis = analyze_expr(condition, constants, input_name)?;
            if condition_analysis.kind != DynamicValueKind::Bool {
                return Err(DynamicBranchBodyError::Semantic {
                    message: "if condition must be BOOL".to_owned(),
                    span: None,
                });
            }
            let then_analysis = analyze_expr(then_expr, constants, input_name)?;
            let else_analysis = analyze_expr(else_expr, constants, input_name)?;
            if then_analysis.kind != else_analysis.kind {
                return Err(DynamicBranchBodyError::Semantic {
                    message: "if branches must produce the same value kind".to_owned(),
                    span: None,
                });
            }

            let dynamic = match eval_static(condition, constants, input_name)? {
                Some(DynamicValue::Bool(true)) => then_analysis.dynamic,
                Some(DynamicValue::Bool(false)) => else_analysis.dynamic,
                Some(DynamicValue::Int(_)) => {
                    return Err(DynamicBranchBodyError::Invariant {
                        message: "boolean if condition evaluated as INT".to_owned(),
                    });
                }
                None => true,
            };

            Ok(ExprAnalysis {
                kind: then_analysis.kind,
                dynamic,
            })
        }
    }
}

fn eval_static(
    expr: &ControlExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
) -> DynamicBranchBodyResult<Option<DynamicValue>> {
    match expr {
        ControlExpr::Int(value) => Ok(Some(DynamicValue::Int(*value))),
        ControlExpr::Name(name) if input_name == Some(name.as_str()) => Ok(None),
        ControlExpr::Name(name) => {
            constants
                .get(name)
                .copied()
                .map(Some)
                .ok_or_else(|| DynamicBranchBodyError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                })
        }
        ControlExpr::Add(lhs, rhs) => {
            let Some(DynamicValue::Int(lhs)) = eval_static(lhs, constants, input_name)? else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) = eval_static(rhs, constants, input_name)? else {
                return Ok(None);
            };
            let value = lhs
                .checked_add(rhs)
                .ok_or_else(|| DynamicBranchBodyError::Semantic {
                    message: "checked integer addition overflow".to_owned(),
                    span: None,
                })?;
            Ok(Some(DynamicValue::Int(value)))
        }
        ControlExpr::Compare { op, lhs, rhs } => {
            let Some(DynamicValue::Int(lhs)) = eval_static(lhs, constants, input_name)? else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) = eval_static(rhs, constants, input_name)? else {
                return Ok(None);
            };
            let value = match op {
                CompareOp::Eq => lhs == rhs,
                CompareOp::Ne => lhs != rhs,
                CompareOp::Lt => lhs < rhs,
                CompareOp::Le => lhs <= rhs,
                CompareOp::Gt => lhs > rhs,
                CompareOp::Ge => lhs >= rhs,
            };
            Ok(Some(DynamicValue::Bool(value)))
        }
        ControlExpr::If {
            condition,
            then_expr,
            else_expr,
        } => match eval_static(condition, constants, input_name)? {
            Some(DynamicValue::Bool(true)) => eval_static(then_expr, constants, input_name),
            Some(DynamicValue::Bool(false)) => eval_static(else_expr, constants, input_name),
            Some(DynamicValue::Int(_)) => Err(DynamicBranchBodyError::Invariant {
                message: "boolean if condition evaluated as INT".to_owned(),
            }),
            None => Ok(None),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct LoweredExpr {
    register: RegisterId,
}

struct ControlLowerer<'a> {
    plan: &'a V09BranchBodyPlan,
    instructions: Vec<Instruction>,
    next_register: u32,
    input_register: Option<RegisterId>,
    dynamic_branch_count: usize,
    selective_branch_count: usize,
}

impl<'a> ControlLowerer<'a> {
    fn new(plan: &'a V09BranchBodyPlan) -> Self {
        Self {
            plan,
            instructions: Vec::new(),
            next_register: 0,
            input_register: None,
            dynamic_branch_count: 0,
            selective_branch_count: 0,
        }
    }

    fn allocate(&mut self) -> RegisterId {
        let register = RegisterId(self.next_register);
        self.next_register += 1;
        register
    }

    fn emit_const(&mut self, value: DynamicValue) -> LoweredExpr {
        let register = self.allocate();
        self.instructions.push(Instruction::Const {
            dst: register,
            value: dynamic_value_to_runtime(value),
        });
        LoweredExpr { register }
    }

    fn ensure_input_register(&mut self) -> RegisterId {
        if let Some(register) = self.input_register {
            return register;
        }
        let register = self.allocate();
        self.instructions.push(Instruction::ReadInputKeyCode {
            dst: register,
            event_index: 0,
        });
        self.input_register = Some(register);
        register
    }

    fn lower_branch_expr(&mut self, expr: &ControlExpr) -> DynamicBranchBodyResult<BranchExpr> {
        if let Some(value) = eval_static(expr, &self.plan.constants, self.plan.input_name())? {
            return Ok(BranchExpr::Value(dynamic_value_to_runtime(value)));
        }

        match expr {
            ControlExpr::Int(value) => Ok(BranchExpr::Value(Value::Int(*value))),
            ControlExpr::Name(name) if self.plan.input_name() == Some(name.as_str()) => {
                Ok(BranchExpr::Register(self.ensure_input_register()))
            }
            ControlExpr::Name(name) => {
                let value = self.plan.constants.get(name).copied().ok_or_else(|| {
                    DynamicBranchBodyError::Invariant {
                        message: format!("selective branch lowerer lost constant '{name}'"),
                    }
                })?;
                Ok(BranchExpr::Value(dynamic_value_to_runtime(value)))
            }
            ControlExpr::Add(lhs, rhs) => Ok(BranchExpr::IntAddChecked {
                lhs: Box::new(self.lower_branch_expr(lhs)?),
                rhs: Box::new(self.lower_branch_expr(rhs)?),
            }),
            ControlExpr::Compare { op, lhs, rhs } => {
                let lhs = Box::new(self.lower_branch_expr(lhs)?);
                let rhs = Box::new(self.lower_branch_expr(rhs)?);
                Ok(match op {
                    CompareOp::Eq => BranchExpr::IntEq { lhs, rhs },
                    CompareOp::Ne => BranchExpr::IntNe { lhs, rhs },
                    CompareOp::Lt => BranchExpr::IntLt { lhs, rhs },
                    CompareOp::Le => BranchExpr::IntLe { lhs, rhs },
                    CompareOp::Gt => BranchExpr::IntGt { lhs, rhs },
                    CompareOp::Ge => BranchExpr::IntGe { lhs, rhs },
                })
            }
            ControlExpr::If {
                condition,
                then_expr,
                else_expr,
            } => match eval_static(condition, &self.plan.constants, self.plan.input_name())? {
                Some(DynamicValue::Bool(true)) => self.lower_branch_expr(then_expr),
                Some(DynamicValue::Bool(false)) => self.lower_branch_expr(else_expr),
                Some(DynamicValue::Int(_)) => Err(DynamicBranchBodyError::Invariant {
                    message: "boolean branch-body if condition evaluated as INT".to_owned(),
                }),
                None => Err(DynamicBranchBodyError::Semantic {
                    message: "V0.9 branch bodies do not yet permit nested dynamic if/else"
                        .to_owned(),
                    span: None,
                }),
            },
        }
    }

    fn lower_expr(&mut self, expr: &ControlExpr) -> DynamicBranchBodyResult<LoweredExpr> {
        if let Some(value) = eval_static(expr, &self.plan.constants, self.plan.input_name())? {
            return Ok(self.emit_const(value));
        }

        match expr {
            ControlExpr::Int(value) => Ok(self.emit_const(DynamicValue::Int(*value))),
            ControlExpr::Name(name) if self.plan.input_name() == Some(name.as_str()) => {
                Ok(LoweredExpr {
                    register: self.ensure_input_register(),
                })
            }
            ControlExpr::Name(name) => {
                let value = self.plan.constants.get(name).copied().ok_or_else(|| {
                    DynamicBranchBodyError::Invariant {
                        message: format!("lowerer lost constant '{name}'"),
                    }
                })?;
                Ok(self.emit_const(value))
            }
            ControlExpr::Add(lhs, rhs) => {
                let lhs = self.lower_expr(lhs)?;
                let rhs = self.lower_expr(rhs)?;
                let dst = self.allocate();
                self.instructions.push(Instruction::IntAddChecked {
                    dst,
                    lhs: lhs.register,
                    rhs: rhs.register,
                });
                Ok(LoweredExpr { register: dst })
            }
            ControlExpr::Compare { op, lhs, rhs } => {
                let lhs = self.lower_expr(lhs)?;
                let rhs = self.lower_expr(rhs)?;
                let dst = self.allocate();
                let instruction = match op {
                    CompareOp::Eq => Instruction::IntEq {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                    CompareOp::Ne => Instruction::IntNe {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                    CompareOp::Lt => Instruction::IntLt {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                    CompareOp::Le => Instruction::IntLe {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                    CompareOp::Gt => Instruction::IntGt {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                    CompareOp::Ge => Instruction::IntGe {
                        dst,
                        lhs: lhs.register,
                        rhs: rhs.register,
                    },
                };
                self.instructions.push(instruction);
                Ok(LoweredExpr { register: dst })
            }
            ControlExpr::If {
                condition,
                then_expr,
                else_expr,
            } => match eval_static(condition, &self.plan.constants, self.plan.input_name())? {
                Some(DynamicValue::Bool(true)) => self.lower_expr(then_expr),
                Some(DynamicValue::Bool(false)) => self.lower_expr(else_expr),
                Some(DynamicValue::Int(_)) => Err(DynamicBranchBodyError::Invariant {
                    message: "boolean if condition evaluated as INT during lowering".to_owned(),
                }),
                None => {
                    let condition = self.lower_expr(condition)?;
                    let then_static =
                        eval_static(then_expr, &self.plan.constants, self.plan.input_name())?;
                    let else_static =
                        eval_static(else_expr, &self.plan.constants, self.plan.input_name())?;
                    let dst = self.allocate();
                    match (then_static, else_static) {
                        (Some(then_value), Some(else_value)) => {
                            if then_value.kind() != else_value.kind() {
                                return Err(DynamicBranchBodyError::Invariant {
                                    message: "V0.9 lowering observed mismatched branch kinds"
                                        .to_owned(),
                                });
                            }
                            self.instructions.push(Instruction::BranchValue {
                                dst,
                                condition: condition.register,
                                then_value: dynamic_value_to_runtime(then_value),
                                else_value: dynamic_value_to_runtime(else_value),
                            });
                        }
                        _ => {
                            let then_expr = self.lower_branch_expr(then_expr)?;
                            let else_expr = self.lower_branch_expr(else_expr)?;
                            self.instructions.push(Instruction::BranchEval {
                                dst,
                                condition: condition.register,
                                then_expr,
                                else_expr,
                            });
                            self.selective_branch_count += 1;
                        }
                    }
                    self.dynamic_branch_count += 1;
                    Ok(LoweredExpr { register: dst })
                }
            },
        }
    }
}

struct ControlParser<'a> {
    source: &'a SourceText,
    tokens: Vec<Token>,
    index: usize,
    expr_nodes: usize,
}

impl<'a> ControlParser<'a> {
    fn new(source: &'a SourceText, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            expr_nodes: 0,
        }
    }

    fn parse_program(&mut self) -> DynamicBranchBodyResult<ControlProgram> {
        let mut input_name = None;
        let mut constants = Vec::new();
        let mut entry = None;

        while !self.at_end() {
            if self.is_word("input") {
                if input_name.is_some() {
                    return Err(
                        self.error_here("only one canonical integer input is allowed in V0.9")
                    );
                }
                self.advance();
                let name = self.expect_ident("expected input name after 'input'")?;
                self.expect_punct(';', "expected ';' after input declaration")?;
                input_name = Some(name);
            } else if self.is_word("const") {
                if constants.len() >= MAX_V09_CONSTANTS {
                    return Err(self.error_here("V0.9 constant bound exceeded"));
                }
                self.advance();
                let name = self.expect_ident("expected constant name after 'const'")?;
                self.expect_punct('=', "expected '=' after constant name")?;
                let expr = self.parse_expr()?;
                self.expect_punct(';', "expected ';' after constant declaration")?;
                constants.push((name, expr));
            } else if self.is_word("entry") {
                if entry.is_some() {
                    return Err(self.error_here("only one entry is allowed in V0.9"));
                }
                self.advance();
                let name = self.expect_ident("expected entry name after 'entry'")?;
                self.expect_word("returns", "expected 'returns' after entry name")?;
                let expr = self.parse_expr()?;
                self.expect_punct(';', "expected ';' after entry expression")?;
                entry = Some((name, expr));
            } else {
                return Err(
                    self.error_here("expected 'input', 'const', or 'entry' in V0.9 control body")
                );
            }
        }

        let (entry_name, entry) = entry.ok_or_else(|| DynamicBranchBodyError::Semantic {
            message: "V0.9 control body requires exactly one entry".to_owned(),
            span: None,
        })?;

        Ok(ControlProgram {
            input_name,
            constants,
            entry_name,
            entry,
        })
    }

    fn parse_expr(&mut self) -> DynamicBranchBodyResult<ControlExpr> {
        if self.is_word("if") {
            self.parse_if()
        } else {
            self.parse_compare()
        }
    }

    fn parse_if(&mut self) -> DynamicBranchBodyResult<ControlExpr> {
        self.expect_word("if", "expected 'if'")?;
        let condition = self.parse_expr()?;
        self.expect_punct('{', "expected '{' after if condition")?;
        let then_expr = self.parse_expr()?;
        self.expect_punct('}', "expected '}' after then expression")?;
        self.expect_word("else", "expected 'else' after then branch")?;
        self.expect_punct('{', "expected '{' after else")?;
        let else_expr = self.parse_expr()?;
        self.expect_punct('}', "expected '}' after else expression")?;
        self.node(ControlExpr::If {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    fn parse_compare(&mut self) -> DynamicBranchBodyResult<ControlExpr> {
        let lhs = self.parse_add()?;
        let Some(op) = self.take_compare_op() else {
            return Ok(lhs);
        };
        let rhs = self.parse_add()?;
        if self.peek_compare_op() {
            return Err(self.error_here("chained comparisons are not part of V0.9"));
        }
        self.node(ControlExpr::Compare {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self) -> DynamicBranchBodyResult<ControlExpr> {
        let mut expr = self.parse_primary()?;
        while self.is_punct('+') {
            self.advance();
            let rhs = self.parse_primary()?;
            expr = self.node(ControlExpr::Add(Box::new(expr), Box::new(rhs)))?;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> DynamicBranchBodyResult<ControlExpr> {
        if self.is_word("if") {
            return self.parse_if();
        }
        if self.is_punct('(') {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect_punct(')', "expected ')' after expression")?;
            return Ok(expr);
        }

        let token = self
            .current()
            .ok_or_else(|| DynamicBranchBodyError::Semantic {
                message: "unexpected end of V0.9 expression".to_owned(),
                span: None,
            })?;
        match token.kind() {
            TokenKind::NumericCandidate => {
                let text = self.source.slice(token.span())?;
                let value = text
                    .parse::<i64>()
                    .map_err(|_| DynamicBranchBodyError::Syntax {
                        message: format!("invalid V0.9 integer literal '{text}'"),
                        span: token.span(),
                    })?;
                self.advance();
                self.node(ControlExpr::Int(value))
            }
            TokenKind::Identifier => {
                let name = self.source.slice(token.span())?.to_owned();
                validate_name(&name, Some(token.span()))?;
                self.advance();
                self.node(ControlExpr::Name(name))
            }
            _ => Err(self
                .error_here("expected integer, name, if-expression, or parenthesized expression")),
        }
    }

    fn take_compare_op(&mut self) -> Option<CompareOp> {
        let op = if self.matches_puncts('=', '=') {
            Some(CompareOp::Eq)
        } else if self.matches_puncts('!', '=') {
            Some(CompareOp::Ne)
        } else if self.matches_puncts('<', '=') {
            Some(CompareOp::Le)
        } else if self.matches_puncts('>', '=') {
            Some(CompareOp::Ge)
        } else if self.is_punct('<') {
            Some(CompareOp::Lt)
        } else if self.is_punct('>') {
            Some(CompareOp::Gt)
        } else {
            None
        };
        if let Some(op) = op {
            if matches!(
                op,
                CompareOp::Eq | CompareOp::Ne | CompareOp::Le | CompareOp::Ge
            ) {
                self.advance();
                self.advance();
            } else {
                self.advance();
            }
        }
        op
    }

    fn peek_compare_op(&self) -> bool {
        self.matches_puncts('=', '=')
            || self.matches_puncts('!', '=')
            || self.matches_puncts('<', '=')
            || self.matches_puncts('>', '=')
            || self.is_punct('<')
            || self.is_punct('>')
    }

    fn node(&mut self, expr: ControlExpr) -> DynamicBranchBodyResult<ControlExpr> {
        self.expr_nodes += 1;
        if self.expr_nodes > MAX_V09_EXPR_NODES {
            return Err(self.error_here("V0.9 expression node bound exceeded"));
        }
        Ok(expr)
    }

    fn expect_ident(&mut self, message: &str) -> DynamicBranchBodyResult<String> {
        let token = self
            .current()
            .ok_or_else(|| DynamicBranchBodyError::Semantic {
                message: message.to_owned(),
                span: None,
            })?;
        if !matches!(token.kind(), TokenKind::Identifier) {
            return Err(self.error_here(message));
        }
        let name = self.source.slice(token.span())?.to_owned();
        validate_name(&name, Some(token.span()))?;
        self.advance();
        Ok(name)
    }

    fn expect_word(&mut self, word: &str, message: &str) -> DynamicBranchBodyResult<()> {
        if self.is_word(word) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_punct(&mut self, punct: char, message: &str) -> DynamicBranchBodyResult<()> {
        if self.is_punct(punct) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn is_word(&self, word: &str) -> bool {
        self.current().is_some_and(|token| {
            matches!(token.kind(), TokenKind::Identifier)
                && self
                    .source
                    .slice(token.span())
                    .is_ok_and(|text| text == word)
        })
    }

    fn is_punct(&self, punct: char) -> bool {
        self.current()
            .is_some_and(|token| token.kind() == &TokenKind::Punctuation(punct))
    }

    fn matches_puncts(&self, first: char, second: char) -> bool {
        self.tokens
            .get(self.index)
            .is_some_and(|token| token.kind() == &TokenKind::Punctuation(first))
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|token| token.kind() == &TokenKind::Punctuation(second))
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn advance(&mut self) {
        self.index = self.index.saturating_add(1);
    }

    fn at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn error_here(&self, message: impl Into<String>) -> DynamicBranchBodyError {
        let span = self
            .current()
            .map(Token::span)
            .unwrap_or_else(|| self.source.full_span());
        DynamicBranchBodyError::Syntax {
            message: message.into(),
            span,
        }
    }
}

fn validate_name(name: &str, span: Option<SourceSpan>) -> DynamicBranchBodyResult<()> {
    if name.is_empty() || name.len() > MAX_V09_NAME_BYTES {
        return Err(DynamicBranchBodyError::Semantic {
            message: format!("name '{name}' exceeds V0.9 name bounds"),
            span,
        });
    }
    Ok(())
}

fn dynamic_value_to_runtime(value: DynamicValue) -> Value {
    match value {
        DynamicValue::Int(value) => Value::Int(value),
        DynamicValue::Bool(value) => Value::Bool(value),
    }
}

fn encode_dynamic_value(value: DynamicValue, out: &mut Vec<u8>) {
    match value {
        DynamicValue::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        DynamicValue::Bool(value) => {
            out.push(2);
            out.push(u8::from(value));
        }
    }
}

fn encode_expr(expr: &ControlExpr, out: &mut Vec<u8>) {
    match expr {
        ControlExpr::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        ControlExpr::Name(name) => {
            out.push(2);
            encode_string(name, out);
        }
        ControlExpr::Add(lhs, rhs) => {
            out.push(3);
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        ControlExpr::Compare { op, lhs, rhs } => {
            out.push(4);
            out.push(op.tag());
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        ControlExpr::If {
            condition,
            then_expr,
            else_expr,
        } => {
            out.push(5);
            encode_expr(condition, out);
            encode_expr(then_expr, out);
            encode_expr(else_expr, out);
        }
    }
}

fn encode_value_map(values: &BTreeMap<String, DynamicValue>, out: &mut Vec<u8>) {
    out.extend_from_slice(&(values.len() as u32).to_be_bytes());
    for (name, value) in values {
        encode_string(name, out);
        encode_dynamic_value(*value, out);
    }
}

fn encode_string(value: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&(value.len() as u32).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn push_component(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(&(value.len() as u32).to_be_bytes());
    out.extend_from_slice(value);
}
