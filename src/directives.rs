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
                Value::Int(i) if *i >= 0 => *i as usize,
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

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub left: String,
    pub op: ComparisonOp,
    pub right: String,
}

/// A directive that conditionally returns one of two strings based on a context value.
///
/// The conditional directive supports both simple boolean conditions and comparison operations.
/// The syntax is `{condition?true_part:false_part}`.
///
/// # Supported Comparisons
/// - `{var==value?true:false}` - Equality comparison
/// - `{var!=value?true:false}` - Inequality comparison  
/// - `{var<value?true:false}` - Less than (numeric)
/// - `{var<=value?true:false}` - Less than or equal (numeric)
/// - `{var>value?true:false}` - Greater than (numeric)
/// - `{var>=value?true:false}` - Greater than or equal (numeric)
/// - `{var?true:false}` - Simple boolean/existence check
///
/// # Value Resolution
/// - Variable names are resolved from the context first
/// - If not found in context, treated as literal values
/// - Boolean values: "true", "false" (case-insensitive)
/// - Numeric values: integers and floats
/// - String values: everything else
///
/// # Example
///
/// ```no_run
/// let mut ctx = Context::new();
/// ctx.insert("age", Value::Int(25));
/// ctx.insert("name", Value::String("Alice".into()));
/// ctx.insert("is_admin", Value::Bool(true));
///
/// // Numeric comparison
/// let directive = ConditionalDirective::new("age>=18", "Adult", "Minor");
/// assert_eq!(directive.execute(&ctx).unwrap(), "Adult");
///
/// // String comparison
/// let directive = ConditionalDirective::new("name==Alice", "Welcome Alice", "Unknown user");
/// assert_eq!(directive.execute(&ctx).unwrap(), "Welcome Alice");
///
/// // Boolean check
/// let directive = ConditionalDirective::new("is_admin", "Admin Panel", "User Panel");
/// assert_eq!(directive.execute(&ctx).unwrap(), "Admin Panel");
/// ```
#[derive(Debug)]
pub struct ConditionalDirective {
    condition: Condition,
    parts: (String, String),
}

impl ConditionalDirective {
    pub fn new(condition_str: &str, true_part: &str, false_part: &str) -> Self {
        let condition = Self::parse_condition(condition_str).unwrap_or_else(|| Condition {
            left: condition_str.to_string(),
            op: ComparisonOp::Equal,
            right: "true".to_string(),
        });

        Self {
            condition,
            parts: (true_part.to_string(), false_part.to_string()),
        }
    }

    fn parse_condition(condition_str: &str) -> Option<Condition> {
        let condition_str = condition_str.trim();

        // Check for two-character operators first
        if let Some(pos) = condition_str.find("==") {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::Equal,
                right: right[2..].trim().to_string(),
            });
        }

        if let Some(pos) = condition_str.find("!=") {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::NotEqual,
                right: right[2..].trim().to_string(),
            });
        }

        if let Some(pos) = condition_str.find("<=") {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::LessThanOrEqual,
                right: right[2..].trim().to_string(),
            });
        }

        if let Some(pos) = condition_str.find(">=") {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::GreaterThanOrEqual,
                right: right[2..].trim().to_string(),
            });
        }

        // Check for single-character operators
        if let Some(pos) = condition_str.find('<') {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::LessThan,
                right: right[1..].trim().to_string(),
            });
        }

        if let Some(pos) = condition_str.find('>') {
            let (left, right) = condition_str.split_at(pos);
            return Some(Condition {
                left: left.trim().to_string(),
                op: ComparisonOp::GreaterThan,
                right: right[1..].trim().to_string(),
            });
        }

        // If no operator found, treat as simple boolean check (var == true)
        Some(Condition {
            left: condition_str.to_string(),
            op: ComparisonOp::Equal,
            right: "true".to_string(),
        })
    }

    fn resolve_value(&self, name: &str, ctx: &Context) -> String {
        // First check if it's a context variable
        if let Some(value) = ctx.get(name) {
            return s!(value);
        }

        // If not in context, use as literal
        name.to_string()
    }

    fn parse_as_bool(&self, value: &str) -> Option<bool> {
        match value.to_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    fn parse_as_number(&self, value: &str) -> Option<f64> {
        value.parse::<f64>().ok()
    }

    fn evaluate_condition(&self, ctx: &Context) -> bool {
        let left_val = self.resolve_value(&self.condition.left, ctx);
        let right_val = self.resolve_value(&self.condition.right, ctx);

        match self.condition.op {
            ComparisonOp::Equal => {
                // Try boolean comparison first
                if let (Some(l), Some(r)) = (
                    self.parse_as_bool(&left_val),
                    self.parse_as_bool(&right_val),
                ) {
                    return l == r;
                }

                // Try numeric comparison
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    return l == r;
                }

                // Fall back to string comparison
                left_val == right_val
            }

            ComparisonOp::NotEqual => {
                // Try boolean comparison first
                if let (Some(l), Some(r)) = (
                    self.parse_as_bool(&left_val),
                    self.parse_as_bool(&right_val),
                ) {
                    return l != r;
                }

                // Try numeric comparison
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    return l != r;
                }

                // Fall back to string comparison
                left_val != right_val
            }

            ComparisonOp::LessThan => {
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    l < r
                } else {
                    false
                }
            }

            ComparisonOp::LessThanOrEqual => {
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    l <= r
                } else {
                    false
                }
            }

            ComparisonOp::GreaterThan => {
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    l > r
                } else {
                    false
                }
            }

            ComparisonOp::GreaterThanOrEqual => {
                if let (Some(l), Some(r)) = (
                    self.parse_as_number(&left_val),
                    self.parse_as_number(&right_val),
                ) {
                    l >= r
                } else {
                    false
                }
            }
        }
    }
}

