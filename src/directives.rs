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
        Ok(self.0.to_string())
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
            Ok(v.to_string())
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
            Some(p) => p.to_string(),
            None => self.0.to_string(),
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
pub struct DefaultParser;

impl Parser for DefaultParser {
    fn parse(tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
        match tokens {
            // {variable}
            [Token::Literal(s)] => Some(Box::new(ReplaceDirective(s.clone()))),

            // {pattern:count}
            [fist_part, Token::Symbol(':'), second_part] => Some(Box::new(RepeatDirective(
                fist_part.to_string(),
                second_part.to_string(),
            ))),

            // Just return the original string
            _ => Some(Box::new(NoDirective(content.to_owned()))),
        }
    }
}
