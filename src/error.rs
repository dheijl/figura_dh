use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemplateError {
    #[error("There's a missing '{0}' in your template.")]
    MissingOpenDelimiter(char),

    #[error("There's a missing '{0}' in your template.")]
    MissingClosedDelimiter(char),

    /// This error is specifically used if the delimiters
    /// are equivalent, (i.e. '/'), so understanding which
    /// delimiter, open or closed, is missing might be tricky.
    #[error("There's a missing '{0}' in your template.")]
    MissingDelimiter(char),

    #[error("Trying to use a value that doesn't exist: '{0}' doesn't point to any value")]
    NoValueFound(String),

    #[error("You are trying to use a repeat operation with a non-uint count: '{0}'")]
    NonUIntForCountVariable(String),
}
