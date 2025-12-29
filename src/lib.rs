#![warn(clippy::use_self)]

mod lexer;
mod parser;

use std::{any::Any, collections::HashMap, fmt};

// A Value type used in templating contexts.
#[derive(Debug)]
pub enum Value {
    /// Heap-allocated string.
    String(String),

    /// Static string slice.
    ///
    /// E.G "Hello, world!" typed directly into the template.
    Str(&'static str),

    /// 64-bit integer.
    Int(i64),

    /// 64-bit floating point number.
    Float(f64),

    /// Custom value.
    /// Must implement `Debug`
    Custom(Box<dyn Any>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Custom(v) => write!(f, "{:?}", v),
        }
    }
}

pub type Context = HashMap<&'static str, Value>;
