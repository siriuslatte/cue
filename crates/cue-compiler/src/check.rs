use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use crate::Diagnostic;
use crate::ast::Program;

pub(crate) struct CheckedProgram {
    pub scenes: Vec<Scene>,
}

pub(crate) struct Scene {
    pub name: String,
    pub lines: Vec<Dialogue>,
}

pub(crate) struct Dialogue {
    pub actor: String,
    pub text: String,
}

pub(crate) fn check(program: Program) -> Result<CheckedProgram, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    let mut actors = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut scenes = Vec::new();

    for actor in program.actors {
        match actors.entry(actor.name.value) {
            Entry::Occupied(entry) => errors.push(Diagnostic::new(
                format!("actor '{}' is already defined", entry.key()),
                actor.name.span,
            )),
            Entry::Vacant(entry) => {
                entry.insert(actor.id);
            }
        }
    }

    for scene in program.scenes {
        if !names.insert(scene.name.value.clone()) {
            errors.push(Diagnostic::new(
                format!("scene '{}' is already defined", scene.name.value),
                scene.name.span,
            ));
        }
        let mut lines = Vec::new();
        for line in scene.lines {
            match actors.get(&line.actor.value) {
                Some(actor) => lines.push(Dialogue {
                    actor: actor.clone(),
                    text: line.text,
                }),
                None => errors.push(Diagnostic::new(
                    format!("unknown actor '{}'", line.actor.value),
                    line.actor.span,
                )),
            }
        }
        scenes.push(Scene {
            name: scene.name.value,
            lines,
        });
    }

    if errors.is_empty() {
        Ok(CheckedProgram { scenes })
    } else {
        Err(errors)
    }
}
