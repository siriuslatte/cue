use crate::{Diagnostic, Span};

#[derive(Debug, PartialEq)]
pub(crate) enum Kind {
    Name(String),
    Text(String),
    Number(f64),
    LeftParen,
    RightParen,
    Colon,
    Comma,
    Semicolon,
    Dot,
    Concat,
    Question,
    Coalesce,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,
    Assign,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    End,
}

#[derive(Debug)]
pub(crate) struct Token {
    pub kind: Kind,
    pub span: Span,
    pub new_line: bool,
}

pub(crate) fn scan(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut scanner = Scanner { source, offset: 0 };
    let mut tokens = Vec::new();
    let mut new_line = false;

    while let Some(character) = scanner.peek() {
        let start = scanner.offset;
        let kind = match character {
            c if c.is_whitespace() => {
                new_line |= c == '\n';
                scanner.next();
                continue;
            }
            '#' => {
                while scanner.peek().is_some_and(|c| c != '\n') {
                    scanner.next();
                }
                continue;
            }
            '"' => Kind::Text(scanner.text()?),
            c if c.is_ascii_alphabetic() || c == '_' => {
                scanner.next();
                while scanner
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    scanner.next();
                }
                Kind::Name(source[start..scanner.offset].to_owned())
            }
            c if c.is_ascii_digit() => Kind::Number(scanner.number()?),
            '{' => scanner.symbol(Kind::LeftBrace),
            '}' => scanner.symbol(Kind::RightBrace),
            '[' => scanner.symbol(Kind::LeftBracket),
            ']' => scanner.symbol(Kind::RightBracket),
            '(' => scanner.symbol(Kind::LeftParen),
            ')' => scanner.symbol(Kind::RightParen),
            ':' => scanner.symbol(Kind::Colon),
            ',' => scanner.symbol(Kind::Comma),
            ';' => scanner.symbol(Kind::Semicolon),
            '+' => scanner.symbol(Kind::Plus),
            '-' => scanner.symbol(Kind::Minus),
            '*' => scanner.symbol(Kind::Star),
            '/' => scanner.symbol(Kind::Slash),
            '%' => scanner.symbol(Kind::Percent),
            '^' => scanner.symbol(Kind::Power),
            '.' => scanner.pair('.', Kind::Concat, Kind::Dot),
            '?' => scanner.pair('?', Kind::Coalesce, Kind::Question),
            '=' => scanner.pair('=', Kind::Equal, Kind::Assign),
            '<' => scanner.pair('=', Kind::LessEqual, Kind::Less),
            '>' => scanner.pair('=', Kind::GreaterEqual, Kind::Greater),
            '!' if source[start..].starts_with("!=") => {
                scanner.pair('=', Kind::NotEqual, Kind::NotEqual)
            }
            _ => {
                scanner.next();
                return Err(Diagnostic::new(
                    format!("unexpected character '{character}'"),
                    Span {
                        start,
                        end: scanner.offset,
                    },
                ));
            }
        };
        tokens.push(Token {
            kind,
            new_line,
            span: Span {
                start,
                end: scanner.offset,
            },
        });
        new_line = false;
    }

    tokens.push(Token {
        kind: Kind::End,
        new_line,
        span: Span {
            start: source.len(),
            end: source.len(),
        },
    });
    Ok(tokens)
}

struct Scanner<'a> {
    source: &'a str,
    offset: usize,
}

impl Scanner<'_> {
    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    fn symbol(&mut self, kind: Kind) -> Kind {
        self.next();
        kind
    }

    fn pair(&mut self, second: char, pair: Kind, single: Kind) -> Kind {
        self.next();
        if self.peek() == Some(second) {
            self.next();
            pair
        } else {
            single
        }
    }

    fn number(&mut self) -> Result<f64, Diagnostic> {
        let start = self.offset;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.next();
        }
        if self.peek() == Some('.')
            && self.source[self.offset + 1..].starts_with(|c: char| c.is_ascii_digit())
        {
            self.next();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.next();
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.next();
            if matches!(self.peek(), Some('+' | '-')) {
                self.next();
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.next();
            }
        }
        let literal = &self.source[start..self.offset];
        let span = Span {
            start,
            end: self.offset,
        };
        let value: f64 = literal
            .parse()
            .map_err(|_| Diagnostic::new("invalid number literal", span))?;
        if !value.is_finite() {
            return Err(Diagnostic::new("number literal must be finite", span));
        }
        Ok(value)
    }

    fn text(&mut self) -> Result<String, Diagnostic> {
        let start = self.offset;
        self.next();
        let mut value = String::new();

        while let Some(character) = self.next() {
            match character {
                '"' => return Ok(value),
                '\n' | '\r' => break,
                '\\' => match self.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some('0') => value.push('\0'),
                    Some(escape) => {
                        return Err(Diagnostic::new(
                            format!("unknown escape sequence '\\{escape}'"),
                            Span {
                                start: self.offset - escape.len_utf8() - 1,
                                end: self.offset,
                            },
                        ));
                    }
                    None => break,
                },
                _ => value.push(character),
            }
        }

        Err(Diagnostic::new(
            "unterminated string",
            Span {
                start,
                end: self.offset,
            },
        ))
    }
}
