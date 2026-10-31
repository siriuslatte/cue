use std::collections::VecDeque;

use crate::ast::*;
use crate::lexer::{Kind, Token};
use crate::{Diagnostic, Span};

type Result<T> = std::result::Result<T, Diagnostic>;

pub(crate) fn parse(tokens: Vec<Token>) -> Result<Program> {
    let mut parser = Parser {
        tokens: tokens.into(),
        depth: 0,
        last_end: 0,
    };
    let mut program = Program::default();
    while parser.peek().kind != Kind::End {
        let token = parser.take();
        let Kind::Name(declaration) = token.kind else {
            return Err(Diagnostic::new("expected a declaration", token.span));
        };
        match declaration.as_str() {
            "actor" => program.actors.push(Actor {
                name: parser.name()?,
                id: parser.text()?,
            }),
            "enum" => program.enums.push(parser.enumeration()?),
            "scene" => program.scenes.push(Scene {
                name: parser.name()?,
                body: parser.block("scene")?,
            }),
            _ => {
                return Err(Diagnostic::new(
                    "expected 'actor', 'enum', or 'scene'",
                    token.span,
                ));
            }
        }
        parser.eat(&Kind::Semicolon);
    }
    Ok(program)
}

struct Parser {
    tokens: VecDeque<Token>,
    depth: usize,
    last_end: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        self.tokens.front().expect("token stream ends with EOF")
    }

    fn take(&mut self) -> Token {
        let token = self.tokens.pop_front().expect("token stream ends with EOF");
        self.last_end = token.span.end;
        token
    }

    fn eat(&mut self, kind: &Kind) -> bool {
        if &self.peek().kind == kind {
            self.take();
            true
        } else {
            false
        }
    }

    fn word(&mut self, word: &str) -> bool {
        self.eat(&Kind::Name(word.to_owned()))
    }

    fn name(&mut self) -> Result<Name> {
        let token = self.take();
        match token.kind {
            Kind::Name(value)
                if !matches!(
                    value.as_str(),
                    "let"
                        | "var"
                        | "if"
                        | "else"
                        | "for"
                        | "in"
                        | "match"
                        | "true"
                        | "false"
                        | "none"
                        | "not"
                        | "and"
                        | "or"
                        | "actor"
                        | "scene"
                        | "enum"
                ) =>
            {
                Ok(Name {
                    value,
                    span: token.span,
                })
            }
            _ => Err(Diagnostic::new("expected an identifier", token.span)),
        }
    }

    fn text(&mut self) -> Result<String> {
        let token = self.take();
        match token.kind {
            Kind::Text(text) => Ok(text),
            _ => Err(Diagnostic::new("expected a quoted string", token.span)),
        }
    }

    fn expect(&mut self, kind: Kind, message: &str) -> Result<Span> {
        let token = self.take();
        if token.kind == kind {
            Ok(token.span)
        } else {
            Err(Diagnostic::new(message, token.span))
        }
    }

    fn enter(&mut self) -> Result<()> {
        self.depth += 1;
        if self.depth > 128 {
            Err(Diagnostic::new(
                "syntax nesting exceeds 128 levels",
                self.peek().span,
            ))
        } else {
            Ok(())
        }
    }

    fn enumeration(&mut self) -> Result<Enum> {
        let name = self.name()?;
        self.expect(Kind::LeftBrace, "expected '{' after the enum name")?;
        let mut variants = Vec::new();
        while !self.eat(&Kind::RightBrace) {
            let variant = self.name()?;
            let payload = if self.eat(&Kind::LeftParen) {
                let ty = self.ty()?;
                self.expect(Kind::RightParen, "expected ')' after the payload type")?;
                Some(ty)
            } else {
                None
            };
            variants.push((variant, payload));
            if !self.eat(&Kind::Comma) {
                self.expect(Kind::RightBrace, "expected ',' or '}' after the variant")?;
                break;
            }
        }
        Ok(Enum { name, variants })
    }

    fn ty(&mut self) -> Result<TypeExpr> {
        self.enter()?;
        let start = self.peek().span.start;
        let kind = if self.eat(&Kind::LeftBrace) {
            let mut fields = Vec::new();
            while !self.eat(&Kind::RightBrace) {
                let field = self.name()?;
                self.expect(Kind::Colon, "expected ':' after the field name")?;
                fields.push((field, self.ty()?));
                if !self.eat(&Kind::Comma) {
                    self.expect(Kind::RightBrace, "expected ',' or '}' after the field type")?;
                    break;
                }
            }
            TypeKind::Record(fields)
        } else {
            TypeKind::Name(self.name()?.value)
        };
        let mut ty = TypeExpr {
            kind,
            span: Span {
                start,
                end: self.last_end,
            },
        };
        loop {
            let kind = if self.eat(&Kind::Question) {
                TypeKind::Optional(Box::new(ty))
            } else if self.eat(&Kind::LeftBracket) {
                self.expect(Kind::RightBracket, "expected ']' in list type")?;
                TypeKind::List(Box::new(ty))
            } else {
                break;
            };
            ty = TypeExpr {
                kind,
                span: Span {
                    start,
                    end: self.last_end,
                },
            };
        }
        self.depth -= 1;
        Ok(ty)
    }

    fn block(&mut self, description: &str) -> Result<Vec<Stmt>> {
        self.enter()?;
        self.expect(Kind::LeftBrace, "expected '{' to start the block")?;
        let mut body = Vec::new();
        while !self.eat(&Kind::RightBrace) {
            if self.peek().kind == Kind::End {
                return Err(Diagnostic::new(
                    format!("expected '}}' to close the {description}"),
                    self.peek().span,
                ));
            }
            body.push(self.statement()?);
            self.eat(&Kind::Semicolon);
        }
        self.depth -= 1;
        Ok(body)
    }

    fn statement(&mut self) -> Result<Stmt> {
        let start = self.peek().span.start;
        let kind = if self.eat(&Kind::LeftBracket) {
            let actor = self.name()?;
            self.expect(Kind::RightBracket, "expected ']' after the actor name")?;
            // Bare strings remain the common case; expressions use parentheses so
            // the next dialogue's '[' cannot become a list lookup on the text.
            let text = if self.peek().kind == Kind::LeftParen {
                self.take();
                let value = self.expression(0)?;
                self.expect(Kind::RightParen, "expected ')' after dialogue text")?;
                value
            } else {
                let span = self.peek().span;
                Expr {
                    kind: ExprKind::Text(self.text()?),
                    span,
                }
            };
            StmtKind::Dialogue(actor, text)
        } else if matches!(&self.peek().kind, Kind::Name(v) if v == "let" || v == "var") {
            let mutable = self.take().kind == Kind::Name("var".into());
            let name = self.name()?;
            let annotation = if self.eat(&Kind::Colon) {
                Some(self.ty()?)
            } else {
                None
            };
            self.expect(Kind::Assign, "expected '=' after the local binding")?;
            StmtKind::Bind {
                name,
                mutable,
                annotation,
                value: self.expression(0)?,
            }
        } else if self.word("if") {
            self.conditional()?
        } else if self.word("for") {
            let name = self.name()?;
            self.expect(
                Kind::Name("in".into()),
                "expected 'in' after the loop binding",
            )?;
            let collection = self.expression(0)?;
            StmtKind::For {
                name,
                collection,
                body: self.block("loop")?,
            }
        } else if self.word("match") {
            let value = self.expression(0)?;
            self.expect(Kind::LeftBrace, "expected '{' after the match value")?;
            let mut arms = Vec::new();
            while !self.eat(&Kind::RightBrace) {
                let enumeration = self.name()?;
                self.expect(Kind::Dot, "expected '.' before the variant")?;
                let variant = self.name()?;
                let binding = if self.eat(&Kind::LeftParen) {
                    let name = self.name()?;
                    self.expect(Kind::RightParen, "expected ')' after the payload binding")?;
                    Some(name)
                } else {
                    None
                };
                arms.push(Arm {
                    enumeration,
                    variant,
                    binding,
                    body: self.block("match arm")?,
                });
                self.eat(&Kind::Comma);
            }
            StmtKind::Match { value, arms }
        } else if self.peek().kind == Kind::LeftBrace {
            StmtKind::Block(self.block("block")?)
        } else {
            let target = self.expression(0)?;
            self.expect(Kind::Assign, "expected '=' after the assignment target")?;
            StmtKind::Assign(target, self.expression(0)?)
        };
        Ok(Stmt {
            kind,
            span: Span {
                start,
                end: self.last_end,
            },
        })
    }

    fn conditional(&mut self) -> Result<StmtKind> {
        let condition = self.expression(0)?;
        let yes = self.block("conditional")?;
        let no = if self.word("else") {
            if self.word("if") {
                self.enter()?;
                let start = self.peek().span.start;
                let kind = self.conditional()?;
                self.depth -= 1;
                vec![Stmt {
                    kind,
                    span: Span {
                        start,
                        end: self.last_end,
                    },
                }]
            } else {
                self.block("conditional")?
            }
        } else {
            Vec::new()
        };
        Ok(StmtKind::If { condition, yes, no })
    }

    fn expression(&mut self, minimum: u8) -> Result<Expr> {
        self.enter()?;
        let mut left = self.prefix()?;
        let mut comparison = false;
        loop {
            let start = left.span.start;
            let kind = if minimum <= 19 && !self.peek().new_line && self.eat(&Kind::Dot) {
                ExprKind::Field(Box::new(left), self.name()?)
            } else if minimum <= 19 && !self.peek().new_line && self.eat(&Kind::LeftBracket) {
                let index = self.expression(0)?;
                self.expect(Kind::RightBracket, "expected ']' after the index")?;
                ExprKind::Index(Box::new(left), Box::new(index))
            } else if minimum <= 19 && !self.peek().new_line && self.eat(&Kind::LeftParen) {
                let mut args = Vec::new();
                while !self.eat(&Kind::RightParen) {
                    args.push(self.expression(0)?);
                    if !self.eat(&Kind::Comma) {
                        self.expect(Kind::RightParen, "expected ',' or ')' after the argument")?;
                        break;
                    }
                }
                ExprKind::Call(Box::new(left), args)
            } else if let Some((operator, power, right)) = self.operator() {
                if power < minimum {
                    break;
                }
                let token = self.take();
                if operator.comparison() && comparison {
                    return Err(Diagnostic::new("comparisons cannot be chained", token.span));
                }
                comparison |= operator.comparison();
                let rhs = self.expression(if right { power } else { power + 1 })?;
                ExprKind::Binary(operator, Box::new(left), Box::new(rhs))
            } else {
                break;
            };
            left = Expr {
                kind,
                span: Span {
                    start,
                    end: self.last_end,
                },
            };
            if left.depth() > 48 {
                return Err(Diagnostic::new(
                    "expression nesting exceeds 48 levels",
                    left.span,
                ));
            }
        }
        if left.depth() > 48 {
            return Err(Diagnostic::new(
                "expression nesting exceeds 48 levels",
                left.span,
            ));
        }
        self.depth -= 1;
        Ok(left)
    }

    fn prefix(&mut self) -> Result<Expr> {
        let token = self.take();
        let kind = match token.kind {
            Kind::Number(value) => ExprKind::Number(value),
            Kind::Text(value) => ExprKind::Text(value),
            Kind::Name(ref value) if value == "true" || value == "false" => {
                ExprKind::Bool(value == "true")
            }
            Kind::Name(ref value) if value == "none" => ExprKind::None,
            Kind::Minus => ExprKind::Unary(Unary::Negate, Box::new(self.expression(15)?)),
            Kind::Name(ref value) if value == "not" => {
                ExprKind::Unary(Unary::Not, Box::new(self.expression(15)?))
            }
            Kind::Name(value) => ExprKind::Name(Name {
                value,
                span: token.span,
            }),
            Kind::LeftParen => {
                let mut value = self.expression(0)?;
                let end = self
                    .expect(Kind::RightParen, "expected ')' after the expression")?
                    .end;
                value.span = Span {
                    start: token.span.start,
                    end,
                };
                return Ok(value);
            }
            Kind::LeftBracket => {
                let mut items = Vec::new();
                while !self.eat(&Kind::RightBracket) {
                    items.push(self.expression(0)?);
                    if !self.eat(&Kind::Comma) {
                        self.expect(
                            Kind::RightBracket,
                            "expected ',' or ']' after the list item",
                        )?;
                        break;
                    }
                }
                ExprKind::List(items)
            }
            Kind::LeftBrace => {
                let mut fields = Vec::new();
                while !self.eat(&Kind::RightBrace) {
                    let name = self.name()?;
                    self.expect(Kind::Colon, "expected ':' after the field name")?;
                    fields.push((name, self.expression(0)?));
                    if !self.eat(&Kind::Comma) {
                        self.expect(
                            Kind::RightBrace,
                            "expected ',' or '}' after the record field",
                        )?;
                        break;
                    }
                }
                ExprKind::Record(fields)
            }
            _ => return Err(Diagnostic::new("expected an expression", token.span)),
        };
        Ok(Expr {
            kind,
            span: Span {
                start: token.span.start,
                end: self.last_end,
            },
        })
    }

    fn operator(&self) -> Option<(Binary, u8, bool)> {
        use Binary::*;
        Some(match &self.peek().kind {
            Kind::Coalesce => (Coalesce, 1, true),
            Kind::Name(v) if v == "or" => (Or, 3, false),
            Kind::Name(v) if v == "and" => (And, 5, false),
            Kind::Equal => (Equal, 7, false),
            Kind::NotEqual => (NotEqual, 7, false),
            Kind::Less => (Less, 7, false),
            Kind::LessEqual => (LessEqual, 7, false),
            Kind::Greater => (Greater, 7, false),
            Kind::GreaterEqual => (GreaterEqual, 7, false),
            Kind::Concat => (Concat, 9, true),
            Kind::Plus => (Add, 11, false),
            Kind::Minus => (Subtract, 11, false),
            Kind::Star => (Multiply, 13, false),
            Kind::Slash => (Divide, 13, false),
            Kind::Percent => (Remainder, 13, false),
            Kind::Power => (Power, 17, true),
            _ => return None,
        })
    }
}
