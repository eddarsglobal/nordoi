use nordoi_kernel::{
    analyze_module_unit, compile_execution_plan_boundary, compile_minimal_body_boundary,
    compile_nair_lowering_boundary, compile_project_v14, compile_pure_binding_boundary,
    compile_pure_binding_execution_plan_boundary, compile_pure_binding_nair_boundary,
    compile_pure_condition_boundary, compile_pure_condition_execution_plan_boundary,
    compile_pure_expression_boundary, compile_pure_expression_execution_plan_boundary,
    compile_pure_expression_nair_boundary, compile_pure_result_boundary,
    compile_pure_result_execution_plan_boundary, compile_pure_result_nair_boundary,
    compile_resolved_semantic_boundary, discover_module_imports_v13,
    execute_acyclic_runtime_call_source_v11, execute_bounded_runtime_call_source_v10,
    execute_core_source_v06, execute_dynamic_branch_body_source_v09,
    execute_dynamic_control_source_v08, execute_dynamic_source_v07, execute_module_graph_v13,
    execute_nested_function_control_source_v12, execute_pure_binding_source_v04,
    execute_pure_condition_source_v05, execute_pure_expression_source_v03,
    execute_pure_result_source_v02, execute_source_v01, execute_static_if_source_v05,
    inspect_project_package_v14, lex, parse, parse_project_manifest_v14, validate_module_name_v13,
    AcyclicRuntimeCallError, AstElement, BoundedRuntimeCallError, BranchExpr, CallExpr,
    CompilerError, ConditionalCoreError, CoreFunctionsError, CoreValue, Delimiter,
    DynamicBranchBodyError, DynamicControlError, DynamicInputError, InputBatch, InputDeviceId,
    InputEvent, InputPayload, InputSequence, InputSource, InputTarget, Instruction, LexError,
    ModuleError, NestedFunctionControlError, NsirBodyState, NsirMinimalBody, NsirPureBindingForm,
    NsirPureConditionForm, NsirPureExpressionForm, NsirPureResultForm, ParseError,
    PureBindingExecutionError, PureBindingPlanForm, PureConditionCompilerError,
    PureConditionPlanError, PureConditionPlanForm, PureExpressionExecutionError,
    PureExpressionPlanForm, PureResultExecutionError, PureResultPlanForm, SemanticPlanForm,
    SemanticPureBindingExpressionOp, SemanticPureCondition, SemanticPureExpressionOp,
    SourceExecutionError, SourceId, SourceSpan, SourceText, StaticIfCondition, StaticIfOperand,
    Token, TokenKind, Value, MAX_V13_MODULES, MAX_V14_PACKAGE_BYTES, V14_LOCK_FILE,
    V14_MANIFEST_FILE, V14_PACKAGE_MAJOR, V14_PACKAGE_MINOR,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const TOOL_VERSION: &str = "T0.1";
const COMPILER_VERSION: &str = "C0.2";
const MAX_TOOL_INPUT_BYTES: u64 = 16 * 1024 * 1024;
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 2;
const EXIT_IO: u8 = 3;
const EXIT_FRONTEND: u8 = 4;
const EXIT_RUNTIME: u8 = 5;

const HELP: &str = "NORDOI T0.1 tooling\n\
\n\
Usage:\n\
  nordoi lex <path|->\n\
  nordoi parse <path|->\n\
  nordoi module <path|->\n\
  nordoi semantic <path|->\n\
  nordoi body <path|->\n\
  nordoi result <path|->\n\
  nordoi expr <path|->\n\
  nordoi condition <path|->\n\
  nordoi condition-plan <path|->\n\
  nordoi condition-run <path|->\n\
  nordoi if-run <path|->\n\
  nordoi core-run <path|->\n\
  nordoi dynamic-run <path|-> <key-code>\n\
  nordoi branch-run <path|-> <key-code>\n\
  nordoi branch-body-run <path|-> <key-code>\n\
  nordoi call-run <path|-> <key-code>\n\
  nordoi call-graph-run <path|-> <key-code>\n\
  nordoi function-control-run <path|-> <key-code>\n\
  nordoi module-graph-run <source-root> <entry-module> <key-code>\n\
  nordoi build <project-root> [--locked]\n\
  nordoi package-info <package.npkg>\n\
  nordoi bindings <path|->\n\
  nordoi bindings-plan <path|->\n\
  nordoi bindings-lower <path|->\n\
  nordoi bindings-run <path|->\n\
  nordoi expr-plan <path|->\n\
  nordoi expr-lower <path|->\n\
  nordoi expr-run <path|->\n\
  nordoi result-plan <path|->\n\
  nordoi result-lower <path|->\n\
  nordoi result-run <path|->\n\
  nordoi plan <path|->\n\
  nordoi lower <path|->\n\
  nordoi run <path|->\n\
  nordoi --help\n\
  nordoi --version\n\
\n\
Commands:\n\
  lex      Print the lossless L0.1 token stream.\n\
  parse    Print the lossless L0.2 structural AST.\n\
  module   Print the L0.3 contextual module identity.\n\
  semantic Print the validated C0.2 HIR-NSIR registry boundary.\n\
  body     Print the fully understood L0.5 minimal body boundary.\n\
  result      Print the L0.6 pure-result boundary.\n\
  expr        Print the L0.7 pure-expression boundary.\n\
  condition      Print the L0.9 pure boolean/comparison semantic boundary.\n\
  condition-plan Print the C0.11 pure-condition execution plan.\n\
  condition-run  V0.5 execute pure booleans/comparisons through NAIR 0.6/0.8.\n\
  if-run         V0.5 compile-time-select and execute pure if/else with zero runtime branch cost.\n\
  core-run       V0.6 execute pure functions, parameters, immutable locals and core operators after proof-driven folding.\n\
  dynamic-run    V0.7 execute canonical keyboard input through NAIR runtime integer computation.\n\
  branch-run     V0.8 execute one governed dynamic if/else decision through NAIR 0.10.\n\
  branch-body-run V0.9 lazily evaluate only the selected dynamic branch body through NAIR 0.11.\n\
  call-run       V1.0 execute bounded direct pure runtime calls through NAIR 0.12.\n\
  call-graph-run V1.1 execute acyclic bounded pure runtime call graphs through NAIR 0.13.\n\
  function-control-run V1.2 execute selective structured if/else inside bounded pure runtime function bodies through NAIR 0.14.\n\
  module-graph-run V1.3 resolve real .noi modules/imports statically, erase them before NAIR, and execute the entry module.\n\
  build            V1.4 build a deterministic NORDOI project package and canonical lock file.\n\
  package-info     V1.4 validate and inspect a deterministic .npkg package without source access.\n\
  bindings      Print the L0.8 pure named-binding semantic boundary.\n\
  bindings-plan Print the C0.9 pure-binding execution plan.\n\
  bindings-lower Lower C0.9 pure bindings to existing NAIR with zero runtime binding storage.\n\
  bindings-run   Execute C0.10 pure-binding NAIR with zero runtime binding state.\n\
  expr-plan   Print the C0.7 pure-expression execution plan.\n\
  expr-lower  Lower the C0.7 postfix plan faithfully to NAIR 0.6/0.7.\n\
  expr-run    Execute the C0.8 pure-expression NAIR and validate every SSA result.\n\
  result-plan  Print the C0.5 pure-result execution plan.\n\
  result-lower Lower the C0.5 pure-result plan to existing NAIR 0.6 primitives.\n\
  result-run   Execute the C0.6 pure-result NAIR and validate the observed result.\n\
  plan         Print the C0.3 zero-work executable semantic plan.\n\
  lower    Lower the C0.3 validated zero-work plan to NAIR 0.6.\n\
  run      Execute the certified C0.4 HALT-only NAIR through the closed runtime.\n\
\n\
Use '-' as the path to read UTF-8 source from standard input.\n\
C0.2 resolves type/effect symbols. L0.5 understands the minimal body. C0.3 plans zero work.\n\
C0.3 plan does not lower or execute NAIR. C0.4 lower performs the explicit NAIR 0.6 lowering.\n\
C0.4 lowering does not execute the runtime.\n\
V0.1 run is the explicit source-to-closed-runtime execution boundary.\n\
L0.6 result is additive: it does not create a C0.3 plan, lower NAIR, or execute runtime work.\n\
C0.5 result-plan is additive: it plans L0.6 values but does not lower NAIR or execute runtime work.\n\
C0.6 result-lower is additive: it lowers C0.5 values to existing NAIR 0.6 Const/Halt and does not execute runtime work.\n\
V0.2 result-run executes C0.6 through the closed runtime and validates transient result registers without granting I/O or authority.\n\
L0.7 expr is additive: it evaluates only pure checked integer addition and does not plan, lower, or execute runtime work.\n\
C0.7 expr-plan is additive: it preserves L0.7 postfix calculation order without lowering NAIR or executing runtime work.\n\
C0.8 expr-lower is additive: it maps each postfix INT/ADD to Const/ADD_INT_CHECKED SSA instructions and never executes runtime work.\n\
V0.3 expr-run executes C0.8 through the closed runtime and validates every transient SSA register without I/O, effects, or authority.\n\
L0.8 bindings adds immutable named compile-time bindings and reference resolution without planning, NAIR, runtime work, storage, effects, or authority.\n\
C0.9 bindings-plan preserves canonical binding identities and postfix references with zero runtime storage, without NAIR lowering or runtime execution.\n\
C0.10 bindings-lower erases immutable binding references at compile time and reuses existing NAIR 0.6/0.7 instructions with no runtime binding lookup or storage.\n\
V0.4 bindings-run executes C0.10 through the closed runtime while proving zero binding-specific runtime storage or lookup.\n\
L0.9 condition adds pure boolean literals and integer comparisons without planning, NAIR, runtime work, storage, effects, or authority.\n\
C0.11 condition-plan preserves exact L0.9 condition identity and truth with zero work/storage, without branches, NAIR lowering, or runtime execution.\n\
V0.5 condition-run lowers pure boolean/comparison plans to NAIR and executes them through the closed runtime.\n\
V0.5 if-run validates both pure integer branches, proves the static condition, erases the dead branch before NAIR, and executes only the selected branch.\n\
V0.6 core-run adds checked + - * /, == != < <= > >=, && || !, immutable locals, pure functions/parameters/calls and nested static if expressions; closed pure programs inline/fold to minimal CONST+HALT NAIR without runtime call frames.\n\
V0.7 dynamic-run adds one explicit canonical integer input (`input name;`) sourced from keyboard key-code event 0; dynamic checked addition/comparison survives into NAIR 0.9 while static expressions still collapse to CONST+HALT.\n\
V0.8 branch-run adds structured `if condition { value } else { value }`; dynamic conditions survive as NAIR 0.10 BRANCH_VALUE while static conditions are erased before runtime.\n\
V0.9 branch-body-run adds selective pure branch-body evaluation through NAIR 0.11 BRANCH_EVAL; only the chosen branch expression executes, while the unselected branch performs zero runtime expression work.\n\
V1.0 call-run adds bounded direct pure runtime calls through NAIR 0.12 CALL_EVAL; calls are non-recursive, direct-only, and runtime call depth is certified at one.\n\
V1.1 call-graph-run adds statically acyclic bounded call graphs through NAIR 0.13 nested call expressions; recursion, cycles, indirect calls, and unbounded call depth remain forbidden.\n\
V1.2 function-control-run adds selective structured if/else inside pure runtime function bodies through NAIR 0.14; only the selected arm executes and the acyclic bounded call graph remains enforced.\n\
V1.3 module-graph-run adds statically resolved real .noi module/import graphs; imports are erased before NAIR, import cycles and ambiguities fail closed, and runtime filesystem authority remains NONE.\n\
V1.4 build adds strict NORDOI.toml projects, canonical NORDOI.lock state, deterministic .npkg artifacts, and --locked drift rejection with no dependency network access.\n";

fn main() -> ExitCode {
    ExitCode::from(run())
}

fn run() -> u8 {
    let arguments: Vec<_> = env::args_os().skip(1).collect();

    if arguments.is_empty()
        || (arguments.len() == 1 && arguments[0].as_os_str() == OsStr::new("--help"))
    {
        return match write_stdout(HELP.as_bytes()) {
            Ok(()) => EXIT_OK,
            Err(error) => {
                report_io_error("<stdout>", &error);
                EXIT_IO
            }
        };
    }

    if arguments.len() == 1 && arguments[0].as_os_str() == OsStr::new("--version") {
        let version = format!(
            "nordoi {TOOL_VERSION} (compiler {COMPILER_VERSION}, kernel K1.18, NAIR 0.6)\n"
        );
        return match write_stdout(version.as_bytes()) {
            Ok(()) => EXIT_OK,
            Err(error) => {
                report_io_error("<stdout>", &error);
                EXIT_IO
            }
        };
    }

    let command = arguments
        .first()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();

    if command.as_ref() == "module-graph-run" {
        if arguments.len() != 4 {
            report_usage_error("module-graph-run expects <source-root> <entry-module> <key-code>");
            return EXIT_USAGE;
        }
        let key_code = match arguments[3].to_string_lossy().parse::<u32>() {
            Ok(value) => value,
            Err(_) => {
                report_usage_error("module-graph-run key-code must be an unsigned 32-bit integer");
                return EXIT_USAGE;
            }
        };
        return run_module_graph_from_root_v13_cli(
            Path::new(&arguments[1]),
            arguments[2].to_string_lossy().as_ref(),
            key_code,
        );
    }

    if command.as_ref() == "build" {
        let locked = match arguments.len() {
            2 => false,
            3 if arguments[2].as_os_str() == OsStr::new("--locked") => true,
            _ => {
                report_usage_error("build expects <project-root> [--locked]");
                return EXIT_USAGE;
            }
        };
        return run_project_build_v14_cli(Path::new(&arguments[1]), locked);
    }

    if command.as_ref() == "package-info" {
        if arguments.len() != 2 {
            report_usage_error("package-info expects exactly one .npkg path");
            return EXIT_USAGE;
        }
        return run_package_info_v14_cli(Path::new(&arguments[1]));
    }
    let dynamic_key_code = if matches!(
        command.as_ref(),
        "dynamic-run"
            | "branch-run"
            | "branch-body-run"
            | "call-run"
            | "call-graph-run"
            | "function-control-run"
    ) {
        if arguments.len() != 3 {
            report_usage_error(&format!(
                "{command} expects a source path and one canonical key-code"
            ));
            return EXIT_USAGE;
        }
        match arguments[2].to_string_lossy().parse::<u32>() {
            Ok(value) => Some(value),
            Err(_) => {
                report_usage_error(&format!(
                    "{command} key-code must be an unsigned 32-bit integer"
                ));
                return EXIT_USAGE;
            }
        }
    } else {
        if arguments.len() != 2 {
            report_usage_error("expected a command and exactly one source path");
            return EXIT_USAGE;
        }
        None
    };

    if !matches!(
        command.as_ref(),
        "lex"
            | "parse"
            | "module"
            | "semantic"
            | "body"
            | "result"
            | "expr"
            | "condition"
            | "condition-plan"
            | "condition-run"
            | "if-run"
            | "core-run"
            | "dynamic-run"
            | "branch-run"
            | "branch-body-run"
            | "call-run"
            | "call-graph-run"
            | "function-control-run"
            | "bindings"
            | "bindings-plan"
            | "bindings-lower"
            | "bindings-run"
            | "expr-plan"
            | "expr-lower"
            | "expr-run"
            | "result-plan"
            | "result-lower"
            | "result-run"
            | "plan"
            | "lower"
            | "run"
    ) {
        report_usage_error(&format!("unknown command '{}'", escape_fragment(&command)));
        return EXIT_USAGE;
    }

    let (name, text) = match load_source(&arguments[1]) {
        Ok(input) => input,
        Err(error) => {
            report_load_error(&arguments[1], &error);
            return EXIT_IO;
        }
    };

    let source = match SourceText::new(SourceId::new(1), name, text) {
        Ok(source) => source,
        Err(error) => {
            let source_name = arguments[1].to_string_lossy();
            report_plain_error("source", source_name.as_ref(), &error);
            return EXIT_FRONTEND;
        }
    };

    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    let result = match command.as_ref() {
        "lex" => run_lex(&source, &mut output),
        "parse" => run_parse(&source, &mut output),
        "module" => run_module(&source, &mut output),
        "semantic" => run_semantic(&source, &mut output),
        "body" => run_body(&source, &mut output),
        "result" => run_result(&source, &mut output),
        "expr" => run_expression(&source, &mut output),
        "condition" => run_condition(&source, &mut output),
        "condition-plan" => run_condition_plan(&source, &mut output),
        "condition-run" => run_condition_source_v05(&source, &mut output),
        "if-run" => run_static_if_source_v05(&source, &mut output),
        "core-run" => run_core_source_v06(&source, &mut output),
        "dynamic-run" => run_dynamic_source_v07_cli(
            &source,
            dynamic_key_code.expect("dynamic-run key code validated"),
            &mut output,
        ),
        "branch-run" => run_dynamic_control_source_v08_cli(
            &source,
            dynamic_key_code.expect("branch-run key code validated"),
            &mut output,
        ),
        "branch-body-run" => run_dynamic_branch_body_source_v09_cli(
            &source,
            dynamic_key_code.expect("branch-body-run key code validated"),
            &mut output,
        ),
        "call-run" => run_bounded_runtime_call_source_v10_cli(
            &source,
            dynamic_key_code.expect("call-run key code validated"),
            &mut output,
        ),
        "call-graph-run" => run_acyclic_runtime_call_source_v11_cli(
            &source,
            dynamic_key_code.expect("call-graph-run key code validated"),
            &mut output,
        ),
        "function-control-run" => run_nested_function_control_source_v12_cli(
            &source,
            dynamic_key_code.expect("function-control-run key code validated"),
            &mut output,
        ),
        "bindings" => run_bindings(&source, &mut output),
        "bindings-plan" => run_bindings_plan(&source, &mut output),
        "bindings-lower" => run_bindings_lower(&source, &mut output),
        "bindings-run" => run_pure_binding_source(&source, &mut output),
        "expr-plan" => run_expression_plan(&source, &mut output),
        "expr-lower" => run_expression_lower(&source, &mut output),
        "expr-run" => run_pure_expression_source(&source, &mut output),
        "result-plan" => run_result_plan(&source, &mut output),
        "result-lower" => run_result_lower(&source, &mut output),
        "result-run" => run_pure_result_source(&source, &mut output),
        "plan" => run_plan(&source, &mut output),
        "lower" => run_lower(&source, &mut output),
        "run" => run_source(&source, &mut output),
        _ => unreachable!("validated command must be exhaustive"),
    };

    match result {
        CommandResult::Success => match output.flush() {
            Ok(()) => EXIT_OK,
            Err(error) => {
                report_io_error("<stdout>", &error);
                EXIT_IO
            }
        },
        CommandResult::OutputFailure(error) => {
            report_io_error("<stdout>", &error);
            EXIT_IO
        }
        CommandResult::LexFailure(error) => {
            report_frontend_error("lex", &source, error.span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::ParseFailure(error) => {
            report_frontend_error("parse", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::ModuleFailure(error) => {
            report_frontend_error("module", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::CompilerFailure(error) => {
            report_frontend_error("semantic", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::BodyCompilerFailure(error) => {
            report_frontend_error("body", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureExpressionCompilerFailure(error) => {
            report_frontend_error("expr", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureConditionCompilerFailure(error) => {
            report_frontend_error("condition", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureConditionPlanFailure(error) => {
            report_frontend_error("condition-plan", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::ConditionalCoreFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::CoreFunctionsFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::DynamicInputFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::DynamicControlFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::DynamicBranchBodyFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::BoundedRuntimeCallFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::AcyclicRuntimeCallFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::NestedFunctionControlFailure(command, error) => {
            if error.is_frontend_failure() {
                report_frontend_error(command, &source, error.primary_span(), &error);
                EXIT_FRONTEND
            } else {
                report_plain_error(command, source.name(), &error);
                EXIT_RUNTIME
            }
        }
        CommandResult::PureBindingCompilerFailure(error) => {
            report_frontend_error("bindings", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureBindingPlanCompilerFailure(error) => {
            report_frontend_error("bindings-plan", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureBindingLowerCompilerFailure(error) => {
            report_frontend_error("bindings-lower", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureBindingExecutionFailure(error) => match &error {
            PureBindingExecutionError::Compiler(compiler) => {
                report_frontend_error("bindings-run", &source, compiler.primary_span(), compiler);
                EXIT_FRONTEND
            }
            PureBindingExecutionError::Runtime(_)
            | PureBindingExecutionError::InvariantViolation { .. } => {
                report_plain_error("bindings-run", source.name(), &error);
                EXIT_RUNTIME
            }
        },
        CommandResult::PureExpressionPlanCompilerFailure(error) => {
            report_frontend_error("expr-plan", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureExpressionLowerCompilerFailure(error) => {
            report_frontend_error("expr-lower", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureExpressionExecutionFailure(error) => match &error {
            PureExpressionExecutionError::Compiler(compiler) => {
                report_frontend_error("expr-run", &source, compiler.primary_span(), compiler);
                EXIT_FRONTEND
            }
            PureExpressionExecutionError::Runtime(_)
            | PureExpressionExecutionError::InvariantViolation { .. } => {
                report_plain_error("expr-run", source.name(), &error);
                EXIT_RUNTIME
            }
        },
        CommandResult::PureResultCompilerFailure(error) => {
            report_frontend_error("result", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureResultPlanCompilerFailure(error) => {
            report_frontend_error("result-plan", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureResultLowerCompilerFailure(error) => {
            report_frontend_error("result-lower", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::PureResultExecutionFailure(error) => match &error {
            PureResultExecutionError::Compiler(compiler) => {
                report_frontend_error("result-run", &source, compiler.primary_span(), compiler);
                EXIT_FRONTEND
            }
            PureResultExecutionError::Runtime(_)
            | PureResultExecutionError::InvariantViolation { .. } => {
                report_plain_error("result-run", source.name(), &error);
                EXIT_RUNTIME
            }
        },
        CommandResult::PlanCompilerFailure(error) => {
            report_frontend_error("plan", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::LowerCompilerFailure(error) => {
            report_frontend_error("lower", &source, error.primary_span(), &error);
            EXIT_FRONTEND
        }
        CommandResult::SourceExecutionFailure(error) => match &error {
            SourceExecutionError::Compiler(compiler) => {
                report_frontend_error("run", &source, compiler.primary_span(), compiler);
                EXIT_FRONTEND
            }
            SourceExecutionError::Runtime(_) | SourceExecutionError::InvariantViolation { .. } => {
                report_plain_error("run", source.name(), &error);
                EXIT_RUNTIME
            }
        },
    }
}

enum CommandResult {
    Success,
    OutputFailure(io::Error),
    LexFailure(LexError),
    ParseFailure(ParseError),
    ModuleFailure(ModuleError),
    CompilerFailure(CompilerError),
    BodyCompilerFailure(CompilerError),
    PureExpressionCompilerFailure(CompilerError),
    PureConditionCompilerFailure(PureConditionCompilerError),
    PureConditionPlanFailure(PureConditionPlanError),
    ConditionalCoreFailure(&'static str, ConditionalCoreError),
    CoreFunctionsFailure(&'static str, CoreFunctionsError),
    DynamicInputFailure(&'static str, DynamicInputError),
    DynamicControlFailure(&'static str, DynamicControlError),
    DynamicBranchBodyFailure(&'static str, DynamicBranchBodyError),
    BoundedRuntimeCallFailure(&'static str, BoundedRuntimeCallError),
    AcyclicRuntimeCallFailure(&'static str, AcyclicRuntimeCallError),
    NestedFunctionControlFailure(&'static str, NestedFunctionControlError),
    PureBindingCompilerFailure(CompilerError),
    PureBindingPlanCompilerFailure(CompilerError),
    PureBindingLowerCompilerFailure(CompilerError),
    PureBindingExecutionFailure(PureBindingExecutionError),
    PureExpressionPlanCompilerFailure(CompilerError),
    PureExpressionLowerCompilerFailure(CompilerError),
    PureExpressionExecutionFailure(PureExpressionExecutionError),
    PureResultCompilerFailure(CompilerError),
    PureResultPlanCompilerFailure(CompilerError),
    PureResultLowerCompilerFailure(CompilerError),
    PureResultExecutionFailure(PureResultExecutionError),
    PlanCompilerFailure(CompilerError),
    LowerCompilerFailure(CompilerError),
    SourceExecutionFailure(SourceExecutionError),
}

fn run_lex(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let tokens = match lex(source) {
        Ok(tokens) => tokens,
        Err(error) => return CommandResult::LexFailure(error),
    };

    if let Err(error) = writeln!(
        output,
        "source \"{}\" bytes={} tokens={}",
        escape_fragment(source.name()),
        source.len().get(),
        tokens.len()
    ) {
        return CommandResult::OutputFailure(error);
    }

    for token in &tokens {
        if let Err(error) = write_token_line(source, token, 0, output) {
            return CommandResult::OutputFailure(error);
        }
    }

    CommandResult::Success
}

fn run_parse(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let file = match parse(source) {
        Ok(file) => file,
        Err(error) => return CommandResult::ParseFailure(error),
    };

    if let Err(error) = writeln!(
        output,
        "file {}..{} top-level-elements={}",
        file.span().start().get(),
        file.span().end().get(),
        file.elements().len()
    ) {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = write_ast_elements(source, file.elements(), 1, output) {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = write_token_line(source, file.eof(), 1, output) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_module(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match analyze_module_unit(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::ModuleFailure(error),
    };

    let write_result = match unit.module() {
        Some(declaration) => {
            let canonical = match declaration.path().canonical_text(source) {
                Ok(canonical) => canonical,
                Err(error) => return CommandResult::ModuleFailure(error),
            };
            writeln!(
                output,
                "module \"{}\" span={}..{} segments={}",
                escape_fragment(&canonical),
                declaration.span().start().get(),
                declaration.span().end().get(),
                declaration.path().segments().len()
            )
        }
        None => writeln!(output, "module <anonymous>"),
    };

    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "file {}..{} top-level-elements={}",
        unit.file().span().start().get(),
        unit.file().span().end().get(),
        unit.file().elements().len()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_semantic(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_resolved_semantic_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::CompilerFailure(error),
    };

    let module = match unit.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let body = match unit.body_state() {
        NsirBodyState::Unlowered => "UNLOWERED",
    };
    let identity = hex_bytes(&unit.canonical_identity_bytes());
    let origin = unit.origin();

    let semantic = hex_bytes(&unit.canonical_semantic_bytes());
    let registry_witness = hex_bytes(&unit.canonical_c02_bytes());
    if let Err(error) = writeln!(
        output,
        "nsir module={module} body={body} identity={identity} types={} effects={} semantic={semantic} registry={registry_witness}",
        unit.type_declaration_count(),
        unit.effect_declaration_count()
    ) {
        return CommandResult::OutputFailure(error);
    }
    if let Err(error) = writeln!(
        output,
        "symbols types=[{}] effects=[{}]",
        format_type_symbols(unit.registry()),
        format_effect_symbols(unit.registry())
    ) {
        return CommandResult::OutputFailure(error);
    }
    if let Err(error) = writeln!(
        output,
        "origin file={}..{} body={}..{}",
        origin.file_span().start().get(),
        origin.file_span().end().get(),
        origin.body_span().start().get(),
        origin.body_span().end().get()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_body(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_minimal_body_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::BodyCompilerFailure(error),
    };

    let semantic = unit.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c02 = hex_bytes(&semantic.canonical_c02_bytes());
    let l05 = hex_bytes(&unit.canonical_l05_bytes());

    let result = match unit.body() {
        NsirMinimalBody::Empty => writeln!(
            output,
            "body module={module} form=EMPTY pure=true c02={c02} l05={l05}"
        ),
        NsirMinimalBody::Entry(entry) => writeln!(
            output,
            "body module={module} form=ENTRY name=\"{}\" pure={} effects={} c02={c02} l05={l05}",
            escape_fragment(entry.name().as_str()),
            entry.is_pure(),
            entry.required_effects().effects().len()
        ),
    };
    if let Err(error) = result {
        return CommandResult::OutputFailure(error);
    }

    let origin = semantic.origin();
    if let Err(error) = match unit.entry() {
        Some(entry) => writeln!(
            output,
            "origin body={}..{} entry={}..{}",
            origin.body_span().start().get(),
            origin.body_span().end().get(),
            entry.origin_span().start().get(),
            entry.origin_span().end().get()
        ),
        None => writeln!(
            output,
            "origin body={}..{} entry=<none>",
            origin.body_span().start().get(),
            origin.body_span().end().get()
        ),
    } {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_result(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_pure_result_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::PureResultCompilerFailure(error),
    };

    let semantic = unit.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c02 = hex_bytes(&semantic.canonical_c02_bytes());
    let l06 = hex_bytes(&unit.canonical_l06_bytes());

    let write_result = match unit.form() {
        NsirPureResultForm::Empty => writeln!(
            output,
            "result module={module} form=EMPTY pure=true value=NONE effects=0 c02={c02} l06={l06}"
        ),
        NsirPureResultForm::Entry(entry) => match entry.result_i64() {
            Some(value) => writeln!(
                output,
                "result module={module} form=ENTRY name=\"{}\" pure={} value=INT({value}) effects={} c02={c02} l06={l06}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                entry.required_effects().effects().len()
            ),
            None => writeln!(
                output,
                "result module={module} form=ENTRY name=\"{}\" pure={} value=NONE effects={} c02={c02} l06={l06}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                entry.required_effects().effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let origin = semantic.origin();
    let origin_result = match unit.entry() {
        Some(entry) => match entry.result() {
            Some(result) => writeln!(
                output,
                "origin body={}..{} entry={}..{} result={}..{}",
                origin.body_span().start().get(),
                origin.body_span().end().get(),
                entry.origin_span().start().get(),
                entry.origin_span().end().get(),
                result.origin_span().start().get(),
                result.origin_span().end().get()
            ),
            None => writeln!(
                output,
                "origin body={}..{} entry={}..{} result=<none>",
                origin.body_span().start().get(),
                origin.body_span().end().get(),
                entry.origin_span().start().get(),
                entry.origin_span().end().get()
            ),
        },
        None => writeln!(
            output,
            "origin body={}..{} entry=<none> result=<none>",
            origin.body_span().start().get(),
            origin.body_span().end().get()
        ),
    };
    if let Err(error) = origin_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "execution=NOT_PLANNED nair=UNCHANGED runtime=NOT_INVOKED authority=NONE"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_condition(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_pure_condition_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::PureConditionCompilerFailure(error),
    };

    let semantic = unit.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l09 = hex_bytes(&unit.canonical_l09_bytes());

    let write_result = match unit.form() {
        NsirPureConditionForm::Empty => writeln!(
            output,
            "condition module={module} form=EMPTY pure=true kind=NONE value=NONE effects=0 authority=NONE l09={l09}"
        ),
        NsirPureConditionForm::Entry(entry) => match entry.condition() {
            None => writeln!(
                output,
                "condition module={module} form=ENTRY entry=\"{}\" pure={} kind=NONE value=NONE effects={} authority=NONE l09={l09}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                entry.required_effects().effects().len()
            ),
            Some(SemanticPureCondition::Bool(value)) => writeln!(
                output,
                "condition module={module} form=ENTRY entry=\"{}\" pure={} kind=BOOL({value}) value=BOOL({value}) effects={} authority=NONE l09={l09}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                entry.required_effects().effects().len()
            ),
            Some(SemanticPureCondition::IntCompare { lhs, comparator, rhs }) => writeln!(
                output,
                "condition module={module} form=ENTRY entry=\"{}\" pure={} kind=INT_COMPARE({lhs}{}{rhs}) value=BOOL({}) effects={} authority=NONE l09={l09}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                comparator.symbol(),
                entry.result_bool().expect("condition exists"),
                entry.required_effects().effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_condition_plan(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let plan = match compile_pure_condition_execution_plan_boundary(source) {
        Ok(plan) => plan,
        Err(error) => return CommandResult::PureConditionPlanFailure(error),
    };

    let semantic = plan.condition_semantics().semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l09 = hex_bytes(&plan.condition_semantics().canonical_l09_bytes());
    let c011 = hex_bytes(&plan.canonical_c011_bytes());

    let write_result = match plan.form() {
        PureConditionPlanForm::Empty => writeln!(
            output,
            "condition-plan module={module} form=EMPTY pure=true kind=NONE value=NONE work=0 storage=0 effects=0 authority=NONE l09={l09} c011={c011}"
        ),
        PureConditionPlanForm::Entry(entry) => match entry.condition() {
            None => writeln!(
                output,
                "condition-plan module={module} form=ENTRY entry=\"{}\" pure={} kind=NONE value=NONE work={} storage={} effects={} authority=NONE l09={l09} c011={c011}",
                escape_fragment(entry.name().as_str()),
                entry.required_effects().is_pure(),
                plan.work_item_count(),
                plan.runtime_storage_item_count(),
                entry.required_effects().effects().len()
            ),
            Some(planned) => match planned.condition() {
                SemanticPureCondition::Bool(value) => writeln!(
                    output,
                    "condition-plan module={module} form=ENTRY entry=\"{}\" pure={} kind=BOOL({value}) value=BOOL({}) work={} storage={} effects={} authority=NONE l09={l09} c011={c011}",
                    escape_fragment(entry.name().as_str()),
                    entry.required_effects().is_pure(),
                    planned.value(),
                    plan.work_item_count(),
                    plan.runtime_storage_item_count(),
                    entry.required_effects().effects().len()
                ),
                SemanticPureCondition::IntCompare { lhs, comparator, rhs } => writeln!(
                    output,
                    "condition-plan module={module} form=ENTRY entry=\"{}\" pure={} kind=INT_COMPARE({lhs}{}{rhs}) value=BOOL({}) work={} storage={} effects={} authority=NONE l09={l09} c011={c011}",
                    escape_fragment(entry.name().as_str()),
                    entry.required_effects().is_pure(),
                    comparator.symbol(),
                    planned.value(),
                    plan.work_item_count(),
                    plan.runtime_storage_item_count(),
                    entry.required_effects().effects().len()
                ),
            },
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "branching=UNDEFINED lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_condition_source_v05(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_pure_condition_source_v05(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::ConditionalCoreFailure("condition-run", error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let semantic = plan.condition_semantics().semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let receipt = hex_bytes(&report.canonical_v05_receipt_bytes());
    let witness = hex_bytes(&lowering.canonical_v05_condition_nair_bytes());
    let result_text = match report.result_bool() {
        Some(value) => format!("BOOL({value})"),
        None => "NONE".to_owned(),
    };
    let kind = match plan.condition() {
        None => "NONE".to_owned(),
        Some(planned) => match planned.condition() {
            SemanticPureCondition::Bool(value) => format!("BOOL({value})"),
            SemanticPureCondition::IntCompare {
                lhs,
                comparator,
                rhs,
            } => {
                format!("INT_COMPARE({lhs}{}{rhs})", comparator.symbol())
            }
        },
    };
    let entry = plan
        .entry()
        .map(|entry| format!("\"{}\"", escape_fragment(entry.name().as_str())))
        .unwrap_or_else(|| "<none>".to_owned());

    if let Err(error) = writeln!(
        output,
        "condition-run module={module} entry={entry} kind={kind} result={result_text} work={} storage={} effects={} authority=NONE nair-instructions={} nair-minor=0.{} witness={witness} receipt={receipt}",
        plan.work_item_count(),
        plan.runtime_storage_item_count(),
        plan.required_effects().len(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor()
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} registers={} domains={} atoms={} transactions={} scheduled={} quiescent={} result={result_text}",
        format_v05_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_static_if_source_v05(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_static_if_source_v05(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::ConditionalCoreFailure("if-run", error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let selected = plan.selected_semantics();
    let semantic = selected.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let bindings = format_pure_bindings(selected.bindings().bindings());
    let result_text = match report.result_i64() {
        Some(value) => format!("INT({value})"),
        None => "NONE".to_owned(),
    };
    let entry = selected
        .entry()
        .map(|entry| format!("\"{}\"", escape_fragment(entry.name().as_str())))
        .unwrap_or_else(|| "<none>".to_owned());
    let witness = hex_bytes(&lowering.canonical_v05_lowering_bytes());
    let receipt = hex_bytes(&report.canonical_v05_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "if-run module={module} entry={entry} bindings={bindings} condition={} condition-value={} selected={} result={result_text} runtime-branches={} dead-branch-eliminated={} dead-branch-instructions={} nair-instructions={} nair-minor=0.{} authority=NONE witness={witness} receipt={receipt}",
        format_static_if_condition(plan.condition()),
        plan.condition_value(),
        plan.selected_branch().as_str(),
        plan.runtime_branch_count(),
        plan.dead_branch_eliminated(),
        lowering.dead_branch_instruction_count(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v05_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_core_source_v06(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_core_source_v06(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::CoreFunctionsFailure("core-run", error),
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let result_text = match report.result() {
        CoreValue::Int(value) => format!("INT({value})"),
        CoreValue::Bool(value) => format!("BOOL({value})"),
    };
    let witness = hex_bytes(lowering.canonical_v06_witness_bytes());
    let receipt = hex_bytes(report.canonical_v06_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "core-run module={module} entry=\"{}\" functions={} globals={} source-ops={} calls={} inlined-calls={} static-ifs={} result={result_text} constant-folded={} runtime-calls={} runtime-branches={} nair-instructions={} nair-minor=0.{} authority=NONE witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        plan.function_count(),
        plan.global_binding_count(),
        plan.source_operation_count(),
        plan.function_call_count(),
        plan.inlined_call_count(),
        plan.static_if_count(),
        report.constant_folded(),
        plan.runtime_call_count(),
        plan.runtime_branch_count(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    let nair_text = match lowering.result() {
        CoreValue::Int(value) => format!("[CONST r0 INT({value}),HALT]"),
        CoreValue::Bool(value) => format!("[CONST r0 BOOL({value}),HALT]"),
    };
    if let Err(error) = writeln!(
        output,
        "nair instructions={nair_text} runtime replay={} executed={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        runtime.runtime().replay_key,
        execution.executed_instructions,
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_dynamic_source_v07_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_dynamic_source_v07(source, &input) {
        Ok(report) => report,
        Err(error) => return CommandResult::DynamicInputFailure("dynamic-run", error),
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v07_witness_bytes());
    let receipt = hex_bytes(report.canonical_v07_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "dynamic-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} dynamic={} result={result_text} runtime-computed={} runtime-calls=0 runtime-branches=0 nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v07_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_v07_nair_instructions(instructions: &[Instruction]) -> String {
    let parts = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::ReadInputKeyCode { dst, event_index } => {
                format!("READ_INPUT_KEY_CODE r{} event={event_index}", dst.0)
            }
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => format!("CONST r{} INT({value})", dst.0),
            Instruction::Const {
                dst,
                value: Value::Bool(value),
            } => format!("CONST r{} BOOL({value})", dst.0),
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntEq { dst, lhs, rhs } => {
                format!("INT_EQ r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntNe { dst, lhs, rhs } => {
                format!("INT_NE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLt { dst, lhs, rhs } => {
                format!("INT_LT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLe { dst, lhs, rhs } => {
                format!("INT_LE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGt { dst, lhs, rhs } => {
                format!("INT_GT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGe { dst, lhs, rhs } => {
                format!("INT_GE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::Halt => "HALT".to_owned(),
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join(","))
}

fn run_dynamic_control_source_v08_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_dynamic_control_source_v08(source, &input) {
        Ok(report) => report,
        Err(error) => return CommandResult::DynamicControlFailure("branch-run", error),
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v08_witness_bytes());
    let receipt = hex_bytes(report.canonical_v08_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "branch-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} dynamic={} result={result_text} runtime-computed={} runtime-calls=0 runtime-branches={} nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        report.runtime_branches(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-branches={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v08_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        execution.runtime_branches,
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_v08_nair_instructions(instructions: &[Instruction]) -> String {
    let parts = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::ReadInputKeyCode { dst, event_index } => {
                format!("READ_INPUT_KEY_CODE r{} event={event_index}", dst.0)
            }
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => format!("CONST r{} INT({value})", dst.0),
            Instruction::Const {
                dst,
                value: Value::Bool(value),
            } => format!("CONST r{} BOOL({value})", dst.0),
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntEq { dst, lhs, rhs } => {
                format!("INT_EQ r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntNe { dst, lhs, rhs } => {
                format!("INT_NE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLt { dst, lhs, rhs } => {
                format!("INT_LT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLe { dst, lhs, rhs } => {
                format!("INT_LE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGt { dst, lhs, rhs } => {
                format!("INT_GT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGe { dst, lhs, rhs } => {
                format!("INT_GE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::BranchValue {
                dst,
                condition,
                then_value,
                else_value,
            } => format!(
                "BRANCH_VALUE r{} cond=r{} then={} else={}",
                dst.0,
                condition.0,
                format_v08_branch_value(then_value),
                format_v08_branch_value(else_value)
            ),
            Instruction::Halt => "HALT".to_owned(),
            other => format!("UNEXPECTED({other:?})"),
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join(","))
}

fn format_v08_branch_value(value: &Value) -> String {
    match value {
        Value::Int(value) => format!("INT({value})"),
        Value::Bool(value) => format!("BOOL({value})"),
        other => format!("UNSUPPORTED({other:?})"),
    }
}

fn run_dynamic_branch_body_source_v09_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_dynamic_branch_body_source_v09(source, &input) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::DynamicBranchBodyFailure("branch-body-run", error);
        }
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v09_witness_bytes());
    let receipt = hex_bytes(report.canonical_v09_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "branch-body-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} dynamic={} result={result_text} runtime-computed={} runtime-calls=0 runtime-branches={} selected-branch-instructions={} discarded-branch-instructions={} nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        report.runtime_branches(),
        report.selected_branch_instructions(),
        report.discarded_branch_instructions(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-branches={} selected-branch-instructions={} discarded-branch-instructions={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v09_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        execution.runtime_branches,
        report.selected_branch_instructions(),
        report.discarded_branch_instructions(),
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_v09_nair_instructions(instructions: &[Instruction]) -> String {
    let parts = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::ReadInputKeyCode { dst, event_index } => {
                format!("READ_INPUT_KEY_CODE r{} event={event_index}", dst.0)
            }
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => format!("CONST r{} INT({value})", dst.0),
            Instruction::Const {
                dst,
                value: Value::Bool(value),
            } => format!("CONST r{} BOOL({value})", dst.0),
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntEq { dst, lhs, rhs } => {
                format!("INT_EQ r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntNe { dst, lhs, rhs } => {
                format!("INT_NE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLt { dst, lhs, rhs } => {
                format!("INT_LT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLe { dst, lhs, rhs } => {
                format!("INT_LE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGt { dst, lhs, rhs } => {
                format!("INT_GT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGe { dst, lhs, rhs } => {
                format!("INT_GE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::BranchValue {
                dst,
                condition,
                then_value,
                else_value,
            } => format!(
                "BRANCH_VALUE r{} cond=r{} then={} else={}",
                dst.0,
                condition.0,
                format_v08_branch_value(then_value),
                format_v08_branch_value(else_value)
            ),
            Instruction::BranchEval {
                dst,
                condition,
                then_expr,
                else_expr,
            } => format!(
                "BRANCH_EVAL r{} cond=r{} then={} else={}",
                dst.0,
                condition.0,
                format_v09_branch_expr(then_expr),
                format_v09_branch_expr(else_expr)
            ),
            Instruction::Halt => "HALT".to_owned(),
            other => format!("UNEXPECTED({other:?})"),
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join(","))
}

fn format_v09_branch_expr(expr: &BranchExpr) -> String {
    match expr {
        BranchExpr::Value(Value::Int(value)) => format!("INT({value})"),
        BranchExpr::Value(Value::Bool(value)) => format!("BOOL({value})"),
        BranchExpr::Value(other) => format!("UNSUPPORTED({other:?})"),
        BranchExpr::Register(register) => format!("REG(r{})", register.0),
        BranchExpr::IntAddChecked { lhs, rhs } => format!(
            "ADD({}, {})",
            format_v09_branch_expr(lhs),
            format_v09_branch_expr(rhs)
        ),
        BranchExpr::IntEq { lhs, rhs } => {
            format!(
                "EQ({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
        BranchExpr::IntNe { lhs, rhs } => {
            format!(
                "NE({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
        BranchExpr::IntLt { lhs, rhs } => {
            format!(
                "LT({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
        BranchExpr::IntLe { lhs, rhs } => {
            format!(
                "LE({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
        BranchExpr::IntGt { lhs, rhs } => {
            format!(
                "GT({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
        BranchExpr::IntGe { lhs, rhs } => {
            format!(
                "GE({}, {})",
                format_v09_branch_expr(lhs),
                format_v09_branch_expr(rhs)
            )
        }
    }
}

fn format_static_if_condition(condition: &StaticIfCondition) -> String {
    match condition {
        StaticIfCondition::Bool(value) => format!("BOOL({value})"),
        StaticIfCondition::IntCompare {
            lhs,
            comparator,
            rhs,
        } => format!(
            "{}{}{}",
            format_static_if_operand(lhs),
            comparator.symbol(),
            format_static_if_operand(rhs)
        ),
    }
}

fn format_static_if_operand(operand: &StaticIfOperand) -> String {
    match operand {
        StaticIfOperand::Int(value) => value.to_string(),
        StaticIfOperand::Binding(name) => escape_fragment(name),
    }
}

fn format_v05_nair_instructions(instructions: &[Instruction]) -> String {
    let parts: Vec<String> = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => {
                format!("CONST r{} INT({value})", dst.0)
            }
            Instruction::Const {
                dst,
                value: Value::Bool(value),
            } => {
                format!("CONST r{} BOOL({value})", dst.0)
            }
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntEq { dst, lhs, rhs } => {
                format!("INT_EQ r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntNe { dst, lhs, rhs } => {
                format!("INT_NE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLt { dst, lhs, rhs } => {
                format!("INT_LT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLe { dst, lhs, rhs } => {
                format!("INT_LE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGt { dst, lhs, rhs } => {
                format!("INT_GT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGe { dst, lhs, rhs } => {
                format!("INT_GE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::Halt => "HALT".to_owned(),
            other => format!("UNEXPECTED({other:?})"),
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn run_bounded_runtime_call_source_v10_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_bounded_runtime_call_source_v10(source, &input) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::BoundedRuntimeCallFailure("call-run", error);
        }
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v10_witness_bytes());
    let receipt = hex_bytes(report.canonical_v10_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "call-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} functions={} dynamic={} result={result_text} runtime-computed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.function_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        report.runtime_calls(),
        report.runtime_branches(),
        report.call_body_instructions(),
        report.max_call_depth(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v10_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        report.runtime_calls(),
        execution.runtime_branches,
        report.call_body_instructions(),
        report.max_call_depth(),
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_acyclic_runtime_call_source_v11_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_acyclic_runtime_call_source_v11(source, &input) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::AcyclicRuntimeCallFailure("call-graph-run", error);
        }
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v11_witness_bytes());
    let receipt = hex_bytes(report.canonical_v11_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "call-graph-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} functions={} dynamic={} result={result_text} runtime-computed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} certified-max-call-depth={} nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.function_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        report.runtime_calls(),
        report.runtime_branches(),
        report.call_body_instructions(),
        report.max_call_depth(),
        plan.max_call_depth(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v10_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        report.runtime_calls(),
        execution.runtime_branches,
        report.call_body_instructions(),
        report.max_call_depth(),
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_nested_function_control_source_v12_cli(
    source: &SourceText,
    key_code: u32,
    output: &mut impl Write,
) -> CommandResult {
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_nested_function_control_source_v12(source, &input) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::NestedFunctionControlFailure("function-control-run", error);
        }
    };

    let plan = report.plan();
    let lowering = report.lowering();
    let module = match plan.module() {
        Some(name) => format!("\"{}\"", escape_fragment(name)),
        None => "<anonymous>".to_owned(),
    };
    let input_name = plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());
    let result_text = report.result().as_text();
    let witness = hex_bytes(lowering.canonical_v12_witness_bytes());
    let receipt = hex_bytes(report.canonical_v12_receipt_bytes());

    if let Err(error) = writeln!(
        output,
        "function-control-run module={module} entry=\"{}\" input={input_name} input-events={} key-code={} constants={} functions={} dynamic={} result={result_text} runtime-computed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} certified-max-call-depth={} nair-instructions={} nair-minor=0.{} authority=NONE input-boundary=CANONICAL witness={witness} receipt={receipt}",
        escape_fragment(plan.entry_name()),
        input.len(),
        key_code,
        plan.constant_count(),
        plan.function_count(),
        plan.is_dynamic(),
        report.runtime_computed(),
        report.runtime_calls(),
        report.runtime_branches(),
        report.call_body_instructions(),
        report.max_call_depth(),
        plan.max_call_depth(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    let runtime = report.runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        format_v10_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        report.runtime_calls(),
        execution.runtime_branches,
        report.call_body_instructions(),
        report.max_call_depth(),
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_v10_nair_instructions(instructions: &[Instruction]) -> String {
    let parts = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::ReadInputKeyCode { dst, event_index } => {
                format!("READ_INPUT_KEY_CODE r{} event={event_index}", dst.0)
            }
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => format!("CONST r{} INT({value})", dst.0),
            Instruction::Const {
                dst,
                value: Value::Bool(value),
            } => format!("CONST r{} BOOL({value})", dst.0),
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntEq { dst, lhs, rhs } => {
                format!("INT_EQ r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntNe { dst, lhs, rhs } => {
                format!("INT_NE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLt { dst, lhs, rhs } => {
                format!("INT_LT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntLe { dst, lhs, rhs } => {
                format!("INT_LE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGt { dst, lhs, rhs } => {
                format!("INT_GT r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::IntGe { dst, lhs, rhs } => {
                format!("INT_GE r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::CallEval {
                dst,
                function_id,
                args,
                body,
            } => {
                let args = args
                    .iter()
                    .map(|arg| format!("r{}", arg.0))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "CALL_EVAL r{} fn={} args=[{}] body={}",
                    dst.0,
                    function_id,
                    args,
                    format_v10_call_expr(body)
                )
            }
            Instruction::Halt => "HALT".to_owned(),
            other => format!("UNEXPECTED({other:?})"),
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join(","))
}

fn format_v10_call_expr(expr: &CallExpr) -> String {
    match expr {
        CallExpr::Value(Value::Int(value)) => format!("INT({value})"),
        CallExpr::Value(Value::Bool(value)) => format!("BOOL({value})"),
        CallExpr::Value(other) => format!("UNSUPPORTED({other:?})"),
        CallExpr::Parameter(index) => format!("PARAM({index})"),
        CallExpr::IntAddChecked { lhs, rhs } => format!(
            "ADD({}, {})",
            format_v10_call_expr(lhs),
            format_v10_call_expr(rhs)
        ),
        CallExpr::IntEq { lhs, rhs } => {
            format!(
                "EQ({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IntNe { lhs, rhs } => {
            format!(
                "NE({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IntLt { lhs, rhs } => {
            format!(
                "LT({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IntLe { lhs, rhs } => {
            format!(
                "LE({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IntGt { lhs, rhs } => {
            format!(
                "GT({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IntGe { lhs, rhs } => {
            format!(
                "GE({}, {})",
                format_v10_call_expr(lhs),
                format_v10_call_expr(rhs)
            )
        }
        CallExpr::IfElse {
            condition,
            then_expr,
            else_expr,
        } => format!(
            "IF({}, {}, {})",
            format_v10_call_expr(condition),
            format_v10_call_expr(then_expr),
            format_v10_call_expr(else_expr)
        ),
        CallExpr::DirectCall {
            function_id,
            args,
            body,
        } => {
            let args = args
                .iter()
                .map(format_v10_call_expr)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "CALL(fn={function_id}, args=[{args}], body={})",
                format_v10_call_expr(body)
            )
        }
    }
}

fn run_bindings(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_pure_binding_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::PureBindingCompilerFailure(error),
    };

    let semantic = unit.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l08 = hex_bytes(&unit.canonical_l08_bytes());
    let binding_text = format_pure_bindings(unit.bindings().bindings());

    let write_result = match unit.form() {
        NsirPureBindingForm::Empty => writeln!(
            output,
            "bindings module={module} form=EMPTY bindings={binding_text} count={} pure=true ops=NONE value=NONE nodes=0 effects=0 authority=NONE l08={l08}",
            unit.bindings().bindings().len()
        ),
        NsirPureBindingForm::Entry(entry) => match entry.expression() {
            Some(expression) => writeln!(
                output,
                "bindings module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} pure={} ops={} value=INT({}) nodes={} effects={} authority=NONE l08={l08}",
                escape_fragment(entry.name().as_str()),
                unit.bindings().bindings().len(),
                entry.required_effects().is_pure(),
                format_pure_binding_ops(expression.ops()),
                expression.value(),
                expression.node_count(),
                entry.required_effects().effects().len()
            ),
            None => writeln!(
                output,
                "bindings module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} pure={} ops=NONE value=NONE nodes=0 effects={} authority=NONE l08={l08}",
                escape_fragment(entry.name().as_str()),
                unit.bindings().bindings().len(),
                entry.required_effects().is_pure(),
                entry.required_effects().effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_pure_bindings(bindings: &[nordoi_kernel::NsirPureBindingSymbol]) -> String {
    let parts: Vec<String> = bindings
        .iter()
        .map(|binding| {
            format!(
                "#{}:{}=INT({})",
                binding.id().get(),
                escape_fragment(binding.name().as_str()),
                binding.value()
            )
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn format_pure_binding_ops(ops: &[SemanticPureBindingExpressionOp]) -> String {
    let parts: Vec<String> = ops
        .iter()
        .map(|op| match op {
            SemanticPureBindingExpressionOp::Int(value) => format!("INT({value})"),
            SemanticPureBindingExpressionOp::Binding(id) => format!("BINDING({})", id.get()),
            SemanticPureBindingExpressionOp::Add => "ADD".to_owned(),
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn run_bindings_plan(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let plan = match compile_pure_binding_execution_plan_boundary(source) {
        Ok(plan) => plan,
        Err(error) => return CommandResult::PureBindingPlanCompilerFailure(error),
    };

    let binding_semantics = plan.binding_semantics();
    let semantic = binding_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l08 = hex_bytes(&binding_semantics.canonical_l08_bytes());
    let c09 = hex_bytes(&plan.canonical_c09_bytes());
    let binding_text = format_pure_bindings(binding_semantics.bindings().bindings());

    let write_result = match plan.form() {
        PureBindingPlanForm::Empty => writeln!(
            output,
            "bindings-plan module={module} form=EMPTY bindings={binding_text} count={} ops=NONE value=NONE nodes=0 work={} storage={} effects={} authority=NONE l08={l08} c09={c09}",
            plan.binding_count(),
            plan.work_item_count(),
            plan.runtime_storage_item_count(),
            plan.required_effects().len()
        ),
        PureBindingPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => writeln!(
                output,
                "bindings-plan module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops={} value=INT({}) nodes={} work={} storage={} effects={} authority=NONE l08={l08} c09={c09}",
                escape_fragment(entry.name().as_str()),
                plan.binding_count(),
                format_pure_binding_ops(expression.ops()),
                expression.value(),
                expression.node_count(),
                plan.work_item_count(),
                plan.runtime_storage_item_count(),
                plan.required_effects().len()
            ),
            None => writeln!(
                output,
                "bindings-plan module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops=NONE value=NONE nodes=0 work={} storage={} effects={} authority=NONE l08={l08} c09={c09}",
                escape_fragment(entry.name().as_str()),
                plan.binding_count(),
                plan.work_item_count(),
                plan.runtime_storage_item_count(),
                plan.required_effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_bindings_lower(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let artifact = match compile_pure_binding_nair_boundary(source) {
        Ok(artifact) => artifact,
        Err(error) => return CommandResult::PureBindingLowerCompilerFailure(error),
    };

    let plan = artifact.plan();
    let binding_semantics = plan.binding_semantics();
    let semantic = binding_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l08 = hex_bytes(&binding_semantics.canonical_l08_bytes());
    let c09 = hex_bytes(&plan.canonical_c09_bytes());
    let c010 = hex_bytes(&artifact.canonical_c010_bytes());
    let nair = hex_bytes(artifact.canonical_nair_bytes());
    let binding_text = format_pure_bindings(binding_semantics.bindings().bindings());

    let write_result = match plan.form() {
        PureBindingPlanForm::Empty => writeln!(
            output,
            "bindings-lower module={module} form=EMPTY bindings={binding_text} count={} ops=NONE value=NONE nodes=0 work={} storage={} effects={} authority=NONE nair-instructions={} result-register=NONE nair-minor=0.{} l08={l08} c09={c09} c010={c010}",
            plan.binding_count(),
            plan.work_item_count(),
            plan.runtime_storage_item_count(),
            plan.required_effects().len(),
            artifact.nair_instruction_count(),
            artifact.nair_format_minor()
        ),
        PureBindingPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => {
                let register = artifact
                    .result_register()
                    .expect("C0.10 binding plan with a value must publish its result register");
                writeln!(
                    output,
                    "bindings-lower module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops={} value=INT({}) nodes={} work={} storage={} effects={} authority=NONE nair-instructions={} result-register=r{} nair-minor=0.{} l08={l08} c09={c09} c010={c010}",
                    escape_fragment(entry.name().as_str()),
                    plan.binding_count(),
                    format_pure_binding_ops(expression.ops()),
                    expression.value(),
                    expression.node_count(),
                    plan.work_item_count(),
                    plan.runtime_storage_item_count(),
                    plan.required_effects().len(),
                    artifact.nair_instruction_count(),
                    register.0,
                    artifact.nair_format_minor()
                )
            }
            None => writeln!(
                output,
                "bindings-lower module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops=NONE value=NONE nodes=0 work={} storage={} effects={} authority=NONE nair-instructions={} result-register=NONE nair-minor=0.{} l08={l08} c09={c09} c010={c010}",
                escape_fragment(entry.name().as_str()),
                plan.binding_count(),
                plan.work_item_count(),
                plan.runtime_storage_item_count(),
                plan.required_effects().len(),
                artifact.nair_instruction_count(),
                artifact.nair_format_minor()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "nair version=0.{} instructions={} bytes={nair} binding-runtime-storage=0 binding-runtime-lookups=0 runtime=NOT_INVOKED",
        artifact.nair_format_minor(),
        format_expression_nair_instructions(artifact.program().instructions())
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_pure_binding_source(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_pure_binding_source_v04(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::PureBindingExecutionFailure(error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let binding_semantics = plan.binding_semantics();
    let semantic = binding_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let binding_text = format_pure_bindings(binding_semantics.bindings().bindings());
    let c010 = hex_bytes(&lowering.canonical_c010_bytes());
    let receipt = hex_bytes(&report.canonical_v04_receipt_bytes());
    let observed = report.runtime();
    let runtime = observed.runtime();
    let execution = &runtime.execution.execution;

    let write_result = match plan.form() {
        PureBindingPlanForm::Empty => writeln!(
            output,
            "bindings-run module={module} form=EMPTY bindings={binding_text} count={} ops=NONE result=NONE result-register=NONE nodes=0 work={} storage={} effects={} authority=NONE nair-instructions={} nair-minor=0.{} c010={c010} receipt={receipt}",
            plan.binding_count(),
            plan.work_item_count(),
            plan.runtime_storage_item_count(),
            plan.required_effects().len(),
            lowering.nair_instruction_count(),
            lowering.nair_format_minor()
        ),
        PureBindingPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => {
                let register = lowering
                    .result_register()
                    .expect("C0.10 binding plan with a value must publish its result register");
                writeln!(
                    output,
                    "bindings-run module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops={} result=INT({}) result-register=r{} nodes={} work={} storage={} effects={} authority=NONE nair-instructions={} nair-minor=0.{} c010={c010} receipt={receipt}",
                    escape_fragment(entry.name().as_str()),
                    plan.binding_count(),
                    format_pure_binding_ops(expression.ops()),
                    expression.value(),
                    register.0,
                    expression.node_count(),
                    plan.work_item_count(),
                    plan.runtime_storage_item_count(),
                    plan.required_effects().len(),
                    lowering.nair_instruction_count(),
                    lowering.nair_format_minor()
                )
            }
            None => writeln!(
                output,
                "bindings-run module={module} form=ENTRY entry=\"{}\" bindings={binding_text} count={} ops=NONE result=NONE result-register=NONE nodes=0 work={} storage={} effects={} authority=NONE nair-instructions={} nair-minor=0.{} c010={c010} receipt={receipt}",
                escape_fragment(entry.name().as_str()),
                plan.binding_count(),
                plan.work_item_count(),
                plan.runtime_storage_item_count(),
                plan.required_effects().len(),
                lowering.nair_instruction_count(),
                lowering.nair_format_minor()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let result_text = match report.result_i64() {
        Some(value) => format!("INT({value})"),
        None => "NONE".to_owned(),
    };
    if let Err(error) = writeln!(
        output,
        "runtime replay={} executed={} input={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text} binding-runtime-storage={} binding-runtime-lookups={}",
        runtime.replay_key,
        execution.executed_instructions,
        runtime.input_events,
        observed.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.execution.frames.len(),
        runtime.execution.created_input_bridges,
        execution.scheduled_work,
        observed.is_quiescent(),
        report.binding_runtime_storage_item_count(),
        report.binding_runtime_lookup_count()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_expression(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let unit = match compile_pure_expression_boundary(source) {
        Ok(unit) => unit,
        Err(error) => return CommandResult::PureExpressionCompilerFailure(error),
    };

    let semantic = unit.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c02 = hex_bytes(&semantic.canonical_c02_bytes());
    let l07 = hex_bytes(&unit.canonical_l07_bytes());

    let write_result = match unit.form() {
        NsirPureExpressionForm::Empty => writeln!(
            output,
            "expr module={module} form=EMPTY pure=true ops=NONE value=NONE nodes=0 effects=0 c02={c02} l07={l07}"
        ),
        NsirPureExpressionForm::Entry(entry) => match entry.expression() {
            Some(expression) => writeln!(
                output,
                "expr module={module} form=ENTRY name=\"{}\" pure={} ops={} value=INT({}) nodes={} effects={} c02={c02} l07={l07}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                format_pure_expression_ops(expression.ops()),
                expression.value(),
                expression.node_count(),
                entry.required_effects().effects().len()
            ),
            None => writeln!(
                output,
                "expr module={module} form=ENTRY name=\"{}\" pure={} ops=NONE value=NONE nodes=0 effects={} c02={c02} l07={l07}",
                escape_fragment(entry.name().as_str()),
                entry.is_pure(),
                entry.required_effects().effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let origin = semantic.origin();
    let origin_result = match unit.entry() {
        Some(entry) => match entry.expression() {
            Some(expression) => writeln!(
                output,
                "origin body={}..{} entry={}..{} expression={}..{}",
                origin.body_span().start().get(),
                origin.body_span().end().get(),
                entry.origin_span().start().get(),
                entry.origin_span().end().get(),
                expression.origin_span().start().get(),
                expression.origin_span().end().get()
            ),
            None => writeln!(
                output,
                "origin body={}..{} entry={}..{} expression=<none>",
                origin.body_span().start().get(),
                origin.body_span().end().get(),
                entry.origin_span().start().get(),
                entry.origin_span().end().get()
            ),
        },
        None => writeln!(
            output,
            "origin body={}..{} entry=<none> expression=<none>",
            origin.body_span().start().get(),
            origin.body_span().end().get()
        ),
    };
    if let Err(error) = origin_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "planning=NOT_DEFINED nair=UNCHANGED runtime=NOT_INVOKED authority=NONE"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_pure_expression_ops(ops: &[SemanticPureExpressionOp]) -> String {
    let parts: Vec<String> = ops
        .iter()
        .map(|op| match op {
            SemanticPureExpressionOp::Int(value) => format!("INT({value})"),
            SemanticPureExpressionOp::Add => "ADD".to_owned(),
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn run_expression_plan(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let plan = match compile_pure_expression_execution_plan_boundary(source) {
        Ok(plan) => plan,
        Err(error) => return CommandResult::PureExpressionPlanCompilerFailure(error),
    };

    let expression_semantics = plan.expression_semantics();
    let semantic = expression_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l07 = hex_bytes(&expression_semantics.canonical_l07_bytes());
    let c07 = hex_bytes(&plan.canonical_c07_bytes());

    let write_result = match plan.form() {
        PureExpressionPlanForm::Empty => writeln!(
            output,
            "expr-plan module={module} form=EMPTY ops=NONE value=NONE nodes=0 work={} effects={} authority=NONE l07={l07} c07={c07}",
            plan.work_item_count(),
            plan.required_effects().len()
        ),
        PureExpressionPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => writeln!(
                output,
                "expr-plan module={module} form=ENTRY entry=\"{}\" ops={} value=INT({}) nodes={} work={} effects={} authority=NONE l07={l07} c07={c07}",
                escape_fragment(entry.name().as_str()),
                format_pure_expression_ops(expression.ops()),
                expression.value(),
                expression.node_count(),
                plan.work_item_count(),
                plan.required_effects().len()
            ),
            None => writeln!(
                output,
                "expr-plan module={module} form=ENTRY entry=\"{}\" ops=NONE value=NONE nodes=0 work={} effects={} authority=NONE l07={l07} c07={c07}",
                escape_fragment(entry.name().as_str()),
                plan.work_item_count(),
                plan.required_effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_expression_lower(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let artifact = match compile_pure_expression_nair_boundary(source) {
        Ok(artifact) => artifact,
        Err(error) => return CommandResult::PureExpressionLowerCompilerFailure(error),
    };

    let plan = artifact.plan();
    let expression_semantics = plan.expression_semantics();
    let semantic = expression_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c07 = hex_bytes(&plan.canonical_c07_bytes());
    let c08 = hex_bytes(&artifact.canonical_c08_bytes());
    let nair = hex_bytes(artifact.canonical_nair_bytes());

    let write_result = match plan.form() {
        PureExpressionPlanForm::Empty => writeln!(
            output,
            "expr-lower module={module} form=EMPTY ops=NONE value=NONE nodes=0 work={} effects={} authority=NONE nair-instructions={} result-register=NONE nair-minor=0.{} c07={c07} c08={c08}",
            plan.work_item_count(),
            plan.required_effects().len(),
            artifact.nair_instruction_count(),
            artifact.nair_format_minor()
        ),
        PureExpressionPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => {
                let register = artifact
                    .result_register()
                    .expect("C0.8 expression plan with a value must publish its result register");
                writeln!(
                    output,
                    "expr-lower module={module} form=ENTRY entry=\"{}\" ops={} value=INT({}) nodes={} work={} effects={} authority=NONE nair-instructions={} result-register=r{} nair-minor=0.{} c07={c07} c08={c08}",
                    escape_fragment(entry.name().as_str()),
                    format_pure_expression_ops(expression.ops()),
                    expression.value(),
                    expression.node_count(),
                    plan.work_item_count(),
                    plan.required_effects().len(),
                    artifact.nair_instruction_count(),
                    register.0,
                    artifact.nair_format_minor()
                )
            }
            None => writeln!(
                output,
                "expr-lower module={module} form=ENTRY entry=\"{}\" ops=NONE value=NONE nodes=0 work={} effects={} authority=NONE nair-instructions={} result-register=NONE nair-minor=0.{} c07={c07} c08={c08}",
                escape_fragment(entry.name().as_str()),
                plan.work_item_count(),
                plan.required_effects().len(),
                artifact.nair_instruction_count(),
                artifact.nair_format_minor()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "nair version=0.{} instructions={} bytes={nair} runtime=NOT_INVOKED",
        artifact.nair_format_minor(),
        format_expression_nair_instructions(artifact.program().instructions())
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_expression_nair_instructions(instructions: &[Instruction]) -> String {
    let parts: Vec<String> = instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::Const {
                dst,
                value: Value::Int(value),
            } => format!("CONST r{} INT({value})", dst.0),
            Instruction::IntAddChecked { dst, lhs, rhs } => {
                format!("ADD_INT_CHECKED r{} r{} r{}", dst.0, lhs.0, rhs.0)
            }
            Instruction::Halt => "HALT".to_owned(),
            other => format!("UNEXPECTED({other:?})"),
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn run_pure_expression_source(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_pure_expression_source_v03(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::PureExpressionExecutionFailure(error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let expression_semantics = plan.expression_semantics();
    let semantic = expression_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c08 = hex_bytes(&lowering.canonical_c08_bytes());
    let receipt = hex_bytes(&report.canonical_v03_receipt_bytes());
    let observed = report.runtime();
    let runtime = observed.runtime();
    let execution = &runtime.execution.execution;

    let write_result = match plan.form() {
        PureExpressionPlanForm::Empty => writeln!(
            output,
            "expr-run module={module} form=EMPTY ops=NONE result=NONE result-register=NONE work=0 effects=0 authority=NONE nair-instructions={} nair-minor=0.{} c08={c08} receipt={receipt}",
            lowering.nair_instruction_count(),
            lowering.nair_format_minor()
        ),
        PureExpressionPlanForm::Entry(entry) => match entry.expression() {
            Some(expression) => {
                let register = lowering
                    .result_register()
                    .expect("C0.8 expression plan with a value must publish its result register");
                writeln!(
                    output,
                    "expr-run module={module} form=ENTRY entry=\"{}\" ops={} result=INT({}) result-register=r{} nodes={} work=0 effects=0 authority=NONE nair-instructions={} nair-minor=0.{} c08={c08} receipt={receipt}",
                    escape_fragment(entry.name().as_str()),
                    format_pure_expression_ops(expression.ops()),
                    expression.value(),
                    register.0,
                    expression.node_count(),
                    lowering.nair_instruction_count(),
                    lowering.nair_format_minor()
                )
            }
            None => writeln!(
                output,
                "expr-run module={module} form=ENTRY entry=\"{}\" ops=NONE result=NONE result-register=NONE nodes=0 work=0 effects=0 authority=NONE nair-instructions={} nair-minor=0.{} c08={c08} receipt={receipt}",
                escape_fragment(entry.name().as_str()),
                lowering.nair_instruction_count(),
                lowering.nair_format_minor()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let result_text = match report.result_i64() {
        Some(value) => format!("INT({value})"),
        None => "NONE".to_owned(),
    };
    if let Err(error) = writeln!(
        output,
        "runtime replay={} executed={} input={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        runtime.replay_key,
        execution.executed_instructions,
        runtime.input_events,
        observed.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.execution.frames.len(),
        runtime.execution.created_input_bridges,
        execution.scheduled_work,
        observed.is_quiescent()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_result_plan(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let plan = match compile_pure_result_execution_plan_boundary(source) {
        Ok(plan) => plan,
        Err(error) => return CommandResult::PureResultPlanCompilerFailure(error),
    };

    let result_semantics = plan.result_semantics();
    let semantic = result_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l06 = hex_bytes(&result_semantics.canonical_l06_bytes());
    let c05 = hex_bytes(&plan.canonical_c05_bytes());

    let write_result = match plan.form() {
        PureResultPlanForm::Empty => writeln!(
            output,
            "result-plan module={module} form=EMPTY result=NONE work={} effects={} authority=NONE l06={l06} c05={c05}",
            plan.work_item_count(),
            plan.required_effects().len()
        ),
        PureResultPlanForm::Entry(entry) => match entry.result_i64() {
            Some(value) => writeln!(
                output,
                "result-plan module={module} form=ENTRY entry=\"{}\" result=INT({value}) work={} effects={} authority=NONE l06={l06} c05={c05}",
                escape_fragment(entry.name().as_str()),
                plan.work_item_count(),
                plan.required_effects().len()
            ),
            None => writeln!(
                output,
                "result-plan module={module} form=ENTRY entry=\"{}\" result=NONE work={} effects={} authority=NONE l06={l06} c05={c05}",
                escape_fragment(entry.name().as_str()),
                plan.work_item_count(),
                plan.required_effects().len()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "lowering=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_result_lower(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let artifact = match compile_pure_result_nair_boundary(source) {
        Ok(artifact) => artifact,
        Err(error) => return CommandResult::PureResultLowerCompilerFailure(error),
    };

    let plan = artifact.plan();
    let result_semantics = plan.result_semantics();
    let semantic = result_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c05 = hex_bytes(&plan.canonical_c05_bytes());
    let c06 = hex_bytes(&artifact.canonical_c06_bytes());
    let nair = hex_bytes(artifact.canonical_nair_bytes());

    let write_result = match plan.form() {
        PureResultPlanForm::Empty => writeln!(
            output,
            "result-lower module={module} form=EMPTY result=NONE work={} effects={} authority=NONE nair-instructions={} result-register=NONE c05={c05} c06={c06}",
            plan.work_item_count(),
            plan.required_effects().len(),
            artifact.nair_instruction_count()
        ),
        PureResultPlanForm::Entry(entry) => match entry.result_i64() {
            Some(value) => {
                let register = artifact
                    .result_register()
                    .expect("C0.6 result plan with a value must publish its result register");
                writeln!(
                    output,
                    "result-lower module={module} form=ENTRY entry=\"{}\" result=INT({value}) work={} effects={} authority=NONE nair-instructions={} result-register=r{} c05={c05} c06={c06}",
                    escape_fragment(entry.name().as_str()),
                    plan.work_item_count(),
                    plan.required_effects().len(),
                    artifact.nair_instruction_count(),
                    register.0
                )
            }
            None => writeln!(
                output,
                "result-lower module={module} form=ENTRY entry=\"{}\" result=NONE work={} effects={} authority=NONE nair-instructions={} result-register=NONE c05={c05} c06={c06}",
                escape_fragment(entry.name().as_str()),
                plan.work_item_count(),
                plan.required_effects().len(),
                artifact.nair_instruction_count()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let instructions = match artifact.result_i64() {
        Some(value) => format!("[CONST r0 INT({value}),HALT]"),
        None => "[HALT]".to_owned(),
    };
    if let Err(error) = writeln!(
        output,
        "nair version=0.6 instructions={instructions} bytes={nair} runtime=NOT_INVOKED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_pure_result_source(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_pure_result_source_v02(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::PureResultExecutionFailure(error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let result_semantics = plan.result_semantics();
    let semantic = result_semantics.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c06 = hex_bytes(&lowering.canonical_c06_bytes());
    let receipt = hex_bytes(&report.canonical_v02_receipt_bytes());
    let observed = report.runtime();
    let runtime = observed.runtime();
    let execution = &runtime.execution.execution;

    let write_result = match plan.form() {
        PureResultPlanForm::Empty => writeln!(
            output,
            "result-run module={module} form=EMPTY result=NONE result-register=NONE work=0 effects=0 authority=NONE nair-instructions={} c06={c06} receipt={receipt}",
            lowering.nair_instruction_count()
        ),
        PureResultPlanForm::Entry(entry) => match entry.result_i64() {
            Some(value) => {
                let register = lowering
                    .result_register()
                    .expect("C0.6 result plan with a value must publish its result register");
                writeln!(
                    output,
                    "result-run module={module} form=ENTRY entry=\"{}\" result=INT({value}) result-register=r{} work=0 effects=0 authority=NONE nair-instructions={} c06={c06} receipt={receipt}",
                    escape_fragment(entry.name().as_str()),
                    register.0,
                    lowering.nair_instruction_count()
                )
            }
            None => writeln!(
                output,
                "result-run module={module} form=ENTRY entry=\"{}\" result=NONE result-register=NONE work=0 effects=0 authority=NONE nair-instructions={} c06={c06} receipt={receipt}",
                escape_fragment(entry.name().as_str()),
                lowering.nair_instruction_count()
            ),
        },
    };
    if let Err(error) = write_result {
        return CommandResult::OutputFailure(error);
    }

    let result_text = match report.result_i64() {
        Some(value) => format!("INT({value})"),
        None => "NONE".to_owned(),
    };
    if let Err(error) = writeln!(
        output,
        "runtime replay={} executed={} input={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={result_text}",
        runtime.replay_key,
        execution.executed_instructions,
        runtime.input_events,
        observed.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.execution.frames.len(),
        runtime.execution.created_input_bridges,
        execution.scheduled_work,
        observed.is_quiescent()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_plan(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let plan = match compile_execution_plan_boundary(source) {
        Ok(plan) => plan,
        Err(error) => return CommandResult::PlanCompilerFailure(error),
    };

    let body = plan.body_semantics();
    let semantic = body.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let l05 = hex_bytes(&body.canonical_l05_bytes());
    let c03 = hex_bytes(&plan.canonical_c03_bytes());

    let result = match plan.form() {
        SemanticPlanForm::Empty => writeln!(
            output,
            "plan module={module} form=EMPTY work={} effects={} authority=NONE l05={l05} c03={c03}",
            plan.work_item_count(),
            plan.required_effects().len()
        ),
        SemanticPlanForm::Entry(entry) => writeln!(
            output,
            "plan module={module} form=ENTRY entry=\"{}\" work={} effects={} authority=NONE l05={l05} c03={c03}",
            escape_fragment(entry.name().as_str()),
            plan.work_item_count(),
            plan.required_effects().len()
        ),
    };
    if let Err(error) = result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "lowering=UNDEFINED runtime=NOT_INVOKED nair=UNCHANGED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_lower(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let artifact = match compile_nair_lowering_boundary(source) {
        Ok(artifact) => artifact,
        Err(error) => return CommandResult::LowerCompilerFailure(error),
    };

    let plan = artifact.plan();
    let body = plan.body_semantics();
    let semantic = body.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c03 = hex_bytes(&plan.canonical_c03_bytes());
    let c04 = hex_bytes(&artifact.canonical_c04_bytes());
    let nair = hex_bytes(artifact.canonical_nair_bytes());

    let result = match plan.form() {
        SemanticPlanForm::Empty => writeln!(
            output,
            "lower module={module} form=EMPTY work={} effects={} authority=NONE nair-instructions={} c03={c03} c04={c04}",
            artifact.semantic_work_item_count(),
            plan.required_effects().len(),
            artifact.nair_instruction_count()
        ),
        SemanticPlanForm::Entry(entry) => writeln!(
            output,
            "lower module={module} form=ENTRY entry=\"{}\" work={} effects={} authority=NONE nair-instructions={} c03={c03} c04={c04}",
            escape_fragment(entry.name().as_str()),
            artifact.semantic_work_item_count(),
            plan.required_effects().len(),
            artifact.nair_instruction_count()
        ),
    };
    if let Err(error) = result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "nair version=0.6 instructions=[HALT] bytes={nair} runtime=NOT_INVOKED"
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn run_source(source: &SourceText, output: &mut impl Write) -> CommandResult {
    let report = match execute_source_v01(source) {
        Ok(report) => report,
        Err(error) => return CommandResult::SourceExecutionFailure(error),
    };

    let lowering = report.lowering();
    let plan = lowering.plan();
    let body = plan.body_semantics();
    let semantic = body.semantic();
    let module = match semantic.module().canonical_text() {
        Some(name) => format!("\"{}\"", escape_fragment(&name)),
        None => "<anonymous>".to_owned(),
    };
    let c04 = hex_bytes(&lowering.canonical_c04_bytes());
    let receipt = hex_bytes(&report.canonical_v01_receipt_bytes());
    let runtime = report.runtime();
    let execution = &runtime.execution.execution;

    let result = match plan.form() {
        SemanticPlanForm::Empty => writeln!(
            output,
            "run module={module} form=EMPTY work=0 effects=0 authority=NONE nair-instructions={} c04={c04} receipt={receipt}",
            lowering.nair_instruction_count()
        ),
        SemanticPlanForm::Entry(entry) => writeln!(
            output,
            "run module={module} form=ENTRY entry=\"{}\" work=0 effects=0 authority=NONE nair-instructions={} c04={c04} receipt={receipt}",
            escape_fragment(entry.name().as_str()),
            lowering.nair_instruction_count()
        ),
    };
    if let Err(error) = result {
        return CommandResult::OutputFailure(error);
    }

    if let Err(error) = writeln!(
        output,
        "runtime replay={} executed={} input={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result=HALTED",
        runtime.replay_key,
        execution.executed_instructions,
        runtime.input_events,
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.execution.frames.len(),
        runtime.execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent()
    ) {
        return CommandResult::OutputFailure(error);
    }

    CommandResult::Success
}

fn format_type_symbols(registry: &nordoi_kernel::SemanticRegistry) -> String {
    registry
        .types()
        .iter()
        .map(|symbol| format!("{}:{}", symbol.id().get(), symbol.name().as_str()))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_effect_symbols(registry: &nordoi_kernel::SemanticRegistry) -> String {
    registry
        .effects()
        .iter()
        .map(|symbol| format!("{}:{}", symbol.id().get(), symbol.name().as_str()))
        .collect::<Vec<_>>()
        .join(",")
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn write_ast_elements(
    source: &SourceText,
    elements: &[AstElement],
    depth: usize,
    output: &mut impl Write,
) -> io::Result<()> {
    for element in elements {
        match element {
            AstElement::Token(token) => write_token_line(source, token, depth, output)?,
            AstElement::Group(group) => {
                let indent = "  ".repeat(depth);
                writeln!(
                    output,
                    "{indent}group {} {}..{} elements={}",
                    delimiter_label(group.delimiter()),
                    group.span().start().get(),
                    group.span().end().get(),
                    group.elements().len()
                )?;
                write_token_line(source, group.open(), depth + 1, output)?;
                write_ast_elements(source, group.elements(), depth + 1, output)?;
                write_token_line(source, group.close(), depth + 1, output)?;
            }
        }
    }
    Ok(())
}

fn write_token_line(
    source: &SourceText,
    token: &Token,
    depth: usize,
    output: &mut impl Write,
) -> io::Result<()> {
    let span = token.span();
    let start = position_label(source, span.start().get());
    let end = position_label(source, span.end().get());
    let text = source.slice(span).unwrap_or("<invalid-span>");
    let indent = "  ".repeat(depth);
    writeln!(
        output,
        "{indent}token {} {}..{} {}..{} \"{}\"",
        token_kind_label(token.kind()),
        span.start().get(),
        span.end().get(),
        start,
        end,
        escape_fragment(text)
    )
}

fn position_label(source: &SourceText, raw_offset: u32) -> String {
    match source.position(nordoi_kernel::ByteOffset::new(raw_offset)) {
        Ok(position) => format!("{}:{}", position.line(), position.column()),
        Err(_) => "?:?".to_owned(),
    }
}

fn token_kind_label(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier => "IDENTIFIER".to_owned(),
        TokenKind::NumericCandidate => "NUMERIC_CANDIDATE".to_owned(),
        TokenKind::QuotedText => "QUOTED_TEXT".to_owned(),
        TokenKind::Punctuation(character) => {
            format!("PUNCTUATION({})", escape_fragment(&character.to_string()))
        }
        TokenKind::Whitespace => "WHITESPACE".to_owned(),
        TokenKind::LineComment => "LINE_COMMENT".to_owned(),
        TokenKind::BlockComment => "BLOCK_COMMENT".to_owned(),
        TokenKind::Eof => "EOF".to_owned(),
    }
}

fn delimiter_label(delimiter: Delimiter) -> &'static str {
    match delimiter {
        Delimiter::Parenthesis => "PARENTHESIS",
        Delimiter::Bracket => "BRACKET",
        Delimiter::Brace => "BRACE",
    }
}

fn run_module_graph_from_root_v13_cli(root: &Path, entry_module: &str, key_code: u32) -> u8 {
    let sources = match load_module_graph_sources_v13(root, entry_module) {
        Ok(sources) => sources,
        Err(exit) => return exit,
    };
    let input = InputBatch {
        events: vec![InputEvent {
            sequence: InputSequence(1),
            source: InputSource::Keyboard,
            device: InputDeviceId(1),
            target: InputTarget::Global,
            payload: InputPayload::Key {
                code: key_code,
                pressed: true,
                repeat: false,
            },
        }],
    };
    let report = match execute_module_graph_v13(&sources, entry_module, &input) {
        Ok(report) => report,
        Err(error) => {
            report_plain_error("module-graph-run", entry_module, &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else {
                EXIT_RUNTIME
            };
        }
    };

    let plan = report.plan();
    let inner_plan = report.inner().plan();
    let lowering = report.inner().lowering();
    let result_text = report.result().as_text();
    let module_order = format!("[{}]", plan.module_order().join(","));
    let witness = hex_bytes(plan.canonical_v13_witness_bytes());
    let receipt = hex_bytes(report.canonical_v13_receipt_bytes());
    let input_name = inner_plan
        .input_name()
        .map(|name| format!("\"{}\"", escape_fragment(name)))
        .unwrap_or_else(|| "<none>".to_owned());

    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    if let Err(error) = writeln!(
        output,
        "module-graph-run entry-module=\"{}\" modules={} imports={} module-order={} entry=\"{}\" input={} input-events={} key-code={} functions={} dynamic={} result={} runtime-computed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} certified-max-call-depth={} nair-instructions={} nair-minor=0.{} authority=NONE import-resolution=STATIC runtime-fs=NONE witness={} receipt={}",
        escape_fragment(plan.entry_module()),
        plan.module_count(),
        plan.import_count(),
        module_order,
        escape_fragment(inner_plan.entry_name()),
        input_name,
        input.len(),
        key_code,
        inner_plan.function_count(),
        inner_plan.is_dynamic(),
        result_text,
        report.inner().runtime_computed(),
        report.runtime_calls(),
        report.runtime_branches(),
        report.call_body_instructions(),
        report.max_call_depth(),
        inner_plan.max_call_depth(),
        lowering.nair_instruction_count(),
        lowering.nair_format_minor(),
        witness,
        receipt,
    ) {
        report_io_error("<stdout>", &error);
        return EXIT_IO;
    }

    let runtime = report.inner().runtime();
    let execution = &runtime.runtime().execution.execution;
    if let Err(error) = writeln!(
        output,
        "nair instructions={} runtime replay={} executed={} runtime-calls={} runtime-branches={} call-body-instructions={} max-call-depth={} registers={} domains={} atoms={} transactions={} frames={} bridges={} scheduled={} quiescent={} result={}",
        format_v10_nair_instructions(lowering.program().instructions()),
        runtime.runtime().replay_key,
        execution.executed_instructions,
        report.runtime_calls(),
        report.runtime_branches(),
        report.call_body_instructions(),
        report.max_call_depth(),
        runtime.final_registers().len(),
        execution.created_domains,
        execution.created_atoms,
        execution.committed_transactions + execution.rolled_back_transactions,
        runtime.runtime().execution.frames.len(),
        runtime.runtime().execution.created_input_bridges,
        execution.scheduled_work,
        runtime.is_quiescent(),
        result_text,
    ) {
        report_io_error("<stdout>", &error);
        return EXIT_IO;
    }

    match output.flush() {
        Ok(()) => EXIT_OK,
        Err(error) => {
            report_io_error("<stdout>", &error);
            EXIT_IO
        }
    }
}

fn load_module_graph_sources_v13(root: &Path, entry_module: &str) -> Result<Vec<SourceText>, u8> {
    if let Err(error) = validate_module_name_v13(entry_module) {
        report_plain_error("module-graph-run", entry_module, &error);
        return Err(EXIT_FRONTEND);
    }
    let mut pending = BTreeSet::new();
    let mut loaded = BTreeMap::new();
    pending.insert(entry_module.to_owned());
    let mut next_source_id = 1u32;

    while let Some(module_name) = pending.pop_first() {
        if loaded.contains_key(&module_name) {
            continue;
        }
        if loaded.len() >= MAX_V13_MODULES {
            report_plain_error(
                "module-graph-run",
                entry_module,
                format_args!("module graph exceeds certified bound {MAX_V13_MODULES}"),
            );
            return Err(EXIT_FRONTEND);
        }

        let path = module_path_v13(root, &module_name);
        let (name, text) = match load_source(path.as_os_str()) {
            Ok(value) => value,
            Err(error) => {
                report_load_error(path.as_os_str(), &error);
                return Err(EXIT_IO);
            }
        };
        let source = match SourceText::new(SourceId::new(next_source_id), name, text) {
            Ok(source) => source,
            Err(error) => {
                let path_name = path.to_string_lossy();
                report_plain_error("module-graph-run", path_name.as_ref(), &error);
                return Err(EXIT_FRONTEND);
            }
        };
        next_source_id = next_source_id.saturating_add(1);

        let discovery = match discover_module_imports_v13(&source) {
            Ok(discovery) => discovery,
            Err(error) => {
                report_plain_error("module-graph-run", source.name(), &error);
                return Err(EXIT_FRONTEND);
            }
        };
        if discovery.module() != module_name {
            report_plain_error(
                "module-graph-run",
                source.name(),
                format_args!(
                    "module path requested '{}' but file declares '{}'",
                    module_name,
                    discovery.module()
                ),
            );
            return Err(EXIT_FRONTEND);
        }
        for import in discovery.imports() {
            if !loaded.contains_key(import) {
                pending.insert(import.clone());
            }
        }
        loaded.insert(module_name, source);
    }

    Ok(loaded.into_values().collect())
}

fn module_path_v13(root: &Path, module: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for segment in module.split('.') {
        path.push(segment);
    }
    path.set_extension("noi");
    path
}

fn run_project_build_v14_cli(project_root: &Path, locked: bool) -> u8 {
    let manifest_path = project_root.join(V14_MANIFEST_FILE);
    let manifest_text = match fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        Err(error) => {
            report_io_error(manifest_path.to_string_lossy().as_ref(), &error);
            return EXIT_IO;
        }
    };
    let manifest = match parse_project_manifest_v14(&manifest_text) {
        Ok(manifest) => manifest,
        Err(error) => {
            report_plain_error("build", manifest_path.to_string_lossy().as_ref(), &error);
            return EXIT_FRONTEND;
        }
    };

    let source_root = project_root.join(manifest.source_root());
    let sources = match load_module_graph_sources_v14(&source_root, manifest.entry_module()) {
        Ok(sources) => sources,
        Err(exit) => return exit,
    };
    let build = match compile_project_v14(&manifest, &sources) {
        Ok(build) => build,
        Err(error) => {
            report_plain_error("build", project_root.to_string_lossy().as_ref(), &error);
            return if error.is_frontend_failure() {
                EXIT_FRONTEND
            } else {
                EXIT_RUNTIME
            };
        }
    };

    let lock_path = project_root.join(V14_LOCK_FILE);
    if locked {
        let existing = match fs::read_to_string(&lock_path) {
            Ok(text) => text,
            Err(error) => {
                report_io_error(lock_path.to_string_lossy().as_ref(), &error);
                return EXIT_IO;
            }
        };
        if existing != build.lock_text() {
            report_plain_error(
                "build",
                lock_path.to_string_lossy().as_ref(),
                "--locked rejected project drift: NORDOI.lock does not match the current canonical build graph",
            );
            return EXIT_FRONTEND;
        }
    }

    let build_dir = project_root.join("build");
    if let Err(error) = fs::create_dir_all(&build_dir) {
        report_io_error(build_dir.to_string_lossy().as_ref(), &error);
        return EXIT_IO;
    }
    let package_path = build_dir.join(manifest.package_file_name());
    if let Err(error) = write_atomic_file(&package_path, build.package_bytes()) {
        report_io_error(package_path.to_string_lossy().as_ref(), &error);
        return EXIT_IO;
    }
    if !locked {
        if let Err(error) = write_atomic_file(&lock_path, build.lock_text().as_bytes()) {
            report_io_error(lock_path.to_string_lossy().as_ref(), &error);
            return EXIT_IO;
        }
    }

    let relative_package = Path::new("build").join(manifest.package_file_name());
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    if let Err(error) = writeln!(
        output,
        "build project=\"{}\" version=\"{}\" entry-module=\"{}\" modules={} imports={} nair-minor=0.{} nair-instructions={} package=\"{}\" lock=\"{}\" locked={} reproducible=true dependency-network=NONE runtime-fs=NONE authority=NONE witness={}",
        escape_fragment(manifest.name()),
        escape_fragment(manifest.version()),
        escape_fragment(manifest.entry_module()),
        build.module_plan().module_count(),
        build.module_plan().import_count(),
        build.lowering().nair_format_minor(),
        build.lowering().nair_instruction_count(),
        escape_fragment(relative_package.to_string_lossy().as_ref()),
        V14_LOCK_FILE,
        locked,
        hex_bytes(build.canonical_v14_build_witness_bytes()),
    ) {
        report_io_error("<stdout>", &error);
        return EXIT_IO;
    }
    match output.flush() {
        Ok(()) => EXIT_OK,
        Err(error) => {
            report_io_error("<stdout>", &error);
            EXIT_IO
        }
    }
}

fn run_package_info_v14_cli(path: &Path) -> u8 {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            report_io_error(path.to_string_lossy().as_ref(), &error);
            return EXIT_IO;
        }
    };
    let bytes = match read_bounded(file) {
        Ok(bytes) => bytes,
        Err(error) => {
            report_load_error(path.as_os_str(), &error);
            return EXIT_IO;
        }
    };
    if bytes.len() > MAX_V14_PACKAGE_BYTES {
        report_plain_error(
            "package-info",
            path.to_string_lossy().as_ref(),
            format_args!(
                "package has {} bytes; V1.4 maximum is {} bytes",
                bytes.len(),
                MAX_V14_PACKAGE_BYTES
            ),
        );
        return EXIT_FRONTEND;
    }
    let info = match inspect_project_package_v14(&bytes) {
        Ok(info) => info,
        Err(error) => {
            report_plain_error("package-info", path.to_string_lossy().as_ref(), &error);
            return EXIT_FRONTEND;
        }
    };

    let module_order = format!("[{}]", info.module_order().join(","));
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    if let Err(error) = writeln!(
        output,
        "package-info project=\"{}\" version=\"{}\" entry-module=\"{}\" source-root=\"{}\" modules={} imports={} module-order={} nair-minor=0.{} nair-instructions={} package-format={}.{} dependency-network=NONE runtime-fs=NONE authority=NONE witness={}",
        escape_fragment(info.name()),
        escape_fragment(info.version()),
        escape_fragment(info.entry_module()),
        escape_fragment(info.source_root()),
        info.module_order().len(),
        info.import_edges().len(),
        module_order,
        info.nair_format_minor(),
        info.nair_instruction_count(),
        V14_PACKAGE_MAJOR,
        V14_PACKAGE_MINOR,
        hex_bytes(info.build_witness()),
    ) {
        report_io_error("<stdout>", &error);
        return EXIT_IO;
    }
    match output.flush() {
        Ok(()) => EXIT_OK,
        Err(error) => {
            report_io_error("<stdout>", &error);
            EXIT_IO
        }
    }
}

fn load_module_graph_sources_v14(root: &Path, entry_module: &str) -> Result<Vec<SourceText>, u8> {
    if let Err(error) = validate_module_name_v13(entry_module) {
        report_plain_error("build", entry_module, &error);
        return Err(EXIT_FRONTEND);
    }
    let mut pending = BTreeSet::new();
    let mut loaded = BTreeMap::new();
    pending.insert(entry_module.to_owned());
    let mut next_source_id = 1u32;

    while let Some(module_name) = pending.pop_first() {
        if loaded.contains_key(&module_name) {
            continue;
        }
        if loaded.len() >= MAX_V13_MODULES {
            report_plain_error(
                "build",
                entry_module,
                format_args!("module graph exceeds certified bound {MAX_V13_MODULES}"),
            );
            return Err(EXIT_FRONTEND);
        }

        let path = module_path_v13(root, &module_name);
        let (name, text) = match load_source(path.as_os_str()) {
            Ok(value) => value,
            Err(error) => {
                report_load_error(path.as_os_str(), &error);
                return Err(EXIT_IO);
            }
        };
        let source = match SourceText::new(SourceId::new(next_source_id), name, text) {
            Ok(source) => source,
            Err(error) => {
                report_plain_error("build", path.to_string_lossy().as_ref(), &error);
                return Err(EXIT_FRONTEND);
            }
        };
        next_source_id = next_source_id.saturating_add(1);

        let discovery = match discover_module_imports_v13(&source) {
            Ok(discovery) => discovery,
            Err(error) => {
                report_plain_error("build", source.name(), &error);
                return Err(EXIT_FRONTEND);
            }
        };
        if discovery.module() != module_name {
            report_plain_error(
                "build",
                source.name(),
                format_args!(
                    "module path requested '{}' but file declares '{}'",
                    module_name,
                    discovery.module()
                ),
            );
            return Err(EXIT_FRONTEND);
        }
        for import in discovery.imports() {
            if !loaded.contains_key(import) {
                pending.insert(import.clone());
            }
        }
        loaded.insert(module_name, source);
    }

    Ok(loaded.into_values().collect())
}

fn write_atomic_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("nordoi-output");
    let temp_path = path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    fs::write(&temp_path, bytes)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(error)
        }
    }
}

fn load_source(path: &OsStr) -> Result<(String, String), LoadError> {
    if path == OsStr::new("-") {
        let stdin = io::stdin();
        let bytes = read_bounded(stdin.lock())?;
        let text = String::from_utf8(bytes).map_err(|_| LoadError::InvalidUtf8)?;
        return Ok(("<stdin>".to_owned(), text));
    }

    let path = Path::new(path);
    let file = File::open(path).map_err(LoadError::Io)?;
    let bytes = read_bounded(file)?;
    let text = String::from_utf8(bytes).map_err(|_| LoadError::InvalidUtf8)?;
    Ok((path.to_string_lossy().into_owned(), text))
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, LoadError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_TOOL_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(LoadError::Io)?;
    if bytes.len() as u64 > MAX_TOOL_INPUT_BYTES {
        return Err(LoadError::TooLarge {
            bytes: bytes.len() as u64,
            maximum: MAX_TOOL_INPUT_BYTES,
        });
    }
    Ok(bytes)
}

enum LoadError {
    Io(io::Error),
    TooLarge { bytes: u64, maximum: u64 },
    InvalidUtf8,
}

fn report_load_error(path: &OsStr, error: &LoadError) {
    let name = if path == OsStr::new("-") {
        "<stdin>".to_owned()
    } else {
        path.to_string_lossy().into_owned()
    };

    match error {
        LoadError::Io(error) => report_io_error(&name, error),
        LoadError::TooLarge { bytes, maximum } => report_plain_error(
            "input",
            &name,
            format_args!("input has {bytes} bytes; T0.1 maximum is {maximum} bytes"),
        ),
        LoadError::InvalidUtf8 => {
            report_plain_error("input", &name, "source is not valid UTF-8");
        }
    }
}

fn report_frontend_error(
    stage: &str,
    source: &SourceText,
    span: Option<SourceSpan>,
    error: &dyn std::fmt::Display,
) {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let name = escape_fragment(source.name());
    if let Some(span) = span {
        if let Ok(position) = source.position(span.start()) {
            let _ = writeln!(
                stderr,
                "error[{stage}]: \"{name}\":{}:{}: {error}",
                position.line(),
                position.column()
            );
            return;
        }
    }
    let _ = writeln!(stderr, "error[{stage}]: \"{name}\": {error}");
}

fn report_plain_error(stage: &str, name: &str, error: impl std::fmt::Display) {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let name = escape_fragment(name);
    let _ = writeln!(stderr, "error[{stage}]: \"{name}\": {error}");
}

fn report_io_error(name: &str, error: &io::Error) {
    report_plain_error("io", name, error);
}

fn report_usage_error(message: &str) {
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let _ = writeln!(stderr, "error[usage]: {message}\n\n{HELP}");
}

fn write_stdout(bytes: &[u8]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn escape_fragment(text: &str) -> String {
    text.chars().flat_map(char::escape_default).collect()
}
