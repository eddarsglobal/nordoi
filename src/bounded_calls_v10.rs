use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{compile_resolved_semantic_boundary, CompilerError};
use crate::dynamic_input_v07::{DynamicValue, DynamicValueKind};
use crate::frontend::{lex, LexError, SourceError, SourceSpan, SourceText, Token, TokenKind};
use crate::input::{InputBatch, InputError};
use crate::nair::{CallExpr, Instruction, NairProgram, RegisterId};
use crate::runtime::{run_closed_call_observed, RuntimeCallObservedReport, RuntimeError};
use crate::value::Value;

const V10_WITNESS_DOMAIN: &[u8] = b"NORDOI-V1.0-BOUNDED-RUNTIME-CALLS\0";
const V10_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V1.0-BOUNDED-RUNTIME-CALLS-RECEIPT\0";

pub const MAX_V10_CONSTANTS: usize = 256;
pub const MAX_V10_FUNCTIONS: usize = 32;
pub const MAX_V10_PARAMS: usize = 8;
pub const MAX_V10_EXPR_NODES: usize = 4096;
pub const MAX_V10_NAME_BYTES: usize = 128;
pub const MAX_V10_RUNTIME_CALLS: usize = 32;
pub const MAX_V10_RUNTIME_CALL_DEPTH: usize = 1;

#[derive(Debug)]
pub enum BoundedRuntimeCallError {
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

impl BoundedRuntimeCallError {
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

impl Display for BoundedRuntimeCallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Lex(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => write!(f, "V1.0 call syntax error: {message}"),
            Self::Semantic { message, .. } => write!(f, "V1.0 call semantic error: {message}"),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Invariant { message } => write!(f, "V1.0 call invariant failed: {message}"),
        }
    }
}

