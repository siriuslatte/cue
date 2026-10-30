use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cue", version, about = "Compile Cue source into Luau")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile a source file into a Luau module.
    Build {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Check a source file without writing output.
    Check { input: PathBuf },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let (Command::Build { input, .. } | Command::Check { input }) = &cli.command;
    let source =
        fs::read_to_string(input).map_err(|error| format!("{}: {error}", input.display()))?;

    let result = match &cli.command {
        Command::Build { .. } => cue_compiler::compile(&source).map(Some),
        Command::Check { .. } => cue_compiler::check(&source).map(|_| None),
    };
    let generated = result.map_err(|errors| {
        errors
            .iter()
            .map(|error| {
                let (line, column) = error.location(&source);
                format!("{}:{line}:{column}: error: {error}", input.display())
            })
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    if let (Some(generated), Command::Build { input, output }) = (generated, cli.command) {
        if output
            .canonicalize()
            .ok()
            .is_some_and(|path| Some(path) == input.canonicalize().ok())
        {
            return Err("output must be different from the source file".to_owned());
        }
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        fs::write(&output, generated).map_err(|error| format!("{}: {error}", output.display()))?;
    }
    Ok(())
}
