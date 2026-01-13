#![warn(clippy::use_self)]

mod directive;
mod lexer;
mod parser;
mod traits;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Write};

pub use directive::*;
pub use lexer::*;
pub use parser::*;

use crate::traits::ToAstring;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Str(Cow<'a, str>),
    StaticStr(&'static str),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl<'a> Value<'a> {
    pub fn write_to(&self, buf: &mut String) -> fmt::Result {
        match self {
            Self::Str(s) => write!(buf, "{}", s)?,
            Self::StaticStr(s) => write!(buf, "{}", s)?,
            Self::Int(n) => write!(buf, "{}", n)?,
            Self::Float(n) => write!(buf, "{}", n)?,
            Self::Bool(b) => write!(buf, "{}", b)?,
        }

        Ok(())
    }
}

impl<'a> Value<'a> {
    pub fn as_cow(&'a self) -> Cow<'a, str> {
        match self {
            Self::Str(s) => Cow::Borrowed(s),
            Self::StaticStr(s) => Cow::Borrowed(s),
            Self::Int(v) => Cow::Owned(v.to_astring()),
            Self::Float(v) => Cow::Owned(v.to_astring()),
            Self::Bool(v) => Cow::Owned(v.to_string()),
        }
    }
}

pub type Context<'a> = HashMap<&'static str, Value<'a>>;

pub struct Template<'a, const O: char, const C: char> {
    directives: Vec<Box<dyn Directive + 'a>>,
}

impl<'a, const C: char, const O: char> fmt::Debug for Template<'a, O, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Template<'{}', '{}'>", O, C)
    }
}

impl<'a, const O: char, const C: char> Template<'a, O, C> {
    pub fn compile<P: Parser>(input: &'a str) -> Result<Self, String> {
        let mut directives: Vec<Box<dyn Directive>> = Vec::new();
        let mut cursor = 0;
        let mut chars = input.char_indices().peekable();

        let arena = RefCell::new(String::new());

        while let Some((idx, ch)) = chars.next() {
            if ch == O {
                // Handle escaped opening delimiter (e.g. "{{")
                if let Some(&(_, next_char)) = chars.peek() {
                    if next_char == O {
                        if idx > cursor {
                            directives.push(Box::new(LiteralDirective(Cow::Borrowed(
                                &input[cursor..idx],
                            ))));
                        }

                        directives.push(Box::new(LiteralDirective(Cow::Owned(O.to_string()))));
                        chars.next();
                        cursor = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
                        continue;
                    }
                }

                if idx > cursor {
                    directives.push(Box::new(LiteralDirective(Cow::Borrowed(
                        &input[cursor..idx],
                    ))));
                }

                let start = idx + ch.len_utf8();
                let mut depth = 1;
                let mut end = start;
                let mut found_close = false;

                while let Some((c_idx, c_char)) = chars.next() {
                    if O == C {
                        if c_char == C {
                            depth -= 1;
                        }
                    } else {
                        if c_char == O {
                            depth += 1;
                        } else if c_char == C {
                            depth -= 1;
                        }
                    }

                    if depth == 0 {
                        end = c_idx;
                        found_close = true;
                        cursor = c_idx + C.len_utf8();
                        break;
                    }
                }

                if !found_close {
                    return Err(format!("Unclosed delimiter '{}'", O));
                }

                let content = &input[start..end];

                arena.borrow_mut().clear();

                let tokens: Vec<Token> = TemplateLexer::new(content).collect();

                match P::parse(&tokens) {
                    Some(directive) => directives.push(directive),
                    None => return Err(format!("Failed to parse expression: '{}'", content)),
                }
            } else if ch == C {
                if let Some(&(_, next_char)) = chars.peek() {
                    if next_char == C {
                        if idx > cursor {
                            directives.push(Box::new(LiteralDirective(Cow::Borrowed(
                                &input[cursor..idx],
                            ))));
                        }

                        directives.push(Box::new(LiteralDirective(Cow::Owned(C.to_string()))));
                        chars.next();
                        cursor = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
                        continue;
                    }
                }
            }
        }

        if cursor < input.len() {
            directives.push(Box::new(LiteralDirective(Cow::Borrowed(&input[cursor..]))));
        }

        Ok(Self { directives })
    }

    pub fn format(&self, ctx: &Context, buf: &mut String) -> fmt::Result {
        buf.reserve(self.directives.len() * 8);

        for directive in &self.directives {
            directive.format(ctx, buf)?;
        }

        Ok(())
    }
}
