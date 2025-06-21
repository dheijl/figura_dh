//! Template formatting engine core module.

mod directives;
mod error;

pub use directives::*;
pub use error::*;

use std::{collections::HashMap, fmt::Display};

/// A simple value type used in templating context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Boolean value.
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

/// A key-value map representing template context data.
///
/// # Example
/// ```no_run
/// use std::collections::HashMap;
/// use your_crate::Value;
///
/// let mut ctx: Context = HashMap::new();
/// ctx.insert("name", Value::String("Alice".to_string()));
/// ```
pub type Context = HashMap<&'static str, Value>;

/// A token used in directive parsing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    /// Represents a delimiter character (e.g., `{` or `}`).
    Delimiter(char),
    /// A literal string.
    Literal(String),
    /// A symbolic character (e.g., `:`, `+`, etc.).
    Symbol(char),
    /// An integer literal.
    Int(i64),
    /// Any unrecognized character.
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

/// Specifies alignment options for formatted output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Left-aligned output (`<`).
    Left,
    /// Right-aligned output (`>`).
    Right,
    /// Center-aligned output (`^`).
    Center,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

impl Alignment {
    /// Parses a character into an [`Alignment`] option.
    ///
    /// Returns `None` if the character doesn't match any known alignment.
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Alignment::Left),
            '>' => Some(Alignment::Right),
            '^' => Some(Alignment::Center),
            _ => None,
        }
    }
}

/// Represents a part of a parsed template: either a literal or a directive.
#[derive(Debug)]
pub enum Part {
    /// Literal string part of the template.
    Literal(String),
    /// A directive that implements the [`Directive`] trait.
    Directive(Box<dyn Directive>),
}

/// A parsed template, parameterized by opening (`O`) and closing (`C`) delimiters.
///
/// This struct can parse a template string and then format it using a [`Context`].
///
/// # Example
///
/// ```no_run
/// let tpl = Template::parse::<DefaultParser>("Hello, {name}!")?;
/// let mut ctx = Context::new();
/// ctx.insert("name", Value::String("World".into()));
/// let result = tpl.format(&ctx)?;
/// assert_eq!(result, "Hello, World!");
/// ```
#[derive(Debug)]
pub struct Template<const O: char = '{', const C: char = '}'> {
    parts: Vec<Part>,
    alignment: Alignment,
}

impl<const O: char, const C: char> Template<O, C> {
    /// Parses a template string into a [`Template`] instance.
    ///
    /// Returns an error if the template has invalid syntax or cannot be parsed.
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

                    if let Some(d) = P::parse(&tokens, &content) {
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

    /// Formats the template using the given context.
    ///
    /// Each directive is executed against the provided context map.
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

    /// Validates the delimiters in the template string.
    ///
    /// Ensures that all opening delimiters have a matching closing one.
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

    /// Returns the detected alignment of the template
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

#[cfg(test)]
mod tokenization {
    use super::*;

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = Template::<'{', '}'>::tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_simple_literal() {
        let tokens = Template::<'{', '}'>::tokenize("hello");
        assert_eq!(tokens, vec![Token::Literal("hello".to_string())]);
    }

    #[test]
    fn test_tokenize_simple_number() {
        let tokens = Template::<'{', '}'>::tokenize("123");
        assert_eq!(tokens, vec![Token::Int(123)]);
    }

    #[test]
    fn test_tokenize_single_symbol() {
        let tokens = Template::<'{', '}'>::tokenize(":");
        assert_eq!(tokens, vec![Token::Symbol(':')]);
    }

    #[test]
    fn test_tokenize_delimiters() {
        let tokens = Template::<'{', '}'>::tokenize("{}");
        assert_eq!(tokens, vec![Token::Delimiter('{'), Token::Delimiter('}')]);
    }

    #[test]
    fn test_tokenize_mixed_literal_and_number() {
        let tokens = Template::<'{', '}'>::tokenize("hello123");
        assert_eq!(
            tokens,
            vec![Token::Literal("hello".to_string()), Token::Int(123)]
        );
    }

    #[test]
    fn test_tokenize_number_and_literal() {
        let tokens = Template::<'{', '}'>::tokenize("123hello");
        assert_eq!(
            tokens,
            vec![Token::Int(123), Token::Literal("hello".to_string())]
        );
    }

    #[test]
    fn test_tokenize_literal_with_underscore() {
        let tokens = Template::<'{', '}'>::tokenize("hello_world");
        assert_eq!(tokens, vec![Token::Literal("hello_world".to_string())]);
    }

    #[test]
    fn test_tokenize_literal_with_whitespace() {
        let tokens = Template::<'{', '}'>::tokenize("hello world");
        assert_eq!(tokens, vec![Token::Literal("hello world".to_string())]);
    }

    #[test]
    fn test_tokenize_multiple_symbols() {
        let tokens = Template::<'{', '}'>::tokenize(":+=-");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol(':'),
                Token::Symbol('+'),
                Token::Symbol('='),
                Token::Symbol('-')
            ]
        );
    }