impl Error for BoundedRuntimeCallError {
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

impl From<CompilerError> for BoundedRuntimeCallError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<LexError> for BoundedRuntimeCallError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<SourceError> for BoundedRuntimeCallError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<RuntimeError> for BoundedRuntimeCallError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for BoundedRuntimeCallError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type BoundedRuntimeCallResult<T> = Result<T, BoundedRuntimeCallError>;

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
enum RuntimeExpr {
    Int(i64),
    Name(String),
    Add(Box<RuntimeExpr>, Box<RuntimeExpr>),
    Compare {
        op: CompareOp,
        lhs: Box<RuntimeExpr>,
        rhs: Box<RuntimeExpr>,
    },
    Call {
        name: String,
        args: Vec<RuntimeExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeFunction {
    name: String,
    params: Vec<String>,
    body: RuntimeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeCallProgram {
    input_name: Option<String>,
    constants: Vec<(String, RuntimeExpr)>,
    functions: Vec<RuntimeFunction>,
    entry_name: String,
    entry: RuntimeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CertifiedFunction {
    id: u32,
    params: Vec<String>,
    body: RuntimeExpr,
    result_kind: DynamicValueKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V10RuntimeCallPlan {
    module: Option<String>,
    input_name: Option<String>,
    constants: BTreeMap<String, DynamicValue>,
    functions: BTreeMap<String, CertifiedFunction>,
    entry_name: String,
    entry: RuntimeExpr,
    result_kind: DynamicValueKind,
    dynamic: bool,
    canonical_semantics: Vec<u8>,
}

impl V10RuntimeCallPlan {
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

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn result_kind(&self) -> DynamicValueKind {
        self.result_kind
    }

    pub fn is_dynamic(&self) -> bool {
        self.dynamic
    }

    pub fn canonical_v10_semantic_bytes(&self) -> &[u8] {
        &self.canonical_semantics
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V10RuntimeCallNairArtifact {
    program: NairProgram,
    result_register: RegisterId,
    canonical_nair: Vec<u8>,
    witness: Vec<u8>,
    runtime_call_count: usize,
}

impl V10RuntimeCallNairArtifact {
    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn result_register(&self) -> RegisterId {
        self.result_register
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn canonical_v10_witness_bytes(&self) -> &[u8] {
        &self.witness
    }

    pub fn nair_instruction_count(&self) -> usize {
        self.program.len()
    }

    pub fn nair_format_minor(&self) -> u16 {
        self.program.required_format_minor()
    }

    pub fn runtime_call_count(&self) -> usize {
        self.runtime_call_count
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V10RuntimeCallExecutionReport {
    plan: V10RuntimeCallPlan,
    lowering: V10RuntimeCallNairArtifact,
    runtime: RuntimeCallObservedReport,
    result: DynamicValue,
    receipt: Vec<u8>,
}

impl V10RuntimeCallExecutionReport {
    pub fn plan(&self) -> &V10RuntimeCallPlan {
        &self.plan
    }

    pub fn lowering(&self) -> &V10RuntimeCallNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeCallObservedReport {
        &self.runtime
    }

    pub fn result(&self) -> DynamicValue {
        self.result
    }

    pub fn runtime_computed(&self) -> bool {
        self.plan.is_dynamic()
    }

    pub fn runtime_calls(&self) -> usize {
        self.runtime.call_work().runtime_calls
    }

    pub fn call_body_instructions(&self) -> usize {
        self.runtime.call_work().call_body_instructions
    }

    pub fn max_call_depth(&self) -> usize {
        self.runtime.call_work().max_call_depth
    }

    pub fn runtime_branches(&self) -> usize {
        self.runtime.runtime().execution.execution.runtime_branches
    }

    pub fn canonical_v10_receipt_bytes(&self) -> &[u8] {
        &self.receipt
    }
}

pub fn compile_bounded_runtime_call_plan_v10(
    source: &SourceText,
) -> BoundedRuntimeCallResult<V10RuntimeCallPlan> {
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

    let mut parser = RuntimeCallParser::new(source, tokens);
    let program = parser.parse_program()?;

    let mut all_names = BTreeSet::new();
    if let Some(input_name) = &program.input_name {
        validate_name(input_name, None)?;
        all_names.insert(input_name.clone());
    }

    let mut constants = BTreeMap::new();
    for (name, expr) in &program.constants {
        validate_name(name, None)?;
        if !all_names.insert(name.clone()) {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!("duplicate V1.0 top-level name '{name}'"),
                span: None,
            });
        }
        if contains_call(expr) {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!("constant '{name}' cannot call functions in V1.0"),
                span: None,
            });
        }
        let value =
            eval_static_no_call(expr, &constants, None, &BTreeMap::new())?.ok_or_else(|| {
                BoundedRuntimeCallError::Semantic {
                    message: format!("constant '{name}' must be fully static"),
                    span: None,
                }
            })?;
        constants.insert(name.clone(), value);
    }

    if program.functions.len() > MAX_V10_FUNCTIONS {
        return Err(BoundedRuntimeCallError::Semantic {
            message: "V1.0 function bound exceeded".to_owned(),
            span: None,
        });
    }

    let mut sorted_functions = program.functions.clone();
    sorted_functions.sort_by(|a, b| a.name.cmp(&b.name));
    let mut functions = BTreeMap::new();
    for (index, function) in sorted_functions.into_iter().enumerate() {
        validate_name(&function.name, None)?;
        if !all_names.insert(function.name.clone()) {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!("duplicate V1.0 top-level name '{}'", function.name),
                span: None,
            });
        }
        if function.params.len() > MAX_V10_PARAMS {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!(
                    "function '{}' exceeds the V1.0 parameter bound",
                    function.name
                ),
                span: None,
            });
        }
        let mut params = BTreeSet::new();
        for param in &function.params {
            validate_name(param, None)?;
            if !params.insert(param.clone()) {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: format!("function '{}' repeats parameter '{param}'", function.name),
                    span: None,
                });
            }
        }
        if contains_call(&function.body) {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!(
                    "function '{}' cannot call another function in V1.0; runtime call depth is fixed at 1",
                    function.name
                ),
                span: None,
            });
        }
        if program.input_name.as_deref().is_some_and(|input_name| {
            !function.params.iter().any(|param| param == input_name)
                && contains_name(&function.body, Some(input_name))
        }) {
            return Err(BoundedRuntimeCallError::Semantic {
                message: format!(
                    "function '{}' must receive dynamic input through explicit parameters",
                    function.name
                ),
                span: None,
            });
        }
        let param_kinds = function
            .params
            .iter()
            .map(|param| (param.clone(), DynamicValueKind::Int))
            .collect::<BTreeMap<_, _>>();
        let analysis = analyze_no_call_expr(&function.body, &constants, &param_kinds)?;
        functions.insert(
            function.name.clone(),
            CertifiedFunction {
                id: u32::try_from(index).map_err(|_| BoundedRuntimeCallError::Invariant {
                    message: "function id overflow".to_owned(),
                })?,
                params: function.params,
                body: function.body,
                result_kind: analysis.kind,
            },
        );
    }

    if functions.is_empty() {
        return Err(BoundedRuntimeCallError::Semantic {
            message: "V1.0 requires at least one pure function declaration".to_owned(),
            span: None,
        });
    }

    validate_name(&program.entry_name, None)?;
    if !contains_call(&program.entry) {
        return Err(BoundedRuntimeCallError::Semantic {
            message: "V1.0 entry must contain at least one direct function call".to_owned(),
            span: None,
        });
    }

    let analysis = analyze_entry_expr(
        &program.entry,
        &constants,
        program.input_name.as_deref(),
        &functions,
        true,
    )?;

    let mut canonical = Vec::new();
    canonical.extend_from_slice(V10_WITNESS_DOMAIN);
    encode_optional_string(program.input_name.as_deref(), &mut canonical);
    encode_value_map(&constants, &mut canonical);
    canonical.extend_from_slice(&(functions.len() as u32).to_be_bytes());
    for (name, function) in &functions {
        encode_string(name, &mut canonical);
        canonical.extend_from_slice(&function.id.to_be_bytes());
        canonical.extend_from_slice(&(function.params.len() as u32).to_be_bytes());
        for param in &function.params {
            encode_string(param, &mut canonical);
        }
        encode_expr(&function.body, &mut canonical);
        canonical.push(match function.result_kind {
            DynamicValueKind::Int => 1,
            DynamicValueKind::Bool => 2,
        });
    }
    encode_string(&program.entry_name, &mut canonical);
    encode_expr(&program.entry, &mut canonical);
    canonical.push(match analysis.kind {
        DynamicValueKind::Int => 1,
        DynamicValueKind::Bool => 2,
    });
    canonical.push(u8::from(analysis.dynamic));

    Ok(V10RuntimeCallPlan {
        module: semantic.module().canonical_text(),
        input_name: program.input_name,
        constants,
        functions,
        entry_name: program.entry_name,
        entry: program.entry,
        result_kind: analysis.kind,
        dynamic: analysis.dynamic,
        canonical_semantics: canonical,
    })
}

