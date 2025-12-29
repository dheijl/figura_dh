use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    /// Identifier token (name of variable used)
    Ident(String),
    Assign,

    /// Numbers
    Int(i64),
    Float(f64),

    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,

    /// Verbatim string
    Literal(String),

    /// Operators
    Not,
    Plus,
    Minus,
    Star,
    Slash,

    /// Comparison
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,

    /// Special
    End,
    Unknown(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(v) => write!(f, "{}", v),
            Self::Assign => write!(f, "Assign"),
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Literal(v) => write!(f, "{}", v),
            Self::LParen => write!(f, "LParen"),
            Self::RParen => write!(f, "RParen"),
            Self::LSquare => write!(f, "LSquare"),
            Self::RSquare => write!(f, "RSquare"),
            Self::LCurly => write!(f, "LCurly"),
            Self::RCurly => write!(f, "RCurly"),
            Self::Not => write!(f, "Not"),
            Self::Plus => write!(f, "Plus"),
            Self::Minus => write!(f, "Minus"),
            Self::Star => write!(f, "Star"),
            Self::Slash => write!(f, "Slash"),
            Self::Equal => write!(f, "Equal"),
            Self::NotEqual => write!(f, "NotEqual"),
            Self::GreaterThan => write!(f, "GreaterThan"),
            Self::LessThan => write!(f, "LessThan"),
            Self::GreaterThanOrEqual => write!(f, "GreaterThanOrEqual"),
            Self::LessThanOrEqual => write!(f, "LessThanOrEqual"),
            Self::And => write!(f, "And"),
            Self::Or => write!(f, "Or"),
            Self::End => write!(f, "End"),
            Self::Unknown(v) => write!(f, "Unknown({})", v),
        }
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    #[default]
    Left,

    Right,

    Center,
}

#[derive(Debug, Clone)]
struct Lexer {
    input: String,
    ch: char,
    cursor: usize,
}

impl Lexer {
    const EOF: char = '\0';

    pub fn new(input: &str) -> Self {
        let input = input.to_string();
        let ch = input.chars().next().unwrap_or(Self::EOF);

        let mut lexer = Self {
            input,
            ch,
            cursor: 0,
        };

        lexer.advance();
        lexer
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.advance();
        }
    }

    fn advance(&mut self) {
        if self.cursor >= self.input.len() {
            self.ch = Self::EOF;
        } else {
            // Safe to unwrap, since the length is checked
            self.ch = self.input.chars().nth(self.cursor).unwrap();
        }

        self.cursor += 1;
    }

    #[inline]
    fn peek(&mut self) -> Option<char> {
        self.input.chars().nth(self.cursor)
    }

    /// Reads a string literal, defined
    /// by encapsulating quotes
    ///
    /// E.G "Hello, world!" -> Literal(String(Hello, world!))
    fn read_string(&mut self) -> String {
        let mut output = String::new();

        // Skip opening quote
        self.advance();

        while self.ch != '"' && self.ch != Self::EOF {
            // Handle \
            if self.ch == '\\' {
                // Skip it
                self.advance();

                match self.ch {
                    'n' => output.push('\n'),
                    't' => output.push('\t'),
                    'r' => output.push('\r'),
                    '\\' => output.push('\\'),
                    '"' => output.push('"'),
                    '0' => output.push('\x00'),

                    // If unknown, include it verbatim
                    _ => output.push(self.ch),
                }

                self.advance();
            } else {
                output.push(self.ch);
                self.advance();
            }
        }

        // `self.ch` here is  '"', skip it
        // TODO: Handle  EOF
        self.advance();

        return output;
    }

    /// Reads a sequence of characters that is not a string literal.
    ///
    /// NOTE: This sequence of characters doesnt start with a digit.
    fn read_ident(&mut self) -> String {
        let mut output = String::new();

        while self.ch.is_ascii_alphabetic() || self.ch.is_ascii_digit() || self.ch == '_' {
            output.push(self.ch);
            self.advance();
        }

        output
    }

    /// Reads a sequence of digits (or .)
    /// And returns the literal string of it
    ///
    /// The lexer then shall see if the output is integer or float
    /// because the function returns a tuple of the literal string and a boolean indicating if it is a float
    fn read_number(&mut self) -> (String, bool) {
        let mut output = String::new();
        let mut decimal_point = false;

        while self.ch.is_ascii_digit()
            || (self.ch == '.'
                && !decimal_point
                && self.peek().map_or(false, |c| c.is_ascii_digit()))
        {
            if self.ch == '.' {
                decimal_point = true;
            }

            output.push(self.ch);
            self.advance();
        }

        (output, decimal_point)
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.ch == Self::EOF {
            return Token::End;
        }

        match self.ch {
            // =, could be ==
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Token::Equal
                } else {
                    self.advance();

                    Token::Assign
                }
            }

            '+' => {
                self.advance();
                Token::Plus
            }

            '-' => {
                self.advance();
                Token::Minus
            }

            '*' => {
                self.advance();
                Token::Star
            }

            '/' => {
                self.advance();
                Token::Slash
            }

            '(' => {
                self.advance();
                Token::LParen
            }

            ')' => {
                self.advance();
                Token::RParen
            }

            '[' => {
                self.advance();
                Token::LSquare
            }

            ']' => {
                self.advance();
                Token::RSquare
            }

            '{' => {
                self.advance();
                Token::LCurly
            }

            '}' => {
                self.advance();
                Token::RCurly
            }

            // !, could be !=
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Token::NotEqual
                } else {
                    self.advance();

                    Token::Not
                }
            }

            // <, could be <=
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Token::LessThanOrEqual
                } else {
                    self.advance();

                    Token::LessThan
                }
            }

            // >, could be >=
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Token::GreaterThanOrEqual
                } else {
                    self.advance();

                    Token::GreaterThan
                }
            }

            // &, must be &&
            // TODO: bitwise and
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    self.advance();

                    Token::And
                } else {
                    self.advance();

                    Token::Unknown('&')
                }
            }

            // |, must be ||
            // TODO: bitwise or
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    self.advance();

                    Token::Or
                } else {
                    self.advance();

                    Token::Unknown('|')
                }
            }

            // Read string literal
            '"' => Token::Literal(self.read_string()),

            // Identifier
            c if c.is_ascii_alphabetic() || c == '_' => {
                return Token::Ident(self.read_ident());
            }

            // Number
            c if c.is_ascii_digit() => {
                let (number, is_float) = self.read_number();
                return if is_float {
                    number
                        .parse::<f64>()
                        .map(Token::Float)
                        .unwrap_or(Token::Unknown(c))
                } else {
                    number
                        .parse::<i64>()
                        .map(Token::Int)
                        .unwrap_or(Token::Unknown(c))
                };
            }

            c => {
                self.advance();
                Token::Unknown(c)
            }
        }
    }
}

