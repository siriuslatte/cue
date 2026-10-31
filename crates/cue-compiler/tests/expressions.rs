use cue_compiler::{check, compile};
use rstest::rstest;

#[rstest]
#[case::literal_overflow("let x = 1e309", "finite")]
#[case::bad_exponent("let x = 1e+", "invalid number")]
#[case::unknown("let x = missing", "unknown name")]
#[case::self_reference("let x = x + 1", "unknown name")]
#[case::duplicate("let x = 1; let x = 2", "already defined")]
#[case::immutable("let x = [1]; x[1] = 2", "immutable")]
#[case::immutable_record("let x = { y: 1 }; x.y = 2", "immutable")]
#[case::wrong_assignment("var x = 1; x = true", "expected number")]
#[case::wrong_binary_initializer("let x: bool = 1 + 2", "expected bool")]
#[case::wrong_condition("if 1 + 2 {}", "expected bool")]
#[case::wrong_unary("let x = not 0", "expected bool")]
#[case::wrong_boolean("let x = 0 and true", "expected bool")]
#[case::wrong_concat("let x = \"a\" .. 1", "expected string")]
#[case::wrong_add("let x = \"a\" + \"b\"", "expected number")]
#[case::wrong_order("let x = \"a\" < \"b\"", "expected number")]
#[case::comparison_chain("let x = 1 < 2 < 3", "cannot be chained")]
#[case::wrong_equality("let x = \"3\" == 3", "cannot compare")]
#[case::nonoptional_none_equality("let x = 1 == none", "cannot compare")]
#[case::different_record_fields(
    "let a: { x: number? } = { x: 1 }; let b = { x: 1 }; let result = a == b",
    "cannot compare"
)]
#[case::skipped_unknown("let x = false and missing", "unknown name")]
#[case::skipped_type_error("let x = false and (\"3\" + 1 > 0)", "expected number")]
#[case::none_without_context("let x = none", "cannot infer")]
#[case::empty_without_context("let x = []", "cannot infer")]
#[case::optional_arithmetic("let x: number? = 1; let y = x + 1", "expected number")]
#[case::nonoptional_coalesce("let x = 1 ?? 2", "requires an optional")]
#[case::wrong_fallback("let x: number? = none; let y = x ?? false", "expected number")]
#[case::mixed_list("let x = [1, true]", "one type")]
#[case::non_list("let x = 3[1]", "requires a list")]
#[case::wrong_index("let x = [1][true]", "expected number")]
#[case::unknown_field("let x = { count: 1 }; let y = x.cout", "unknown field")]
#[case::duplicate_field("let x = { count: 1, count: 2 }", "duplicate field")]
#[case::missing_field("let x: { count: number } = {}", "missing field")]
#[case::extra_field("let x: { count: number } = { count: 1, typo: 2 }", "unknown field")]
#[case::duplicate_type_field("let x: { a: number, a: bool } = {}", "duplicate field")]
#[case::unknown_type("let x: Numeric = 1", "unknown type")]
#[case::loop_not_list("for item in 5 {}", "requires a list")]
#[case::loop_immutable("for item in [1] { item = 2 }", "immutable")]
#[case::loop_scope("for item in [1] {} let x = item", "unknown name")]
#[case::block_scope("if true { let item = 1 } let x = item", "unknown name")]
#[case::non_record_assignment("var x = 1; x.y = 2", "requires a non-optional record")]
#[case::non_list_assignment("var x = 1; x[1] = 2", "requires a non-optional list")]
#[case::unknown_assignment_field("var x = { count: 1 }; x.cout = 2", "unknown field")]
#[case::bad_target("([1])[1] = 2", "assignment target")]
#[case::invalidated_narrowing(
    "var x: number? = 1; if x != none { x = none; let y = x + 1 }",
    "expected number"
)]
#[case::conditional_invalidation(
    "var x: number? = 1; if x != none { if true { x = none } let y = x + 1 }",
    "expected number"
)]
#[case::loop_invalidation(
    "var x: number? = 1; if x != none { for n in [1] { x = none } let y = x + 1 }",
    "expected number"
)]
#[case::loop_carried_invalidation(
    "var x: number? = 1; if x != none { for n in [1, 2] { let y = x + 1; x = none } }",
    "expected number"
)]
#[case::wrong_len("let x = len(3)", "requires a list")]
#[case::wrong_string("let x = string([1])", "requires a number, bool, or string")]
#[case::wrong_arity("let x = len([1], [2])", "one argument")]
#[case::unknown_function("let x = wat(1)", "unknown function")]
fn rejects_invalid_expressions(#[case] body: &str, #[case] message: &str) {
    let source = format!("scene TEST {{ {body} }}");
    let errors = check(&source).expect_err("source should fail checking");
    assert!(
        errors.iter().any(|error| error.message.contains(message)),
        "{errors:?}"
    );
    assert_eq!(errors, compile(&source).unwrap_err());
}

