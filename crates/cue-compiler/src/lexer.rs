use crate::{Diagnostic, Span};

#[derive(Debug, PartialEq)]
pub(crate) enum Kind {
    Name(String),
    Text(String),
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
}

pub(crate) fn scan(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut scanner = Scanner { source, offset: 0 };
    let mut tokens = Vec::new();

    while let Some(character) = scanner.peek() {
        let start = scanner.offset;
        let kind = match character {
            c if c.is_whitespace() => {
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
            '{' => {
                scanner.next();
                Kind::LeftBrace
            }
            '}' => {
                scanner.next();
                Kind::RightBrace
            }
            '[' => {
                scanner.next();
                Kind::LeftBracket
            }
            ']' => {
                scanner.next();
                Kind::RightBracket
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
            span: Span {
                start,
                end: scanner.offset,
            },
        });
    }

    tokens.push(Token {
        kind: Kind::End,
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
