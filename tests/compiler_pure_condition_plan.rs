use nordoi_kernel::{
    compile_pure_condition_boundary, compile_pure_condition_execution_plan_boundary,
    PureConditionPlanForm, SemanticPureComparator, SemanticPureCondition, SourceId, SourceText,
};

fn source(id: u32, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), "condition-plan.noi", text).unwrap()
}

#[test]
fn boolean_true_plans_as_zero_work_zero_storage() {
    let plan =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns true;"))
            .unwrap();
    assert_eq!(plan.result_bool(), Some(true));
    assert_eq!(plan.work_item_count(), 0);
    assert_eq!(plan.runtime_storage_item_count(), 0);
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn boolean_false_remains_a_real_planned_value() {
    let plan =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns false;"))
            .unwrap();
    assert_eq!(plan.result_bool(), Some(false));
    assert!(matches!(
        plan.condition().unwrap().condition(),
        SemanticPureCondition::Bool(false)
    ));
}

#[test]
fn comparison_plan_preserves_operands_and_comparator() {
    let plan =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns 20 <= 22;"))
            .unwrap();
    assert_eq!(plan.result_bool(), Some(true));
    assert!(matches!(
        plan.condition().unwrap().condition(),
        SemanticPureCondition::IntCompare {
            lhs: 20,
            comparator: SemanticPureComparator::Le,
            rhs: 22
        }
    ));
}

#[test]
fn all_comparators_keep_their_l09_meaning_in_the_plan() {
    let cases = [
        ("1 == 1", true),
        ("1 != 1", false),
        ("1 < 2", true),
        ("2 <= 2", true),
        ("3 > 4", false),
        ("4 >= 4", true),
    ];
    for (index, (condition, expected)) in cases.into_iter().enumerate() {
        let text = format!("entry main returns {condition};");
        let plan = compile_pure_condition_execution_plan_boundary(&source(index as u32 + 1, &text))
            .unwrap();
        assert_eq!(plan.result_bool(), Some(expected), "condition={condition}");
    }
}

#[test]
fn c011_witness_has_explicit_domain() {
    let plan =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns true;"))
            .unwrap();
    assert!(plan
        .canonical_c011_bytes()
        .starts_with(b"NORDOI-C0.11-PURE-CONDITION-PLAN\0"));
}

#[test]
fn c011_preserves_exact_l09_witness() {
    let src = source(1, "module demo; entry main returns 5 >= 5;");
    let l09 = compile_pure_condition_boundary(&src).unwrap();
    let plan = compile_pure_condition_execution_plan_boundary(&src).unwrap();
    assert_eq!(
        l09.canonical_l09_bytes(),
        plan.condition_semantics().canonical_l09_bytes()
    );
}

#[test]
fn equal_semantics_ignore_comments_spacing_and_source_id() {
    let a = compile_pure_condition_execution_plan_boundary(&source(
        1,
        "module demo; effect Net; entry main returns 20<=22;",
    ))
    .unwrap();
    let b = compile_pure_condition_execution_plan_boundary(&source(
        99,
        "module /*x*/ demo ; effect Net ; entry /*y*/ main returns 20 <= 22 ;",
    ))
    .unwrap();
    assert_eq!(a.canonical_c011_bytes(), b.canonical_c011_bytes());
}

#[test]
fn same_truth_from_different_condition_forms_keeps_distinct_plan_identity() {
    let literal =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns true;"))
            .unwrap();
    let compare =
        compile_pure_condition_execution_plan_boundary(&source(2, "entry main returns 1 < 2;"))
            .unwrap();
    assert_eq!(literal.result_bool(), compare.result_bool());
    assert_ne!(
        literal.canonical_c011_bytes(),
        compare.canonical_c011_bytes()
    );
}

#[test]
fn same_truth_from_different_comparators_keeps_distinct_plan_identity() {
    let lt =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns 1 < 2;"))
            .unwrap();
    let ne =
        compile_pure_condition_execution_plan_boundary(&source(2, "entry main returns 1 != 2;"))
            .unwrap();
    assert_eq!(lt.result_bool(), Some(true));
    assert_eq!(ne.result_bool(), Some(true));
    assert_ne!(lt.canonical_c011_bytes(), ne.canonical_c011_bytes());
}

#[test]
fn module_identity_changes_plan_identity() {
    let a = compile_pure_condition_execution_plan_boundary(&source(
        1,
        "module a; entry main returns true;",
    ))
    .unwrap();
    let b = compile_pure_condition_execution_plan_boundary(&source(
        2,
        "module b; entry main returns true;",
    ))
    .unwrap();
    assert_ne!(a.canonical_c011_bytes(), b.canonical_c011_bytes());
}

#[test]
fn entry_name_changes_plan_identity() {
    let a = compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns true;"))
        .unwrap();
    let b = compile_pure_condition_execution_plan_boundary(&source(2, "entry other returns true;"))
        .unwrap();
    assert_ne!(a.canonical_c011_bytes(), b.canonical_c011_bytes());
}

#[test]
fn empty_body_has_no_condition_and_zero_cost() {
    let plan = compile_pure_condition_execution_plan_boundary(&source(1, "module demo;")).unwrap();
    assert!(matches!(plan.form(), PureConditionPlanForm::Empty));
    assert_eq!(plan.result_bool(), None);
    assert_eq!(plan.work_item_count(), 0);
    assert_eq!(plan.runtime_storage_item_count(), 0);
    assert!(plan.required_effects().is_empty());
}

#[test]
fn plain_entry_is_distinct_from_false_condition() {
    let none = compile_pure_condition_execution_plan_boundary(&source(1, "entry main;")).unwrap();
    let no =
        compile_pure_condition_execution_plan_boundary(&source(2, "entry main returns false;"))
            .unwrap();
    assert_eq!(none.result_bool(), None);
    assert_eq!(no.result_bool(), Some(false));
    assert_ne!(none.canonical_c011_bytes(), no.canonical_c011_bytes());
}

#[test]
fn declared_effect_does_not_become_plan_requirement() {
    let plan = compile_pure_condition_execution_plan_boundary(&source(
        1,
        "effect Network; entry main returns 1 < 2;",
    ))
    .unwrap();
    assert!(plan.required_effects().is_empty());
    assert!(plan.is_pure());
    assert!(!plan.requires_host_authority());
}

#[test]
fn plan_is_deterministic_for_equal_source() {
    let src = source(1, "module demo; entry main returns 7 != 8;");
    let a = compile_pure_condition_execution_plan_boundary(&src).unwrap();
    let b = compile_pure_condition_execution_plan_boundary(&src).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.canonical_c011_bytes(), b.canonical_c011_bytes());
}

#[test]
fn older_l09_semantic_boundary_remains_unchanged() {
    let src = source(1, "entry main returns 9 < 10;");
    let before = compile_pure_condition_boundary(&src).unwrap();
    let _plan = compile_pure_condition_execution_plan_boundary(&src).unwrap();
    let after = compile_pure_condition_boundary(&src).unwrap();
    assert_eq!(before.canonical_l09_bytes(), after.canonical_l09_bytes());
}

#[test]
fn c011_plan_does_not_require_nair_or_runtime_state() {
    let plan =
        compile_pure_condition_execution_plan_boundary(&source(1, "entry main returns 4 == 4;"))
            .unwrap();
    assert_eq!(plan.work_item_count(), 0);
    assert_eq!(plan.runtime_storage_item_count(), 0);
    assert!(!plan.requires_host_authority());
}