#[rstest]
#[case::empty_enum("enum State {}", "at least one")]
#[case::duplicate_enum("enum State { Idle } enum State { Active }", "already defined")]
#[case::reserved_type("enum number { One }", "already defined")]
#[case::duplicate_variant("enum State { Idle, Idle }", "already defined")]
#[case::unknown_payload("enum State { Active(Missing) }", "unknown type")]
#[case::missing_payload(
    "enum State { Active(number) } scene TEST { let x = State.Active }",
    "requires one payload"
)]
#[case::extra_payload(
    "enum State { Idle } scene TEST { let x = State.Idle(1) }",
    "no payload"
)]
#[case::bad_payload(
    "enum State { Active(number) } scene TEST { let x = State.Active(true) }",
    "expected number"
)]
#[case::unknown_variant(
    "enum State { Idle } scene TEST { let x = State.Active }",
    "unknown variant"
)]
#[case::enum_annotation(
    "enum State { Idle } scene TEST { let x: bool = State.Idle }",
    "expected bool"
)]
#[case::enum_nominality(
    "enum A { Idle } enum B { Idle } scene TEST { let x = A.Idle == B.Idle }",
    "cannot compare"
)]
#[case::missing_match(
    "enum A { One, Two } scene TEST { match A.One { A.One {} } }",
    "missing variants: Two"
)]
#[case::duplicate_match(
    "enum A { One } scene TEST { match A.One { A.One {} A.One {} } }",
    "duplicate match arm"
)]
#[case::payload_pattern(
    "enum A { One(number) } scene TEST { match A.One(1) { A.One {} } }",
    "payload binding"
)]
#[case::pattern_scope(
    "enum A { One(number) } scene TEST { match A.One(1) { A.One(n) {} } let x = n }",
    "unknown name"
)]
fn rejects_invalid_enums(#[case] source: &str, #[case] message: &str) {
    let errors = check(source).expect_err("source should fail checking");
    assert!(
        errors.iter().any(|error| error.message.contains(message)),
        "{errors:?}"
    );
}

#[rstest]
#[case::annotations("let a: number[] = []; let b: number? = none; let c = [none, 1]")]
#[case::records("let a: { count: number, label: string? } = { count: 3 }")]
#[case::nested_context("let a: { count: number? }[] = [{}, { count: 1 }, { count: none }]")]
#[case::shadowing("let count = 3; if true { let count = count + 2 } let result = count")]
#[case::narrowing("let x: number? = 1; if x != none { let y = x + 1 }")]
#[case::else_narrowing("let x: number? = 1; if x == none {} else { let y = x + 1 }")]
#[case::and_narrowing("let x: number? = 1; if x != none and x > 0 { let y = x + 1 }")]
#[case::or_narrowing("let x: number? = 1; if x == none or x > 0 {}")]
#[case::negated_narrowing("let x: number? = 1; if not (none == x) { let y = x + 1 }")]
#[case::narrowed_write("var x: { a: number }? = { a: 1 }; if x != none { x.a = 2; let y = x.a }")]
#[case::stable_loop_narrowing(
    "var x: number? = 1; if x != none { for n in [1, 2] { let y = x + n } }"
)]
#[case::shadowed_loop_write(
    "var x: number? = 1; if x != none { for n in [1, 2] { let y = x + n; var x = 3; x = 4 } }"
)]
#[case::nested_write("var x = [{ a: 1 }]; x[1].a = 2")]
#[case::parenthesized_comparison("let x = (1 < 2) == true")]
#[case::multiline("let x = [\n1,\n2,\n]\nlet y = x[\n1\n] ?? 0")]
#[case::binding_namespace("let NPC = 1; [NPC] \"Still the actor\"")]
fn accepts_typed_bodies(#[case] body: &str) {
    check(&format!("actor NPC \"npc\" scene TEST {{ {body} }}")).unwrap();
}

#[test]
fn points_at_the_invalid_operand() {
    let source = "scene TEST {\n    let count = 1 + true\n}";
    let error = &check(source).unwrap_err()[0];
    assert_eq!(error.location(source), (2, 21));
    assert_eq!(&source[error.span.start..error.span.end].trim(), &"true");
}

#[test]
fn rejects_excessive_nesting_without_overflowing_the_stack() {
    let source = format!(
        "scene TEST {{ let x = {}1{} }}",
        "(".repeat(200),
        ")".repeat(200)
    );
    assert!(check(&source).unwrap_err()[0].message.contains("nesting"));
}

#[test]
fn rejects_long_operator_chains_without_overflowing_the_stack() {
    let source = format!("scene TEST {{ let x = 1{} }}", " + 1".repeat(10000));
    assert!(
        check(&source).unwrap_err()[0]
            .message
            .contains("expression nesting")
    );
}

#[rstest]
fn accepts_conformance_fixtures(
    #[files("tests/fixtures/conformance/*.cue")] path: std::path::PathBuf,
) {
    let source = std::fs::read_to_string(path).unwrap();
    check(&source).unwrap();
    assert_eq!(compile(&source).unwrap(), compile(&source).unwrap());
}
