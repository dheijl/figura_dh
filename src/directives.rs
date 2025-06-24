use smacro::s;

use crate::{Context, TemplateError, Token, Value};
use std::fmt::Debug;

/// A trait representing an action to be executed when a directive is found
/// within a template.
///
/// Directives are units of logic that consume a section of a template and
/// produce a string based on the provided [`Context`]. They may replace content,
/// repeat content, evaluate conditions, etc.
///
/// # Example
///
/// ```no_run
/// struct MyDirective;
///
/// impl Directive for MyDirective {
///     fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
///         Ok("Hello from directive!".to_string())
///     }
/// }
/// ```
///
/// # Errors
/// Implementations of this trait may return a [`TemplateError`] if evaluation fails,
/// such as when a required variable is missing from the context.
pub trait Directive: Debug {
    /// Executes the directive using the provided context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The template rendering context providing variable values.
    ///
    /// # Returns
    ///
    /// A `Result` containing the rendered string, or a `TemplateError` if evaluation fails.
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError>;
}

/// A trait for parsing a list of [`Token`]s into a [`Directive`] that can later be executed.
///
/// Parsers determine how specific token patterns should be interpreted and
/// mapped to executable directives.
///
/// # Example
///
/// ```no_run
/// struct MyParser;
///
/// impl Parser for MyParser {
///     fn parse(tokens: &[Token]) -> Option<Box<dyn Directive>> {
///         if tokens == [Token::Literal("hello".into())] {
///             Some(Box::new(NoDirective("hello".into())))
///         } else {
///             None
///         }
///     }
/// }
/// ```
///
/// # Note
/// A parser may choose to return `None` if it cannot recognize the token sequence,
/// unless it’s a default parser (e.g., [`DefaultParser`]) that always provides a fallback.
pub trait Parser {
    /// Attempts to parse the provided tokens into a `Directive`.
    ///
    /// # Arguments
    ///
    /// * `tokens` - A slice of `Token`s representing a segment of the template.
    /// * `content` - The original string that has been tokenized
    ///
    /// # Returns
    ///
    /// An `Option` containing a boxed `Directive` if parsing was successful,
    /// or `None` if the tokens do not match any known directive pattern.
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn Directive>>;
}

/// A fallback directive that performs no substitution or transformation,
/// simply returning the original content.
///
/// This directive is useful when no specific transformation is required,
/// or as a default when a parser cannot recognize a pattern.
///
/// # Example
///
/// ```no_run
/// let directive = NoDirective("unchanged".into());
/// assert_eq!(directive.execute(&Context::new()).unwrap(), "unchanged");
/// ```
#[derive(Debug)]
pub struct NoDirective(String);

impl Directive for NoDirective {
    /// Returns the original content unchanged.
    fn execute(&self, _: &Context) -> Result<String, TemplateError> {
        Ok(s!(self.0))
    }
}

/// A directive that replaces a string literal with a value from the context.
///
/// Typically used when a directive like `{name}` appears in a template,
/// and "name" is a key in the context.
///
/// # Errors
/// Returns [`TemplateError::NoValueFound`] if the key is not present in the context.
///
/// # Example
///
/// ```no_run
/// let mut ctx = Context::new();
/// ctx.insert("name", Value::String("Alice".into()));
/// let directive = ReplaceDirective("name".into());
/// assert_eq!(directive.execute(&ctx).unwrap(), "Alice");
/// ```
#[derive(Debug)]
pub struct ReplaceDirective(String);

impl Directive for ReplaceDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        if let Some(v) = ctx.get(self.0.as_str()) {
            Ok(s!(v))
        } else {
            Err(TemplateError::NoValueFound(self.0.clone()))
        }
    }
}

/// A directive that repeats a pattern a specified number of times.
///
/// Supports both literals and context-based values for the pattern and count.
/// For example, `{hello:3}` will yield `"hellohellohello"`.
///
/// # Behavior
/// - If `pattern` or `count` are not found in context, they are treated as literals.
/// - `count` must be a positive integer, either directly or from context.
///
/// # Errors
/// Returns [`TemplateError::ExecutionError`] if:
/// - `count` is non-numeric or negative
/// - A context variable exists but is of a non-integer type
///
/// # Example
///
/// ```no_run
/// let mut ctx = Context::new();
/// ctx.insert("word", Value::String("hi".into()));
/// ctx.insert("times", Value::Int(3));
///
/// let directive = RepeatDirective("word".into(), "times".into());
/// assert_eq!(directive.execute(&ctx).unwrap(), "hihihi");
/// ```
#[derive(Debug)]
pub struct RepeatDirective(String, String);

impl Directive for RepeatDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        // Check if the literal is a context value
        // If not, use it directly
        let pattern = match ctx.get(self.0.as_str()) {
            Some(p) => s!(p),
            None => s!(self.0),
        };

        // Check if count is a context value
        // If not, check if it can be parsed into a usize,
        // If not return an error
        let count = match ctx.get(self.1.as_str()) {
            Some(c) => match c {
                Value::Int(i) if *i > 0 => *i as usize,
                _ => {
                    return Err(TemplateError::ExecutionError(
                        "Could not parse a numeric value for the repeat count".to_string(),
                    ));
                }
            },
            None => self.1.parse::<usize>().map_err(|_| {
                TemplateError::ExecutionError(
                    "Could not parse a numeric value for the repeat count".to_string(),
                )
            })?,
        };

        Ok(pattern.repeat(count))
    }
}