impl Directive for ConditionalDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        // Handle simple boolean check (backward compatibility)
        if self.condition.op == ComparisonOp::Equal && self.condition.right == "true" {
            // Check if the condition is a context value
            // If it exists and is not a boolean, treat it as true
            // If it exists and is a boolean, use its value
            // If it doesn't exist, treat it as false
            let condition_result = match ctx.get(self.condition.left.as_str()) {
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

            if condition_result {
                return Ok(s!(self.parts.0));
            } else {
                return Ok(s!(self.parts.1));
            }
        }

        // Handle comparison operations
        if self.evaluate_condition(ctx) {
            Ok(s!(self.parts.0))
        } else {
            Ok(s!(self.parts.1))
        }
    }
}

/// A directive that matches a variable against multiple cases and returns the corresponding value.
///
/// The switch directive works like a switch statement, checking a variable against multiple
/// patterns and returning the first matching result. The syntax is:
/// `{[variable](case1:result1)(case2:result2)(case3:result3)}`
///
/// # Behavior
/// - The variable is resolved from the context first, then treated as literal if not found
/// - Cases are checked in order until a match is found
/// - If no case matches, returns an empty string
/// - Both the variable and case values support context variable resolution
///
/// # Example
///
/// ```no_run
/// let mut ctx = Context::new();
/// ctx.insert("status", Value::String("active".into()));
///
/// // switch directive with literal cases
/// let directive = SwitchDirective::new("status", vec![
///     ("active".into(), "System Online".into()),
///     ("inactive".into(), "System Offline".into()),
///     ("maintenance".into(), "Under Maintenance".into()),
/// ]);
/// assert_eq!(directive.execute(&ctx).unwrap(), "System Online");
/// ```
#[derive(Debug)]
pub struct SwitchDirective {
    variable: String,
    cases: Vec<(String, String)>, // (pattern, result)
}

impl SwitchDirective {
    pub fn new(variable: String, cases: Vec<(String, String)>) -> Self {
        Self { variable, cases }
    }

    fn resolve_value(&self, name: &str, ctx: &Context) -> String {
        // First check if it's a context variable
        if let Some(value) = ctx.get(name) {
            return s!(value);
        }

        // If not in context, use as literal
        name.to_string()
    }
}

impl Directive for SwitchDirective {
    fn execute(&self, ctx: &Context) -> Result<String, TemplateError> {
        let variable_value = self.resolve_value(&self.variable, ctx);

        // Check each case in order
        for (pattern, result) in &self.cases {
            let pattern_value = self.resolve_value(pattern, ctx);
            if variable_value == pattern_value {
                return Ok(self.resolve_value(result, ctx));
            }
        }

        // No match found, return empty string
        Ok(String::new())
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
            [first_part, Token::Symbol(':'), second_part] => {
                // Check if this is actually a conditional with comparison operators
                if Self::contains_question_mark(tokens) {
                    return Self::parse_conditional(tokens);
                }
                Some(Box::new(RepeatDirective(s!(first_part), s!(second_part))))
            }

            // Simple conditional: {condition?part1:part2}
            [
                Token::Slice(condition),
                Token::Symbol('?'),
                Token::Slice(part1),
                Token::Symbol(':'),
                Token::Slice(part2),
            ] => {
                let conditional = ConditionalDirective::new(condition, part1, part2);
                Some(Box::new(conditional))
            }

            // Handle any other token pattern
            _ => {
                // Check if it's a bracket conditional first
                if Self::is_bracket_conditional(tokens) {
                    Self::parse_bracket_conditional(tokens)
                } else if Self::is_switch_directive(tokens) {
                    Self::parse_switch_directive(tokens)
                } else if Self::contains_question_mark(tokens) {
                    Self::parse_conditional(tokens)
                } else {
                    Some(Box::new(NoDirective(content.to_owned())))
                }
            }
        }
    }
}

