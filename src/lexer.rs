use core::mem;
use std::{iter::Peekable, rc::Rc, str::Chars};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    Ident(Rc<str>),
    Assign,

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
    Underscore,

    /// Verbatim string
    Literal(Rc<str>),

    // Operations
    Not,
    Plus,
    Minus,
    Star,
    Slash,

    // Comparison
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals,
    And,
    Or,

    // Special
    Arrow,

    Unknown(char),
}

impl Token {
    const EOF: char = '\0';

    /// Check if a character can form a multi-character token
    #[inline]
    fn try_double_char(first: char, second: char) -> Option<Token> {
        match (first, second) {
            ('=', '=') => Some(Token::Equals),
            ('-', '>') => Some(Token::Arrow),
            ('!', '=') => Some(Token::NotEquals),
            ('<', '=') => Some(Token::LessThanEquals),
            ('>', '=') => Some(Token::GreaterThanEquals),
            ('&', '&') => Some(Token::And),
            ('|', '|') => Some(Token::Or),

            _ => None,
        }
    }

    #[inline]
    fn try_single_char(ch: char) -> Option<Token> {
        match ch {
            '=' => Some(Token::Assign),
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            '[' => Some(Token::LSquare),
            ']' => Some(Token::RSquare),
            '{' => Some(Token::LCurly),
            '}' => Some(Token::RCurly),
            ':' => Some(Token::Colon),
            ';' => Some(Token::Semicolon),
            '?' => Some(Token::Question),
            '|' => Some(Token::Pipe),
            '!' => Some(Token::Not),
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '>' => Some(Token::GreaterThan),
            '<' => Some(Token::LessThan),

            _ => None,
        }
    }
}

