use std::{fs, path::PathBuf};

use cue_compiler::{check, compile};
use rstest::rstest;

#[rstest]
fn compiles_valid_fixtures(#[files("tests/fixtures/pass/*.cue")] path: PathBuf) {
    let source = fs::read_to_string(&path).unwrap();
    let output = compile(&source).expect("valid fixture should compile");
    let name = format!("output_{}", path.file_stem().unwrap().to_str().unwrap());

    insta::with_settings!({ description => source.as_str() }, {
        insta::assert_snapshot!(name, output);
    });
}

#[rstest]
fn rejects_invalid_fixtures(#[files("tests/fixtures/fail/*.cue")] path: PathBuf) {
    let source = fs::read_to_string(&path).unwrap();
    let errors = check(&source).expect_err("invalid fixture should fail checking");
    assert_eq!(compile(&source).unwrap_err(), errors);
    let name = format!("errors_{}", path.file_stem().unwrap().to_str().unwrap());

    insta::with_settings!({ description => source.as_str() }, {
        insta::assert_debug_snapshot!(name, errors);
    });
}

#[rstest]
#[case::actor("actor")]
#[case::actor_value("actor NPC")]
#[case::scene("scene")]
#[case::scene_body("scene INTRO")]
#[case::dialogue_actor("scene INTRO { [")]
#[case::dialogue_text("scene INTRO { [NPC]")]
#[case::dialogue_bracket("scene INTRO { [NPC")]
#[case::scene_close("scene INTRO {")]
#[case::string_close("actor NPC \"unfinished")]
#[case::unknown_escape(r#"actor NPC "bad\q""#)]
fn rejects_incomplete_or_malformed_source(#[case] source: &str) {
    let errors = compile(source).expect_err("malformed source should fail compilation");
    assert!(!errors.is_empty());
}

#[rstest]
#[case::multiline("# hello\nscene INTRO {\n    [MISSING] \"Hello\"\n}", (3, 6))]
#[case::utf8_column("actor NPC \"café\" scene INTRO { [MISSING] \"Hello\" }", (1, 33))]
fn reports_character_locations(#[case] source: &str, #[case] expected: (usize, usize)) {
    let errors = check(source).unwrap_err();
    assert_eq!(errors[0].location(source), expected);
}

#[rstest]
#[case::example(include_str!("../../../examples/hello.cue"))]
#[case::empty("")]
#[case::comment("# An empty content file is valid.\n")]
fn accepts_complete_programs(#[case] source: &str) {
    check(source).unwrap();
}

#[test]
fn emits_deterministically() {
    let source = include_str!("../../../examples/hello.cue");
    assert_eq!(compile(source).unwrap(), compile(source).unwrap());
}
