use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Project(PathBuf);

impl Project {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "cue-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("main.cue"), source).unwrap();
        Self(path)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cue"));
        command.current_dir(&self.0);
        command
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn builds_a_module_and_creates_output_directories() {
    let project = Project::new("scene INTRO {}");
    let output = project
        .command()
        .args(["build", "main.cue", "-o", "build/main.luau"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(project.0.join("build/main.luau"))
            .unwrap()
            .contains("version = 1")
    );
}

#[test]
fn invalid_source_does_not_replace_existing_output() {
    let project = Project::new("scene INTRO { [UNKNOWN] \"Hello\" }");
    fs::write(project.0.join("main.luau"), "previous build").unwrap();
    let output = project
        .command()
        .args(["build", "main.cue", "-o", "main.luau"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("main.cue:1:16: error: unknown actor")
    );
    assert_eq!(
        fs::read_to_string(project.0.join("main.luau")).unwrap(),
        "previous build"
    );
}

#[test]
fn refuses_to_overwrite_the_source() {
    let project = Project::new("scene INTRO {}");
    let output = project
        .command()
        .args(["build", "main.cue", "-o", "main.cue"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(project.0.join("main.cue")).unwrap(),
        "scene INTRO {}"
    );
}

#[test]
fn check_has_no_output_side_effects() {
    let project = Project::new("scene INTRO {}");
    let output = project
        .command()
        .args(["check", "main.cue"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(fs::read_dir(&project.0).unwrap().count(), 1);
}

#[test]
fn missing_input_is_an_error() {
    let project = Project::new("");
    let output = project
        .command()
        .args(["check", "missing.cue"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing.cue"));
}
