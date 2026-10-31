use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::{TempDir, prelude::*};
use predicates::prelude::*;
use rstest::{fixture, rstest};

#[fixture]
fn project() -> TempDir {
    let project = TempDir::new().unwrap();
    project
        .child("main.cue")
        .write_str("scene INTRO {}")
        .unwrap();
    project
}

#[rstest]
fn builds_a_module_and_creates_output_directories(project: TempDir) {
    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["build", "main.cue", "-o", "build/main.luau"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    project
        .child("build/main.luau")
        .assert(predicate::str::contains("version = 2"));
}

#[rstest]
fn invalid_source_does_not_replace_existing_output(project: TempDir) {
    project
        .child("main.cue")
        .write_str("scene INTRO { [UNKNOWN] \"Hello\" }")
        .unwrap();
    project
        .child("main.luau")
        .write_str("previous build")
        .unwrap();

    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["build", "main.cue", "-o", "main.luau"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains(
            "main.cue:1:16: error: unknown actor 'UNKNOWN'",
        ));

    project.child("main.luau").assert("previous build");
}

#[rstest]
fn invalid_source_does_not_create_output(project: TempDir) {
    project
        .child("main.cue")
        .write_str("scene INTRO {")
        .unwrap();

    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["build", "main.cue", "-o", "build/main.luau"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("expected '}' to close the scene"));

    project.child("build").assert(predicate::path::missing());
}

#[rstest]
#[case::same_path("main.cue")]
#[case::relative_alias("./main.cue")]
fn refuses_to_overwrite_the_source(project: TempDir, #[case] output: &str) {
    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["build", "main.cue", "-o", output])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "output must be different from the source file",
        ));

    project.child("main.cue").assert("scene INTRO {}");
}

#[rstest]
fn check_has_no_output_side_effects(project: TempDir) {
    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["check", "main.cue"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
    project.child("main.cue").assert("scene INTRO {}");
}

#[rstest]
fn missing_input_is_an_error(project: TempDir) {
    cargo_bin_cmd!("cue")
        .current_dir(project.path())
        .args(["check", "missing.cue"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("missing.cue"));
}

#[test]
fn help_lists_the_available_commands() {
    cargo_bin_cmd!("cue")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("build").and(predicate::str::contains("check")));
}
