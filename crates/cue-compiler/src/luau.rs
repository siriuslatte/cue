use std::fmt::Write;

use crate::{
    PROGRAM_VERSION, Span,
    ast::{Binary, Unary},
    checked::{self, Access, Expr, ExprKind, StmtKind},
};

pub(crate) fn emit(program: &checked::Program, source: &str) -> String {
    let mut emitter = Emitter {
        output: String::new(),
        indent: 0,
        temporary: 0,
        source,
        scene: "",
        line_starts: vec![0],
        spill_locals: false,
    };
    emitter
        .line_starts
        .extend(source.match_indices('\n').map(|(index, _)| index + 1));
    emitter.line("-- Compiled by Cue.");
    emitter.line("return {");
    emitter.indent += 1;
    emitter.line(&format!("version = {PROGRAM_VERSION},"));
    emitter.line("scenes = {");
    emitter.indent += 1;
    for scene in &program.scenes {
        emitter.scene = &scene.name;
        // Reserve registers for expression temporaries. Large scenes use one
        // indexed frame; small scenes keep ordinary Luau locals.
        emitter.spill_locals = scene.locals > 32;
        emitter.line(&format!("[{}] = function(cue)", quote(&scene.name)));
        emitter.indent += 1;
        if emitter.spill_locals {
            emitter.line(&format!("local vars = table.create({})", scene.locals));
        }
        emitter.statements(&scene.body);
        emitter.indent -= 1;
        emitter.line("end,");
    }
    emitter.indent -= 1;
    emitter.line("},");
    emitter.indent -= 1;
    emitter.line("}");
    emitter.output
}

struct Emitter<'a> {
    output: String,
    indent: usize,
    temporary: usize,
    source: &'a str,
    scene: &'a str,
    line_starts: Vec<usize>,
    spill_locals: bool,
}

