#![warn(clippy::use_self)]

mod directive;
mod err;
mod lexer;
mod parser;

use std::{collections::HashMap, fmt};

// Re-exports
pub use crate::directive::*;
pub use crate::err::TemplateError;
pub use crate::lexer::{Lexer, Token};
pub use crate::parser::*;

// A Value type used in templating contexts.
#[derive(Debug)]
pub enum Value {
    /// Heap-allocated string.
    String(String),

    /// Static string slice.
    ///
    /// E.G "Hello, world!" typed directly into the template.
    Str(&'static str),

    /// 64-bit integer.
    Int(i64),

    /// 64-bit floating point number.
    Float(f64),

    /// Boolean value.
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
        }
    }
}

pub type Context = HashMap<&'static str, Value>;

/// Represents a part of a template,
/// which can be either the text outside directives, and the code inside them.
///
/// # Example
/// "Hello, I am {name} and I am {age}" -> [Text("Hello, I am "), Directive(Directive::new("name")), Text(" and I am "), Directive(Directive::new("age"))]
pub enum Part {
    Text(String),
    Directive(Box<dyn Directive>),
}

pub struct Template<const O: char = '{', const C: char = '}'> {
    pub parts: Vec<Part>,
}

impl<const O: char, const C: char> Template<O, C> {
    pub fn validate(input: &str) -> isize {
        let mut chars = input.chars().peekable();

        if O == C {
            // Same delimiter: toggle between inside/outside
            let mut inside = false;

            while let Some(ch) = chars.next() {
                match ch {
                    '\\' => {
                        chars.next(); // Skip escaped character
                    }
                    c if c == O => {
                        inside = !inside;
                    }
                    _ => {}
                }
            }

            // If still inside, we have an unmatched delimiter
            if inside { 1 } else { 0 }
        } else {
            // Different delimiters: use depth tracking
            let mut depth: isize = 0;

            while let Some(ch) = chars.next() {
                match ch {
                    '\\' => {
                        chars.next();
                    }
                    c if c == O => depth += 1,
                    c if c == C => depth -= 1,
                    _ => {}
                }

                if depth < 0 {
                    return depth;
                }
            }

            depth
        }
    }

    #[inline]
    pub fn parse<T: AsRef<str>>(input: T) -> Result<Self, TemplateError> {
        Self::with_parser::<DefaultParser>(input.as_ref())
    }

    pub fn with_parser<P: Parser>(input: &str) -> Result<Self, TemplateError> {
        let input = input.as_ref();
        let depth = Self::validate(input);

        match depth {
            d if d > 0 => return Err(TemplateError::MissingDelimiter(C)),
            d if d < 0 => return Err(TemplateError::MissingDelimiter(O)),
            _ => {}
        };

        let mut chars = input.chars().peekable();
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut directive_content = String::new();
        let mut depth = 0isize;

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next) = chars.next() {
                        if depth == 0 {
                            text.push(next);
                        } else {
                            directive_content.push(next);
                        }
                    }
                }

                c if c == O => {
                    if O == C {
                        // Same delimiter: toggle
                        if depth == 0 {
                            if !text.is_empty() {
                                parts.push(Part::Text(std::mem::take(&mut text)));
                            }

                            depth = 1;
                        } else {
                            let tokens = Lexer::tokenize(&directive_content);
                            let dir = P::parse(&tokens)?;

                            parts.push(Part::Directive(dir));
                            directive_content.clear();
                            depth = 0;
                        }
                    } else {
                        // Different delimiters: nest
                        if depth == 0 {
                            if !text.is_empty() {
                                parts.push(Part::Text(std::mem::take(&mut text)));
                            }
                        } else {
                            directive_content.push(c);
                        }
                        depth += 1;
                    }
                }

                c if c == C => {
                    // Only reached when O != C
                    depth -= 1;

                    if depth == 0 {
                        let tokens = Lexer::tokenize(&directive_content);
                        let dir = P::parse(&tokens)?;

                        parts.push(Part::Directive(dir));
                        directive_content.clear();
                    } else {
                        directive_content.push(c);
                    }
                }

                c => {
                    if depth == 0 {
                        text.push(c);
                    } else {
                        directive_content.push(c);
                    }
                }
            }
        }

        if !text.is_empty() {
            parts.push(Part::Text(text));
        }

        Ok(Self { parts })
    }

    #[inline]
    pub fn format(&mut self, ctx: &HashMap<&'static str, Value>) -> Result<String, TemplateError> {
        let mut output = String::new();

        for part in std::mem::take(&mut self.parts) {
            match part {
                Part::Text(str) => output.push_str(&str),
                Part::Directive(dir) => {
                    let v = dir.execute(ctx)?;
                    output.push_str(&v);
                }
            }
        }

        Ok(output)
    }
}

