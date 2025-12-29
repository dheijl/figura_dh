use crate::{Context, lexer::Token};

pub trait Directive {
    fn execute(&self, ctx: &Context) -> Result<String, String>;
}

pub trait Parser {
    fn parse(input: &[Token]) -> Result<Vec<Box<dyn Directive>>, Vec<String>>;
}
