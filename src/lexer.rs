use std::{fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    /// Identifier token (name of variable used)
    Ident(Rc<str>),
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
    Colon,
    Semicolon,

    Question,
    Pipe,
    Arrow,
    Underscore,

    /// Verbatim string
    Literal(Rc<str>),

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

    Unknown(char),
}

impl Token {
    pub fn as_string(&self) -> Rc<str> {
        match self {
            Self::Ident(v) | Self::Literal(v) => Rc::clone(v),
            Self::Int(v) => Rc::from(v.to_string()),
            Self::Float(v) => Rc::from(v.to_string()),
            Self::Colon => Rc::from(":"),
            Self::Question => Rc::from("?"),
            Self::Pipe => Rc::from("|"),
            Self::Arrow => Rc::from("=>"),
            _ => Rc::from(""),
        }
    }
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
            Self::Colon => write!(f, "Colon"),
            Self::Semicolon => write!(f, "Semicolon"),
            Self::Question => write!(f, "Question"),
            Self::Pipe => write!(f, "Pipe"),
            Self::Arrow => write!(f, "Arrow"),
            Self::Underscore => write!(f, "Underscore"),
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
            Self::Unknown(v) => write!(f, "Unknown({})", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    ch: char,
    cursor: usize,
}

impl Lexer {
    const EOF: char = '\0';

    fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let ch = chars.first().copied().unwrap_or(Self::EOF);
        Self {
            input: chars,
            ch,
            cursor: 0,
        }
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.advance();
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.cursor += 1;
        self.ch = self.input.get(self.cursor).copied().unwrap_or(Self::EOF);
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.input.get(self.cursor + 1).copied()
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

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.ch == Self::EOF {
            return None;
        }

        match self.ch {
            // =, could be == or =>
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Some(Token::Equal)
                } else if self.peek() == Some('>') {
                    self.advance();
                    self.advance();

                    Some(Token::Arrow)
                } else {
                    self.advance();

                    Some(Token::Assign)
                }
            }

            '+' => {
                self.advance();
                Some(Token::Plus)
            }

            '-' => {
                self.advance();
                Some(Token::Minus)
            }

            '*' => {
                self.advance();
                Some(Token::Star)
            }

            '/' => {
                self.advance();
                Some(Token::Slash)
            }

            '(' => {
                self.advance();
                Some(Token::LParen)
            }

            ')' => {
                self.advance();
                Some(Token::RParen)
            }

            '[' => {
                self.advance();
                Some(Token::LSquare)
            }

            ']' => {
                self.advance();
                Some(Token::RSquare)
            }

            '{' => {
                self.advance();
                Some(Token::LCurly)
            }

            '}' => {
                self.advance();
                Some(Token::RCurly)
            }

            ':' => {
                self.advance();
                Some(Token::Colon)
            }

            ';' => {
                self.advance();
                Some(Token::Semicolon)
            }

            '?' => {
                self.advance();
                Some(Token::Question)
            }

