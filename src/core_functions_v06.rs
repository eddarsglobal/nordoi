use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::compiler::{compile_resolved_semantic_boundary, CompilerError};
use crate::frontend::{ByteOffset, SourceError, SourceSpan, SourceText};
use crate::input::{InputBatch, InputError};
use crate::nair::{Instruction, NairProgram, RegisterId};
use crate::runtime::{run_closed_observed, RuntimeError, RuntimeObservedReport};
use crate::value::Value;

const V06_CORE_WITNESS_DOMAIN: &[u8] = b"NORDOI-V0.6-CORE-FUNCTIONS\0";
const V06_CORE_RECEIPT_DOMAIN: &[u8] = b"NORDOI-V0.6-CORE-FUNCTIONS-EXECUTION-RECEIPT\0";

pub const MAX_V06_FUNCTIONS: usize = 128;
pub const MAX_V06_PARAMS: usize = 32;
pub const MAX_V06_BINDINGS: usize = 256;
pub const MAX_V06_EXPR_NODES: usize = 4096;
pub const MAX_V06_CALL_DEPTH: usize = 64;
pub const MAX_V06_NAME_BYTES: usize = 128;

#[derive(Debug)]
pub enum CoreFunctionsError {
    Compiler(CompilerError),
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

impl CoreFunctionsError {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Compiler(error) => error.primary_span(),
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

impl Display for CoreFunctionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => Display::fmt(error, f),
            Self::Source(error) => Display::fmt(error, f),
            Self::Syntax { message, .. } => write!(f, "V0.6 core syntax error: {message}"),
            Self::Semantic { message, .. } => write!(f, "V0.6 core semantic error: {message}"),
            Self::Runtime(error) => Display::fmt(error, f),
            Self::Invariant { message } => write!(f, "V0.6 core invariant failed: {message}"),
        }
    }
}

impl Error for CoreFunctionsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compiler(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::Syntax { .. } | Self::Semantic { .. } | Self::Invariant { .. } => None,
        }
    }
}

impl From<CompilerError> for CoreFunctionsError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<SourceError> for CoreFunctionsError {
    fn from(value: SourceError) -> Self {
        Self::Source(value)
    }
}

impl From<RuntimeError> for CoreFunctionsError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<InputError> for CoreFunctionsError {
    fn from(value: InputError) -> Self {
        Self::Runtime(RuntimeError::from(value))
    }
}

pub type CoreFunctionsResult<T> = Result<T, CoreFunctionsError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreValue {
    Int(i64),
    Bool(bool),
}

impl CoreValue {
    pub fn as_text(self) -> String {
        match self {
            Self::Int(value) => format!("INT({value})"),
            Self::Bool(value) => format!("BOOL({value})"),
        }
    }

    fn tag(self) -> u8 {
        match self {
            Self::Int(_) => 1,
            Self::Bool(_) => 2,
        }
    }

    fn encode(self, out: &mut Vec<u8>) {
        out.push(self.tag());
        match self {
            Self::Int(value) => out.extend_from_slice(&value.to_be_bytes()),
            Self::Bool(value) => out.push(u8::from(value)),
        }
    }