impl Emitter<'_> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        writeln!(self.output, "{text}").unwrap();
    }

    fn open(&mut self, text: &str) {
        self.line(text);
        self.indent += 1;
    }

    fn close(&mut self) {
        self.indent -= 1;
        self.line("end");
    }

    fn temp(&mut self) -> String {
        let name = format!("t{}", self.temporary);
        self.temporary += 1;
        name
    }

    fn store(&mut self, expression: &str) -> String {
        let name = self.temp();
        self.line(&format!("local {name} = {expression}"));
        name
    }

    fn location(&self, span: Span) -> String {
        let line = self
            .line_starts
            .partition_point(|start| *start <= span.start);
        let column = self.source[self.line_starts[line - 1]..span.start]
            .chars()
            .count()
            + 1;
        quote(&format!("{}:{line}:{column}", self.scene))
    }

    fn local(&self, id: usize) -> String {
        if self.spill_locals {
            format!("vars[{}]", id + 1)
        } else {
            format!("v{id}")
        }
    }

    fn statements(&mut self, statements: &[checked::Stmt]) {
        for statement in statements {
            if let StmtKind::Bind(local, _) = &statement.kind
                && !self.spill_locals
            {
                self.line(&format!("local {}", self.local(*local)));
            }
            // Temporaries live only for this statement, rather than accumulating
            // against Luau's per-function register limit across a long scene.
            self.open("do");
            self.line(&format!("cue.tick({})", self.location(statement.span)));
            match &statement.kind {
                StmtKind::Dialogue(actor, text) => {
                    let text = self.expression(text);
                    self.line(&format!("cue.dialogue({}, {text})", quote(actor)));
                }
                StmtKind::Bind(local, value) => {
                    let value = self.expression(value);
                    self.line(&format!("{} = {value}", self.local(*local)));
                }
                StmtKind::Assign { local, path, value } => {
                    let mut containers = Vec::new();
                    let mut receiver = self.local(*local);
                    for access in path {
                        let key = match access {
                            Access::Field(name) => quote(name),
                            Access::Index(index) => {
                                let key = self.expression(index);
                                self.line(&format!(
                                    "cue.index({receiver}, {key}, true, {})",
                                    self.location(index.span)
                                ));
                                key
                            }
                        };
                        let child = self.store(&format!("{receiver}[{key}]"));
                        containers.push((receiver, key));
                        receiver = child;
                    }
                    let mut value = self.expression(value);
                    for (container, key) in containers.into_iter().rev() {
                        value = self.store(&format!("cue.set({container}, {key}, {value})"));
                    }
                    self.line(&format!("{} = {value}", self.local(*local)));
                }
                StmtKind::If { condition, yes, no } => {
                    let condition = self.expression(condition);
                    self.open(&format!("if {condition} then"));
                    self.statements(yes);
                    if !no.is_empty() {
                        self.indent -= 1;
                        self.line("else");
                        self.indent += 1;
                        self.statements(no);
                    }
                    self.close();
                }
                StmtKind::For {
                    local,
                    collection,
                    body,
                } => {
                    let collection = self.expression(collection);
                    let collection = self.store(&collection);
                    let index = self.temp();
                    self.open(&format!("for {index} = 1, #{collection} do"));
                    self.line(&format!("cue.tick({})", self.location(statement.span)));
                    self.line(&format!(
                        "{}{} = {collection}[{index}]",
                        if self.spill_locals { "" } else { "local " },
                        self.local(*local)
                    ));
                    self.statements(body);
                    self.close();
                }
                StmtKind::Match { value, arms } => {
                    let value = self.expression(value);
                    for (index, arm) in arms.iter().enumerate() {
                        if index > 0 {
                            self.indent -= 1;
                        }
                        self.open(&format!(
                            "{} {value}.variant == {} then",
                            if index == 0 { "if" } else { "elseif" },
                            quote(&arm.variant)
                        ));
                        if let Some(local) = arm.binding {
                            self.line(&format!(
                                "{}{} = {value}.payload",
                                if self.spill_locals { "" } else { "local " },
                                self.local(local)
                            ));
                        }
                        self.statements(&arm.body);
                    }
                    self.close();
                }
                StmtKind::Block(body) => self.statements(body),
            }
            self.close();
        }
    }

    fn expression(&mut self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Number(value) => return value.to_string(),
            ExprKind::Bool(value) => return value.to_string(),
            ExprKind::Text(value) => return quote(value),
            ExprKind::None => return "cue.none".into(),
            ExprKind::Local(local) => return self.local(*local),
            _ => {}
        }
        let result = self.temp();
        self.line(&format!("local {result}"));
        self.open("do");
        let expression = self.compute(expr);
        self.line(&format!("{result} = {expression}"));
        self.close();
        result
    }

    fn compute(&mut self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Number(value) => value.to_string(),
            ExprKind::Bool(value) => value.to_string(),
            ExprKind::Text(value) => quote(value),
            ExprKind::None => "cue.none".into(),
            ExprKind::Local(local) => self.local(*local),
            ExprKind::List(items) => {
                let list = self.store(&format!("table.create({})", items.len()));
                for (index, item) in items.iter().enumerate() {
                    self.open("do");
                    let value = self.expression(item);
                    self.line(&format!("{list}[{}] = {value}", index + 1));
                    self.close();
                }
                format!("table.freeze({list})")
            }
            ExprKind::Record(fields) => {
                let record = self.store("{}");
                for (name, item) in fields {
                    self.open("do");
                    let value = self.expression(item);
                    self.line(&format!("{record}[{}] = {value}", quote(name)));
                    self.close();
                }
                format!("table.freeze({record})")
            }
            ExprKind::Enum {
                enumeration,
                variant,
                payload,
            } => {
                let payload = payload
                    .as_ref()
                    .map(|value| self.expression(value))
                    .unwrap_or_else(|| "cue.none".into());
                format!(
                    "table.freeze({{ enum = {}, variant = {}, payload = {payload} }})",
                    quote(enumeration),
                    quote(variant)
                )
            }
            ExprKind::Unary(operator, value) => {
                let value = self.expression(value);
                match operator {
                    Unary::Not => format!("not {value}"),
                    Unary::Negate => format!("(-{value})"),
                }
            }
            ExprKind::Binary(operator, left, right) => {
                self.binary(*operator, left, right, expr.span)
            }
            ExprKind::Field(receiver, field) => {
                let receiver = self.expression(receiver);
                format!("{receiver}[{}]", quote(field))
            }
            ExprKind::Index(receiver, index) => {
                let receiver = self.expression(receiver);
                let index = self.expression(index);
                format!(
                    "cue.index({receiver}, {index}, false, {})",
                    self.location(expr.span)
                )
            }
            ExprKind::Length(value) => {
                let value = self.expression(value);
                format!("#{value}")
            }
            ExprKind::String(value) => {
                let value = self.expression(value);
                format!("tostring({value})")
            }
        }
    }

    fn binary(&mut self, operator: Binary, left: &Expr, right: &Expr, span: Span) -> String {
        use Binary::*;
        let left = self.expression(left);
        if matches!(operator, And | Or | Coalesce) {
            let result = self.store(&left);
            let test = match operator {
                And => result.clone(),
                Or => format!("not {result}"),
                _ => format!("{result} == cue.none"),
            };
            self.open(&format!("if {test} then"));
            let right = self.expression(right);
            self.line(&format!("{result} = {right}"));
            self.close();
            return result;
        }
        let right = self.expression(right);
        match operator {
            Equal => format!("cue.equal({left}, {right}, {})", self.location(span)),
            NotEqual => format!("not cue.equal({left}, {right}, {})", self.location(span)),
            Add | Subtract | Multiply | Divide | Remainder | Power => {
                let op = match operator {
                    Add => "+",
                    Subtract => "-",
                    Multiply => "*",
                    Divide => "/",
                    Remainder => "%",
                    _ => "^",
                };
                if matches!(operator, Divide | Remainder) {
                    self.line(&format!("cue.divisor({right}, {})", self.location(span)));
                }
                format!("cue.number({left} {op} {right}, {})", self.location(span))
            }
            _ => {
                let op = match operator {
                    Less => "<",
                    LessEqual => "<=",
                    Greater => ">",
                    GreaterEqual => ">=",
                    Concat => "..",
                    _ => unreachable!("short-circuit operators handled above"),
                };
                format!("{left} {op} {right}")
            }
        }
    }
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
