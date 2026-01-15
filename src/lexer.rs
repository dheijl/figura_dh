use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'a> {
    /// An identifier (variable name).
    ///
    /// Identifiers start with a letter or underscore and can contain
    /// letters, digits, and underscores.
    ///
    /// Examples: `name`, `user_id`, `_temp`
    Ident(&'a str),

    /// Assignment operator `=`.
    Assign,

    /// An integer literal.
    ///
    /// Contains the string representation of the integer for parsing.
    /// Examples: `"42"`, `"0"`, `"1000"`
    Int(&'a str),

    /// A floating-point literal.
    ///
    /// Contains the string representation of the float for parsing.
    /// Must contain a decimal point followed by digits.
    /// Examples: `"3.14"`, `"0.5"`
    Float(&'a str),

    /// Left parenthesis `(`.
    LParen,
    /// Right parenthesis `)`.
    RParen,
    /// Left square bracket `[`.
    LSquare,
    /// Right square bracket `]`.
    RSquare,
    /// Left curly brace `{`.
    LCurly,
    /// Right curly brace `}`.
    RCurly,
    /// Colon `:`.
    Colon,
    /// Semicolon `;`.
    Semicolon,

    /// Question mark `?` (used in ternary conditionals).
    Question,
    /// Pipe `|` (single pipe, not logical OR).
    Pipe,
    /// Underscore `_`.
    Underscore,

    /// A string literal enclosed in quotes.
    ///
    /// String literals can be enclosed in either single or double quotes
    /// and support escape sequences like `\n`, `\t`, `\\`, etc.
    ///
    /// The `Cow` allows for zero-copy when there are no escape sequences,
    /// but allocates when escape processing is needed.
    ///
    /// Examples: `"hello"`, `'world'`, `"line\nbreak"`
    Literal(Cow<'a, str>),

    // Unary and binary operations
    /// Logical NOT `!`.
    Not,
    /// Addition `+`.
    Plus,
    /// Subtraction/negation `-`.
    Minus,
    /// Multiplication `*`.
    Star,
    /// Division `/`.
    Slash,

    // Comparison operators
    /// Equality `==`.
    Equals,
    /// Inequality `!=`.
    NotEquals,
    /// Greater than `>`.
    GreaterThan,
    /// Greater than or equal `>=`.
    GreaterThanEquals,
    /// Less than `<`.
    LessThan,
    /// Less than or equal `<=`.
    LessThanEquals,
    /// Logical AND `&&`.
    And,
    /// Logical OR `||`.
    Or,

    /// An unknown/unexpected character.
    ///
    /// Used when the lexer encounters a character it doesn't recognize.
    /// This allows the parser to handle the error gracefully.
    Unknown(char),
}

pub struct TemplateLexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> TemplateLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            cursor: 0,
        }
    }

    #[inline]
    fn current(&self) -> u8 {
        if self.cursor < self.bytes.len() {
            self.bytes[self.cursor]
        } else {
            0
        }
    }

    #[inline]
    fn peek(&mut self) -> u8 {
        if self.cursor + 1 < self.bytes.len() {
            self.bytes[self.cursor + 1]
        } else {
            0
        }
    }

    #[inline]
    fn advance(&mut self) {
        if self.cursor < self.bytes.len() {
            self.cursor += 1;
        }
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.cursor < self.bytes.len() && self.current().is_ascii_whitespace() {
            self.cursor += 1;
        }
    }

    fn read_literal(&mut self, del: char) -> Cow<'a, str> {
        let del = del as u8;
        let start = self.cursor;
        let mut tmp_cursor = start;
        let mut escaped = false;

        while tmp_cursor < self.bytes.len() {
            match self.bytes[tmp_cursor] {
                c if c == del => break,
                b'\\' => {
                    escaped = true;
                    tmp_cursor += 2;
                }
                _ => tmp_cursor += 1,
            }
        }

        if !escaped {
            let len = tmp_cursor - start;
            self.cursor = tmp_cursor + 1; // skip closing quote

            return Cow::Borrowed(&self.input[start..start + len]);
        }

        let mut out = String::with_capacity(tmp_cursor - start);

        while self.cursor < self.bytes.len() {
            match self.current() {
                c if c == del => {
                    self.advance();
                    break;
                }
                b'\\' => {
                    self.advance();

                    if self.cursor < self.bytes.len() {
                        match self.current() {
                            b'n' => out.push('\n'),
                            b't' => out.push('\t'),
                            b'r' => out.push('\r'),
                            b'\\' => out.push('\\'),
                            b'0' => out.push('\0'),
                            c if c == del => out.push(del as char),
                            c => out.push(c as char),
                        }

                        self.advance();
                    }
                }
                c => {
                    out.push(c as char);
                    self.advance();
                }
            }
        }

        Cow::Owned(out)
    }

    fn read_ident(&mut self, start: usize) -> &'a str {
        while self.cursor < self.bytes.len() {
            let b = self.bytes[self.cursor];

            if b.is_ascii_alphabetic() || b.is_ascii_digit() || b == b'_' {
                self.cursor += 1;
            } else {
                break;
            }
        }

        &self.input[start..self.cursor]
    }

    fn read_number(&mut self, start: usize) -> Token<'a> {
        let mut is_float = false;

        while self.cursor < self.bytes.len() && self.current().is_ascii_digit() {
            self.cursor += 1;
        }

        if self.current() == b'.' && self.peek().is_ascii_digit() {
            is_float = true;
            self.cursor += 1; // skip '.'

            while self.cursor < self.bytes.len() && self.current().is_ascii_digit() {
                self.cursor += 1;
            }
        }

        let slice = &self.input[start..self.cursor];

        if is_float {
            Token::Float(slice)
        } else {
            Token::Int(slice)
        }
    }

    #[inline]
    fn check_double(
        &mut self,
        expected: u8,
        matched: Token<'a>,
        unmatched: Token<'a>,
    ) -> Token<'a> {
        if self.current() == expected {
            self.advance();
            matched
        } else {
            unmatched
        }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();

        if self.cursor >= self.bytes.len() {
            return None;
        }

        let start = self.cursor;
        let ch = self.current();
        self.advance();

        match ch {
            b'(' => Some(Token::LParen),
            b')' => Some(Token::RParen),
            b'[' => Some(Token::LSquare),
            b']' => Some(Token::RSquare),
            b'{' => Some(Token::LCurly),
            b'}' => Some(Token::RCurly),
            b':' => Some(Token::Colon),
            b';' => Some(Token::Semicolon),
            b'?' => Some(Token::Question),
            b'+' => Some(Token::Plus),
            b'-' => Some(Token::Minus),
            b'*' => Some(Token::Star),
            b'/' => Some(Token::Slash),
            b'=' => Some(self.check_double(b'=', Token::Equals, Token::Assign)),
            b'!' => Some(self.check_double(b'=', Token::NotEquals, Token::Not)),
            b'<' => Some(self.check_double(b'=', Token::LessThanEquals, Token::LessThan)),
            b'>' => Some(self.check_double(b'=', Token::GreaterThanEquals, Token::GreaterThan)),
            b'&' => Some(self.check_double(b'&', Token::And, Token::Unknown('&'))),
            b'|' => Some(self.check_double(b'|', Token::Or, Token::Pipe)),
            b'"' => Some(Token::Literal(self.read_literal('"'))),
            b'\'' => Some(Token::Literal(self.read_literal('\''))),
            b'_' => {
                let next = self.current();
                if next.is_ascii_alphabetic() || next.is_ascii_digit() || next == b'_' {
                    Some(Token::Ident(self.read_ident(start)))
                } else {
                    Some(Token::Underscore)
                }
            }
            b if b.is_ascii_alphabetic() => Some(Token::Ident(self.read_ident(start))),
            b if b.is_ascii_digit() => Some(self.read_number(start)),
            b => Some(Token::Unknown(b as char)),
        }
    }
}

impl<'a> Iterator for TemplateLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
