use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableKind {
    String(String),
    Int(String),
    UInt(String),
    Bool(String),
    Literal(String),
}

impl Display for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableKind::String(_) => write!(f, "string"),
            VariableKind::Int(_) => write!(f, "int"),
            VariableKind::UInt(_) => write!(f, "uint"),
            VariableKind::Bool(_) => write!(f, "bool"),
            VariableKind::Literal(_) => write!(f, "literal"),
        }
    }
}

pub trait VariableNames {
    const STRINGS: &[&str] = &[];
    const INTS: &[&str] = &[];
    const UINTS: &[&str] = &[];
    const BOOLEANS: &[&str] = &[];

    fn kind(name: &str) -> VariableKind {
        let owned = name.to_owned();

        if Self::INTS.contains(&name) {
            return VariableKind::Int(owned);
        }

        if Self::UINTS.contains(&name) {
            return VariableKind::UInt(owned);
        }

        if Self::BOOLEANS.contains(&name) {
            return VariableKind::Bool(owned);
        }

        if Self::STRINGS.contains(&name) {
            return VariableKind::String(owned);
        }

        // Not a variable name, treat as literal
        VariableKind::Literal(owned)
    }
}

pub struct EmtpyVariables;

impl VariableNames for EmtpyVariables {}
