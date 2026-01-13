use crate::{
    RepeatDirective,
    directive::{Directive, EmptyDirective, ReplaceDirective},
    lexer::Token,
};
use std::borrow::Cow;

pub trait Parser {
    // The Box must know that the Directive inside lives as long as 'a
    fn parse<'a>(tokens: &[Token<'a>]) -> Option<Box<dyn Directive + 'a>>;
}

pub struct DefaultParser;

impl Parser for DefaultParser {
    fn parse<'a>(tokens: &[Token<'a>]) -> Option<Box<dyn Directive + 'a>> {
        match tokens {
            [Token::Ident(ident)] => Some(Box::new(ReplaceDirective(ident))),

            [
                p @ (Token::Ident(_) | Token::Literal(_)),
                Token::Colon,
                c @ (Token::Ident(_) | Token::Int(_)),
            ] => {
                let pattern = match p {
                    Token::Ident(s) => Cow::Borrowed(*s),
                    Token::Literal(cow) => cow.clone(),
                    _ => unreachable!(),
                };

                let count = match c {
                    Token::Ident(s) | Token::Int(s) => *s,
                    _ => unreachable!(),
                };

                Some(Box::new(RepeatDirective(pattern, count)))
            }

            _ => Some(Box::new(EmptyDirective)),
        }
    }
}
