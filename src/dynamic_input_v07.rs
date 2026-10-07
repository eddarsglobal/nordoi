use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{compile_resolved_semantic_boundary, CompilerError};
use crate::frontend::{lex, LexError, SourceError, SourceSpan, SourceText, Token, TokenKind};
use crate::input::{InputBatch, InputError};
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V07_WITNESS_DOMAIN: &[u8] = b"NORDOI-V0.7-DYNAMIC-INPUT-RUNTIME-COMPUTATION\0";
const V07_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.7-DYNAMIC-INPUT-RUNTIME-COMPUTATION-RECEIPT\0";

pub const MAX_V07_CONSTANTS: usize = 256;
pub const MAX_V07_EXPR_NODES: usize = 4096;
pub const MAX_V07_NAME_BYTES: usize = 128;

#[derive(Debug)]
pub enum DynamicInputError {
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

impl DynamicInputError {
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

impl Display for DynamicInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => write!(f, "V0.7 dynamic syntax error: {message}"),
            Self::Semantic { message, .. } => write!(f, "V0.7 dynamic semantic error: {message}"),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Invariant { message } => write!(f, "V0.7 dynamic invariant failed: {message}"),
        }
    }
}

impl Error for DynamicInputError {
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

impl From<CompilerError> for DynamicInputError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<LexError> for DynamicInputError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for DynamicInputError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<RuntimeError> for DynamicInputError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for DynamicInputError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type DynamicInputResult<T> = Result<T, DynamicInputError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicValueKind {
    Int,
    Bool,
}

impl DynamicValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Int => "INT",
            Self::Bool => "BOOL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicValue {
    Int(i64),
    Bool(bool),
}

impl DynamicValue {
    pub const fn kind(self) -> DynamicValueKind {
        match self {
            Self::Int(_) => DynamicValueKind::Int,
            Self::Bool(_) => DynamicValueKind::Bool,
        }
    }

    pub fn as_text(self) -> String {
        match self {
            Self::Int(value) => format!("INT({value})"),
            Self::Bool(value) => format!("BOOL({value})"),
        }
    }

    fn to_runtime(self) -> Value {
        match self {
            Self::Int(value) => Value::Int(value),
            Self::Bool(value) => Value::Bool(value),
        }
    }