// lib.rs tests (add to existing validate_tests or create new module)
#[cfg(test)]
mod parse_tests {
    use super::*;

    type Tpl = Template<'{', '}'>;

    // ==================== Basic Parsing Tests ====================

    #[test]
    fn test_parse_empty_string() {
        let tpl = Tpl::parse("").unwrap();
        assert_eq!(tpl.parts.len(), 0);
    }

    #[test]
    fn test_parse_only_text() {
        let tpl = Tpl::parse("hello world").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "hello world"));
    }

    #[test]
    fn test_parse_single_directive() {
        let tpl = Tpl::parse("{name}").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Directive(_)));
    }

    #[test]
    fn test_parse_text_before_directive() {
        let tpl = Tpl::parse("Hello, {name}").unwrap();
        assert_eq!(tpl.parts.len(), 2);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "Hello, "));
        assert!(matches!(&tpl.parts[1], Part::Directive(_)));
    }

    #[test]
    fn test_parse_text_after_directive() {
        let tpl = Tpl::parse("{name}!").unwrap();
        assert_eq!(tpl.parts.len(), 2);
        assert!(matches!(&tpl.parts[0], Part::Directive(_)));
        assert!(matches!(&tpl.parts[1], Part::Text(s) if s == "!"));
    }

    #[test]
    fn test_parse_text_around_directive() {
        let tpl = Tpl::parse("Hello, {name}!").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "Hello, "));
        assert!(matches!(&tpl.parts[1], Part::Directive(_)));
        assert!(matches!(&tpl.parts[2], Part::Text(s) if s == "!"));
    }

    #[test]
    fn test_parse_multiple_directives() {
        let tpl = Tpl::parse("{first} {second}").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Directive(_)));
        assert!(matches!(&tpl.parts[1], Part::Text(s) if s == " "));
        assert!(matches!(&tpl.parts[2], Part::Directive(_)));
    }

    #[test]
    fn test_parse_adjacent_directives() {
        let tpl = Tpl::parse("{a}{b}{c}").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Directive(_)));
        assert!(matches!(&tpl.parts[1], Part::Directive(_)));
        assert!(matches!(&tpl.parts[2], Part::Directive(_)));
    }

    #[test]
    fn test_parse_complex_template() {
        let tpl = Tpl::parse("Dear {title} {name}, your order #{order_id} is ready.").unwrap();
        assert_eq!(tpl.parts.len(), 7);
    }

    // ==================== Escape Sequence Tests ====================

    #[test]
    fn test_parse_escaped_opening_in_text() {
        let tpl = Tpl::parse("use \\{ for braces").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "use { for braces"));
    }

    #[test]
    fn test_parse_escaped_closing_in_text() {
        let tpl = Tpl::parse("use \\} for braces").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "use } for braces"));
    }

    #[test]
    fn test_parse_escaped_backslash() {
        let tpl = Tpl::parse("path\\\\to\\\\file").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "path\\to\\file"));
    }

    #[test]
    fn test_parse_escaped_in_directive() {
        // {name\}} - the backslash inside directive creates: name, \, }
        // After the first }, depth becomes 0, leaving "}" as text
        // This creates a parsing scenario the parser doesn't handle
        let result = Tpl::parse("{name\\}}");
        // The escaped } inside directive is passed to lexer as "name}"
        // which creates tokens [Ident("name"), RCurly] - unhandled pattern
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_escaped_regular_char() {
        let tpl = Tpl::parse("\\n is newline").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "n is newline"));
    }

    #[test]
    fn test_parse_trailing_backslash() {
        let tpl = Tpl::parse("trailing\\").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "trailing"));
    }

    #[test]
    fn test_parse_mixed_escapes_and_directives() {
        let tpl = Tpl::parse("\\{not a directive\\} but {this} is").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "{not a directive} but "));
        assert!(matches!(&tpl.parts[1], Part::Directive(_)));
        assert!(matches!(&tpl.parts[2], Part::Text(s) if s == " is"));
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_parse_unmatched_opening() {
        let result = Tpl::parse("{unclosed");
        // Use the guard syntax:
        assert!(matches!(result, Err(TemplateError::MissingDelimiter(c)) if c == '}'));
    }

    #[test]
    fn test_parse_unmatched_closing() {
        let result = Tpl::parse("extra}");
        assert!(matches!(result, Err(TemplateError::MissingDelimiter(c)) if c == '{'));
    }

    // ==================== Custom Delimiter Tests ====================

    #[test]
    fn test_custom_delimiters_angle_brackets() {
        type AngleTpl = Template<'<', '>'>;
        let tpl = AngleTpl::parse("<name>").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Directive(_)));
    }

    #[test]
    fn test_custom_delimiters_square_brackets() {
        type SquareTpl = Template<'[', ']'>;
        let tpl = SquareTpl::parse("Hello [name]!").unwrap();
        assert_eq!(tpl.parts.len(), 3);
    }

    #[test]
    fn test_custom_delimiters_parens() {
        type ParenTpl = Template<'(', ')'>;
        let tpl = ParenTpl::parse("Value: (value)").unwrap();
        assert_eq!(tpl.parts.len(), 2);
    }

    #[test]
    fn test_custom_delimiters_dollar() {
        type DollarTpl = Template<'$', '$'>;
        let tpl = DollarTpl::parse("Hello $name$!").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "Hello "));
        assert!(matches!(&tpl.parts[1], Part::Directive(_)));
        assert!(matches!(&tpl.parts[2], Part::Text(s) if s == "!"));
    }

    #[test]
    fn test_custom_delimiters_preserve_default_braces() {
        type AngleTpl = Template<'<', '>'>;
        let tpl = AngleTpl::parse("{not a directive} but <this> is").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "{not a directive} but "));
    }

    // ==================== Validate Function Tests ====================

    #[test]
    fn test_validate_returns_zero_for_balanced() {
        assert_eq!(Tpl::validate("{}"), 0);
        assert_eq!(Tpl::validate("{{}}"), 0);
        assert_eq!(Tpl::validate("{}{}"), 0);
    }

    #[test]
    fn test_validate_returns_positive_for_unclosed() {
        assert!(Tpl::validate("{") > 0);
        assert!(Tpl::validate("{{") > 0);
        assert!(Tpl::validate("{{}") > 0);
    }

    #[test]
    fn test_validate_returns_negative_for_extra_closing() {
        assert!(Tpl::validate("}") < 0);
        assert!(Tpl::validate("}}") < 0);
        assert!(Tpl::validate("{}}") < 0);
    }

    #[test]
    fn test_validate_depth_value() {
        assert_eq!(Tpl::validate("{{{"), 3);
        assert_eq!(Tpl::validate("{{{{{}}}"), 2);
    }

    // ==================== Unicode and Special Characters ====================

    #[test]
    fn test_parse_unicode_in_text() {
        let tpl = Tpl::parse("Héllo Wörld 🌍").unwrap();
        assert_eq!(tpl.parts.len(), 1);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "Héllo Wörld 🌍"));
    }

    #[test]
    fn test_parse_unicode_around_directive() {
        let tpl = Tpl::parse("Привет, {name}!").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "Привет, "));
    }

    #[test]
    fn test_parse_emoji_in_text() {
        let tpl = Tpl::parse("Hello 👋 {name} 🎉").unwrap();
        assert_eq!(tpl.parts.len(), 3);
    }

    #[test]
    fn test_parse_multiline() {
        let tpl = Tpl::parse("line1\n{var}\nline3").unwrap();
        assert_eq!(tpl.parts.len(), 3);
        assert!(matches!(&tpl.parts[0], Part::Text(s) if s == "line1\n"));
        assert!(matches!(&tpl.parts[2], Part::Text(s) if s == "\nline3"));
    }

    #[test]
    fn test_parse_tabs() {
        let tpl = Tpl::parse("\t{var}\t").unwrap();
        assert_eq!(tpl.parts.len(), 3);
    }

    // ==================== Real-World Template Tests ====================

    #[test]
    fn test_parse_email_template() {
        let template = "Dear {title} {last_name},\n\n\
                        Thank you for your order #{order_id}.\n\
                        Your total is ${amount}.\n\n\
                        Best regards,\n\
                        {company_name}";
        let tpl = Tpl::parse(template).unwrap();
        // Count directives: title, last_name, order_id, amount, company_name = 5
        let directive_count = tpl
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Directive(_)))
            .count();
        assert_eq!(directive_count, 5);
    }

    #[test]
    fn test_parse_html_template() {
        let template = "<div class=\"greeting\">Hello, {name}!</div>";
        let tpl = Tpl::parse(template).unwrap();
        assert_eq!(tpl.parts.len(), 3);
    }

    #[test]
    fn test_parse_url_template() {
        let template = "https://api.example.com/users/{user_id}/posts/{post_id}";
        let tpl = Tpl::parse(template).unwrap();
        let directive_count = tpl
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Directive(_)))
            .count();
        assert_eq!(directive_count, 2);
    }

    #[test]
    fn test_parse_json_like() {
        // Note: This tests that regular JSON braces would cause issues with default delimiters
        type AngleTpl = Template<'<', '>'>;
        let template = "{\"name\": \"<name>\", \"age\": <age>}";
        let tpl = AngleTpl::parse(template).unwrap();
        let directive_count = tpl
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Directive(_)))
            .count();
        assert_eq!(directive_count, 2);
    }
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    type Tpl = Template<'{', '}'>;
    const O: char = '{';
    const C: char = '}';

    // ==================== Basic Matching Tests ====================

    #[test]
    fn test_empty_string() {
        assert_eq!(Tpl::validate(""), 0);
    }

    #[test]
    fn test_no_delimiters() {
        assert_eq!(Tpl::validate("hello world"), 0);
        assert_eq!(Tpl::validate("abc123"), 0);
    }

    #[test]
    fn test_single_pair() {
        assert_eq!(Tpl::validate(&format!("{}{}", O, C)), 0);
    }

    #[test]
    fn test_nested_pairs() {
        assert_eq!(Tpl::validate(&format!("{}{}{}{}", O, O, C, C)), 0);
    }

    #[test]
    fn test_multiple_sequential_pairs() {
        assert_eq!(Tpl::validate(&format!("{}{}{}{}", O, C, O, C)), 0);
    }

    #[test]
    fn test_deeply_nested() {
        assert_eq!(Tpl::validate(&format!("{}{}{}{}{}{}", O, O, O, C, C, C)), 0);
    }

    #[test]
    fn test_with_text_between() {
        assert_eq!(Tpl::validate(&format!("hello{}world{}", O, C)), 0);
        assert_eq!(Tpl::validate(&format!("{}hello{}world{}{}", O, O, C, C)), 0);
    }

    // ==================== Unmatched Delimiter Tests ====================

    #[test]
    fn test_unmatched_opening() {
        assert!(Tpl::validate(&format!("{}", O)) > 0);
        assert!(Tpl::validate(&format!("{}{}", O, O)) > 0);
    }

    #[test]
    fn test_unmatched_closing() {
        assert!(Tpl::validate(&format!("{}", C)) < 0);
        assert!(Tpl::validate(&format!("{}{}", C, C)) < 0);
    }

    #[test]
    fn test_closing_before_opening() {
        assert!(Tpl::validate(&format!("{}{}", C, O)) < 0);
    }

    #[test]
    fn test_mismatched_pairs() {
        assert!(Tpl::validate(&format!("{}{}{}", O, O, C)) > 0);
        assert!(Tpl::validate(&format!("{}{}{}", O, C, C)) < 0);
    }

    // ==================== Escape Sequence Tests ====================

    #[test]
    fn test_escaped_opening_delimiter() {
        assert_eq!(Tpl::validate(&format!("\\{}", O)), 0);
    }

    #[test]
    fn test_escaped_closing_delimiter() {
        assert_eq!(Tpl::validate(&format!("\\{}", C)), 0);
    }

    #[test]
    fn test_escaped_delimiter_doesnt_count() {
        assert_eq!(Tpl::validate(&format!("{}\\{}{}", O, C, C)), 0);
    }

    #[test]
    fn test_escaped_opening_in_balanced_expression() {
        assert_eq!(Tpl::validate(&format!("{}\\{}{}", O, O, C)), 0);
    }

    #[test]
    fn test_escaped_closing_in_balanced_expression() {
        assert_eq!(Tpl::validate(&format!("{}\\{}{}", O, C, C)), 0);
    }

    #[test]
    fn test_multiple_escaped_delimiters() {
        assert_eq!(Tpl::validate(&format!("\\{}\\{}", O, C)), 0);
    }

    #[test]
    fn test_escaped_backslash_then_delimiter() {
        // \\ followed by { means the backslash is escaped, { is NOT escaped
        assert!(Tpl::validate(&format!("\\\\{}", O)) > 0);
        assert_eq!(Tpl::validate(&format!("\\\\{}{}", O, C)), 0);
    }

    #[test]
    fn test_escaped_regular_character() {
        assert_eq!(Tpl::validate("\\a"), 0);
        assert_eq!(Tpl::validate("\\n"), 0);
    }

    #[test]
    fn test_trailing_backslash() {
        assert_eq!(Tpl::validate("\\"), 0);
        assert_eq!(Tpl::validate(&format!("{}{}\\", O, C)), 0);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_only_escaped_delimiters() {
        assert_eq!(Tpl::validate(&format!("\\{}\\{}\\{}\\{}", O, C, O, C)), 0);
    }

    #[test]
    fn test_very_deeply_nested() {
        let deep = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            O, O, O, O, O, O, O, O, O, O, C, C, C, C, C, C, C, C, C, C
        );
        assert_eq!(Tpl::validate(&deep), 0);
    }

    #[test]
    fn test_alternating_escaped_and_real() {
        // \{ { \} } = escaped-open, real-open, escaped-close, real-close = balanced
        assert_eq!(Tpl::validate(&format!("\\{}{}\\{}{}", O, O, C, C)), 0);
    }

    #[test]
    fn test_depth_tracking() {
        assert_eq!(Tpl::validate("{{{"), 3);
        assert_eq!(Tpl::validate("{{{{"), 4);
        assert_eq!(Tpl::validate("{{}{{"), 3);
    }
}

