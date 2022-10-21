use std::{borrow::Cow, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'v> {
    String(Cow<'v, str>),
    Num(f64),
    Bool(bool),
    Nil,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    String,
    Num,
    Bool,
    Nil,
}

impl<'v> Value<'v> {
    pub fn string(s: impl Into<Cow<'v, str>>) -> Self {
        Self::String(s.into())
    }

    pub fn ty(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Num(_) => Type::Num,
            Value::Bool(_) => Type::Bool,
            Value::Nil => Type::Nil,
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Num(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "string"),
            Type::Num => write!(f, "number"),
            Type::Bool => write!(f, "bool"),
            Type::Nil => write!(f, "nil"),
        }
    }
}
