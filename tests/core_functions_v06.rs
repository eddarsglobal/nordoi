use nordoi_kernel::{
    compile_core_plan_v06, execute_core_source_v06, lower_core_plan_v06, CoreValue, SourceId,
    SourceText, Value,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(606), "v06.noi", text).unwrap()
}

#[test]
fn pure_function_parameters_and_call_execute_to_42() {
    let report = execute_core_source_v06(&source(
        "fn add(a, b) { a + b } entry main returns add(20, 22);",
    ))
    .unwrap();
    assert_eq!(report.result(), CoreValue::Int(42));
    assert_eq!(report.plan().function_count(), 1);
    assert_eq!(report.plan().function_call_count(), 1);
    assert_eq!(report.plan().inlined_call_count(), 1);
    assert_eq!(report.plan().runtime_call_count(), 0);
    assert_eq!(report.lowering().nair_instruction_count(), 2);
    assert_eq!(report.lowering().nair_format_minor(), 6);
    assert_eq!(report.runtime().final_registers().len(), 1);
    assert_eq!(
        report
            .runtime()
            .register(report.lowering().result_register()),
        Some(&Value::Int(42))
    );
    assert!(report.runtime().is_quiescent());
}

#[test]
fn arithmetic_precedence_sub_mul_div_are_checked() {
    for (expr, expected) in [
        ("2 + 3 * 4", 14),
        ("50 - 8", 42),
        ("6 * 7", 42),
        ("84 / 2", 42),
        ("100 / 5 + 22", 42),
    ] {
        let text = format!("entry main returns {expr};");
        assert_eq!(
            execute_core_source_v06(&source(&text)).unwrap().result(),
            CoreValue::Int(expected),
            "expr={expr}"
        );
    }
}

#[test]
fn boolean_logic_and_comparisons_are_supported() {
    let report = execute_core_source_v06(&source(
        "entry main returns !(false || false) && (20 < 22) && (42 == 42);",
    ))
    .unwrap();
    assert_eq!(report.result(), CoreValue::Bool(true));
    assert_eq!(report.lowering().nair_instruction_count(), 2);
    assert_eq!(report.lowering().nair_format_minor(), 6);
}

#[test]
fn boolean_equality_is_typed() {
    assert_eq!(
        execute_core_source_v06(&source("entry main returns true != false;"))
            .unwrap()
            .result(),
        CoreValue::Bool(true)
    );
}

#[test]
fn nested_calls_inline_without_runtime_frames() {
    let report = execute_core_source_v06(&source(
        "fn inc(x) { x + 1 } fn twice(x) { inc(inc(x)) } entry main returns twice(40);",
    ))
    .unwrap();
    assert_eq!(report.result(), CoreValue::Int(42));
    assert_eq!(report.plan().function_call_count(), 3);
    assert_eq!(report.plan().runtime_call_count(), 0);
    assert!(report.runtime().runtime().execution.frames.is_empty());
}

#[test]
fn function_local_const_and_static_if_work_together() {
    let report = execute_core_source_v06(&source(
        "fn choose(x) { const doubled = x * 2; if doubled >= 40 { doubled + 2 } else { 0 } } entry main returns choose(20);",
    ))
    .unwrap();
    assert_eq!(report.result(), CoreValue::Int(42));
    assert_eq!(report.plan().static_if_count(), 1);
    assert_eq!(report.plan().runtime_branch_count(), 0);
    assert_eq!(report.lowering().nair_instruction_count(), 2);
}

#[test]
fn global_immutable_binding_can_feed_function() {
    let report = execute_core_source_v06(&source(
        "const base = 20 + 1; fn double(x) { x * 2 } entry main returns double(base);",
    ))
    .unwrap();
    assert_eq!(report.result(), CoreValue::Int(42));
    assert_eq!(report.plan().global_binding_count(), 1);
    assert_eq!(
        report.plan().globals().get("base"),
        Some(&CoreValue::Int(21))
    );
}

#[test]
fn unused_function_costs_zero_operational_nair() {
    let a = execute_core_source_v06(&source("entry main returns 42;")).unwrap();
    let b = execute_core_source_v06(&source("fn unused(x) { x * 999 } entry main returns 42;"))
        .unwrap();
    assert_eq!(
        a.lowering().canonical_nair_bytes(),
        b.lowering().canonical_nair_bytes()
    );
    assert_ne!(
        a.lowering().canonical_v06_witness_bytes(),
        b.lowering().canonical_v06_witness_bytes()
    );
    assert_eq!(a.lowering().nair_instruction_count(), 2);
    assert_eq!(b.lowering().nair_instruction_count(), 2);
}

