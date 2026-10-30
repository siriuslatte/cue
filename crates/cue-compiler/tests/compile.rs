use cue_compiler::{check, compile};

#[test]
fn resolves_actors_declared_after_scenes() {
    let source = "scene INTRO { [NPC] \"Hello\" } actor NPC \"mechanic\"";
    let output = compile(source).unwrap();
    assert!(output.contains("actor = \"mechanic\""));
    assert!(output.contains("[\"INTRO\"]"));
}

#[test]
fn reports_unknown_actor_at_its_source_location() {
    let source = "# hello\nscene INTRO {\n    [MISSING] \"Hello\"\n}";
    let errors = compile(source).unwrap_err();
    assert_eq!(errors[0].message, "unknown actor 'MISSING'");
    assert_eq!(errors[0].location(source), (3, 6));
}

#[test]
fn reports_duplicate_declarations() {
    let source = "actor NPC \"a\" actor NPC \"b\" scene INTRO {} scene INTRO {}";
    let errors = check(source).unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(errors[0].message.contains("actor 'NPC' is already defined"));
    assert!(
        errors[1]
            .message
            .contains("scene 'INTRO' is already defined")
    );
}

#[test]
fn rejects_unclosed_scenes() {
    let errors = compile("scene INTRO {").unwrap_err();
    assert_eq!(errors[0].message, "expected '}' to close the scene");
}

#[test]
fn rejects_unterminated_strings_and_unknown_escapes() {
    assert!(check("actor NPC \"unfinished").is_err());
    let errors = check(r#"actor NPC "bad\q""#).unwrap_err();
    assert!(errors[0].message.contains("unknown escape sequence"));
}

#[test]
fn checks_without_requiring_output() {
    assert!(check(include_str!("../../../examples/hello.cue")).is_ok());
    assert!(check("# An empty content file is valid.\n").is_ok());
}

#[test]
fn emits_deterministically() {
    let source = include_str!("../../../examples/hello.cue");
    assert_eq!(compile(source).unwrap(), compile(source).unwrap());
}

#[test]
fn rejects_incomplete_declarations_without_panicking() {
    for source in [
        "actor",
        "actor NPC",
        "scene",
        "scene INTRO",
        "scene INTRO { [",
        "scene INTRO { [NPC]",
        "scene INTRO { [NPC",
    ] {
        assert!(
            compile(source).is_err(),
            "accepted incomplete source: {source}"
        );
    }
}
