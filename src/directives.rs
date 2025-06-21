use crate::{Context, TemplateError, Token, Value};
use std::fmt::Debug;

pub trait Directive: Debug {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError>;
}

pub trait Parser {
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive>>;
}

/// A directive that does nothing,
/// just returns the original content.
///
/// Useful where you want the parser to have
/// a fallback directive so it doesn't return `None`.
///
/// In fact, it is the implementation for the default parser.
#[derive(Debug)]
pub struct NoDirective(String);

impl Directive for NoDirective {
    fn execute(&self, _: &Context) -> Result<String, TemplateError> {
        Ok(self.0.to_string())
    }
}

#[derive(Debug)]
pub struct ReplaceDirective(String);

impl Directive for ReplaceDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        if let Some(v) = ctx.get(self.0.as_str()) {
            Ok(v.to_string())
        } else {
            Err(TemplateError::NoValueFound(self.0.clone()))
        }
    }
}

#[derive(Debug)]
pub struct RepeatDirective(String, String);

impl Directive for RepeatDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        // Check if the literal is a context value
        // If not, use it directly
        let pattern = match ctx.get(self.0.as_str()) {
            Some(p) => p.to_string(),
            None => self.0.to_string(),
        };

        // Check if count is a context value
        // If not, check if it can be parsed into a usize,
        // If not return an error
        let count = match ctx.get(self.1.as_str()) {
            Some(c) => match c {
                Value::Int(i) if *i > 0 => *i as usize,
                _ => return Err(TemplateError::NonUIntForCountVariable(self.1.clone())),
            },
            None => self
                .1
                .parse::<usize>()
                .map_err(|_| TemplateError::NonUIntForCountVariable(self.1.clone()))?,
        };

        Ok(pattern.repeat(count))
    }
}

pub struct DefaultParser;

impl Parser for DefaultParser {
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive>> {
        match tokens {
            // {variable}
            [Token::Literal(s)] => Some(Box::new(ReplaceDirective(s.clone()))),

            // {pattern:count}
            [fist_part, Token::Symbol(':'), second_part] => Some(Box::new(RepeatDirective(
                fist_part.to_string(),
                second_part.to_string(),
            ))),

            // Just return the original string
            t => Some(Box::new(NoDirective(
                t.iter().map(|token| token.to_string()).collect::<String>(),
            ))),
        }
    }
}