    #[test]
    fn test_tokenize_complex_expression() {
        let tokens = Template::<'{', '}'>::tokenize("name:pad:10");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("name".to_string()),
                Token::Symbol(':'),
                Token::Literal("pad".to_string()),
                Token::Symbol(':'),
                Token::Int(10)
            ]
        );
    }

    #[test]
    fn test_tokenize_with_delimiters_mixed() {
        let tokens = Template::<'{', '}'>::tokenize("hello{world}123");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("hello".to_string()),
                Token::Delimiter('{'),
                Token::Literal("world".to_string()),
                Token::Delimiter('}'),
                Token::Int(123)
            ]
        );
    }

    #[test]
    fn test_tokenize_multiple_numbers() {
        let tokens = Template::<'{', '}'>::tokenize("123 456 789");
        assert_eq!(
            tokens,
            vec![
                Token::Int(123),
                Token::Literal(" ".to_string()),
                Token::Int(456),
                Token::Literal(" ".to_string()),
                Token::Int(789)
            ]
        );
    }

    #[test]
    fn test_tokenize_negative_number_as_symbol_and_number() {
        // Note: negative numbers are tokenized as separate symbol and number
        let tokens = Template::<'{', '}'>::tokenize("-123");
        assert_eq!(tokens, vec![Token::Symbol('-'), Token::Int(123)]);
    }

    #[test]
    fn test_tokenize_large_number() {
        let tokens = Template::<'{', '}'>::tokenize("9223372036854775807");
        assert_eq!(tokens, vec![Token::Int(9223372036854775807i64)]);
    }

    #[test]
    fn test_tokenize_zero() {
        let tokens = Template::<'{', '}'>::tokenize("0");
        assert_eq!(tokens, vec![Token::Int(0)]);
    }

    #[test]
    fn test_tokenize_leading_zeros() {
        let tokens = Template::<'{', '}'>::tokenize("007");
        assert_eq!(tokens, vec![Token::Int(7)]);
    }

    #[test]
    fn test_tokenize_special_characters() {
        let tokens = Template::<'{', '}'>::tokenize("@#$%^&*()");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol('@'),
                Token::Symbol('#'),
                Token::Symbol('$'),
                Token::Symbol('%'),
                Token::Symbol('^'),
                Token::Symbol('&'),
                Token::Symbol('*'),
                Token::Symbol('('),
                Token::Symbol(')')
            ]
        );
    }

    #[test]
    fn test_tokenize_tabs_and_newlines() {
        let tokens = Template::<'{', '}'>::tokenize("hello\tworld\n");
        assert_eq!(tokens, vec![Token::Literal("hello\tworld\n".to_string())]);
    }

    #[test]
    fn test_tokenize_mixed_complex() {
        let tokens = Template::<'{', '}'>::tokenize("user:format:left:20");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("user".to_string()),
                Token::Symbol(':'),
                Token::Literal("format".to_string()),
                Token::Symbol(':'),
                Token::Literal("left".to_string()),
                Token::Symbol(':'),
                Token::Int(20)
            ]
        );
    }

    #[test]
    fn test_tokenize_function_call_style() {
        let tokens = Template::<'{', '}'>::tokenize("func(arg1, 42)");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("func".to_string()),
                Token::Symbol('('),
                Token::Literal("arg".to_string()),
                Token::Int(1),
                Token::Symbol(','),
                Token::Literal(" ".to_string()),
                Token::Int(42),
                Token::Symbol(')')
            ]
        );
    }

    #[test]
    fn test_tokenize_with_custom_delimiters() {
        let tokens = Template::<'[', ']'>::tokenize("name[value]123");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("name".to_string()),
                Token::Delimiter('['),
                Token::Literal("value".to_string()),
                Token::Delimiter(']'),
                Token::Int(123)
            ]
        );
    }

    #[test]
    fn test_tokenize_only_whitespace() {
        let tokens = Template::<'{', '}'>::tokenize("   \t\n  ");
        assert_eq!(tokens, vec![Token::Literal("   \t\n  ".to_string())]);
    }

    #[test]
    fn test_tokenize_alternating_types() {
        let tokens = Template::<'{', '}'>::tokenize("a1b2c3");
        assert_eq!(
            tokens,
            vec![
                Token::Literal("a".to_string()),
                Token::Int(1),
                Token::Literal("b".to_string()),
                Token::Int(2),
                Token::Literal("c".to_string()),
                Token::Int(3)
            ]
        );
    }
}

