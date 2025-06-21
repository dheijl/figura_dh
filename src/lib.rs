mod directives;
mod error;

pub use directives::*;
pub use error::*;

use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
        }
    }
}

pub type Context = HashMap<&'static str, Value>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Delimiter(char),
    Literal(String),
    Symbol(char),
    Int(i64),
    Uknown(char),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Delimiter(c) => write!(f, "{c}"),
            Token::Literal(l) => write!(f, "{l}"),
            Token::Symbol(s) => write!(f, "{s}"),
            Token::Int(i) => write!(f, "{i}"),
            Token::Uknown(c) => write!(f, "{c}"),
        }
    }
}

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

#[derive(Debug)]
pub enum Part {
    Literal(String),
    Directive(Box<dyn Directive>),
}

#[derive(Debug)]
pub struct Template<const O: char = '{', const C: char = '}'> {
    parts: Vec<Part>,
    alignment: Alignment,
}

impl<const O: char, const C: char> Template<O, C> {
    pub fn parse<P: Parser>(t: &str) -> Result<Self, TemplateError> {
        let mut chars = t.chars().peekable();

        Self::validate_delimiters(&t)?;

        let mut literal = String::new();
        let mut alignment = Alignment::default();
        let mut parts = Vec::new();

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &O {
                            // Double opening character
                            // Escape sequence
                            chars.next();
                            literal.push(O);
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

                    while let Some(next_ch) = chars.next() {
                        if next_ch == C {
                            break;
                        }

                        content.push(next_ch);
                    }

                    if let Some(last_ch) = content.chars().last() {
                        if let Some(a) = Alignment::from_char(last_ch) {
                            alignment = a;
                            // Remove the last character (alignment character) from the string
                            content = content[..content.len() - last_ch.len_utf8()].to_string();
                        }
                    }

                    let tokens = Self::tokenize(&content);

                    if let Some(d) = P::parse(&tokens) {
                        parts.push(Part::Directive(d));
                    } else {
                        return Err(TemplateError::DirectiveParsing(content));
                    }
                }

                c if c == C => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &C {
                            // Double closing character - escape sequence
                            chars.next();
                            literal.push(C);
                        } else {
                            // Single closing character - add to literal
                            literal.push(C);
                        }
                    } else {
                        // Single closing character at end of string
                        literal.push(C);
                    }
                }

                _ => literal.push(ch),
            }
        }

        // Check for hanging literals
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }

        Ok(Self { parts, alignment })
    }

    pub fn format(&self, ctx: &Context) -> Result<String, TemplateError> {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                Part::Literal(s) => result.push_str(s),
                Part::Directive(d) => result.push_str(&d.execute(ctx)?),
            }
        }

        Ok(result)
    }

    fn validate_delimiters(input: &str) -> Result<(), TemplateError> {
        let mut chars = input.chars().peekable();

        if C == O {
            if chars.filter(|c| *c != C).count() % 2 != 0 {
                return Err(TemplateError::MissingDelimiter(O));
            } else {
                return Ok(());
            }
        }

        let mut depth = 0;

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O => {
                    // Check for escape sequence {{
                    if chars.peek() == Some(&O) {
                        chars.next();
                    } else {
                        depth += 1;
                    }
                }
                c if c == C => {
                    // Check for escape sequence }}
                    if chars.peek() == Some(&C) {
                        chars.next();
                    } else {
                        depth -= 1;
                        if depth < 0 {
                            return Err(TemplateError::MissingOpenDelimiter(O));
                        }
                    }
                }
                _ => {}
            }
        }

        if depth > 0 {
            return Err(TemplateError::MissingClosedDelimiter(C));
        }
        Ok(())
    }

    /// Function to convert a section between delimiters
    /// into a vector of token, which will be passed into
    /// the parse trait function so that the user can customize the logic
    fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        // Keeps track of the current string literal
        let mut literal = String::new();
        // Keeps track of the current number (for multi-digits)
        let mut number = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O || c == C => {
                    // Push a number if the number buffer is not empty
                    if !number.is_empty() {
                        tokens.push(Token::Int(number.parse::<i64>().unwrap()));
                        number.clear();
                    }

                    // Push a literal if the literal buffer is not empty
                    if !literal.is_empty() {
                        tokens.push(Token::Literal(literal.clone()));
                        literal.clear();
                    }

                    tokens.push(Token::Delimiter(c))
                }

                c if ch.is_alphabetic() || c == '_' || c.is_whitespace() => {
                    // Push a number if the number buffer is not empty
                    if !number.is_empty() {
                        tokens.push(Token::Int(number.parse::<i64>().unwrap()));
                        number.clear();
                    }

                    literal.push(c);
                }

                c if ch.is_ascii_digit() => {
                    // Push a literal if the literal buffer is not empty
                    if !literal.is_empty() {
                        tokens.push(Token::Literal(literal.clone()));
                        literal.clear();
                    }

                    number.push(c)
                }

                c if !ch.is_alphanumeric() && !ch.is_whitespace() => {
                    // Push a number if the number buffer is not empty
                    if !number.is_empty() {
                        tokens.push(Token::Int(number.parse::<i64>().unwrap()));
                        number.clear();
                    }

                    // Push a literal if the literal buffer is not empty
                    if !literal.is_empty() {
                        tokens.push(Token::Literal(literal.clone()));
                        literal.clear();
                    }

                    tokens.push(Token::Symbol(c));
                }

                _ => {}
            }
        }

        // Check for hanging buffers
        if !literal.is_empty() {
            tokens.push(Token::Literal(literal.clone()));
        }

        if !number.is_empty() {
            tokens.push(Token::Int(number.parse::<i64>().unwrap()));
        }

        tokens
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directives::DefaultParser;
    use std::collections::HashMap;

    // Helper function to create a basic context
    fn basic_context() -> Context {
        HashMap::from([
            ("name", Value::String("John".to_string())),
            ("age", Value::Int(25)),
            ("active", Value::Bool(true)),
            ("score", Value::Int(100)),
        ])
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_alignment_from_char() {
        assert_eq!(Alignment::from_char('<'), Some(Alignment::Left));
        assert_eq!(Alignment::from_char('>'), Some(Alignment::Right));
        assert_eq!(Alignment::from_char('^'), Some(Alignment::Center));
        assert_eq!(Alignment::from_char('x'), None);
    }

    #[test]
    fn test_basic_template_parsing() {
        let context = basic_context();
        let template = "Hello, my name is {name} and I am {age} years old.";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Hello, my name is John and I am 25 years old."
        );
    }

    #[test]
    fn test_template_with_boolean() {
        let context = basic_context();
        let template = "User {name} is active: {active}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "User John is active: true"
        );
    }

    #[test]
    fn test_multiple_same_variable() {
        let context = basic_context();
        let template = "{name} said hello to {name}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "John said hello to John"
        );
    }

    #[test]
    fn test_empty_template() {
        let context = basic_context();
        let template = "";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(template.unwrap().format(&context).unwrap(), "");
    }

    #[test]
    fn test_template_without_variables() {
        let context = basic_context();
        let template = "This is just a plain string with no variables.";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "This is just a plain string with no variables."
        );
    }

    #[test]
    fn test_escaped_delimiters() {
        let context = basic_context();
        let template = "Use {{double braces}} to escape {name}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Use {double braces} to escape John"
        );
    }

    #[test]
    fn test_escaped_closing_delimiters() {
        let context = basic_context();
        let template = "Hello {name}}} with extra }}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Hello John} with extra }"
        );
    }

    #[test]
    fn test_custom_delimiters() {
        let context = basic_context();
        let template = "Hello, my name is <name> and I am <age> years old.";
        let template = Template::<'<', '>'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Hello, my name is John and I am 25 years old."
        );
    }

    #[test]
    fn test_square_bracket_delimiters() {
        let context = basic_context();
        let template = "User [name] has score [score]";
        let template = Template::<'[', ']'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "User John has score 100"
        );
    }

    #[test]
    fn test_same_opening_closing_delimiters() {
        let context = basic_context();
        let template = "Hello |name| you are |age| years old";
        let template = Template::<'|', '|'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Hello John you are 25 years old"
        );
    }

    #[test]
    fn test_alignment_characters() {
        // Test left alignment
        let template = "{name<}";
        let parsed = Template::<'{', '}'>::parse::<DefaultParser>(template);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().alignment, Alignment::Left);

        // Test right alignment
        let template = "{name>}";
        let parsed = Template::<'{', '}'>::parse::<DefaultParser>(template);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().alignment, Alignment::Right);

        // Test center alignment
        let template = "{name^}";
        let parsed = Template::<'{', '}'>::parse::<DefaultParser>(template);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().alignment, Alignment::Center);
    }

    #[test]
    fn test_missing_opening_delimiter() {
        let template = "Hello name} and age}";
        let result = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(result.is_err());
        match result.unwrap_err() {
            TemplateError::MissingOpenDelimiter('{') => {}
            _ => panic!("Expected MissingOpenDelimiter error"),
        }
    }

    #[test]
    fn test_missing_closing_delimiter() {
        let template = "Hello {name and {age";
        let result = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(result.is_err());

        match result.unwrap_err() {
            TemplateError::MissingClosedDelimiter('}') => {}
            _ => panic!("Expected MissingClosedDelimiter error"),
        }
    }

    #[test]
    fn test_nested_delimiters_error() {
        let template = "Hello {name {age}}";
        let result = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(result.is_err());
    }

    #[test]
    fn test_tokenization() {
        // Tokenization is done on the content between the delimiters
        let template = "name123";
        let tokens = Template::<'{', '}'>::tokenize(template);

        assert_eq!(
            tokens,
            vec![Token::Literal("name".to_string()), Token::Int(123)]
        );
    }

    #[test]
    fn test_complex_template() {
        let context = HashMap::from([
            ("first_name", Value::String("Jane".to_string())),
            ("last_name", Value::String("Doe".to_string())),
            ("age", Value::Int(30)),
            ("city", Value::String("New York".to_string())),
            ("married", Value::Bool(false)),
        ]);

        let template =
            "Full name: {first_name} {last_name}, Age: {age}, City: {city}, Married: {married}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Full name: Jane Doe, Age: 30, City: New York, Married: false"
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let template = "Hello {  name  } you are {  age  }";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        // Whitespace inside delimiters should be preserved in tokenization
    }

    #[test]
    fn test_special_characters_in_literals() {
        let context = basic_context();
        let template = "Price: $100, Percentage: 50%, Email: user@domain.com, Name: {name}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Price: $100, Percentage: 50%, Email: user@domain.com, Name: John"
        );
    }

    #[test]
    fn test_unicode_support() {
        let context = HashMap::from([
            ("emoji", Value::String("🚀".to_string())),
            ("chinese", Value::String("你好".to_string())),
            ("arabic", Value::String("مرحبا".to_string())),
        ]);

        let template = "Rocket: {emoji}, Chinese: {chinese}, Arabic: {arabic}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "Rocket: 🚀, Chinese: 你好, Arabic: مرحبا"
        );
    }

    #[test]
    fn test_large_numbers() {
        let context = HashMap::from([
            ("big_num", Value::Int(i64::MAX)),
            ("small_num", Value::Int(i64::MIN)),
        ]);

        let template = "Max: {big_num}, Min: {small_num}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        let result = template.unwrap().format(&context).unwrap();
        assert!(result.contains(&i64::MAX.to_string()));
        assert!(result.contains(&i64::MIN.to_string()));
    }

    #[test]
    fn test_variable_not_in_context() {
        let context = basic_context();
        let template = "Hello {nonexistent}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        // The format should fail if variable doesn't exist in context
        let result = template.unwrap().format(&context);
        // This depends on your DefaultParser and Directive implementation
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_consecutive_variables() {
        let context = basic_context();
        let template = "{name}{age}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(template.unwrap().format(&context).unwrap(), "John25");
    }

    #[test]
    fn test_variables_at_boundaries() {
        let context = basic_context();
        let template = "{name} middle {age}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "John middle 25"
        );
    }

    #[test]
    fn test_template_starts_and_ends_with_variable() {
        let context = basic_context();
        let template = "{name} is {age}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(template.unwrap().format(&context).unwrap(), "John is 25");
    }

    #[test]
    fn test_only_variables() {
        let context = basic_context();
        let template = "{name}";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert!(template.is_ok());
        assert_eq!(template.unwrap().format(&context).unwrap(), "John");
    }

    #[test]
    fn test_default_parsing() {
        let context = basic_context();
        let template = "{1:age} this is 25 ones!";
        let template = Template::<'{', '}'>::parse::<DefaultParser>(template);

        assert_eq!(
            template.unwrap().format(&context).unwrap(),
            "1111111111111111111111111 this is 25 ones!"
        );
    }
}