#[test]
fn function_declaration_order_is_not_semantic() {
    let a = compile_core_plan_v06(&source(
        "fn add(a,b){a+b} fn mul(a,b){a*b} entry main returns add(20,22);",
    ))
    .unwrap();
    let b = compile_core_plan_v06(&source(
        "fn mul(a,b){a*b} fn add(a,b){a+b} entry main returns add(20,22);",
    ))
    .unwrap();
    assert_eq!(
        a.canonical_v06_semantic_bytes(),
        b.canonical_v06_semantic_bytes()
    );
}

#[test]
fn whitespace_comments_and_source_id_do_not_change_witness() {
    let a =
        compile_core_plan_v06(&source("fn add(a,b){a+b} entry main returns add(20,22);")).unwrap();
    let b_source = SourceText::new(
        SourceId::new(999),
        "other.noi",
        "// comment\nfn add ( a , b ) { /* pure */ a + b }\nentry main returns add(20, 22);",
    )
    .unwrap();
    let b = compile_core_plan_v06(&b_source).unwrap();
    assert_eq!(
        a.canonical_v06_semantic_bytes(),
        b.canonical_v06_semantic_bytes()
    );
}

#[test]
fn recursive_functions_fail_closed() {
    let error = compile_core_plan_v06(&source(
        "fn loop(x) { loop(x) } entry main returns loop(1);",
    ))
    .unwrap_err();
    assert!(error.is_frontend_failure());
    assert!(error.to_string().contains("recursive pure function"));
}

#[test]
fn indirect_recursion_fails_closed() {
    let error = compile_core_plan_v06(&source(
        "fn a(x){b(x)} fn b(x){a(x)} entry main returns a(1);",
    ))
    .unwrap_err();
    assert!(error.to_string().contains("recursive pure function"));
}

#[test]
fn wrong_arity_and_unknown_function_fail_closed() {
    assert!(
        compile_core_plan_v06(&source("fn add(a,b){a+b} entry main returns add(1);",))
            .unwrap_err()
            .to_string()
            .contains("expects 2 argument")
    );
    assert!(compile_core_plan_v06(&source("entry main returns missing(1);")).is_err());
}

#[test]
fn division_by_zero_fails_before_runtime() {
    let error = compile_core_plan_v06(&source("entry main returns 42 / 0;")).unwrap_err();
    assert!(error.is_frontend_failure());
    assert!(error.to_string().contains("division by zero"));
}

#[test]
fn checked_overflow_fails_before_runtime() {
    let error =
        compile_core_plan_v06(&source("entry main returns 9223372036854775807 * 2;")).unwrap_err();
    assert!(error.to_string().contains("multiplication overflow"));
}

#[test]
fn dead_if_branch_is_still_semantically_validated() {
    let error = compile_core_plan_v06(&source(
        "entry main returns if true { 42 } else { missing + 1 };",
    ))
    .unwrap_err();
    assert!(error.to_string().contains("unknown immutable binding"));
}

#[test]
fn if_branch_value_kinds_must_match() {
    let error = compile_core_plan_v06(&source("entry main returns if true { 42 } else { false };"))
        .unwrap_err();
    assert!(error.to_string().contains("same value kind"));
}

#[test]
fn local_binding_shadowing_is_rejected() {
    let error = compile_core_plan_v06(&source(
        "const x = 20; fn f(a) { const x = 22; x } entry main returns f(1);",
    ))
    .unwrap_err();
    assert!(error.to_string().contains("shadows an existing name"));
}

#[test]
fn lowering_is_always_const_halt_for_closed_pure_program() {
    let plan = compile_core_plan_v06(&source(
        "fn calc(a,b){ (a * b) / 2 } entry main returns calc(6,14);",
    ))
    .unwrap();
    let lowering = lower_core_plan_v06(&plan).unwrap();
    assert_eq!(lowering.result(), CoreValue::Int(42));
    assert_eq!(lowering.nair_instruction_count(), 2);
    assert_eq!(lowering.nair_format_minor(), 6);
}

#[test]
fn semantic_module_and_type_effect_prelude_survive_v06() {
    let report = execute_core_source_v06(&source(
        "module demo.core; type User; effect Network; fn add(a,b){a+b} entry main returns add(20,22);",
    ))
    .unwrap();
    assert_eq!(report.plan().module(), Some("demo.core"));
    assert_eq!(report.result(), CoreValue::Int(42));
}
