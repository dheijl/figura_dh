pub trait Delimiter {
    fn open() -> char;
    fn closed() -> char;
}

pub struct Parentheses;

impl Delimiter for Parentheses {
    fn open() -> char {
        '('
    }

    fn closed() -> char {
        ')'
    }
}

pub struct SquareBrackets;

impl Delimiter for SquareBrackets {
    fn open() -> char {
        '['
    }

    fn closed() -> char {
        ']'
    }
}

pub struct CurlyBrackets;

impl Delimiter for CurlyBrackets {
    fn open() -> char {
        '{'
    }

    fn closed() -> char {
        '}'
    }
}

pub struct AngleBrackets;

impl Delimiter for AngleBrackets {
    fn open() -> char {
        '<'
    }

    fn closed() -> char {
        '>'
    }
}