#[cfg(test)]
mod parsing {
    use super::*;
    use std::fmt::Debug;

    // Mock directive for testing
    #[derive(Debug, Clone)]
    struct MockDirective {
        content: String,
    }

    impl Directive for MockDirective {
        fn execute(&self, _ctx: &Context) -> Result<String, TemplateError> {
            Ok(format!("DIRECTIVE[{}]", self.content))
        }
    }

    // Mock parser that creates simple directives
    struct MockParser;

    impl Parser for MockParser {
        fn parse(_tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
            Some(Box::new(MockDirective {
                content: content.to_owned(),
            }))
        }
    }

    // Test helper to create a context
    fn create_test_context() -> Context {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String("John".to_string()));
        ctx.insert("age", Value::Int(30));
        ctx.insert("active", Value::Bool(true));
        ctx
    }

    #[test]
    fn test_parse_empty_template() {
        let template = Template::<'{', '}'>::parse::<MockParser>("").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_literal_only() {
        let template = Template::<'{', '}'>::parse::<MockParser>("Hello World").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_parse_single_directive() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_literal_with_directive() {
        let template = Template::<'{', '}'>::parse::<MockParser>("Hello {name}!").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name]!");
    }

    #[test]
    fn test_parse_multiple_directives() {
        let template =
            Template::<'{', '}'>::parse::<MockParser>("{name} is {age} years old").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name] is DIRECTIVE[age] years old");
    }

    #[test]
    fn test_parse_consecutive_directives() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name}{age}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]DIRECTIVE[age]");
    }

    #[test]
    fn test_parse_escaped_opening_delimiter() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{{not a directive}}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "{not a directive}");
    }

    #[test]
    fn test_parse_escaped_closing_delimiter() {
        let template = Template::<'{', '}'>::parse::<MockParser>("This is }} not escaped").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "This is } not escaped");
    }

    #[test]
    fn test_parse_mixed_escaped_and_directive() {
        let template =
            Template::<'{', '}'>::parse::<MockParser>("{{escaped}} {name} }}also escaped{{")
                .unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "{escaped} DIRECTIVE[name] }also escaped{");
    }

    #[test]
    fn test_parse_directive_with_alignment_left() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name<}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_directive_with_alignment_right() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name>}").unwrap();
        assert_eq!(template.alignment(), Alignment::Right);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_directive_with_alignment_center() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name^}").unwrap();
        assert_eq!(template.alignment(), Alignment::Center);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_default_alignment() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left);
    }

    #[test]
    fn test_parse_complex_directive_content() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{user:format:pad:20}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[user:format:pad:20]");
    }

    #[test]
    fn test_parse_directive_with_numbers() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{value:123:pad}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[value:123:pad]");
    }

    #[test]
    fn test_parse_custom_delimiters() {
        let template = Template::<'[', ']'>::parse::<MockParser>("Hello [name]!").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name]!");
    }

    #[test]
    fn test_parse_custom_delimiters_with_escaping() {
        let template =
            Template::<'[', ']'>::parse::<MockParser>("[[escaped]] [name] ]]also]]").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "[escaped] DIRECTIVE[name] ]also]");
    }

    #[test]
    fn test_parse_same_delimiters() {
        let template = Template::<'|', '|'>::parse::<MockParser>("Hello |name| World").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name] World");
    }

    // Error cases
    #[test]
    fn test_parse_missing_closing_delimiter() {
        let result = Template::<'{', '}'>::parse::<MockParser>("Hello {name");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_missing_opening_delimiter() {
        let result = Template::<'{', '}'>::parse::<MockParser>("Hello name}");
        assert!(matches!(
            result,
            Err(TemplateError::MissingOpenDelimiter('{'))
        ));
    }

    #[test]
    fn test_parse_nested_delimiters() {
        let result = Template::<'{', '}'>::parse::<MockParser>("Hello {name {age}}");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_unbalanced_delimiters_complex() {
        let result = Template::<'{', '}'>::parse::<MockParser>("{name} {age");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_extra_closing_delimiter() {
        let result = Template::<'{', '}'>::parse::<MockParser>("Hello {name}} extra");

        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    // Parser that fails to parse certain tokens
    struct FailingParser;

    impl Parser for FailingParser {
        fn parse(_tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
            if content.contains("fail") {
                None
            } else {
                Some(Box::new(MockDirective {
                    content: content.to_owned(),
                }))
            }
        }
    }

    #[test]
    fn test_parse_directive_parsing_failure() {
        let result = Template::<'{', '}'>::parse::<FailingParser>("Hello {fail}");
        assert!(matches!(result, Err(TemplateError::DirectiveParsing(_))));
    }

    #[test]
    fn test_parse_whitespace_in_directive() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{  name with spaces  }").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[  name with spaces  ]");
    }

    #[test]
    fn test_parse_empty_directive() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[]");
    }

    #[test]
    fn test_parse_directive_with_special_chars() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{name@#$%^&*()}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name@#$%^&*()]");
    }

    #[test]
    fn test_parse_multiple_alignment_characters() {
        // Only the last character should be treated as alignment
        let template = Template::<'{', '}'>::parse::<MockParser>("{name<>^}").unwrap();
        assert_eq!(template.alignment(), Alignment::Center);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name<>]");
    }

    #[test]
    fn test_parse_alignment_in_middle_not_treated_as_alignment() {
        let template = Template::<'{', '}'>::parse::<MockParser>("{na<me}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left); // default
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[na<me]");
    }

    #[test]
    fn test_parse_unicode_content() {
        let template = Template::<'{', '}'>::parse::<MockParser>("Hello {名前} 🌟").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[名前] 🌟");
    }

    #[test]
    fn test_parse_long_template() {
        let long_template = "Start ".repeat(100) + "{name}" + &" End".repeat(100);
        let template = Template::<'{', '}'>::parse::<MockParser>(&long_template).unwrap();
        let result = template.format(&create_test_context()).unwrap();
        let expected = "Start ".repeat(100) + "DIRECTIVE[name]" + &" End".repeat(100);
        assert_eq!(result, expected);
    }
}
