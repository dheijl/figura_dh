use std::fmt;

#[derive(Debug, Clone)]
pub enum TemplateError {
    /// Occurs when the template string is missing an opening or closing delimiter.
    ///
    /// # Example: "Hello }world" -> MissingDelimiter('{')
    MissingDelimiter(char),

    /// Occurs when the parser is unable to handle a given token pattern.
    ///
    /// # NOTE:
    /// This error **never** occurs when using [`DefaultParser`]
    /// And shall be used when implementing a custom parser.
    DirectiveParsing(String),

    /// Represents a generic failure during directive execution.
    ///
    /// That could mean that the [`RepeatDirective`] was expected a number but didn't find one,
    /// or when a key is used but not found in the context.
    DirectiveExecution(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingDelimiter(c) => write!(f, "Missing delimiter '{}'", c),
            Self::DirectiveParsing(msg) => write!(f, "Error parsing directive: {}", msg),
            Self::DirectiveExecution(msg) => write!(f, "Error executing directive: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}
