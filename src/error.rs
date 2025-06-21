use thiserror::Error;

/// Represents all possible errors that can occur when parsing or executing a template.
///
/// These errors cover mismatched delimiters, missing values in the context,
/// unrecognized directives, and runtime issues during directive execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemplateError {
    /// Occurs when the template is missing an opening delimiter.
    ///
    /// This typically happens during validation when the internal nesting counter
    /// becomes negative. For example, a closing delimiter appears before an opening one.
    ///
    /// # Example
    /// Template: `Hello }world{` → MissingOpenDelimiter('}')
    #[error("There's a missing '{0}' in your template.")]
    MissingOpenDelimiter(char),

    /// Occurs when the template is missing a closing delimiter.
    ///
    /// This typically happens when an opening delimiter is never matched with a close.
    ///
    /// # Example
    /// Template: `Hello {world` → MissingClosedDelimiter('}')
    #[error("There's a missing '{0}' in your template.")]
    MissingClosedDelimiter(char),

    /// Used when delimiters are symmetrical (e.g., `/`) and the parser cannot determine
    /// whether the open or close is missing.
    ///
    /// This is useful for ambiguous cases where the same symbol is used to start and end blocks.
    #[error("There's a missing '{0}' in your template.")]
    MissingDelimiter(char),

    /// Raised during execution when a variable is referenced in the template
    /// but not found in the provided context.
    ///
    /// This error prevents undefined substitutions from silently failing.
    ///
    /// # Example
    /// Template: `Hello {name}` → context does not contain "name"
    #[error("Trying to use a value that doesn't exist: '{0}' doesn't point to any value")]
    NoValueFound(String),

    /// Indicates that no directive parser was able to handle a given token pattern.
    ///
    /// # Note
    /// This error is **never** emitted when using `DefaultParser`, since it always
    /// returns a [`NoDirective`] fallback instead.
    ///
    /// # Example
    /// Input: `{#unknown}` → no directive registered for `#unknown`
    #[error("Could not parse a directive for the literal '{0}'")]
    DirectiveParsing(String),

    /// Represents a generic failure that occurred while executing a directive.
    ///
    /// This is used when a directive encounters logic issues at runtime, such as
    /// an invalid repeat count or unexpected input types.
    ///
    /// # Example
    /// RepeatDirective receives a string where a number was expected.
    #[error("There was an error while exectuting a directive: {0}")]
    ExecutionError(String),
}
