use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'a> {
    Ident(&'a str),
    Assign,

    Int(&'a str),
    Float(&'a str),

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
    /// Cow because when the string has escape sequences '\n', etc
    /// A Heap allocated string is needed
    Literal(Cow<'a, str>),

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

    fn read_literal(&mut self) -> Cow<'a, str> {
        let start = self.cursor;
        let mut tmp_cursor = start;
        let mut escaped = false;

        while tmp_cursor < self.bytes.len() {
            match self.bytes[tmp_cursor] {
                b'"' => break,
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
                b'"' => {
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
                            b'"' => out.push('"'),
                            b'0' => out.push('\0'),
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
            b'*' => Some(Token::Star),
            b'/' => Some(Token::Slash),
            b'=' => Some(self.check_double(b'=', Token::Equals, Token::Assign)),
            b'!' => Some(self.check_double(b'=', Token::NotEquals, Token::Not)),
            b'<' => Some(self.check_double(b'=', Token::LessThanEquals, Token::LessThan)),
            b'>' => Some(self.check_double(b'=', Token::GreaterThanEquals, Token::GreaterThan)),
            b'&' => Some(self.check_double(b'&', Token::And, Token::Unknown('&'))),
            b'|' => Some(self.check_double(b'|', Token::Or, Token::Pipe)),
            b'"' => Some(Token::Literal(self.read_literal())),
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