    fn to_runtime_value(self) -> Value {
        match self {
            Self::Int(value) => Value::Int(value),
            Self::Bool(value) => Value::Bool(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinaryOp {
    fn tag(self) -> u8 {
        match self {
            Self::Add => 1,
            Self::Sub => 2,
            Self::Mul => 3,
            Self::Div => 4,
            Self::Eq => 5,
            Self::Ne => 6,
            Self::Lt => 7,
            Self::Le => 8,
            Self::Gt => 9,
            Self::Ge => 10,
            Self::And => 11,
            Self::Or => 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CoreExpr {
    Int(i64),
    Bool(bool),
    Name(String),
    Unary {
        op: UnaryOp,
        expr: Box<CoreExpr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<CoreExpr>,
        rhs: Box<CoreExpr>,
    },
    Call {
        name: String,
        args: Vec<CoreExpr>,
    },
    If {
        condition: Box<CoreExpr>,
        then_block: CoreBlock,
        else_block: CoreBlock,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct CoreBinding {
    name: String,
    expr: CoreExpr,
}

#[derive(Debug, Clone, PartialEq)]
struct CoreBlock {
    bindings: Vec<CoreBinding>,
    result: Box<CoreExpr>,
}

#[derive(Debug, Clone, PartialEq)]
struct CoreFunction {
    name: String,
    params: Vec<String>,
    body: CoreBlock,
}

#[derive(Debug, Clone, PartialEq)]
struct CoreProgram {
    globals: Vec<CoreBinding>,
    functions: Vec<CoreFunction>,
    entry_name: String,
    entry: CoreExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct EvalStats {
    operation_count: usize,
    call_count: usize,
    if_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct EvalSourceContext<'a> {
    source: &'a SourceText,
    base: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct V06CorePlan {
    module: Option<String>,
    program: CoreProgram,
    globals: BTreeMap<String, CoreValue>,
    result: CoreValue,
    stats: EvalStats,
    canonical_semantics: Vec<u8>,
}

impl V06CorePlan {
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn entry_name(&self) -> &str {
        &self.program.entry_name
    }

    pub fn function_count(&self) -> usize {
        self.program.functions.len()
    }

    pub fn global_binding_count(&self) -> usize {
        self.program.globals.len()
    }

    pub fn source_operation_count(&self) -> usize {
        self.stats.operation_count
    }

    pub fn function_call_count(&self) -> usize {
        self.stats.call_count
    }

    pub fn static_if_count(&self) -> usize {
        self.stats.if_count
    }

    pub fn result(&self) -> CoreValue {
        self.result
    }

    pub fn globals(&self) -> &BTreeMap<String, CoreValue> {
        &self.globals
    }

    pub fn canonical_v06_semantic_bytes(&self) -> &[u8] {
        &self.canonical_semantics
    }

    pub fn runtime_call_count(&self) -> usize {
        0
    }

    pub fn runtime_branch_count(&self) -> usize {
        0
    }

    pub fn inlined_call_count(&self) -> usize {
        self.stats.call_count
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct V06CoreNairArtifact {
    result: CoreValue,
    program: NairProgram,
    canonical_nair: Vec<u8>,
    result_register: RegisterId,
    witness: Vec<u8>,
}

impl V06CoreNairArtifact {
    pub fn program(&self) -> &NairProgram {
        &self.program
    }

    pub fn result(&self) -> CoreValue {
        self.result
    }

    pub fn result_register(&self) -> RegisterId {
        self.result_register
    }

    pub fn canonical_nair_bytes(&self) -> &[u8] {
        &self.canonical_nair
    }

    pub fn canonical_v06_witness_bytes(&self) -> &[u8] {
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
pub struct V06CoreExecutionReport {
    plan: V06CorePlan,
    lowering: V06CoreNairArtifact,
    runtime: RuntimeObservedReport,
    receipt: Vec<u8>,
}

impl V06CoreExecutionReport {
    pub fn plan(&self) -> &V06CorePlan {
        &self.plan
    }

    pub fn lowering(&self) -> &V06CoreNairArtifact {
        &self.lowering
    }

    pub fn runtime(&self) -> &RuntimeObservedReport {
        &self.runtime
    }

    pub fn result(&self) -> CoreValue {
        self.plan.result()
    }

    pub fn canonical_v06_receipt_bytes(&self) -> &[u8] {
        &self.receipt
    }

    pub fn constant_folded(&self) -> bool {
        true
    }
}

pub fn compile_core_plan_v06(source: &SourceText) -> CoreFunctionsResult<V06CorePlan> {
    let semantic = compile_resolved_semantic_boundary(source)?;
    let body_span = semantic.origin().body_span();
    let body = source.slice(body_span)?;
    let mut parser = Parser::new(source, body, body_span.start().get() as usize)?;
    let program = parser.parse_program()?;

    validate_declaration_uniqueness(source, body_span.start().get() as usize, &program)?;
    validate_program_structure(source, body_span.start().get() as usize, &program)?;

    let functions = function_map(&program);
    let mut globals = BTreeMap::new();
    let mut stats = EvalStats::default();
    let mut call_stack = Vec::new();
    let eval_context = EvalSourceContext {
        source,
        base: body_span.start().get() as usize,
    };

    for binding in &program.globals {
        if globals.contains_key(&binding.name) {
            return Err(semantic_error(
                source,
                body_span.start().get() as usize,
                0,
                format!("duplicate global binding '{}'", binding.name),
            ));
        }
        let value = eval_expr(
            &eval_context,
            &binding.expr,
            &globals,
            &functions,
            &globals,
            &mut call_stack,
            &mut stats,
        )?;
        globals.insert(binding.name.clone(), value);
    }

    let result = eval_expr(
        &eval_context,
        &program.entry,
        &globals,
        &functions,
        &globals,
        &mut call_stack,
        &mut stats,
    )?;

    let mut canonical = Vec::new();
    canonical.extend_from_slice(V06_CORE_WITNESS_DOMAIN);
    push_component(&mut canonical, &semantic.canonical_c02_bytes());
    encode_program(&program, &mut canonical);
    encode_value_map(&globals, &mut canonical);
    result.encode(&mut canonical);
    canonical.extend_from_slice(&(stats.operation_count as u32).to_be_bytes());
    canonical.extend_from_slice(&(stats.call_count as u32).to_be_bytes());
    canonical.extend_from_slice(&(stats.if_count as u32).to_be_bytes());

    Ok(V06CorePlan {
        module: semantic.module().canonical_text(),
        program,
        globals,
        result,
        stats,
        canonical_semantics: canonical,
    })
}

pub fn lower_core_plan_v06(plan: &V06CorePlan) -> CoreFunctionsResult<V06CoreNairArtifact> {
    let result_register = RegisterId(0);
    let program = NairProgram::from_instructions(vec![
        Instruction::Const {
            dst: result_register,
            value: plan.result().to_runtime_value(),
        },
        Instruction::Halt,
    ]);
    program
        .validate()
        .map_err(|error| CoreFunctionsError::Invariant {
            message: format!("V0.6 optimized NAIR failed validation: {error}"),
        })?;
    let canonical_nair =
        program
            .canonical_bytes()
            .map_err(|error| CoreFunctionsError::Invariant {
                message: format!("V0.6 optimized NAIR failed canonical encoding: {error}"),
            })?;
    if program.required_format_minor() != 6 || program.len() != 2 {
        return Err(CoreFunctionsError::Invariant {
            message: "closed pure V0.6 program must optimize to CONST + HALT in base NAIR 0.6"
                .to_owned(),
        });
    }

    let mut witness = Vec::new();
    witness.extend_from_slice(V06_CORE_WITNESS_DOMAIN);
    push_component(&mut witness, plan.canonical_v06_semantic_bytes());
    push_component(&mut witness, &canonical_nair);
    plan.result().encode(&mut witness);
    witness.extend_from_slice(&0u32.to_be_bytes());
    witness.extend_from_slice(&0u32.to_be_bytes());

    Ok(V06CoreNairArtifact {
        result: plan.result(),
        program,
        canonical_nair,
        result_register,
        witness,
    })
}

pub fn execute_core_source_v06(source: &SourceText) -> CoreFunctionsResult<V06CoreExecutionReport> {
    let plan = compile_core_plan_v06(source)?;
    let lowering = lower_core_plan_v06(&plan)?;
    let input = InputBatch::default();
    let canonical_input = input.canonical_bytes()?;
    let runtime = run_closed_observed(lowering.program(), &input)?;

    let observed = runtime
        .register(lowering.result_register())
        .ok_or_else(|| CoreFunctionsError::Invariant {
            message: "V0.6 result register is missing after runtime execution".to_owned(),
        })?;
    let expected = lowering.result().to_runtime_value();
    if observed != &expected {
        return Err(CoreFunctionsError::Invariant {
            message: format!(
                "V0.6 runtime result differs from compile-time proof: expected {expected:?}, got {observed:?}"
            ),
        });
    }
    validate_zero_runtime_state(&runtime)?;

    let mut receipt = Vec::new();
    receipt.extend_from_slice(V06_CORE_RECEIPT_DOMAIN);
    push_component(&mut receipt, lowering.canonical_v06_witness_bytes());
    push_component(&mut receipt, &canonical_input);
    receipt.extend_from_slice(&runtime.runtime().replay_key.value().to_be_bytes());
    plan.result().encode(&mut receipt);
    receipt.push(u8::from(runtime.is_quiescent()));

    Ok(V06CoreExecutionReport {
        plan,
        lowering,
        runtime,
        receipt,
    })
}

fn validate_zero_runtime_state(runtime: &RuntimeObservedReport) -> CoreFunctionsResult<()> {
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
        return Err(CoreFunctionsError::Invariant {
            message: "V0.6 pure core must end quiescent with zero persistent state, runtime calls, effects and bridges"
                .to_owned(),
        });
    }
    Ok(())
}

fn function_map(program: &CoreProgram) -> BTreeMap<&str, &CoreFunction> {
    program
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect()
}

fn validate_declaration_uniqueness(
    source: &SourceText,
    base: usize,
    program: &CoreProgram,
) -> CoreFunctionsResult<()> {
    let mut names = BTreeSet::new();
    for binding in &program.globals {
        if !names.insert(binding.name.as_str()) {
            return Err(semantic_error(
                source,
                base,
                0,
                format!("duplicate top-level name '{}'", binding.name),
            ));
        }
    }
    for function in &program.functions {
        if !names.insert(function.name.as_str()) {
            return Err(semantic_error(
                source,
                base,
                0,
                format!("duplicate top-level name '{}'", function.name),
            ));
        }
        let mut params = BTreeSet::new();
        for param in &function.params {
            if !params.insert(param.as_str()) {
                return Err(semantic_error(
                    source,
                    base,
                    0,
                    format!(
                        "function '{}' has duplicate parameter '{param}'",
                        function.name
                    ),
                ));
            }
        }
    }
    if names.contains(program.entry_name.as_str()) {
        return Err(semantic_error(
            source,
            base,
            0,
            format!(
                "entry name '{}' conflicts with a top-level declaration",
                program.entry_name
            ),
        ));
    }
    Ok(())
}

fn validate_program_structure(
    source: &SourceText,
    base: usize,
    program: &CoreProgram,
) -> CoreFunctionsResult<()> {
    let function_arities = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.params.len()))
        .collect::<BTreeMap<_, _>>();
    let global_names = program
        .globals
        .iter()
        .map(|binding| binding.name.clone())
        .collect::<BTreeSet<_>>();

    let mut visible_globals = BTreeSet::new();
    for binding in &program.globals {
        validate_expr_structure(
            source,
            base,
            &binding.expr,
            &visible_globals,
            &function_arities,
        )?;
        visible_globals.insert(binding.name.clone());
    }

    for function in &program.functions {
        let mut scope = global_names.clone();
        scope.extend(function.params.iter().cloned());
        validate_block_structure(source, base, &function.body, &scope, &function_arities)?;
    }

    validate_expr_structure(
        source,
        base,
        &program.entry,
        &global_names,
        &function_arities,
    )?;
    validate_call_graph_acyclic(source, base, program)?;
    Ok(())
}

fn validate_block_structure(
    source: &SourceText,
    base: usize,
    block: &CoreBlock,
    parent_scope: &BTreeSet<String>,
    function_arities: &BTreeMap<String, usize>,
) -> CoreFunctionsResult<()> {
    let mut scope = parent_scope.clone();
    for binding in &block.bindings {
        if scope.contains(&binding.name) {
            return Err(semantic_error(
                source,
                base,
                0,
                format!(
                    "local immutable binding '{}' shadows an existing name",
                    binding.name
                ),
            ));
        }
        validate_expr_structure(source, base, &binding.expr, &scope, function_arities)?;
        scope.insert(binding.name.clone());
    }
    validate_expr_structure(source, base, &block.result, &scope, function_arities)
}

fn validate_expr_structure(
    source: &SourceText,
    base: usize,
    expr: &CoreExpr,
    scope: &BTreeSet<String>,
    function_arities: &BTreeMap<String, usize>,
) -> CoreFunctionsResult<()> {
    match expr {
        CoreExpr::Int(_) | CoreExpr::Bool(_) => Ok(()),
        CoreExpr::Name(name) => {
            if scope.contains(name) {
                Ok(())
            } else {
                Err(semantic_error(
                    source,
                    base,
                    0,
                    format!("unknown immutable binding or parameter '{name}'"),
                ))
            }
        }
        CoreExpr::Unary { expr, .. } => {
            validate_expr_structure(source, base, expr, scope, function_arities)
        }
        CoreExpr::Binary { lhs, rhs, .. } => {
            validate_expr_structure(source, base, lhs, scope, function_arities)?;
            validate_expr_structure(source, base, rhs, scope, function_arities)
        }
        CoreExpr::Call { name, args } => {
            let expected = function_arities.get(name).copied().ok_or_else(|| {
                semantic_error(source, base, 0, format!("unknown pure function '{name}'"))
            })?;
            if expected != args.len() {
                return Err(semantic_error(
                    source,
                    base,
                    0,
                    format!(
                        "function '{name}' expects {expected} argument(s), got {}",
                        args.len()
                    ),
                ));
            }
            for arg in args {
                validate_expr_structure(source, base, arg, scope, function_arities)?;
            }
            Ok(())
        }
        CoreExpr::If {
            condition,
            then_block,
            else_block,
        } => {
            validate_expr_structure(source, base, condition, scope, function_arities)?;
            validate_block_structure(source, base, then_block, scope, function_arities)?;
            validate_block_structure(source, base, else_block, scope, function_arities)
        }
    }
}

fn validate_call_graph_acyclic(
    source: &SourceText,
    base: usize,
    program: &CoreProgram,
) -> CoreFunctionsResult<()> {
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for function in &program.functions {
        let mut calls = BTreeSet::new();
        collect_calls_block(&function.body, &mut calls);
        graph.insert(function.name.clone(), calls);
    }

    fn visit(
        node: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        temporary: &mut BTreeSet<String>,
        permanent: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if permanent.contains(node) {
            return Ok(());
        }
        if !temporary.insert(node.to_owned()) {
            return Err(node.to_owned());
        }
        if let Some(edges) = graph.get(node) {
            for edge in edges {
                if graph.contains_key(edge) {
                    visit(edge, graph, temporary, permanent)?;
                }
            }
        }
        temporary.remove(node);
        permanent.insert(node.to_owned());
        Ok(())
    }

    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for name in graph.keys() {
        if let Err(cycle) = visit(name, &graph, &mut temporary, &mut permanent) {
            return Err(semantic_error(
                source,
                base,
                0,
                format!("recursive pure function cycle includes '{cycle}' and is not part of V0.6"),
            ));
        }
    }
    Ok(())
}

fn collect_calls_block(block: &CoreBlock, calls: &mut BTreeSet<String>) {
    for binding in &block.bindings {
        collect_calls_expr(&binding.expr, calls);
    }
    collect_calls_expr(&block.result, calls);
}

fn collect_calls_expr(expr: &CoreExpr, calls: &mut BTreeSet<String>) {
    match expr {
        CoreExpr::Int(_) | CoreExpr::Bool(_) | CoreExpr::Name(_) => {}
        CoreExpr::Unary { expr, .. } => collect_calls_expr(expr, calls),
        CoreExpr::Binary { lhs, rhs, .. } => {
            collect_calls_expr(lhs, calls);
            collect_calls_expr(rhs, calls);
        }
        CoreExpr::Call { name, args } => {
            calls.insert(name.clone());
            for arg in args {
                collect_calls_expr(arg, calls);
            }
        }
        CoreExpr::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_calls_expr(condition, calls);
            collect_calls_block(then_block, calls);
            collect_calls_block(else_block, calls);
        }
    }
}

fn eval_expr(
    context: &EvalSourceContext<'_>,
    expr: &CoreExpr,
    env: &BTreeMap<String, CoreValue>,
    functions: &BTreeMap<&str, &CoreFunction>,
    globals: &BTreeMap<String, CoreValue>,
    call_stack: &mut Vec<String>,
    stats: &mut EvalStats,
) -> CoreFunctionsResult<CoreValue> {
    match expr {
        CoreExpr::Int(value) => Ok(CoreValue::Int(*value)),
        CoreExpr::Bool(value) => Ok(CoreValue::Bool(*value)),
        CoreExpr::Name(name) => env
            .get(name)
            .or_else(|| globals.get(name))
            .copied()
            .ok_or_else(|| {
                semantic_error(
                    context.source,
                    context.base,
                    0,
                    format!("unknown immutable binding or parameter '{name}'"),
                )
            }),
        CoreExpr::Unary {
            op: UnaryOp::Not,
            expr,
        } => {
            stats.operation_count += 1;
            match eval_expr(context, expr, env, functions, globals, call_stack, stats)? {
                CoreValue::Bool(value) => Ok(CoreValue::Bool(!value)),
                CoreValue::Int(_) => Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    "operator '!' requires BOOL".to_owned(),
                )),
            }
        }
        CoreExpr::Binary { op, lhs, rhs } => {
            stats.operation_count += 1;
            let left = eval_expr(context, lhs, env, functions, globals, call_stack, stats)?;
            let right = eval_expr(context, rhs, env, functions, globals, call_stack, stats)?;
            eval_binary(context.source, context.base, *op, left, right)
        }
        CoreExpr::Call { name, args } => {
            stats.call_count += 1;
            let function = functions.get(name.as_str()).copied().ok_or_else(|| {
                semantic_error(
                    context.source,
                    context.base,
                    0,
                    format!("unknown pure function '{name}'"),
                )
            })?;
            if args.len() != function.params.len() {
                return Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    format!(
                        "function '{}' expects {} argument(s), got {}",
                        name,
                        function.params.len(),
                        args.len()
                    ),
                ));
            }
            if call_stack.len() >= MAX_V06_CALL_DEPTH {
                return Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    format!("pure function call depth exceeds {MAX_V06_CALL_DEPTH}"),
                ));
            }
            if call_stack.iter().any(|active| active == name) {
                return Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    format!("recursive pure function '{name}' is not part of V0.6"),
                ));
            }

            let mut local = globals.clone();
            for (param, arg) in function.params.iter().zip(args) {
                let value = eval_expr(context, arg, env, functions, globals, call_stack, stats)?;
                local.insert(param.clone(), value);
            }

            call_stack.push(name.clone());
            let result = eval_block(
                context,
                &function.body,
                &local,
                functions,
                globals,
                call_stack,
                stats,
            );
            call_stack.pop();
            result
        }
        CoreExpr::If {
            condition,
            then_block,
            else_block,
        } => {
            stats.if_count += 1;
            let condition_value = eval_expr(
                context, condition, env, functions, globals, call_stack, stats,
            )?;
            let then_value = eval_block(
                context, then_block, env, functions, globals, call_stack, stats,
            )?;
            let else_value = eval_block(
                context, else_block, env, functions, globals, call_stack, stats,
            )?;
            if std::mem::discriminant(&then_value) != std::mem::discriminant(&else_value) {
                return Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    "if/else branches must produce the same value kind".to_owned(),
                ));
            }
            match condition_value {
                CoreValue::Bool(true) => Ok(then_value),
                CoreValue::Bool(false) => Ok(else_value),
                CoreValue::Int(_) => Err(semantic_error(
                    context.source,
                    context.base,
                    0,
                    "if condition must be BOOL".to_owned(),
                )),
            }
        }
    }
}

fn eval_block(
    context: &EvalSourceContext<'_>,
    block: &CoreBlock,
    parent: &BTreeMap<String, CoreValue>,
    functions: &BTreeMap<&str, &CoreFunction>,
    globals: &BTreeMap<String, CoreValue>,
    call_stack: &mut Vec<String>,
    stats: &mut EvalStats,
) -> CoreFunctionsResult<CoreValue> {
    let mut local = parent.clone();
    for binding in &block.bindings {
        if local.contains_key(&binding.name) {
            return Err(semantic_error(
                context.source,
                context.base,
                0,
                format!(
                    "local immutable binding '{}' shadows an existing name",
                    binding.name
                ),
            ));
        }
        let value = eval_expr(
            context,
            &binding.expr,
            &local,
            functions,
            globals,
            call_stack,
            stats,
        )?;
        local.insert(binding.name.clone(), value);
    }
    eval_expr(
        context,
        &block.result,
        &local,
        functions,
        globals,
        call_stack,
        stats,
    )
}

fn eval_binary(
    source: &SourceText,
    base: usize,
    op: BinaryOp,
    left: CoreValue,
    right: CoreValue,
) -> CoreFunctionsResult<CoreValue> {
    use BinaryOp::*;
    match op {
        Add | Sub | Mul | Div => {
            let (lhs, rhs) = match (left, right) {
                (CoreValue::Int(lhs), CoreValue::Int(rhs)) => (lhs, rhs),
                _ => {
                    return Err(semantic_error(
                        source,
                        base,
                        0,
                        "arithmetic operators require INT operands".to_owned(),
                    ))
                }
            };
            let value = match op {
                Add => lhs.checked_add(rhs).ok_or_else(|| {
                    semantic_error(
                        source,
                        base,
                        0,
                        "checked integer addition overflow".to_owned(),
                    )
                })?,
                Sub => lhs.checked_sub(rhs).ok_or_else(|| {
                    semantic_error(
                        source,
                        base,
                        0,
                        "checked integer subtraction overflow".to_owned(),
                    )
                })?,
                Mul => lhs.checked_mul(rhs).ok_or_else(|| {
                    semantic_error(
                        source,
                        base,
                        0,
                        "checked integer multiplication overflow".to_owned(),
                    )
                })?,
                Div => {
                    if rhs == 0 {
                        return Err(semantic_error(
                            source,
                            base,
                            0,
                            "checked integer division by zero".to_owned(),
                        ));
                    }
                    lhs.checked_div(rhs).ok_or_else(|| {
                        semantic_error(
                            source,
                            base,
                            0,
                            "checked integer division overflow".to_owned(),
                        )
                    })?
                }
                _ => unreachable!("arithmetic branch only"),
            };
            Ok(CoreValue::Int(value))
        }
        Eq | Ne => {
            let equal = match (left, right) {
                (CoreValue::Int(lhs), CoreValue::Int(rhs)) => lhs == rhs,
                (CoreValue::Bool(lhs), CoreValue::Bool(rhs)) => lhs == rhs,
                _ => {
                    return Err(semantic_error(
                        source,
                        base,
                        0,
                        "equality operands must have the same value kind".to_owned(),
                    ))
                }
            };
            Ok(CoreValue::Bool(if op == Eq { equal } else { !equal }))
        }
        Lt | Le | Gt | Ge => {
            let (lhs, rhs) = match (left, right) {
                (CoreValue::Int(lhs), CoreValue::Int(rhs)) => (lhs, rhs),
                _ => {
                    return Err(semantic_error(
                        source,
                        base,
                        0,
                        "ordered comparisons require INT operands".to_owned(),
                    ))
                }
            };
            let value = match op {
                Lt => lhs < rhs,
                Le => lhs <= rhs,
                Gt => lhs > rhs,
                Ge => lhs >= rhs,
                _ => unreachable!("ordered comparison branch only"),
            };
            Ok(CoreValue::Bool(value))
        }
        And | Or => {
            let (lhs, rhs) = match (left, right) {
                (CoreValue::Bool(lhs), CoreValue::Bool(rhs)) => (lhs, rhs),
                _ => {
                    return Err(semantic_error(
                        source,
                        base,
                        0,
                        "logical operators require BOOL operands".to_owned(),
                    ))
                }
            };
            Ok(CoreValue::Bool(if op == And {
                lhs && rhs
            } else {
                lhs || rhs
            }))
        }
    }
}

fn semantic_error(
    source: &SourceText,
    base: usize,
    relative: usize,
    message: String,
) -> CoreFunctionsError {
    let raw = base
        .saturating_add(relative)
        .min(source.len().get() as usize) as u32;
    let span = source.span(ByteOffset::new(raw), ByteOffset::new(raw)).ok();
    CoreFunctionsError::Semantic { message, span }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKindV06 {
    Ident(String),
    Int(i64),
    Fn,
    Const,
    Entry,
    Returns,
    If,
    Else,
    True,
    False,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TokenV06 {
    kind: TokenKindV06,
    start: usize,
    end: usize,
}

struct Lexer<'a> {
    source: &'a SourceText,
    text: &'a str,
    base: usize,
    index: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a SourceText, text: &'a str, base: usize) -> Self {
        Self {
            source,
            text,
            base,
            index: 0,
        }
    }

    fn lex(mut self) -> CoreFunctionsResult<Vec<TokenV06>> {
        let bytes = self.text.as_bytes();
        let mut tokens = Vec::new();
        while self.index < bytes.len() {
            let byte = bytes[self.index];
            if byte.is_ascii_whitespace() {
                self.index += 1;
                continue;
            }
            if byte == b'/' && self.index + 1 < bytes.len() && bytes[self.index + 1] == b'/' {
                self.index += 2;
                while self.index < bytes.len() && !matches!(bytes[self.index], b'\n' | b'\r') {
                    self.index += 1;
                }
                continue;
            }
            if byte == b'/' && self.index + 1 < bytes.len() && bytes[self.index + 1] == b'*' {
                let start = self.index;
                self.index += 2;
                let mut closed = false;
                while self.index + 1 < bytes.len() {
                    if bytes[self.index] == b'*' && bytes[self.index + 1] == b'/' {
                        self.index += 2;
                        closed = true;
                        break;
                    }
                    self.index += 1;
                }
                if !closed {
                    return Err(self.syntax(start, self.index, "unterminated block comment"));
                }
                continue;
            }
            let start = self.index;
            if byte.is_ascii_digit() {
                self.index += 1;
                while self.index < bytes.len() && bytes[self.index].is_ascii_digit() {
                    self.index += 1;
                }
                let raw = &self.text[start..self.index];
                if raw.len() > 1 && raw.starts_with('0') {
                    return Err(self.syntax(
                        start,
                        self.index,
                        "integer literal must be canonical decimal without leading zeroes",
                    ));
                }
                let value = raw.parse::<i64>().map_err(|_| {
                    self.syntax(
                        start,
                        self.index,
                        "integer literal exceeds signed 64-bit range",
                    )
                })?;
                tokens.push(TokenV06 {
                    kind: TokenKindV06::Int(value),
                    start,
                    end: self.index,
                });
                continue;
            }
            if byte.is_ascii_alphabetic() || byte == b'_' {
                self.index += 1;
                while self.index < bytes.len()
                    && (bytes[self.index].is_ascii_alphanumeric() || bytes[self.index] == b'_')
                {
                    self.index += 1;
                }
                let raw = &self.text[start..self.index];
                if raw.len() > MAX_V06_NAME_BYTES {
                    return Err(self.syntax(
                        start,
                        self.index,
                        "identifier exceeds V0.6 name bound",
                    ));
                }
                let kind = match raw {
                    "fn" => TokenKindV06::Fn,
                    "const" => TokenKindV06::Const,
                    "entry" => TokenKindV06::Entry,
                    "returns" => TokenKindV06::Returns,
                    "if" => TokenKindV06::If,
                    "else" => TokenKindV06::Else,
                    "true" => TokenKindV06::True,
                    "false" => TokenKindV06::False,
                    _ => TokenKindV06::Ident(raw.to_owned()),
                };
                tokens.push(TokenV06 {
                    kind,
                    start,
                    end: self.index,
                });
                continue;
            }

            let (kind, width) = match byte {
                b'(' => (TokenKindV06::LParen, 1),
                b')' => (TokenKindV06::RParen, 1),
                b'{' => (TokenKindV06::LBrace, 1),
                b'}' => (TokenKindV06::RBrace, 1),
                b',' => (TokenKindV06::Comma, 1),
                b';' => (TokenKindV06::Semicolon, 1),
                b'+' => (TokenKindV06::Plus, 1),
                b'-' => (TokenKindV06::Minus, 1),
                b'*' => (TokenKindV06::Star, 1),
                b'/' => (TokenKindV06::Slash, 1),
                b'=' if self.peek_byte(1) == Some(b'=') => (TokenKindV06::EqEq, 2),
                b'=' => (TokenKindV06::Assign, 1),
                b'!' if self.peek_byte(1) == Some(b'=') => (TokenKindV06::Ne, 2),
                b'!' => (TokenKindV06::Bang, 1),
                b'<' if self.peek_byte(1) == Some(b'=') => (TokenKindV06::Le, 2),
                b'<' => (TokenKindV06::Lt, 1),
                b'>' if self.peek_byte(1) == Some(b'=') => (TokenKindV06::Ge, 2),
                b'>' => (TokenKindV06::Gt, 1),
                b'&' if self.peek_byte(1) == Some(b'&') => (TokenKindV06::AndAnd, 2),
                b'|' if self.peek_byte(1) == Some(b'|') => (TokenKindV06::OrOr, 2),
                _ => {
                    return Err(self.syntax(
                        start,
                        start + 1,
                        format!("unsupported V0.6 source byte 0x{byte:02x}"),
                    ))
                }
            };
            self.index += width;
            tokens.push(TokenV06 {
                kind,
                start,
                end: self.index,
            });
        }
        tokens.push(TokenV06 {
            kind: TokenKindV06::Eof,
            start: self.index,
            end: self.index,
        });
        Ok(tokens)
    }

    fn peek_byte(&self, offset: usize) -> Option<u8> {
        self.text.as_bytes().get(self.index + offset).copied()
    }

    fn syntax(&self, start: usize, end: usize, message: impl Into<String>) -> CoreFunctionsError {
        let raw_start = (self.base + start).min(self.source.len().get() as usize) as u32;
        let raw_end = (self.base + end).min(self.source.len().get() as usize) as u32;
        let span = self
            .source
            .span(ByteOffset::new(raw_start), ByteOffset::new(raw_end))
            .unwrap_or_else(|_| self.source.full_span());
        CoreFunctionsError::Syntax {
            message: message.into(),
            span,
        }
    }
}

struct Parser<'a> {
    source: &'a SourceText,
    base: usize,
    tokens: Vec<TokenV06>,
    index: usize,
    expr_nodes: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a SourceText, body: &'a str, base: usize) -> CoreFunctionsResult<Self> {
        let tokens = Lexer::new(source, body, base).lex()?;
        Ok(Self {
            source,
            base,
            tokens,
            index: 0,
            expr_nodes: 0,
        })
    }

    fn parse_program(&mut self) -> CoreFunctionsResult<CoreProgram> {
        let mut globals = Vec::new();
        let mut functions = Vec::new();
        while !self.is(&TokenKindV06::Entry) {
            if self.is(&TokenKindV06::Const) {
                if globals.len() >= MAX_V06_BINDINGS {
                    return Err(self.error_here("too many top-level immutable bindings"));
                }
                globals.push(self.parse_binding()?);
            } else if self.is(&TokenKindV06::Fn) {
                if functions.len() >= MAX_V06_FUNCTIONS {
                    return Err(self.error_here("too many pure functions"));
                }
                functions.push(self.parse_function()?);
            } else if self.is(&TokenKindV06::Eof) {
                return Err(
                    self.error_here("expected 'entry' after optional const/function declarations")
                );
            } else {
                return Err(self.error_here("expected contextual 'const', 'fn', or 'entry'"));
            }
        }

        self.expect(TokenKindV06::Entry, "expected 'entry'")?;
        let entry_name = self.expect_ident("expected entry name")?;
        self.expect(
            TokenKindV06::Returns,
            "expected contextual 'returns' after entry name",
        )?;
        let entry = self.parse_expr()?;
        self.expect(
            TokenKindV06::Semicolon,
            "expected ';' after entry expression",
        )?;
        self.expect(
            TokenKindV06::Eof,
            "unexpected tokens after entry declaration",
        )?;

        Ok(CoreProgram {
            globals,
            functions,
            entry_name,
            entry,
        })
    }

    fn parse_binding(&mut self) -> CoreFunctionsResult<CoreBinding> {
        self.expect(TokenKindV06::Const, "expected 'const'")?;
        let name = self.expect_ident("expected immutable binding name")?;
        self.expect(
            TokenKindV06::Assign,
            "expected '=' after immutable binding name",
        )?;
        let expr = self.parse_expr()?;
        self.expect(
            TokenKindV06::Semicolon,
            "expected ';' after immutable binding",
        )?;
        Ok(CoreBinding { name, expr })
    }

    fn parse_function(&mut self) -> CoreFunctionsResult<CoreFunction> {
        self.expect(TokenKindV06::Fn, "expected 'fn'")?;
        let name = self.expect_ident("expected function name")?;
        self.expect(TokenKindV06::LParen, "expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.is(&TokenKindV06::RParen) {
            loop {
                if params.len() >= MAX_V06_PARAMS {
                    return Err(self.error_here("too many function parameters"));
                }
                params.push(self.expect_ident("expected parameter name")?);
                if self.is(&TokenKindV06::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(
            TokenKindV06::RParen,
            "expected ')' after function parameters",
        )?;
        let body = self.parse_block()?;
        Ok(CoreFunction { name, params, body })
    }

    fn parse_block(&mut self) -> CoreFunctionsResult<CoreBlock> {
        self.expect(TokenKindV06::LBrace, "expected '{'")?;
        let mut bindings = Vec::new();
        while self.is(&TokenKindV06::Const) {
            if bindings.len() >= MAX_V06_BINDINGS {
                return Err(self.error_here("too many local immutable bindings"));
            }
            bindings.push(self.parse_binding()?);
        }
        let result = self.parse_expr()?;
        if self.is(&TokenKindV06::Semicolon) {
            self.advance();
        }
        self.expect(TokenKindV06::RBrace, "expected '}' after block result")?;
        Ok(CoreBlock {
            bindings,
            result: Box::new(result),
        })
    }

    fn parse_expr(&mut self) -> CoreFunctionsResult<CoreExpr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> CoreFunctionsResult<CoreExpr> {
        let mut expr = self.parse_and()?;
        while self.is(&TokenKindV06::OrOr) {
            self.advance();
            let rhs = self.parse_and()?;
            expr = self.node(CoreExpr::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            })?;
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> CoreFunctionsResult<CoreExpr> {
        let mut expr = self.parse_compare()?;
        while self.is(&TokenKindV06::AndAnd) {
            self.advance();
            let rhs = self.parse_compare()?;
            expr = self.node(CoreExpr::Binary {
                op: BinaryOp::And,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            })?;
        }
        Ok(expr)
    }

    fn parse_compare(&mut self) -> CoreFunctionsResult<CoreExpr> {
        let mut expr = self.parse_add()?;
        loop {
            let op = if self.is(&TokenKindV06::EqEq) {
                Some(BinaryOp::Eq)
            } else if self.is(&TokenKindV06::Ne) {
                Some(BinaryOp::Ne)
            } else if self.is(&TokenKindV06::Lt) {
                Some(BinaryOp::Lt)
            } else if self.is(&TokenKindV06::Le) {
                Some(BinaryOp::Le)
            } else if self.is(&TokenKindV06::Gt) {
                Some(BinaryOp::Gt)
            } else if self.is(&TokenKindV06::Ge) {
                Some(BinaryOp::Ge)
            } else {
                None
            };
            let Some(op) = op else { break };
            self.advance();
            let rhs = self.parse_add()?;
            expr = self.node(CoreExpr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            })?;
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> CoreFunctionsResult<CoreExpr> {
        let mut expr = self.parse_mul()?;
        loop {
            let op = if self.is(&TokenKindV06::Plus) {
                Some(BinaryOp::Add)
            } else if self.is(&TokenKindV06::Minus) {
                Some(BinaryOp::Sub)
            } else {
                None
            };
            let Some(op) = op else { break };
            self.advance();
            let rhs = self.parse_mul()?;
            expr = self.node(CoreExpr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            })?;
        }
        Ok(expr)
    }

    fn parse_mul(&mut self) -> CoreFunctionsResult<CoreExpr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.is(&TokenKindV06::Star) {
                Some(BinaryOp::Mul)
            } else if self.is(&TokenKindV06::Slash) {
                Some(BinaryOp::Div)
            } else {
                None
            };
            let Some(op) = op else { break };
            self.advance();
            let rhs = self.parse_unary()?;
            expr = self.node(CoreExpr::Binary {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            })?;
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> CoreFunctionsResult<CoreExpr> {
        if self.is(&TokenKindV06::Bang) {
            self.advance();
            let expr = self.parse_unary()?;
            return self.node(CoreExpr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> CoreFunctionsResult<CoreExpr> {
        if self.is(&TokenKindV06::If) {
            self.advance();
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            self.expect(TokenKindV06::Else, "expected 'else' after if block")?;
            let else_block = self.parse_block()?;
            return self.node(CoreExpr::If {
                condition: Box::new(condition),
                then_block,
                else_block,
            });
        }

        let token = self.current().clone();
        match token.kind {
            TokenKindV06::Int(value) => {
                self.advance();
                self.node(CoreExpr::Int(value))
            }
            TokenKindV06::True => {
                self.advance();
                self.node(CoreExpr::Bool(true))
            }
            TokenKindV06::False => {
                self.advance();
                self.node(CoreExpr::Bool(false))
            }
            TokenKindV06::Ident(name) => {
                self.advance();
                if self.is(&TokenKindV06::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.is(&TokenKindV06::RParen) {
                        loop {
                            if args.len() >= MAX_V06_PARAMS {
                                return Err(self.error_here("too many function-call arguments"));
                            }
                            args.push(self.parse_expr()?);
                            if self.is(&TokenKindV06::Comma) {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(
                        TokenKindV06::RParen,
                        "expected ')' after function arguments",
                    )?;
                    self.node(CoreExpr::Call { name, args })
                } else {
                    self.node(CoreExpr::Name(name))
                }
            }
            TokenKindV06::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKindV06::RParen, "expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(self.error_here(
                "expected integer, boolean, name, call, if-expression, or parenthesized expression",
            )),
        }
    }

    fn node(&mut self, expr: CoreExpr) -> CoreFunctionsResult<CoreExpr> {
        self.expr_nodes += 1;
        if self.expr_nodes > MAX_V06_EXPR_NODES {
            return Err(self.error_here("V0.6 expression node bound exceeded"));
        }
        Ok(expr)
    }

    fn expect_ident(&mut self, message: &str) -> CoreFunctionsResult<String> {
        match self.current().kind.clone() {
            TokenKindV06::Ident(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_here(message)),
        }
    }

    fn expect(&mut self, expected: TokenKindV06, message: &str) -> CoreFunctionsResult<()> {
        if self.is(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn is(&self, expected: &TokenKindV06) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(expected)
    }

    fn current(&self) -> &TokenV06 {
        &self.tokens[self.index.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
    }

    fn error_here(&self, message: impl Into<String>) -> CoreFunctionsError {
        let token = self.current();
        let raw_start = (self.base + token.start).min(self.source.len().get() as usize) as u32;
        let raw_end = (self.base + token.end).min(self.source.len().get() as usize) as u32;
        let span = self
            .source
            .span(ByteOffset::new(raw_start), ByteOffset::new(raw_end))
            .unwrap_or_else(|_| self.source.full_span());
        CoreFunctionsError::Syntax {
            message: message.into(),
            span,
        }
    }
}

fn encode_program(program: &CoreProgram, out: &mut Vec<u8>) {
    let mut globals = program.globals.iter().collect::<Vec<_>>();
    globals.sort_by(|left, right| left.name.cmp(&right.name));
    out.extend_from_slice(&(globals.len() as u32).to_be_bytes());
    for binding in globals {
        encode_string(&binding.name, out);
        encode_expr(&binding.expr, out);
    }

    let mut functions = program.functions.iter().collect::<Vec<_>>();
    functions.sort_by(|left, right| left.name.cmp(&right.name));
    out.extend_from_slice(&(functions.len() as u32).to_be_bytes());
    for function in functions {
        encode_string(&function.name, out);
        out.extend_from_slice(&(function.params.len() as u32).to_be_bytes());
        for param in &function.params {
            encode_string(param, out);
        }
        encode_block(&function.body, out);
    }

    encode_string(&program.entry_name, out);
    encode_expr(&program.entry, out);
}

fn encode_block(block: &CoreBlock, out: &mut Vec<u8>) {
    out.extend_from_slice(&(block.bindings.len() as u32).to_be_bytes());
    for binding in &block.bindings {
        encode_string(&binding.name, out);
        encode_expr(&binding.expr, out);
    }
    encode_expr(&block.result, out);
}

fn encode_expr(expr: &CoreExpr, out: &mut Vec<u8>) {
    match expr {
        CoreExpr::Int(value) => {
            out.push(1);
            out.extend_from_slice(&value.to_be_bytes());
        }
        CoreExpr::Bool(value) => {
            out.push(2);
            out.push(u8::from(*value));
        }
        CoreExpr::Name(name) => {
            out.push(3);
            encode_string(name, out);
        }
        CoreExpr::Unary {
            op: UnaryOp::Not,
            expr,
        } => {
            out.push(4);
            out.push(1);
            encode_expr(expr, out);
        }
        CoreExpr::Binary { op, lhs, rhs } => {
            out.push(5);
            out.push(op.tag());
            encode_expr(lhs, out);
            encode_expr(rhs, out);
        }
        CoreExpr::Call { name, args } => {
            out.push(6);
            encode_string(name, out);
            out.extend_from_slice(&(args.len() as u32).to_be_bytes());
            for arg in args {
                encode_expr(arg, out);
            }
        }
        CoreExpr::If {
            condition,
            then_block,
            else_block,
        } => {
            out.push(7);
            encode_expr(condition, out);
            encode_block(then_block, out);
            encode_block(else_block, out);
        }
    }
}

fn encode_value_map(values: &BTreeMap<String, CoreValue>, out: &mut Vec<u8>) {
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
