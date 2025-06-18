mod delimiters;
mod error;

use std::collections::HashMap;

pub use delimiters::*;
pub use error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

impl Alignment {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Alignment::Left),
            '>' => Some(Alignment::Right),
            '^' => Some(Alignment::Center),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
}

impl Value {
    pub fn from_str(s: &str) -> Self {
        match s {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => {
                if let Ok(i) = s.parse::<i64>() {
                    Value::Int(i)
                } else {
                    Value::String(s.to_string())
                }
            }
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
        }
    }
}

type Context = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Replace(String),
    Repeat {
        pattern: String,
        count_var: String,
    },
    Conditional {
        cond_var: String,
        true_part: String,
        false_part: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Literal(String),
    Operation(Op),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template<D: Delimiter> {
    parts: Vec<Part>,
    alignment: Alignment,
    _delimiter: std::marker::PhantomData<D>,
}

impl<D: Delimiter> Template<D> {
    pub fn parse<T: AsRef<str>>(t: T) -> Result<Self, TemplateError> {
        let t = t.as_ref();

        Template::<D>::validate_delimiters(&t)?;

        let mut chars = t.chars().peekable();
        let mut alignment = Alignment::default();
        let mut parts = Vec::new();

        let open = D::open();
        let closed = D::closed();

        // Keep track of the current literal string,
        // meaning that if a non-variable is encountered,
        // accumulate it, and when an open bracket is found, push it
        let mut literal = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                c if c == open => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &open {
                            // Double opening character
                            // Escape sequence
                            chars.next();
                            literal.push(open);
                            continue;
                        }
                    }

                    // If a string literal has been accumulated,
                    // push its content and clear it
                    if !literal.is_empty() {
                        parts.push(Part::Literal(literal.clone()));
                        literal.clear();
                    }

                    // The text content in-between the delimiters
                    let mut content = String::new();

                    while let Some(var_ch) = chars.next() {
                        if var_ch == closed {
                            break;
                        }

                        content.push(var_ch);
                    }

                    // Check for alignment characters at the end
                    if let Some(last_ch) = content.chars().last() {
                        if let Some(a) = Alignment::from_char(last_ch) {
                            alignment = a;
                            content = content[..content.len() - last_ch.len_utf8()].to_string();
                        }
                    }

                    parts.push(Part::Operation(Self::parse_operation(&content)))
                }
                c if c == closed => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &closed {
                            // Double closing character
                            // Escape sequence
                            chars.next();
                            literal.push(closed);
                        }
                    }
                }

                _ => literal.push(ch),
            }
        }

        // Check for hanging literals
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }

        Ok(Template {
            parts,
            alignment,
            _delimiter: std::marker::PhantomData,
        })
    }

    fn validate_delimiters(input: &str) -> Result<(), TemplateError> {
        let mut depth = 0;

        let chars = input.chars();
        let open = D::open();
        let closed = D::closed();

        if open == closed {
            let delimiters = chars.filter(|c| c == &open).count();

            if delimiters % 2 != 0 {
                return Err(TemplateError::MissingDelimiter(closed));
            }

            return Ok(());
        }

        for ch in chars {
            match ch {
                c if c == open => depth += 1,
                c if c == closed => {
                    depth -= 1;

                    if depth < 0 {
                        return Err(TemplateError::MissingOpenDelimiter(open));
                    }
                }
                _ => {}
            }
        }

        if depth > 0 {
            return Err(TemplateError::MissingClosedDelimiter(closed));
        }

        Ok(())
    }

    fn parse_operation(content: &str) -> Op {
        let has_colon = content.contains(':');
        let has_qm = content.contains('?');

        // Repeat operation
        if has_colon && !has_qm {
            let parts = content.split(':').collect::<Vec<_>>();

            if parts.len() != 2 {
                return Op::Replace(content.to_owned());
            }

            let pattern = parts[0].to_string();
            let count_var = parts[1].to_string();

            return Op::Repeat { pattern, count_var };
        }

        // Conditional operation
        if has_qm {
            let parts = content.split('?').collect::<Vec<_>>();

            if parts.len() != 2 {
                return Op::Replace(content.to_owned());
            }

            let cond = parts[1].split(':').collect::<Vec<_>>();

            if cond.len() != 2 {
                return Op::Replace(content.to_owned());
            }

            let cond_var = parts[0].to_string();
            let true_part = cond[0].to_string();
            let false_part = cond[1].to_string();

            return Op::Conditional {
                cond_var,
                true_part,
                false_part,
            };
        }

        Op::Replace(content.to_owned())
    }

    pub fn format(&self, ctx: &Context) -> Result<String, TemplateError> {
        let mut result = String::new();

        for p in self.parts() {
            match p {
                Part::Literal(s) => result.push_str(s),
                Part::Operation(Op::Replace(name)) => match ctx.get(name) {
                    Some(v) => result.push_str(&v.to_string()),
                    None => return Err(TemplateError::NoValueFound(name.to_string())),
                },
                Part::Operation(Op::Repeat { pattern, count_var }) => {
                    let pattern = match ctx.get(pattern) {
                        Some(value) => value.to_string(),
                        None => pattern.to_string(),
                    };

                    let count = if let Some(count) = ctx.get(count_var) {
                        count
                    } else {
                        &Value::from_str(&count_var)
                    };

                    match count {
                        Value::Int(i) if *i >= 0 => {
                            result.push_str(&pattern.repeat((*i).try_into().unwrap()))
                        }
                        _ => {
                            return Err(TemplateError::NonUIntForCountVariable(
                                count_var.to_string(),
                            ));
                        }
                    }
                }

                // TODO: Conditional operations
                _ => {}
            }
        }

        Ok(result)
    }

    pub fn parts(&self) -> &Vec<Part> {
        &self.parts
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use crate::{delimiters::*, *};

    #[test]
    fn delimiter_validation_curly() {
        let r = Template::<CurlyBrackets>::parse("My name is {name}!");

        assert!(r.is_ok());

        let r = Template::<CurlyBrackets>::parse("{greet, I am {name}!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter('}'))));

        let r = Template::<CurlyBrackets>::parse("{greet}, I am name}");

        assert!(matches!(r, Err(TemplateError::MissingOpenDelimiter('{'))))
    }

    #[test]
    fn delimiter_validation_square() {
        let r = Template::<SquareBrackets>::parse("My name is [name]!");

        assert!(r.is_ok());

        let r = Template::<SquareBrackets>::parse("[greet, I am [name]!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter(']'))));

        let r = Template::<SquareBrackets>::parse("[greet], I am name]");

        assert!(matches!(r, Err(TemplateError::MissingOpenDelimiter('['))))
    }

    #[test]
    fn delimiter_validation_parentheses() {
        let r = Template::<Parentheses>::parse("My name is (name)!");

        assert!(r.is_ok());

        let r = Template::<Parentheses>::parse("(greet, I am (name)!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter(')'))));

        let r = Template::<Parentheses>::parse("(greet), I am name)");

        assert!(matches!(r, Err(TemplateError::MissingOpenDelimiter('('))))
    }

    #[test]
    fn delimiter_validation_angle() {
        let r = Template::<AngleBrackets>::parse("My name is <name>!");

        assert!(r.is_ok());

        let r = Template::<AngleBrackets>::parse("<greet, I am <name>!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter('>'))));

        let r = Template::<AngleBrackets>::parse("<greet>, I am name>");

        assert!(matches!(r, Err(TemplateError::MissingOpenDelimiter('<'))))
    }

    #[test]
    fn delimiter_validation_custom() {
        struct MyDelimiter;

        impl Delimiter for MyDelimiter {
            fn open() -> char {
                'd'
            }

            fn closed() -> char {
                'b'
            }
        }

        let r = Template::<MyDelimiter>::parse("My name is dnameb!");

        assert!(r.is_ok());

        let r = Template::<MyDelimiter>::parse("dgreet, I am dnameb!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter('b'))));

        let r = Template::<MyDelimiter>::parse("dgreetb, I am nameb");

        assert!(matches!(r, Err(TemplateError::MissingOpenDelimiter('d'))))
    }

    #[test]
    fn delimiter_validation_equivalent() {
        struct MyDelimiter;

        impl Delimiter for MyDelimiter {
            fn open() -> char {
                '/'
            }

            fn closed() -> char {
                '/'
            }
        }

        let r = Template::<MyDelimiter>::parse("My name is /name/!");

        assert!(r.is_ok());

        let r = Template::<MyDelimiter>::parse("/greet, I am /name/!");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter('/'))));

        let r = Template::<MyDelimiter>::parse("/greet/, I am name/");

        assert!(matches!(r, Err(TemplateError::MissingClosedDelimiter('/'))))
    }
}