pub fn lower_bounded_runtime_call_plan_v10(
    plan: &V10RuntimeCallPlan,
) -> BoundedRuntimeCallResult<V10RuntimeCallNairArtifact> {
    let mut lowerer = RuntimeCallLowerer::new(plan);
    let lowered = lowerer.lower_expr(&plan.entry)?;
    lowerer.instructions.push(Instruction::Halt);

    if lowerer.runtime_call_count > MAX_V10_RUNTIME_CALLS {
        return Err(BoundedRuntimeCallError::Semantic {
            message: "V1.0 runtime call bound exceeded".to_owned(),
            span: None,
        });
    }

    let program = NairProgram::from_instructions(lowerer.instructions);
    program
        .validate()
        .map_err(|error| BoundedRuntimeCallError::Invariant {
            message: format!("V1.0 generated NAIR failed validation: {error}"),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| BoundedRuntimeCallError::Invariant {
                message: format!("V1.0 generated NAIR failed canonical encoding: {error}"),
            })?;

    if lowerer.runtime_call_count > 0 && program.required_format_minor() != 12 {
        return Err(BoundedRuntimeCallError::Invariant {
            message: "dynamic V1.0 call must require NAIR 0.12 semantics".to_owned(),
        });
    }
    if !plan.is_dynamic() && (program.required_format_minor() != 6 || program.len() != 2) {
        return Err(BoundedRuntimeCallError::Invariant {
            message: "fully static V1.0 source must collapse to base NAIR 0.6 CONST + HALT"
                .to_owned(),
        });
    }

    let mut witness = Vec::new();
    witness.extend_from_slice(V10_WITNESS_DOMAIN);
    push_component(&mut witness, plan.canonical_v10_semantic_bytes());
    push_component(&mut witness, &canonical_nair);

    Ok(V10RuntimeCallNairArtifact {
        program,
        result_register: lowered.register,
        canonical_nair,
        witness,
        runtime_call_count: lowerer.runtime_call_count,
    })
}

pub fn execute_bounded_runtime_call_source_v10(
    source: &SourceText,
    input: &InputBatch,
) -> BoundedRuntimeCallResult<V10RuntimeCallExecutionReport> {
    let plan = compile_bounded_runtime_call_plan_v10(source)?;
    let lowering = lower_bounded_runtime_call_plan_v10(&plan)?;
    let runtime = run_closed_call_observed(lowering.program(), input)?;
    let value = runtime
        .register(lowering.result_register())
        .ok_or_else(|| BoundedRuntimeCallError::Invariant {
            message: format!(
                "V1.0 result register r{} is missing after runtime execution",
                lowering.result_register().0
            ),
        })?;
    let result = match (plan.result_kind(), value) {
        (DynamicValueKind::Int, Value::Int(value)) => DynamicValue::Int(*value),
        (DynamicValueKind::Bool, Value::Bool(value)) => DynamicValue::Bool(*value),
        (expected, actual) => {
            return Err(BoundedRuntimeCallError::Invariant {
                message: format!(
                    "V1.0 result kind mismatch: expected {}, got {actual:?}",
                    expected.as_str()
                ),
            });
        }
    };

    if runtime.call_work().runtime_calls != lowering.runtime_call_count() {
        return Err(BoundedRuntimeCallError::Invariant {
            message: format!(
                "V1.0 runtime call observation mismatch: lowered={} executed={}",
                lowering.runtime_call_count(),
                runtime.call_work().runtime_calls
            ),
        });
    }
    if runtime.call_work().max_call_depth > MAX_V10_RUNTIME_CALL_DEPTH {
        return Err(BoundedRuntimeCallError::Invariant {
            message: format!(
                "V1.0 runtime call depth exceeded: observed={}",
                runtime.call_work().max_call_depth
            ),
        });
    }
    if runtime.runtime().execution.execution.runtime_branches != 0 {
        return Err(BoundedRuntimeCallError::Invariant {
            message: "V1.0 runtime call slice must not introduce runtime branches".to_owned(),
        });
    }

    let mut receipt = Vec::new();
    receipt.extend_from_slice(V10_RECEIPT_DOMAIN);
    push_component(&mut receipt, lowering.canonical_v10_witness_bytes());
    receipt.extend_from_slice(&(runtime.call_work().runtime_calls as u64).to_be_bytes());
    receipt.extend_from_slice(&(runtime.call_work().call_body_instructions as u64).to_be_bytes());
    receipt.extend_from_slice(&(runtime.call_work().max_call_depth as u64).to_be_bytes());
    encode_dynamic_value(result, &mut receipt);

    Ok(V10RuntimeCallExecutionReport {
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

fn analyze_no_call_expr(
    expr: &RuntimeExpr,
    constants: &BTreeMap<String, DynamicValue>,
    param_kinds: &BTreeMap<String, DynamicValueKind>,
) -> BoundedRuntimeCallResult<ExprAnalysis> {
    match expr {
        RuntimeExpr::Int(_) => Ok(ExprAnalysis {
            kind: DynamicValueKind::Int,
            dynamic: false,
        }),
        RuntimeExpr::Name(name) => {
            if let Some(kind) = param_kinds.get(name) {
                return Ok(ExprAnalysis {
                    kind: *kind,
                    dynamic: true,
                });
            }
            let value = constants
                .get(name)
                .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                    message: format!("unknown function-body name '{name}'"),
                    span: None,
                })?;
            Ok(ExprAnalysis {
                kind: value.kind(),
                dynamic: false,
            })
        }
        RuntimeExpr::Add(lhs, rhs) => {
            let lhs = analyze_no_call_expr(lhs, constants, param_kinds)?;
            let rhs = analyze_no_call_expr(rhs, constants, param_kinds)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: "checked '+' requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Int,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        RuntimeExpr::Compare { lhs, rhs, .. } => {
            let lhs = analyze_no_call_expr(lhs, constants, param_kinds)?;
            let rhs = analyze_no_call_expr(rhs, constants, param_kinds)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: "integer comparison requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Bool,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        RuntimeExpr::Call { .. } => Err(BoundedRuntimeCallError::Semantic {
            message: "function bodies cannot call functions in V1.0".to_owned(),
            span: None,
        }),
    }
}

fn analyze_entry_expr(
    expr: &RuntimeExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
    functions: &BTreeMap<String, CertifiedFunction>,
    calls_allowed: bool,
) -> BoundedRuntimeCallResult<ExprAnalysis> {
    match expr {
        RuntimeExpr::Int(_) => Ok(ExprAnalysis {
            kind: DynamicValueKind::Int,
            dynamic: false,
        }),
        RuntimeExpr::Name(name) => {
            if input_name == Some(name.as_str()) {
                return Ok(ExprAnalysis {
                    kind: DynamicValueKind::Int,
                    dynamic: true,
                });
            }
            let value = constants
                .get(name)
                .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                })?;
            Ok(ExprAnalysis {
                kind: value.kind(),
                dynamic: false,
            })
        }
        RuntimeExpr::Add(lhs, rhs) => {
            let lhs = analyze_entry_expr(lhs, constants, input_name, functions, calls_allowed)?;
            let rhs = analyze_entry_expr(rhs, constants, input_name, functions, calls_allowed)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: "checked '+' requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Int,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        RuntimeExpr::Compare { lhs, rhs, .. } => {
            let lhs = analyze_entry_expr(lhs, constants, input_name, functions, calls_allowed)?;
            let rhs = analyze_entry_expr(rhs, constants, input_name, functions, calls_allowed)?;
            if lhs.kind != DynamicValueKind::Int || rhs.kind != DynamicValueKind::Int {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: "integer comparison requires integer operands".to_owned(),
                    span: None,
                });
            }
            Ok(ExprAnalysis {
                kind: DynamicValueKind::Bool,
                dynamic: lhs.dynamic || rhs.dynamic,
            })
        }
        RuntimeExpr::Call { name, args } => {
            if !calls_allowed {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: "nested runtime calls are not part of V1.0".to_owned(),
                    span: None,
                });
            }
            let function =
                functions
                    .get(name)
                    .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                        message: format!("unknown function '{name}'"),
                        span: None,
                    })?;
            if args.len() != function.params.len() {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: format!(
                        "function '{name}' expects {} argument(s), got {}",
                        function.params.len(),
                        args.len()
                    ),
                    span: None,
                });
            }
            let mut dynamic = false;
            for arg in args {
                if contains_call(arg) {
                    return Err(BoundedRuntimeCallError::Semantic {
                        message: "nested function calls are forbidden in V1.0".to_owned(),
                        span: None,
                    });
                }
                let analysis = analyze_entry_expr(arg, constants, input_name, functions, false)?;
                if analysis.kind != DynamicValueKind::Int {
                    return Err(BoundedRuntimeCallError::Semantic {
                        message: format!("function '{name}' parameters are INT-only in V1.0"),
                        span: None,
                    });
                }
                dynamic |= analysis.dynamic;
            }
            Ok(ExprAnalysis {
                kind: function.result_kind,
                dynamic,
            })
        }
    }
}