/// A directive that conditionally returns one of two strings based on a context value.
///
/// The conditional directive evaluates a condition variable from the context and returns
/// different content based on whether the condition is truthy or falsy. The syntax is
/// `{condition?true_part:false_part}`.
///
/// # Condition Evaluation
/// - If the condition variable exists in context and is a [`Value::Bool`], its boolean value is used
/// - If the condition variable exists but is not a boolean, it is treated as `true` (truthy)
/// - If the condition variable does not exist in context, it is treated as `false` (falsy)
///
/// This behavior mirrors common programming language patterns where variables can be
/// evaluated in a boolean context.
///
/// # Example
///
/// ```no_run
/// let mut ctx = Context::new();
/// ctx.insert("is_admin", Value::Bool(true));
/// ctx.insert("username", Value::String("Alice".into()));
///
/// // Boolean condition
/// let directive = ConditionalDirective {
///     condition: "is_admin".into(),
///     parts: ("Admin Panel".into(), "User Panel".into()),
/// };
/// assert_eq!(directive.execute(&ctx).unwrap(), "Admin Panel");
///
/// // Non-boolean but existing condition (truthy)
/// let directive2 = ConditionalDirective {
///     condition: "username".into(),
///     parts: ("Logged In".into(), "Guest".into()),
/// };
/// assert_eq!(directive2.execute(&ctx).unwrap(), "Logged In");
///
/// // Non-existing condition (falsy)
/// let directive3 = ConditionalDirective {
///     condition: "missing_var".into(),
///     parts: ("Found".into(), "Not Found".into()),
/// };
/// assert_eq!(directive3.execute(&ctx).unwrap(), "Not Found");
/// ```
#[derive(Debug)]
pub struct ConditionalDirective {
    condition: String,
    parts: (String, String),
}

impl Directive for ConditionalDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        // Check if the condition is a context value
        // If it exists and is not a boolean, treat it as true
        // If it exists and is a boolean, use its value
        // If it doesn't exist, treat it as false
        let condition = match ctx.get(self.condition.as_str()) {
            Some(v) => match v {
                // Use the value if its an actual boolean
                Value::Bool(b) => *b,
                // It exists, so true
                // Similar to what happens in most programming languages
                // Where you can check if a variable exists by doing `if var {}`
                _ => true,
            },
            // If it doesn't exist, return false
            None => false,
        };

        if condition {
            Ok(s!(self.parts.0))
        } else {
            Ok(s!(self.parts.1))
        }
    }
}

/// The default parser used to convert template tokens into executable directives.
///
/// It supports three kinds of directives:
/// - Replacement: `{variable}` → [`ReplaceDirective`]
/// - Repetition: `{pattern:count}` → [`RepeatDirective`]
/// - Fallback: any other input → [`NoDirective`]
///
/// # Example
///
/// ```no_run
/// let tokens = vec![Token::Literal("name".into())];
/// let directive = DefaultParser::parse(&tokens).unwrap();
/// ```
///
/// # Note
/// This parser **never returns `None`**, ensuring that all token sequences are turned into
/// a directive, even if it’s just [`NoDirective`].
///
/// To create a custom pasrser but still mantain the default behavior,
/// you can implement the [`Parser`] trait and call `DefaultParser::parse`
/// within your custom parser.
pub struct DefaultParser;

impl Parser for DefaultParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
        match tokens {
            // {variable}
            [Token::Slice(s)] => Some(Box::new(ReplaceDirective(s.clone()))),

            // {pattern:count}
            [fist_part, Token::Symbol(':'), second_part] => {
                Some(Box::new(RepeatDirective(s!(fist_part), s!(second_part))))
            }

            // {condition?part1:part2}
            [
                Token::Slice(condition),
                Token::Symbol('?'),
                Token::Slice(part1),
                Token::Symbol(':'),
                Token::Slice(part2),
            ] => Some(Box::new(ConditionalDirective {
                condition: s!(condition),
                parts: (s!(part1), s!(part2)),
            })),

            // Just return the original string
            _ => Some(Box::new(NoDirective(content.to_owned()))),
        }
    }
}

#[cfg(test)]
mod default_parser {
    use crate::{Template, Value};
    use smacro::map;

    #[test]
    fn test_replace_directive() {
        let template = "Hello, {name}!";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map! {
            "name" => Value::String("Alice".to_string()),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Hello, Alice!");

        let template =
            "There was a cat named {cat_name}, who was {age} years old. Its owner was {owner}.";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map! {
            "cat_name" => Value::String("Whiskers".to_string()),
            "age" => Value::Int(5),
            "owner" => Value::String("Bob".to_string()),
        };

        assert_eq!(
            template.format(&ctx).unwrap(),
            "There was a cat named Whiskers, who was 5 years old. Its owner was Bob."
        );
    }

    #[test]
    fn test_repeat_directive() {
        let template = "Repeat: {word:3}";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map! {
            "word" => Value::String("hi".to_string()),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Repeat: hihihi");

        // Test with a variable count
        let template = "Repeat: {word:count}";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map! {
            "word" => Value::String("hi".to_string()),
            "count" => Value::Int(3),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Repeat: hihihi");

        // Test with a non-integer count
        let template = "Repeat: {word:-1}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        assert!(template.format(&ctx).is_err());

        // Test with a literal pattern and count
        let template = "Repeat: {hello:2}";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map![];

        assert_eq!(template.format(&ctx).unwrap(), "Repeat: hellohello");
    }

    #[test]
    fn test_conditional_directive() {
        let template = "{is_admin?Admin Panel:User Panel}";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        let ctx = map! {
            "is_admin" => Value::Bool(true),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Admin Panel");

        let ctx = map! {
            "is_admin" => Value::Bool(false),
        };
        assert_eq!(template.format(&ctx).unwrap(), "User Panel");

        let ctx = map! {
            "username" => Value::String("Alice".to_string()),
        };

        let template = "{username?Logged In:Guest}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        assert_eq!(template.format(&ctx).unwrap(), "Logged In");
    }
}