impl DefaultParser {
    fn contains_question_mark(tokens: &[Token]) -> bool {
        tokens
            .iter()
            .any(|token| matches!(token, Token::Symbol('?')))
    }

    fn is_switch_directive(tokens: &[Token]) -> bool {
        // Check if the pattern starts with '[' and contains '(' and ')'
        if tokens.is_empty() {
            return false;
        }

        // Look for the pattern: [variable](case:result)(case:result)...
        let has_opening_bracket = tokens.iter().any(|t| matches!(t, Token::Symbol('[')));
        let has_closing_bracket = tokens.iter().any(|t| matches!(t, Token::Symbol(']')));
        let has_parentheses = tokens.iter().any(|t| matches!(t, Token::Symbol('(')))
            && tokens.iter().any(|t| matches!(t, Token::Symbol(')')));

        if !has_opening_bracket || !has_closing_bracket || !has_parentheses {
            return false;
        }

        // Count parentheses pairs to distinguish between switch and conditional directives
        let paren_pairs = Self::count_parentheses_pairs(tokens);

        // Switch directive: [var](case:result)(case:result)... (has colons in parentheses)
        // Conditional directive: [condition](true_case)(false_case) (no colons, exactly 2 pairs)
        if paren_pairs == 2 {
            // Check if parentheses contain colons (match) or not (conditional)
            return Self::parentheses_contain_colons(tokens);
        }

        // More than 2 pairs, likely a switch directive
        paren_pairs > 2
    }

