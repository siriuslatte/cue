mod ast;
mod check;
mod checked;
mod diagnostic;
mod lexer;
mod luau;
mod parser;
mod types;

pub use diagnostic::{Diagnostic, Span};

pub const PROGRAM_VERSION: u32 = 2;

/// Compile Cue source into a Luau module accepted by the runtime.
pub fn compile(source: &str) -> Result<String, Vec<Diagnostic>> {
    let tokens = lexer::scan(source).map_err(|error| vec![error])?;
    let program = parser::parse(tokens).map_err(|error| vec![error])?;
    let checked = check::check(program)?;
    Ok(luau::emit(&checked, source))
}

/// Check a source file without producing output.
pub fn check(source: &str) -> Result<(), Vec<Diagnostic>> {
    let tokens = lexer::scan(source).map_err(|error| vec![error])?;
    let program = parser::parse(tokens).map_err(|error| vec![error])?;
    check::check(program).map(|_| ())
}