    fn encode(self, out: &mut Vec<u8>) {
        match self {
            Self::Int(value) => {
                out.push(1);
                out.extend_from_slice(&value.to_be_bytes());
            }
            Self::Bool(value) => {
                out.push(2);
                out.push(u8::from(value));
            }
        }
    }
}

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
enum DynamicExpr {
    Int(i64),
    Name(String),
    Add(Box<DynamicExpr>, Box<DynamicExpr>),
    Compare {
        op: CompareOp,
        lhs: Box<DynamicExpr>,
        rhs: Box<DynamicExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DynamicProgram {
    input_name: Option<String>,
    constants: Vec<(String, DynamicExpr)>,
    entry_name: String,
    entry: DynamicExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V07DynamicPlan {
    module: Option<String>,
    input_name: Option<String>,
    constants: BTreeMap<String, DynamicValue>,
    entry_name: String,
    entry: DynamicExpr,
    result_kind: DynamicValueKind,
    dynamic: bool,
    canonical_semantics: Vec<u8>,
}

impl V07DynamicPlan {
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

    pub fn canonical_v07_semantic_bytes(&self) -> &[u8] {
        &self.canonical_semantics
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V07DynamicNairArtifact {
    program: NairProgram,
    result_register: RegisterId,
    canonical_nair: Vec<u8>,
    witness: Vec<u8>,
}

impl V07DynamicNairArtifact {
    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn result_register(&self) -> RegisterId {
        self.result_register
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn canonical_v07_witness_bytes(&self) -> &[u8] {
        &self.witness
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.program.len()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.program.required_format_minor()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V07DynamicExecutionReport {
    plan: V07DynamicPlan,
    lowering: V07DynamicNairArtifact,
    runtime: RuntimeObservedReport,
    result: DynamicValue,
    receipt: Vec<u8>,
}

impl V07DynamicExecutionReport {
    pub fn plan(&self) -> &V07DynamicPlan {
        &self.plan
    }

    pub fn lowering(&self) -> &V07DynamicNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeObservedReport {
        &self.runtime
    }

    pub fn result(&self) -> DynamicValue {
        self.result
    }

    pub fn runtime_computed(&self) -> bool {
        self.plan.is_dynamic()
    }

    pub fn canonical_v07_receipt_bytes(&self) -> &[u8] {
        &self.receipt
    }
}

pub fn compile_dynamic_plan_v07(source: &SourceText) -> DynamicInputResult<V07DynamicPlan> {
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

    let mut parser = DynamicParser::new(source, tokens);
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
            return Err(DynamicInputError::Semantic {
                message: format!("duplicate dynamic-scope name '{name}'"),
                span: None,
            });
        }
        let analysis = analyze_expr(expr, &constants, program.input_name.as_deref())?;
        if analysis.dynamic {
            return Err(DynamicInputError::Semantic {
                message: format!("constant '{name}' cannot depend on runtime input"),
                span: None,
            });
        }
        let value =
            eval_static(expr, &constants, program.input_name.as_deref())?.ok_or_else(|| {
                DynamicInputError::Invariant {
                    message: format!(
                        "static constant '{name}' did not produce a compile-time value"
                    ),
                }
            })?;
        constants.insert(name.clone(), value);
    }

    let analysis = analyze_expr(&program.entry, &constants, program.input_name.as_deref())?;
    let dynamic = analysis.dynamic;

    let mut canonical = Vec::new();
    canonical.extend_from_slice(V07_WITNESS_DOMAIN);
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
    canonical.push(u8::from(dynamic));

    Ok(V07DynamicPlan {
        module: semantic.module().canonical_text(),
        input_name: program.input_name,
        constants,
        entry_name: program.entry_name,
        entry: program.entry,
        result_kind: analysis.kind,
        dynamic,
        canonical_semantics: canonical,
    })
}

pub fn lower_dynamic_plan_v07(plan: &V07DynamicPlan) -> DynamicInputResult<V07DynamicNairArtifact> {
    let mut lowerer = DynamicLowerer::new(plan);
    let lowered = lowerer.lower_expr(&plan.entry)?;
    lowerer.instructions.push(Instruction::Halt);
    let program = NairProgram::from_instructions(lowerer.instructions);
    program
        .validate()
        .map_err(|error| DynamicInputError::Invariant {
            message: format!("V0.7 generated NAIR failed validation: {error}"),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| DynamicInputError::Invariant {
                message: format!("V0.7 generated NAIR failed canonical encoding: {error}"),
            })?;

    if plan.is_dynamic() && program.required_format_minor() != 9 {
        return Err(DynamicInputError::Invariant {
            message: "dynamic V0.7 program must require NAIR 0.9 input-register semantics"
                .to_owned(),
        });
    }
    if !plan.is_dynamic() && (program.required_format_minor() != 6 || program.len() != 2) {
        return Err(DynamicInputError::Invariant {
            message: "static V0.7 subset must still collapse to base NAIR 0.6 CONST + HALT"
                .to_owned(),
        });
    }

    let mut witness = Vec::new();
    witness.extend_from_slice(V07_WITNESS_DOMAIN);
    push_component(&mut witness, plan.canonical_v07_semantic_bytes());
    push_component(&mut witness, &canonical_nair);

    Ok(V07DynamicNairArtifact {
        program,
        result_register: lowered.register,
        canonical_nair,
        witness,
    })
}

pub fn execute_dynamic_source_v07(
    source: &SourceText,
    input: &InputBatch,
) -> DynamicInputResult<V07DynamicExecutionReport> {
    let plan = compile_dynamic_plan_v07(source)?;
    let lowering = lower_dynamic_plan_v07(&plan)?;
    let runtime = run_closed_observed(lowering.program(), input)?;
    let value = runtime
        .register(lowering.result_register())
        .ok_or_else(|| DynamicInputError::Invariant {
            message: format!(
                "V0.7 result register r{} is missing after runtime execution",
                lowering.result_register().0
            ),
        })?;
    let result = match (plan.result_kind(), value) {
        (DynamicValueKind::Int, Value::Int(value)) => DynamicValue::Int(*value),
        (DynamicValueKind::Bool, Value::Bool(value)) => DynamicValue::Bool(*value),
        (expected, actual) => {
            return Err(DynamicInputError::Invariant {
                message: format!(
                    "V0.7 result kind mismatch: expected {}, got {actual:?}",
                    expected.as_str()
                ),
            });
        }
    };

    let input_bytes = input.canonical_bytes()?;
    let mut receipt = Vec::new();
    receipt.extend_from_slice(V07_RECEIPT_DOMAIN);
    push_component(&mut receipt, lowering.canonical_v07_witness_bytes());
    push_component(&mut receipt, &input_bytes);
    receipt.extend_from_slice(&runtime.runtime().replay_key.value().to_be_bytes());
    result.encode(&mut receipt);

    Ok(V07DynamicExecutionReport {
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
    expr: &DynamicExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
) -> DynamicInputResult<ExprAnalysis> {
    match expr {
        DynamicExpr::Int(_) => Ok(ExprAnalysis {
            kind: DynamicValueKind::Int,
            dynamic: false,
        }),
        DynamicExpr::Name(name) => {
            if input_name == Some(name.as_str()) {
                return Ok(ExprAnalysis {
                    kind: DynamicValueKind::Int,
                    dynamic: true,
                });
            }
            let value = constants
                .get(name)
                .ok_or_else(|| DynamicInputError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                })?;
            Ok(ExprAnalysis {
                kind: value.kind(),
                dynamic: false,
            })
        }
        DynamicExpr::Add(lhs, rhs) => {
            let lhs = analyze_expr(lhs, constants, input_name)?;
            let rhs = analyze_expr(rhs, constants, input_name)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(DynamicInputError::Semantic {
                    message: "checked '+' requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Int,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        DynamicExpr::Compare { lhs, rhs, .. } => {
            let lhs = analyze_expr(lhs, constants, input_name)?;
            let rhs = analyze_expr(rhs, constants, input_name)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(DynamicInputError::Semantic {
                    message: "integer comparison requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Bool,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
    }
}

fn eval_static(
    expr: &DynamicExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
) -> DynamicInputResult<Option<DynamicValue>> {
    match expr {
        DynamicExpr::Int(value) => Ok(Some(DynamicValue::Int(*value))),
        DynamicExpr::Name(name) if input_name == Some(name.as_str()) => Ok(None),
        DynamicExpr::Name(name) => {
            constants
                .get(name)
                .copied()
                .map(Some)
                .ok_or_else(|| DynamicInputError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                })
        }
        DynamicExpr::Add(lhs, rhs) => {
            let Some(DynamicValue::Int(lhs)) = eval_static(lhs, constants, input_name)? else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) = eval_static(rhs, constants, input_name)? else {
                return Ok(None);
            };
            let value = lhs
                .checked_add(rhs)
                .ok_or_else(|| DynamicInputError::Semantic {
                    message: "checked integer addition overflow".to_owned(),
                    span: None,
                })?;
            Ok(Some(DynamicValue::Int(value)))
        }
        DynamicExpr::Compare { op, lhs, rhs } => {
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
    }
}

#[derive(Debug, Clone, Copy)]
struct LoweredExpr {
    register: RegisterId,
}

struct DynamicLowerer<'a> {
    plan: &'a V07DynamicPlan,
    instructions: Vec<Instruction>,
    next_register: u32,
    input_register: Option<RegisterId>,
}

impl<'a> DynamicLowerer<'a> {
    fn new(plan: &'a V07DynamicPlan) -> Self {
        Self {
            plan,
            instructions: Vec::new(),
            next_register: 0,
            input_register: None,
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
            value: value.to_runtime(),
        });
        LoweredExpr { register }
    }

    fn lower_expr(&mut self, expr: &DynamicExpr) -> DynamicInputResult<LoweredExpr> {
        if let Some(value) = eval_static(expr, &self.plan.constants, self.plan.input_name())? {
            return Ok(self.emit_const(value));
        }

        match expr {
            DynamicExpr::Int(value) => Ok(self.emit_const(DynamicValue::Int(*value))),
            DynamicExpr::Name(name) if self.plan.input_name() == Some(name.as_str()) => {
                if let Some(register) = self.input_register {
                    return Ok(LoweredExpr { register });
                }
                let register = self.allocate();
                self.instructions.push(Instruction::ReadInputKeyCode {
                    dst: register,
                    event_index: 0,
                });
                self.input_register = Some(register);
                Ok(LoweredExpr { register })
            }
            DynamicExpr::Name(name) => {
                let value = self.plan.constants.get(name).copied().ok_or_else(|| {
                    DynamicInputError::Invariant {
                        message: format!("lowerer lost constant '{name}'"),
                    }
                })?;
                Ok(self.emit_const(value))
            }
            DynamicExpr::Add(lhs, rhs) => {
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
            DynamicExpr::Compare { op, lhs, rhs } => {
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
        }
    }
}

struct DynamicParser<'a> {
    source: &'a SourceText,
    tokens: Vec<Token>,
    index: usize,
    expr_nodes: usize,
}

impl<'a> DynamicParser<'a> {
    fn new(source: &'a SourceText, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            expr_nodes: 0,
        }
    }

    fn parse_program(&mut self) -> DynamicInputResult<DynamicProgram> {
        let mut input_name = None;
        let mut constants = Vec::new();
        let mut entry = None;

        while !self.at_end() {
            if self.is_word("input") {
                if input_name.is_some() {
                    return Err(
                        self.error_here("only one canonical integer input is allowed in V0.7")
                    );
                }
                self.advance();
                let name = self.expect_ident("expected input name after 'input'")?;
                self.expect_punct(';', "expected ';' after input declaration")?;
                input_name = Some(name);
            } else if self.is_word("const") {
                if constants.len() >= MAX_V07_CONSTANTS {
                    return Err(self.error_here("V0.7 constant bound exceeded"));
                }
                self.advance();
                let name = self.expect_ident("expected constant name after 'const'")?;
                self.expect_punct('=', "expected '=' after constant name")?;
                let expr = self.parse_expr()?;
                self.expect_punct(';', "expected ';' after constant declaration")?;
                constants.push((name, expr));
            } else if self.is_word("entry") {
                if entry.is_some() {
                    return Err(self.error_here("only one entry is allowed in V0.7"));
                }
                self.advance();
                let name = self.expect_ident("expected entry name after 'entry'")?;
                self.expect_word("returns", "expected 'returns' after entry name")?;
                let expr = self.parse_expr()?;
                self.expect_punct(';', "expected ';' after entry expression")?;
                entry = Some((name, expr));
            } else {
                return Err(
                    self.error_here("expected 'input', 'const', or 'entry' in V0.7 dynamic body")
                );
            }
        }

        let (entry_name, entry) = entry.ok_or_else(|| DynamicInputError::Semantic {
            message: "V0.7 dynamic body requires exactly one entry".to_owned(),
            span: None,
        })?;

        Ok(DynamicProgram {
            input_name,
            constants,
            entry_name,
            entry,
        })
    }

    fn parse_expr(&mut self) -> DynamicInputResult<DynamicExpr> {
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> DynamicInputResult<DynamicExpr> {
        let lhs = self.parse_add()?;
        let Some(op) = self.take_compare_op() else {
            return Ok(lhs);
        };
        let rhs = self.parse_add()?;
        if self.peek_compare_op() {
            return Err(self.error_here("chained comparisons are not part of V0.7"));
        }
        self.node(DynamicExpr::Compare {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self) -> DynamicInputResult<DynamicExpr> {
        let mut expr = self.parse_primary()?;
        while self.is_punct('+') {
            self.advance();
            let rhs = self.parse_primary()?;
            expr = self.node(DynamicExpr::Add(Box::new(expr), Box::new(rhs)))?;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> DynamicInputResult<DynamicExpr> {
        if self.is_punct('(') {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect_punct(')', "expected ')' after expression")?;
            return Ok(expr);
        }

        let token = self.current().ok_or_else(|| DynamicInputError::Semantic {
            message: "unexpected end of V0.7 expression".to_owned(),
            span: None,
        })?;
        match token.kind() {
            TokenKind::NumericCandidate => {
                let text = self.source.slice(token.span())?;
                let value = text.parse::<i64>().map_err(|_| DynamicInputError::Syntax {
                    message: format!("invalid V0.7 integer literal '{text}'"),
                    span: token.span(),
                })?;
                self.advance();
                self.node(DynamicExpr::Int(value))
            }
            TokenKind::Identifier => {
                let name = self.source.slice(token.span())?.to_owned();
                validate_name(&name, Some(token.span()))?;
                self.advance();
                self.node(DynamicExpr::Name(name))
            }
            _ => Err(self.error_here("expected integer, name, or parenthesized expression")),
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

    fn node(&mut self, expr: DynamicExpr) -> DynamicInputResult<DynamicExpr> {
        self.expr_nodes += 1;
        if self.expr_nodes > MAX_V07_EXPR_NODES {
            return Err(self.error_here("V0.7 expression node bound exceeded"));
        }
        Ok(expr)
    }

    fn expect_ident(&mut self, message: &str) -> DynamicInputResult<String> {
        let token = self.current().ok_or_else(|| DynamicInputError::Semantic {
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

    fn expect_word(&mut self, word: &str, message: &str) -> DynamicInputResult<()> {
        if self.is_word(word) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_punct(&mut self, punct: char, message: &str) -> DynamicInputResult<()> {
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

    fn error_here(&self, message: impl Into<String>) -> DynamicInputError {
        let span = self
            .current()
            .map(Token::span)
            .unwrap_or_else(|| self.source.full_span());
        DynamicInputError::Syntax {
            message: message.into(),
            span,
        }
    }
}

fn validate_name(name: &str, span: Option<SourceSpan>) -> DynamicInputResult<()> {
    if name.is_empty() || name.len() > MAX_V07_NAME_BYTES {
        return Err(DynamicInputError::Semantic {
            message: format!("name '{name}' exceeds V0.7 name bounds"),
            span,
        });
    }
    Ok(())
}

fn encode_expr(expr: &DynamicExpr, out: &mut Vec<u8>) {
    match expr {
        DynamicExpr::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        DynamicExpr::Name(name) => {
            out.push(2);
            encode_string(name, out);
        }
        DynamicExpr::Add(lhs, rhs) => {
            out.push(3);
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        DynamicExpr::Compare { op, lhs, rhs } => {
            out.push(4);
            out.push(op.tag());
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
    }
}

fn encode_value_map(values: &BTreeMap<String, DynamicValue>, out: &mut Vec<u8>) {
    out.extend_from_slice(&(values.len() as u32).to_be_bytes());
    for (name, value) in values {
        encode_string(name, out);
        value.encode(out);
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
