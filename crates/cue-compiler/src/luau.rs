use std::fmt::Write;

use crate::PROGRAM_VERSION;
use crate::check::CheckedProgram;

pub(crate) fn emit(program: &CheckedProgram) -> String {
    let mut output = format!(
        "-- Compiled by Cue.\nreturn {{\n    version = {PROGRAM_VERSION},\n    scenes = {{\n"
    );
    for scene in &program.scenes {
        writeln!(output, "        [{}] = {{", quote(&scene.name)).unwrap();
        for line in &scene.lines {
            writeln!(
                output,
                "            {{ actor = {}, text = {} }},",
                quote(&line.actor),
                quote(&line.text)
            )
            .unwrap();
        }
        output.push_str("        },\n");
    }
    output.push_str("    },\n}\n");
    output
}

fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => {
                for byte in c.encode_utf8(&mut [0; 4]).bytes() {
                    write!(output, "\\{byte:03}").unwrap();
                }
            }
            c => output.push(c),
        }
    }
    output.push('"');
    output
}
