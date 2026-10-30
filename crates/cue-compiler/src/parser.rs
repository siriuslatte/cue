use std::collections::VecDeque;

use crate::Diagnostic;
use crate::ast::{Actor, Dialogue, Name, Program, Scene};
use crate::lexer::{Kind, Token};

pub(crate) fn parse(tokens: Vec<Token>) -> Result<Program, Diagnostic> {
    let mut parser = Parser {
        tokens: tokens.into(),
    };
    let mut program = Program::default();

    while parser.peek().kind != Kind::End {
        let declaration = parser.name()?;
        match declaration.value.as_str() {
            "actor" => program.actors.push(Actor {
                name: parser.name()?,
                id: parser.text()?,
            }),
            "scene" => program.scenes.push(parser.scene()?),
            _ => {
                return Err(Diagnostic::new(
                    "expected 'actor' or 'scene'",
                    declaration.span,
                ));
            }
        }
    }

    Ok(program)
}

struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    fn peek(&self) -> &Token {
        self.tokens.front().expect("token stream ends with EOF")
    }

    fn take(&mut self) -> Token {
        self.tokens.pop_front().expect("token stream ends with EOF")
    }

    fn name(&mut self) -> Result<Name, Diagnostic> {
        let token = self.take();
        match token.kind {
            Kind::Name(value) => Ok(Name {
                value,
                span: token.span,
            }),
            _ => Err(Diagnostic::new("expected an identifier", token.span)),
        }
    }

    fn text(&mut self) -> Result<String, Diagnostic> {
        let token = self.take();
        match token.kind {
            Kind::Text(value) => Ok(value),
            _ => Err(Diagnostic::new("expected a quoted string", token.span)),
        }
    }

    fn expect(&mut self, kind: Kind, message: &str) -> Result<(), Diagnostic> {
        let token = self.take();
        if token.kind == kind {
            Ok(())
        } else {
            Err(Diagnostic::new(message, token.span))
        }
    }

    fn scene(&mut self) -> Result<Scene, Diagnostic> {
        let name = self.name()?;
        self.expect(Kind::LeftBrace, "expected '{' after the scene name")?;
        let mut lines = Vec::new();

        while self.peek().kind != Kind::RightBrace {
            if self.peek().kind == Kind::End {
                return Err(Diagnostic::new(
                    "expected '}' to close the scene",
                    self.peek().span,
                ));
            }
            self.expect(Kind::LeftBracket, "expected '[ACTOR]' before dialogue")?;
            let actor = self.name()?;
            self.expect(Kind::RightBracket, "expected ']' after the actor name")?;
            lines.push(Dialogue {
                actor,
                text: self.text()?,
            });
        }
        self.take();
        Ok(Scene { name, lines })
    }
}