fn eval_static_no_call(
    expr: &RuntimeExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
    params: &BTreeMap<String, DynamicValue>,
) -> BoundedRuntimeCallResult<Option<DynamicValue>> {
    match expr {
        RuntimeExpr::Int(value) => Ok(Some(DynamicValue::Int(*value))),
        RuntimeExpr::Name(name) if input_name == Some(name.as_str()) => Ok(None),
        RuntimeExpr::Name(name) => {
            if let Some(value) = params.get(name) {
                return Ok(Some(*value));
            }
            constants.get(name).copied().map(Some).ok_or_else(|| {
                BoundedRuntimeCallError::Semantic {
                    message: format!("unknown name '{name}'"),
                    span: None,
                }
            })
        }
        RuntimeExpr::Add(lhs, rhs) => {
            let Some(DynamicValue::Int(lhs)) =
                eval_static_no_call(lhs, constants, input_name, params)?
            else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) =
                eval_static_no_call(rhs, constants, input_name, params)?
            else {
                return Ok(None);
            };
            let value = lhs
                .checked_add(rhs)
                .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                    message: "checked integer addition overflow".to_owned(),
                    span: None,
                })?;
            Ok(Some(DynamicValue::Int(value)))
        }
        RuntimeExpr::Compare { op, lhs, rhs } => {
            let Some(DynamicValue::Int(lhs)) =
                eval_static_no_call(lhs, constants, input_name, params)?
            else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) =
                eval_static_no_call(rhs, constants, input_name, params)?
            else {
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
        RuntimeExpr::Call { .. } => Ok(None),
    }
}