#[cfg(test)]
mod value_tests {
    use crate::Value;

    #[test]
    fn test_value_display_string() {
        let v = Value::String("hello".to_string());
        assert_eq!(format!("{}", v), "hello");
    }

    #[test]
    fn test_value_display_str() {
        let v = Value::Str("world");
        assert_eq!(format!("{}", v), "world");
    }

    #[test]
    fn test_value_display_int() {
        let v = Value::Int(42);
        assert_eq!(format!("{}", v), "42");
    }

    #[test]
    fn test_value_display_negative_int() {
        let v = Value::Int(-100);
        assert_eq!(format!("{}", v), "-100");
    }

    #[test]
    fn test_value_display_float() {
        let v = Value::Float(3.14);
        assert_eq!(format!("{}", v), "3.14");
    }

    #[test]
    fn test_value_display_bool_true() {
        let v = Value::Bool(true);
        assert_eq!(format!("{}", v), "true");
    }

    #[test]
    fn test_value_display_bool_false() {
        let v = Value::Bool(false);
        assert_eq!(format!("{}", v), "false");
    }

    #[test]
    fn test_value_debug() {
        let v = Value::String("test".to_string());
        let debug = format!("{:?}", v);
        assert!(debug.contains("String"));
        assert!(debug.contains("test"));
    }
}