            // !, could be !=
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Some(Token::NotEqual)
                } else {
                    self.advance();

                    Some(Token::Not)
                }
            }

            // <, could be <=
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Some(Token::LessThanOrEqual)
                } else {
                    self.advance();

                    Some(Token::LessThan)
                }
            }

            // >, could be >=
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();

                    Some(Token::GreaterThanOrEqual)
                } else {
                    self.advance();

                    Some(Token::GreaterThan)
                }
            }

            // &, must be &&
            // TODO: bitwise and
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    self.advance();

                    Some(Token::And)
                } else {
                    self.advance();

                    Some(Token::Unknown('&'))
                }
            }

            // |, must be ||
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    self.advance();

                    Some(Token::Or)
                } else {
                    self.advance();

                    Some(Token::Pipe)
                }
            }

            '_' => {
                self.advance();

                Some(Token::Underscore)
            }

            // Read string literal
            '"' => Some(Token::Literal(self.read_string().into())),

            // Identifier
            c if c.is_ascii_alphabetic() || c == '_' => {
                return Some(Token::Ident(self.read_ident().into()));
            }

            // Number
            c if c.is_ascii_digit() => {
                let (number, is_float) = self.read_number();
                return if is_float {
                    Some(
                        number
                            .parse::<f64>()
                            .map(Token::Float)
                            .unwrap_or(Token::Unknown(c)),
                    )
                } else {
                    Some(
                        number
                            .parse::<i64>()
                            .map(Token::Int)
                            .unwrap_or(Token::Unknown(c)),
                    )
                };
            }

            c => {
                self.advance();
                Some(Token::Unknown(c))
            }
        }
    }

    #[inline]
    pub fn tokenize(input: &str) -> Vec<Token> {
        Self::new(input).collect::<Vec<_>>()
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod lexer_tests {
    use crate::lexer::{Lexer, Token};
    use std::rc::Rc;

    // ==================== Basic Token Tests ====================

    #[test]
    fn test_empty_input() {
        let tokens = Lexer::tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = Lexer::tokenize("   \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_single_identifier() {
        let tokens = Lexer::tokenize("name");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "name"));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let tokens = Lexer::tokenize("first_name");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "first_name"));
    }

    #[test]
    fn test_identifier_with_numbers() {
        let tokens = Lexer::tokenize("var123");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "var123"));
    }

    #[test]
    fn test_underscore_prefix() {
        let tokens = Lexer::tokenize("_private");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Underscore);
        assert!(matches!(&tokens[1], Token::Ident(s) if &**s == "private"));
    }

    // ==================== Number Tests ====================

    #[test]
    fn test_integer() {
        let tokens = Lexer::tokenize("42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Int(42));
    }

    #[test]
    fn test_negative_integer() {
        let tokens = Lexer::tokenize("-42");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Minus);
        assert_eq!(tokens[1], Token::Int(42));
    }

    #[test]
    fn test_zero() {
        let tokens = Lexer::tokenize("0");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Int(0));
    }

    #[test]
    fn test_large_integer() {
        let tokens = Lexer::tokenize("9223372036854775807");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Int(i64::MAX));
    }

    #[test]
    fn test_float() {
        let tokens = Lexer::tokenize("3.14");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Float(3.14));
    }

    #[test]
    fn test_float_leading_zero() {
        let tokens = Lexer::tokenize("0.5");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Float(0.5));
    }

    #[test]
    fn test_float_multiple_digits() {
        let tokens = Lexer::tokenize("123.456");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Float(123.456));
    }

    #[test]
    fn test_number_followed_by_dot_no_digit() {
        // "42.abc" = 3 tokens: Int(42), Unknown('.'), Ident("abc")
        // The dot is not consumed as part of the number since no digit follows
        let tokens = Lexer::tokenize("42.abc");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Int(42));
        assert_eq!(tokens[1], Token::Unknown('.'));
        assert!(matches!(&tokens[2], Token::Ident(s) if &**s == "abc"));
    }

    #[test]
    fn test_integer_overflow_becomes_unknown() {
        let tokens = Lexer::tokenize("99999999999999999999999999999");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Unknown(_)));
    }

    // ==================== String Literal Tests ====================

    #[test]
    fn test_simple_string() {
        let tokens = Lexer::tokenize("\"hello\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "hello"));
    }

    #[test]
    fn test_empty_string() {
        let tokens = Lexer::tokenize("\"\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == ""));
    }

    #[test]
    fn test_string_with_spaces() {
        let tokens = Lexer::tokenize("\"hello world\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "hello world"));
    }

    #[test]
    fn test_string_escape_newline() {
        let tokens = Lexer::tokenize("\"line1\\nline2\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "line1\nline2"));
    }

    #[test]
    fn test_string_escape_tab() {
        let tokens = Lexer::tokenize("\"col1\\tcol2\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "col1\tcol2"));
    }

    #[test]
    fn test_string_escape_carriage_return() {
        let tokens = Lexer::tokenize("\"line\\r\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "line\r"));
    }

    #[test]
    fn test_string_escape_backslash() {
        let tokens = Lexer::tokenize("\"path\\\\to\\\\file\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "path\\to\\file"));
    }

    #[test]
    fn test_string_escape_quote() {
        let tokens = Lexer::tokenize("\"say \\\"hello\\\"\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "say \"hello\""));
    }

    #[test]
    fn test_string_escape_null() {
        let tokens = Lexer::tokenize("\"null\\0char\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "null\x00char"));
    }

    #[test]
    fn test_string_unknown_escape() {
        let tokens = Lexer::tokenize("\"\\x\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Literal(s) if &**s == "x"));
    }

    // ==================== Operator Tests ====================

    #[test]
    fn test_assign() {
        let tokens = Lexer::tokenize("=");
        assert_eq!(tokens, vec![Token::Assign]);
    }

    #[test]
    fn test_equal() {
        let tokens = Lexer::tokenize("==");
        assert_eq!(tokens, vec![Token::Equal]);
    }

    #[test]
    fn test_arrow() {
        let tokens = Lexer::tokenize("=>");
        assert_eq!(tokens, vec![Token::Arrow]);
    }

    #[test]
    fn test_not() {
        let tokens = Lexer::tokenize("!");
        assert_eq!(tokens, vec![Token::Not]);
    }

    #[test]
    fn test_not_equal() {
        let tokens = Lexer::tokenize("!=");
        assert_eq!(tokens, vec![Token::NotEqual]);
    }

    #[test]
    fn test_plus() {
        let tokens = Lexer::tokenize("+");
        assert_eq!(tokens, vec![Token::Plus]);
    }

    #[test]
    fn test_minus() {
        let tokens = Lexer::tokenize("-");
        assert_eq!(tokens, vec![Token::Minus]);
    }

    #[test]
    fn test_star() {
        let tokens = Lexer::tokenize("*");
        assert_eq!(tokens, vec![Token::Star]);
    }

    #[test]
    fn test_slash() {
        let tokens = Lexer::tokenize("/");
        assert_eq!(tokens, vec![Token::Slash]);
    }

    #[test]
    fn test_less_than() {
        let tokens = Lexer::tokenize("<");
        assert_eq!(tokens, vec![Token::LessThan]);
    }

    #[test]
    fn test_less_than_or_equal() {
        let tokens = Lexer::tokenize("<=");
        assert_eq!(tokens, vec![Token::LessThanOrEqual]);
    }

    #[test]
    fn test_greater_than() {
        let tokens = Lexer::tokenize(">");
        assert_eq!(tokens, vec![Token::GreaterThan]);
    }

    #[test]
    fn test_greater_than_or_equal() {
        let tokens = Lexer::tokenize(">=");
        assert_eq!(tokens, vec![Token::GreaterThanOrEqual]);
    }

    #[test]
    fn test_and() {
        let tokens = Lexer::tokenize("&&");
        assert_eq!(tokens, vec![Token::And]);
    }

    #[test]
    fn test_single_ampersand_unknown() {
        let tokens = Lexer::tokenize("&");
        assert_eq!(tokens, vec![Token::Unknown('&')]);
    }

    #[test]
    fn test_or() {
        let tokens = Lexer::tokenize("||");
        assert_eq!(tokens, vec![Token::Or]);
    }

    #[test]
    fn test_pipe() {
        let tokens = Lexer::tokenize("|");
        assert_eq!(tokens, vec![Token::Pipe]);
    }

    // ==================== Delimiter Tests ====================

    #[test]
    fn test_lparen() {
        let tokens = Lexer::tokenize("(");
        assert_eq!(tokens, vec![Token::LParen]);
    }

    #[test]
    fn test_rparen() {
        let tokens = Lexer::tokenize(")");
        assert_eq!(tokens, vec![Token::RParen]);
    }

    #[test]
    fn test_lsquare() {
        let tokens = Lexer::tokenize("[");
        assert_eq!(tokens, vec![Token::LSquare]);
    }

    #[test]
    fn test_rsquare() {
        let tokens = Lexer::tokenize("]");
        assert_eq!(tokens, vec![Token::RSquare]);
    }

    #[test]
    fn test_lcurly() {
        let tokens = Lexer::tokenize("{");
        assert_eq!(tokens, vec![Token::LCurly]);
    }

    #[test]
    fn test_rcurly() {
        let tokens = Lexer::tokenize("}");
        assert_eq!(tokens, vec![Token::RCurly]);
    }

    #[test]
    fn test_colon() {
        let tokens = Lexer::tokenize(":");
        assert_eq!(tokens, vec![Token::Colon]);
    }

    #[test]
    fn test_semicolon() {
        let tokens = Lexer::tokenize(";");
        assert_eq!(tokens, vec![Token::Semicolon]);
    }

    #[test]
    fn test_question() {
        let tokens = Lexer::tokenize("?");
        assert_eq!(tokens, vec![Token::Question]);
    }

    #[test]
    fn test_underscore() {
        let tokens = Lexer::tokenize("_");
        assert_eq!(tokens, vec![Token::Underscore]);
    }

    // ==================== Complex Expression Tests ====================

    #[test]
    fn test_repeat_pattern() {
        let tokens = Lexer::tokenize("pattern : 5");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "pattern"));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::Int(5));
    }

    #[test]
    fn test_conditional_pattern() {
        let tokens = Lexer::tokenize("cond ? yes : no");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "cond"));
        assert_eq!(tokens[1], Token::Question);
        assert!(matches!(&tokens[2], Token::Ident(s) if &**s == "yes"));
        assert_eq!(tokens[3], Token::Colon);
        assert!(matches!(&tokens[4], Token::Ident(s) if &**s == "no"));
    }

    #[test]
    fn test_switch_pattern() {
        let tokens = Lexer::tokenize("val | a => x | b => y | _ => z");
        assert_eq!(tokens.len(), 13);
        assert!(matches!(&tokens[0], Token::Ident(s) if &**s == "val"));
        assert_eq!(tokens[1], Token::Pipe);
        assert!(matches!(&tokens[2], Token::Ident(s) if &**s == "a"));
        assert_eq!(tokens[3], Token::Arrow);
    }

    #[test]
    fn test_arithmetic_expression() {
        let tokens = Lexer::tokenize("a + b * c - d / e");
        assert_eq!(tokens.len(), 9);
        assert!(matches!(&tokens[0], Token::Ident(_)));
        assert_eq!(tokens[1], Token::Plus);
        assert!(matches!(&tokens[2], Token::Ident(_)));
        assert_eq!(tokens[3], Token::Star);
    }

    #[test]
    fn test_comparison_chain() {
        let tokens = Lexer::tokenize("a < b && c > d || e == f");
        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn test_nested_parens() {
        let tokens = Lexer::tokenize("((a + b) * c)");
        let expected = vec![
            Token::LParen,
            Token::LParen,
            Token::Ident(Rc::from("a")),
            Token::Plus,
            Token::Ident(Rc::from("b")),
            Token::RParen,
            Token::Star,
            Token::Ident(Rc::from("c")),
            Token::RParen,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_mixed_brackets() {
        let tokens = Lexer::tokenize("arr[0]{key}(fn)");

        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[1], Token::LSquare);
        assert_eq!(tokens[3], Token::RSquare);
        assert_eq!(tokens[4], Token::LCurly);
        assert_eq!(tokens[6], Token::RCurly);
        assert_eq!(tokens[7], Token::LParen);
        assert_eq!(tokens[9], Token::RParen);
    }

    // ==================== Whitespace Handling Tests ====================

    #[test]
    fn test_multiple_spaces() {
        let tokens = Lexer::tokenize("a    b");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tabs_between_tokens() {
        let tokens = Lexer::tokenize("a\t\tb");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_newlines_between_tokens() {
        let tokens = Lexer::tokenize("a\n\nb");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_mixed_whitespace() {
        let tokens = Lexer::tokenize("  a \t\n  b  \n");
        assert_eq!(tokens.len(), 2);
    }

    // ==================== Unknown Character Tests ====================

    #[test]
    fn test_unknown_tilde() {
        let tokens = Lexer::tokenize("~");
        assert_eq!(tokens, vec![Token::Unknown('~')]);
    }

    #[test]
    fn test_unknown_at() {
        let tokens = Lexer::tokenize("@");
        assert_eq!(tokens, vec![Token::Unknown('@')]);
    }

    #[test]
    fn test_unknown_hash() {
        let tokens = Lexer::tokenize("#");
        assert_eq!(tokens, vec![Token::Unknown('#')]);
    }

    #[test]
    fn test_unknown_backtick() {
        let tokens = Lexer::tokenize("`");
        assert_eq!(tokens, vec![Token::Unknown('`')]);
    }

    // ==================== Token Display Tests ====================

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::Assign), "Assign");
        assert_eq!(format!("{}", Token::Plus), "Plus");
        assert_eq!(format!("{}", Token::Int(42)), "42");
        assert_eq!(format!("{}", Token::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Token::Ident(Rc::from("foo"))), "foo");
        assert_eq!(format!("{}", Token::Unknown('?')), "Unknown(?)");
    }

    // ==================== Token as_string Tests ====================

    #[test]
    fn test_token_as_string_ident() {
        let token = Token::Ident(Rc::from("myvar"));
        assert_eq!(&*token.as_string(), "myvar");
    }

    #[test]
    fn test_token_as_string_literal() {
        let token = Token::Literal(Rc::from("hello"));
        assert_eq!(&*token.as_string(), "hello");
    }

    #[test]
    fn test_token_as_string_int() {
        let token = Token::Int(123);
        assert_eq!(&*token.as_string(), "123");
    }

    #[test]
    fn test_token_as_string_float() {
        let token = Token::Float(3.5);
        assert_eq!(&*token.as_string(), "3.5");
    }

    #[test]
    fn test_token_as_string_colon() {
        assert_eq!(&*Token::Colon.as_string(), ":");
    }

    #[test]
    fn test_token_as_string_question() {
        assert_eq!(&*Token::Question.as_string(), "?");
    }

    #[test]
    fn test_token_as_string_pipe() {
        assert_eq!(&*Token::Pipe.as_string(), "|");
    }

    #[test]
    fn test_token_as_string_arrow() {
        assert_eq!(&*Token::Arrow.as_string(), "=>");
    }

    #[test]
    fn test_token_as_string_other_returns_empty() {
        assert_eq!(&*Token::Plus.as_string(), "");
        assert_eq!(&*Token::Minus.as_string(), "");
        assert_eq!(&*Token::LParen.as_string(), "");
    }
}