struct TemplateLexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> TemplateLexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    #[inline]
    fn advance(&mut self) -> Option<char> {
        self.input.next()
    }

    #[inline]
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }

            self.advance();
        }
    }

    fn read_literal(&mut self) -> Rc<str> {
        let mut output = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => break,

                '\\' => match self.advance() {
                    Some('n') => output.push('\n'),
                    Some('t') => output.push('\t'),
                    Some('r') => output.push('\r'),
                    Some('\\') => output.push('\\'),
                    Some('"') => output.push('"'),
                    Some('0') => output.push('\0'),
                    Some(c) => output.push(c),
                    None => break,
                },

                c => output.push(c),
            }
        }

        Rc::from(output)
    }

    fn read_ident(&mut self, first: char) -> Rc<str> {
        let mut output = String::from(first);

        while let Some(&ch) = self.peek() {
            if ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_' {
                output.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        Rc::from(output)
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut output = String::from(first);
        let mut is_float = false;

        while let Some(&ch) = self.peek() {
            if ch.is_ascii_digit() {
                output.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if let Some(&'.') = self.peek() {
            let mut iter_clone = self.input.clone();

            iter_clone.next();

            if let Some(next_ch) = iter_clone.next() {
                if next_ch.is_ascii_digit() {
                    is_float = true;

                    output.push(self.advance().unwrap());

                    while let Some(&ch) = self.peek() {
                        if ch.is_ascii_digit() {
                            output.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if is_float {
            output
                .parse::<f64>()
                .map(Token::Float)
                .unwrap_or(Token::Unknown(first))
        } else {
            output
                .parse::<i64>()
                .map(Token::Int)
                .unwrap_or(Token::Unknown(first))
        }
    }
    fn check_double(&mut self, expected: char, matched: Token, unmatched: Token) -> Token {
        if let Some(&ch) = self.peek() {
            if ch == expected {
                self.advance();
                return matched;
            }
        }
        unmatched
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let ch = self.advance()?;
        match ch {
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            '[' => Some(Token::LSquare),
            ']' => Some(Token::RSquare),
            '{' => Some(Token::LCurly),
            '}' => Some(Token::RCurly),
            ':' => Some(Token::Colon),
            ';' => Some(Token::Semicolon),
            '?' => Some(Token::Question),
            '=' => Some(self.check_double('=', Token::Equals, Token::Assign)),
            '!' => Some(self.check_double('=', Token::NotEquals, Token::Not)),
            '<' => Some(self.check_double('=', Token::LessThanEquals, Token::LessThan)),
            '>' => Some(self.check_double('=', Token::GreaterThanEquals, Token::GreaterThan)),
            '-' => Some(self.check_double('>', Token::Arrow, Token::Minus)),
            '&' => Some(self.check_double('&', Token::And, Token::Unknown('&'))),
            '|' => Some(self.check_double('|', Token::Or, Token::Pipe)),
            '+' => Some(Token::Plus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '"' => Some(Token::Literal(self.read_literal())),
            '_' => {
                if let Some(&next) = self.peek() {
                    if next.is_ascii_alphabetic() || next.is_ascii_digit() || next == '_' {
                        return Some(Token::Ident(self.read_ident(ch)));
                    }
                }

                Some(Token::Underscore)
            }
            c if c.is_ascii_alphabetic() => Some(Token::Ident(self.read_ident(c))),
            c if c.is_ascii_digit() => Some(self.read_number(c)),

            c => Some(Token::Unknown(c)),
        }
    }
}

impl<'a> Iterator for TemplateLexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub struct Lexer<const O: char, const C: char> {
    templates: Vec<String>,
}

impl<const O: char, const C: char> Lexer<O, C> {
    /// Compile time check for forbbidden delimiters
    const _CHECK: () = {
        assert!(O != '\\', "opening delimiter cannot be a backslash (`\\`)");
        assert!(C != '\\', "closing delimiter cannot be a backslash (`\\`)");
    };

    pub fn new<S>(input: S) -> Self
    where
        S: AsRef<str>,
    {
        _ = Self::_CHECK;

        let mut templates = Vec::new();
        let mut chars = input.as_ref().chars().peekable();
        let mut in_template = false;
        let mut curr_template = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O => {
                    // Skip double openings (escape)
                    if chars.peek() == Some(&O) {
                        chars.next();

                        if in_template {
                            curr_template.push(O);
                        }

                        continue;
                    }

                    if !in_template {
                        in_template = true;
                    } else {
                        // Nested opening delimiter
                        curr_template.push(ch);
                    }
                }

                c if c == C => {
                    if chars.peek() == Some(&C) {
                        chars.next();

                        if in_template {
                            curr_template.push(C);
                        }

                        continue;
                    }

                    if in_template {
                        in_template = false;

                        if !curr_template.is_empty() {
                            templates.push(mem::take(&mut curr_template));
                        }
                    }
                }

                ch => {
                    if in_template {
                        curr_template.push(ch)
                    }
                }
            }
        }

        Self { templates }
    }

    fn tokenize(self) -> Vec<Vec<Token>> {
        let mut results = Vec::with_capacity(self.templates.len());

        for template in self.templates {
            let lexer = TemplateLexer::new(&template);

            results.push(lexer.collect::<Vec<_>>());
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type L = Lexer<'{', '}'>;

    fn tokenize(input: &str) -> Vec<Token> {
        TemplateLexer::new(input).collect()
    }

    #[test]
    fn test_basic_arithmetic() {
        let input = "10 + 20 * 5";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Int(10),
                Token::Plus,
                Token::Int(20),
                Token::Star,
                Token::Int(5),
            ]
        );
    }

    #[test]
    fn test_identifiers_and_assignment() {
        let input = "my_var = x_1";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Ident(Rc::from("my_var")),
                Token::Assign,
                Token::Ident(Rc::from("x_1")),
            ]
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(tokenize("123"), vec![Token::Int(123)]);
        assert_eq!(tokenize("123.456"), vec![Token::Float(123.456)]);

        let tokens = tokenize("123.");
        assert_eq!(tokens, vec![Token::Int(123), Token::Unknown('.')]);
    }

    #[test]
    fn test_string_literals() {
        let input = r#" "Hello World" "Escaped\nNewline" "#;
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::Literal(Rc::from("Hello World")),
                Token::Literal(Rc::from("Escaped\nNewline")),
            ]
        );
    }

    #[test]
    fn test_comparison_and_logic() {
        let input = "a >= b && c != d || e < f";
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::Ident(Rc::from("a")),
                Token::GreaterThanEquals,
                Token::Ident(Rc::from("b")),
                Token::And,
                Token::Ident(Rc::from("c")),
                Token::NotEquals,
                Token::Ident(Rc::from("d")),
                Token::Or,
                Token::Ident(Rc::from("e")),
                Token::LessThan,
                Token::Ident(Rc::from("f")),
            ]
        );
    }

    #[test]
    fn test_grouping_delimiters() {
        let input = "( [ { } ] )";
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::LSquare,
                Token::LCurly,
                Token::RCurly,
                Token::RSquare,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_arrow_and_minus() {
        let input = "a -> b - c";
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::Ident(Rc::from("a")),
                Token::Arrow,
                Token::Ident(Rc::from("b")),
                Token::Minus,
                Token::Ident(Rc::from("c")),
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let input = "func(x, 10) | filter";
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::Ident(Rc::from("func")),
                Token::LParen,
                Token::Ident(Rc::from("x")),
                Token::Unknown(','),
                Token::Int(10),
                Token::RParen,
                Token::Pipe,
                Token::Ident(Rc::from("filter")),
            ]
        );
    }

    #[test]
    fn test_underscores() {
        assert_eq!(tokenize("_"), vec![Token::Underscore]);
        assert_eq!(tokenize("_var"), vec![Token::Ident(Rc::from("_var"))]);
    }

    #[test]
    fn test_outer_lexer_integration() {
        let input = "Hello { name }! score: { 1 + 2 }";
        let lexer = Lexer::<'{', '}'>::new(input);
        let results = lexer.tokenize();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0], vec![Token::Ident(Rc::from("name"))]);

        assert_eq!(results[1], vec![Token::Int(1), Token::Plus, Token::Int(2)]);
    }

    #[test]
    fn test_outer_lexer_escaping() {
        let input = "Hello {{ ignored }} { actual }";
        let lexer = Lexer::<'{', '}'>::new(input);
        let results = lexer.tokenize();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vec![Token::Ident(Rc::from("actual"))]);
    }

    #[test]
    fn empty_input() {
        let input = "";
        let lexer = L::new(input);

        assert!(lexer.templates.is_empty());
    }

    #[test]
    fn no_templates() {
        let input = "Just some random text without any brackets.";
        let lexer = L::new(input);

        assert!(lexer.templates.is_empty());
    }

    #[test]
    fn basic_single() {
        let input = "Hello! My name is {name}!";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["name"]);
    }

    #[test]
    fn multiple_templates() {
        let input = "{first} then {second} and {third}";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["first", "second", "third"]);
    }

    #[test]
    fn consecutive_templates() {
        let input = "{a}{b}{c}";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_inside_template() {
        let input = "This is empty {}";
        let lexer = L::new(input);

        assert!(lexer.templates.is_empty());
    }

    #[test]
    fn escaped_openers() {
        let input = "This is {{escaped}}";
        let lexer = L::new(input);

        assert!(lexer.templates.is_empty());
    }

    #[test]
    fn ignore_text_inside_escaped_blocks() {
        let input = "This is {{literal}} but this is {captured}";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["captured"]);
    }

    #[test]
    fn unclosed_template_at_end() {
        let input = "This is {started but never finished";
        let lexer = L::new(input);

        assert!(lexer.templates.is_empty());
    }

    #[test]
    fn unclosed_template_followed_by_valid_one() {
        let input = "start {broken {valid}";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["broken {valid"]);
    }

    #[test]
    fn different_delimiters() {
        type SquareL = Lexer<'[', ']'>;
        let input = "Hello [name], how are [you]?";
        let lexer = SquareL::new(input);

        assert_eq!(lexer.templates, vec!["name", "you"]);
    }

    #[test]
    fn whitespace_inside_template() {
        let input = "{  spaced  }";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["  spaced  "]);
    }

    #[test]
    fn complex_directive_syntax() {
        let input = "I love {dish->capitalize}!";
        let lexer = L::new(input);

        assert_eq!(lexer.templates, vec!["dish->capitalize"]);
    }
}