fn eval_static_entry(
    expr: &RuntimeExpr,
    constants: &BTreeMap<String, DynamicValue>,
    input_name: Option<&str>,
    functions: &BTreeMap<String, CertifiedFunction>,
) -> BoundedRuntimeCallResult<Option<DynamicValue>> {
    match expr {
        RuntimeExpr::Call { name, args } => {
            let function =
                functions
                    .get(name)
                    .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                        message: format!("unknown function '{name}'"),
                        span: None,
                    })?;
            if args.len() != function.params.len() {
                return Err(BoundedRuntimeCallError::Semantic {
                    message: format!(
                        "function '{name}' expects {} argument(s), got {}",
                        function.params.len(),
                        args.len()
                    ),
                    span: None,
                });
            }
            let mut params = BTreeMap::new();
            for (param, arg) in function.params.iter().zip(args) {
                let Some(value) = eval_static_entry(arg, constants, input_name, functions)? else {
                    return Ok(None);
                };
                if value.kind() != DynamicValueKind::Int {
                    return Err(BoundedRuntimeCallError::Semantic {
                        message: format!("function '{name}' parameters are INT-only in V1.0"),
                        span: None,
                    });
                }
                params.insert(param.clone(), value);
            }
            eval_static_no_call(&function.body, constants, None, &params)
        }
        RuntimeExpr::Int(_) | RuntimeExpr::Name(_) => {
            eval_static_no_call(expr, constants, input_name, &BTreeMap::new())
        }
        RuntimeExpr::Add(lhs, rhs) => {
            let Some(DynamicValue::Int(lhs)) =
                eval_static_entry(lhs, constants, input_name, functions)?
            else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) =
                eval_static_entry(rhs, constants, input_name, functions)?
            else {
                return Ok(None);
            };
            let value = lhs
                .checked_add(rhs)
                .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                    message: "checked integer addition overflow".to_owned(),
                    span: None,
                })?;
            Ok(Some(DynamicValue::Int(value)))
        }
        RuntimeExpr::Compare { op, lhs, rhs } => {
            let Some(DynamicValue::Int(lhs)) =
                eval_static_entry(lhs, constants, input_name, functions)?
            else {
                return Ok(None);
            };
            let Some(DynamicValue::Int(rhs)) =
                eval_static_entry(rhs, constants, input_name, functions)?
            else {
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

struct RuntimeCallLowerer<'a> {
    plan: &'a V10RuntimeCallPlan,
    instructions: Vec<Instruction>,
    next_register: u32,
    input_register: Option<RegisterId>,
    runtime_call_count: usize,
}

impl<'a> RuntimeCallLowerer<'a> {
    fn new(plan: &'a V10RuntimeCallPlan) -> Self {
        Self {
            plan,
            instructions: Vec::new(),
            next_register: 0,
            input_register: None,
            runtime_call_count: 0,
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

    fn lower_expr(&mut self, expr: &RuntimeExpr) -> BoundedRuntimeCallResult<LoweredExpr> {
        if let Some(value) = eval_static_entry(
            expr,
            &self.plan.constants,
            self.plan.input_name(),
            &self.plan.functions,
        )? {
            return Ok(self.emit_const(value));
        }

        match expr {
            RuntimeExpr::Int(value) => Ok(self.emit_const(DynamicValue::Int(*value))),
            RuntimeExpr::Name(name) if self.plan.input_name() == Some(name.as_str()) => {
                Ok(LoweredExpr {
                    register: self.ensure_input_register(),
                })
            }
            RuntimeExpr::Name(name) => {
                let value = self.plan.constants.get(name).copied().ok_or_else(|| {
                    BoundedRuntimeCallError::Invariant {
                        message: format!("V1.0 lowerer lost constant '{name}'"),
                    }
                })?;
                Ok(self.emit_const(value))
            }
            RuntimeExpr::Add(lhs, rhs) => {
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
            RuntimeExpr::Compare { op, lhs, rhs } => {
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
            RuntimeExpr::Call { name, args } => {
                let function = self.plan.functions.get(name).ok_or_else(|| {
                    BoundedRuntimeCallError::Invariant {
                        message: format!("V1.0 lowerer lost function '{name}'"),
                    }
                })?;
                if args.len() != function.params.len() {
                    return Err(BoundedRuntimeCallError::Invariant {
                        message: format!("V1.0 call arity changed for function '{name}'"),
                    });
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    let lowered = self.lower_expr(arg)?;
                    lowered_args.push(lowered.register);
                }
                let param_index = function
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, name)| (name.clone(), index as u16))
                    .collect::<BTreeMap<_, _>>();
                let body = lower_call_expr(&function.body, &self.plan.constants, &param_index)?;
                let dst = self.allocate();
                self.instructions.push(Instruction::CallEval {
                    dst,
                    function_id: function.id,
                    args: lowered_args,
                    body,
                });
                self.runtime_call_count += 1;
                Ok(LoweredExpr { register: dst })
            }
        }
    }
}

fn lower_call_expr(
    expr: &RuntimeExpr,
    constants: &BTreeMap<String, DynamicValue>,
    params: &BTreeMap<String, u16>,
) -> BoundedRuntimeCallResult<CallExpr> {
    match expr {
        RuntimeExpr::Int(value) => Ok(CallExpr::Value(Value::Int(*value))),
        RuntimeExpr::Name(name) => {
            if let Some(index) = params.get(name) {
                return Ok(CallExpr::Parameter(*index));
            }
            let value =
                constants
                    .get(name)
                    .copied()
                    .ok_or_else(|| BoundedRuntimeCallError::Invariant {
                        message: format!("V1.0 call-body lowerer lost name '{name}'"),
                    })?;
            Ok(CallExpr::Value(dynamic_value_to_runtime(value)))
        }
        RuntimeExpr::Add(lhs, rhs) => Ok(CallExpr::IntAddChecked {
            lhs: Box::new(lower_call_expr(lhs, constants, params)?),
            rhs: Box::new(lower_call_expr(rhs, constants, params)?),
        }),
        RuntimeExpr::Compare { op, lhs, rhs } => {
            let lhs = Box::new(lower_call_expr(lhs, constants, params)?);
            let rhs = Box::new(lower_call_expr(rhs, constants, params)?);
            Ok(match op {
                CompareOp::Eq => CallExpr::IntEq { lhs, rhs },
                CompareOp::Ne => CallExpr::IntNe { lhs, rhs },
                CompareOp::Lt => CallExpr::IntLt { lhs, rhs },
                CompareOp::Le => CallExpr::IntLe { lhs, rhs },
                CompareOp::Gt => CallExpr::IntGt { lhs, rhs },
                CompareOp::Ge => CallExpr::IntGe { lhs, rhs },
            })
        }
        RuntimeExpr::Call { .. } => Err(BoundedRuntimeCallError::Invariant {
            message: "V1.0 function body unexpectedly contains a nested call".to_owned(),
        }),
    }
}

struct RuntimeCallParser<'a> {
    source: &'a SourceText,
    tokens: Vec<Token>,
    index: usize,
    expr_nodes: usize,
}

impl<'a> RuntimeCallParser<'a> {
    fn new(source: &'a SourceText, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            expr_nodes: 0,
        }
    }

    fn parse_program(&mut self) -> BoundedRuntimeCallResult<RuntimeCallProgram> {
        let mut input_name = None;
        let mut constants = Vec::new();
        let mut functions = Vec::new();
        let mut entry = None;

        while !self.at_end() {
            if self.is_word("input") {
                if input_name.is_some() {
                    return Err(
                        self.error_here("only one canonical integer input is allowed in V1.0")
                    );
                }
                self.advance();
                let name = self.expect_ident("expected input name after 'input'")?;
                self.expect_punct(';', "expected ';' after input declaration")?;
                input_name = Some(name);
            } else if self.is_word("const") {
                if constants.len() >= MAX_V10_CONSTANTS {
                    return Err(self.error_here("V1.0 constant bound exceeded"));
                }
                self.advance();
                let name = self.expect_ident("expected constant name after 'const'")?;
                self.expect_punct('=', "expected '=' after constant name")?;
                let expr = self.parse_expr(true)?;
                self.expect_punct(';', "expected ';' after constant declaration")?;
                constants.push((name, expr));
            } else if self.is_word("fn") {
                if functions.len() >= MAX_V10_FUNCTIONS {
                    return Err(self.error_here("V1.0 function bound exceeded"));
                }
                functions.push(self.parse_function()?);
            } else if self.is_word("entry") {
                if entry.is_some() {
                    return Err(self.error_here("only one entry is allowed in V1.0"));
                }
                self.advance();
                let name = self.expect_ident("expected entry name after 'entry'")?;
                self.expect_word("returns", "expected 'returns' after entry name")?;
                let expr = self.parse_expr(true)?;
                self.expect_punct(';', "expected ';' after entry expression")?;
                entry = Some((name, expr));
            } else {
                return Err(self
                    .error_here("expected 'input', 'const', 'fn', or 'entry' in V1.0 call body"));
            }
        }

        let (entry_name, entry) = entry.ok_or_else(|| BoundedRuntimeCallError::Semantic {
            message: "V1.0 call body requires exactly one entry".to_owned(),
            span: None,
        })?;

        Ok(RuntimeCallProgram {
            input_name,
            constants,
            functions,
            entry_name,
            entry,
        })
    }

    fn parse_function(&mut self) -> BoundedRuntimeCallResult<RuntimeFunction> {
        self.expect_word("fn", "expected 'fn'")?;
        let name = self.expect_ident("expected function name after 'fn'")?;
        self.expect_punct('(', "expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.is_punct(')') {
            loop {
                if params.len() >= MAX_V10_PARAMS {
                    return Err(self.error_here("V1.0 function parameter bound exceeded"));
                }
                params.push(self.expect_ident("expected function parameter name")?);
                if self.is_punct(',') {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect_punct(')', "expected ')' after function parameters")?;
        self.expect_word("returns", "expected 'returns' after function parameters")?;
        let body = self.parse_expr(false)?;
        self.expect_punct(';', "expected ';' after function body")?;
        Ok(RuntimeFunction { name, params, body })
    }

    fn parse_expr(&mut self, calls_allowed: bool) -> BoundedRuntimeCallResult<RuntimeExpr> {
        self.parse_compare(calls_allowed)
    }

    fn parse_compare(&mut self, calls_allowed: bool) -> BoundedRuntimeCallResult<RuntimeExpr> {
        let lhs = self.parse_add(calls_allowed)?;
        let Some(op) = self.take_compare_op() else {
            return Ok(lhs);
        };
        let rhs = self.parse_add(calls_allowed)?;
        if self.peek_compare_op() {
            return Err(self.error_here("chained comparisons are not part of V1.0"));
        }
        self.node(RuntimeExpr::Compare {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self, calls_allowed: bool) -> BoundedRuntimeCallResult<RuntimeExpr> {
        let mut expr = self.parse_primary(calls_allowed)?;
        while self.is_punct('+') {
            self.advance();
            let rhs = self.parse_primary(calls_allowed)?;
            expr = self.node(RuntimeExpr::Add(Box::new(expr), Box::new(rhs)))?;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self, calls_allowed: bool) -> BoundedRuntimeCallResult<RuntimeExpr> {
        if self.is_punct('(') {
            self.advance();
            let expr = self.parse_expr(calls_allowed)?;
            self.expect_punct(')', "expected ')' after expression")?;
            return Ok(expr);
        }

        let token = self
            .current()
            .ok_or_else(|| BoundedRuntimeCallError::Semantic {
                message: "unexpected end of V1.0 expression".to_owned(),
                span: None,
            })?;
        match token.kind() {
            TokenKind::NumericCandidate => {
                let text = self.source.slice(token.span())?;
                let value = text
                    .parse::<i64>()
                    .map_err(|_| BoundedRuntimeCallError::Syntax {
                        message: format!("invalid V1.0 integer literal '{text}'"),
                        span: token.span(),
                    })?;
                self.advance();
                self.node(RuntimeExpr::Int(value))
            }
            TokenKind::Identifier => {
                let name = self.source.slice(token.span())?.to_owned();
                validate_name(&name, Some(token.span()))?;
                self.advance();
                if self.is_punct('(') {
                    if !calls_allowed {
                        return Err(
                            self.error_here("function bodies cannot call functions in V1.0")
                        );
                    }
                    self.advance();
                    let mut args = Vec::new();
                    if !self.is_punct(')') {
                        loop {
                            if args.len() >= MAX_V10_PARAMS {
                                return Err(
                                    self.error_here("V1.0 function-call argument bound exceeded")
                                );
                            }
                            args.push(self.parse_expr(false)?);
                            if self.is_punct(',') {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_punct(')', "expected ')' after function arguments")?;
                    self.node(RuntimeExpr::Call { name, args })
                } else {
                    self.node(RuntimeExpr::Name(name))
                }
            }
            _ => Err(self.error_here(
                "expected integer, name, direct function call, or parenthesized expression",
            )),
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

    fn node(&mut self, expr: RuntimeExpr) -> BoundedRuntimeCallResult<RuntimeExpr> {
        self.expr_nodes += 1;
        if self.expr_nodes > MAX_V10_EXPR_NODES {
            return Err(self.error_here("V1.0 expression node bound exceeded"));
        }
        Ok(expr)
    }

    fn expect_ident(&mut self, message: &str) -> BoundedRuntimeCallResult<String> {
        let token = self
            .current()
            .ok_or_else(|| BoundedRuntimeCallError::Semantic {
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

    fn expect_word(&mut self, word: &str, message: &str) -> BoundedRuntimeCallResult<()> {
        if self.is_word(word) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn expect_punct(&mut self, punct: char, message: &str) -> BoundedRuntimeCallResult<()> {
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

    fn error_here(&self, message: impl Into<String>) -> BoundedRuntimeCallError {
        let span = self
            .current()
            .map(Token::span)
            .unwrap_or_else(|| self.source.full_span());
        BoundedRuntimeCallError::Syntax {
            message: message.into(),
            span,
        }
    }
}

fn contains_call(expr: &RuntimeExpr) -> bool {
    match expr {
        RuntimeExpr::Call { .. } => true,
        RuntimeExpr::Add(lhs, rhs) | RuntimeExpr::Compare { lhs, rhs, .. } => {
            contains_call(lhs) || contains_call(rhs)
        }
        RuntimeExpr::Int(_) | RuntimeExpr::Name(_) => false,
    }
}

fn contains_name(expr: &RuntimeExpr, target: Option<&str>) -> bool {
    let Some(target) = target else {
        return false;
    };
    match expr {
        RuntimeExpr::Name(name) => name == target,
        RuntimeExpr::Add(lhs, rhs) | RuntimeExpr::Compare { lhs, rhs, .. } => {
            contains_name(lhs, Some(target)) || contains_name(rhs, Some(target))
        }
        RuntimeExpr::Call { args, .. } => args.iter().any(|arg| contains_name(arg, Some(target))),
        RuntimeExpr::Int(_) => false,
    }
}

fn validate_name(name: &str, span: Option<SourceSpan>) -> BoundedRuntimeCallResult<()> {
    if name.is_empty() || name.len() > MAX_V10_NAME_BYTES {
        return Err(BoundedRuntimeCallError::Semantic {
            message: format!("name '{name}' exceeds V1.0 name bounds"),
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

fn encode_expr(expr: &RuntimeExpr, out: &mut Vec<u8>) {
    match expr {
        RuntimeExpr::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        RuntimeExpr::Name(name) => {
            out.push(2);
            encode_string(name, out);
        }
        RuntimeExpr::Add(lhs, rhs) => {
            out.push(3);
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        RuntimeExpr::Compare { op, lhs, rhs } => {
            out.push(4);
            out.push(op.tag());
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        RuntimeExpr::Call { name, args } => {
            out.push(5);
            encode_string(name, out);
            out.extend_from_slice(&(args.len() as u32).to_be_bytes());
            for arg in args {
                encode_expr(arg, out);
            }
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

fn encode_optional_string(value: Option<&str>, out: &mut Vec<u8>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_string(value, out);
        }
        None => out.push(0),
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