pub struct Template<const O: char = '{', const C: char = '}'> {
    alignment: Alignment,
}

impl<const O: char, const C: char> Template<O, C> {
    pub fn new<T: AsRef<str>>(input: T) -> Self {
        Self {
            alignment: Alignment::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic token tests
    #[test]
    fn test_lexer_int() {
        let input = "123";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
    }

    #[test]
    fn test_lexer_large_int() {
        let input = "9223372036854775807";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(9223372036854775807));
    }

    #[test]
    fn test_lexer_zero() {
        let input = "0";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(0));
    }

    #[test]
    fn test_lexer_float() {
        let input = "123.456";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Float(123.456));
    }

    #[test]
    fn test_lexer_float_leading_zero() {
        let input = "0.5";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Float(0.5));
    }

    #[test]
    fn test_lexer_ident() {
        let input = "abc";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("abc".to_string()));
    }

    #[test]
    fn test_lexer_ident_with_underscore() {
        let input = "_private_var";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("_private_var".to_string()));
    }

    #[test]
    fn test_lexer_ident_with_numbers() {
        let input = "var123";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("var123".to_string()));
    }

    #[test]
    fn test_lexer_ident_mixed() {
        let input = "my_var_1";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("my_var_1".to_string()));
    }

    // String literal tests
    #[test]
    fn test_lexer_string() {
        let input = "\"hello\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Literal("hello".to_string()));
    }

    #[test]
    fn test_lexer_empty_string() {
        let input = "\"\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Literal("".to_string()));
    }

    #[test]
    fn test_lexer_string_with_spaces() {
        let input = "\"hello world\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello world".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_newline() {
        let input = "\"hello\\nworld\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\nworld".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_tab() {
        let input = "\"hello\\tworld\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\tworld".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_return() {
        let input = "\"hello\\rworld\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\rworld".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_backslash() {
        let input = "\"hello\\\\world\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\\world".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_quote() {
        let input = "\"hello\\\"world\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\"world".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_escape_null() {
        let input = "\"hello\\0world\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("hello\x00world".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_unknown_escape() {
        let input = "\"hello\\xworld\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("helloxworld".to_string())
        );
    }

    #[test]
    fn test_lexer_string_with_multiple_escapes() {
        let input = "\"line1\\nline2\\tindented\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next_token(),
            Token::Literal("line1\nline2\tindented".to_string())
        );
    }

    // Operator tests
    #[test]
    fn test_lexer_assign() {
        let input = "=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Assign);
    }

    #[test]
    fn test_lexer_not() {
        let input = "!";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Not);
    }

    // Comparison operators
    #[test]
    fn test_lexer_equal() {
        let input = "==";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Equal);
    }

    #[test]
    fn test_lexer_not_equal() {
        let input = "!=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::NotEqual);
    }

    #[test]
    fn test_lexer_greater_than() {
        let input = ">";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::GreaterThan);
    }

    #[test]
    fn test_lexer_less_than() {
        let input = "<";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::LessThan);
    }

    #[test]
    fn test_lexer_greater_than_or_equal() {
        let input = ">=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::GreaterThanOrEqual);
    }

    #[test]
    fn test_lexer_less_than_or_equal() {
        let input = "<=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::LessThanOrEqual);
    }

    #[test]
    fn test_lexer_and() {
        let input = "&&";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::And);
    }

    #[test]
    fn test_lexer_or() {
        let input = "||";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Or);
    }

    #[test]
    fn test_lexer_single_ampersand() {
        let input = "&";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Unknown('&'));
    }

    #[test]
    fn test_lexer_single_pipe() {
        let input = "|";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Unknown('|'));
    }

    // Whitespace handling
    #[test]
    fn test_lexer_whitespace_between_tokens() {
        let input = "123   456";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Int(456));
    }

    #[test]
    fn test_lexer_leading_whitespace() {
        let input = "   123";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
    }

    #[test]
    fn test_lexer_trailing_whitespace() {
        let input = "123   ";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::End);
    }

    #[test]
    fn test_lexer_tabs_and_newlines() {
        let input = "123\t\n456";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Int(456));
    }

    // Multiple token sequences
    #[test]
    fn test_lexer_assignment_expression() {
        let input = "x = 42";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::End);
    }

    #[test]
    fn test_lexer_comparison_expression() {
        let input = "x == 10";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Int(10));
    }

    #[test]
    fn test_lexer_logical_expression() {
        let input = "x > 5 && y < 10";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::GreaterThan);
        assert_eq!(lexer.next_token(), Token::Int(5));
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Ident("y".to_string()));
        assert_eq!(lexer.next_token(), Token::LessThan);
        assert_eq!(lexer.next_token(), Token::Int(10));
    }

    #[test]
    fn test_lexer_complex_expression() {
        let input = "result = (x != 0) || (y >= 100)";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("result".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::Int(0));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Or);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("y".to_string()));
        assert_eq!(lexer.next_token(), Token::GreaterThanOrEqual);
        assert_eq!(lexer.next_token(), Token::Int(100));
        assert_eq!(lexer.next_token(), Token::RParen);
    }

    #[test]
    fn test_lexer_string_assignment() {
        let input = "name = \"John Doe\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("name".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Literal("John Doe".to_string()));
    }

    #[test]
    fn test_lexer_mixed_types() {
        let input = "x = 42 y = 3.14 z = \"test\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(42));
        assert_eq!(lexer.next_token(), Token::Ident("y".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Float(3.14));
        assert_eq!(lexer.next_token(), Token::Ident("z".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Literal("test".to_string()));
    }

    // Edge cases
    #[test]
    fn test_lexer_empty_input() {
        let input = "";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::End);
    }

    #[test]
    fn test_lexer_only_whitespace() {
        let input = "   \t\n  ";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::End);
    }

    #[test]
    fn test_lexer_unknown_characters() {
        let input = "@#$%";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Unknown('@'));
        assert_eq!(lexer.next_token(), Token::Unknown('#'));
        assert_eq!(lexer.next_token(), Token::Unknown('$'));
        assert_eq!(lexer.next_token(), Token::Unknown('%'));
    }

    #[test]
    fn test_lexer_no_space_between_tokens() {
        let input = "x=42";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(42));
    }

    #[test]
    fn test_lexer_consecutive_operators() {
        let input = "==!=>=<=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::GreaterThanOrEqual);
        assert_eq!(lexer.next_token(), Token::LessThanOrEqual);
    }

    #[test]
    fn test_lexer_number_followed_by_ident() {
        let input = "123abc";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Ident("abc".to_string()));
    }

    #[test]
    fn test_lexer_decimal_without_trailing_digit() {
        let input = "123.";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Unknown('.'));
    }

    #[test]
    fn test_lexer_multiple_decimals() {
        let input = "123.456.789";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Float(123.456));
        assert_eq!(lexer.next_token(), Token::Unknown('.'));
        assert_eq!(lexer.next_token(), Token::Int(789));
    }

    #[test]
    fn test_lexer_end_token_repeated() {
        let input = "";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::End);
        assert_eq!(lexer.next_token(), Token::End);
        assert_eq!(lexer.next_token(), Token::End);
    }

    #[test]
    fn test_lexer_all_comparison_operators() {
        let input = "< > <= >= == !=";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::LessThan);
        assert_eq!(lexer.next_token(), Token::GreaterThan);
        assert_eq!(lexer.next_token(), Token::LessThanOrEqual);
        assert_eq!(lexer.next_token(), Token::GreaterThanOrEqual);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
    }

    #[test]
    fn test_lexer_underscore_only_ident() {
        let input = "_";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("_".to_string()));
    }

    #[test]
    fn test_lexer_multiple_underscores() {
        let input = "___test___";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Ident("___test___".to_string()));
    }
}