    fn count_parentheses_pairs(tokens: &[Token]) -> usize {
        let mut count = 0;
        let mut i = 0;

        while i < tokens.len() {
            if matches!(tokens[i], Token::Symbol('(')) {
                count += 1;
                // Skip to the matching closing parenthesis
                let mut paren_count = 1;
                i += 1;
                while i < tokens.len() && paren_count > 0 {
                    match tokens[i] {
                        Token::Symbol('(') => paren_count += 1,
                        Token::Symbol(')') => paren_count -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        count
    }

    fn parentheses_contain_colons(tokens: &[Token]) -> bool {
        let mut i = 0;

        while i < tokens.len() {
            if matches!(tokens[i], Token::Symbol('(')) {
                let mut paren_count = 1;
                i += 1;

                while i < tokens.len() && paren_count > 0 {
                    match tokens[i] {
                        Token::Symbol('(') => paren_count += 1,
                        Token::Symbol(')') => paren_count -= 1,
                        Token::Symbol(':') if paren_count == 1 => return true,
                        _ => {}
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        false
    }

    fn is_bracket_conditional(tokens: &[Token]) -> bool {
        if tokens.is_empty() {
            return false;
        }

        let has_opening_bracket = tokens.iter().any(|t| matches!(t, Token::Symbol('[')));
        let has_closing_bracket = tokens.iter().any(|t| matches!(t, Token::Symbol(']')));
        let has_parentheses = tokens.iter().any(|t| matches!(t, Token::Symbol('(')))
            && tokens.iter().any(|t| matches!(t, Token::Symbol(')')));

        if !has_opening_bracket || !has_closing_bracket || !has_parentheses {
            return false;
        }

        let paren_pairs = Self::count_parentheses_pairs(tokens);

        // Conditional directive: exactly 2 parentheses pairs with no colons inside
        paren_pairs == 2 && !Self::parentheses_contain_colons(tokens)
    }

    fn parse_bracket_conditional(tokens: &[Token]) -> Option<Box<dyn Directive>> {
        // Find the opening and closing brackets for the condition
        let bracket_start = tokens
            .iter()
            .position(|t| matches!(t, Token::Symbol('[')))?;
        let bracket_end = tokens
            .iter()
            .position(|t| matches!(t, Token::Symbol(']')))?;

        if bracket_start >= bracket_end {
            return None;
        }

        // Extract the condition
        let condition_tokens = &tokens[bracket_start + 1..bracket_end];
        let condition_str = Self::tokens_to_string(condition_tokens);

        // Parse the two cases after the closing bracket
        let remaining_tokens = &tokens[bracket_end + 1..];
        let cases = Self::parse_conditional_cases(remaining_tokens)?;

        if cases.len() != 2 {
            return None;
        }

        let conditional = ConditionalDirective::new(&condition_str, &cases[0], &cases[1]);
        Some(Box::new(conditional))
    }

    fn parse_conditional_cases(tokens: &[Token]) -> Option<Vec<String>> {
        let mut cases = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Look for opening parenthesis
            if !matches!(tokens[i], Token::Symbol('(')) {
                i += 1;
                continue;
            }

            // Find the matching closing parenthesis
            let mut paren_count = 1;
            let mut j = i + 1;

            while j < tokens.len() && paren_count > 0 {
                match tokens[j] {
                    Token::Symbol('(') => paren_count += 1,
                    Token::Symbol(')') => paren_count -= 1,
                    _ => {}
                }
                j += 1;
            }

            if paren_count != 0 {
                // Unmatched parentheses
                return None;
            }

            // Extract tokens between parentheses
            let case_tokens = &tokens[i + 1..j - 1];
            let case_str = Self::tokens_to_string(case_tokens);

            cases.push(case_str);

            i = j;
        }

        if cases.is_empty() { None } else { Some(cases) }
    }

    fn parse_switch_directive(tokens: &[Token]) -> Option<Box<dyn Directive>> {
        // Find the opening and closing brackets for the variable
        let bracket_start = tokens
            .iter()
            .position(|t| matches!(t, Token::Symbol('[')))?;
        let bracket_end = tokens
            .iter()
            .position(|t| matches!(t, Token::Symbol(']')))?;

        if bracket_start >= bracket_end {
            return None;
        }

        // Extract the variable name
        let variable_tokens = &tokens[bracket_start + 1..bracket_end];
        let variable = Self::tokens_to_string(variable_tokens);

        // Parse the cases after the closing bracket
        let remaining_tokens = &tokens[bracket_end + 1..];
        let cases = Self::parse_match_cases(remaining_tokens)?;

        Some(Box::new(SwitchDirective::new(variable, cases)))
    }

    fn parse_match_cases(tokens: &[Token]) -> Option<Vec<(String, String)>> {
        let mut cases = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Look for opening parenthesis
            if !matches!(tokens[i], Token::Symbol('(')) {
                i += 1;
                continue;
            }

            // Find the matching closing parenthesis
            let mut paren_count = 1;
            let mut j = i + 1;

            while j < tokens.len() && paren_count > 0 {
                match tokens[j] {
                    Token::Symbol('(') => paren_count += 1,
                    Token::Symbol(')') => paren_count -= 1,
                    _ => {}
                }
                j += 1;
            }

            if paren_count != 0 {
                // Unmatched parentheses
                return None;
            }

            // Extract tokens between parentheses
            let case_tokens = &tokens[i + 1..j - 1];

            // Find the colon separator
            let colon_pos = case_tokens
                .iter()
                .position(|t| matches!(t, Token::Symbol(':')))?;

            // Extract pattern and result
            let pattern_tokens = &case_tokens[..colon_pos];
            let result_tokens = &case_tokens[colon_pos + 1..];

            let pattern = Self::tokens_to_string(pattern_tokens);
            let result = Self::tokens_to_string(result_tokens);

            cases.push((pattern, result));

            i = j;
        }

        if cases.is_empty() { None } else { Some(cases) }
    }

    fn parse_conditional(tokens: &[Token]) -> Option<Box<dyn Directive>> {
        // Find the positions of '?' and ':' symbols
        let question_pos = tokens
            .iter()
            .position(|t| matches!(t, Token::Symbol('?')))?;
        let colon_pos = tokens
            .iter()
            .rposition(|t| matches!(t, Token::Symbol(':')))?;

        // Ensure we have the right order: condition ? part1 : part2
        if question_pos >= colon_pos {
            return None;
        }

        // Extract the condition part (everything before '?')
        let condition_tokens = &tokens[..question_pos];
        let condition_str = Self::tokens_to_string(condition_tokens);

        // Extract part1 (between '?' and ':')
        let part1_tokens = &tokens[question_pos + 1..colon_pos];
        let part1_str = Self::tokens_to_string(part1_tokens);

        // Extract part2 (everything after ':')
        let part2_tokens = &tokens[colon_pos + 1..];
        let part2_str = Self::tokens_to_string(part2_tokens);

        let conditional = ConditionalDirective::new(&condition_str, &part1_str, &part2_str);
        Some(Box::new(conditional))
    }

    fn tokens_to_string(tokens: &[Token]) -> String {
        tokens
            .iter()
            .map(|token| match token {
                Token::Slice(s) => s.clone(),
                Token::Symbol(c) => c.to_string(),
                Token::Int(i) => i.to_string(),
                Token::Float(f) => f.to_string(),
                Token::Uknown(u) => u.to_string(),
            })
            .collect::<Vec<_>>()
            .join("")
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
        let template = "{[is_admin]?(Admin Panel)(User Panel)}";
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

    #[test]
    fn test_equality_comparisons() {
        let template = "{name==Alice?Welcome Alice:Unknown user}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "name" => Value::String("Alice".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Welcome Alice");

        let ctx = map! {
            "name" => Value::String("Bob".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Unknown user");
    }

    #[test]
    fn test_numeric_comparisons() {
        let template = "{age>=18?Adult:Minor}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "age" => Value::Int(25),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Adult");

        let ctx = map! {
            "age" => Value::Int(16),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Minor");
    }

    #[test]
    fn test_backward_comparisons() {
        let template = "{10>=value?Greater than or equal:Less than}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "value" => Value::Int(5),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Greater than or equal");

        let ctx = map! {
            "value" => Value::Int(15),
        };

        assert_eq!(template.format(&ctx).unwrap(), "Less than");
    }

    #[test]
    fn test_boolean_comparisons() {
        let template = "{is_admin==true?Admin Panel:User Panel}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "is_admin" => Value::Bool(true),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Admin Panel");

        let ctx = map! {
            "is_admin" => Value::Bool(false),
        };
        assert_eq!(template.format(&ctx).unwrap(), "User Panel");
    }

    #[test]
    fn test_literal_comparisons() {
        let template = "{status==active?System Online:System Offline}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        // Test with literal values (not in context)
        let ctx = map![];
        assert_eq!(template.format(&ctx).unwrap(), "System Offline");
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that simple boolean conditions still work
        let template = "{is_admin?Admin Panel:User Panel}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "is_admin" => Value::Bool(true),
        };
        assert_eq!(template.format(&ctx).unwrap(), "Admin Panel");

        let ctx = map! {
            "username" => Value::String("Alice".to_string()),
        };

        let template = "{username?Logged In:Guest}";
        let template = Template::<'{', '}'>::parse(template).unwrap();
        assert_eq!(template.format(&ctx).unwrap(), "Logged In");
    }

    #[test]
    fn test_switch_directive_basic() {
        let template =
            "status: {[status](active:online)(inactive:offline)(maintenance:under maintenance)}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "status" => Value::String("active".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "status: online");

        let ctx = map! {
            "status" => Value::String("inactive".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "status: offline");

        let ctx = map! {
            "status" => Value::String("maintenance".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "status: under maintenance");
    }

    #[test]
    fn test_switch_directive_no_match() {
        let template = "status: {[status](active:online)(inactive:offline)}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "status" => Value::String("unknown".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "status: ");
    }

    #[test]
    fn test_switch_directive_with_context_variables() {
        let template = "User: {[user_type](admin:Administrator)(user:Regular User)}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map! {
            "user_type" => Value::String("admin".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "User: Administrator");

        let ctx = map! {
            "user_type" => Value::String("user".to_string()),
        };
        assert_eq!(template.format(&ctx).unwrap(), "User: Regular User");
    }

    #[test]
    fn test_switch_directive_variable_not_in_context() {
        let template = "status: {[unknown_var](active:online)(inactive:offline)}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map![];
        assert_eq!(template.format(&ctx).unwrap(), "status: ");
    }

    #[test]
    fn test_switch_directive_literal_matching() {
        // Test when the variable itself is a literal that matches a case
        let template = "status: {[active](active:System is running)(inactive:System is down)}";
        let template = Template::<'{', '}'>::parse(template).unwrap();

        let ctx = map![];
        assert_eq!(template.format(&ctx).unwrap(), "status: System is running");
    }
}
